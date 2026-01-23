//! Path rendering types and styles for the renderer.
//!
//! Provides path data structures and rendering styles compatible with lyon.

use crate::Vertex2D;
use archflow_ecs::{Color, Position, Shape, ShapeType};
use std::f32::consts::PI;

/// Stroke style for path rendering
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrokeStyle {
    pub width: f32,
    pub color: Color,
    pub line_cap: LineCap,
    pub line_join: LineJoin,
    pub miter_limit: f32,
}

impl Default for StrokeStyle {
    fn default() -> Self {
        Self {
            width: 2.0,
            color: Color::new(0.0, 0.0, 0.0, 1.0),
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            miter_limit: 4.0,
        }
    }
}

impl StrokeStyle {
    /// Creates a new stroke style with the given width and color
    pub fn new(width: f32, color: Color) -> Self {
        Self {
            width,
            color,
            ..Default::default()
        }
    }

    /// Sets the line cap style
    pub fn with_line_cap(mut self, line_cap: LineCap) -> Self {
        self.line_cap = line_cap;
        self
    }

    /// Sets the line join style
    pub fn with_line_join(mut self, line_join: LineJoin) -> Self {
        self.line_join = line_join;
        self
    }
}

/// Line cap style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineCap {
    Butt,
    Round,
    Square,
}

/// Line join style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineJoin {
    Miter,
    Round,
    Bevel,
}

/// Fill style for path rendering
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FillStyle {
    pub color: Color,
    pub fill_rule: FillRule,
}

impl Default for FillStyle {
    fn default() -> Self {
        Self {
            color: Color::new(0.0, 0.0, 0.0, 1.0),
            fill_rule: FillRule::NonZero,
        }
    }
}

impl FillStyle {
    /// Creates a new fill style with the given color
    pub fn new(color: Color) -> Self {
        Self {
            color,
            ..Default::default()
        }
    }

    /// Sets the fill rule
    pub fn with_fill_rule(mut self, fill_rule: FillRule) -> Self {
        self.fill_rule = fill_rule;
        self
    }
}

/// Fill rule for winding order
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillRule {
    NonZero,
    EvenOdd,
}

/// Tessellated path result
#[derive(Debug, Clone)]
pub struct TessellatedPath {
    pub vertices: Vec<Vertex2D>,
    pub indices: Vec<u32>,
}

impl Default for TessellatedPath {
    fn default() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}

/// Path tessellator placeholder
///
/// Note: Full lyon tessellation integration requires more complex setup.
/// This provides the API structure for path tessellation.
#[derive(Debug, Default)]
pub struct PathTessellator;

impl PathTessellator {
    /// Creates a new path tessellator
    pub fn new() -> Self {
        Self
    }

    /// Tessellates a rectangle
    pub fn tessellate_rect(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        fill: Option<FillStyle>,
        _stroke: Option<StrokeStyle>,
    ) -> TessellatedPath {
        // Create vertices for a simple quad
        let mut vertices = Vec::with_capacity(4);
        let color = fill
            .map(|f| [f.color.r, f.color.g, f.color.b, f.color.a])
            .unwrap_or([0.0, 0.0, 0.0, 1.0]);

        // Triangle 1: top-left, top-right, bottom-left
        vertices.push(Vertex2D {
            position: [x, y],
            uv: [0.0, 0.0],
            color,
        });
        vertices.push(Vertex2D {
            position: [x + width, y],
            uv: [1.0, 0.0],
            color,
        });
        vertices.push(Vertex2D {
            position: [x, y + height],
            uv: [0.0, 1.0],
            color,
        });

        // Triangle 2: top-right, bottom-right, bottom-left
        vertices.push(Vertex2D {
            position: [x + width, y],
            uv: [1.0, 0.0],
            color,
        });
        vertices.push(Vertex2D {
            position: [x + width, y + height],
            uv: [1.0, 1.0],
            color,
        });
        vertices.push(Vertex2D {
            position: [x, y + height],
            uv: [0.0, 1.0],
            color,
        });

        let indices: Vec<u32> = (0..6).collect();

        TessellatedPath { vertices, indices }
    }

    /// Tessellates an ellipse using triangle fan approximation
    pub fn tessellate_ellipse(
        &self,
        cx: f32,
        cy: f32,
        rx: f32,
        ry: f32,
        fill: Option<FillStyle>,
        _stroke: Option<StrokeStyle>,
    ) -> TessellatedPath {
        let segments = 32;
        let mut vertices = Vec::with_capacity(segments + 2);
        let color = fill
            .map(|f| [f.color.r, f.color.g, f.color.b, f.color.a])
            .unwrap_or([0.0, 0.0, 0.0, 1.0]);

        // Center vertex
        vertices.push(Vertex2D {
            position: [cx, cy],
            uv: [0.5, 0.5],
            color,
        });

        // Edge vertices
        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * 2.0 * PI;
            vertices.push(Vertex2D {
                position: [cx + angle.cos() * rx, cy + angle.sin() * ry],
                uv: [0.5 + 0.5 * angle.cos(), 0.5 + 0.5 * angle.sin()],
                color,
            });
        }

