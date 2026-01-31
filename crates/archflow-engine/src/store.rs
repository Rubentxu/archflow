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
    Rectangle = 0,
    Circle = 1,
    Ellipse = 2,
    Line = 3,
    Triangle = 4,
    Diamond = 5,
    Cylinder = 6, // Database shape
    Person = 7,   // User/Actor icon
    RoundedRect = 8,
    DashedRect = 9,
}

/// Architecture metadata for C4 model entities
/// Stored separately as "cold data" - only accessed on selection/inspection
#[derive(Clone, Debug)]
pub struct ArchitectureData {
    pub name: String,
    pub c4_level: u8,
    pub entity_type: u8,
    pub cloud_provider: u8,
    pub technology: String,
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
    pub fn with_capacity(entities: usize, total_chars: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(total_chars),
            offsets: vec![(0, 0); entities],
            free_list: Vec::new(),
        }
    }

    pub fn set(&mut self, entity_idx: usize, text: &str) {
        let bytes = text.as_bytes();
        let start = self.buffer.len();
        self.buffer.extend_from_slice(bytes);
        self.offsets[entity_idx] = (start, bytes.len());
    }

    #[inline(always)]
    pub fn get(&self, entity_idx: usize) -> &str {
        let (start, len) = self.offsets[entity_idx];
        unsafe { core::str::from_utf8_unchecked(&self.buffer[start..start + len]) }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.offsets.fill((0, 0));
        self.free_list.clear();
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
        let index = if let Some(idx) = self.free_list.pop() {
            idx as usize
        } else {
            if self.alive_count >= MAX_ENTITIES {
                panic!("EntityStore at maximum capacity (MAX_ENTITIES)");
            }
            self.alive_count
        };

        let generation = self.generations[index];
        let id = EntityId::from_parts(Index(index as u32), Generation(generation));

        // Initialize transform
        self.transforms[index] = [pos.x, pos.y, size.x, size.y];
        self.world_transform[index] = [pos.x, pos.y, size.x, size.y];
        self.local_transform[index] = [pos.x, pos.y, size.x, size.y];

        // Initialize metadata (Rectangle shape=1, layer=0, visible=true, not selected, not locked)
        // Layout: [shape:4 | layer:4 | visibility:1 | selected:1 | locked:1 | padding:21]
        // shape=1 (Rectangle) in bits 0-3, visible=true in bit 8
        self.metadata[index] = 0x0101; // shape=1 (bits 0-3), visible=true (bit 8)

        // Mark dirty
        self.dirty_transform.insert(index);
        self.dirty_render.insert(index);
        self.dirty_z_order = true;

        // Add to draw order
        self.draw_order.push(index as u32);

        self.alive_count += 1;
        id
    }

    /// Despawn an entity, marking its slot as free
    pub fn despawn(&mut self, id: EntityId) -> bool {
        let index = id.index().0 as usize;

        if index >= MAX_ENTITIES || self.generations[index] != id.generation().0 {
            return false; // Invalid or stale EntityId
        }

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
}
