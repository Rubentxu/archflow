// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Core - Path System with Bézier Curves
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 18
//
// GPU-based Bézier curve engine for vector paths:
// - Path commands for MoveTo, LineTo, QuadTo, CubicTo
// - GPU-ready structure for SDF-based curve rendering
// - Efficient bounding box calculation for culling
// ═══════════════════════════════════════════════════════════════════════════════

#![no_std]
#![warn(missing_docs)]
#![warn(clippy::all)]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use crate::math::{Color, Vec2};

/// Path command for building vector paths
///
/// These commands define a sequence of 2D drawing operations that can be
/// rendered on the GPU using SDF-based techniques.
#[derive(Clone, Debug, PartialEq)]
pub enum PathCommand {
    /// Move to a point without drawing (starts a new sub-path)
    MoveTo(Vec2),

    /// Draw a straight line from the current point to the target
    LineTo(Vec2),

    /// Draw a quadratic Bézier curve with one control point
    QuadTo {
        /// Control point that influences the curve shape
        control: Vec2,
        /// End point of the curve
        end: Vec2,
    },

    /// Draw a cubic Bézier curve with two control points
    CubicTo {
        /// First control point (influences start of curve)
        ctrl1: Vec2,
        /// Second control point (influences end of curve)
        ctrl2: Vec2,
        /// End point of the curve
        end: Vec2,
    },

    /// Close the current sub-path by drawing a line to the start
    Close,
}

impl PathCommand {
    /// Get the end point of this command (if applicable)
    #[must_use]
    pub const fn end_point(&self) -> Option<Vec2> {
        match self {
            PathCommand::MoveTo(p) => Some(*p),
            PathCommand::LineTo(p) => Some(*p),
            PathCommand::QuadTo { end, .. } => Some(*end),
            PathCommand::CubicTo { end, .. } => Some(*end),
            PathCommand::Close => None,
        }
    }

    /// Check if this command is a drawing command (vs. a move/close)
    #[must_use]
    pub const fn is_drawing(&self) -> bool {
        matches!(
            self,
            PathCommand::LineTo(_) | PathCommand::QuadTo { .. } | PathCommand::CubicTo { .. }
        )
    }
}

/// A vector path composed of path commands
///
/// Paths represent 2D vector graphics that can be rendered using GPU-accelerated
/// SDF techniques. The structure is designed for efficient transfer to GPU memory.
#[derive(Clone, Debug, PartialEq)]
pub struct Path {
    /// Sequence of path commands
    pub commands: Vec<PathCommand>,

    /// Stroke width in pixels
    pub stroke_width: f32,

    /// Stroke color
    pub stroke_color: Color,

    /// Fill color (None for no fill)
    pub fill_color: Option<Color>,

    /// Cached bounding box for efficient culling
    bounding_box: Option<(Vec2, Vec2)>,
}

impl Default for Path {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            stroke_width: 1.0,
            stroke_color: Color::rgb(255, 255, 255),
            fill_color: None,
            bounding_box: None,
        }
    }
}

impl Path {
    /// Create a new empty path
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new path with the given stroke color
    #[must_use]
    pub const fn with_stroke(color: Color) -> Self {
        Self {
            commands: Vec::new(),
            stroke_width: 1.0,
            stroke_color: color,
            fill_color: None,
            bounding_box: None,
        }
    }

    /// Create a new path with both stroke and fill
    #[must_use]
    pub const fn with_colors(stroke: Color, fill: Color) -> Self {
        Self {
            commands: Vec::new(),
            stroke_width: 1.0,
            stroke_color: stroke,
            fill_color: Some(fill),
            bounding_box: None,
        }
    }

    /// Set the stroke width
    #[inline]
    pub fn set_stroke_width(&mut self, width: f32) {
        self.stroke_width = width;
        self.invalidate_cache();
    }

    /// Add a move command
    #[inline]
    pub fn move_to(&mut self, x: f32, y: f32) -> &mut Self {
        self.commands.push(PathCommand::MoveTo(Vec2::new(x, y)));
        self.invalidate_cache();
        self
    }

    /// Add a line command
    #[inline]
    pub fn line_to(&mut self, x: f32, y: f32) -> &mut Self {
        self.commands.push(PathCommand::LineTo(Vec2::new(x, y)));
        self.invalidate_cache();
        self
    }

