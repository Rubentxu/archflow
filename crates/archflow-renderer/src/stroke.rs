//! Stroke style implementation

use super::{Color, LineCap, LineJoin};

/// Estilo de stroke para paths
#[derive(Debug, Clone)]
pub struct StrokeStyle {
    /// Color de la línea
    pub color: Color,
    /// Ancho de línea en pixels
    pub width: f32,
    /// Estilo de terminación de línea
    pub line_cap: LineCap,
    /// Estilo de unión entre segmentos
    pub line_join: LineJoin,
    /// Patrón de dash (vacío = línea continua)
    pub dash_pattern: Option<Vec<f32>>,
    /// Límite de miter (para LineJoin::Miter)
    pub miter_limit: f32,
}

impl Default for StrokeStyle {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            dash_pattern: None,
            miter_limit: 4.0,
        }
    }
}

impl StrokeStyle {
    /// Crear un nuevo estilo de stroke
    pub fn new(color: Color, width: f32) -> Self {
        Self {
            color,
            width,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            dash_pattern: None,
            miter_limit: 4.0,
        }
    }

    /// Configurar el ancho de línea
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Configurar el color
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Configurar el estilo de terminación
    pub fn line_cap(mut self, line_cap: LineCap) -> Self {
        self.line_cap = line_cap;
        self
    }

    /// Configurar el estilo de unión
    pub fn line_join(mut self, line_join: LineJoin) -> Self {
        self.line_join = line_join;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_core::Color;

    #[test]
    fn test_stroke_style_default() {
        let style = StrokeStyle::default();
        assert_eq!(style.color, Color::BLACK);
        assert_eq!(style.width, 1.0);
    }

    #[test]
    fn test_stroke_style_builder() {
        let style = StrokeStyle::new(Color::RED, 2.0).line_cap(LineCap::Round);
        assert_eq!(style.color, Color::RED);
        assert_eq!(style.width, 2.0);
        assert_eq!(style.line_cap, LineCap::Round);
    }
}
