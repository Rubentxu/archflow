// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Engine - EntityStore with Structure of Arrays (SoA)
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 4
//
// EntityStore implements Data-Oriented Design principles:
// - Structure of Arrays (SoA) for cache efficiency
// - Bit-packing for metadata to reduce memory footprint
// - Transform hierarchy for grouping/frames
// - Dirty tracking for selective updates
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use archflow_core::{EntityId, Generation, Index, Vec2};
use fixedbitset::FixedBitSet;
use heapless::Vec as HeaplessVec;

use crate::command::Command;

// Tracing support (conditionally compiled)
#[cfg(feature = "tracing")]
use tracing::{debug, error, info, trace, warn};

/// Maximum number of entities supported in the store
pub const MAX_ENTITIES: usize = 100_000;

/// Maximum number of glyphs across all text entities
pub const MAX_GLYPHS: usize = 500_000;

/// Maximum number of connections between entities
pub const MAX_CONNECTIONS: usize = 200_000;

/// Maximum total characters in string pool
pub const MAX_TEXT_LENGTH: usize = 50_000;

/// Shape types that can be rendered
/// Values 0-15 fit in 4 bits of metadata
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapeType {
    /// Standard rectangle shape
    Rectangle = 0,
    /// Circle/ellipse shape
    Circle = 1,
    /// Ellipse shape (different aspect ratio)
    Ellipse = 2,
    /// Straight line
    Line = 3,
    /// Triangle shape
    Triangle = 4,
    /// Diamond/rhombus shape
    Diamond = 5,
    /// Cylinder shape (typically for databases)
    Cylinder = 6,
    /// Person/user actor icon
    Person = 7,
    /// Rectangle with rounded corners
    RoundedRect = 8,
    /// Rectangle with dashed border
    DashedRect = 9,
}

/// Architecture metadata for C4 model entities
/// Stored separately as "cold data" - only accessed on selection/inspection
#[derive(Clone, Debug)]
pub struct ArchitectureData {
    /// Name of the entity
    pub name: String,
    /// C4 level (0=System, 1=Container, 2=Component, 3=Code)
    pub c4_level: u8,
    /// Entity type identifier
    pub entity_type: u8,
    /// Cloud provider (0=None, 1=AWS, 2=GCP, 3=Azure)
    pub cloud_provider: u8,
    /// Technology stack description
    pub technology: String,
    /// Detailed description of the entity
    pub description: String,
}

/// String Pool for zero-allocation string storage
///
/// Problem with Vec<String>:
/// - 10,000 entities = 10,000 heap allocations
/// - Each String has 24 bytes overhead + capacity
/// - Cache misses when iterating (scattered memory)
///
/// String Pool Solution:
/// - Single Vec<u8> containing all concatenated strings
/// - Offset table (start, len) per EntityId
pub struct StringPool {
    buffer: Vec<u8>,
    offsets: Vec<(usize, usize)>,
    free_list: Vec<usize>,
}

impl StringPool {
    /// Create a new StringPool with the given capacity.
    ///
    /// # Arguments
    /// * `entities` - Maximum number of entities to store strings for
    /// * `total_chars` - Total character capacity for the buffer
    #[must_use]
    pub fn with_capacity(entities: usize, total_chars: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(total_chars),
            offsets: vec![(0, 0); entities],
            free_list: Vec::new(),
        }
    }

    /// Set the text for an entity.
    ///
    /// # Arguments
    /// * `entity_idx` - Index of the entity
    /// * `text` - Text content to store
    pub fn set(&mut self, entity_idx: usize, text: &str) {
        let bytes = text.as_bytes();
        let start = self.buffer.len();
        self.buffer.extend_from_slice(bytes);
        self.offsets[entity_idx] = (start, bytes.len());
    }

    /// Get the text for an entity.
    ///
    /// # Arguments
    /// * `entity_idx` - Index of the entity
    ///
    /// # Returns
    /// The text content
    #[inline(always)]
    #[must_use]
    pub fn get(&self, entity_idx: usize) -> &str {
        let (start, len) = self.offsets[entity_idx];
        unsafe { core::str::from_utf8_unchecked(&self.buffer[start..start + len]) }
    }

    /// Clear all strings from the pool.
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.offsets.fill((0, 0));
        self.free_list.clear();
    }

    // ═══════════════════════════════════════════════════════════
    // SERIALIZATION HELPERS
    // ═══════════════════════════════════════════════════════════

    /// Get the buffer (for serialization)
    #[inline(always)]
    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    /// Set the buffer directly (for deserialization)
    #[inline(always)]
    pub fn set_buffer(&mut self, data: &[u8]) {
        self.buffer.clear();
        self.buffer.extend_from_slice(data);
    }

    /// Clear the offsets (for deserialization)
    #[inline(always)]
    pub fn clear_offsets(&mut self) {
        self.offsets.fill((0, 0));
    }
}

