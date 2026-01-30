//! ArchFlow Web - Production-ready WASM module for browser application
//!
//! This module provides WebAssembly bindings for ArchFlow web application.

#![warn(missing_docs, rust_2018_idioms)]

use serde::Deserialize;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

// Use serde-wasm-bindgen for JsValue serialization
use serde_wasm_bindgen::from_value;

mod shapes;
mod state;

// Re-export managers from SDK
pub use archflow_sdk::wasm::alignment::JsAlignmentManager;
pub use archflow_sdk::wasm::group::JsGroupManager;
pub use archflow_sdk::wasm::library::JsLibraryManager;
pub use archflow_sdk::wasm::properties::JsPropertiesManager;

/// Main production WASM struct exposed to JavaScript
/// Uses ArchFlowEditor from SDK as the core editor
#[wasm_bindgen]
pub struct ArchFlowEditor {
    /// Inner SDK editor
    editor: archflow_sdk::wasm::ArchFlowEditor,
    /// Canvas rendering context
    context: CanvasRenderingContext2d,
    /// Canvas dimensions
    width: u32,
    height: u32,
}

/// Point with x and y coordinates
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Point {
    x: f32,
    y: f32,
}

/// Selection with shape IDs
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Selection {
    shape_ids: Vec<String>,
}

/// Shape representation
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Shape {
    id: String,
    shape_type: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    #[serde(default)]
    fill_color: Color,
    #[serde(default)]
    stroke_color: Option<Color>,
    #[serde(default)]
    stroke_width: f32,
    #[serde(default)]
    selected: bool,
}

/// Color representation
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
}

#[wasm_bindgen]
impl ArchFlowEditor {
    /// Creates a new editor instance with production initialization
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement) -> Result<ArchFlowEditor, JsValue> {
        // Initialize panic hook for better error messages
        console_error_panic_hook::set_once();

        // Get canvas context
        let context = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("Failed to get 2d context"))?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| JsValue::from_str("Failed to cast to CanvasRenderingContext2d"))?;

        let width = canvas.width();
        let height = canvas.height();

        // Create SDK editor
        let editor = archflow_sdk::wasm::ArchFlowEditor::new(width as f32, height as f32);

        Ok(ArchFlowEditor {
            editor,
            context,
            width,
            height,
        })
    }

    /// Resizes the canvas
    #[wasm_bindgen]
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    // === Shape Operations ===

    #[wasm_bindgen]
    pub fn add_rect(&mut self, x: f64, y: f64, width: f64, height: f64) -> String {
        self.editor
            .create_rectangle(x as f32, y as f32, width as f32, height as f32)
    }

    #[wasm_bindgen]
    pub fn add_ellipse(&mut self, x: f64, y: f64, radius_x: f64, radius_y: f64) -> String {
        self.editor
            .create_ellipse(x as f32, y as f32, radius_x as f32, radius_y as f32)
    }

    #[wasm_bindgen]
    pub fn add_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) -> String {
        self.editor
            .create_line(x1 as f32, y1 as f32, x2 as f32, y2 as f32)
    }

    #[wasm_bindgen]
    pub fn delete_shape(&mut self, id: &str) -> bool {
        self.editor.delete_shape(id)
    }

    #[wasm_bindgen]
    pub fn clear_selection(&mut self) {
        self.editor.clear_selection();
    }

    #[wasm_bindgen]
    pub fn select(&mut self, id: &str) {
        self.editor.select(id);
    }

    #[wasm_bindgen]
    pub fn get_shape(&self, id: &str) -> JsValue {
        self.editor.get_shape(id)
    }

    #[wasm_bindgen]
    pub fn get_all_shapes(&self) -> JsValue {
        self.editor.get_all_shapes()
    }

    #[wasm_bindgen]
    pub fn get_selection(&self) -> JsValue {
        self.editor.get_selection()
    }

    // === Viewport Operations ===

    #[wasm_bindgen]
    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.editor.pan(dx as f32, dy as f32);
    }

    #[wasm_bindgen]
    pub fn zoom_at(&mut self, x: f64, y: f64, factor: f64) {
        self.editor.zoom_at(x as f32, y as f32, factor as f32);
    }

    #[wasm_bindgen]
    pub fn zoom_to_fit(&mut self) {
        self.editor.zoom_to_fit();
    }

    #[wasm_bindgen]
    pub fn screen_to_canvas(&self, x: f32, y: f32) -> JsValue {
        self.editor.screen_to_canvas(x, y)
    }

    // === Grid Configuration ===

    #[wasm_bindgen]
    pub fn get_grid_config(&self) -> JsValue {
        self.editor.get_grid_config()
    }

    #[wasm_bindgen]
    pub fn set_grid_config(&self, config: JsValue) {
        self.editor.set_grid_config(config);
    }

    // === C4 Level ===

    #[wasm_bindgen]
    pub fn get_c4_level(&self) -> String {
        self.editor.get_c4_level()
    }

    #[wasm_bindgen]
    pub fn set_c4_level(&self, level: &str) {
        self.editor.set_c4_level(level);
    }

    // === Rendering ===

    #[wasm_bindgen]
    pub fn render(&mut self) {
        // Clear canvas
        self.context.set_fill_style(&JsValue::from_str("#1e1e1e"));
        let _ = self
            .context
            .fill_rect(0.0, 0.0, self.width as f64, self.height as f64);

        // Render all shapes
        let shapes = self.editor.get_all_shapes();
        if let Ok(js_shapes) = from_value::<Vec<Shape>>(shapes) {
            for shape in js_shapes {
                self.render_shape(&shape);
            }
        }
    }

    // === SDK Accessors ===

    /// Gets the underlying SDK editor
    #[wasm_bindgen]
    pub fn get_editor(&self) -> *const archflow_sdk::wasm::ArchFlowEditor {
        &self.editor as *const _
    }

    /// Gets the library manager
    #[wasm_bindgen]
    pub fn get_library_manager(&self) -> JsLibraryManager {
        JsLibraryManager::new()
    }

    /// Gets the properties manager
    #[wasm_bindgen]
    pub fn get_properties_manager(&self) -> JsPropertiesManager {
        JsPropertiesManager::new()
    }

    /// Gets the alignment manager
    #[wasm_bindgen]
    pub fn get_alignment_manager(&self) -> JsAlignmentManager {
        JsAlignmentManager::new()
    }

    /// Gets the group manager
    #[wasm_bindgen]
    pub fn get_group_manager(&self) -> JsGroupManager {
        JsGroupManager::new()
    }
}

