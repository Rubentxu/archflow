//! Spatial indexing for efficient queries
//!
//! This module provides spatial indexing functionality from `archflow-spatial`.

use crate::{EntityId, Rect, Shape, Vec2};

/// Options for hit testing.
#[derive(Clone, Debug, Default)]
pub struct HitTestOptions {
    /// Whether to test only filled shapes
    pub fill_only: bool,
    /// Whether to include shapes in locked layers
    pub include_locked: bool,
}

/// Result of a hit test.
#[derive(Clone, Debug)]
pub struct HitTestResult {
    /// The hit shape ID
    pub entity_id: EntityId,
    /// The hit point in canvas coordinates
    pub point: Vec2,
}

/// Spatial index trait for efficient spatial queries.
pub trait SpatialIndex: Send + Sync {
    /// Inserts a shape into the index.
    fn insert(&mut self, shape: Shape) -> Result<(), String>;

    /// Removes a shape from the index.
    fn remove(&mut self, id: EntityId) -> Option<Shape>;

    /// Finds shapes at a given point.
    fn hit_test(&self, point: Vec2, options: &HitTestOptions) -> Vec<HitTestResult>;

    /// Finds shapes in a given rectangle.
    fn query_rect(&self, rect: Rect) -> Vec<EntityId>;

    /// Updates a shape in the index.
    fn update(&mut self, shape: Shape) -> Result<(), String>;
}