    /// Add a quadratic curve command
    #[inline]
    pub fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) -> &mut Self {
        self.commands.push(PathCommand::QuadTo {
            control: Vec2::new(cx, cy),
            end: Vec2::new(x, y),
        });
        self.invalidate_cache();
        self
    }

    /// Add a cubic curve command
    #[inline]
    pub fn cubic_to(
        &mut self,
        c1x: f32,
        c1y: f32,
        c2x: f32,
        c2y: f32,
        x: f32,
        y: f32,
    ) -> &mut Self {
        self.commands.push(PathCommand::CubicTo {
            ctrl1: Vec2::new(c1x, c1y),
            ctrl2: Vec2::new(c2x, c2y),
            end: Vec2::new(x, y),
        });
        self.invalidate_cache();
        self
    }

    /// Close the current sub-path
    #[inline]
    pub fn close(&mut self) -> &mut Self {
        self.commands.push(PathCommand::Close);
        self
    }

    /// Clear all commands
    #[inline]
    pub fn clear(&mut self) {
        self.commands.clear();
        self.invalidate_cache();
    }

    /// Get the number of commands
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if the path is empty
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Calculate the bounding box of this path
    #[must_use]
    pub fn bounding_box(&mut self) -> (Vec2, Vec2) {
        if let Some(bb) = self.bounding_box {
            return bb;
        }

        if self.commands.is_empty() {
            let origin = Vec2::new(0.0, 0.0);
            self.bounding_box = Some((origin, origin));
            return (origin, origin);
        }

        let mut min = Vec2::new(f32::MAX, f32::MAX);
        let mut max = Vec2::new(f32::MIN, f32::MIN);
        let mut current = Vec2::new(0.0, 0.0);

        for cmd in &self.commands {
            match cmd {
                PathCommand::MoveTo(p) => {
                    current = *p;
                    self.update_bounds(&mut min, &mut max, *p);
                }
                PathCommand::LineTo(p) => {
                    self.update_bounds(&mut min, &mut max, *p);
                    current = *p;
                }
                PathCommand::QuadTo { control, end } => {
                    // For quadratic curves, the bounding box includes the control point
                    self.update_bounds(&mut min, &mut max, *control);
                    self.update_bounds(&mut min, &mut max, *end);
                    current = *end;
                }
                PathCommand::CubicTo { ctrl1, ctrl2, end } => {
                    // For cubic curves, include both control points
                    self.update_bounds(&mut min, &mut max, *ctrl1);
                    self.update_bounds(&mut min, &mut max, *ctrl2);
                    self.update_bounds(&mut min, &mut max, *end);
                    current = *end;
                }
                PathCommand::Close => {
                    // Close doesn't extend the bounding box
                }
            }
        }

        let result = (min, max);
        self.bounding_box = Some(result);
        result
    }

    #[inline]
    fn update_bounds(&self, min: &mut Vec2, max: &mut Vec2, p: Vec2) {
        min.x = min.x.min(p.x);
        min.y = min.y.min(p.y);
        max.x = max.x.max(p.x);
        max.y = max.y.max(p.y);
    }

    #[inline]
    fn invalidate_cache(&mut self) {
        self.bounding_box = None;
    }
}

/// Builder for creating common path shapes
pub struct PathBuilder {
    path: Path,
}

impl Default for PathBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PathBuilder {
    /// Create a new path builder
    #[must_use]
    pub fn new() -> Self {
        Self { path: Path::new() }
    }

    /// Set the stroke color
    #[must_use]
    pub const fn stroke(mut self, color: Color) -> Self {
        self.path.stroke_color = color;
        self
    }

    /// Set the fill color
    #[must_use]
    pub const fn fill(mut self, color: Color) -> Self {
        self.path.fill_color = Some(color);
        self
    }

    /// Set the stroke width
    #[must_use]
    pub const fn width(mut self, width: f32) -> Self {
        self.path.stroke_width = width;
        self
    }

    /// Build a rectangle path
    #[must_use]
    pub fn rect(x: f32, y: f32, w: f32, h: f32) -> Path {
        let mut path = Path::new();
        path.move_to(x, y)
            .line_to(x + w, y)
            .line_to(x + w, y + h)
            .line_to(x, y + h)
            .close();
        path
    }

    /// Build a circle path using cubic Bézier curves
    ///
    /// Uses 4 cubic curves to approximate a circle with minimal error.
    /// The control point offset is 4/3 * tan(π/8) ≈ 0.55228475 for smooth corners.
    #[must_use]
    pub fn circle(cx: f32, cy: f32, r: f32) -> Path {
        let k = 0.55228475; // 4/3 * tan(π/8)
        let r_x = r * k;
        let r_y = r * k;

        let mut path = Path::new();
        path.move_to(cx - r, cy)
            .cubic_to(cx - r, cy - r_y, cx - r_x, cy - r, cx, cy - r)
            .cubic_to(cx + r_x, cy - r, cx + r, cy - r_y, cx + r, cy)
            .cubic_to(cx + r, cy + r_y, cx + r_x, cy + r, cx, cy + r)
            .cubic_to(cx - r_x, cy + r, cx - r, cy + r_y, cx - r, cy)
            .close();
        path
    }

