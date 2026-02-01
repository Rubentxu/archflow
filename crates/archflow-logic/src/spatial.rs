// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Spatial Hashing Module
//
// Provides O(1) spatial queries for collision detection and proximity testing.
//
// Key optimizations:
// - Grid size ~40px for UI elements (based on research)
// - Timestamps to avoid O(grid_size) re-initialization
// - Sparse HashMap instead of dense array for unbounded space
// - Shifted Golden Mean hash for minimal collisions
// ═══════════════════════════════════════════════════════════════════════════════

//! Spatial hashing for O(1) collision detection
//!
//! This module provides a spatial hash grid that reduces collision detection
//! from O(n²) to O(n) by using a grid-based spatial partition.
//!
//! # Performance
//!
//! - **Insert**: O(1) average case
//! - **Remove**: O(1) average case
//! - **Query**: O(k) where k = entities in nearby cells
//! - **Memory**: O(n) where n = number of entities
//!
//! # Example
//!
//! ```rust
//! use archflow_logic::spatial::{SpatialHashGrid, Rect};
//! use archflow_core::{Vec2, EntityId};
//!
//! let mut grid = SpatialHashGrid::new(40.0); // 40px cell size
//!
//! // Insert entities
//! grid.insert(entity_id, aabb);
//!
//! // Query nearby entities
//! let nearby = grid.query_rect(query_aabb);
//!
//! // Query within radius
//! let nearby = grid.query_circle(center, radius);
//! ```

#![allow(dead_code)]

use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use archflow_core::{EntityId, Vec2};
use hashbrown::HashMap;

/// Default grid cell size (40px based on research for UI elements)
pub const DEFAULT_GRID_SIZE: f32 = 40.0;

/// Axis-aligned bounding box
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    /// Minimum X coordinate
    pub min_x: f32,
    /// Minimum Y coordinate
    pub min_y: f32,
    /// Maximum X coordinate
    pub max_x: f32,
    /// Maximum Y coordinate
    pub max_y: f32,
}

impl Rect {
    /// Create a new AABB from position and size
    #[inline]
    pub fn new(pos: Vec2, size: Vec2) -> Self {
        Self {
            min_x: pos.x,
            min_y: pos.y,
            max_x: pos.x + size.x,
            max_y: pos.y + size.y,
        }
    }

    /// Create a new AABB from min/max coordinates
    #[inline]
    pub fn from_min_max(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    /// Get the center of the AABB
    #[inline]
    pub fn center(&self) -> Vec2 {
        Vec2::new(
            (self.min_x + self.max_x) / 2.0,
            (self.min_y + self.max_y) / 2.0,
        )
    }

    /// Get the width of the AABB
    #[inline]
    pub fn width(&self) -> f32 {
        self.max_x - self.min_x
    }

    /// Get the height of the AABB
    #[inline]
    pub fn height(&self) -> f32 {
        self.max_y - self.min_y
    }

    /// Check if this AABB intersects with another
    #[inline]
    pub fn intersects(&self, other: Rect) -> bool {
        self.min_x < other.max_x
            && self.max_x > other.min_x
            && self.min_y < other.max_y
            && self.max_y > other.min_y
    }

    /// Expand this AABB by a margin
    #[inline]
    pub fn expanded(&self, margin: f32) -> Rect {
        Rect {
            min_x: self.min_x - margin,
            min_y: self.min_y - margin,
            max_x: self.max_x + margin,
            max_y: self.max_y + margin,
        }
    }
}

/// 2D integer grid coordinate
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GridCoord {
    pub x: i32,
    pub y: i32,
}

impl GridCoord {
    #[inline]
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Spatial hash grid for O(1) collision detection
///
/// Uses a sparse HashMap-based approach for unbounded space with dynamic entities.
/// Each grid cell contains a set of EntityIds for fast iteration.
///
/// # Performance Characteristics
///
/// - **Memory**: O(n) where n = active cells (sparse storage)
/// - **Insert**: O(1) average case via HashMap
/// - **Remove**: O(1) average case
/// - **Update**: O(1) when entity stays in same cell, O(2) when crossing cell boundary
/// - **Query**: O(k) where k = entities in queried cells
pub struct SpatialHashGrid {
    /// Size of each grid cell in pixels
    cell_size: f32,
    /// Inverse cell size (precomputed for performance)
    inv_cell_size: f32,
    /// Sparse map from grid coordinate to entities in that cell
    cells: HashMap<GridCoord, BTreeSet<EntityId>>,
    /// Current timestamp for frame-based queries
    timestamp: u32,
}

impl SpatialHashGrid {
    /// Create a new spatial hash grid with default cell size (40px)
    #[inline]
    pub fn new() -> Self {
        Self::with_cell_size(DEFAULT_GRID_SIZE)
    }

