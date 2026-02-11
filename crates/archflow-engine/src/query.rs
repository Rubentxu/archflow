// ═══════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Engine - ECS Query Abstraction Layer
//
// Provides type-safe query abstraction over EntityStore.
// Enables clean access patterns without direct EntityStore coupling.
//
// Usage:
// let store = EntityStore::new();
// let results = query_visible(&store);
// for result in results {
//     // Process result
// }
// ═══════════════════════════════════════════════════════════════════════════════════════

use alloc::vec::Vec;

use archflow_core::{EntityId, Generation, Index};

use super::EntityStore;

// ═══════════════════════════════════════════════════════════════════════════════════════
// Query Results
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Basic query result with transform data
#[derive(Debug, Clone, Copy)]
pub struct QueryResult {
    /// Entity identifier
    pub entity: EntityId,
    /// Entity index in storage
    pub index: usize,
    /// X position
    pub x: f32,
    /// Y position
    pub y: f32,
    /// Width of bounding box
    pub width: f32,
    /// Height of bounding box
    pub height: f32,
    /// Z-order layer
    pub layer: u8,
    /// Whether the entity is visible
    pub is_visible: bool,
    /// Whether the entity is selected
    pub is_selected: bool,
    /// Whether the entity is locked
    pub is_locked: bool,
}

/// Full renderable entity with visual properties
#[derive(Debug)]
pub struct RenderableResult {
    /// Entity identifier
    pub entity: EntityId,
    /// Entity index in storage
    pub index: usize,
    /// X position
    pub x: f32,
    /// Y position
    pub y: f32,
    /// Width of bounding box
    pub width: f32,
    /// Height of bounding box
    pub height: f32,
    /// Z-order layer
    pub layer: u8,
    /// Fill color (ARGB format)
    pub fill_color: u32,
    /// Stroke color (ARGB format)
    pub stroke_color: u32,
    /// Stroke width in pixels
    pub stroke_width: f32,
    /// Texture index for GPU rendering
    pub texture_index: u16,
    /// UV coordinates for texture mapping [u_min, v_min, u_max, v_max]
    pub uv_rect: [f32; 4],
    /// Whether the entity is visible
    pub is_visible: bool,
    /// Whether the entity is selected
    pub is_selected: bool,
    /// Whether the entity is locked
    pub is_locked: bool,
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// Render Query (for GpuRenderer Integration)
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Query iterator optimized for GPU renderer
///
/// Provides efficient iteration over renderable entities with:
/// - Early exit on visibility culling
/// - Viewport culling support
/// - Phase-based categorization
#[derive(Copy, Clone)]
pub struct RenderQuery<'a> {
    store: &'a EntityStore,
    transform_view: TransformView<'a>,
    color_view: ColorView<'a>,
    metadata_view: MetadataView<'a>,
}

impl<'a> RenderQuery<'a> {
    /// Create a new render query
    #[inline(always)]
    #[must_use]
    pub fn new(store: &'a EntityStore) -> Self {
        Self {
            store,
            transform_view: TransformView::new(store),
            color_view: ColorView::new(store),
            metadata_view: MetadataView::new(store),
        }
    }

    /// Get entity position as Vec2
    #[inline(always)]
    pub fn pos(&self, index: usize) -> Option<(f32, f32)> {
        self.transform_view.position(index)
    }

    /// Get entity size
    /// Gets the size (width, height) for an entity.
    #[inline(always)]
    pub fn size(&self, index: usize) -> Option<(f32, f32)> {
        self.transform_view.size(index)
    }

    /// Get fill color
    #[inline(always)]
    pub fn fill_color(&self, index: usize) -> Option<u32> {
        self.color_view.fill_color(index)
    }

    /// Get stroke color
    #[inline(always)]
    pub fn stroke_color(&self, index: usize) -> Option<u32> {
        self.color_view.stroke_color(index)
    }

    /// Get stroke width
    #[inline(always)]
    pub fn stroke_width(&self, index: usize) -> Option<f32> {
        self.color_view.stroke_width(index)
    }

