//! ArchFlow Web - Production-ready WASM module for browser application
//!
//! This module provides WebAssembly bindings for ArchFlow web application.
//! Uses Canvas 2D API via web-sys for rendering with production-ready patterns.

use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

mod shapes;
mod state;

#[cfg(test)]
mod tests;

pub use shapes::{RemoteCursor, Shape, ShapeId, ShapeStore, ShapeType};
pub use state::{EditorState, InteractionState, Rect, Tool};

// Re-export library manager from SDK
pub use archflow_sdk::wasm::library::JsLibraryManager;
pub use archflow_sdk::wasm::properties::JsPropertiesManager;
pub use archflow_sdk::wasm::alignment::JsAlignmentManager;
pub use archflow_sdk::wasm::group::JsGroupManager;

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
    /// Cached grid off-screen canvas
    grid_canvas: Option<web_sys::OffscreenCanvas>,
}

#[wasm_bindgen]
impl ArchFlowEditor {
    /// Creates a new editor instance with production initialization
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: web_sys::HtmlCanvasElement) -> Result<ArchFlowEditor, JsValue> {
        // Initialize panic hook for better error messages
        console_error_panic_hook::set_once();

        // Initialize logging
        console_log::init_with_level(log::Level::Info).map_err(|_| "Failed to init logging")?;

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

        // Log initialization
        log::info!("ArchFlowEditor initialized with dimensions {}x{}", width, height);

        Ok(ArchFlowEditor {
            editor,
            context,
            width,
            height,
            grid_canvas: None,
        })
    }

    /// Resizes the canvas and updates viewport
    #[wasm_bindgen]
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        // Invalidate cached grid
        self.grid_canvas = None;
    }

    // === Tool Management (kept for compatibility) ===

    /// Sets the current tool (legacy method)
    #[wasm_bindgen]
    pub fn set_tool(&mut self, _tool: &str) {
        // Legacy - tools are now managed by SDK
        log::debug!("Tool setting is now handled by SDK");
    }

    // === Input Handling (delegates to SDK) ===

    #[wasm_bindgen]
    pub fn on_mousedown(&mut self, x: f64, y: f64, button: u16) {
        // Convert screen to canvas coordinates
        let canvas_point = self.editor.screen_to_canvas(x as f32, y as f32);
        let point = archflow_core::Vec2::new(canvas_point.x, canvas_point.y);
        self.editor.canvas.borrow_mut().on_mousedown(point, button);
    }

    #[wasm_bindgen]
    pub fn on_mousemove(&mut self, x: f64, y: f64) {
        let canvas_point = self.editor.screen_to_canvas(x as f32, y as f32);
        let point = archflow_core::Vec2::new(canvas_point.x, canvas_point.y);
        self.editor.canvas.borrow_mut().on_mousemove(point);
    }

    #[wasm_bindgen]
    pub fn on_mouseup(&mut self, x: f64, y: f64) {
        let canvas_point = self.editor.screen_to_canvas(x as f32, y as f32);
        let point = archflow_core::Vec2::new(canvas_point.x, canvas_point.y);
        self.editor.canvas.borrow_mut().on_mouseup(point);
    }

    #[wasm_bindgen]
    pub fn on_wheel(&mut self, x: f64, y: f64, zoom_out: bool) {
        let factor = if zoom_out { 0.9 } else { 1.1 };
        self.editor.zoom_at(x as f32, y as f32, factor);
    }

    // === Keyboard Events ===

    /// Handles key down event with full keyboard support
    #[wasm_bindgen]
    pub fn on_keydown(&mut self, key: &str, shift: bool, ctrl: bool) {
        // Use SDK's keyboard handler
        self.editor.handle_keydown(key, shift, ctrl);
    }

    /// Handles key up event
    #[wasm_bindgen]
    pub fn on_keyup(&mut self, key: &str) {
        self.editor.handle_keyup(key);
    }

    // === Shape Operations (delegates to SDK) ===

    #[wasm_bindgen]
    pub fn add_rect(&mut self, x: f64, y: f64, width: f64, height: f64) -> String {
        self.editor.create_rectangle(x as f32, y as f32, width as f32, height as f32)
    }

    #[wasm_bindgen]
    pub fn add_ellipse(&mut self, x: f64, y: f64, radius_x: f64, radius_y: f64) -> String {
        self.editor.create_ellipse(x as f32, y as f32, radius_x as f32, radius_y as f32)
    }

    #[wasm_bindgen]
    pub fn add_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) -> String {
        self.editor.create_line(x1 as f32, y1 as f32, x2 as f32, y2 as f32)
    }

    #[wasm_bindgen]
    pub fn delete_selected(&mut self) {
        let selection = self.editor.get_selection();
        for shape_id in selection.shape_ids {
            self.editor.delete_shape(&shape_id);
        }
    }

    #[wasm_bindgen]
    pub fn clear(&mut self) {
        let shapes = self.editor.get_all_shapes();
        for shape in shapes.into_serde::<Vec<archflow_sdk::wasm::JsShape>>().unwrap_or_default() {
            self.editor.delete_shape(&shape.id);
        }
    }

    #[wasm_bindgen]
    pub fn clear_selection(&mut self) {
        self.editor.clear_selection();
    }

    #[wasm_bindgen]
    pub fn select_all(&mut self) {
        let shapes = self.editor.get_all_shapes();
        let ids: Vec<String> = shapes
            .into_serde::<Vec<archflow_sdk::wasm::JsShape>>()
            .unwrap_or_default()
            .into_iter()
            .map(|s| s.id)
            .collect();
        self.editor.select_multiple(ids);
    }

    // === Query Methods ===

    #[wasm_bindgen]
    pub fn shape_count(&self) -> usize {
        let shapes = self.editor.get_all_shapes();
        shapes
            .into_serde::<Vec<archflow_sdk::wasm::JsShape>>()
            .map(|v| v.len())
            .unwrap_or(0)
    }

    #[wasm_bindgen]
    pub fn selection_count(&self) -> usize {
        let selection = self.editor.get_selection();
        selection
            .into_serde::<archflow_sdk::wasm::JsSelection>()
            .map(|s| s.shape_ids.len())
            .unwrap_or(0)
    }

    #[wasm_bindgen]
    pub fn can_undo(&self) -> bool {
        // SDK command manager tracks undo state
        false // Placeholder - implement with command history
    }

    #[wasm_bindgen]
    pub fn can_redo(&self) -> bool {
        false // Placeholder - implement with command history
    }

    #[wasm_bindgen]
    pub fn get_zoom(&self) -> f32 {
        // Get zoom from viewport
        1.0 // Placeholder - get from viewport
    }

    // === SDK Accessors (Production API) ===

    /// Gets the underlying SDK editor for advanced operations
    #[wasm_bindgen]
    pub fn get_editor(&self) -> &archflow_sdk::wasm::ArchFlowEditor {
        &self.editor
    }

    /// Gets the library manager for component library operations
    #[wasm_bindgen]
    pub fn get_library_manager(&self) -> JsLibraryManager {
        JsLibraryManager::new()
    }

    /// Gets the properties manager for property panel operations
    #[wasm_bindgen]
    pub fn get_properties_manager(&self) -> JsPropertiesManager {
        JsPropertiesManager::new()
    }

    /// Gets the alignment manager for alignment operations
    #[wasm_bindgen]
    pub fn get_alignment_manager(&self) -> JsAlignmentManager {
        JsAlignmentManager::new()
    }

    /// Gets the group manager for group operations
    #[wasm_bindgen]
    pub fn get_group_manager(&self) -> JsGroupManager {
        JsGroupManager::new()
    }

    // === Rendering ===

    #[wasm_bindgen]
    pub fn render(&mut self) {
        // Clear canvas
        self.context.set_fill_style(&JsValue::from_str("#1e1e1e"));
        let _ = self.context.fill_rect(0.0, 0.0, self.width as f64, self.height as f64);

        // Save context state
        self.context.save();

        // Get viewport transformation
        let viewport = self.editor.canvas.borrow().viewport();
        let transform = viewport.transform();

        // Apply transformations
        self.context
            .translate(transform.m41 as f64, transform.m42 as f64)
            .ok();
        self.context
            .scale(transform.m11 as f64, transform.m22 as f64)
            .ok();

        // Render cached grid (optimized)
        self.render_cached_grid();

        // Render all shapes via SDK
        let shapes = self.editor.get_all_shapes();
        if let Ok(js_shapes) = shapes.into_serde::<Vec<archflow_sdk::wasm::JsShape>>() {
            for shape in js_shapes {
                self.render_js_shape(&shape);
            }
        }

        // Restore context
        self.context.restore();

        // Render selection bounds
        self.render_selection_bounds();
    }

    // === Collaboration Simulation ===

    #[wasm_bindgen]
    pub fn simulate_remote_cursor(&mut self, _x: f64, _y: f64, _name: &str) {
        // Remote cursors are handled by collab module
        log::debug!("Remote cursor simulation - use collab module");
    }

    #[wasm_bindgen]
    pub fn get_delta(&self) -> Vec<u8> {
        // Serialize canvas state for collaboration
        Vec::new() // Placeholder - implement with collab delta
    }
}

