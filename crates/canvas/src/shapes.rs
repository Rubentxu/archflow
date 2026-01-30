//! Shape types and geometry
//!
//! This module consolidates shape definitions from `archflow-primitives`
//! and `archflow-sdk` into a unified, coherent API.

use crate::{Color, EntityId, Rect, Vec2};
use serde::{Deserialize, Serialize};

/// Represents the geometric properties of a shape (position, size, rotation).
///
/// This reduces Connascence of Type by grouping related fields into a cohesive unit.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShapeGeometry {
    /// Position of the shape (top-left corner) in canvas coordinates
    pub position: Vec2,
    /// Width and height of the shape
    pub size: Vec2,
    /// Rotation in degrees (0.0 = no rotation)
    pub rotation: f32,
}

impl ShapeGeometry {
    /// Creates a new geometry with the given position, size, and rotation.
    #[inline]
    pub fn new(position: Vec2, size: Vec2, rotation: f32) -> Self {
        Self {
            position,
            size,
            rotation,
        }
    }

    /// Returns the bounding rectangle of the geometry.
    #[inline]
    pub fn bounds(&self) -> Rect {
        Rect::from_min_max(self.position, self.position + self.size)
    }

    /// Returns the center point of the geometry.
    #[inline]
    pub fn center(&self) -> Vec2 {
        self.position + self.size / 2.0
    }
}

/// Represents stroke properties for shapes.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stroke {
    /// Stroke color (None means no stroke)
    pub color: Option<Color>,
    /// Stroke width in pixels
    pub width: f32,
}

impl Default for Stroke {
    fn default() -> Self {
        Self {
            color: None,
            width: 0.0,
        }
    }
}

impl Stroke {
    /// Creates a new stroke with the given color and width.
    #[inline]
    pub fn new(color: Option<Color>, width: f32) -> Self {
        Self {
            color,
            width: width.max(0.0),
        }
    }

    /// Returns true if this stroke is visible.
    #[inline]
    pub fn is_visible(&self) -> bool {
        self.color.is_some() && self.width > 0.0
    }
}

/// Represents the visual style properties of a shape.
///
/// Groups all visual appearance properties together, improving type safety
/// and reducing Connascence of Type.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShapeStyle {
    /// Fill color
    pub fill_color: Color,
    /// Stroke properties
    pub stroke: Stroke,
    /// Opacity (0.0 = fully transparent, 1.0 = fully opaque)
    pub opacity: f32,
}

impl Default for ShapeStyle {
    fn default() -> Self {
        Self {
            fill_color: Color::TRANSPARENT,
            stroke: Stroke::default(),
            opacity: 1.0,
        }
    }
}

impl ShapeStyle {
    /// Creates a new style with the given fill color.
    #[inline]
    pub fn with_fill(fill_color: Color) -> Self {
        Self {
            fill_color,
            stroke: Stroke::default(),
            opacity: 1.0,
        }
    }

    /// Creates a solid style with the given fill and stroke.
    #[inline]
    pub fn solid(fill_color: Color, stroke_color: Option<Color>, stroke_width: f32) -> Self {
        Self {
            fill_color,
            stroke: Stroke::new(stroke_color, stroke_width),
            opacity: 1.0,
        }
    }
}

/// Type of shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShapeType {
    Rectangle,
    Ellipse,
    Line,
    Path,
    Text,
    Image,
    Group,
}

impl Default for ShapeType {
    fn default() -> Self {
        Self::Rectangle
    }
}

/// A shape in the canvas.
///
/// Consolidates shape concepts from `archflow-primitives` and `archflow-sdk`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Shape {
    /// Unique shape ID
    pub id: EntityId,
    /// Shape type
    pub shape_type: ShapeType,
    /// Geometric properties
    pub geometry: ShapeGeometry,
    /// Visual style properties
    pub style: ShapeStyle,
    /// Layer ID
    pub layer_id: EntityId,
    /// Whether the shape is selected
    pub selected: bool,
}

impl Shape {
    /// Creates a new rectangle shape.
    #[inline]
    pub fn new_rectangle(x: f32, y: f32, width: f32, height: f32) -> Self {
        let geometry = ShapeGeometry::from_components(x, y, width, height, 0.0);
        let style = ShapeStyle::with_fill(Color::rgb(0.2, 0.4, 0.8));
        Self {
            id: EntityId::new(),
            shape_type: ShapeType::Rectangle,
            geometry,
            style,
            layer_id: EntityId::new(),
            selected: false,
        }
    }

    /// Gets the shape type.
    #[inline]
    pub fn shape_type(&self) -> ShapeType {
        self.shape_type
    }

    /// Gets the bounding rectangle.
    #[inline]
    pub fn bounds(&self) -> Rect {
        self.geometry.bounds()
    }

    /// Checks if a point is inside the shape.
    #[inline]
    pub fn contains_point(&self, point: Vec2) -> bool {
        self.bounds().contains(point)
    }
}

// Extension for ShapeGeometry to support from_components
impl ShapeGeometry {
    #[inline]
    pub fn from_components(x: f32, y: f32, width: f32, height: f32, rotation: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            size: Vec2::new(width.abs(), height.abs()),
            rotation,
        }
    }
}