/// EntityStore with Structure of Arrays (SoA) layout
///
/// Optimizations applied:
/// - 64-byte alignment for SIMD read efficiency
/// - Bit-packing of metadata in u32 (saves ~12 bytes per entity vs separate structs)
/// - Transform hierarchy for supporting groups/frames without performance penalty
///
/// Memory Layout:
/// - HOT DATA (Cache Lines 0-2): Accessed every frame by renderer
/// - TRANSFORM HIERARCHY: Parent-child relationships
/// - COLD DATA: Accessed only on selection/inspection
pub struct EntityStore {
    // ═══════════════════════════════════════════════════════════
    // HOT DATA (Cache Line 0-2): Accessed every frame by render
    // ═══════════════════════════════════════════════════════════
    /// Transforms: [x, y, w, h] - 16 bytes per entity
    pub transforms: Vec<[f32; 4]>,

    /// Metadata packed in u32 to save memory
    /// Layout: [shape:4 | layer:4 | visibility:1 | selected:1 | locked:1 | padding:21]
    pub metadata: Vec<u32>,

    /// Colors packed as 0xRRGGBBAA
    pub colors: Vec<u32>,

    /// Texture indices (0 = solid color, 1..N = atlas index)
    pub texture_index: Vec<u16>,

    /// UV rectangles in texture atlas [u, v, w, h]
    pub uv_rects: Vec<[f32; 4]>,

    /// Color tints for visual feedback (RGBA)
    pub color_tints: Vec<[f32; 4]>,

    /// Text glyph indices into global glyph buffer
    pub text_glyph_start: Vec<u32>,

    /// Number of glyphs per text entity
    pub text_glyph_count: Vec<u16>,

    /// Font scale for MSDF text rendering
    pub text_scale: Vec<f32>,

    // ═══════════════════════════════════════════════════════════
    // TRANSFORM HIERARCHY (V3.0)
    // ═══════════════════════════════════════════════════════════
    /// Parent entity for grouping/frames
    pub parent_id: Vec<Option<EntityId>>,

    /// Transform relative to parent
    pub local_transform: Vec<[f32; 4]>,

    /// Cached world-space transform (actual render position)
    pub world_transform: Vec<[f32; 4]>,

    /// Marked when parent moves, needs propagation to children
    pub dirty_hierarchy: FixedBitSet,

    // ═══════════════════════════════════════════════════════════
    // COLD DATA (Access only on selection/inspection)
    // ═══════════════════════════════════════════════════════════
    /// Architecture metadata for C4 entities
    pub arch_data: Vec<Option<Box<ArchitectureData>>>,

    /// String pool for entity names and labels
    pub string_pool: StringPool,

    // ═══════════════════════════════════════════════════════════
    // MANAGEMENT (Infrastructure)
    // ═══════════════════════════════════════════════════════════
    /// Generation counter for EntityId validation
    generations: Vec<u8>,

    /// LIFO stack of free indices
    free_list: Vec<u32>,

    /// Number of alive entities
    alive_count: usize,

    /// Dirty tracking for SpatialHash updates
    pub dirty_transform: FixedBitSet,

    /// Dirty tracking for GPU upload
    pub dirty_render: FixedBitSet,

    /// Dirty tracking for text layout recalculation
    pub dirty_text: FixedBitSet,

    /// Z-order render list [idx0, idx1, ...]
    pub draw_order: Vec<u32>,

