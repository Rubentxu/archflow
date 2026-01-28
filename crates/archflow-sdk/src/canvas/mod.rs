//! Canvas module for the infinite canvas system
//!
//! This module provides the main Canvas type that combines viewport,
//! background, and layer management for the infinite canvas.

use crate::background::{BackgroundRenderer, GridConfig};
use crate::layers::{C4Level, Layer, LayerManager};
use crate::viewport::{Viewport, ViewportManager};
use archflow_core::{Color, EntityId, Rect, Transform, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents an operation that can be performed on a canvas
#[derive(Clone, Debug, PartialEq)]
pub enum CanvasOperation {
    /// Create a new shape
    CreateShape(Shape),
    /// Update an existing shape
    UpdateShape(EntityId, Shape, Shape),
    /// Delete a shape
    DeleteShape(EntityId, Shape),
}

/// Error type for canvas operations
#[derive(Debug, thiserror::Error)]
pub enum CanvasError {
    #[error("Shape not found: {0}")]
    ShapeNotFound(EntityId),
    #[error("Layer not found: {0}")]
    LayerNotFound(EntityId),
    #[error("Invalid viewport: {0}")]
    InvalidViewport(&'static str),
    #[error("Canvas error: {0}")]
    Other(&'static str),
}

/// Represents the geometric properties of a shape.
///
/// This struct encapsulates position, dimensions, and rotation, reducing
/// Connascence of Type by grouping related fields into a cohesive unit.
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
    ///
    /// # Arguments
    ///
    /// * `position` - Top-left corner position
    /// * `size` - Width and height (both must be non-negative)
    /// * `rotation` - Rotation in degrees
    #[inline]
    pub fn new(position: Vec2, size: Vec2, rotation: f32) -> Self {
        Self {
            position,
            size,
            rotation,
        }
    }

    /// Creates a geometry from individual position and dimension values.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate of top-left corner
    /// * `y` - Y coordinate of top-left corner
    /// * `width` - Shape width (must be non-negative)
    /// * `height` - Shape height (must be non-negative)
    /// * `rotation` - Rotation in degrees
    #[inline]
    pub fn from_components(x: f32, y: f32, width: f32, height: f32, rotation: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            size: Vec2::new(width.abs(), height.abs()),
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

    /// Applies a translation to the geometry.
    ///
    /// # Arguments
    ///
    /// * `delta` - Translation vector
    ///
    /// # Returns
    ///
    /// A new geometry with the position translated
    #[inline]
    pub fn translated(&self, delta: Vec2) -> Self {
        Self {
            position: self.position + delta,
            size: self.size,
            rotation: self.rotation,
        }
    }

    /// Checks if a point is within the geometry bounds.
    ///
    /// # Arguments
    ///
    /// * `point` - Point to check in canvas coordinates
    ///
    /// # Returns
    ///
    /// True if the point is inside the bounds
    #[inline]
    pub fn contains(&self, point: Vec2) -> bool {
        let bounds = self.bounds();
        point.x >= bounds.min.x
            && point.x <= bounds.max.x
            && point.y >= bounds.min.y
            && point.y <= bounds.max.y
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
    ///
    /// # Arguments
    ///
    /// * `color` - Stroke color (use `None` for no stroke)
    /// * `width` - Stroke width in pixels (must be non-negative)
    #[inline]
    pub fn new(color: Option<Color>, width: f32) -> Self {
        Self {
            color,
            width: width.max(0.0),
        }
    }

    /// Returns true if this stroke is visible (has color and positive width).
    #[inline]
    pub fn is_visible(&self) -> bool {
        self.color.is_some() && self.width > 0.0
    }
}

/// Represents the visual style properties of a shape.
///
/// This struct groups all visual appearance properties together,
/// improving type safety and reducing Connascence of Type.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShapeStyle {
    /// Fill color (transparent means no fill)
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
    ///
    /// # Arguments
    ///
    /// * `fill_color` - Fill color (use `Color::TRANSPARENT` for no fill)
    #[inline]
    pub fn with_fill(fill_color: Color) -> Self {
        Self {
            fill_color,
            stroke: Stroke::default(),
            opacity: 1.0,
        }
    }

    /// Creates a solid style with the given fill and stroke.
    ///
    /// # Arguments
    ///
    /// * `fill_color` - Fill color
    /// * `stroke_color` - Stroke color (use `None` for no stroke)
    /// * `stroke_width` - Stroke width in pixels
    #[inline]
    pub fn solid(fill_color: Color, stroke_color: Option<Color>, stroke_width: f32) -> Self {
        Self {
            fill_color,
            stroke: Stroke::new(stroke_color, stroke_width),
            opacity: 1.0,
        }
    }

    /// Sets the stroke properties.
    ///
    /// # Arguments
    ///
    /// * `stroke` - Stroke to use
    ///
    /// # Returns
    ///
    /// A new style with the updated stroke
    #[inline]
    pub fn with_stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = stroke;
        self
    }

    /// Sets the opacity.
    ///
    /// # Arguments
    ///
    /// * `opacity` - Opacity value (0.0 to 1.0)
    ///
    /// # Returns
    ///
    /// A new style with the updated opacity
    #[inline]
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }
}

/// Custom properties for shapes with type-safe access.
///
/// This replaces the generic `serde_json::Value` approach with specific
/// typed accessors for common property types.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ShapeProperties {
    inner: HashMap<String, PropertyValue>,
}

