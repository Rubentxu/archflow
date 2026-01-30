//! WASM bindings for Component Library System

use crate::library::{
    ComponentData, ComponentGeometry, ComponentLibrary, ComponentStyle, ItemPreview,
    LibraryCategory, LibraryItem, LibraryShapeType, manager::LibraryManager,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use wasm_bindgen::prelude::*;

/// WASM wrapper for LibraryManager
#[wasm_bindgen]
pub struct JsLibraryManager {
    inner: LibraryManager,
}

#[wasm_bindgen]
impl JsLibraryManager {
    /// Creates a new library manager with built-in libraries
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: LibraryManager::new(),
        }
    }

    /// Gets all libraries as JSON string
    #[wasm_bindgen]
    pub fn get_libraries(&self) -> Result<String, JsValue> {
        let libraries: Vec<&ComponentLibrary> = self.inner.get_all_libraries();
        serde_json::to_string(&libraries)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Gets active libraries as JSON string
    #[wasm_bindgen]
    pub fn get_active_libraries(&self) -> Result<String, JsValue> {
        let libraries: Vec<&ComponentLibrary> = self.inner.get_active_libraries();
        serde_json::to_string(&libraries)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Gets a specific library by ID
    #[wasm_bindgen]
    pub fn get_library(&self, library_id: &str) -> Result<String, JsValue> {
        match self.inner.get_library(library_id) {
            Some(library) => serde_json::to_string(library)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e))),
            None => Err(JsValue::from_str(&format!(
                "Library not found: {}",
                library_id
            ))),
        }
    }

    /// Searches for items across all libraries
    #[wasm_bindgen]
    pub fn search_items(&self, query: &str) -> Result<String, JsValue> {
        let results = self.inner.search_items(query);
        let items: Vec<(String, &LibraryItem)> = results
            .into_iter()
            .map(|(lib, item)| (lib.id.clone(), item))
            .collect();

        serde_json::to_string(&items)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Gets a specific item from a library
    #[wasm_bindgen]
    pub fn get_item(&self, library_id: &str, item_id: &str) -> Result<String, JsValue> {
        match self.inner.get_item(library_id, item_id) {
            Ok(item) => serde_json::to_string(item)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e))),
            Err(e) => Err(JsValue::from_str(&format!("Item not found: {}", e))),
        }
    }

    /// Gets component data for instantiation
    #[wasm_bindgen]
    pub fn get_component_data(&self, library_id: &str, item_id: &str) -> Result<String, JsValue> {
        match self.inner.get_item(library_id, item_id) {
            Ok(item) => serde_json::to_string(&item.data)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e))),
            Err(e) => Err(JsValue::from_str(&format!("Item not found: {}", e))),
        }
    }

    /// Adds an item to recent items
    #[wasm_bindgen]
    pub fn add_to_recent(&mut self, library_id: &str, item_id: &str) {
        self.inner.add_to_recent(library_id, item_id);
    }

    /// Gets recent items
    #[wasm_bindgen]
    pub fn get_recent_items(&self) -> Result<String, JsValue> {
        let recent = self.inner.get_recent_items();
        let items: Vec<(String, &LibraryItem)> = recent
            .into_iter()
            .map(|(lib, item)| (lib.id.clone(), item))
            .collect();

        serde_json::to_string(&items)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Adds an item to favorites
    #[wasm_bindgen]
    pub fn add_to_favorites(&mut self, library_id: &str, item_id: &str) {
        self.inner.add_to_favorites(library_id, item_id);
    }

    /// Removes an item from favorites
    #[wasm_bindgen]
    pub fn remove_from_favorites(&mut self, library_id: &str, item_id: &str) {
        self.inner.remove_from_favorites(library_id, item_id);
    }

    /// Gets favorited items
    #[wasm_bindgen]
    pub fn get_favorites(&self) -> Result<String, JsValue> {
        let favorites = self.inner.get_favorites();
        let items: Vec<(String, &LibraryItem)> = favorites
            .into_iter()
            .map(|(lib, item)| (lib.id.clone(), item))
            .collect();

        serde_json::to_string(&items)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Imports a library from JSON string
    #[wasm_bindgen]
    pub fn import_library(&mut self, json: &str) -> Result<(), JsValue> {
        match self.inner.import_library(json) {
            Ok(_) => Ok(()),
            Err(e) => Err(JsValue::from_str(&format!("Import error: {}", e))),
        }
    }

    /// Exports a library to JSON string
    #[wasm_bindgen]
    pub fn export_library(&self, library_id: &str) -> Result<String, JsValue> {
        match self.inner.export_library(library_id) {
            Ok(json) => Ok(json),
            Err(e) => Err(JsValue::from_str(&format!("Export error: {}", e))),
        }
    }
}

impl Default for JsLibraryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// JavaScript-compatible library item info
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "JsLibraryItemInfo")]
pub struct JsLibraryItemInfo {
    pub library_id: String,
    pub library_name: String,
    pub category_id: String,
    pub category_name: String,
    pub item_id: String,
    pub item_name: String,
    pub preview: ItemPreview,
}

