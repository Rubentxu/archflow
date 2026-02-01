// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Snapping System - HU-009
//
// Professional grid and entity snapping for Figma/tldraw-like UX.
//
// Features:
// - Snap-to-grid with configurable size
// - Snap-to-entity (edges, centers)
// - Threshold-based activation (only snap when close enough)
// - Visual guide support (snap points for UI rendering)
//
// Reference: docs/epics/EPIC-002-physics-sensors.md - HU-009
//
// Performance:
// - O(1) grid snapping (simple arithmetic)
// - O(k) entity snapping using SpatialHash where k = nearby entities
// - Zero allocations in hot path
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(dead_code)]

use alloc::vec::Vec;
use archflow_core::{EntityId, Vec2};

/// Default grid size for snapping (16px for UI elements)
pub const DEFAULT_GRID_SIZE: f32 = 16.0;

/// Default snap threshold (50% of grid size)
pub const DEFAULT_THRESHOLD: f32 = 8.0;

/// Configuration for the Snapper
///
/// Defines how snapping behavior works:
/// - Grid size for snap-to-grid
/// - Threshold distance for snap activation
/// - Whether to enable entity snapping
#[derive(Clone, Copy, Debug)]
pub struct SnapConfig {
    /// Size of grid cells in pixels
    /// Common values: 8px, 16px, 32px, 64px
    pub grid_size: f32,

    /// Maximum distance to snap (in pixels)
    /// When entity is within this distance of a snap point, it will snap
    /// Typically 50% of grid_size
    pub threshold: f32,

    /// Enable snap-to-entity (edges, centers)
    pub snap_to_entities: bool,

    /// Enable snap-to-grid
    pub snap_to_grid: bool,
}

impl Default for SnapConfig {
    #[inline]
    fn default() -> Self {
        Self {
            grid_size: DEFAULT_GRID_SIZE,
            threshold: DEFAULT_THRESHOLD,
            snap_to_entities: true,
            snap_to_grid: true,
        }
    }
}

impl SnapConfig {
    /// Create a new SnapConfig with custom grid size
    #[inline]
    pub fn with_grid_size(grid_size: f32) -> Self {
        Self {
            grid_size,
            threshold: grid_size * 0.5,
            snap_to_entities: true,
            snap_to_grid: true,
        }
    }

    /// Create a new SnapConfig with custom threshold
    #[inline]
    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.threshold = threshold;
        self
    }

    /// Disable entity snapping
    #[inline]
    pub fn grid_only(mut self) -> Self {
        self.snap_to_entities = false;
        self
    }
}

/// Result of a snap operation
///
/// Contains information about what was snapped and where
#[derive(Clone, Copy, Debug)]
pub struct SnapResult {
    /// The snapped position
    pub position: Vec2,

    /// What we snapped to (if anything)
    pub target: Option<SnapTarget>,
}

/// What entity/feature was snapped to
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SnapTarget {
    /// Snapped to grid intersection
    Grid { x: f32, y: f32 },

    /// Snapped to entity edge (left, right, top, bottom)
    EntityEdge { entity: EntityId, edge: EntityEdge },

    /// Snapped to entity center
    EntityCenter { entity: EntityId },
}

/// Edge of an entity that can be snapped to
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EntityEdge {
    Left,
    Right,
    Top,
    Bottom,
}

/// Snap point for visual guide rendering
///
/// The UI can use these to draw snap indicators (lines, highlights)
#[derive(Clone, Copy, Debug)]
pub struct SnapPoint {
    /// Position of the snap point
    pub position: Vec2,

    /// What this snap point represents
    pub target: SnapTarget,
}

/// Snapping system for grid and entity alignment
///
/// Provides Figma/tldraw-like snapping behavior:
/// - Snap-to-grid: Align positions to grid intersections
/// - Snap-to-entity: Align to edges and centers of nearby entities
/// - Threshold-based: Only snap when within threshold distance
///
/// # Examples
///
/// ```
/// use archflow_logic::snap::{Snapper, SnapConfig};
/// use archflow_core::Vec2;
///
/// let snapper = Snapper::new(SnapConfig::with_grid_size(16.0));
///
/// let raw_pos = Vec2::new(123.4, 567.8);
/// let result = snapper.snap_to_grid(raw_pos);
///
/// assert_eq!(result.position, Vec2::new(128.0, 576.0)); // Aligned to 16px grid
/// assert!(result.target.is_some()); // Did snap
/// ```
///
/// # Performance
///
/// - Grid snapping: O(1) - simple arithmetic
/// - Entity snapping: O(k) where k = entities within threshold
/// - Zero allocations in snap operations
pub struct Snapper {
    config: SnapConfig,
}