// Private rendering methods with optimizations
impl ArchFlowEditor {
    /// Creates cached grid off-screen canvas for performance
    fn ensure_grid_cache(&mut self) {
        if self.grid_canvas.is_none() {
            if let Ok(offscreen) = web_sys::OffscreenCanvas::new(self.width, self.height) {
                let ctx = offscreen
                    .get_context("2d")
                    .ok()
                    .flatten()
                    .and_then(|c| c.dyn_into::<web_sys::OffscreenCanvasRenderingContext2d>().ok());

                if let Some(ctx) = ctx {
                    self.render_grid_to_context(&ctx);
                    self.grid_canvas = Some(offscreen);
                }
            }
        }
    }

    /// Renders cached grid image (optimized)
    fn render_cached_grid(&mut self) {
        self.ensure_grid_cache();
        if let Some(offscreen) = &self.grid_canvas {
            if let Ok bitmap = offscreen.transfer_to_image_bitmap() {
                let _ = self.context.draw_image_with_image_bitmap(&bitmap, 0.0, 0.0);
            }
        }
    }

    /// Renders grid to a context (used for caching)
    fn render_grid_to_context(&self, ctx: &web_sys::OffscreenCanvasRenderingContext2d) {
        let grid_config = self.editor.get_grid_config();
        if let Ok(config) = grid_config.into_serde::<archflow_sdk::background::GridConfig>() {
            if !config.show_grid {
                return;
            }

            let spacing = config.spacing as f64;
            let dot_color = format!(
                "rgba({},{},{},{})",
                (config.dot_color.r * 255.0) as u8,
                (config.dot_color.g * 255.0) as u8,
                (config.dot_color.b * 255.0) as u8,
                config.dot_color.a
            );

            ctx.set_fill_style(&JsValue::from_str(&dot_color));

            let mut x = 0.0;
            while x < self.width as f64 {
                let mut y = 0.0;
                while y < self.height as f64 {
                    let _ = ctx.arc(x, y, config.dot_radius as f64, 0.0, std::f64::consts::PI * 2.0);
                    let _ = ctx.fill();
                    y += spacing;
                }
                x += spacing;
            }
        }
    }