    /// Get texture index
    #[inline(always)]
    pub fn texture_index(&self, index: usize) -> Option<u16> {
        self.color_view.texture_index(index)
    }

    /// Get UV rect
    #[inline(always)]
    pub fn uv_rect(&self, index: usize) -> Option<[f32; 4]> {
        self.color_view.uv_rect(index)
    }

    /// Get layer
    #[inline(always)]
    pub fn layer(&self, index: usize) -> Option<u8> {
        self.metadata_view.layer(index)
    }

    /// Check if entity is visible
    #[inline(always)]
    pub fn is_visible(&self, index: usize) -> Option<bool> {
        self.metadata_view.is_visible(index)
    }

    /// Check if entity is locked
    #[inline(always)]
    pub fn is_locked(&self, index: usize) -> Option<bool> {
        self.metadata_view.is_locked(index)
    }

    /// Check if entity is alive
    #[inline(always)]
    pub fn is_alive(&self, index: usize) -> bool {
        self.store.is_alive_index(index)
    }

    /// Get text glyph count (for phase determination)
    #[inline(always)]
    pub fn text_glyph_count(&self, index: usize) -> u16 {
        self.store.text_glyph_count[index]
    }

    /// Get the underlying store reference
    #[inline(always)]
    pub fn store(&self) -> &'a EntityStore {
        self.store
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// Query Functions
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Query all visible entities
#[inline(always)]
pub fn query_visible(store: &EntityStore) -> Vec<QueryResult> {
    let mut results = Vec::new();
    for idx in 0..super::MAX_ENTITIES {
        if store.is_alive_index(idx) && store.is_visible(idx) {
            let t = store.transforms[idx];
            results.push(QueryResult {
                entity: EntityId::from_parts(Index(idx as u32), Generation(0)),
                index: idx,
                x: t[0],
                y: t[1],
                width: t[2],
                height: t[3],
                layer: store.layer(idx),
                is_visible: true,
                is_selected: store.is_selected(idx),
                is_locked: (store.metadata[idx] & (1 << 10)) != 0,
            });
        }
    }
    results
}

/// Query entities with dirty render state
#[inline(always)]
pub fn query_dirty_render(store: &EntityStore) -> Vec<QueryResult> {
    let mut results = Vec::new();
    for idx in store.dirty_render.ones() {
        if store.is_alive_index(idx) {
            let t = store.transforms[idx];
            results.push(QueryResult {
                entity: EntityId::from_parts(Index(idx as u32), Generation(0)),
                index: idx,
                x: t[0],
                y: t[1],
                width: t[2],
                height: t[3],
                layer: store.layer(idx),
                is_visible: store.is_visible(idx),
                is_selected: store.is_selected(idx),
                is_locked: (store.metadata[idx] & (1 << 10)) != 0,
            });
        }
    }
    results
}

/// Query renderable entities (visible, dirty, not locked)
#[inline(always)]
pub fn query_renderable(store: &EntityStore) -> Vec<RenderableResult> {
    let mut results = Vec::new();
    for idx in store.dirty_render.ones() {
        if store.is_alive_index(idx) && store.is_visible(idx) {
            let is_locked = (store.metadata[idx] & (1 << 10)) != 0;
            if !is_locked {
                let t = store.transforms[idx];
                results.push(RenderableResult {
                    entity: EntityId::from_parts(Index(idx as u32), Generation(0)),
                    index: idx,
                    x: t[0],
                    y: t[1],
                    width: t[2],
                    height: t[3],
                    layer: store.layer(idx),
                    fill_color: store.colors[idx],
                    stroke_color: store.stroke_colors[idx],
                    stroke_width: store.stroke_widths[idx],
                    texture_index: store.texture_index[idx],
                    uv_rect: store.uv_rects[idx],
                    is_visible: true,
                    is_selected: store.is_selected(idx),
                    is_locked: false,
                });
            }
        }
    }
    results
}

/// Query all alive entities
#[inline(always)]
pub fn query_alive(store: &EntityStore) -> Vec<QueryResult> {
    let mut results = Vec::with_capacity(store.alive_count());
    for idx in 0..super::MAX_ENTITIES {
        if store.is_alive_index(idx) {
            let t = store.transforms[idx];
            results.push(QueryResult {
                entity: EntityId::from_parts(Index(idx as u32), Generation(0)),
                index: idx,
                x: t[0],
                y: t[1],
                width: t[2],
                height: t[3],
                layer: store.layer(idx),
                is_visible: store.is_visible(idx),
                is_selected: store.is_selected(idx),
                is_locked: (store.metadata[idx] & (1 << 10)) != 0,
            });
        }
    }
    results
}

/// Mark entities as clean after rendering
#[inline(always)]
pub fn mark_clean(store: &mut EntityStore, indices: &[usize]) {
    for &idx in indices {
        store.dirty_render.remove(idx);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// Component Views
// ═══════════════════════════════════════════════════════════════════════════════════════

/// View over transform components
#[derive(Copy, Clone)]
pub struct TransformView<'a>(&'a EntityStore);

impl<'a> TransformView<'a> {
    /// Creates a new transform view.
    #[inline(always)]
    #[must_use]
    pub fn new(store: &'a EntityStore) -> Self {
        Self(store)
    }

    /// Gets the transform array for an entity.
    #[inline(always)]
    pub fn transform(&self, index: usize) -> Option<[f32; 4]> {
        if !self.0.is_alive_index(index) {
            return None;
        }
        Some(self.0.transforms[index])
    }

    /// Gets the position (x, y) for an entity.
    #[inline(always)]
    pub fn position(&self, index: usize) -> Option<(f32, f32)> {
        if !self.0.is_alive_index(index) {
            return None;
        }
        let t = self.0.transforms[index];
        Some((t[0], t[1]))
    }

    /// Gets the size (width, height) for an entity.
    #[inline(always)]
    pub fn size(&self, index: usize) -> Option<(f32, f32)> {
        if !self.0.is_alive_index(index) {
            return None;
        }
        let t = self.0.transforms[index];
        Some((t[2], t[3]))
    }

    /// Gets the world transform for an entity.
    #[inline(always)]
    pub fn world_transform(&self, index: usize) -> Option<[f32; 4]> {
        if !self.0.is_alive_index(index) {
            return None;
        }
        Some(self.0.world_transform[index])
    }
}

/// View over color components
#[derive(Copy, Clone)]
pub struct ColorView<'a>(&'a EntityStore);

impl<'a> ColorView<'a> {
    /// Creates a new color view.
    #[inline(always)]
    #[must_use]
    pub fn new(store: &'a EntityStore) -> Self {
        Self(store)
    }