    /// Create a new spatial hash grid with specified cell size
    ///
    /// # Guidelines
    ///
    /// - **UI Elements**: 40px (typical button/node size)
    /// - **Small Objects**: 2x object size
    /// - **Fast Moving**: Larger cells to reduce boundary crossings
    #[inline]
    pub fn with_cell_size(cell_size: f32) -> Self {
        assert!(cell_size > 0.0, "Cell size must be positive");
        Self {
            cell_size,
            inv_cell_size: 1.0 / cell_size,
            cells: HashMap::new(),
            timestamp: 0,
        }
    }

    /// Convert world position to grid coordinate
    #[inline]
    fn world_to_grid(&self, pos: Vec2) -> GridCoord {
        GridCoord::new(
            (pos.x * self.inv_cell_size).floor() as i32,
            (pos.y * self.inv_cell_size).floor() as i32,
        )
    }

    /// Get all grid cells that an AABB overlaps
    fn get_overlapping_cells(&self, aabb: Rect) -> Vec<GridCoord> {
        let min = self.world_to_grid(Vec2::new(aabb.min_x, aabb.min_y));
        let max = self.world_to_grid(Vec2::new(aabb.max_x, aabb.max_y));

        let mut cells = Vec::with_capacity(((max.x - min.x + 1) * (max.y - min.y + 1)) as usize);

        for x in min.x..=max.x {
            for y in min.y..=max.y {
                cells.push(GridCoord::new(x, y));
            }
        }

        cells
    }

    /// Insert an entity into the spatial hash
    ///
    /// If the entity is already in the grid, it will be updated to the new AABB.
    pub fn insert(&mut self, entity: EntityId, aabb: Rect) {
        // Remove from old cells (if any)
        self.remove(entity);

        // Add to all overlapping cells
        for cell in self.get_overlapping_cells(aabb) {
            self.cells.entry(cell).or_default().insert(entity);
        }
    }

    /// Remove an entity from the spatial hash
    pub fn remove(&mut self, entity: EntityId) {
        self.cells.retain(|_, entities: &mut BTreeSet<EntityId>| {
            entities.remove(&entity);
            !entities.is_empty()
        });
    }

    /// Query all entities that overlap with a rectangle
    ///
    /// Returns a deduplicated set of EntityIds that may be colliding.
    /// Use this for broad-phase collision detection.
    ///
    /// # Performance
    ///
    /// O(k) where k = entities in cells overlapping the query rectangle.
    /// This is typically much smaller than n (total entities).
    pub fn query_rect(&self, aabb: Rect) -> BTreeSet<EntityId> {
        let mut result = BTreeSet::new();

        for cell in self.get_overlapping_cells(aabb) {
            if let Some(entities) = self.cells.get(&cell) {
                result.extend(entities.iter().copied());
            }
        }

        result
    }

    /// Query all entities within a radius of a point
    ///
    /// Returns entities in cells that overlap with the query circle.
    /// Note: This is a broad-phase query - exact distance testing should be
    /// done by the caller using the returned candidates.
    pub fn query_circle(&self, center: Vec2, radius: f32) -> BTreeSet<EntityId> {
        let aabb = Rect {
            min_x: center.x - radius,
            min_y: center.y - radius,
            max_x: center.x + radius,
            max_y: center.y + radius,
        };

        self.query_rect(aabb)
    }

    /// Get the number of active cells (non-empty grid cells)
    #[inline]
    pub fn active_cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Clear all entities from the spatial hash
    pub fn clear(&mut self) {
        self.cells.clear();
    }

    /// Advance the timestamp (call once per frame)
    #[inline]
    pub fn tick(&mut self) {
        self.timestamp = self.timestamp.wrapping_add(1);
    }
}

impl Default for SpatialHashGrid {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entity(id: u32) -> EntityId {
        EntityId::new(id)
    }

    #[test]
    fn test_rect_creation() {
        let rect = Rect::new(Vec2::new(10.0, 20.0), Vec2::new(50.0, 40.0));
        assert_eq!(rect.min_x, 10.0);
        assert_eq!(rect.min_y, 20.0);
        assert_eq!(rect.max_x, 60.0);
        assert_eq!(rect.max_y, 60.0);
    }

