//! WASM Bridge for Rust ↔ JavaScript Collaboration
//!
//! This module provides the main bridge between Rust WASM and JavaScript,
//! enabling zero-copy shared memory communication and efficient delta synchronization.

use wasm_bindgen::prelude::*;

use crate::binary_delta_codec::{BinaryDeltaCodec, ShapeField};
use crate::shared_buffer::{RenderAttribute, SharedBuffer};

/// Bridge principal Rust → JavaScript para colaboración.
#[wasm_bindgen]
pub struct WasmBridge {
    /// Shared buffer for render data (zero-copy)
    shared_buffer: SharedBuffer,
    /// Codec for binary delta encoding (not used, kept for API compatibility)
    #[allow(dead_code)]
    codec: BinaryDeltaCodec,
    /// Local state: ID -> (x, y, color)
    state: Vec<Option<(f32, f32, [u8; 4])>>,
    /// Track which IDs have been modified
    dirty_ids: Vec<u64>,
}

#[wasm_bindgen]
impl WasmBridge {
    /// Creates a new WasmBridge with the specified capacity.
    #[wasm_bindgen(constructor)]
    pub fn new(max_elements: usize) -> Self {
        console_error_panic_hook::set_once();

        Self {
            shared_buffer: SharedBuffer::new(max_elements),
            codec: BinaryDeltaCodec,
            state: vec![None; max_elements],
            dirty_ids: Vec::new(),
        }
    }

    /// Gets the render buffer pointer for JavaScript access.
    #[wasm_bindgen(getter)]
    pub fn render_buffer_ptr(&self) -> *const RenderAttribute {
        self.shared_buffer.get_ptr()
    }

    /// Gets the render buffer length (number of elements).
    #[wasm_bindgen(getter)]
    pub fn render_buffer_len(&self) -> usize {
        self.shared_buffer.len()
    }

    /// Gets the maximum buffer capacity.
    #[wasm_bindgen(getter)]
    pub fn render_buffer_capacity(&self) -> usize {
        self.shared_buffer.max_elements()
    }

    /// Gets the number of dirty (modified) records.
    #[wasm_bindgen(getter)]
    pub fn dirty_count(&self) -> usize {
        self.dirty_ids.len()
    }

    /// Updates a single record's position.
    #[wasm_bindgen]
    pub fn update_position(&mut self, id: u64, x: f32, y: f32) {
        self.ensure_capacity(id);
        let idx = id as usize;

        let color = self.state[idx]
            .map(|(_, _, c)| c)
            .unwrap_or([255, 255, 255, 255]);
        self.state[idx] = Some((x, y, color));

        if !self.dirty_ids.contains(&id) {
            self.dirty_ids.push(id);
        }
    }

    /// Updates a single record's color.
    #[wasm_bindgen]
    pub fn update_color(&mut self, id: u64, r: u8, g: u8, b: u8, a: u8) {
        self.ensure_capacity(id);
        let idx = id as usize;

        let pos = self.state[idx]
            .map(|(x, y, _)| (x, y))
            .unwrap_or((0.0, 0.0));
        self.state[idx] = Some((pos.0, pos.1, [r, g, b, a]));

        if !self.dirty_ids.contains(&id) {
            self.dirty_ids.push(id);
        }
    }

    /// Updates a single record's size.
    #[wasm_bindgen]
    pub fn update_size(&mut self, _id: u64, _width: f32, _height: f32) {
        // Size updates could affect bounds but for now we
        // just mark as dirty for re-render
    }

    /// Deletes a record.
    #[wasm_bindgen]
    pub fn delete(&mut self, id: u64) {
        if id as usize >= self.state.len() {
            return;
        }

        self.state[id as usize] = None;
        if !self.dirty_ids.contains(&id) {
            self.dirty_ids.push(id);
        }
    }