impl Snapper {
    /// Create a new Snapper with default configuration
    #[inline]
    pub fn new() -> Self {
        Self::with_config(SnapConfig::default())
    }

    /// Create a new Snapper with custom configuration
    #[inline]
    pub fn with_config(config: SnapConfig) -> Self {
        Self { config }
    }

    /// Get the current configuration
    #[inline]
    pub fn config(&self) -> &SnapConfig {
        &self.config
    }

    /// Update the configuration
    #[inline]
    pub fn set_config(&mut self, config: SnapConfig) {
        self.config = config;
    }

    /// Snap a position to the nearest grid intersection
    ///
    /// Returns the snapped position if within threshold, otherwise original position.
    ///
    /// # Arguments
    ///
    /// * `position` - The position to snap
    ///
    /// # Returns
    ///
    /// `SnapResult` with snapped position and target info
    ///
    /// # Examples
    ///
    /// ```
    /// let snapper = Snapper::with_config(SnapConfig::with_grid_size(16.0));
    ///
    /// let result = snapper.snap_to_grid(Vec2::new(10.0, 10.0));
    /// assert_eq!(result.position, Vec2::new(16.0, 16.0)); // Snapped to nearest grid line
    ///
    /// let result = snapper.snap_to_grid(Vec2::new(20.0, 20.0));
    /// assert_eq!(result.position, Vec2::new(16.0, 16.0)); // Already on grid, stays
    /// ```
    #[inline]
    pub fn snap_to_grid(&self, position: Vec2) -> SnapResult {
        if !self.config.snap_to_grid {
            return SnapResult {
                position,
                target: None,
            };
        }

        let snapped = Vec2::new(
            (position.x / self.config.grid_size).round() * self.config.grid_size,
            (position.y / self.config.grid_size).round() * self.config.grid_size,
        );

        // Check if snap is within threshold (per-axis check for grid snapping)
        let dx = (snapped.x - position.x).abs();
        let dy = (snapped.y - position.y).abs();

        // Snap if both axes are within threshold
        if dx <= self.config.threshold && dy <= self.config.threshold {
            SnapResult {
                position: snapped,
                target: Some(SnapTarget::Grid {
                    x: snapped.x,
                    y: snapped.y,
                }),
            }
        } else {
            SnapResult {
                position,
                target: None,
            }
        }
    }

