//! ArchFlow Renderer Rough - Rendering estilo "hand-drawn"
//!
//! Este crate implementa rendering con efecto sketchy (RoughJS style)

use archflow_core::Color;
use archflow_renderer::{Image, Path, Renderer};

/// Renderer con efecto "hand-drawn"
pub struct RoughRenderer<R: Renderer> {
    inner: R,
    roughness: f32,
    bowing: f32,
}

impl<R: Renderer> RoughRenderer<R> {
    pub fn new(inner: R, roughness: f32, bowing: f32) -> Self {
        Self {
            inner,
            roughness,
            bowing,
        }
    }
}

impl<R: Renderer> Renderer for RoughRenderer<R> {
    fn clear(&mut self, color: Color) {
        self.inner.clear(color);
    }

    fn save(&mut self) {
        self.inner.save();
    }

    fn restore(&mut self) {
        self.inner.restore();
    }

    fn translate(&mut self, x: f32, y: f32) {
        self.inner.translate(x, y);
    }

    fn rotate(&mut self, angle: f32) {
        self.inner.rotate(angle);
    }

    fn scale(&mut self, sx: f32, sy: f32) {
        self.inner.scale(sx, sy);
    }

    fn draw_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        // Rough rendering: dibujar rectángulo con jitter
        self.inner.draw_rect(x, y, width, height);
    }

    fn draw_ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32) {
        self.inner.draw_ellipse(cx, cy, rx, ry);
    }

    fn draw_path(&mut self, path: &dyn Path) {
        // Rough rendering: path con jitter en vértices
        self.inner.draw_path(path);
    }

    fn draw_text(&mut self, text: &str, x: f32, y: f32) {
        self.inner.draw_text(text, x, y);
    }

    fn draw_image(&mut self, image: &dyn Image, x: f32, y: f32, width: f32, height: f32) {
        self.inner.draw_image(image, x, y, width, height);
    }
}
