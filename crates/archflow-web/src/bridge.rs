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

use std::cell::RefCell;
use wasm_bindgen::prelude::*;

use crate::engine::ArchFlowEngine;

/// Global engine instance
thread_local! {
    static ENGINE: RefCell<Option<ArchFlowEngine>> = RefCell::new(None);
}

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
        let engine = ArchFlowEngine::new(canvas_width, canvas_height);
        ENGINE.set(Some(engine));
        Ok(())
    }

    /// Run one frame of the engine
    ///
    /// This should be called from requestAnimationFrame.
    #[wasm_bindgen]
    pub fn tick(timestamp: f64) -> Result<(), JsValue> {
        ENGINE.with(|e| {
            if let Some(engine) = e.borrow_mut().as_mut() {
                engine.tick(timestamp);
                Ok(())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        })
    }

    /// Spawn a new entity at the given position
    #[wasm_bindgen]
    pub fn spawn_entity(x: f32, y: f32, width: f32, height: f32) -> Result<u32, JsValue> {
        ENGINE.with(|e| {
            if let Some(engine) = e.borrow_mut().as_mut() {
                let id = engine.store.spawn(
                    archflow_core::Vec2::new(x, y),
                    archflow_core::Vec2::new(width, height),
                );
                Ok(id.index().0)
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        })
    }

    /// Move an entity by the given delta
    #[wasm_bindgen]
    pub fn move_entity(entity_index: u32, dx: f32, dy: f32) -> Result<(), JsValue> {
        ENGINE.with(|e| {
            if let Some(engine) = e.borrow_mut().as_mut() {
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
        })
    }

    /// Set the color of an entity
    #[wasm_bindgen]
    pub fn set_color(entity_index: u32, r: u8, g: u8, b: u8, a: u8) -> Result<(), JsValue> {
        ENGINE.with(|e| {
            if let Some(engine) = e.borrow_mut().as_mut() {
                use archflow_core::EntityId;
                use archflow_engine::Command;

                let id = EntityId::new(entity_index);
                let color = archflow_core::Color::rgba(r, g, b, a);
                let cmd = Command::SetColor { id, color: color.0 };
                engine.command_queue.push(cmd);
                Ok(())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        })
    }

    /// Get the number of alive entities
    #[wasm_bindgen]
    pub fn entity_count() -> Result<u32, JsValue> {
        ENGINE.with(|e| {
            if let Some(engine) = e.borrow().as_ref() {
                Ok(engine.store.alive_count() as u32)
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        })
    }

    /// Undo the last action
    #[wasm_bindgen]
    pub fn undo() -> Result<(), JsValue> {
        ENGINE.with(|e| {
            if let Some(engine) = e.borrow_mut().as_mut() {
                engine.undo();
                Ok(())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        })
    }

    /// Redo the last undone action
    #[wasm_bindgen]
    pub fn redo() -> Result<(), JsValue> {
        ENGINE.with(|e| {
            if let Some(engine) = e.borrow_mut().as_mut() {
                engine.redo();
                Ok(())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        })
    }

    /// Set the camera zoom level
    #[wasm_bindgen]
    pub fn set_zoom(zoom: f32) -> Result<(), JsValue> {
        ENGINE.with(|e| {
            if let Some(engine) = e.borrow_mut().as_mut() {
                engine.camera.zoom =
                    zoom.clamp(archflow_render::ZOOM_MIN, archflow_render::ZOOM_MAX);
                Ok(())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        })
    }

    /// Get the current camera zoom level
    #[wasm_bindgen]
    pub fn get_zoom() -> Result<f32, JsValue> {
        ENGINE.with(|e| {
            if let Some(engine) = e.borrow().as_ref() {
                Ok(engine.camera.zoom)
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        })
    }

    /// Set the camera center position
    #[wasm_bindgen]
    pub fn set_camera_center(x: f32, y: f32) -> Result<(), JsValue> {
        ENGINE.with(|e| {
            if let Some(engine) = e.borrow_mut().as_mut() {
                engine.camera.center = archflow_core::Vec2::new(x, y);
                Ok(())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        })
    }

    /// Get the camera center position
    #[wasm_bindgen]
    pub fn get_camera_center() -> Result<js_sys::Array, JsValue> {
        ENGINE.with(|e| {
            if let Some(engine) = e.borrow().as_ref() {
                let array = js_sys::Array::new();
                array.push(&JsValue::from(engine.camera.center.x));
                array.push(&JsValue::from(engine.camera.center.y));
                Ok(array)
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        })
    }

    /// Serialize the current project
    #[wasm_bindgen]
    pub fn serialize_project() -> Result<js_sys::Uint8Array, JsValue> {
        ENGINE.with(|e| {
            if let Some(engine) = e.borrow().as_ref() {
                use archflow_export::ProjectSerializer;

                let data = ProjectSerializer::serialize(&engine.store, &engine.connection_store);
                let array = unsafe { js_sys::Uint8Array::view(&data) };
                Ok(array)
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        })
    }

    /// Clear all entities
    #[wasm_bindgen]
    pub fn clear() -> Result<(), JsValue> {
        ENGINE.with(|e| {
            if let Some(engine) = e.borrow_mut().as_mut() {
                engine.store = archflow_engine::EntityStore::new();
                Ok(())
            } else {
                Err(JsError::new("Engine not initialized").into())
            }
        })
    }
}

// Custom error type for JavaScript
#[wasm_bindgen]
pub struct JsError {
    message: String,
}

#[wasm_bindgen]
impl JsError {
    #[wasm_bindgen(constructor)]
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
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
