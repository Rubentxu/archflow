//! Shape definitions for the ArchFlow demo
//!
//! This module defines the shape types and storage used by the demo.
//! Designed for efficient rendering via Canvas 2D API.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Global counter for generating unique shape IDs
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Types of shapes supported by the demo
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapeType {
    Rectangle,
    Ellipse,
    Line,
}

/// Unique identifier for a shape
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ShapeId(pub u64);

impl ShapeId {
    /// Generates the next unique shape ID
    pub fn next() -> Self {
        ShapeId(NEXT_ID.fetch_add(1, Ordering::SeqCst))
    }
}

/// A renderable shape in the demo
#[derive(Clone, Debug)]
pub struct Shape {
    pub id: ShapeId,
    pub shape_type: ShapeType,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: [u8; 4],
    pub rotation: f32,
}

impl Shape {
    /// Converts the shape's color to CSS rgba() string
    pub fn color_as_css(&self) -> String {
        format!(
            "rgba({},{},{},{})",
            self.color[0], self.color[1], self.color[2], self.color[3]
        )
    }

    /// Checks if a point is inside this shape's bounds
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }

    /// Returns the center point of the shape
    pub fn center(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

/// Remote cursor representation for collaboration demo
#[derive(Clone, Debug)]
pub struct RemoteCursor {
    pub x: f64,
    pub y: f64,
    pub name: String,
    pub color: [u8; 4],
}

impl RemoteCursor {
    /// Creates a new remote cursor with a color based on the name
    pub fn new(x: f64, y: f64, name: &str) -> Self {
        let hash = name.bytes().fold(0u64, |acc, b| acc * 31 + b as u64);
        Self {
            x,
            y,
            name: name.to_string(),
            color: [
                ((hash >> 24) & 0xFF) as u8,
                ((hash >> 16) & 0xFF) as u8,
                ((hash >> 8) & 0xFF) as u8,
                200,
            ],
        }
    }
}

/// Storage for all shapes in the demo
///
/// Maintains shapes in insertion order for consistent rendering.
#[derive(Clone, Debug, Default)]
pub struct ShapeStore {
    shapes: HashMap<ShapeId, Shape>,
    order: Vec<ShapeId>,
}

impl ShapeStore {
    /// Creates a new empty shape store
    pub fn new() -> Self {
        Self {
            shapes: HashMap::new(),
            order: Vec::new(),
        }
    }

    /// Adds a shape to the store
    pub fn add(&mut self, shape: Shape) -> ShapeId {
        let id = shape.id;
        self.shapes.insert(id, shape.clone());
        self.order.push(id);
        id
    }

    /// Removes a shape from the store
    pub fn remove(&mut self, id: ShapeId) -> Option<Shape> {
        self.order.retain(|&oid| oid != id);
        self.shapes.remove(&id)
    }

    /// Gets a reference to a shape
    pub fn get(&self, id: ShapeId) -> Option<&Shape> {
        self.shapes.get(&id)
    }

    /// Gets a mutable reference to a shape
    pub fn get_mut(&mut self, id: ShapeId) -> Option<&mut Shape> {
        self.shapes.get_mut(&id)
    }

    /// Checks if a shape exists
    pub fn contains(&self, id: ShapeId) -> bool {
        self.shapes.contains_key(&id)
    }

    /// Iterates over all shapes in insertion order
    pub fn iter(&self) -> impl Iterator<Item = &Shape> {
        self.order.iter().filter_map(|id| self.shapes.get(id))
    }

    /// Returns the number of shapes
    pub fn count(&self) -> usize {
        self.shapes.len()
    }

    /// Finds the topmost shape at a given point
    ///
    /// Searches in reverse order (top to bottom in rendering).
    pub fn find_at_point(&self, x: f64, y: f64) -> Option<ShapeId> {
        for id in self.order.iter().rev() {
            if let Some(shape) = self.shapes.get(id) {
                if shape.contains(x, y) {
                    return Some(*id);
                }
            }
        }
        None
    }

    /// Clears all shapes
    pub fn clear(&mut self) {
        self.shapes.clear();
        self.order.clear();
    }
}