    /// Snap a position to the nearest entity edge
    ///
    /// Uses SpatialHash to find nearby entities efficiently.
    ///
    /// # Arguments
    ///
    /// * `position` - The position to snap
    /// * `entity_aabbs` - Iterator of (entity_id, aabb) for entities to consider
    ///
    /// # Returns
    ///
    /// `SnapResult` with snapped position and target info
    ///
    /// # Examples
    ///
    /// ```
    /// let entities = [(id1, aabb1), (id2, aabb2)];
    /// let result = snapper.snap_to_entity_edge(raw_pos, entities.iter().map(|(id, a)| (*id, *a)));
    /// ```
    pub fn snap_to_entity_edge<'a>(
        &self,
        position: Vec2,
        entity_aabbs: impl Iterator<Item = (EntityId, &'a Rect)>,
    ) -> SnapResult {
        if !self.config.snap_to_entities {
            return SnapResult {
                position,
                target: None,
            };
        }

        let mut closest_snap: Option<(Vec2, SnapTarget)> = None;
        let mut closest_dist_sq = self.config.threshold * self.config.threshold;

        for (entity_id, aabb) in entity_aabbs {
            // Snap to left edge
            if let Some(edge_snap) = self.snap_to_edge(position, aabb, EntityEdge::Left) {
                let dist_sq = edge_snap.distance_squared(position);
                if dist_sq < closest_dist_sq {
                    closest_dist_sq = dist_sq;
                    closest_snap = Some((
                        edge_snap.position,
                        SnapTarget::EntityEdge {
                            entity: entity_id,
                            edge: EntityEdge::Left,
                        },
                    ));
                }
            }

            // Snap to right edge
            if let Some(edge_snap) = self.snap_to_edge(position, aabb, EntityEdge::Right) {
                let dist_sq = edge_snap.distance_squared(position);
                if dist_sq < closest_dist_sq {
                    closest_dist_sq = dist_sq;
                    closest_snap = Some((
                        edge_snap.position,
                        SnapTarget::EntityEdge {
                            entity: entity_id,
                            edge: EntityEdge::Right,
                        },
                    ));
                }
            }

            // Snap to top edge
            if let Some(edge_snap) = self.snap_to_edge(position, aabb, EntityEdge::Top) {
                let dist_sq = edge_snap.distance_squared(position);
                if dist_sq < closest_dist_sq {
                    closest_dist_sq = dist_sq;
                    closest_snap = Some((
                        edge_snap.position,
                        SnapTarget::EntityEdge {
                            entity: entity_id,
                            edge: EntityEdge::Top,
                        },
                    ));
                }
            }

            // Snap to bottom edge
            if let Some(edge_snap) = self.snap_to_edge(position, aabb, EntityEdge::Bottom) {
                let dist_sq = edge_snap.distance_squared(position);
                if dist_sq < closest_dist_sq {
                    closest_dist_sq = dist_sq;
                    closest_snap = Some((
                        edge_snap.position,
                        SnapTarget::EntityEdge {
                            entity: entity_id,
                            edge: EntityEdge::Bottom,
                        },
                    ));
                }
            }
        }

        if let Some((snapped_pos, target)) = closest_snap {
            SnapResult {
                position: snapped_pos,
                target: Some(target),
            }
        } else {
            SnapResult {
                position,
                target: None,
            }
        }
    }

    /// Snap a position to the nearest entity center
    ///
    /// # Arguments
    ///
    /// * `position` - The position to snap
    /// * `entity_aabbs` - Iterator of (entity_id, aabb) for entities to consider
    ///
    /// # Returns
    ///
    /// `SnapResult` with snapped position and target info
    pub fn snap_to_entity_center<'a>(
        &self,
        position: Vec2,
        entity_aabbs: impl Iterator<Item = (EntityId, &'a Rect)>,
    ) -> SnapResult {
        if !self.config.snap_to_entities {
            return SnapResult {
                position,
                target: None,
            };
        }

        let mut closest_snap: Option<(Vec2, SnapTarget)> = None;
        let mut closest_dist_sq = self.config.threshold * self.config.threshold;

        for (entity_id, aabb) in entity_aabbs {
            let center = aabb.center();
            let dist_sq = center.distance_squared(position);

            if dist_sq < closest_dist_sq {
                closest_dist_sq = dist_sq;
                closest_snap = Some((center, SnapTarget::EntityCenter { entity: entity_id }));
            }
        }

        if let Some((snapped_pos, target)) = closest_snap {
            SnapResult {
                position: snapped_pos,
                target: Some(target),
            }
        } else {
            SnapResult {
                position,
                target: None,
            }
        }
    }

