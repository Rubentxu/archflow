//! ArchFlow WASM - Bindings para JavaScript
//!
//! Este crate expone el engine a JavaScript/WebAssembly

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ArchFlowEngine {
    document: archflow_workspace::Document,
}

#[wasm_bindgen]
impl ArchFlowEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<ArchFlowEngine, JsValue> {
        Ok(Self {
            document: archflow_workspace::Document::new(),
        })
    }

    #[wasm_bindgen]
    pub fn create_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let id = archflow_core::EntityId::new();
        self.document.create_entity(id);
        let _ = (x, y, width, height);
    }

    #[wasm_bindgen]
    pub fn undo(&mut self) {
        let _ = self.document.undo();
    }

    #[wasm_bindgen]
    pub fn redo(&mut self) {
        let _ = self.document.redo();
    }
}
