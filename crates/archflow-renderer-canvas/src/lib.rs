//! ArchFlow Renderer Canvas - Implementación Canvas 2D
//!
//! Este crate implementa el Renderer trait para Canvas 2D usando web-sys

use archflow_core::Color;
use archflow_renderer::{Image, Path, Renderer};
use web_sys::{wasm_bindgen::JsCast, wasm_bindgen::JsValue};

pub struct CanvasRenderer {
    context: web_sys::CanvasRenderingContext2d,
    canvas: web_sys::HtmlCanvasElement,
}

impl CanvasRenderer {
    pub fn new(canvas: web_sys::HtmlCanvasElement) -> Self {
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        Self { context, canvas }
    }
}

impl Renderer for CanvasRenderer {
    fn clear(&mut self, color: Color) {
        let color_str = format!(
            "rgba({}, {}, {}, {})",
            (color.r * 255.0) as u8,
            (color.g * 255.0) as u8,
            (color.b * 255.0) as u8,
            color.a
        );
        self.context.set_fill_style(&JsValue::from_str(&color_str));
        self.context.fill_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );
    }

    fn save(&mut self) {
        let _ = self.context.save();
    }

    fn restore(&mut self) {
        let _ = self.context.restore();
    }

    fn translate(&mut self, x: f32, y: f32) {
        self.context.translate(x as f64, y as f64).unwrap();
    }

    fn rotate(&mut self, angle: f32) {
        self.context.rotate(angle as f64).unwrap();
    }

    fn scale(&mut self, sx: f32, sy: f32) {
        self.context.scale(sx as f64, sy as f64).unwrap();
    }

    fn draw_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.context
            .fill_rect(x as f64, y as f64, width as f64, height as f64);
    }

    fn draw_ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32) {
        let _ = self.context.ellipse(
            cx as f64,
            cy as f64,
            rx as f64,
            ry as f64,
            0.0,
            0.0,
            std::f64::consts::PI * 2.0,
        );
    }

    fn draw_path(&mut self, path: &dyn Path) {
        // Implementación básica - mover a kurbo para path real
        let _ = path;
    }

    fn draw_text(&mut self, text: &str, x: f32, y: f32) {
        self.context.fill_text(text, x as f64, y as f64).unwrap();
    }

    fn draw_image(&mut self, image: &dyn Image, x: f32, y: f32, width: f32, height: f32) {
        let _ = image;
        let _ = (x, y, width, height);
        // Implementación pendiente de ImageBitmap
    }
}
