//! WASM bindings for Layer System

use crate::EntityId;
use crate::layers::{C4Level, Layer, LayerManager};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use wasm_bindgen::prelude::*;

/// WASM wrapper for LayerManager
#[wasm_bindgen]
pub struct JsLayerManager {
    inner: LayerManager,
}

#[wasm_bindgen]
impl JsLayerManager {
    /// Creates a new layer manager
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: LayerManager::new(),
        }
    }

    /// Creates a new layer
    #[wasm_bindgen]
    pub fn create_layer(&mut self, c4_level: &str, name: &str) -> String {
        let level = match c4_level {
            "Context" => C4Level::Context,
            "Container" => C4Level::Container,
            "Component" => C4Level::Component,
            "Code" => C4Level::Code,
            _ => C4Level::Context,
        };
        let id = self.inner.create_layer(level, name.to_string());
        id.to_string()
    }

    /// Gets a layer as JSON
    #[wasm_bindgen]
    pub fn get_layer(&self, layer_id: &str) -> Result<String, JsValue> {
        if let Some(id) = EntityId::from_str(layer_id) {
            if let Some(layer) = self.inner.get_layer(id) {
                serde_json::to_string(layer)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
            } else {
                Err(JsValue::from_str("Layer not found"))
            }
        } else {
            Err(JsValue::from_str("Invalid layer ID"))
        }
    }

    /// Gets all layers as JSON
    #[wasm_bindgen]
    pub fn get_all_layers(&self) -> Result<String, JsValue> {
        let layers: Vec<&Layer> = self.inner.all_layers();
        serde_json::to_string(&layers)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Gets layers in z-order
    #[wasm_bindgen]
    pub fn get_layers_in_order(&self) -> Result<String, JsValue> {
        let layers: Vec<&Layer> = self.inner.get_layers_in_order();
        serde_json::to_string(&layers)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Sets layer visibility
    #[wasm_bindgen]
    pub fn set_layer_visibility(&mut self, layer_id: &str, visible: bool) -> bool {
        if let Some(id) = EntityId::from_str(layer_id) {
            self.inner.set_layer_visibility(id, visible)
        } else {
            false
        }
    }

    /// Sets layer locked state
    #[wasm_bindgen]
    pub fn set_layer_locked(&mut self, layer_id: &str, locked: bool) -> bool {
        if let Some(id) = EntityId::from_str(layer_id) {
            self.inner.set_layer_locked(id, locked)
        } else {
            false
        }
    }

    /// Sets layer opacity
    #[wasm_bindgen]
    pub fn set_layer_opacity(&mut self, layer_id: &str, opacity: f32) -> bool {
        if let Some(id) = EntityId::from_str(layer_id) {
            self.inner.set_layer_opacity(id, opacity)
        } else {
            false
        }
    }

    /// Deletes a layer
    #[wasm_bindgen]
    pub fn delete_layer(&mut self, layer_id: &str) -> bool {
        if let Some(id) = EntityId::from_str(layer_id) {
            self.inner.delete_layer(id)
        } else {
            false
        }
    }

    /// Moves a layer up
    #[wasm_bindgen]
    pub fn move_layer_up(&mut self, layer_id: &str) -> bool {
        if let Some(id) = EntityId::from_str(layer_id) {
            self.inner.move_layer_up(id)
        } else {
            false
        }
    }

    /// Moves a layer down
    #[wasm_bindgen]
    pub fn move_layer_down(&mut self, layer_id: &str) -> bool {
        if let Some(id) = EntityId::from_str(layer_id) {
            self.inner.move_layer_down(id)
        } else {
            false
        }
    }

    /// Moves a layer to the top
    #[wasm_bindgen]
    pub fn move_layer_to_top(&mut self, layer_id: &str) -> bool {
        if let Some(id) = EntityId::from_str(layer_id) {
            self.inner.move_layer_to_top(id)
        } else {
            false
        }
    }

    /// Moves a layer to the bottom
    #[wasm_bindgen]
    pub fn move_layer_to_bottom(&mut self, layer_id: &str) -> bool {
        if let Some(id) = EntityId::from_str(layer_id) {
            self.inner.move_layer_to_bottom(id)
        } else {
            false
        }
    }

    /// Renames a layer
    #[wasm_bindgen]
    pub fn rename_layer(&mut self, layer_id: &str, name: &str) -> bool {
        if let Some(id) = EntityId::from_str(layer_id) {
            self.inner.rename_layer(id, name.to_string())
        } else {
            false
        }
    }

    /// Gets the current C4 level
    #[wasm_bindgen]
    pub fn get_current_level(&self) -> String {
        format!("{:?}", self.inner.current_level())
    }

    /// Sets the current C4 level
    #[wasm_bindgen]
    pub fn set_current_level(&mut self, level: &str) {
        let c4_level = match level {
            "Context" => C4Level::Context,
            "Container" => C4Level::Container,
            "Component" => C4Level::Component,
            "Code" => C4Level::Code,
            _ => C4Level::Context,
        };
        self.inner.set_current_level(c4_level);
    }

    /// Gets layers for a specific C4 level
    #[wasm_bindgen]
    pub fn get_layers_for_level(&self, level: &str) -> Result<String, JsValue> {
        let c4_level = match level {
            "Context" => C4Level::Context,
            "Container" => C4Level::Container,
            "Component" => C4Level::Component,
            "Code" => C4Level::Code,
            _ => C4Level::Context,
        };
        let layers: Vec<&Layer> = self.inner.get_layers_for_level(c4_level);
        serde_json::to_string(&layers)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Gets the number of layers
    #[wasm_bindgen]
    pub fn layer_count(&self) -> usize {
        self.inner.layer_count()
    }

    /// Gets layer count for current level
    #[wasm_bindgen]
    pub fn get_layer_count_for_current_level(&self) -> usize {
        self.inner
            .get_layers_for_level(self.inner.current_level())
            .len()
    }
}

impl Default for JsLayerManager {
    fn default() -> Self {
        Self::new()
    }
}

/// JavaScript-compatible layer info
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "JsLayerInfo")]
pub struct JsLayerInfo {
    pub id: String,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
    pub z_order: i32,
    pub shape_count: usize,
    pub c4_level: String,
}

