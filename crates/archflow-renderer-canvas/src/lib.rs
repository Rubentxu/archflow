//! ArchFlow Canvas 2D Renderer
//!
//! Implementación del Renderer trait usando la API Canvas 2D de browsers.

use archflow_core::Color;
use archflow_renderer::{
    FontFamily, FontStyle, FontStyleType, FontWeight, Image, LineCap, LineJoin, Path as PathTrait,
    PixelFormat, Renderer as RendererTrait, StrokeStyle,
};
use std::f64::consts::PI;
use wasm_bindgen::prelude::JsValue;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

/// Renderer Canvas 2D para browsers
pub struct CanvasRenderer {
    context: CanvasRenderingContext2d,
    canvas: HtmlCanvasElement,
    width: u32,
    height: u32,
}

impl CanvasRenderer {
    /// Crear nuevo renderer desde elemento canvas
    pub fn new(canvas: HtmlCanvasElement) -> Self {
        let width = canvas.width();
        let height = canvas.height();

        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .expect("failed to get 2d context");

        Self {
            context,
            canvas,
            width,
            height,
        }
    }

    /// Crear renderer con dimensiones específicas
    pub fn with_size(canvas: HtmlCanvasElement, width: u32, height: u32) -> Self {
        let _ = canvas.set_width(width);
        let _ = canvas.set_height(height);
        Self::new(canvas)
    }

    /// Ancho en pixels
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Alto en pixels
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Convertir color a JsValue para canvas
    fn color_to_js_value(color: Color) -> JsValue {
        let color_str = format!(
            "rgba({}, {}, {}, {})",
            (color.r * 255.0) as u8,
            (color.g * 255.0) as u8,
            (color.b * 255.0) as u8,
            color.a
        );
        JsValue::from_str(&color_str)
    }
}

impl RendererTrait for CanvasRenderer {
    fn clear(&mut self, color: Color) {
        self.context.save();
        self.context.set_fill_style(&Self::color_to_js_value(color));
        self.context
            .fill_rect(0.0, 0.0, self.width as f64, self.height as f64);
        self.context.restore();
    }

    fn save(&mut self) {
        let _ = self.context.save();
    }

    fn restore(&mut self) {
        let _ = self.context.restore();
    }

    fn translate(&mut self, x: f32, y: f32) {
        let _ = self.context.translate(x as f64, y as f64);
    }

    fn rotate(&mut self, angle: f32) {
        let _ = self.context.rotate(angle as f64);
    }

    fn scale(&mut self, sx: f32, sy: f32) {
        let _ = self.context.scale(sx as f64, sy as f64);
    }

    fn reset_transform(&mut self) {
        let _ = self.context.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
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
            PI * 2.0,
        );
    }

    fn draw_path(&mut self, path: &dyn PathTrait) {
        let _ = path;
        // Usar beginPath y stroke para renderizado básico
        self.context.stroke();
    }

    fn fill_path(&mut self, path: &dyn PathTrait, color: Color) {
        let _ = path;
        self.context.set_fill_style(&Self::color_to_js_value(color));
        self.context.fill();
    }

    fn stroke_path(&mut self, path: &dyn PathTrait, style: &StrokeStyle) {
        let _ = path;
        self.context.set_line_width(style.width as f64);
        self.context
            .set_stroke_style(&Self::color_to_js_value(style.color));
        self.context.stroke();
    }

    fn draw_text(&mut self, text: &str, x: f32, y: f32, font: &FontStyle) {
        let font_str = format!("{}px sans-serif", font.size);
        self.context.set_font(&font_str);
        self.context
            .set_fill_style(&Self::color_to_js_value(font.color));
        let _ = self.context.fill_text(text, x as f64, y as f64);
    }

    fn draw_image(&mut self, image: &dyn Image, x: f32, y: f32, width: f32, height: f32) {
        let _ = image;
        let _ = (x, y, width, height);
        // Implementación pendiente de ImageBitmap
    }

    fn draw_image_slice(
        &mut self,
        image: &dyn Image,
        src_x: f32,
        src_y: f32,
        src_width: f32,
        src_height: f32,
        dst_x: f32,
        dst_y: f32,
        dst_width: f32,
        dst_height: f32,
    ) {
        let _ = image;
        let _ = (
            src_x, src_y, src_width, src_height, dst_x, dst_y, dst_width, dst_height,
        );
        // Implementación pendiente
    }
}

/// ImageBitmap wrapper para Canvas
#[derive(Clone)]
pub struct CanvasImageBitmap {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl CanvasImageBitmap {
    /// Crear nueva imagen bitmap
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            width,
            height,
            data,
        }
    }
}

impl Image for CanvasImageBitmap {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn data(&self) -> &[u8] {
        &self.data
    }

    fn pixel_format(&self) -> PixelFormat {
        PixelFormat::Rgba8
    }
}

/// Builder para CanvasRenderer
pub struct CanvasRendererBuilder {
    canvas: Option<HtmlCanvasElement>,
    width: u32,
    height: u32,
}

impl Default for CanvasRendererBuilder {
    fn default() -> Self {
        Self {
            canvas: None,
            width: 800,
            height: 600,
        }
    }
}

impl CanvasRendererBuilder {
    /// Setear elemento canvas
    pub fn canvas(mut self, canvas: HtmlCanvasElement) -> Self {
        self.canvas = Some(canvas);
        self
    }

    /// Setear dimensiones
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Construir renderer
    pub fn build(self) -> Option<CanvasRenderer> {
        let canvas = self.canvas?;
        let _ = canvas.set_width(self.width);
        let _ = canvas.set_height(self.height);
        Some(CanvasRenderer::new(canvas))
    }
}