    /// Marked when draw order changes
    pub dirty_z_order: bool,

    /// Command queue pre-allocated, reused
    pub command_queue: HeaplessVec<Command, 1024>,
}

impl EntityStore {
    /// Create a new EntityStore with pre-allocated capacity
    pub fn new() -> Self {
        let capacity = MAX_ENTITIES;

        Self {
            // Hot data
            transforms: vec![[0.0, 0.0, 100.0, 60.0]; capacity],
            metadata: vec![0; capacity],
            colors: vec![0xFFCCDDEE; capacity], // Default light blue
            texture_index: vec![0; capacity],
            uv_rects: vec![[0.0, 0.0, 1.0, 1.0]; capacity],
            color_tints: vec![[1.0, 1.0, 1.0, 1.0]; capacity],
            text_glyph_start: vec![0; capacity],
            text_glyph_count: vec![0; capacity],
            text_scale: vec![16.0; capacity],

            // Transform hierarchy
            parent_id: vec![None; capacity],
            local_transform: vec![[0.0, 0.0, 100.0, 60.0]; capacity],
            world_transform: vec![[0.0, 0.0, 100.0, 60.0]; capacity],
            dirty_hierarchy: FixedBitSet::with_capacity(capacity),

            // Cold data
            arch_data: vec![None; capacity],
            string_pool: StringPool::with_capacity(capacity, MAX_TEXT_LENGTH),

            // Management
            generations: vec![0; capacity],
            free_list: Vec::with_capacity(128),
            alive_count: 0,
            dirty_transform: FixedBitSet::with_capacity(capacity),
            dirty_render: FixedBitSet::with_capacity(capacity),
            dirty_text: FixedBitSet::with_capacity(capacity),
            draw_order: Vec::with_capacity(capacity),
            dirty_z_order: false,
            command_queue: HeaplessVec::new(),
        }
    }

    /// Spawn a new entity at the given position with size
    /// Returns the EntityId with generation counter
    pub fn spawn(&mut self, pos: Vec2, size: Vec2) -> EntityId {
        #[cfg(feature = "tracing")]
        debug!(
            target: "archflow::engine::store",
            pos = ?pos,
            size = ?size,
            alive_count = self.alive_count,
            "Entity spawn requested"
        );

        let index = if let Some(idx) = self.free_list.pop() {
            let idx_usize = idx as usize;
            #[cfg(feature = "tracing")]
            trace!(target: "archflow::engine::store", reuse_index = idx_usize, "Reusing free slot");

            idx_usize
        } else {
            if self.alive_count >= MAX_ENTITIES {
                #[cfg(feature = "tracing")]
                error!(
                    target: "archflow::engine::store",
                    alive_count = self.alive_count,
                    max = MAX_ENTITIES,
                    "EntityStore at maximum capacity"
                );
                panic!("EntityStore at maximum capacity (MAX_ENTITIES)");
            }
            self.alive_count
        };

        let generation = self.generations[index];
        let id = EntityId::from_parts(Index(index as u32), Generation(generation));

        #[cfg(feature = "tracing")]
        trace!(
            target: "archflow::engine::store",
            entity_id = ?id,
            index,
            generation = generation,
            "Generated new EntityId"
        );

        // Initialize transform
        self.transforms[index] = [pos.x, pos.y, size.x, size.y];
        self.world_transform[index] = [pos.x, pos.y, size.x, size.y];
        self.local_transform[index] = [pos.x, pos.y, size.x, size.y];

        // Initialize metadata (Rectangle shape=1, layer=0, visible=true, not selected, not locked)
        // Layout: [shape:4 | layer:4 | visibility:1 | selected:1 | locked:1 | padding:21]
        // shape=1 (Rectangle) in bits 0-3, visible=true in bit 8
        self.metadata[index] = 0x0101; // shape=1 (bits 0-3), visible=true (bit 8)

        // Don't mark dirty on spawn - entities are clean when first created
        // The renderer will pick up new entities during sync_from_store
        // Mark transform dirty for hierarchy updates if needed
        self.dirty_transform.insert(index);
        self.dirty_z_order = true;

        // Add to draw order
        self.draw_order.push(index as u32);

        self.alive_count += 1;

        #[cfg(feature = "tracing")]
        {
            let usage_percent = (self.alive_count as f32 / MAX_ENTITIES as f32) * 100.0;
            if usage_percent > 90.0 {
                warn!(
                    target: "archflow::engine::store",
                    alive_count = self.alive_count,
                    usage_pct = usage_percent,
                    "EntityStore approaching capacity"
                );
            }
            info!(
                target: "archflow::engine::store",
                entity_id = ?id,
                total_entities = self.alive_count,
                "Entity spawned successfully"
            );
        }

        id
    }