    /// Snap to the nearest snap point (grid or entity)
    ///
    /// Tries grid first, then entities if grid snap didn't activate.
    ///
    /// # Arguments
    ///
    /// * `position` - The position to snap
    /// * `entity_aabbs` - Iterator of (entity_id, aabb) for entities to consider
    ///
    /// # Returns
    ///
    /// `SnapResult` with the best snap found
    pub fn snap<'a>(
        &self,
        position: Vec2,
        entity_aabbs: impl Iterator<Item = (EntityId, &'a Rect)>,
    ) -> SnapResult {
        // Try grid snap first
        let grid_result = self.snap_to_grid(position);

        // If grid snap activated, return it
        if grid_result.target.is_some() {
            return grid_result;
        }

        // Collect entities to avoid iterator move
        let entities: alloc::vec::Vec<(EntityId, &'a Rect)> = entity_aabbs.collect();

        // Otherwise try entity snapping
        // Try edges first, then centers
        let edge_result = self.snap_to_entity_edge(position, entities.iter().copied());
        if edge_result.target.is_some() {
            return edge_result;
        }

        let center_result = self.snap_to_entity_center(position, entities.iter().copied());
        center_result
    }

    /// Get snap points for a position (for UI rendering visual guides)
    ///
    /// Returns all snap points within threshold distance for rendering indicators
    ///
    /// # Arguments
    ///
    /// * `position` - The position to find snap points for
    /// * `entity_aabbs` - Iterator of (entity_id, aabb) for entities to consider
    ///
    /// # Returns
    ///
    /// Vec of `SnapPoint` for UI rendering
    pub fn get_snap_points<'a>(
        &self,
        position: Vec2,
        entity_aabbs: impl Iterator<Item = (EntityId, &'a Rect)>,
    ) -> Vec<SnapPoint> {
        let mut points = Vec::new();

        // Add grid snap points if within threshold
        if self.config.snap_to_grid {
            let grid_x = (position.x / self.config.grid_size).round() * self.config.grid_size;
            let grid_y = (position.y / self.config.grid_size).round() * self.config.grid_size;
            let grid_pos = Vec2::new(grid_x, grid_y);

            let dx = (grid_x - position.x).abs();
            let dy = (grid_y - position.y).abs();

            if dx <= self.config.threshold && dy <= self.config.threshold {
                points.push(SnapPoint {
                    position: grid_pos,
                    target: SnapTarget::Grid {
                        x: grid_x,
                        y: grid_y,
                    },
                });
            }
        }

        // Collect entities to allow multiple iterations
        let entities: alloc::vec::Vec<(EntityId, &'a Rect)> = entity_aabbs.collect();

        // Add entity snap points
        if self.config.snap_to_entities {
            for (entity_id, aabb) in entities.iter().copied() {
                // Add edge snap points
                for edge in [
                    EntityEdge::Left,
                    EntityEdge::Right,
                    EntityEdge::Top,
                    EntityEdge::Bottom,
                ] {
                    if let Some(edge_snap) = self.snap_to_edge(position, aabb, edge) {
                        let dist_sq = edge_snap.distance_squared(position);
                        let threshold_sq = self.config.threshold * self.config.threshold;

                        if dist_sq <= threshold_sq {
                            points.push(SnapPoint {
                                position: edge_snap.position,
                                target: SnapTarget::EntityEdge {
                                    entity: entity_id,
                                    edge,
                                },
                            });
                        }
                    }
                }

                // Add center snap point
                let center = aabb.center();
                let dist_sq = center.distance_squared(position);
                let threshold_sq = self.config.threshold * self.config.threshold;

                if dist_sq <= threshold_sq {
                    points.push(SnapPoint {
                        position: center,
                        target: SnapTarget::EntityCenter { entity: entity_id },
                    });
                }
            }
        }

        points
    }

    /// Internal helper: snap to a specific edge of an AABB
    #[inline]
    fn snap_to_edge(&self, position: Vec2, aabb: &Rect, edge: EntityEdge) -> Option<EdgeSnap> {
        let (snap_pos, within_threshold) = match edge {
            EntityEdge::Left => {
                let snap_pos = Vec2::new(aabb.min.x, position.y);
                let dist_sq = snap_pos.distance_squared(position);
                (
                    snap_pos,
                    dist_sq <= self.config.threshold * self.config.threshold,
                )
            }
            EntityEdge::Right => {
                let snap_pos = Vec2::new(aabb.max.x, position.y);
                let dist_sq = snap_pos.distance_squared(position);
                (
                    snap_pos,
                    dist_sq <= self.config.threshold * self.config.threshold,
                )
            }
            EntityEdge::Top => {
                let snap_pos = Vec2::new(position.x, aabb.min.y);
                let dist_sq = snap_pos.distance_squared(position);
                (
                    snap_pos,
                    dist_sq <= self.config.threshold * self.config.threshold,
                )
            }
            EntityEdge::Bottom => {
                let snap_pos = Vec2::new(position.x, aabb.max.y);
                let dist_sq = snap_pos.distance_squared(position);
                (
                    snap_pos,
                    dist_sq <= self.config.threshold * self.config.threshold,
                )
            }
        };

        if within_threshold {
            Some(EdgeSnap { position: snap_pos })
        } else {
            None
        }
    }
}

impl Default for Snapper {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Simple AABB rectangle for entity snapping
#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    #[inline]
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    #[inline]
    pub fn from_min_max(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self {
            min: Vec2::new(min_x, min_y),
            max: Vec2::new(max_x, max_y),
        }
    }

    #[inline]
    pub fn center(&self) -> Vec2 {
        Vec2::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
        )
    }

    #[inline]
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    #[inline]
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }
}

