//! RenderBatch - Zero-Copy WebGPU Upload Bridge
//!
//! Provides zero-copy access to WASM memory for efficient GPU upload of dirty ranges.
//! This module bridges the SOA entity store with WebGPU rendering, enabling partial
//! buffer uploads instead of transferring entire buffers every frame.

use crate::EntityId;
use crate::store::EntityStore;
use js_sys::Float32Array;
use wasm_bindgen::prelude::*;

/// Zero-copy render batch for WebGPU upload.
///
/// Owns interleaved GPU-ready data converted from SOA format, enabling efficient
/// partial buffer uploads through zero-copy TypedArray views.
///
/// # Memory Layout
///
/// ```text
/// RenderBatch
/// ├── positions: Vec<f32>     → Interleaved [x0, y0, x1, y1, ...]
/// ├── colors: Vec<f32>        → Interleaved [r0, g0, b0, a0, r1, g1, b1, a1, ...]
/// ├── position_dirty_range    → (start_index, length) of dirty positions
/// └── color_dirty_range       → (start_index, length) of dirty colors
/// ```
///
/// # Zero-Copy Semantics
///
/// The `*_slice()` methods use `Float32Array::view()` which creates a JavaScript
/// TypedArray view directly into the Vec's memory without copying. This enables:
///
/// - **Direct GPU Upload**: WebGPU can read directly from WASM memory
/// - **Partial Updates**: Only dirty ranges are transferred
/// - **No Allocation**: JavaScript side gets a view, not a copy
///
/// # Examples
///
/// ```rust,no_run
/// use soa_entity::{EntityStore, RenderBatch};
/// use archflow_core::Vec2;
///
/// let mut store = EntityStore::new(1000);
/// let id = store.spawn().unwrap();
/// store.set_position(id, Vec2::new(10.0, 20.0)).unwrap();
///
/// let batch = RenderBatch::from_store(&store);
///
/// // Get zero-copy view of ALL positions for GPU upload
/// let positions_view = batch.positions_slice();
///
/// // Get zero-copy view of ONLY dirty positions (optimal)
/// let dirty_positions = batch.positions_dirty_slice();
/// let offset = batch.position_dirty_byte_offset();
/// // Upload to WebGPU: queue.writeBuffer(buffer, offset, dirty_positions)
/// ```

#[wasm_bindgen]
pub struct RenderBatch {
    /// Number of valid entities in the batch
    count: usize,

    /// Interleaved position data: [x0, y0, x1, y1, ...]
    positions: Vec<f32>,

    /// Interleaved color data: [r0, g0, b0, a0, r1, g1, b1, a1, ...]
    colors: Vec<f32>,

    /// Dirty range for positions: (start_index, length)
    position_dirty_range: Option<(usize, usize)>,

    /// Dirty range for colors: (start_index, length)
    color_dirty_range: Option<(usize, usize)>,
}

impl RenderBatch {
    /// Creates a new RenderBatch from an EntityStore.
    ///
    /// This method copies and interleaves SOA data into GPU-ready format,
    /// capturing dirty ranges for efficient partial uploads.
    ///
    /// # Arguments
    ///
    /// * `store` - Reference to the entity store
    ///
    /// # Returns
    ///
    /// A new RenderBatch with interleaved data and captured dirty ranges
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use soa_entity::{EntityStore, RenderBatch};
    ///
    /// let store = EntityStore::new(1000);
    /// let batch = RenderBatch::from_store(&store);
    /// ```
    pub fn from_store(store: &EntityStore) -> Self {
        let capacity = store.capacity();
        let count = store.count();

        // Pre-allocate interleaved buffers
        let mut positions = Vec::with_capacity(count * 2);
        let mut colors = Vec::with_capacity(count * 4);

        // Copy and interleave SOA data into GPU-ready format
        // We iterate through all indices and check validity using is_valid
        for index in 0..capacity {
            // Construct EntityId with the correct generation for this index
            let generation = store.generation_for_index(index);

            // Skip unused slots (generation 0 means never spawned)
            if generation == 0 {
                continue;
            }

            let id = EntityId::new(index, generation);

            if store.is_valid(id) {
                // Interleave positions: [x, y]
                if let Some(pos) = store.position(id) {
                    positions.push(pos.x);
                    positions.push(pos.y);
                }

                // Interleave colors: [r, g, b, a]
                if let Some(col) = store.color(id) {
                    colors.push(col.r);
                    colors.push(col.g);
                    colors.push(col.b);
                    colors.push(col.a);
                }
            }
        }

        // Calculate dirty ranges from bitsets
        let position_ranges = store.calculate_dirty_ranges(store.dirty_positions());
        let color_ranges = store.calculate_dirty_ranges(store.dirty_colors());

        // Take the first contiguous range (most common case)
        let position_dirty_range = position_ranges.first().copied();
        let color_dirty_range = color_ranges.first().copied();

        Self {
            count,
            positions,
            colors,
            position_dirty_range,
            color_dirty_range,
        }
    }
}