    /// Despawn an entity, marking its slot as free
    pub fn despawn(&mut self, id: EntityId) -> bool {
        #[cfg(feature = "tracing")]
        debug!(target: "archflow::engine::store", entity_id = ?id, "Entity despawn requested");

        let index = id.index().0 as usize;

        if index >= MAX_ENTITIES || self.generations[index] != id.generation().0 {
            #[cfg(feature = "tracing")]
            warn!(
                target: "archflow::engine::store",
                entity_id = ?id,
                index,
                "Despawn failed: invalid or stale EntityId"
            );
            return false; // Invalid or stale EntityId
        }

        #[cfg(feature = "tracing")]
        trace!(
            target: "archflow::engine::store",
            index,
            old_generation = self.generations[index],
            "Invalidating EntityId"
        );

        // Increment generation to invalidate existing EntityIds
        self.generations[index] = self.generations[index].wrapping_add(1);

        // Add to free list
        self.free_list.push(index as u32);

        // Remove from draw order
        self.draw_order.retain(|&idx| idx as usize != index);
        self.dirty_z_order = true;

        // Clear dirty flags
        self.dirty_transform.remove(index);
        self.dirty_render.remove(index);
        self.dirty_text.remove(index);
        self.dirty_hierarchy.remove(index);

        self.alive_count -= 1;

        #[cfg(feature = "tracing")]
        info!(
            target: "archflow::engine::store",
            entity_id = ?id,
            total_entities = self.alive_count,
            "Entity despawned successfully"
        );

        true
    }

    /// Check if an EntityId is valid (alive)
    #[inline(always)]
    pub fn is_alive(&self, id: EntityId) -> bool {
        let index = id.index().0 as usize;
        index < MAX_ENTITIES && self.generations[index] == id.generation().0
    }

    /// Get the shape type from metadata bitfield
    #[inline(always)]
    pub fn shape_type(&self, idx: usize) -> u8 {
        (self.metadata[idx] & 0xF) as u8
    }

    /// Set the shape type (basic shapes: 0=Rect, 1=Circle, etc.)
    #[inline(always)]
    pub fn set_shape_type(&mut self, idx: usize, shape: u8) {
        self.metadata[idx] = (self.metadata[idx] & !0xF) | (shape as u32 & 0xF);
        self.dirty_render.insert(idx);
    }

    /// Check if entity is visible
    #[inline(always)]
    pub fn is_visible(&self, idx: usize) -> bool {
        (self.metadata[idx] & (1 << 8)) != 0
    }

    /// Set visibility (bit 8)
    #[inline(always)]
    pub fn set_visible(&mut self, idx: usize, visible: bool) {
        if visible {
            self.metadata[idx] |= 1 << 8;
        } else {
            self.metadata[idx] &= !(1 << 8);
        }
        self.dirty_render.insert(idx);
    }

    /// Check if entity is selected
    #[inline(always)]
    pub fn is_selected(&self, idx: usize) -> bool {
        (self.metadata[idx] & (1 << 9)) != 0
    }

    /// Set selection (bit 9) - also updates color_tint for visual feedback
    #[inline(always)]
    pub fn set_selected(&mut self, idx: usize, selected: bool) {
        if selected {
            self.metadata[idx] |= 1 << 9;
            // Visual feedback: bluish tint for selection
            self.color_tints[idx] = [0.3, 0.5, 1.0, 0.3];
        } else {
            self.metadata[idx] &= !(1 << 9);
            // Restore normal color
            self.color_tints[idx] = [1.0, 1.0, 1.0, 1.0];
        }
        self.dirty_render.insert(idx);
    }