    /// Gets the fill color for an entity.
    #[inline(always)]
    pub fn fill_color(&self, index: usize) -> Option<u32> {
        if !self.0.is_alive_index(index) {
            return None;
        }
        Some(self.0.colors[index])
    }

    /// Gets the stroke color for an entity.
    #[inline(always)]
    pub fn stroke_color(&self, index: usize) -> Option<u32> {
        if !self.0.is_alive_index(index) {
            return None;
        }
        Some(self.0.stroke_colors[index])
    }

    /// Gets the stroke width for an entity.
    #[inline(always)]
    pub fn stroke_width(&self, index: usize) -> Option<f32> {
        if !self.0.is_alive_index(index) {
            return None;
        }
        Some(self.0.stroke_widths[index])
    }

    /// Gets the texture index for an entity.
    #[inline(always)]
    pub fn texture_index(&self, index: usize) -> Option<u16> {
        if !self.0.is_alive_index(index) {
            return None;
        }
        Some(self.0.texture_index[index])
    }

    /// Gets the UV rect for an entity.
    #[inline(always)]
    pub fn uv_rect(&self, index: usize) -> Option<[f32; 4]> {
        if !self.0.is_alive_index(index) {
            return None;
        }
        Some(self.0.uv_rects[index])
    }
}

/// View over metadata components
#[derive(Copy, Clone)]
pub struct MetadataView<'a>(&'a EntityStore);