/// WASM-exposed API for RenderBatch.
///
/// These methods are callable from JavaScript and provide zero-copy
/// access to the interleaved GPU-ready data.
#[wasm_bindgen]
impl RenderBatch {
    /// Returns the number of entities in the batch.
    #[wasm_bindgen(getter)]
    pub fn count(&self) -> usize {
        self.count
    }

    /// Returns the start index of dirty positions, if any.
    #[wasm_bindgen(getter)]
    pub fn position_dirty_start(&self) -> Option<usize> {
        self.position_dirty_range.map(|(start, _)| start)
    }

    /// Returns the length of dirty positions, if any.
    #[wasm_bindgen(getter)]
    pub fn position_dirty_length(&self) -> Option<usize> {
        self.position_dirty_range.map(|(_, length)| length)
    }

    /// Returns the start index of dirty colors, if any.
    #[wasm_bindgen(getter)]
    pub fn color_dirty_start(&self) -> Option<usize> {
        self.color_dirty_range.map(|(start, _)| start)
    }

    /// Returns the length of dirty colors, if any.
    #[wasm_bindgen(getter)]
    pub fn color_dirty_length(&self) -> Option<usize> {
        self.color_dirty_range.map(|(_, length)| length)
    }

    /// Creates a zero-copy Float32Array view of ALL positions.
    ///
    /// This provides direct access to WASM memory without copying,
    /// enabling efficient WebGPU buffer uploads.
    ///
    /// # Returns
    ///
    /// A JavaScript Float32Array view of all positions [x0, y0, x1, y1, ...]
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // In JavaScript
    /// const positions = batch.positions_slice();
    /// // Upload to WebGPU
    /// device.queue.writeBuffer(
    ///     positionBuffer,
    ///     0,
    ///     positions,
    ///     0,
    ///     positions.length * 4
    /// );
    /// ```
    #[wasm_bindgen]
    pub fn positions_slice(&self) -> Float32Array {
        // SAFETY: The Vec's data is valid for the lifetime of this view.
        // The Float32Array view is used immediately by JavaScript and does not outlive the Vec.
        unsafe { Float32Array::view(&self.positions) }
    }

    /// Creates a zero-copy Float32Array view of DIRTY positions only.
    ///
    /// This is optimized for partial uploads when only some entities changed.
    /// Returns an empty array if no entities are dirty.
    ///
    /// # Returns
    ///
    /// A JavaScript Float32Array view of dirty positions [x, y, x, y, ...]
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const dirtyPositions = batch.positions_dirty_slice();
    /// if (dirtyPositions.length > 0) {
    ///     const offset = batch.position_dirty_byte_offset();
    ///     device.queue.writeBuffer(
    ///         positionBuffer,
    ///         offset,
    ///         dirtyPositions,
    ///         0,
    ///         dirtyPositions.length * 4
    ///     );
    /// }
    /// ```
    #[wasm_bindgen]
    pub fn positions_dirty_slice(&self) -> Float32Array {
        if let Some((start, length)) = self.position_dirty_range {
            let start_idx = start * 2;
            let end_idx = start_idx + length * 2;
            // SAFETY: The Vec's data is valid for the lifetime of this view.
            unsafe { Float32Array::view(&self.positions[start_idx..end_idx]) }
        } else {
            // SAFETY: Empty slice is safe.
            unsafe { Float32Array::view(&self.positions[0..0]) }
        }
    }

    /// Creates a zero-copy Float32Array view of ALL colors.
    ///
    /// Colors are stored as RGBA interleaved: [r0, g0, b0, a0, r1, g1, b1, a1, ...]
    ///
    /// # Returns
    ///
    /// A JavaScript Float32Array view of all colors
    #[wasm_bindgen]
    pub fn colors_slice(&self) -> Float32Array {
        // SAFETY: The Vec's data is valid for the lifetime of this view.
        // The Float32Array view is used immediately by JavaScript and does not outlive the Vec.
        unsafe { Float32Array::view(&self.colors) }
    }

