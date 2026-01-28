//! ArchFlow Demo Web - WASM module for browser demo
//!
//! This module provides the main WASM interface for the interactive demo.
//! Uses Canvas 2D API via web-sys for rendering.

use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

mod shapes;
mod state;

#[cfg(test)]
mod tests;

pub use shapes::{RemoteCursor, Shape, ShapeId, ShapeStore, ShapeType};
pub use state::{DemoState, InteractionState, Rect, Tool};

/// Main WASM struct exposed to JavaScript
#[wasm_bindgen]
pub struct ArchFlowDemo {
    state: DemoState,
    context: CanvasRenderingContext2d,
    width: u32,
    height: u32,
}

#[wasm_bindgen]
impl ArchFlowDemo {
    /// Creates a new demo instance
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: web_sys::HtmlCanvasElement) -> Result<ArchFlowDemo, JsValue> {
        console_error_panic_hook::set_once();

        let context = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("Failed to get 2d context"))?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| JsValue::from_str("Failed to cast to CanvasRenderingContext2d"))?;

        let width = canvas.width();
        let height = canvas.height();

        Ok(ArchFlowDemo {
            state: DemoState::new(),
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

    // === Tool Management ===

    /// Sets the current tool
    #[wasm_bindgen]
    pub fn set_tool(&mut self, tool: &str) {
        self.state.set_tool(tool);
    }

    // === Input Handling ===

    #[wasm_bindgen]
    pub fn on_mousedown(&mut self, x: f64, y: f64, button: u16) {
        self.state.on_mousedown(x, y, button);
    }

    #[wasm_bindgen]
    pub fn on_mousemove(&mut self, x: f64, y: f64) {
        self.state.on_mousemove(x, y);
    }

    #[wasm_bindgen]
    pub fn on_mouseup(&mut self, x: f64, y: f64) {
        self.state.on_mouseup(x, y);
    }

    #[wasm_bindgen]
    pub fn on_wheel(&mut self, x: f64, y: f64, zoom_out: bool) {
        self.state.on_wheel(x, y, zoom_out);
    }

    // === Keyboard Events ===

    /// Handles key down event
    #[wasm_bindgen]
    pub fn on_keydown(&mut self, key: &str, shift: bool, ctrl: bool) {
        match key {
            "ArrowUp" => {
                let dx = if shift { 10.0 } else { 1.0 };
                self.state.nudge_selection(0.0, -dx);
            }
            "ArrowDown" => {
                let dx = if shift { 10.0 } else { 1.0 };
                self.state.nudge_selection(0.0, dx);
            }
            "ArrowLeft" => {
                let dx = if shift { 10.0 } else { 1.0 };
                self.state.nudge_selection(-dx, 0.0);
            }
            "ArrowRight" => {
                let dx = if shift { 10.0 } else { 1.0 };
                self.state.nudge_selection(dx, 0.0);
            }
            "a" if ctrl => {
                self.state.select_all();
            }
            "z" if ctrl => {
                if shift {
                    self.state.redo();
                } else {
                    self.state.undo();
                }
            }
            "y" if ctrl => {
                self.state.redo();
            }
            "Delete" | "Backspace" => {
                self.state.delete_selected();
            }
            "Escape" => {
                self.state.clear_selection();
            }
            " " => {
                // Space bar - switch to pan mode temporarily
                self.state.set_tool("pan");
            }
            "v" => self.state.set_tool("select"),
            "r" => self.state.set_tool("rect"),
            "o" => self.state.set_tool("ellipse"),
            "l" => self.state.set_tool("line"),
            "h" => {
                // Zoom to selection
                self.state.zoom_to_selection();
            }
            "0" if ctrl => {
                // Zoom to fit
                self.state.zoom_to_fit();
            }
            "=" | "+" if ctrl => {
                self.state.zoom_in();
            }
            "-" if ctrl => {
                self.state.zoom_out();
            }
            _ => {}
        }
    }

    /// Handles key up event
    #[wasm_bindgen]
    pub fn on_keyup(&mut self, key: &str) {
        if key == " " {
            // Return to select mode
            self.state.set_tool("select");
        }
    }

    // === Shape Operations ===

    #[wasm_bindgen]
    pub fn add_rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.state.add_rect(x, y, width, height);
    }

    #[wasm_bindgen]
    pub fn add_ellipse(&mut self, x: f64, y: f64, radius_x: f64, radius_y: f64) {
        self.state.add_ellipse(x, y, radius_x, radius_y);
    }

    #[wasm_bindgen]
    pub fn add_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.state.add_line(x1, y1, x2, y2);
    }

    #[wasm_bindgen]
    pub fn delete_selected(&mut self) {
        self.state.delete_selected();
    }

    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.state.clear();
    }

    #[wasm_bindgen]
    pub fn clear_selection(&mut self) {
        self.state.clear_selection();
    }

    #[wasm_bindgen]
    pub fn select_all(&mut self) {
        self.state.select_all();
    }

    // === Query Methods ===

    #[wasm_bindgen]
    pub fn shape_count(&self) -> usize {
        self.state.shape_count()
    }

    #[wasm_bindgen]
    pub fn selection_count(&self) -> usize {
        self.state.selection_count()
    }

    #[wasm_bindgen]
    pub fn can_undo(&self) -> bool {
        self.state.can_undo()
    }

    #[wasm_bindgen]
    pub fn can_redo(&self) -> bool {
        self.state.can_redo()
    }

    #[wasm_bindgen]
    pub fn get_zoom(&self) -> f32 {
        self.state.get_zoom()
    }

    // === Rendering ===

    #[wasm_bindgen]
    pub fn render(&mut self) {
        // Clear canvas
        self.context.set_fill_style(&JsValue::from_str("#1e1e1e"));
        let _ = self
            .context
            .fill_rect(0.0, 0.0, self.width as f64, self.height as f64);

        // Save context state
        self.context.save();

        // Apply pan and zoom transformations
        let (pan_x, pan_y) = self.state.get_pan_offset();
        let zoom = self.state.get_zoom();

        // Apply transformations: translate to pan offset, then scale by zoom
        // The zoom is centered at origin, so we translate to center, scale, then back
        let center_x = self.width as f64 / 2.0;
        let center_y = self.height as f64 / 2.0;

        self.context.translate(center_x, center_y).ok();
        self.context.scale(zoom as f64, zoom as f64).ok();
        self.context
            .translate(-center_x + pan_x, -center_y + pan_y)
            .ok();

        // Render grid (transformed)
        self.render_grid();

        // Render all shapes (transformed)
        for shape in self.state.shapes() {
            self.render_shape(shape);
        }

        // Restore context for UI elements that shouldn't be transformed
        self.context.restore();

        // Render box selection (in screen coordinates, not transformed)
        if let InteractionState::BoxSelecting {
            start_x,
            start_y,
            current_x,
            current_y,
        } = self.state.interaction_state()
        {
            self.render_box_selection(*start_x, *start_y, *current_x, *current_y);
        }

        // Render selection bounds (in screen coordinates, not transformed)
        if let Some(bounds) = self.state.selection_bounds() {
            self.render_selection_bounds(&bounds);
        }
    }

    // === Collaboration Simulation ===

    #[wasm_bindgen]
    pub fn simulate_remote_cursor(&mut self, x: f64, y: f64, name: &str) {
        self.state.add_remote_cursor(x, y, name);
    }

    #[wasm_bindgen]
    pub fn get_delta(&self) -> Vec<u8> {
        self.state.serialize_delta()
    }
}

