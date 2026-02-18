// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow WASM Bridge - JS World (Entry Point)
//
// EPIC-WASM-100 - JsEntityBuilder Connection
// Provides: engine.world.spawn().insert(Component).build()
// Connects to actual ECS EntityStore
// ═══════════════════════════════════════════════════════════════════════════════

#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use wasm_bindgen::prelude::*;

use archflow_core::Vec2;
use archflow_engine::{Command, EntityStore};

use crate::js_entity_builder::JsEntityBuilder;

/// JS World - Entry point for entity creation in JavaScript
///
/// Usage:
/// ```javascript
/// const world = bridge.world;
/// const entity = world.spawn()
///     .insert(Transform.at(100, 200))
///     .insert(Shape.circle())
///     .insert(Color.rgb(255, 0, 0))
///     .insert(Visibility.visible())
///     .build();
/// ```
#[wasm_bindgen]
pub struct JsWorld {
    /// Reference to the entity store (for spawning)
    /// Note: In WASM, we use interior mutability via command queue
    bridge_ptr: u32,
}

#[wasm_bindgen]
impl JsWorld {
    /// Create a new JsWorld with pointer to entity store
    #[wasm_bindgen(constructor)]
    pub fn new(bridge_ptr: u32) -> Self {
        Self { bridge_ptr }
    }

    /// Spawn a new entity with the fluent builder API
    ///
    /// Returns a JsEntityBuilder that can be chained with .insert(Component)
    ///
    /// Example:
    /// ```javascript
    /// world.spawn()
    ///     .insert(Transform.at(100, 200))
    ///     .insert(Shape.circle())
    ///     .build();
    /// ```
    #[wasm_bindgen]
    pub fn spawn(&self) -> JsEntityBuilder {
        JsEntityBuilder::new(self.bridge_ptr)
    }

    /// Get stored entity store pointer
    #[wasm_bindgen]
    pub fn bridge_ptr(&self) -> u32 {
        self.bridge_ptr
    }
}

impl Default for JsWorld {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_creation() {
        let world = JsWorld::new(0);
        assert_eq!(world.bridge_ptr(), 0);
    }
}