    /// Creates a zero-copy Float32Array view of DIRTY colors only.
    ///
    /// This is optimized for partial uploads when only some entities changed.
    /// Returns an empty array if no entities are dirty.
    ///
    /// # Returns
    ///
    /// A JavaScript Float32Array view of dirty colors [r, g, b, a, r, g, b, a, ...]
    #[wasm_bindgen]
    pub fn colors_dirty_slice(&self) -> Float32Array {
        if let Some((start, length)) = self.color_dirty_range {
            let start_idx = start * 4;
            let end_idx = start_idx + length * 4;
            // SAFETY: The Vec's data is valid for the lifetime of this view.
            unsafe { Float32Array::view(&self.colors[start_idx..end_idx]) }
        } else {
            // SAFETY: Empty slice is safe.
            unsafe { Float32Array::view(&self.colors[0..0]) }
        }
    }

    /// Calculates the byte offset for dirty position upload.
    ///
    /// This is useful for WebGPU writeBuffer offset parameter.
    ///
    /// # Returns
    ///
    /// Byte offset for the dirty range, or 0 if no dirty entities
    #[wasm_bindgen]
    pub fn position_dirty_byte_offset(&self) -> usize {
        self.position_dirty_range
            .map(|(start, _)| start * 8) // 2 floats × 4 bytes
            .unwrap_or(0)
    }

    /// Calculates the byte size for dirty position upload.
    ///
    /// This is useful for WebGPU writeBuffer size parameter.
    ///
    /// # Returns
    ///
    /// Byte size for the dirty range, or 0 if no dirty entities
    #[wasm_bindgen]
    pub fn position_dirty_byte_size(&self) -> usize {
        self.position_dirty_range
            .map(|(_, length)| length * 8) // 2 floats × 4 bytes
            .unwrap_or(0)
    }

    /// Calculates the byte offset for dirty color upload.
    ///
    /// This is useful for WebGPU writeBuffer offset parameter.
    ///
    /// # Returns
    ///
    /// Byte offset for the dirty range, or 0 if no dirty entities
    #[wasm_bindgen]
    pub fn color_dirty_byte_offset(&self) -> usize {
        self.color_dirty_range
            .map(|(start, _)| start * 16) // 4 floats × 4 bytes
            .unwrap_or(0)
    }

    /// Calculates the byte size for dirty color upload.
    ///
    /// This is useful for WebGPU writeBuffer size parameter.
    ///
    /// # Returns
    ///
    /// Byte size for the dirty range, or 0 if no dirty entities
    #[wasm_bindgen]
    pub fn color_dirty_byte_size(&self) -> usize {
        self.color_dirty_range
            .map(|(_, length)| length * 16) // 4 floats × 4 bytes
            .unwrap_or(0)
    }
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_core::{Color, Vec2};

    #[test]
    fn test_render_batch_from_store() {
        let mut store = EntityStore::new(100);
        let id1 = store.spawn().unwrap();
        let id2 = store.spawn().unwrap();

        store.set_position(id1, Vec2::new(10.0, 20.0)).unwrap();
        store.set_position(id2, Vec2::new(30.0, 40.0)).unwrap();

        let batch = RenderBatch::from_store(&store);

        assert_eq!(batch.count(), 2);
        assert_eq!(batch.position_dirty_start(), Some(0));
        assert_eq!(batch.position_dirty_length(), Some(2));
    }

    #[test]
    fn test_render_batch_no_dirty_entities() {
        let store = EntityStore::new(100);

        let batch = RenderBatch::from_store(&store);

        assert_eq!(batch.count(), 0);
        assert_eq!(batch.position_dirty_start(), None);
        assert_eq!(batch.position_dirty_length(), None);
    }

    #[test]
    fn test_position_dirty_byte_calculations() {
        let mut store = EntityStore::new(100);
        let id1 = store.spawn().unwrap();
        let id2 = store.spawn().unwrap();

        store.set_position(id1, Vec2::new(10.0, 20.0)).unwrap();
        store.set_position(id2, Vec2::new(30.0, 40.0)).unwrap();

        let batch = RenderBatch::from_store(&store);

        // 2 entities dirty, starting at index 0
        assert_eq!(batch.position_dirty_byte_offset(), 0);
        assert_eq!(batch.position_dirty_byte_size(), 16); // 2 entities × 2 floats × 4 bytes
    }

    #[test]
    fn test_color_dirty_byte_calculations() {
        let mut store = EntityStore::new(100);
        let id1 = store.spawn().unwrap();

        store
            .set_color(id1, Color::rgba(1.0, 0.5, 0.25, 1.0))
            .unwrap();

        let batch = RenderBatch::from_store(&store);

        // 1 entity dirty, starting at index 0
        assert_eq!(batch.color_dirty_byte_offset(), 0);
        assert_eq!(batch.color_dirty_byte_size(), 16); // 1 entity × 4 floats × 4 bytes
    }