impl JsLibraryItemInfo {
    /// Creates info from library, category, and item
    pub fn new(library: &ComponentLibrary, category: &LibraryCategory, item: &LibraryItem) -> Self {
        Self {
            library_id: library.id.clone(),
            library_name: library.name.clone(),
            category_id: category.id.clone(),
            category_name: category.name.clone(),
            item_id: item.id.clone(),
            item_name: item.name.clone(),
            preview: item.preview.clone(),
        }
    }
}

/// JavaScript-compatible shape creation data
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "JsShapeCreationData")]
pub struct JsShapeCreationData {
    pub shape_type: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub fill_color: Option<String>,
    pub stroke_color: Option<String>,
    pub stroke_width: Option<f32>,
    pub opacity: Option<f32>,
}

impl From<&ComponentData> for JsShapeCreationData {
    fn from(data: &ComponentData) -> Self {
        let shape_type = match &data.shape_type {
            LibraryShapeType::Rectangle => "rectangle",
            LibraryShapeType::RoundedRectangle { .. } => "roundedRectangle",
            LibraryShapeType::Circle => "circle",
            LibraryShapeType::Ellipse => "ellipse",
            LibraryShapeType::Diamond => "diamond",
            LibraryShapeType::Triangle => "triangle",
            LibraryShapeType::Hexagon => "hexagon",
            LibraryShapeType::Cylinder => "cylinder",
            LibraryShapeType::Cloud => "cloud",
            LibraryShapeType::Document => "document",
            LibraryShapeType::Line => "line",
            LibraryShapeType::CustomPath { .. } => "custom",
        }
        .to_string();

        Self {
            shape_type,
            x: data.geometry.default_x.unwrap_or(0.0),
            y: data.geometry.default_y.unwrap_or(0.0),
            width: data.geometry.width,
            height: data.geometry.height,
            fill_color: data.style.fill_color.clone(),
            stroke_color: data.style.stroke_color.clone(),
            stroke_width: data.style.stroke_width,
            opacity: data.style.opacity,
        }
    }
}

/// WASM function to create shape data from component data JSON
#[wasm_bindgen]
pub fn create_shape_data_from_component(json: &str) -> Result<String, JsValue> {
    let component_data: ComponentData = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    let shape_data = JsShapeCreationData::from(&component_data);

    serde_json::to_string(&shape_data)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Gets the default libraries JSON
#[wasm_bindgen]
pub fn get_default_libraries() -> Result<String, JsValue> {
    let manager = LibraryManager::new();
    let libraries: Vec<&ComponentLibrary> = manager.get_all_libraries();

    serde_json::to_string(&libraries)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Gets library categories as a flat list
#[wasm_bindgen]
pub fn get_all_categories() -> Result<String, JsValue> {
    let manager = LibraryManager::new();

    let mut categories = Vec::new();
    for library in manager.get_all_libraries() {
        for category in &library.categories {
            categories.push(serde_json::json!({
                "libraryId": library.id,
                "libraryName": library.name,
                "categoryId": category.id,
                "categoryName": category.name,
                "icon": category.icon,
                "itemCount": category.items.len(),
            }));
        }
    }

    serde_json::to_string(&categories)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_library_manager_new() {
        let manager = JsLibraryManager::new();
        let result = manager.get_libraries();
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("general"));
        assert!(json.contains("flowchart"));
    }

    #[test]
    fn test_js_library_manager_search() {
        let manager = JsLibraryManager::new();
        let result = manager.search_items("rect");
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("Rectangle"));
    }

    #[test]
    fn test_create_shape_data() {
        let component = ComponentData::new(LibraryShapeType::Rectangle)
            .with_geometry(ComponentGeometry::new(120.0, 80.0))
            .with_style(
                ComponentStyle::default()
                    .with_fill_color("#3366cc")
                    .with_stroke_color("#ffffff")
                    .with_stroke_width(1.0),
            );

        let json = serde_json::to_string(&component).unwrap();
        let result = create_shape_data_from_component(&json);

        assert!(result.is_ok());
        let shape_json = result.unwrap();
        assert!(shape_json.contains("rectangle"));
        assert!(shape_json.contains("3366cc"));
    }

    #[test]
    fn test_js_shape_creation_data_from_component() {
        let component = ComponentData::new(LibraryShapeType::Circle)
            .with_geometry(ComponentGeometry::new(80.0, 80.0))
            .with_style(ComponentStyle::default().with_fill_color("#ff0000"));

        let data = JsShapeCreationData::from(&component);

        assert_eq!(data.shape_type, "circle");
        assert_eq!(data.width, 80.0);
        assert_eq!(data.height, 80.0);
        assert_eq!(data.fill_color, Some("#ff0000".to_string()));
    }
}