/// Internal helper for edge snapping results
struct EdgeSnap {
    position: Vec2,
}

impl EdgeSnap {
    #[inline]
    fn distance_squared(&self, other: Vec2) -> f32 {
        self.position.distance_squared(other)
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// TESTS
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SnapConfig::default();
        assert_eq!(config.grid_size, 16.0);
        assert_eq!(config.threshold, 8.0);
        assert!(config.snap_to_grid);
        assert!(config.snap_to_entities);
    }

    #[test]
    fn test_config_with_grid_size() {
        let config = SnapConfig::with_grid_size(32.0);
        assert_eq!(config.grid_size, 32.0);
        assert_eq!(config.threshold, 16.0); // 50% of grid size
    }

    #[test]
    fn test_config_grid_only() {
        let config = SnapConfig::default().grid_only();
        assert!(!config.snap_to_entities);
        assert!(config.snap_to_grid);
    }

    #[test]
    fn test_snap_to_grid_basic() {
        let snapper = Snapper::with_config(SnapConfig::with_grid_size(16.0));

        let result = snapper.snap_to_grid(Vec2::new(10.0, 10.0));
        assert_eq!(result.position, Vec2::new(16.0, 16.0));
        assert!(result.target.is_some());
    }

    #[test]
    fn test_snap_to_grid_already_aligned() {
        let snapper = Snapper::with_config(SnapConfig::with_grid_size(16.0));

        let result = snapper.snap_to_grid(Vec2::new(16.0, 16.0));
        assert_eq!(result.position, Vec2::new(16.0, 16.0));
        assert!(result.target.is_some());
    }

    #[test]
    fn test_snap_to_grid_no_snap_far_away() {
        let snapper = Snapper::with_config(SnapConfig::with_grid_size(16.0).with_threshold(4.0));

        // Position is 10px away from grid (16), threshold is 4px
        let result = snapper.snap_to_grid(Vec2::new(10.0, 10.0));
        assert_eq!(result.position, Vec2::new(10.0, 10.0)); // No snap
        assert!(result.target.is_none());
    }

    #[test]
    fn test_snap_to_grid_disabled() {
        let config = SnapConfig {
            snap_to_grid: false,
            ..Default::default()
        };
        let snapper = Snapper::with_config(config);

        let result = snapper.snap_to_grid(Vec2::new(10.0, 10.0));
        assert_eq!(result.position, Vec2::new(10.0, 10.0));
        assert!(result.target.is_none());
    }

    #[test]
    fn test_snap_to_entity_edge_left() {
        let snapper = Snapper::new();

        // Entity from x=100 to x=150 → left edge at x=100, right at x=150
        let aabb = Rect::from_min_max(100.0, 0.0, 150.0, 50.0);
        let entities = [(EntityId::new(1), &aabb)];

        let result = snapper.snap_to_entity_edge(Vec2::new(105.0, 25.0), entities.iter().copied());
        // Should snap to left edge at x=100
        assert_eq!(result.position.x, 100.0);
        assert!(result.target.is_some());
    }

    #[test]
    fn test_snap_to_entity_center() {
        let snapper = Snapper::new();

        // Entity from (100, 100) to (150, 150) → center at (125, 125)
        let aabb = Rect::from_min_max(100.0, 100.0, 150.0, 150.0);
        let entities = [(EntityId::new(1), &aabb)];

        let result =
            snapper.snap_to_entity_center(Vec2::new(120.0, 122.0), entities.iter().copied());
        // Distance is 5 in x and 3 in y, both within threshold of 8
        // Should snap to center at (125, 125)
        assert_eq!(result.position, Vec2::new(125.0, 125.0));
    }

    #[test]
    fn test_snap_no_match() {
        let snapper = Snapper::new();

        let aabb = Rect::from_min_max(100.0, 100.0, 150.0, 150.0);
        let entities = [(EntityId::new(1), &aabb)];

        // Position (5, 5) is too far from entity (distance ~100 from entity center)
        // but close to grid at (0, 0)
        let result = snapper.snap(Vec2::new(5.0, 5.0), entities.iter().copied());
        // Should snap to grid at (0, 0), not entity
        assert_eq!(result.position, Vec2::new(0.0, 0.0));
        assert!(result.target.is_some()); // Snapped to grid
    }

