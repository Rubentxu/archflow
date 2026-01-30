//! Canvas module for the infinite canvas system
//!
//! This module consolidates canvas functionality from `archflow-sdk`,
//! `archflow-spatial`, and `archflow-workspace` into a unified Canvas type.

use crate::{EntityId, Rect, Vec2, shapes::Shape, viewport::Viewport};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents an operation that can be performed on a canvas.
#[derive(Clone, Debug, PartialEq)]
pub enum CanvasOperation {
    /// Create a new shape
    CreateShape(Shape),
    /// Update an existing shape
    UpdateShape(EntityId, Shape, Shape),
    /// Delete a shape
    DeleteShape(EntityId, Shape),
}

/// Error type for canvas operations.
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

/// Changes to apply to a shape.
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
    pub fill_color: Option<crate::Color>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_color: Option<Option<crate::Color>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,
}

/// Selection in the canvas.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Selection {
    /// Selected shape IDs
    pub shapes: Vec<EntityId>,
}

impl Selection {
    /// Returns the number of selected shapes.
    #[inline]
    pub fn len(&self) -> usize {
        self.shapes.len()
    }

    /// Returns true if the selection is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.shapes.is_empty()
    }
}

/// The main Canvas type.
///
/// Consolidates viewport management, shape handling, and selection
/// from the old architecture into a single, coherent type.
#[derive(Debug)]
pub struct Canvas {
    /// Current viewport
    viewport: Viewport,
    /// All shapes indexed by ID
    shapes: HashMap<EntityId, Shape>,
    /// Current selection
    selection: Selection,
    /// Whether the canvas needs re-rendering
    dirty: bool,
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new(800.0, 600.0)
    }
}

