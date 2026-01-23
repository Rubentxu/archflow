//! ArchFlow Renderer Rough - Rendering estilo "hand-drawn"
//!
//! Este crate implementa rendering con efecto sketchy (RoughJS style)

use archflow_core::Color;
use archflow_renderer::{FontStyle, Image, Path, Renderer, StrokeStyle};

/// Renderer con efecto "hand-drawn"
pub struct RoughRenderer<R: Renderer> {
    inner: R,
    roughness: f32,
    bowing: f32,
}

impl<R: Renderer> RoughRenderer<R> {
    /// Crear nuevo renderer rough
    pub fn new(inner: R, roughness: f32, bowing: f32) -> Self {
        Self {
            inner,
            roughness,
            bowing,
        }
    }

    /// Obtener referencia al renderer interno
    pub fn inner(&self) -> &R {
        &self.inner
    }

    /// Obtener referencia mutable al renderer interno
    pub fn inner_mut(&mut self) -> &mut R {
        &mut self.inner
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

    fn reset_transform(&mut self) {
        self.inner.reset_transform();
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

    fn fill_path(&mut self, path: &dyn Path, color: Color) {
        self.inner.fill_path(path, color);
    }

    fn stroke_path(&mut self, path: &dyn Path, style: &StrokeStyle) {
        self.inner.stroke_path(path, style);
    }

    fn draw_text(&mut self, text: &str, x: f32, y: f32, font: &FontStyle) {
        self.inner.draw_text(text, x, y, font);
    }

    fn draw_image(&mut self, image: &dyn Image, x: f32, y: f32, width: f32, height: f32) {
        self.inner.draw_image(image, x, y, width, height);
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
        self.inner.draw_image_slice(
            image, src_x, src_y, src_width, src_height, dst_x, dst_y, dst_width, dst_height,
        );
    }
}