// Private rendering methods
impl ArchFlowDemo {
    fn render_grid(&self) {
        let grid_size = 20.0;
        self.context.set_stroke_style(&JsValue::from_str("#2a2a2a"));
        self.context.set_line_width(1.0);

        let path = match web_sys::Path2d::new() {
            Ok(p) => p,
            Err(_) => return,
        };

        let mut x = 0.0;
        while x < self.width as f64 {
            let _ = path.move_to(x, 0.0);
            let _ = path.line_to(x, self.height as f64);
            x += grid_size;
        }

        let mut y = 0.0;
        while y < self.height as f64 {
            let _ = path.move_to(0.0, y);
            let _ = path.line_to(self.width as f64, y);
            y += grid_size;
        }

        let _ = self.context.stroke_with_path(&path);
    }

    fn render_shape(&self, shape: &Shape) {
        match shape.shape_type {
            ShapeType::Rectangle => self.render_rectangle(shape),
            ShapeType::Ellipse => self.render_ellipse(shape),
            ShapeType::Line => self.render_line(shape),
        }
    }

    fn render_rectangle(&self, shape: &Shape) {
        let color = shape.color_as_css();
        self.context.set_fill_style(&JsValue::from_str(&color));
        self.context.set_stroke_style(&JsValue::from_str("#ffffff"));
        self.context.set_line_width(1.0);

        let path = match web_sys::Path2d::new() {
            Ok(p) => p,
            Err(_) => return,
        };
        let _ = path.rect(shape.x, shape.y, shape.width, shape.height);
        let _ = self.context.fill_with_path_2d(&path);
        let _ = self.context.stroke_with_path(&path);
    }