impl From<&Layer> for JsLayerInfo {
    fn from(layer: &Layer) -> Self {
        Self {
            id: layer.id.to_string(),
            name: layer.name.clone(),
            visible: layer.visible,
            locked: layer.locked,
            opacity: layer.opacity,
            z_order: layer.z_order,
            shape_count: layer.shapes.len(),
            c4_level: format!("{:?}", layer.c4_level),
        }
    }
}

/// Gets all C4 levels as JSON
#[wasm_bindgen]
pub fn get_c4_levels() -> Result<String, JsValue> {
    let levels = vec![
        serde_json::json!({
            "id": "context",
            "name": "Context",
            "description": "System context showing users and external systems",
            "defaultZoom": C4Level::Context.default_zoom()
        }),
        serde_json::json!({
            "id": "container",
            "name": "Container",
            "description": "Container diagram showing applications and databases",
            "defaultZoom": C4Level::Container.default_zoom()
        }),
        serde_json::json!({
            "id": "component",
            "name": "Component",
            "description": "Component diagram showing internal structure",
            "defaultZoom": C4Level::Component.default_zoom()
        }),
        serde_json::json!({
            "id": "code",
            "name": "Code",
            "description": "Class and code-level details",
            "defaultZoom": C4Level::Code.default_zoom()
        }),
    ];

    serde_json::to_string(&levels)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_layer_manager_new() {
        let manager = JsLayerManager::new();
        assert_eq!(manager.layer_count(), 0);
    }

    #[test]
    fn test_js_layer_manager_create() {
        let mut manager = JsLayerManager::new();
        let id = manager.create_layer("Context", "Test Layer");
        assert!(!id.is_empty());
        assert_eq!(manager.layer_count(), 1);
    }

    #[test]
    fn test_js_layer_visibility() {
        let mut manager = JsLayerManager::new();
        let id = manager.create_layer("Context", "Test Layer");

        assert!(manager.set_layer_visibility(&id, false));
        let layer_json = manager.get_layer(&id).unwrap();
        assert!(layer_json.contains("\"visible\":false"));

        assert!(manager.set_layer_visibility(&id, true));
    }

    #[test]
    fn test_js_layer_locked() {
        let mut manager = JsLayerManager::new();
        let id = manager.create_layer("Context", "Test Layer");

        assert!(manager.set_layer_locked(&id, true));
        assert!(manager.set_layer_locked(&id, false));
    }

    #[test]
    fn test_js_layer_reorder() {
        let mut manager = JsLayerManager::new();
        let id1 = manager.create_layer("Context", "Layer 1");
        let id2 = manager.create_layer("Context", "Layer 2");

        assert!(manager.move_layer_to_top(&id1));
        let layers = manager.get_layers_in_order().unwrap();
        assert!(layers.contains(&id2));
        assert!(layers.contains(&id1));
    }

    #[test]
    fn test_js_layer_rename() {
        let mut manager = JsLayerManager::new();
        let id = manager.create_layer("Context", "Original");

        assert!(manager.rename_layer(&id, "Renamed"));
        let layer = manager.get_layer(&id).unwrap();
        assert!(layer.contains("Renamed"));
    }

    #[test]
    fn test_js_layer_delete() {
        let mut manager = JsLayerManager::new();
        let id = manager.create_layer("Context", "Test Layer");
        assert_eq!(manager.layer_count(), 1);

        assert!(manager.delete_layer(&id));
        assert_eq!(manager.layer_count(), 0);
    }
}