impl ShapeProperties {
    /// Creates a new empty properties collection.
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Gets a string property.
    ///
    /// # Arguments
    ///
    /// * `key` - Property key
    ///
    /// # Returns
    ///
    /// The string value if it exists
    #[inline]
    pub fn get_string(&self, key: &str) -> Option<&str> {
        match self.inner.get(key) {
            Some(PropertyValue::String(s)) => Some(s),
            _ => None,
        }
    }

    /// Sets a string property.
    ///
    /// # Arguments
    ///
    /// * `key` - Property key
    /// * `value` - String value
    #[inline]
    pub fn set_string(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.inner
            .insert(key.into(), PropertyValue::String(value.into()));
    }

    /// Gets a number property.
    ///
    /// # Arguments
    ///
    /// * `key` - Property key
    ///
    /// # Returns
    ///
    /// The number value if it exists
    #[inline]
    pub fn get_number(&self, key: &str) -> Option<f64> {
        match self.inner.get(key) {
            Some(PropertyValue::Number(n)) => Some(*n),
            _ => None,
        }
    }

    /// Sets a number property.
    ///
    /// # Arguments
    ///
    /// * `key` - Property key
    /// * `value` - Number value
    #[inline]
    pub fn set_number(&mut self, key: impl Into<String>, value: f64) {
        self.inner.insert(key.into(), PropertyValue::Number(value));
    }

    /// Gets a boolean property.
    ///
    /// # Arguments
    ///
    /// * `key` - Property key
    ///
    /// # Returns
    ///
    /// The boolean value if it exists
    #[inline]
    pub fn get_boolean(&self, key: &str) -> Option<bool> {
        match self.inner.get(key) {
            Some(PropertyValue::Boolean(b)) => Some(*b),
            _ => None,
        }
    }

    /// Sets a boolean property.
    ///
    /// # Arguments
    ///
    /// * `key` - Property key
    /// * `value` - Boolean value
    #[inline]
    pub fn set_boolean(&mut self, key: impl Into<String>, value: bool) {
        self.inner.insert(key.into(), PropertyValue::Boolean(value));
    }

    /// Gets the label property (common use case).
    ///
    /// # Returns
    ///
    /// The label if it exists
    #[inline]
    pub fn label(&self) -> Option<&str> {
        self.get_string("label")
    }

    /// Sets the label property.
    ///
    /// # Arguments
    ///
    /// * `label` - Label text
    #[inline]
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.set_string("label", label);
    }

    /// Gets the locked property (common use case).
    ///
    /// # Returns
    ///
    /// The locked state if set
    #[inline]
    pub fn locked(&self) -> Option<bool> {
        self.get_boolean("locked")
    }

    /// Sets the locked property.
    ///
    /// # Arguments
    ///
    /// * `locked` - Locked state
    #[inline]
    pub fn set_locked(&mut self, locked: bool) {
        self.set_boolean("locked", locked);
    }
}

/// Internal representation of property values.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum PropertyValue {
    String(String),
    Number(f64),
    Boolean(bool),
}