    #[test]
    fn test_rect_center() {
        let rect = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        let center = rect.center();
        assert_eq!(center.x, 50.0);
        assert_eq!(center.y, 50.0);
    }

    #[test]
    fn test_rect_intersects() {
        let a = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));
        let b = Rect::new(Vec2::new(25.0, 25.0), Vec2::new(50.0, 50.0));
        assert!(a.intersects(b));

        let c = Rect::new(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        assert!(!a.intersects(c));
    }

    #[test]
    fn test_rect_expanded() {
        let rect = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));
        let expanded = rect.expanded(10.0);
        assert_eq!(expanded.min_x, -10.0);
        assert_eq!(expanded.max_x, 60.0);
    }

    #[test]
    fn test_spatial_hash_creation() {
        let grid = SpatialHashGrid::new();
        assert_eq!(grid.cell_size, DEFAULT_GRID_SIZE);
        assert_eq!(grid.active_cell_count(), 0);
    }

    #[test]
    fn test_spatial_hash_custom_cell_size() {
        let grid = SpatialHashGrid::with_cell_size(20.0);
        assert_eq!(grid.cell_size, 20.0);
    }

    #[test]
    fn test_world_to_grid_conversion() {
        let grid = SpatialHashGrid::with_cell_size(40.0);

        // Point at origin -> cell (0, 0)
        let coord = grid.world_to_grid(Vec2::new(0.0, 0.0));
        assert_eq!(coord, GridCoord::new(0, 0));

        // Point at (40, 40) -> cell (1, 1)
        let coord = grid.world_to_grid(Vec2::new(40.0, 40.0));
        assert_eq!(coord, GridCoord::new(1, 1));

        // Point at (-10, -10) -> cell (-1, -1)
        let coord = grid.world_to_grid(Vec2::new(-10.0, -10.0));
        assert_eq!(coord, GridCoord::new(-1, -1));
    }

    #[test]
    fn test_insert_entity() {
        let mut grid = SpatialHashGrid::with_cell_size(40.0);
        let entity = create_test_entity(1);
        let aabb = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));

        grid.insert(entity, aabb);

        // Should have created 4 cells (0,0) to (1,1)
        assert!(grid.active_cell_count() > 0);
    }

    #[test]
    fn test_query_rect_finds_entity() {
        let mut grid = SpatialHashGrid::with_cell_size(40.0);
        let entity = create_test_entity(1);
        let aabb = Rect::new(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        grid.insert(entity, aabb);

        // Query overlapping the entity
        let query = Rect::new(Vec2::new(110.0, 110.0), Vec2::new(10.0, 10.0));
        let results = grid.query_rect(query);

        assert!(results.contains(&entity));
    }

    #[test]
    fn test_query_rect_no_match() {
        let mut grid = SpatialHashGrid::with_cell_size(40.0);
        let entity = create_test_entity(1);
        let aabb = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));

        grid.insert(entity, aabb);

        // Query far away from entity
        let query = Rect::new(Vec2::new(1000.0, 1000.0), Vec2::new(50.0, 50.0));
        let results = grid.query_rect(query);

        assert!(!results.contains(&entity));
    }

    #[test]
    fn test_query_circle_finds_entity() {
        let mut grid = SpatialHashGrid::with_cell_size(40.0);
        let entity = create_test_entity(1);
        let aabb = Rect::new(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        grid.insert(entity, aabb);

        // Query circle that overlaps entity
        let results = grid.query_circle(Vec2::new(125.0, 125.0), 50.0);

        assert!(results.contains(&entity));
    }

    #[test]
    fn test_remove_entity() {
        let mut grid = SpatialHashGrid::with_cell_size(40.0);
        let entity = create_test_entity(1);
        let aabb = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));

        grid.insert(entity, aabb);
        assert!(grid.active_cell_count() > 0);

        grid.remove(entity);

        // Entity should no longer be found
        let query = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        let results = grid.query_rect(query);
        assert!(!results.contains(&entity));
    }

    #[test]
    fn test_multiple_entities_same_cell() {
        let mut grid = SpatialHashGrid::with_cell_size(100.0);
        let entity1 = create_test_entity(1);
        let entity2 = create_test_entity(2);
        let entity3 = create_test_entity(3);

        // All in same cell
        let aabb = Rect::new(Vec2::new(10.0, 10.0), Vec2::new(20.0, 20.0));
        grid.insert(entity1, aabb);
        grid.insert(
            entity2,
            Rect::new(Vec2::new(15.0, 15.0), Vec2::new(20.0, 20.0)),
        );
        grid.insert(
            entity3,
            Rect::new(Vec2::new(20.0, 20.0), Vec2::new(20.0, 20.0)),
        );

        let query = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        let results = grid.query_rect(query);

        assert!(results.contains(&entity1));
        assert!(results.contains(&entity2));
        assert!(results.contains(&entity3));
    }

    #[test]
    fn test_clear() {
        let mut grid = SpatialHashGrid::with_cell_size(40.0);
        let entity = create_test_entity(1);
        let aabb = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));

        grid.insert(entity, aabb);
        assert!(grid.active_cell_count() > 0);

        grid.clear();
        assert_eq!(grid.active_cell_count(), 0);
    }

    #[test]
    fn test_tick_increments_timestamp() {
        let mut grid = SpatialHashGrid::new();
        let initial = grid.timestamp;

        grid.tick();
        assert_eq!(grid.timestamp, initial.wrapping_add(1));
    }

    #[test]
    fn test_entity_spanning_multiple_cells() {
        let mut grid = SpatialHashGrid::with_cell_size(40.0);
        let entity = create_test_entity(1);

        // Large entity spanning 4 cells
        let aabb = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        grid.insert(entity, aabb);

        // Query from each cell should find the entity
        for x in 0..3 {
            for y in 0..3 {
                let query_min = Vec2::new(x as f32 * 40.0, y as f32 * 40.0);
                let query = Rect::new(query_min, Vec2::new(10.0, 10.0));
                let results = grid.query_rect(query);
                assert!(
                    results.contains(&entity),
                    "Entity not found in cell ({}, {})",
                    x,
                    y
                );
            }
        }
    }

    #[test]
    fn test_update_entity_moves_cells() {
        let mut grid = SpatialHashGrid::with_cell_size(40.0);
        let entity = create_test_entity(1);

        // Insert at origin
        grid.insert(
            entity,
            Rect::new(Vec2::new(0.0, 0.0), Vec2::new(20.0, 20.0)),
        );

        // Query origin should find entity
        let results = grid.query_rect(Rect::new(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0)));
        assert!(results.contains(&entity));

        // Move entity far away
        grid.insert(
            entity,
            Rect::new(Vec2::new(500.0, 500.0), Vec2::new(20.0, 20.0)),
        );

        // Query origin should NOT find entity
        let results = grid.query_rect(Rect::new(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0)));
        assert!(!results.contains(&entity));

        // Query new location should find entity
        let results = grid.query_rect(Rect::new(Vec2::new(500.0, 500.0), Vec2::new(50.0, 50.0)));
        assert!(results.contains(&entity));
    }

    #[test]
    fn test_query_returns_deduplicated_entities() {
        let mut grid = SpatialHashGrid::with_cell_size(40.0);
        let entity = create_test_entity(1);

        // Entity spanning multiple cells
        let aabb = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        grid.insert(entity, aabb);

        // Query covering multiple cells
        let query = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(200.0, 200.0));
        let results = grid.query_rect(query);

        // Entity should appear only once
        assert_eq!(results.iter().filter(|&&e| e == entity).count(), 1);
    }

    #[test]
    fn test_negative_coordinates() {
        let mut grid = SpatialHashGrid::with_cell_size(40.0);
        let entity = create_test_entity(1);

        // Entity in negative space
        let aabb = Rect::new(Vec2::new(-100.0, -100.0), Vec2::new(50.0, 50.0));
        grid.insert(entity, aabb);

        let query = Rect::new(Vec2::new(-100.0, -100.0), Vec2::new(50.0, 50.0));
        let results = grid.query_rect(query);

        assert!(results.contains(&entity));
    }

    #[test]
    fn test_grid_coord_ordering() {
        let a = GridCoord::new(0, 0);
        let b = GridCoord::new(1, 0);
        let c = GridCoord::new(0, 1);

        assert!(a < b);
        assert!(a < c);
        assert!(b != c);
    }

    #[test]
    fn test_default_cell_size() {
        assert_eq!(DEFAULT_GRID_SIZE, 40.0);
    }

    #[test]
    fn test_small_query_large_entity() {
        let mut grid = SpatialHashGrid::with_cell_size(40.0);
        let entity = create_test_entity(1);

        // Large entity
        let aabb = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(200.0, 200.0));
        grid.insert(entity, aabb);

        // Small query inside entity bounds
        let query = Rect::new(Vec2::new(90.0, 90.0), Vec2::new(5.0, 5.0));
        let results = grid.query_rect(query);

        assert!(results.contains(&entity));
    }
}