    /// Applies a binary delta received from the network.
    /// Returns number of records updated, or -1 if decode error.
    #[wasm_bindgen]
    pub fn apply_delta(&mut self, data: &[u8]) -> Result<usize, JsValue> {
        let delta = BinaryDeltaCodec::decode_delta(data)
            .ok_or_else(|| JsValue::from_str("Invalid delta format"))?;

        self.ensure_capacity(delta.id);
        let idx = delta.id as usize;

        // Apply position
        if let Some((x, y)) = delta.position {
            let current = self.state[idx];
            let color = current.map(|(_, _, c)| c).unwrap_or([255, 255, 255, 255]);
            self.state[idx] = Some((x, y, color));
        }

        // Apply color
        if let Some((r, g, b, a)) = delta.color {
            let current = self.state[idx];
            let (x, y) = current.map(|(x, y, _)| (x, y)).unwrap_or((0.0, 0.0));
            self.state[idx] = Some((x, y, [r, g, b, a]));
        }

        // Track dirty
        if !self.dirty_ids.contains(&delta.id) {
            self.dirty_ids.push(delta.id);
        }

        Ok(1)
    }

    /// Applies multiple deltas from a concatenated buffer.
    #[wasm_bindgen]
    pub fn apply_deltas(&mut self, data: &[u8]) -> usize {
        let mut pos = 0;
        let mut count = 0;

        while pos < data.len() {
            if let Some(delta) = BinaryDeltaCodec::decode_delta(&data[pos..]) {
                let delta_size = Self::delta_size(&data[pos..]);
                pos += delta_size;

                self.ensure_capacity(delta.id);
                let idx = delta.id as usize;

                if let Some((x, y)) = delta.position {
                    let current = self.state[idx];
                    let color = current.map(|(_, _, c)| c).unwrap_or([255, 255, 255, 255]);
                    self.state[idx] = Some((x, y, color));
                }

                if let Some((r, g, b, a)) = delta.color {
                    let current = self.state[idx];
                    let (x, y) = current.map(|(x, y, _)| (x, y)).unwrap_or((0.0, 0.0));
                    self.state[idx] = Some((x, y, [r, g, b, a]));
                }

                if !self.dirty_ids.contains(&delta.id) {
                    self.dirty_ids.push(delta.id);
                }

                count += 1;
            } else {
                break;
            }
        }

        count
    }

    /// Calculates the size of a delta at the given offset.
    fn delta_size(data: &[u8]) -> usize {
        if data.is_empty() {
            return 0;
        }

        if let Some((_, id_len)) = BinaryDeltaCodec::decode_varint(data) {
            if data.len() < id_len + 1 {
                return data.len();
            }
            let mask = data[id_len];
            let mut size = id_len + 1;

            if ShapeField::has_position(mask) {
                size += 8;
            }
            if ShapeField::has_color(mask) {
                size += 4;
            }
            if ShapeField::has_size(mask) {
                size += 8;
            }

            size
        } else {
            data.len()
        }
    }

    /// Prepares changes for network transmission.
    #[wasm_bindgen]
    pub fn serialize_changes(&mut self) -> Vec<u8> {
        let mut result = Vec::new();

        for &id in &self.dirty_ids {
            let idx = id as usize;
            if let Some((x, y, color)) = self.state[idx] {
                let mut delta = Vec::new();
                BinaryDeltaCodec::encode_delta(
                    &mut delta,
                    id,
                    (ShapeField::Position as u8) | (ShapeField::Color as u8),
                    Some((x, y)),
                    Some((color[0], color[1], color[2], color[3])),
                    None,
                );
                result.extend(delta);
            }
        }

        self.dirty_ids.clear();
        result
    }

    /// Prepares changes for a single record.
    #[wasm_bindgen]
    pub fn serialize_record(&self, id: u64) -> Vec<u8> {
        let idx = id as usize;
        if let Some((x, y, color)) = self.state[idx] {
            let mut delta = Vec::new();
            BinaryDeltaCodec::encode_delta(
                &mut delta,
                id,
                (ShapeField::Position as u8) | (ShapeField::Color as u8),
                Some((x, y)),
                Some((color[0], color[1], color[2], color[3])),
                None,
            );
            delta
        } else {
            Vec::new()
        }
    }

    /// Updates the shared buffer for rendering.
    #[wasm_bindgen]
    pub fn update_render_buffer(&mut self) {
        let visible_ids: Vec<u64> = self
            .state
            .iter()
            .enumerate()
            .filter(|(_, opt)| opt.is_some())
            .map(|(idx, _)| idx as u64)
            .collect();

        let get_record = |id: u64| -> Option<(f32, f32, [u8; 4])> {
            self.state.get(id as usize).and_then(|opt| *opt)
        };

        self.shared_buffer.update(&visible_ids, get_record);
    }

