//! SVG Path implementation for the renderer

use super::{Path, PixelFormat};

/// Implementación simple de Path desde SVG path string
#[derive(Debug, Clone)]
pub struct SvgPath {
    svg_data: String,
    bounds: (f32, f32, f32, f32),
}

impl SvgPath {
    /// Crear un nuevo SvgPath desde un string SVG
    pub fn new(svg_path: &str) -> Self {
        Self::parse_bounds(svg_path)
    }

    /// Parsear bounds desde un SVG path string simple
    fn parse_bounds(svg_path: &str) -> Self {
        // Extraer números del path para estimar bounds
        let nums: Vec<f32> = svg_path
            .split(|c: char| {
                c.is_ascii_whitespace() || ",MLCZHVOQSTAm l c z h v o q s t A".contains(c)
            })
            .filter_map(|s| s.parse::<f32>().ok())
            .collect();

        if nums.len() >= 4 {
            let min_x = nums.iter().step_by(2).fold(f32::MAX, |a, &b| a.min(b));
            let min_y = nums
                .iter()
                .skip(1)
                .step_by(2)
                .fold(f32::MAX, |a, &b| a.min(b));
            let max_x = nums.iter().step_by(2).fold(f32::MIN, |a, &b| a.max(b));
            let max_y = nums
                .iter()
                .skip(1)
                .step_by(2)
                .fold(f32::MIN, |a, &b| a.max(b));

            Self {
                svg_data: svg_path.to_string(),
                bounds: (min_x, min_y, max_x - min_x, max_y - min_y),
            }
        } else {
            Self {
                svg_data: svg_path.to_string(),
                bounds: (0.0, 0.0, 100.0, 100.0),
            }
        }
    }

    /// Obtener referencia interna a los datos SVG
    pub fn svg_data(&self) -> &str {
        &self.svg_data
    }
}

impl Path for SvgPath {
    fn to_svg_path(&self) -> String {
        self.svg_data.clone()
    }

    fn bounds(&self) -> (f32, f32, f32, f32) {
        self.bounds
    }

    fn is_empty(&self) -> bool {
        self.svg_data.trim().is_empty()
    }

    fn length(&self) -> f64 {
        // Estimación simple basada en el número de comandos
        self.svg_data.len() as f64 * 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_path_creation() {
        let path = SvgPath::new("M 10 10 L 90 90");
        assert!(!path.is_empty());
        assert_eq!(path.to_svg_path(), "M 10 10 L 90 90");
    }

    #[test]
    fn test_svg_path_bounds() {
        let path = SvgPath::new("M 10 10 L 90 90");
        let (x, y, w, h) = path.bounds();
        assert_eq!(x, 10.0);
        assert_eq!(y, 10.0);
        assert_eq!(w, 80.0);
        assert_eq!(h, 80.0);
    }

    #[test]
    fn test_empty_path() {
        let path = SvgPath::new("");
        assert!(path.is_empty());
    }
}