// Private rendering methods
impl ArchFlowEditor {
    /// Renders a single shape
    fn render_shape(&self, shape: &Shape) {
        let fill_color = format!(
            "rgba({},{},{},{})",
            (shape.fill_color.r * 255.0) as u8,
            (shape.fill_color.g * 255.0) as u8,
            (shape.fill_color.b * 255.0) as u8,
            shape.fill_color.a
        );

        self.context.set_fill_style(&JsValue::from_str(&fill_color));

        if let Some(stroke) = &shape.stroke_color {
            let stroke_color = format!(
                "rgba({},{},{},{})",
                (stroke.r * 255.0) as u8,
                (stroke.g * 255.0) as u8,
                (stroke.b * 255.0) as u8,
                stroke.a
            );
            self.context
                .set_stroke_style(&JsValue::from_str(&stroke_color));
            self.context.set_line_width(shape.stroke_width as f64);
        }

        let path = match web_sys::Path2d::new() {
            Ok(p) => p,
            Err(_) => return,
        };

        match shape.shape_type.as_str() {
            "Rectangle" | "Rect" => {
                let _ = path.rect(
                    shape.x as f64,
                    shape.y as f64,
                    shape.width as f64,
                    shape.height as f64,
                );
            }
            "Ellipse" | "Circle" => {
                let rx = shape.width as f64 / 2.0;
                let ry = shape.height as f64 / 2.0;
                let cx = shape.x as f64 + rx;
                let cy = shape.y as f64 + ry;
                let _ = path.ellipse(cx, cy, rx, ry, 0.0, 0.0, std::f64::consts::PI * 2.0);
            }
            "Line" => {
                let _ = path.move_to(shape.x as f64, shape.y as f64);
                let _ = path.line_to(
                    (shape.x + shape.width) as f64,
                    (shape.y + shape.height) as f64,
                );
            }
            _ => {
                let _ = path.rect(
                    shape.x as f64,
                    shape.y as f64,
                    shape.width as f64,
                    shape.height as f64,
                );
            }
        }

        let _ = self.context.fill_with_path_2d(&path);
        if shape.stroke_color.is_some() {
            let _ = self.context.stroke_with_path(&path);
        }
    }
}

/// Backward compatibility type alias
pub type ArchFlowDemo = ArchFlowEditor;

/// Initialize function for backward compatibility
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: This test can only run on wasm32 target
    // Run with: wasm-pack test --firefox --chrome
    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_editor_creation() {
        // Test that we can at least create the editor struct
        let editor = archflow_sdk::wasm::ArchFlowEditor::new(800.0, 600.0);
        assert_eq!(
            from_value::<Dimensions>(editor.get_dimensions())
                .unwrap()
                .width,
            800.0
        );
    }

    #[derive(Deserialize)]
    struct Dimensions {
        width: f32,
        height: f32,
    }
}