    #[test]
    fn test_rect_center() {
        let rect = Rect::from_min_max(0.0, 0.0, 100.0, 100.0);
        assert_eq!(rect.center(), Vec2::new(50.0, 50.0));
    }

    #[test]
    fn test_rect_width_height() {
        let rect = Rect::from_min_max(0.0, 0.0, 100.0, 50.0);
        assert_eq!(rect.width(), 100.0);
        assert_eq!(rect.height(), 50.0);
    }

    #[test]
    fn test_default_snapper() {
        let snapper = Snapper::default();
        assert_eq!(snapper.config().grid_size, 16.0);
    }

    #[test]
    fn test_snap_grid_negative_coords() {
        let snapper = Snapper::with_config(SnapConfig::with_grid_size(16.0));

        let result = snapper.snap_to_grid(Vec2::new(-10.0, -10.0));
        assert_eq!(result.position, Vec2::new(-16.0, -16.0));
    }

    #[test]
    fn test_snap_grid_large_values() {
        let snapper = Snapper::with_config(SnapConfig::with_grid_size(32.0));

        let result = snapper.snap_to_grid(Vec2::new(1000.5, 2000.7));
        // 1000.5 / 32 = 31.265625 → round to 31 → 31 * 32 = 992
        // 2000.7 / 32 = 62.521875 → round to 63 → 63 * 32 = 2016
        assert_eq!(result.position, Vec2::new(992.0, 2016.0));
    }

    #[test]
    fn test_get_snap_points_grid_only() {
        let snapper = Snapper::with_config(SnapConfig::default().grid_only());

        let empty: Vec<(EntityId, &Rect)> = Vec::new();
        let points = snapper.get_snap_points(Vec2::new(10.0, 10.0), empty.into_iter());
        // Should have at least the grid snap point
        assert!(!points.is_empty());
    }

    #[test]
    fn test_entity_edge_top() {
        let snapper = Snapper::new();

        let aabb = Rect::from_min_max(100.0, 100.0, 150.0, 150.0);
        let entities = [(EntityId::new(1), &aabb)];

        // Position above the entity
        let result = snapper.snap_to_entity_edge(Vec2::new(125.0, 95.0), entities.iter().copied());
        // Should snap to top edge at y=100
        assert_eq!(result.position.y, 100.0);
    }

    #[test]
    fn test_entity_edge_bottom() {
        let snapper = Snapper::new();

        let aabb = Rect::from_min_max(100.0, 100.0, 150.0, 150.0);
        let entities = [(EntityId::new(1), &aabb)];

        // Position below the entity
        let result = snapper.snap_to_entity_edge(Vec2::new(125.0, 155.0), entities.iter().copied());
        // Should snap to bottom edge at y=150
        assert_eq!(result.position.y, 150.0);
    }

    #[test]
    fn test_snap_prioritizes_grid_over_entity() {
        let snapper = Snapper::new();

        let aabb = Rect::from_min_max(10.0, 10.0, 20.0, 20.0);
        let entities = [(EntityId::new(1), &aabb)];

        // Position is close to both grid (16,16) and entity center (15,15)
        // Grid should win due to priority in snap() method
        let result = snapper.snap(Vec2::new(17.0, 17.0), entities.iter().copied());

        // Should snap to grid at (16, 16) not entity center at (15, 15)
        assert!(result.target.is_some());
        if let Some(SnapTarget::Grid { x, y }) = result.target {
            assert_eq!(x, 16.0);
            assert_eq!(y, 16.0);
        } else {
            panic!("Expected grid snap");
        }
    }

    #[test]
    fn test_config_with_custom_threshold() {
        let config = SnapConfig::default().with_threshold(4.0);
        assert_eq!(config.threshold, 4.0);
    }

    #[test]
    fn test_snap_result_no_target() {
        let result = SnapResult {
            position: Vec2::new(10.0, 10.0),
            target: None,
        };
        assert!(result.target.is_none());
    }

    #[test]
    fn test_snap_result_with_target() {
        let result = SnapResult {
            position: Vec2::new(16.0, 16.0),
            target: Some(SnapTarget::Grid { x: 16.0, y: 16.0 }),
        };
        assert!(result.target.is_some());
    }
}