    /// Build an ellipse path using cubic Bézier curves
    #[must_use]
    pub fn ellipse(cx: f32, cy: f32, rx: f32, ry: f32) -> Path {
        let k = 0.55228475;
        let rx_x = rx * k;
        let ry_x = ry * k;
        let rx_y = rx * k;
        let ry_y = ry * k;

        let mut path = Path::new();
        path.move_to(cx - rx, cy)
            .cubic_to(cx - rx, cy - ry_y, cx - rx_x, cy - ry, cx, cy - ry)
            .cubic_to(cx + rx_x, cy - ry, cx + rx, cy - ry_y, cx + rx, cy)
            .cubic_to(cx + rx, cy + ry_y, cx + rx_x, cy + ry, cx, cy + ry)
            .cubic_to(cx - rx_x, cy + ry, cx - rx, cy + ry_y, cx - rx, cy)
            .close();
        path
    }

    /// Build a rounded rectangle path
    #[must_use]
    pub fn rounded_rect(x: f32, y: f32, w: f32, h: f32, r: f32) -> Path {
        let k = 0.55228475;
        let r_x = r * k;
        let r_y = r * k;

        // Clamp radius to half the minimum dimension
        let r = r.min(w / 2.0).min(h / 2.0);

        let mut path = Path::new();
        path.move_to(x, y + r)
            .line_to(x, y + h - r)
            .cubic_to(x, y + h - r_y, x + r_x, y + h, x + r, y + h)
            .line_to(x + w - r, y + h)
            .cubic_to(x + w - r_x, y + h, x + w, y + h - r_y, x + w, y + h - r)
            .line_to(x + w, y + r)
            .cubic_to(x + w, y + r_y, x + w - r_x, y, x + w - r, y)
            .line_to(x + r, y)
            .cubic_to(x + r_x, y, x, y + r_y, x, y + r)
            .close();
        path
    }

    /// Build the final path
    #[must_use]
    pub fn build(self) -> Path {
        self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_creation() {
        let path = Path::new();
        assert!(path.is_empty());
        assert_eq!(path.len(), 0);
    }

    #[test]
    fn test_path_builder() {
        let path = PathBuilder::rect(10.0, 20.0, 100.0, 50.0);
        assert_eq!(path.len(), 5); // move, 3 lines, close
        assert!(path.commands[0].is_drawing() == false);
    }

    #[test]
    fn test_bounding_box_rect() {
        let mut path = PathBuilder::rect(10.0, 20.0, 100.0, 50.0);
        let (min, max) = path.bounding_box();
        assert_eq!(min.x, 10.0);
        assert_eq!(min.y, 20.0);
        assert_eq!(max.x, 110.0);
        assert_eq!(max.y, 70.0);
    }

    #[test]
    fn test_bounding_box_circle() {
        let mut path = PathBuilder::circle(50.0, 50.0, 25.0);
        let (min, max) = path.bounding_box();
        assert_eq!(min.x, 25.0);
        assert_eq!(min.y, 25.0);
        assert_eq!(max.x, 75.0);
        assert_eq!(max.y, 75.0);
    }

    #[test]
    fn test_command_end_point() {
        let cmd = PathCommand::LineTo(Vec2::new(10.0, 20.0));
        assert_eq!(cmd.end_point(), Some(Vec2::new(10.0, 20.0)));

        let cmd = PathCommand::QuadTo {
            control: Vec2::new(5.0, 5.0),
            end: Vec2::new(10.0, 10.0),
        };
        assert_eq!(cmd.end_point(), Some(Vec2::new(10.0, 10.0)));
    }

    #[test]
    fn test_command_is_drawing() {
        assert!(PathCommand::LineTo(Vec2::new(0.0, 0.0)).is_drawing());
        assert!(!PathCommand::MoveTo(Vec2::new(0.0, 0.0)).is_drawing());
        assert!(!PathCommand::Close.is_drawing());
    }

    #[test]
    fn test_path_chaining() {
        let mut path = Path::new();
        path.move_to(0.0, 0.0)
            .line_to(10.0, 0.0)
            .line_to(10.0, 10.0)
            .close();
        assert_eq!(path.len(), 4);
    }

    #[test]
    fn test_rounded_rect() {
        let path = PathBuilder::rounded_rect(0.0, 0.0, 100.0, 100.0, 10.0);
        assert_eq!(path.len(), 10); // 4 corners * 2 commands + 2 lines
    }

    #[test]
    fn test_ellipse() {
        let mut path = PathBuilder::ellipse(50.0, 50.0, 40.0, 30.0);
        let (min, max) = path.bounding_box();
        assert_eq!(min.x, 10.0);
        assert_eq!(min.y, 20.0);
        assert_eq!(max.x, 90.0);
        assert_eq!(max.y, 80.0);
    }

    #[test]
    fn test_path_clear() {
        let mut path = PathBuilder::rect(0.0, 0.0, 10.0, 10.0);
        assert_eq!(path.len(), 5);
        path.clear();
        assert!(path.is_empty());
    }

    #[test]
    fn test_bounding_box_cache() {
        let mut path = PathBuilder::rect(0.0, 0.0, 10.0, 10.0);
        let _ = path.bounding_box();
        // Second call should use cache
        let (min, max) = path.bounding_box();
        assert_eq!(min, Vec2::new(0.0, 0.0));
        assert_eq!(max, Vec2::new(10.0, 10.0));
    }
}