    /// Renders a JS shape from SDK
    fn render_js_shape(&self, shape: &archflow_sdk::wasm::JsShape) {
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
            self.context.set_stroke_style(&JsValue::from_str(&stroke_color));
            self.context.set_line_width(shape.stroke_width as f64);
        }

        let path = match web_sys::Path2d::new() {
            Ok(p) => p,
            Err(_) => return,
        };

        match shape.shape_type.as_str() {
            "Rectangle" | "Rect" => {
                let _ = path.rect(shape.x as f64, shape.y as f64, shape.width as f64, shape.height as f64);
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
                // Default to rectangle
                let _ = path.rect(shape.x as f64, shape.y as f64, shape.width as f64, shape.height as f64);
            }
        }

        let _ = self.context.fill_with_path_2d(&path);
        if shape.stroke_color.is_some() {
            let _ = self.context.stroke_with_path(&path);
        }

        // Render selection indicator
        if shape.selected {
            self.render_selection_outline(shape.x as f64, shape.y as f64, shape.width as f64, shape.height as f64);
        }
    }

    /// Renders selection bounds
    fn render_selection_bounds(&self) {
        let selection = self.editor.get_selection();
        if let Ok(js_selection) = selection.into_serde::<archflow_sdk::wasm::JsSelection>() {
            if !js_selection.shape_ids.is_empty() {
                self.render_selection_outline(
                    js_selection.bounds.x as f64,
                    js_selection.bounds.y as f64,
                    js_selection.bounds.width as f64,
                    js_selection.bounds.height as f64,
                );
            }
        }
    }

    /// Renders selection outline
    fn render_selection_outline(&self, x: f64, y: f64, width: f64, height: f64) {
        self.context.set_stroke_style(&JsValue::from_str("#0066cc"));
        self.context.set_line_width(2.0);
        let _ = self.context.set_line_dash(&JsValue::from_str("5,5"));

        let path = match web_sys::Path2d::new() {
            Ok(p) => p,
            Err(_) => return,
        };
        let _ = path.rect(x, y, width, height);
        let _ = self.context.stroke_with_path(&path);

        let _ = self.context.set_line_dash(&JsValue::from_str(""));

        // Draw handles
        self.render_handle(x, y);
        self.render_handle(x + width, y);
        self.render_handle(x, y + height);
        self.render_handle(x + width, y + height);
    }

    /// Renders a resize handle
    fn render_handle(&self, x: f64, y: f64) {
        self.context.set_fill_style(&JsValue::from_str("#0066cc"));
        let path = match web_sys::Path2d::new() {
            Ok(p) => p,
            Err(_) => return,
        };
        let _ = path.rect(x - 4.0, y - 4.0, 8.0, 8.0);
        let _ = self.context.fill_with_path_2d(&path);
    }
}

/// Backward compatibility type alias
#[wasm_bindgen]
pub type ArchFlowDemo = ArchFlowEditor;
