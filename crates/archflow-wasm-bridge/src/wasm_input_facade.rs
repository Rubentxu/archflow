// ═══════════════════════════════════════════════════════════════════════════════════════
// ArchFlow WASM Bridge - Input Facade
//
// This module provides input handling operations (mouse, keyboard, touch).
// Following the Single Responsibility Principle (SRP) - one facade per domain.
//
// Architecture: EPIC-WASM-103 - Bridge Refactor for SRP
// ═══════════════════════════════════════════════════════════════════════════════════════

#![no_std]

extern crate alloc;

use alloc::string::String;
use js_sys::Array;
use wasm_bindgen::prelude::*;

/// Input Facade - handles user input operations
///
/// Provides a clean interface for input event handling:
/// - Mouse events (click, hover, drag)
/// - Keyboard events
/// - Touch events
#[wasm_bindgen]
pub struct WasmInputFacade {
    /// Reference to the engine (for internal use)
    engine_ptr: u32,
}

#[wasm_bindgen]
impl WasmInputFacade {
    /// Create a new Input Facade
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { engine_ptr: 0 }
    }

    /// Process a mouse click event
    #[wasm_bindgen]
    pub fn on_mouse_click(&self, _x: f32, _y: f32, _button: u8) -> bool {
        false
    }

    /// Process a mouse hover event
    #[wasm_bindgen]
    pub fn on_mouse_hover(&self, _x: f32, _y: f32) -> bool {
        false
    }

    /// Process a keyboard event
    #[wasm_bindgen]
    pub fn on_key_down(&self, _key_code: u32) -> bool {
        false
    }

    /// Get current pointer position
    #[wasm_bindgen]
    pub fn get_pointer_position(&self) -> Result<Array, JsValue> {
        let array = Array::new();
        array.push(&JsValue::from(0.0_f32));
        array.push(&JsValue::from(0.0_f32));
        Ok(array)
    }
}

impl Default for WasmInputFacade {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_facade_creation() {
        let facade = WasmInputFacade::new();
        // Note: get_pointer_position returns Result<Array, JsValue> which requires wasm
        let _ = facade.on_mouse_click(0.0, 0.0, 0); // Stub returns false, just verify it can be called
    }
}
