// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Web - BrickHandle WASM Binding
//
// Epic 5.x: Fluent Bricks API - BrickHandle para control runtime
//
// Handle a registered brick chain that can be controlled at runtime.
// ═══════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]

use alloc::string::String;
use wasm_bindgen::prelude::*;

/// Handle to a registered brick chain
///
/// Returned by BrickChainBuilder.connect() for runtime control.
///
/// # JavaScript Example
/// ```javascript
/// const handle = builder.connect();
/// handle.disable();
/// handle.remove();
/// ```
#[wasm_bindgen]
pub struct BrickHandle {
    /// Unique identifier for the brick chain
    id: String,
}

#[wasm_bindgen]
impl BrickHandle {
    /// Creates a new BrickHandle with the given ID
    #[wasm_bindgen(constructor)]
    pub fn new(id: String) -> Self {
        Self { id }
    }

    /// Get the brick chain ID
    #[wasm_bindgen]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    /// Enable the brick chain
    #[wasm_bindgen]
    pub fn enable(&mut self) {
        // Placeholder - would toggle enabled flag in mapping table
    }

    /// Disable the brick chain
    #[wasm_bindgen]
    pub fn disable(&mut self) {
        // Placeholder
    }

    /// Check if enabled
    #[wasm_bindgen]
    pub fn is_enabled(&self) -> bool {
        true // Assume enabled by default
    }

    /// Remove the brick chain
    #[wasm_bindgen]
    pub fn remove(self) {
        // Placeholder - would remove from mapping table
    }

    /// Toggle enabled state
    #[wasm_bindgen]
    pub fn toggle(&mut self) -> bool {
        self.is_enabled()
    }
}

impl Default for BrickHandle {
    fn default() -> Self {
        Self::new(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brick_handle_creation() {
        let handle = BrickHandle::new(String::from("test"));
        assert_eq!(handle.id(), "test");
    }

    #[test]
    fn test_brick_handle_default() {
        let handle = BrickHandle::default();
        assert!(handle.id().is_empty());
    }
}