impl<'a> MetadataView<'a> {
    /// Creates a new metadata view.
    #[inline(always)]
    #[must_use]
    pub fn new(store: &'a EntityStore) -> Self {
        Self(store)
    }

    /// Gets the layer for an entity.
    #[inline(always)]
    pub fn layer(&self, index: usize) -> Option<u8> {
        if !self.0.is_alive_index(index) {
            return None;
        }
        Some(self.0.layer(index))
    }

    /// Checks if an entity is visible.
    #[inline(always)]
    pub fn is_visible(&self, index: usize) -> Option<bool> {
        if !self.0.is_alive_index(index) {
            return None;
        }
        Some(self.0.is_visible(index))
    }

    /// Checks if an entity is locked.
    #[inline(always)]
    pub fn is_locked(&self, index: usize) -> Option<bool> {
        if !self.0.is_alive_index(index) {
            return None;
        }
        Some((self.0.metadata[index] & (1 << 10)) != 0)
    }

    /// Checks if an entity is selected.
    #[inline(always)]
    pub fn is_selected(&self, index: usize) -> Option<bool> {
        if !self.0.is_alive_index(index) {
            return None;
        }
        Some(self.0.is_selected(index))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_visible_empty() {
        let store = EntityStore::new();
        let results = query_visible(&store);
        assert!(results.is_empty());
    }

    #[test]
    fn test_query_dirty_render_empty() {
        let store = EntityStore::new();
        let results = query_dirty_render(&store);
        assert!(results.is_empty());
    }

    #[test]
    fn test_query_renderable_empty() {
        let store = EntityStore::new();
        let results = query_renderable(&store);
        assert!(results.is_empty());
    }

    #[test]
    fn test_query_alive_empty() {
        let store = EntityStore::new();
        let results = query_alive(&store);
        assert!(results.is_empty());
    }

    #[test]
    fn test_transform_view() {
        let store = EntityStore::new();
        let view = TransformView::new(&store);
        assert!(view.transform(0).is_none());
        assert!(view.position(0).is_none());
    }

    #[test]
    fn test_color_view() {
        let store = EntityStore::new();
        let view = ColorView::new(&store);
        assert!(view.fill_color(0).is_none());
    }

    #[test]
    fn test_metadata_view() {
        let store = EntityStore::new();
        let view = MetadataView::new(&store);
        assert!(view.layer(0).is_none());
        assert!(view.is_visible(0).is_none());
    }

    #[test]
    fn test_render_query_creation() {
        let store = EntityStore::new();
        let query = RenderQuery::new(&store);
        // Query should be created without error
        // Store reference should work
        assert_eq!(query.store().alive_count(), 0);
    }

    #[test]
    fn test_render_query_with_entity() {
        let mut store = EntityStore::new();
        let pos = archflow_core::Vec2::new(100.0, 200.0);
        let size = archflow_core::Vec2::new(50.0, 30.0);
        let _id = store.spawn(pos, size);

        let query = RenderQuery::new(&store);

        // Check position and size (now returns tuples)
        let pos_result = query.pos(0);
        let size_result = query.size(0);

        assert!(pos_result.is_some());
        assert!(size_result.is_some());
        assert_eq!(pos_result.unwrap(), (100.0, 200.0));
        assert_eq!(size_result.unwrap(), (50.0, 30.0));

        // Check alive status
        assert!(query.is_alive(0));
        assert!(!query.is_alive(99999));
    }

    #[test]
    fn test_render_query_properties() {
        let mut store = EntityStore::new();
        let _id = store.spawn(
            archflow_core::Vec2::new(0.0, 0.0),
            archflow_core::Vec2::new(10.0, 10.0),
        );

        let query = RenderQuery::new(&store);

        // Default properties - Query returns Option<T>
        assert!(query.is_visible(0).unwrap_or(false));
        assert!(!query.is_locked(0).unwrap_or(false));
        assert_eq!(query.texture_index(0), Some(0));
        assert_eq!(query.text_glyph_count(0), 0);
        assert_eq!(query.uv_rect(0), Some([0.0, 0.0, 1.0, 1.0]));
    }
}