    /// Get the layer for z-index (bits 4-7)
    #[inline(always)]
    pub fn layer(&self, idx: usize) -> u8 {
        ((self.metadata[idx] >> 4) & 0xF) as u8
    }

    /// Set the layer (bits 4-7)
    #[inline(always)]
    pub fn set_layer(&mut self, idx: usize, layer: u8) {
        self.metadata[idx] = (self.metadata[idx] & !(0xF << 4)) | ((layer as u32 & 0xF) << 4);
        self.dirty_z_order = true;
    }

    /// Get entity position as Vec2
    #[inline(always)]
    pub fn pos(&self, idx: usize) -> Vec2 {
        Vec2::new(self.transforms[idx][0], self.transforms[idx][1])
    }

    /// Get entity size as Vec2
    #[inline(always)]
    pub fn size(&self, idx: usize) -> Vec2 {
        Vec2::new(self.transforms[idx][2], self.transforms[idx][3])
    }

    /// Move entity by delta
    pub fn move_by(&mut self, idx: usize, delta: Vec2) {
        self.transforms[idx][0] += delta.x;
        self.transforms[idx][1] += delta.y;
        self.dirty_transform.insert(idx);
        self.dirty_render.insert(idx);
    }

    /// Set entity position
    pub fn set_pos(&mut self, idx: usize, pos: Vec2) {
        self.transforms[idx][0] = pos.x;
        self.transforms[idx][1] = pos.y;
        self.dirty_transform.insert(idx);
        self.dirty_render.insert(idx);
    }

    /// Set entity size
    pub fn set_size(&mut self, idx: usize, size: Vec2) {
        self.transforms[idx][2] = size.x;
        self.transforms[idx][3] = size.y;
        self.dirty_transform.insert(idx);
        self.dirty_render.insert(idx);
    }

    /// Get world transform (after hierarchy propagation)
    #[inline(always)]
    pub fn world_pos(&self, idx: usize) -> Vec2 {
        Vec2::new(self.world_transform[idx][0], self.world_transform[idx][1])
    }

    /// Get world size
    #[inline(always)]
    pub fn world_size(&self, idx: usize) -> Vec2 {
        Vec2::new(self.world_transform[idx][2], self.world_transform[idx][3])
    }

    /// Set parent for transform hierarchy
    pub fn set_parent(&mut self, idx: usize, parent: Option<EntityId>) {
        self.parent_id[idx] = parent;
        self.dirty_hierarchy.insert(idx);
    }

    /// Update world transforms for hierarchy
    /// Call this after moving parent entities
    pub fn update_hierarchy(&mut self) {
        // Find all entities with dirty hierarchy
        // Propagate transforms from parents to children
        for i in 0..MAX_ENTITIES {
            if !self.dirty_hierarchy.contains(i) {
                continue;
            }

            if let Some(parent_id) = self.parent_id[i] {
                let parent_idx = parent_id.index().0 as usize;
                if parent_idx < MAX_ENTITIES {
                    // Child world = parent world + child local
                    self.world_transform[i][0] =
                        self.world_transform[parent_idx][0] + self.local_transform[i][0];
                    self.world_transform[i][1] =
                        self.world_transform[parent_idx][1] + self.local_transform[i][1];
                }
            } else {
                // No parent, world = local
                self.world_transform[i] = self.transforms[i];
            }

            self.dirty_render.insert(i);
        }

        self.dirty_hierarchy.clear();
    }

    /// Clear all dirty flags
    pub fn clear_dirty_flags(&mut self) {
        self.dirty_transform.clear();
        self.dirty_render.clear();
        self.dirty_text.clear();
        self.dirty_hierarchy.clear();
        self.dirty_z_order = false;
    }

    // ═══════════════════════════════════════════════════════════
    // DIRTY TRACKING API (for Renderer - HU-RENDER-007)
    // ═══════════════════════════════════════════════════════════

