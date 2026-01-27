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

/// A shape in the canvas
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Shape {
    /// Unique shape ID
    pub id: EntityId,
    /// Shape type
    pub shape_type: ShapeType,
    /// Position and dimensions
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    /// Rotation in degrees
    pub rotation: f32,
    /// Visual properties
    pub fill_color: Color,
    pub stroke_color: Option<Color>,
    pub stroke_width: f32,
    pub opacity: f32,
    /// Layer ID
    pub layer_id: Option<EntityId>,
    /// Whether the shape is selected
    pub selected: bool,
    /// Custom properties
    pub properties: HashMap<String, serde_json::Value>,
}

impl Shape {
    /// Creates a new rectangle shape
    #[inline]
    pub fn new_rectangle(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            id: EntityId::new(),
            shape_type: ShapeType::Rectangle,
            x,
            y,
            width,
            height,
            rotation: 0.0,
            fill_color: Color::rgb(0.2, 0.4, 0.8),
            stroke_color: None,
            stroke_width: 0.0,
            opacity: 1.0,
            layer_id: None,
            selected: false,
            properties: HashMap::new(),
        }
    }

    /// Creates a new ellipse shape
    #[inline]
    pub fn new_ellipse(x: f32, y: f32, radius_x: f32, radius_y: f32) -> Self {
        Self {
            id: EntityId::new(),
            shape_type: ShapeType::Ellipse,
            x: x - radius_x,
            y: y - radius_y,
            width: radius_x * 2.0,
            height: radius_y * 2.0,
            rotation: 0.0,
            fill_color: Color::rgb(0.2, 0.6, 0.4),
            stroke_color: None,
            stroke_width: 0.0,
            opacity: 1.0,
            layer_id: None,
            selected: false,
            properties: HashMap::new(),
        }
    }

    /// Creates a new line shape
    #[inline]
    pub fn new_line(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        let min_x = x1.min(x2);
        let min_y = y1.min(y2);
        Self {
            id: EntityId::new(),
            shape_type: ShapeType::Line,
            x: min_x,
            y: min_y,
            width: (x2 - x1).abs(),
            height: (y2 - y1).abs(),
            rotation: 0.0,
            fill_color: Color::TRANSPARENT,
            stroke_color: Some(Color::rgb(0.3, 0.3, 0.3)),
            stroke_width: 2.0,
            opacity: 1.0,
            layer_id: None,
            selected: false,
            properties: HashMap::new(),
        }
    }

    /// Creates a new path shape
    #[inline]
    pub fn new_path(points: Vec<Vec2>) -> Self {
        let bounds = Self::calculate_bounds(&points);
        Self {
            id: EntityId::new(),
            shape_type: ShapeType::Path,
            x: bounds.min.x,
            y: bounds.min.y,
            width: bounds.width(),
            height: bounds.height(),
            rotation: 0.0,
            fill_color: Color::TRANSPARENT,
            stroke_color: Some(Color::rgb(0.3, 0.3, 0.3)),
            stroke_width: 2.0,
            opacity: 1.0,
            layer_id: None,
            selected: false,
            properties: HashMap::new(),
        }
    }

    /// Calculates bounding box from points
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
        Rect::from_min_max(
            Vec2::new(self.x, self.y),
            Vec2::new(self.x + self.width, self.y + self.height),
        )
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