    fn render_ellipse(&self, shape: &Shape) {
        let color = shape.color_as_css();
        self.context.set_fill_style(&JsValue::from_str(&color));
        self.context.set_stroke_style(&JsValue::from_str("#ffffff"));
        self.context.set_line_width(1.0);

        let rx = shape.width / 2.0;
        let ry = shape.height / 2.0;
        let cx = shape.x + rx;
        let cy = shape.y + ry;

        let path = match web_sys::Path2d::new() {
            Ok(p) => p,
            Err(_) => return,
        };
        let _ = path.ellipse(cx, cy, rx, ry, 0.0, 0.0, std::f64::consts::PI * 2.0);
        let _ = self.context.fill_with_path_2d(&path);
        let _ = self.context.stroke_with_path(&path);
    }

    fn render_line(&self, shape: &Shape) {
        self.context
            .set_stroke_style(&JsValue::from_str(&shape.color_as_css()));
        self.context.set_line_width(2.0);

        let path = match web_sys::Path2d::new() {
            Ok(p) => p,
            Err(_) => return,
        };
        let _ = path.move_to(shape.x, shape.y);
        let _ = path.line_to(shape.x + shape.width, shape.y + shape.height);
        let _ = self.context.stroke_with_path(&path);
    }

    fn render_selection_bounds(&self, bounds: &Rect) {
        self.context.set_stroke_style(&JsValue::from_str("#0066cc"));
        self.context.set_line_width(2.0);
        let _ = self.context.set_line_dash(&JsValue::from_str("5,5"));

        let path = match web_sys::Path2d::new() {
            Ok(p) => p,
            Err(_) => return,
        };
        let _ = path.rect(bounds.x, bounds.y, bounds.width, bounds.height);
        let _ = self.context.stroke_with_path(&path);

        let _ = self.context.set_line_dash(&JsValue::from_str(""));

        // Draw resize handles
        self.render_handle(bounds.x, bounds.y);
        self.render_handle(bounds.x + bounds.width, bounds.y);
        self.render_handle(bounds.x, bounds.y + bounds.height);
        self.render_handle(bounds.x + bounds.width, bounds.y + bounds.height);
    }

    fn render_box_selection(&self, x1: f64, y1: f64, x2: f64, y2: f64) {
        let min_x = x1.min(x2);
        let min_y = y1.min(y2);
        let width = (x2 - x1).abs();
        let height = (y2 - y1).abs();

        self.context.set_stroke_style(&JsValue::from_str("#0066cc"));
        self.context.set_line_width(1.0);
        let _ = self.context.set_line_dash(&JsValue::from_str("5,5"));

        let path = match web_sys::Path2d::new() {
            Ok(p) => p,
            Err(_) => return,
        };
        let _ = path.rect(min_x, min_y, width, height);
        let _ = self.context.stroke_with_path(&path);

        let _ = self.context.set_line_dash(&JsValue::from_str(""));

        // Semi-transparent fill
        self.context
            .set_fill_style(&JsValue::from_str("rgba(0, 102, 204, 0.1)"));
        let _ = self.context.fill_rect(min_x, min_y, width, height);
    }

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