    /// Gets the number of records in the bridge.
    #[wasm_bindgen(getter)]
    pub fn record_count(&self) -> usize {
        self.state.iter().filter(|opt| opt.is_some()).count()
    }

    /// Clears all state.
    #[wasm_bindgen]
    pub fn clear(&mut self) {
        for state in &mut self.state {
            *state = None;
        }
        self.dirty_ids.clear();
    }

    /// Ensures the internal state can hold the given ID.
    fn ensure_capacity(&mut self, id: u64) {
        let idx = id as usize;
        if idx >= self.state.len() {
            self.state.resize_with(idx + 1, || None);
            self.shared_buffer = SharedBuffer::new(self.state.len());
        }
    }
}

/// Logs a message to the console from Rust.
#[wasm_bindgen]
pub fn log(message: &str) {
    web_sys::console::log_1(&message.into());
}

/// Logs an error to the console from Rust.
#[wasm_bindgen]
pub fn log_error(message: &str) {
    web_sys::console::error_1(&message.into());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_creation() {
        let bridge = WasmBridge::new(100);
        assert_eq!(bridge.render_buffer_capacity(), 100);
        assert_eq!(bridge.render_buffer_len(), 0);
        assert_eq!(bridge.record_count(), 0);
    }

    #[test]
    fn test_update_position() {
        let mut bridge = WasmBridge::new(100);
        bridge.update_position(1, 100.0, 200.0);
        assert_eq!(bridge.record_count(), 1);
        assert_eq!(bridge.dirty_count(), 1);
    }

    #[test]
    fn test_update_color() {
        let mut bridge = WasmBridge::new(100);
        bridge.update_color(1, 255, 128, 64, 255);
        assert_eq!(bridge.record_count(), 1);
    }

    #[test]
    fn test_apply_delta() {
        let mut bridge = WasmBridge::new(100);

        let mut delta = Vec::new();
        BinaryDeltaCodec::encode_delta(
            &mut delta,
            42,
            (ShapeField::Position as u8) | (ShapeField::Color as u8),
            Some((150.0, 250.0)),
            Some((100, 150, 200, 255)),
            None,
        );

        let result = bridge.apply_delta(&delta);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
        assert_eq!(bridge.record_count(), 1);
    }

    #[test]
    fn test_apply_deltas() {
        let mut bridge = WasmBridge::new(100);

        let mut data = Vec::new();
        for i in 0..5 {
            BinaryDeltaCodec::encode_delta(
                &mut data,
                i as u64,
                ShapeField::Position as u8,
                Some((i as f32 * 10.0, i as f32 * 20.0)),
                None,
                None,
            );
        }

        let count = bridge.apply_deltas(&data);
        assert_eq!(count, 5);
        assert_eq!(bridge.record_count(), 5);
    }

    #[test]
    fn test_serialize_changes() {
        let mut bridge = WasmBridge::new(100);
        bridge.update_position(1, 100.0, 200.0);
        bridge.update_color(2, 255, 0, 0, 255);

        let changes = bridge.serialize_changes();
        assert!(!changes.is_empty());
        assert_eq!(bridge.dirty_count(), 0);
    }

    #[test]
    fn test_delete() {
        let mut bridge = WasmBridge::new(100);
        bridge.update_position(1, 100.0, 200.0);
        assert_eq!(bridge.record_count(), 1);

        bridge.delete(1);
        assert_eq!(bridge.record_count(), 0);
    }

    #[test]
    fn test_clear() {
        let mut bridge = WasmBridge::new(100);
        bridge.update_position(1, 100.0, 200.0);
        bridge.update_position(2, 150.0, 250.0);

        bridge.clear();
        assert_eq!(bridge.record_count(), 0);
        assert_eq!(bridge.dirty_count(), 0);
    }

    #[test]
    fn test_render_buffer_update() {
        let mut bridge = WasmBridge::new(100);
        bridge.update_position(1, 100.0, 200.0);
        bridge.update_position(2, 150.0, 250.0);

        bridge.update_render_buffer();
        assert_eq!(bridge.render_buffer_len(), 2);
    }
}