    /// Mark an entity as needing GPU update
    #[inline(always)]
    pub fn mark_render_dirty(&mut self, idx: usize) {
        self.dirty_render.insert(idx);
    }

    /// Mark an entity as needing transform update
    #[inline(always)]
    pub fn mark_transform_dirty(&mut self, idx: usize) {
        self.dirty_transform.insert(idx);
    }

    /// Check if entity needs GPU update
    #[inline(always)]
    pub fn is_render_dirty(&self, idx: usize) -> bool {
        self.dirty_render.contains(idx)
    }

    /// Take all dirty entities and clear the dirty flag
    ///
    /// Returns an iterator over indices that need GPU update.
    /// This is more efficient than checking each entity individually.
    #[inline(always)]
    pub fn take_dirty_render_entities(&mut self) -> impl Iterator<Item = usize> + '_ {
        // Collect dirty indices and clear the set
        // FixedBitSet::ones() returns an iterator of set bit indices
        let dirty: Vec<usize> = self.dirty_render.ones().collect();
        self.dirty_render.clear();
        dirty.into_iter()
    }

    /// Get count of dirty entities (for performance monitoring)
    #[inline(always)]
    pub fn dirty_render_count(&self) -> usize {
        self.dirty_render.count_ones(..)
    }

    /// Check if any render data is dirty (quick check before full sync)
    #[inline(always)]
    pub fn has_render_dirty(&self) -> bool {
        self.dirty_render.count_ones(..) > 0
    }

    /// Get number of alive entities
    #[inline(always)]
    pub fn alive_count(&self) -> usize {
        self.alive_count
    }

    /// Check if store is empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.alive_count == 0
    }

    // ═══════════════════════════════════════════════════════════
    // SERIALIZATION HELPERS
    // ═══════════════════════════════════════════════════════════

    /// Check if an entity index is alive (for serialization)
    #[inline(always)]
    pub fn is_alive_index(&self, idx: usize) -> bool {
        if idx >= MAX_ENTITIES {
            return false;
        }
        // Check if not in free list (an entity is alive if it's in draw_order and not in free_list)
        if self
            .free_list
            .iter()
            .any(|&free_idx| free_idx as usize == idx)
        {
            return false;
        }
        // Check if it's in the draw order (has been spawned)
        self.draw_order.contains(&(idx as u32))
    }

    /// Get generation for an entity index (for serialization)
    #[inline(always)]
    pub fn generation(&self, idx: usize) -> u8 {
        self.generations[idx]
    }

    /// Set generation for an entity index (for deserialization)
    #[inline(always)]
    pub fn set_generation(&mut self, idx: usize, gen_val: u8) {
        self.generations[idx] = gen_val;
    }

    /// Set alive count directly (for deserialization)
    #[inline(always)]
    pub fn set_alive_count(&mut self, count: usize) {
        self.alive_count = count;
    }

    /// Get a reference to transforms array (for serialization)
    #[inline(always)]
    pub fn transforms_ref(&self) -> &[[f32; 4]] {
        &self.transforms
    }

    /// Get a reference to metadata array (for serialization)
    #[inline(always)]
    pub fn metadata_ref(&self) -> &[u32] {
        &self.metadata
    }

    /// Get a reference to colors array (for serialization)
    #[inline(always)]
    pub fn colors_ref(&self) -> &[u32] {
        &self.colors
    }

    /// Set transform directly (for deserialization)
    #[inline(always)]
    pub fn set_transform(&mut self, idx: usize, transform: [f32; 4]) {
        self.transforms[idx] = transform;
    }

    /// Set local transform directly (for deserialization)
    #[inline(always)]
    pub fn set_local_transform(&mut self, idx: usize, transform: [f32; 4]) {
        self.local_transform[idx] = transform;
    }

    /// Set world transform directly (for deserialization)
    #[inline(always)]
    pub fn set_world_transform(&mut self, idx: usize, transform: [f32; 4]) {
        self.world_transform[idx] = transform;
    }

    /// Set metadata directly (for deserialization)
    #[inline(always)]
    pub fn set_metadata(&mut self, idx: usize, metadata: u32) {
        self.metadata[idx] = metadata;
    }

    /// Set color directly (for deserialization)
    #[inline(always)]
    pub fn set_color(&mut self, idx: usize, color: u32) {
        self.colors[idx] = color;
    }

    /// Set texture index directly (for deserialization)
    #[inline(always)]
    pub fn set_texture_index(&mut self, idx: usize, index: u16) {
        self.texture_index[idx] = index;
    }

    /// Set text glyph count directly (for deserialization)
    #[inline(always)]
    pub fn set_text_glyph_count(&mut self, idx: usize, count: u16) {
        self.text_glyph_count[idx] = count;
    }

    /// Set text glyph start directly (for deserialization)
    #[inline(always)]
    pub fn set_text_glyph_start(&mut self, idx: usize, start: u32) {
        self.text_glyph_start[idx] = start;
    }

    /// Set text scale directly (for deserialization)
    #[inline(always)]
    pub fn set_text_scale(&mut self, idx: usize, scale: f32) {
        self.text_scale[idx] = scale;
    }

    /// Set UV rect directly (for deserialization)
    #[inline(always)]
    pub fn set_uv_rect(&mut self, idx: usize, uv_rect: [f32; 4]) {
        self.uv_rects[idx] = uv_rect;
    }

    /// Set color tint directly (for deserialization)
    #[inline(always)]
    pub fn set_color_tint(&mut self, idx: usize, tint: [f32; 4]) {
        self.color_tints[idx] = tint;
    }

    /// Get local transform array (for serialization)
    #[inline(always)]
    pub fn local_transforms_ref(&self) -> &[[f32; 4]] {
        &self.local_transform
    }

    /// Get world transform array (for serialization)
    #[inline(always)]
    pub fn world_transforms_ref(&self) -> &[[f32; 4]] {
        &self.world_transform
    }

    /// Get texture index array (for serialization)
    #[inline(always)]
    pub fn texture_indices_ref(&self) -> &[u16] {
        &self.texture_index
    }

    /// Get text glyph count array (for serialization)
    #[inline(always)]
    pub fn text_glyph_counts_ref(&self) -> &[u16] {
        &self.text_glyph_count
    }

    /// Get text glyph start array (for serialization)
    #[inline(always)]
    pub fn text_glyph_starts_ref(&self) -> &[u32] {
        &self.text_glyph_start
    }

    /// Get text scale array (for serialization)
    #[inline(always)]
    pub fn text_scales_ref(&self) -> &[f32] {
        &self.text_scale
    }

    /// Get UV rect array (for serialization)
    #[inline(always)]
    pub fn uv_rects_ref(&self) -> &[[f32; 4]] {
        &self.uv_rects
    }

    /// Get color tint array (for serialization)
    #[inline(always)]
    pub fn color_tints_ref(&self) -> &[[f32; 4]] {
        &self.color_tints
    }

    /// Get parent ID array (for serialization)
    #[inline(always)]
    pub fn parent_ids_ref(&self) -> &[Option<EntityId>] {
        &self.parent_id
    }

    /// Set parent ID directly (for deserialization)
    #[inline(always)]
    pub fn set_parent_id(&mut self, idx: usize, parent: Option<EntityId>) {
        self.parent_id[idx] = parent;
    }
}

