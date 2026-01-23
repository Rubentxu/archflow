//! ArchFlow Renderer - Traits de rendering
//!
//! Este crate define los traits abstractos para el sistema de rendering

use archflow_core::Color;

/// Trait principal para renderers
pub trait Renderer {
    /// Limpiar el canvas con un color
    fn clear(&mut self, color: Color);

    /// Guardar estado del renderer
    fn save(&mut self);

    /// Restaurar estado del renderer
    fn restore(&mut self);

    /// Aplicar transformación de traslación
    fn translate(&mut self, x: f32, y: f32);

    /// Aplicar transformación de rotación
    fn rotate(&mut self, angle: f32); // radians

    /// Aplicar transformación de escala
    fn scale(&mut self, sx: f32, sy: f32);

    // Métodos de dibujo
    fn draw_rect(&mut self, x: f32, y: f32, width: f32, height: f32);
    fn draw_ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32);
    fn draw_path(&mut self, path: &dyn Path);
    fn draw_text(&mut self, text: &str, x: f32, y: f32);
    fn draw_image(&mut self, image: &dyn Image, x: f32, y: f32, width: f32, height: f32);
}

// Traits adicionales
pub trait Path {
    fn commands(&self) -> &[PathCommand];
    fn fill(&self) -> Option<&FillStyle>;
    fn stroke(&self) -> Option<&StrokeStyle>;
}

pub trait Image {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn data(&self) -> &[u8];
}

pub struct PathCommand {
    pub command_type: CommandType,
    pub params: [f32; 4],
}

pub enum CommandType {
    MoveTo,
    LineTo,
    QuadTo,
    CubicTo,
    Arc,
    Close,
}

// Estilos
pub struct FillStyle {
    pub color: Color,
    pub opacity: f32,
}

pub struct StrokeStyle {
    pub color: Color,
    pub width: f32,
    pub cap: LineCap,
    pub join: LineJoin,
    pub dash: Option<Vec<f32>>,
}

pub enum LineCap {
    Butt,
    Round,
    Square,
}

pub enum LineJoin {
    Miter,
    Round,
    Bevel,
}