    #[test]
    fn test_partial_dirty_range() {
        let mut store = EntityStore::new(100);

        // Spawn 10 entities
        let ids: Vec<_> = (0..10).map(|_| store.spawn().unwrap()).collect();

        // Mark only entities 5-9 as dirty
        for i in 5..10 {
            store
                .set_position(ids[i], Vec2::new(i as f32, 0.0))
                .unwrap();
        }

        let batch = RenderBatch::from_store(&store);

        // Should capture the first contiguous dirty range (5, 5)
        assert_eq!(batch.position_dirty_start(), Some(5));
        assert_eq!(batch.position_dirty_length(), Some(5));
        assert_eq!(batch.position_dirty_byte_offset(), 40); // 5 × 8 = 40 bytes
    }

    #[test]
    fn test_interleaved_data_format() {
        let mut store = EntityStore::new(100);
        let id1 = store.spawn().unwrap();
        let id2 = store.spawn().unwrap();

        store.set_position(id1, Vec2::new(10.0, 20.0)).unwrap();
        store.set_position(id2, Vec2::new(30.0, 40.0)).unwrap();

        let batch = RenderBatch::from_store(&store);

        // Verify interleaved format: [x0, y0, x1, y1]
        assert_eq!(batch.positions.len(), 4);
        assert_eq!(batch.positions[0], 10.0); // x0
        assert_eq!(batch.positions[1], 20.0); // y0
        assert_eq!(batch.positions[2], 30.0); // x1
        assert_eq!(batch.positions[3], 40.0); // y1
    }

    #[test]
    fn test_color_interleaved_format() {
        let mut store = EntityStore::new(100);
        let id = store.spawn().unwrap();

        store
            .set_color(id, Color::rgba(1.0, 0.5, 0.25, 0.75))
            .unwrap();

        let batch = RenderBatch::from_store(&store);

        // Verify interleaved format: [r, g, b, a]
        assert_eq!(batch.colors.len(), 4);
        assert_eq!(batch.colors[0], 1.0); // r
        assert_eq!(batch.colors[1], 0.5); // g
        assert_eq!(batch.colors[2], 0.25); // b
        assert_eq!(batch.colors[3], 0.75); // a
    }

    #[test]
    fn test_render_batch_with_colors() {
        let mut store = EntityStore::new(100);
        let id = store.spawn().unwrap();

        store
            .set_color(id, Color::rgba(1.0, 0.0, 0.0, 1.0))
            .unwrap();

        let batch = RenderBatch::from_store(&store);

        assert_eq!(batch.color_dirty_start(), Some(0));
        assert_eq!(batch.color_dirty_length(), Some(1));
        assert_eq!(batch.color_dirty_byte_size(), 16); // 1 entity × 4 floats × 4 bytes
    }

    #[test]
    fn test_render_batch_mixed_dirty() {
        let mut store = EntityStore::new(100);
        let id1 = store.spawn().unwrap();
        let id2 = store.spawn().unwrap();

        store.set_position(id1, Vec2::new(10.0, 20.0)).unwrap();
        store
            .set_color(id2, Color::rgba(1.0, 0.0, 0.0, 1.0))
            .unwrap();

        let batch = RenderBatch::from_store(&store);

        // Positions dirty (id1)
        assert!(batch.position_dirty_start().is_some());
        // Colors dirty (id2)
        assert!(batch.color_dirty_start().is_some());
    }

    #[test]
    fn test_empty_batch_calculations() {
        let store = EntityStore::new(100);
        let batch = RenderBatch::from_store(&store);

        assert_eq!(batch.position_dirty_byte_offset(), 0);
        assert_eq!(batch.position_dirty_byte_size(), 0);
        assert_eq!(batch.color_dirty_byte_offset(), 0);
        assert_eq!(batch.color_dirty_byte_size(), 0);
    }

    #[test]
    fn test_batch_with_multiple_entities() {
        let mut store = EntityStore::new(100);

        // Spawn 5 entities with positions and colors
        for i in 0..5 {
            let id = store.spawn().unwrap();
            store
                .set_position(id, Vec2::new(i as f32 * 10.0, i as f32 * 20.0))
                .unwrap();
            store
                .set_color(id, Color::rgba(i as f32 * 0.2, 0.5, 0.5, 1.0))
                .unwrap();
        }

        let batch = RenderBatch::from_store(&store);

        assert_eq!(batch.count(), 5);
        assert_eq!(batch.positions.len(), 10); // 5 entities × 2 floats
        assert_eq!(batch.colors.len(), 20); // 5 entities × 4 floats
    }
}
