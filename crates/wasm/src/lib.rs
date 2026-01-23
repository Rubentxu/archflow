//! ArchFlow WASM - WebAssembly bindings for browser rendering

use archflow_ecs::Color;
use archflow_renderer::{CanvasContext, CanvasPath, Renderer, RendererBuilder};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, Path2d};

/// Initialize panic hook for better error messages
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&"ArchFlow WASM initialized".into());
}

/// WebSys Canvas Context wrapper
#[derive(Debug)]
pub struct WebSysCanvasContext {
    ctx: CanvasRenderingContext2d,
}

impl WebSysCanvasContext {
    pub fn new(ctx: CanvasRenderingContext2d) -> Self {
        Self { ctx }
    }
}

impl CanvasContext for WebSysCanvasContext {
    fn clear_rect(&self, x: f64, y: f64, width: f64, height: f64) {
        self.ctx.clear_rect(x, y, width, height);
    }

    fn set_fill_style(&self, style: &str) {
        self.ctx.set_fill_style(&style.into());
    }

    fn set_stroke_style(&self, style: &str) {
        self.ctx.set_stroke_style(&style.into());
    }

    fn set_line_width(&self, width: f64) {
        self.ctx.set_line_width(width);
    }

    fn set_font(&self, font: &str) {
        self.ctx.set_font(font);
    }

    fn fill_rect(&self, x: f64, y: f64, width: f64, height: f64) {
        self.ctx.fill_rect(x, y, width, height);
    }

    fn ellipse(
        &self,
        x: f64,
        y: f64,
        radius_x: f64,
        radius_y: f64,
        rotation: f64,
        start_angle: f64,
        end_angle: f64,
    ) -> Result<(), String> {
        self.ctx
            .ellipse(x, y, radius_x, radius_y, rotation, start_angle, end_angle)
            .map_err(|e| format!("{:?}", e))
    }

    fn fill_text(&self, text: &str, x: f64, y: f64) -> Result<(), String> {
        self.ctx
            .fill_text(text, x, y)
            .map_err(|e| format!("{:?}", e))?;
        Ok(())
    }

    fn stroke(&self) {
        self.ctx.stroke();
    }

    fn begin_path(&self) {
        self.ctx.begin_path();
    }

    fn fill(&self) {
        self.ctx.fill();
    }

    fn fill_with_path(&self, path: &dyn CanvasPath) {
        if let Some(wasm_path) = path.as_any().downcast_ref::<WebSysPath>() {
            self.ctx.fill_with_path_2d(&wasm_path.path);
        }
    }
}

/// WebSys Path2D wrapper
#[derive(Debug)]
pub struct WebSysPath {
    path: Path2d,
}

impl WebSysPath {
    pub fn new() -> Self {
        Self {
            path: Path2d::new().unwrap(),
        }
    }
}

impl CanvasPath for WebSysPath {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn move_to(&self, x: f64, y: f64) {
        self.path.move_to(x, y);
    }

    fn line_to(&self, x: f64, y: f64) {
        self.path.line_to(x, y);
    }

    fn quadratic_curve_to(&self, cp_x: f64, cp_y: f64, x: f64, y: f64) {
        self.path.quadratic_curve_to(cp_x, cp_y, x, y);
    }

    fn close_path(&self) {
        self.path.close_path();
    }
}

/// Represents a handle to the ArchFlow WASM runtime
#[wasm_bindgen]
pub struct ArchFlowWasm {
    renderer: Rc<RefCell<Renderer>>,
    canvas: HtmlCanvasElement,
    width: u32,
    height: u32,
}

#[wasm_bindgen]
impl ArchFlowWasm {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement) -> Result<ArchFlowWasm, JsValue> {
        let width = canvas.client_width() as u32;
        let height = canvas.client_height() as u32;

        let config = RendererBuilder::new()
            .with_size(width, height)
            .with_samples(1) // Canvas 2D doesn't need MSAA
            .with_vsync(false);

        let mut renderer = Renderer::new(config);

        // Get canvas context and set it on renderer
        let context = canvas
            .get_context("2d")
            .map_err(|_| JsValue::from_str("Failed to get canvas context"))?
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .map_err(|_| JsValue::from_str("Context is not 2D"))?;

        renderer.set_context(Box::new(WebSysCanvasContext::new(context)));

        Ok(ArchFlowWasm {
            renderer: Rc::new(RefCell::new(renderer)),
            canvas,
            width,
            height,
        })
    }

    #[wasm_bindgen]
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.renderer.borrow_mut().resize(width, height);
    }

    /// Render all queued shapes to the canvas
    #[wasm_bindgen]
    pub fn render(&mut self) -> Result<(), JsValue> {
        self.renderer
            .borrow_mut()
            .render()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Clear all queued shapes
    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.renderer.borrow_mut().clear_queue();
    }

    /// Add a rectangle to the render queue (color as separate RGBA u8 values)
    #[wasm_bindgen]
    pub fn add_rect(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    ) {
        let c = Color::new(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        );
        self.renderer
            .borrow_mut()
            .queue_rect(x, y, width, height, c);
    }

    /// Add a rounded rectangle to the render queue (color as separate RGBA u8 values)
    #[wasm_bindgen]
    pub fn add_rounded_rect(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        corner_radius: f32,
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    ) {
        let c = Color::new(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        );
        self.renderer
            .borrow_mut()
            .queue_rounded_rect(x, y, width, height, corner_radius, c);
    }

    /// Add an ellipse to the render queue (color as separate RGBA u8 values)
    #[wasm_bindgen]
    pub fn add_ellipse(
        &mut self,
        cx: f32,
        cy: f32,
        radius_x: f32,
        radius_y: f32,
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    ) {
        let c = Color::new(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        );
        self.renderer
            .borrow_mut()
            .queue_ellipse(cx, cy, radius_x, radius_y, c);
    }

    /// Add text to the render queue (color as separate RGBA u8 values)
    #[wasm_bindgen]
    pub fn add_text(
        &mut self,
        text: &str,
        x: f32,
        y: f32,
        font_size: f32,
        font_family: &str,
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    ) {
        let c = Color::new(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        );
        self.renderer
            .borrow_mut()
            .queue_text(text, x, y, font_size, font_family, c);
    }

    /// Get the number of queued shapes
    #[wasm_bindgen]
    pub fn queued_shapes(&self) -> usize {
        self.renderer.borrow().shape_queue.len()
    }

    #[wasm_bindgen(getter)]
    pub fn canvas(&self) -> HtmlCanvasElement {
        self.canvas.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.height
    }
}

/// WASM memory statistics
#[wasm_bindgen]
pub struct MemoryStats {
    total_bytes: u64,
    used_bytes: u64,
}

#[wasm_bindgen]
impl MemoryStats {
    #[wasm_bindgen(getter)]
    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    #[wasm_bindgen(getter)]
    pub fn used_bytes(&self) -> u64 {
        self.used_bytes
    }
}

/// Get current memory statistics from the WASM runtime
#[wasm_bindgen]
pub fn get_memory_stats() -> MemoryStats {
    MemoryStats {
        total_bytes: 0,
        used_bytes: 0,
    }
}