/// A shape in the canvas
///
/// This struct maintains both the new structured fields (geometry, style)
/// for type safety and the flat fields (x, y, width, etc.) for backwards
/// compatibility and serialization.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Shape {
    /// Unique shape ID
    pub id: EntityId,
    /// Shape type
    pub shape_type: ShapeType,
    /// Geometric properties (position, size, rotation) - NEW STRUCTURED API
    pub geometry: ShapeGeometry,
    /// Visual style properties - NEW STRUCTURED API
    pub style: ShapeStyle,
    /// Layer ID
    pub layer_id: EntityId,
    /// Whether the shape is selected
    pub selected: bool,
    /// Custom properties - NEW TYPED API
    pub properties: ShapeProperties,

    // Backwards compatibility fields (computed from geometry/style)
    /// X position (computed from geometry.position.x)
    pub x: f32,
    /// Y position (computed from geometry.position.y)
    pub y: f32,
    /// Width (computed from geometry.size.x)
    pub width: f32,
    /// Height (computed from geometry.size.y)
    pub height: f32,
    /// Rotation (computed from geometry.rotation)
    pub rotation: f32,
    /// Fill color (computed from style.fill_color)
    pub fill_color: Color,
    /// Stroke color (computed from style.stroke.color)
    pub stroke_color: Option<Color>,
    /// Stroke width (computed from style.stroke.width)
    pub stroke_width: f32,
    /// Opacity (computed from style.opacity)
    pub opacity: f32,
}

impl Shape {
    /// Creates a new rectangle shape
    #[inline]
    pub fn new_rectangle(x: f32, y: f32, width: f32, height: f32) -> Self {
        let geometry = ShapeGeometry::from_components(x, y, width, height, 0.0);
        let style = ShapeStyle::with_fill(Color::rgb(0.2, 0.4, 0.8));
        Self {
            // Backwards compatibility fields first
            id: EntityId::new(),
            shape_type: ShapeType::Rectangle,
            x,
            y,
            width,
            height,
            rotation: 0.0,
            fill_color: style.fill_color,
            stroke_color: style.stroke.color,
            stroke_width: style.stroke.width,
            opacity: style.opacity,
            // New structured fields at the end
            geometry,
            style,
            layer_id: EntityId::new(),
            selected: false,
            properties: ShapeProperties::new(),
        }
    }

    /// Creates a new ellipse shape
    #[inline]
    pub fn new_ellipse(x: f32, y: f32, radius_x: f32, radius_y: f32) -> Self {
        let geometry = ShapeGeometry::from_components(
            x - radius_x,
            y - radius_y,
            radius_x * 2.0,
            radius_y * 2.0,
            0.0,
        );
        let style = ShapeStyle::with_fill(Color::rgb(0.2, 0.6, 0.4));
        Self {
            // Backwards compatibility fields first
            id: EntityId::new(),
            shape_type: ShapeType::Ellipse,
            x: geometry.position.x,
            y: geometry.position.y,
            width: geometry.size.x,
            height: geometry.size.y,
            rotation: 0.0,
            fill_color: style.fill_color,
            stroke_color: style.stroke.color,
            stroke_width: style.stroke.width,
            opacity: style.opacity,
            // New structured fields at the end
            geometry,
            style,
            layer_id: EntityId::new(),
            selected: false,
            properties: ShapeProperties::new(),
        }
    }

    /// Creates a new line shape
    #[inline]
    pub fn new_line(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        let min_x = x1.min(x2);
        let min_y = y1.min(y2);
        let geometry =
            ShapeGeometry::from_components(min_x, min_y, (x2 - x1).abs(), (y2 - y1).abs(), 0.0);
        let style = ShapeStyle::solid(Color::TRANSPARENT, Some(Color::rgb(0.3, 0.3, 0.3)), 2.0);
        Self {
            // Backwards compatibility fields first
            id: EntityId::new(),
            shape_type: ShapeType::Line,
            x: geometry.position.x,
            y: geometry.position.y,
            width: geometry.size.x,
            height: geometry.size.y,
            rotation: 0.0,
            fill_color: style.fill_color,
            stroke_color: style.stroke.color,
            stroke_width: style.stroke.width,
            opacity: style.opacity,
            // New structured fields at the end
            geometry,
            style,
            layer_id: EntityId::new(),
            selected: false,
            properties: ShapeProperties::new(),
        }
    }