impl Canvas {
    /// Creates a new canvas with the given dimensions.
    #[inline]
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            viewport: Viewport::new(screen_width, screen_height),
            shapes: HashMap::new(),
            selection: Selection::default(),
            dirty: true,
        }
    }

    // === Viewport Operations ===

    /// Gets the current viewport.
    #[inline]
    pub fn viewport(&self) -> Viewport {
        self.viewport
    }

    /// Sets the viewport.
    #[inline]
    pub fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
        self.dirty = true;
    }

    /// Pans the viewport.
    #[inline]
    pub fn pan(&mut self, delta: Vec2) {
        self.viewport.center = self.viewport.center + delta;
        self.dirty = true;
    }

    /// Zooms at a screen point.
    #[inline]
    pub fn zoom_at(&mut self, screen_point: Vec2, factor: f32) {
        let canvas_point = self.viewport.screen_to_canvas(screen_point);
        self.viewport.zoom *= factor;
        let new_screen_point = self.viewport.canvas_to_screen(canvas_point);
        self.viewport.center =
            self.viewport.center + (screen_point - new_screen_point) / self.viewport.zoom;
        self.dirty = true;
    }

    /// Zooms in.
    #[inline]
    pub fn zoom_in(&mut self, factor: f32, _center: Option<Vec2>) {
        self.viewport.zoom *= factor;
        self.dirty = true;
    }

    /// Zooms out.
    #[inline]
    pub fn zoom_out(&mut self, factor: f32, _center: Option<Vec2>) {
        self.viewport.zoom /= factor;
        self.dirty = true;
    }

    /// Zooms to fit all content.
    #[inline]
    pub fn zoom_to_fit(&mut self) {
        if self.shapes.is_empty() {
            return;
        }

        let content_bounds = self.get_content_bounds();
        let content_size = content_bounds.size();
        let viewport_size = Vec2::new(self.viewport.screen_width, self.viewport.screen_height);

        let zoom_x = viewport_size.x / content_size.x;
        let zoom_y = viewport_size.y / content_size.y;
        self.viewport.zoom = zoom_x.min(zoom_y) * 0.9; // 90% to leave margin

        self.viewport.center = content_bounds.center();
        self.dirty = true;
    }

    /// Converts a screen coordinate to canvas coordinate.
    #[inline]
    pub fn screen_to_canvas(&self, screen: Vec2) -> Vec2 {
        self.viewport.screen_to_canvas(screen)
    }

    /// Converts a canvas coordinate to screen coordinate.
    #[inline]
    pub fn canvas_to_screen(&self, canvas: Vec2) -> Vec2 {
        self.viewport.canvas_to_screen(canvas)
    }

    // === Shape Operations ===

    /// Creates a rectangle.
    #[inline]
    pub fn create_rectangle(&mut self, x: f32, y: f32, width: f32, height: f32) -> EntityId {
        let shape = Shape::new_rectangle(x, y, width, height);
        let id = shape.id;
        self.shapes.insert(id, shape);
        self.dirty = true;
        id
    }

    /// Creates an ellipse.
    #[inline]
    pub fn create_ellipse(&mut self, x: f32, y: f32, radius_x: f32, radius_y: f32) -> EntityId {
        let shape =
            Shape::new_rectangle(x - radius_x, y - radius_y, radius_x * 2.0, radius_y * 2.0);
        let id = shape.id;
        self.shapes.insert(id, shape);
        self.dirty = true;
        id
    }

    /// Gets a shape by ID.
    #[inline]
    pub fn get_shape(&self, id: EntityId) -> Option<&Shape> {
        self.shapes.get(&id)
    }

    /// Gets a mutable shape by ID.
    #[inline]
    pub fn get_shape_mut(&mut self, id: EntityId) -> Option<&mut Shape> {
        self.shapes.get_mut(&id)
    }

    /// Updates a shape.
    #[inline]
    pub fn update_shape(&mut self, id: EntityId, changes: ShapeChanges) -> bool {
        if let Some(shape) = self.shapes.get_mut(&id) {
            if let Some(x) = changes.x {
                shape.geometry.position.x = x;
            }
            if let Some(y) = changes.y {
                shape.geometry.position.y = y;
            }
            if let Some(width) = changes.width {
                shape.geometry.size.x = width;
            }
            if let Some(height) = changes.height {
                shape.geometry.size.y = height;
            }
            if let Some(rotation) = changes.rotation {
                shape.geometry.rotation = rotation;
            }
            if let Some(fill_color) = changes.fill_color {
                shape.style.fill_color = fill_color;
            }
            if let Some(stroke_color) = changes.stroke_color {
                shape.style.stroke.color = stroke_color;
            }
            if let Some(stroke_width) = changes.stroke_width {
                shape.style.stroke.width = stroke_width;
            }
            if let Some(opacity) = changes.opacity {
                shape.style.opacity = opacity.clamp(0.0, 1.0);
            }
            self.dirty = true;
            true
        } else {
            false
        }
    }

    /// Deletes a shape.
    #[inline]
    pub fn delete_shape(&mut self, id: EntityId) -> bool {
        if self.shapes.remove(&id).is_some() {
            self.selection.shapes.retain(|&shape_id| shape_id != id);
            self.dirty = true;
            true
        } else {
            false
        }
    }

    /// Gets all shapes.
    #[inline]
    pub fn all_shapes(&self) -> Vec<&Shape> {
        self.shapes.values().collect()
    }

    /// Gets the content bounds.
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
            let bounds = shape.bounds();
            min_x = min_x.min(bounds.min.x);
            min_y = min_y.min(bounds.min.y);
            max_x = max_x.max(bounds.max.x);
            max_y = max_y.max(bounds.max.y);
        }

        Rect::from_min_max(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y))
    }

    // === Selection Operations ===

    /// Gets the current selection.
    #[inline]
    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    /// Selects a single shape.
    #[inline]
    pub fn select(&mut self, id: EntityId) {
        if self.shapes.contains_key(&id) {
            self.selection.shapes = vec![id];
            self.dirty = true;
        }
    }

    /// Selects multiple shapes.
    #[inline]
    pub fn select_multiple(&mut self, ids: Vec<EntityId>) {
        let valid_ids: Vec<EntityId> = ids
            .into_iter()
            .filter(|id| self.shapes.contains_key(id))
            .collect();
        self.selection.shapes = valid_ids;
        self.dirty = true;
    }

    /// Selects all shapes.
    #[inline]
    pub fn select_all(&mut self) {
        self.selection.shapes = self.shapes.keys().cloned().collect();
        self.dirty = true;
    }

    /// Clears the selection.
    #[inline]
    pub fn clear_selection(&mut self) {
        self.selection.shapes.clear();
        self.dirty = true;
    }

    // === Render Control ===

    /// Marks the canvas as needing re-render.
    #[inline]
    pub fn invalidate(&mut self) {
        self.dirty = true;
    }

    /// Checks if the canvas needs re-rendering.
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Clears the dirty flag.
    #[inline]
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}

// Add Rect extension for size calculation
trait RectExt {
    fn size(&self) -> Vec2;
    fn center(&self) -> Vec2;
}

impl RectExt for Rect {
    #[inline]
    fn size(&self) -> Vec2 {
        self.max - self.min
    }

    #[inline]
    fn center(&self) -> Vec2 {
        self.min + self.size() / 2.0
    }
}
