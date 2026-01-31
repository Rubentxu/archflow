// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Web - WASM Bridge
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 7, 21
//
// WASM bridge for JavaScript/WebAssembly communication:
// - Exposes engine functions to JavaScript via wasm-bindgen
// - Handles SharedArrayBuffer for lock-free input
// - Provides requestAnimationFrame loop integration
// - Manages canvas and WebGPU context
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]

use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::sync::Arc;
use core::cell::RefCell;
use wasm_bindgen::prelude::*;

use crate::engine::ArchFlowEngine;
use crate::input::{InputProcessor, InputRingBuffer, MAX_POINTERS};

/// Global engine instance (using RefCell for interior mutability in WASM)
static mut ENGINE: Option<ArchFlowEngine> = None;
static mut INPUT_PROCESSOR: Option<InputProcessor> = None;

/// WASM Bridge for JavaScript/WebAssembly communication
///
/// This struct provides the interface between JavaScript and the Rust engine.
/// It manages the engine lifecycle and exposes functions that can be called from JS.
#[wasm_bindgen]
pub struct WasmBridge {
    _private: (),
}

#[wasm_bindgen]
impl WasmBridge {
    /// Create a new WASM bridge
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Initialize the engine
    ///
    /// This should be called once when the application starts.
    #[wasm_bindgen]
    pub fn initialize(canvas_width: f32, canvas_height: f32) -> Result<(), JsValue> {
        unsafe {
            ENGINE = Some(ArchFlowEngine::new(canvas_width, canvas_height));
            INPUT_PROCESSOR = Some(InputProcessor::new());
        }
        Ok(())
    }

    /// Get a pointer to the SharedArrayBuffer for input events
    ///
    /// This returns a pointer to the InputRingBuffer that JavaScript can
    /// write to directly via SharedArrayBuffer.
    #[wasm_bindgen]
    pub fn get_input_buffer_ptr() -> *mut InputRingBuffer {
        unsafe {
            if let Some(processor) = &mut INPUT_PROCESSOR {
                processor.buffer() as *mut InputRingBuffer
            } else {
                core::ptr::null_mut()
            }
        }
    }

    /// Get the size of the input buffer in bytes
    #[wasm_bindgen]
    pub fn get_input_buffer_size() -> usize {
        core::mem::size_of::<InputRingBuffer>()
    }

    /// Run one frame of the engine
    ///
    /// This should be called from requestAnimationFrame.
    #[wasm_bindgen]
    pub fn tick(timestamp: f64) -> Result<(), JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
                // Process input events
                if let Some(processor) = &mut INPUT_PROCESSOR {
                    let events = processor.process_events();

                    for event in events {
                        Self::process_input_event(engine, &event);
                    }
                }

                // Update engine
                engine.tick(timestamp);
                Ok(())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Process a single input event and update the engine
    fn process_input_event(engine: &mut ArchFlowEngine, event: &crate::input::RawInputEvent) {
        use archflow_core::Vec2;

        match event.event_type {
            0 => {
                // Pointer Down - simple hit test
                let world_pos = engine.screen_to_world(event.x, event.y);

                // Simple hit test: check if any entity contains the point
                for &entity_idx in &engine.store.draw_order[..engine.store.alive_count()] {
                    let idx = entity_idx as usize;
                    if !engine.store.is_visible(idx) {
                        continue;
                    }

                    let pos = engine.store.pos(idx);
                    let size = engine.store.size(idx);

                    // Check if point is inside entity bounds
                    let half_size = size / 2.0;
                    let min = pos - half_size;
                    let max = pos + half_size;

                    if world_pos.x >= min.x
                        && world_pos.x <= max.x
                        && world_pos.y >= min.y
                        && world_pos.y <= max.y
                    {
                        // Found entity at position - create EntityId from index
                        let entity_id = archflow_core::EntityId::new(entity_idx);
                        engine.selected_entities.clear();
                        engine.selected_entities.push(entity_id);
                        break;
                    }
                }
            }
            1 => {
                // Pointer Move - update drag if active
                if !engine.selected_entities.is_empty() {
                    let world_delta = engine.screen_delta_to_world(event.x, event.y);

                    for entity_id in &engine.selected_entities {
                        let idx = archflow_core::EntityId::index(*entity_id);
                        let current_pos = engine.store.pos(idx.0 as usize);
                        engine
                            .store
                            .set_pos(idx.0 as usize, current_pos + world_delta);
                    }
                }
            }
            2 => {
                // Pointer Up - end drag
            }
            3 => {
                // Wheel - handled in JS side for camera zoom
            }
            _ => {}
        }
    }

    /// Spawn a new entity at the given position
    #[wasm_bindgen]
    pub fn spawn_entity(x: f32, y: f32, width: f32, height: f32) -> Result<u32, JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
                let id = engine.store.spawn(
                    archflow_core::Vec2::new(x, y),
                    archflow_core::Vec2::new(width, height),
                );