    /// Creates a new path shape
    #[inline]
    pub fn new_path(points: Vec<Vec2>) -> Self {
        let bounds = Self::calculate_bounds(&points);
        let geometry = ShapeGeometry::from_components(
            bounds.min.x,
            bounds.min.y,
            bounds.width(),
            bounds.height(),
            0.0,
        );
        let style = ShapeStyle::solid(Color::TRANSPARENT, Some(Color::rgb(0.3, 0.3, 0.3)), 2.0);
        Self {
            // Backwards compatibility fields first
            id: EntityId::new(),
            shape_type: ShapeType::Path,
            x: geometry.position.x,
            y: geometry.position.y,
            width: geometry.size.x,
            height: geometry.size.y,
            rotation: 0.0,
            fill_color: style.fill_color,
            stroke_color: style.stroke.color,
            stroke_width: style.stroke.width,
            opacity: style.opacity,
            // New structured fields at the end
            geometry,
            style,
            layer_id: EntityId::new(),
            selected: false,
            properties: ShapeProperties::new(),
        }
    }

    /// Updates the geometry and syncs backwards compatibility fields
    #[inline]
    pub fn update_geometry(&mut self, geometry: ShapeGeometry) {
        // Extract values first to avoid move issues
        let x = geometry.position.x;
        let y = geometry.position.y;
        let width = geometry.size.x;
        let height = geometry.size.y;
        let rotation = geometry.rotation;

        self.geometry = geometry;
        self.x = x;
        self.y = y;
        self.width = width;
        self.height = height;
        self.rotation = rotation;
    }

    /// Updates the style and syncs backwards compatibility fields
    #[inline]
    pub fn update_style(&mut self, style: ShapeStyle) {
        // Extract values first to avoid move issues
        let fill_color = style.fill_color;
        let stroke_color = style.stroke.color;
        let stroke_width = style.stroke.width;
        let opacity = style.opacity;

        self.style = style;
        self.fill_color = fill_color;
        self.stroke_color = stroke_color;
        self.stroke_width = stroke_width;
        self.opacity = opacity;
    }

    /// Calculates the bounding box for a list of points
    #[inline]
    fn calculate_bounds(points: &[Vec2]) -> Rect {
        if points.is_empty() {
            return Rect::default();
        }
        let min_x = points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
        let min_y = points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
        let max_x = points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
        let max_y = points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
        Rect::from_min_max(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y))
    }

    /// Gets the bounding rectangle
    #[inline]
    pub fn bounds(&self) -> Rect {
        self.geometry.bounds()
    }

    /// Checks if a point is inside the shape
    #[inline]
    pub fn contains_point(&self, point: Vec2) -> bool {
        self.bounds().contains(point)
    }
}

/// Type of shape
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShapeType {
    /// Rectangle shape
    Rectangle,
    /// Ellipse shape
    Ellipse,
    /// Line shape
    Line,
    /// Path/Polyline shape
    Path,
    /// Text shape
    Text,
    /// Image shape
    Image,
    /// Group of shapes
    Group,
}

impl Default for ShapeType {
    fn default() -> Self {
        Self::Rectangle
    }
}

/// Selection in the canvas
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Selection {
    /// Selected shape IDs
    pub shapes: Vec<EntityId>,
    /// Selection bounds in canvas coordinates
    pub bounds: Rect,
    /// Whether the selection is a box selection
    pub is_box: bool,
}

/// Changes to apply to a shape
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShapeChanges {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_color: Option<Color>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_color: Option<Option<Color>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,
}

/// The main Canvas type
///
/// The canvas combines viewport management, background rendering,
/// layer management, and shape handling into a single type.
#[derive(Debug)]
pub struct Canvas {
    /// Viewport manager
    viewport_manager: ViewportManager,
    /// Background renderer
    background_renderer: BackgroundRenderer,
    /// Layer manager
    layer_manager: LayerManager,
    /// All shapes indexed by ID
    shapes: HashMap<EntityId, Shape>,
    /// Current selection
    selection: Selection,
    /// Document bounds (for constraining viewport)
    document_bounds: Rect,
    /// Whether the canvas needs re-rendering
    dirty: bool,
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new(800.0, 600.0)
    }
}