impl Default for EntityStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_despawn() {
        let mut store = EntityStore::new();

        let id = store.spawn(Vec2::new(100.0, 200.0), Vec2::new(50.0, 30.0));
        assert!(store.is_alive(id));
        assert_eq!(store.alive_count(), 1);

        assert!(store.despawn(id));
        assert!(!store.is_alive(id));
        assert_eq!(store.alive_count(), 0);
    }

    #[test]
    fn test_metadata_bit_packing() {
        let mut store = EntityStore::new();
        let id = store.spawn(Vec2::ZERO, Vec2::new(100.0, 60.0));
        let idx = id.index().0 as usize;

        assert_eq!(store.shape_type(idx), 1); // Default Rectangle
        assert!(store.is_visible(idx));
        assert!(!store.is_selected(idx));

        store.set_shape_type(idx, 2);
        assert_eq!(store.shape_type(idx), 2);

        store.set_visible(idx, false);
        assert!(!store.is_visible(idx));

        store.set_selected(idx, true);
        assert!(store.is_selected(idx));
    }

    #[test]
    fn test_transform_operations() {
        let mut store = EntityStore::new();
        let id = store.spawn(Vec2::new(10.0, 20.0), Vec2::new(100.0, 50.0));
        let idx = id.index().0 as usize;

        assert_eq!(store.pos(idx), Vec2::new(10.0, 20.0));
        assert_eq!(store.size(idx), Vec2::new(100.0, 50.0));

        store.move_by(idx, Vec2::new(5.0, 10.0));
        assert_eq!(store.pos(idx), Vec2::new(15.0, 30.0));

        store.set_size(idx, Vec2::new(200.0, 100.0));
        assert_eq!(store.size(idx), Vec2::new(200.0, 100.0));
    }

    #[test]
    fn test_string_pool() {
        let mut pool = StringPool::with_capacity(10, 100);

        pool.set(0, "Hello");
        pool.set(1, "World");

        assert_eq!(pool.get(0), "Hello");
        assert_eq!(pool.get(1), "World");
    }

    // ════════════════════════════════════════════════════════════════════════
    // DIRTY TRACKING TESTS (HU-RENDER-007)
    // ════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_mark_render_dirty() {
        let mut store = EntityStore::new();
        let id = store.spawn(Vec2::new(10.0, 20.0), Vec2::new(100.0, 50.0));
        let idx = id.index().0 as usize;

        // Entity should not be dirty initially (spawn clears dirty)
        assert!(!store.is_render_dirty(idx));

        // Mark dirty
        store.mark_render_dirty(idx);
        assert!(store.is_render_dirty(idx));
    }

    #[test]
    fn test_take_dirty_render_entities() {
        let mut store = EntityStore::new();

        // Spawn multiple entities
        let id1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let id2 = store.spawn(Vec2::new(20.0, 20.0), Vec2::new(10.0, 10.0));
        let id3 = store.spawn(Vec2::new(40.0, 40.0), Vec2::new(10.0, 10.0));

        let idx1 = id1.index().0 as usize;
        let idx2 = id2.index().0 as usize;
        let idx3 = id3.index().0 as usize;

        // Mark some entities dirty
        store.mark_render_dirty(idx1);
        store.mark_render_dirty(idx3);

        // Verify before take
        assert_eq!(store.dirty_render_count(), 2);

        // Take dirty entities
        let dirty: Vec<usize> = store.take_dirty_render_entities().collect();

        assert_eq!(dirty.len(), 2);
        assert!(dirty.contains(&idx1));
        assert!(dirty.contains(&idx3));
        assert!(!dirty.contains(&idx2));

        // Dirty flag should be cleared
        assert_eq!(store.dirty_render_count(), 0);
        assert!(!store.has_render_dirty());
    }

    #[test]
    fn test_dirty_render_count() {
        let mut store = EntityStore::new();

        assert_eq!(store.dirty_render_count(), 0);

        let id = store.spawn(Vec2::ZERO, Vec2::new(10.0, 10.0));
        let idx = id.index().0 as usize;

        store.mark_render_dirty(idx);
        assert_eq!(store.dirty_render_count(), 1);

        // Spawn more and mark dirty
        for i in 0..5 {
            let e = store.spawn(Vec2::new(i as f32 * 10.0, 0.0), Vec2::new(10.0, 10.0));
            store.mark_render_dirty(e.index().0 as usize);
        }
        assert_eq!(store.dirty_render_count(), 6);
    }

    #[test]
    fn test_has_render_dirty() {
        let mut store = EntityStore::new();

        assert!(!store.has_render_dirty());

        let id = store.spawn(Vec2::ZERO, Vec2::new(10.0, 10.0));
        let idx = id.index().0 as usize;

        // After spawn, dirty flag should be cleared
        assert!(!store.has_render_dirty());

        store.mark_render_dirty(idx);
        assert!(store.has_render_dirty());

        // Clear dirty
        store.take_dirty_render_entities().for_each(|_| {});
        assert!(!store.has_render_dirty());
    }
}