                // Set a random color
                let color = archflow_core::Color::rgb(
                    (js_sys::Math::random() * 255.0) as u8,
                    (js_sys::Math::random() * 255.0) as u8,
                    (js_sys::Math::random() * 255.0) as u8,
                );
                let idx = id.index().0 as usize;
                engine.store.colors[idx] = color.0;

                Ok(id.index().0)
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Move an entity by the given delta
    #[wasm_bindgen]
    pub fn move_entity(entity_index: u32, dx: f32, dy: f32) -> Result<(), JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
                use archflow_core::EntityId;
                use archflow_engine::Command;

                let id = EntityId::new(entity_index);
                let cmd = Command::Move {
                    id,
                    delta: archflow_core::Vec2::new(dx, dy),
                };
                engine.command_queue.push(cmd);
                Ok(())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Set the color of an entity
    #[wasm_bindgen]
    pub fn set_color(entity_index: u32, r: u8, g: u8, b: u8, a: u8) -> Result<(), JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
                use archflow_core::Color;
                use archflow_core::EntityId;
                use archflow_engine::Command;

                let id = EntityId::new(entity_index);
                let color = Color::rgba(r, g, b, a);
                let cmd = Command::SetColor { id, color: color.0 };
                engine.command_queue.push(cmd);
                Ok(())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Get the number of alive entities
    #[wasm_bindgen]
    pub fn entity_count() -> Result<u32, JsValue> {
        unsafe {
            if let Some(engine) = &ENGINE {
                Ok(engine.store.alive_count() as u32)
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Undo the last action
    #[wasm_bindgen]
    pub fn undo() -> Result<(), JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
                engine.undo();
                Ok(())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Redo the last undone action
    #[wasm_bindgen]
    pub fn redo() -> Result<(), JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
                engine.redo();
                Ok(())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Set the camera zoom level
    #[wasm_bindgen]
    pub fn set_zoom(zoom: f32) -> Result<(), JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
                engine.camera.zoom =
                    zoom.clamp(archflow_render::ZOOM_MIN, archflow_render::ZOOM_MAX);
                Ok(())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Get the current camera zoom level
    #[wasm_bindgen]
    pub fn get_zoom() -> Result<f32, JsValue> {
        unsafe {
            if let Some(engine) = &ENGINE {
                Ok(engine.camera.zoom)
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Set the camera center position
    #[wasm_bindgen]
    pub fn set_camera_center(x: f32, y: f32) -> Result<(), JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
                engine.camera.center = archflow_core::Vec2::new(x, y);
                Ok(())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Get the camera center position
    #[wasm_bindgen]
    pub fn get_camera_center() -> Result<js_sys::Array, JsValue> {
        unsafe {
            if let Some(engine) = &ENGINE {
                let array = js_sys::Array::new();
                array.push(&JsValue::from(engine.camera.center.x));
                array.push(&JsValue::from(engine.camera.center.y));
                Ok(array)
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Serialize the current project
    #[wasm_bindgen]
    pub fn serialize_project() -> Result<js_sys::Uint8Array, JsValue> {
        unsafe {
            if let Some(engine) = &ENGINE {
                use archflow_export::ProjectSerializer;

                let data = ProjectSerializer::serialize(&engine.store, &engine.connection_store);
                let array = unsafe { js_sys::Uint8Array::view(&data) };
                Ok(array)
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Clear all entities
    #[wasm_bindgen]
    pub fn clear() -> Result<(), JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
                engine.store = archflow_engine::EntityStore::new();
                engine.selected_entities.clear();
                Ok(())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }

    /// Set the shape type of an entity
    #[wasm_bindgen]
    pub fn set_shape(entity_index: u32, shape: u8) -> Result<(), JsValue> {
        unsafe {
            if let Some(engine) = &mut ENGINE {
                use archflow_core::EntityId;
                use archflow_engine::Command;

                let id = EntityId::new(entity_index);
                let cmd = Command::SetShape { id, shape };
                engine.command_queue.push(cmd);
                Ok(())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        }
    }
}

/// Custom error type for JavaScript
#[wasm_bindgen]
pub struct JsError {
    message: String,
}

#[wasm_bindgen]
impl JsError {
    #[wasm_bindgen(constructor)]
    pub fn new(message: &str) -> Self {
        Self {
            message: alloc::format!("{}", message),
        }
    }

    pub fn message(&self) -> String {
        self.message.clone()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_creation() {
        let bridge = WasmBridge::new();
        // Should not panic
    }

    #[test]
    fn test_js_error() {
        let error = JsError::new("Test error");
        assert_eq!(error.message(), "Test error");
    }

    #[test]
    fn test_input_buffer_size() {
        // The InputRingBuffer should be properly sized
        let expected_size = core::mem::size_of::<InputRingBuffer>();
        assert!(expected_size > 0);
        assert_eq!(expected_size, 4 + 4 + (32 * 128)); // head + tail + data
    }

    #[test]
    #[ignore = "Requires WASM target"]
    fn test_engine_not_initialized() {
        // When engine is not initialized, operations should fail
        let result = WasmBridge::entity_count();
        assert!(
            result.is_err(),
            "Should return error when engine not initialized"
        );
    }
}