        // Create triangle fan indices
        let mut indices = Vec::with_capacity(segments * 3);
        for i in 1..=segments {
            indices.push(0);
            indices.push(i as u32);
            if i < segments {
                indices.push(i as u32 + 1);
            } else {
                indices.push(1); // Close back to first edge vertex
            }
        }

        TessellatedPath { vertices, indices }
    }

    /// Tessellates a line segment
    pub fn tessellate_line(
        &self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        stroke: StrokeStyle,
    ) -> TessellatedPath {
        // Simple line as a thin quad
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt();
        let half_width = stroke.width / 2.0;

        if len < 0.001 {
            return TessellatedPath::default();
        }

        let nx = -dy / len * half_width;
        let ny = dx / len * half_width;

        let color = [
            stroke.color.r,
            stroke.color.g,
            stroke.color.b,
            stroke.color.a,
        ];

        let vertices = vec![
            Vertex2D {
                position: [x1 + nx, y1 + ny],
                uv: [0.0, 0.0],
                color,
            },
            Vertex2D {
                position: [x1 - nx, y1 - ny],
                uv: [0.0, 1.0],
                color,
            },
            Vertex2D {
                position: [x2 + nx, y2 + ny],
                uv: [1.0, 0.0],
                color,
            },
            Vertex2D {
                position: [x2 - nx, y2 - ny],
                uv: [1.0, 1.0],
                color,
            },
        ];

        let indices = vec![0, 1, 2, 2, 1, 3];

        TessellatedPath { vertices, indices }
    }

    /// Tessellates a shape (rect, ellipse, etc.)
    pub fn tessellate_shape(
        &self,
        position: Position,
        shape: &Shape,
        fill: Option<FillStyle>,
        stroke: Option<StrokeStyle>,
    ) -> (TessellatedPath, Option<TessellatedPath>) {
        let x = position.x() - shape.width / 2.0;
        let y = position.y() - shape.height / 2.0;

        match shape.shape_type {
            ShapeType::Rect => {
                let fill_result =
                    self.tessellate_rect(x, y, shape.width, shape.height, fill, stroke);
                (fill_result, None)
            }
            ShapeType::Ellipse => {
                let fill_result = self.tessellate_ellipse(
                    position.x(),
                    position.y(),
                    shape.width / 2.0,
                    shape.height / 2.0,
                    fill,
                    stroke,
                );
                (fill_result, None)
            }
            ShapeType::Line => {
                let line_result = self.tessellate_line(
                    x,
                    y,
                    x + shape.width,
                    y + shape.height,
                    stroke.unwrap_or_default(),
                );
                (TessellatedPath::default(), Some(line_result))
            }
            _ => {
                let fill_result =
                    self.tessellate_rect(x, y, shape.width, shape.height, fill, stroke);
                (fill_result, None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_style_default() {
        let style = FillStyle::default();
        assert_eq!(style.color.r, 0.0);
        assert_eq!(style.fill_rule, FillRule::NonZero);
    }

    #[test]
    fn test_fill_style_new() {
        let style = FillStyle::new(Color::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(style.color.r, 1.0);
    }

    #[test]
    fn test_stroke_style_default() {
        let style = StrokeStyle::default();
        assert_eq!(style.width, 2.0);
        assert_eq!(style.line_cap, LineCap::Butt);
        assert_eq!(style.line_join, LineJoin::Miter);
    }

    #[test]
    fn test_stroke_style_new() {
        let style = StrokeStyle::new(3.0, Color::new(0.0, 0.0, 1.0, 1.0));
        assert_eq!(style.width, 3.0);
        assert_eq!(style.color.b, 1.0);
    }

    #[test]
    fn test_path_tessellator_new() {
        let tess = PathTessellator::new();
        assert!(true);
    }

    #[test]
    fn test_tessellate_rect() {
        let tess = PathTessellator::new();
        let result = tess.tessellate_rect(0.0, 0.0, 100.0, 100.0, None, None);

        // Should have 6 vertices (2 triangles)
        assert_eq!(result.vertices.len(), 6);
        assert_eq!(result.indices.len(), 6);
    }

    #[test]
    fn test_tessellate_rect_with_fill() {
        let tess = PathTessellator::new();
        let fill = FillStyle::new(Color::new(1.0, 0.0, 0.0, 1.0));
        let result = tess.tessellate_rect(0.0, 0.0, 100.0, 100.0, Some(fill), None);

        assert_eq!(result.vertices.len(), 6);
        // First vertex should have red color
        assert!((result.vertices[0].color[0] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_tessellate_ellipse() {
        let tess = PathTessellator::new();
        let fill = FillStyle::new(Color::new(0.0, 1.0, 0.0, 1.0));
        let result = tess.tessellate_ellipse(50.0, 50.0, 25.0, 25.0, Some(fill), None);

        // Should have center + 32 segments = 34 vertices
        assert_eq!(result.vertices.len(), 34);
        // Should have 32 triangles * 3 indices = 96 indices
        assert_eq!(result.indices.len(), 96);
    }

    #[test]
    fn test_tessellate_line() {
        let tess = PathTessellator::new();
        let stroke = StrokeStyle::new(2.0, Color::new(0.0, 0.0, 1.0, 1.0));
        let result = tess.tessellate_line(0.0, 0.0, 100.0, 100.0, stroke);

        // Should have 4 vertices (2 triangles for the line strip)
        assert_eq!(result.vertices.len(), 4);
        assert_eq!(result.indices.len(), 6);
    }

    #[test]
    fn test_tessellate_line_short() {
        let tess = PathTessellator::new();
        let stroke = StrokeStyle::new(2.0, Color::new(0.0, 0.0, 1.0, 1.0));
        // Very short line should return empty
        let result = tess.tessellate_line(0.0, 0.0, 0.0, 0.0, stroke);

        assert!(result.vertices.is_empty());
    }

    #[test]
    fn test_line_cap_variants() {
        assert_ne!(LineCap::Butt, LineCap::Round);
        assert_ne!(LineCap::Round, LineCap::Square);
        assert_ne!(LineCap::Square, LineCap::Butt);
    }

    #[test]
    fn test_line_join_variants() {
        assert_ne!(LineJoin::Miter, LineJoin::Round);
        assert_ne!(LineJoin::Round, LineJoin::Bevel);
        assert_ne!(LineJoin::Bevel, LineJoin::Miter);
    }

    #[test]
    fn test_fill_rule_variants() {
        assert_ne!(FillRule::NonZero, FillRule::EvenOdd);
    }

    #[test]
    fn test_tessellated_path_default() {
        let path = TessellatedPath::default();
        assert!(path.vertices.is_empty());
        assert!(path.indices.is_empty());
    }

    #[test]
    fn test_tessellate_shape_rect() {
        let tess = PathTessellator::new();
        let shape = Shape::rect(100.0, 50.0);
        let position = Position::new(100.0, 100.0);
        let fill = FillStyle::new(Color::new(1.0, 0.0, 0.0, 1.0));

        let result = tess.tessellate_shape(position, &shape, Some(fill), None);

        assert_eq!(result.0.vertices.len(), 6);
    }

    #[test]
    fn test_tessellate_shape_ellipse() {
        let tess = PathTessellator::new();
        let shape = Shape::ellipse(25.0, 25.0);
        let position = Position::new(100.0, 100.0);
        let fill = FillStyle::new(Color::new(0.0, 1.0, 0.0, 1.0));

        let result = tess.tessellate_shape(position, &shape, Some(fill), None);

        assert_eq!(result.0.vertices.len(), 34);
    }

    #[test]
    fn test_tessellate_shape_line() {
        let tess = PathTessellator::new();
        // Line is ShapeType::Line, but in our tessellate_shape we check for ShapeType::Line specifically
        let shape = Shape {
            shape_type: ShapeType::Line,
            width: 100.0,
            height: 2.0,
            rotation: 0.0,
        };
        let position = Position::new(50.0, 50.0);
        let stroke = StrokeStyle::new(2.0, Color::new(0.0, 0.0, 1.0, 1.0));

        let result = tess.tessellate_shape(position, &shape, None, Some(stroke));

        assert!(result.1.is_some());
        assert_eq!(result.1.unwrap().vertices.len(), 4);
    }

    #[test]
    fn test_vertex2d_creation() {
        let vertex = Vertex2D {
            position: [100.0, 200.0],
            uv: [0.5, 0.5],
            color: [1.0, 0.5, 0.25, 1.0],
        };

        assert_eq!(vertex.position, [100.0, 200.0]);
        assert_eq!(vertex.uv, [0.5, 0.5]);
        assert_eq!(vertex.color, [1.0, 0.5, 0.25, 1.0]);
    }
}