impl Canvas {
    /// Creates a new canvas with the given dimensions
    #[inline]
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        let mut canvas = Self {
            viewport_manager: ViewportManager::new(screen_width, screen_height),
            background_renderer: BackgroundRenderer::new(),
            layer_manager: LayerManager::new(),
            shapes: HashMap::new(),
            selection: Selection::default(),
            document_bounds: Rect::default(),
            dirty: true,
        };

        // Create default layers for C4 model
        canvas
            .layer_manager
            .create_layer(C4Level::Context, "Context".to_string());
        canvas
            .layer_manager
            .create_layer(C4Level::Container, "Container".to_string());
        canvas
            .layer_manager
            .create_layer(C4Level::Component, "Component".to_string());
        canvas
            .layer_manager
            .create_layer(C4Level::Code, "Code".to_string());

        canvas
    }

    // === Viewport Operations ===

    /// Gets the current viewport
    #[inline]
    pub fn viewport(&self) -> Viewport {
        self.viewport_manager.viewport()
    }

    /// Sets the viewport
    #[inline]
    pub fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport_manager.set_viewport(viewport);
        self.dirty = true;
    }

    /// Pans the viewport
    #[inline]
    pub fn pan(&mut self, delta: Vec2) {
        self.viewport_manager.pan(delta);
        self.dirty = true;
    }

    /// Zooms at a screen point
    #[inline]
    pub fn zoom_at(&mut self, screen_point: Vec2, factor: f32) {
        self.viewport_manager.zoom_at(screen_point, factor);
        self.dirty = true;
    }

    /// Zooms in
    #[inline]
    pub fn zoom_in(&mut self, factor: f32, center: Option<Vec2>) {
        self.viewport_manager.zoom_in(factor, center);
        self.dirty = true;
    }

    /// Zooms out
    #[inline]
    pub fn zoom_out(&mut self, factor: f32, center: Option<Vec2>) {
        self.viewport_manager.zoom_out(factor, center);
        self.dirty = true;
    }

    /// Zooms to fit all content
    #[inline]
    pub fn zoom_to_fit(&mut self) {
        let content_bounds = self.get_content_bounds();
        self.viewport_manager.zoom_to_content(content_bounds);
        self.dirty = true;
    }

    /// Zooms to fit the selection
    #[inline]
    pub fn zoom_to_selection(&mut self) {
        if self.selection.shapes.is_empty() {
            return;
        }
        let bounds = self.get_selection_bounds();
        self.viewport_manager.zoom_to_fit(bounds);
        self.dirty = true;
    }

    /// Converts a screen coordinate to canvas coordinate
    #[inline]
    pub fn screen_to_canvas(&self, screen: Vec2) -> Vec2 {
        self.viewport_manager.screen_to_canvas(screen)
    }

    /// Converts a canvas coordinate to screen coordinate
    #[inline]
    pub fn canvas_to_screen(&self, canvas: Vec2) -> Vec2 {
        self.viewport_manager.canvas_to_screen(canvas)
    }

    // === Shape Operations ===

    /// Creates a rectangle
    #[inline]
    pub fn create_rectangle(&mut self, x: f32, y: f32, width: f32, height: f32) -> EntityId {
        let shape = Shape::new_rectangle(x, y, width, height);
        let id = shape.id;
        self.shapes.insert(id, shape);
        self.update_document_bounds();
        self.dirty = true;
        id
    }

    /// Creates an ellipse
    #[inline]
    pub fn create_ellipse(&mut self, x: f32, y: f32, radius_x: f32, radius_y: f32) -> EntityId {
        let shape = Shape::new_ellipse(x, y, radius_x, radius_y);
        let id = shape.id;
        self.shapes.insert(id, shape);
        self.update_document_bounds();
        self.dirty = true;
        id
    }

    /// Creates a line
    #[inline]
    pub fn create_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) -> EntityId {
        let shape = Shape::new_line(x1, y1, x2, y2);
        let id = shape.id;
        self.shapes.insert(id, shape);
        self.update_document_bounds();
        self.dirty = true;
        id
    }

    /// Creates a path
    #[inline]
    pub fn create_path(&mut self, points: Vec<Vec2>) -> EntityId {
        let shape = Shape::new_path(points);
        let id = shape.id;
        self.shapes.insert(id, shape);
        self.update_document_bounds();
        self.dirty = true;
        id
    }

    /// Gets a shape by ID
    #[inline]
    pub fn get_shape(&self, id: EntityId) -> Option<&Shape> {
        self.shapes.get(&id)
    }

    /// Gets a mutable shape by ID
    #[inline]
    pub fn get_shape_mut(&mut self, id: EntityId) -> Option<&mut Shape> {
        self.shapes.get_mut(&id)
    }

    /// Updates a shape
    #[inline]
    pub fn update_shape(&mut self, id: EntityId, changes: ShapeChanges) -> bool {
        if let Some(shape) = self.shapes.get_mut(&id) {
            if let Some(x) = changes.x {
                shape.x = x;
            }
            if let Some(y) = changes.y {
                shape.y = y;
            }
            if let Some(width) = changes.width {
                shape.width = width;
            }
            if let Some(height) = changes.height {
                shape.height = height;
            }
            if let Some(rotation) = changes.rotation {
                shape.rotation = rotation;
            }
            if let Some(fill_color) = changes.fill_color {
                shape.fill_color = fill_color;
            }
            if let Some(stroke_color) = changes.stroke_color {
                shape.stroke_color = stroke_color;
            }
            if let Some(stroke_width) = changes.stroke_width {
                shape.stroke_width = stroke_width;
            }
            if let Some(opacity) = changes.opacity {
                shape.opacity = opacity.clamp(0.0, 1.0);
            }
            self.update_document_bounds();
            self.dirty = true;
            true
        } else {
            false
        }
    }

    /// Deletes a shape
    #[inline]
    pub fn delete_shape(&mut self, id: EntityId) -> bool {
        if self.shapes.remove(&id).is_some() {
            self.selection.shapes.retain(|&shape_id| shape_id != id);
            self.update_document_bounds();
            self.dirty = true;
            true
        } else {
            false
        }
    }

    /// Gets all shapes
    #[inline]
    pub fn all_shapes(&self) -> Vec<&Shape> {
        self.shapes.values().collect()
    }

    /// Gets the content bounds
    #[inline]
    pub fn get_content_bounds(&self) -> Rect {
        if self.shapes.is_empty() {
            return Rect::from_min_max(Vec2::new(-1000.0, -1000.0), Vec2::new(1000.0, 1000.0));
        }

        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for shape in self.shapes.values() {
            min_x = min_x.min(shape.x);
            min_y = min_y.min(shape.y);
            max_x = max_x.max(shape.x + shape.width);
            max_y = max_y.max(shape.y + shape.height);
        }

        // Add some padding
        let padding = 50.0;
        Rect::from_min_max(
            Vec2::new(min_x - padding, min_y - padding),
            Vec2::new(max_x + padding, max_y + padding),
        )
    }

    // === Selection Operations ===

    /// Gets the current selection
    #[inline]
    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    /// Selects a single shape
    #[inline]
    pub fn select(&mut self, id: EntityId) {
        if self.shapes.contains_key(&id) {
            self.selection.shapes = vec![id];
            self.update_selection_bounds();
            self.dirty = true;
        }
    }

    /// Selects multiple shapes
    #[inline]
    pub fn select_multiple(&mut self, ids: Vec<EntityId>) {
        let valid_ids: Vec<EntityId> = ids
            .into_iter()
            .filter(|id| self.shapes.contains_key(id))
            .collect();
        self.selection.shapes = valid_ids;
        self.update_selection_bounds();
        self.dirty = true;
    }

    /// Selects all shapes
    #[inline]
    pub fn select_all(&mut self) {
        self.selection.shapes = self.shapes.keys().cloned().collect();
        self.update_selection_bounds();
        self.dirty = true;
    }

    /// Clears the selection
    #[inline]
    pub fn clear_selection(&mut self) {
        self.selection.shapes.clear();
        self.selection.bounds = Rect::default();
        self.dirty = true;
    }

    /// Gets the selection bounds
    #[inline]
    pub fn get_selection_bounds(&self) -> Rect {
        self.selection.bounds
    }

    // === Layer Operations ===

    /// Gets the layer manager
    #[inline]
    pub fn layer_manager(&self) -> &LayerManager {
        &self.layer_manager
    }

    /// Sets the current C4 level
    #[inline]
    pub fn set_c4_level(&mut self, level: C4Level) {
        self.layer_manager.set_current_level(level);
        self.dirty = true;
    }

    /// Gets the current C4 level
    #[inline]
    pub fn c4_level(&self) -> C4Level {
        self.layer_manager.current_level()
    }

    // === Background Operations ===

    /// Gets the background renderer
    #[inline]
    pub fn background_renderer(&self) -> &BackgroundRenderer {
        &self.background_renderer
    }

    /// Sets the grid configuration
    #[inline]
    pub fn set_grid_config(&mut self, config: GridConfig) {
        self.background_renderer.set_grid_config(config);
        self.dirty = true;
    }

    // === Render Control ===

    /// Marks the canvas as needing re-render
    #[inline]
    pub fn invalidate(&mut self) {
        self.dirty = true;
    }

    /// Checks if the canvas needs re-rendering
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Clears the dirty flag
    #[inline]
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Updates the document bounds
    fn update_document_bounds(&mut self) {
        self.document_bounds = self.get_content_bounds();
        self.viewport_manager
            .set_constrained_bounds(Some(self.document_bounds));
    }

    /// Updates the selection bounds
    fn update_selection_bounds(&mut self) {
        if self.selection.shapes.is_empty() {
            self.selection.bounds = Rect::default();
            return;
        }

        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for id in &self.selection.shapes {
            if let Some(shape) = self.shapes.get(id) {
                min_x = min_x.min(shape.x);
                min_y = min_y.min(shape.y);
                max_x = max_x.max(shape.x + shape.width);
                max_y = max_y.max(shape.y + shape.height);
            }
        }

        self.selection.bounds =
            Rect::from_min_max(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_creation() {
        let canvas = Canvas::new(800.0, 600.0);
        assert_eq!(canvas.layer_manager().layer_count(), 4);
    }

    #[test]
    fn test_create_rectangle() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 100.0, 200.0, 150.0);

        let shape = canvas.get_shape(id);
        assert!(shape.is_some());
        let shape = shape.unwrap();
        assert_eq!(shape.x, 100.0);
        assert_eq!(shape.y, 100.0);
        assert_eq!(shape.width, 200.0);
        assert_eq!(shape.height, 150.0);
    }

    #[test]
    fn test_create_ellipse() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_ellipse(200.0, 200.0, 50.0, 75.0);

        let shape = canvas.get_shape(id);
        assert!(shape.is_some());
        let shape = shape.unwrap();
        assert!((shape.x - 150.0).abs() < 0.01);
        assert!((shape.y - 125.0).abs() < 0.01);
    }

    #[test]
    fn test_selection() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id1 = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);
        let id2 = canvas.create_rectangle(200.0, 200.0, 50.0, 50.0);

        canvas.select(id1);
        assert_eq!(canvas.selection().shapes.len(), 1);
        assert_eq!(canvas.selection().shapes[0], id1);

        canvas.select_multiple(vec![id1, id2]);
        assert_eq!(canvas.selection().shapes.len(), 2);
    }

    #[test]
    fn test_delete_shape() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);

        assert!(canvas.delete_shape(id));
        assert!(canvas.get_shape(id).is_none());

        // Deleting non-existent shape should fail
        assert!(!canvas.delete_shape(id));
    }

    #[test]
    fn test_c4_level() {
        let mut canvas = Canvas::new(800.0, 600.0);
        assert_eq!(canvas.c4_level(), C4Level::Context);

        canvas.set_c4_level(C4Level::Container);
        assert_eq!(canvas.c4_level(), C4Level::Container);
    }

    #[test]
    fn test_zoom_to_fit() {
        let mut canvas = Canvas::new(800.0, 600.0);
        canvas.create_rectangle(0.0, 0.0, 100.0, 100.0);
        canvas.create_rectangle(200.0, 200.0, 100.0, 100.0);

        canvas.zoom_to_fit();
        assert!(canvas.viewport().zoom > 0.0);
    }
}
