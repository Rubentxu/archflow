// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow SDK - Snap System API
//
// This module provides the Snapper for professional snapping behavior
// (snap-to-grid, snap-to-entity, snap-to-guide).
//
// Reference: docs/epics/EPIC-SDK-PUBLIC-API.md - Section "API de Snap System"
// ═══════════════════════════════════════════════════════════════════════════════

use archflow_core::Vec2;
use std::vec::Vec;

/// Configuration for the snap system
///
/// Controls all aspects of snapping behavior including grid size,
/// snap threshold, and which snap modes are enabled.
#[derive(Clone, Debug)]
pub struct SnapConfig {
    /// Size of the grid for snap-to-grid (0 = disabled)
    pub grid_size: f32,

    /// Distance threshold to activate snap (0 = 50% of grid_size)
    pub threshold: f32,

    /// Enable snapping to entity edges
    pub snap_to_edges: bool,

    /// Enable snapping to entity centers
    pub snap_to_centers: bool,

    /// Enable snapping to custom guides
    pub snap_to_guides: bool,
}

impl Default for SnapConfig {
    fn default() -> Self {
        Self {
            grid_size: 16.0,
            threshold: 8.0,
            snap_to_edges: true,
            snap_to_centers: true,
            snap_to_guides: true,
        }
    }
}

impl SnapConfig {
    /// Create a new snap configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the grid size
    pub fn with_grid_size(mut self, size: f32) -> Self {
        self.grid_size = size;
        // Auto-adjust threshold to 50% if not explicitly set
        if self.threshold == 8.0 {
            self.threshold = size / 2.0;
        }
        self
    }

    /// Set the snap threshold
    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.threshold = threshold;
        self
    }

    /// Enable snap-to-edges
    pub fn with_edges(mut self, enabled: bool) -> Self {
        self.snap_to_edges = enabled;
        self
    }

    /// Enable snap-to-centers
    pub fn with_centers(mut self, enabled: bool) -> Self {
        self.snap_to_centers = enabled;
        self
    }

    /// Enable snap-to-guides
    pub fn with_guides(mut self, enabled: bool) -> Self {
        self.snap_to_guides = enabled;
        self
    }
}

/// A snap guide (horizontal or vertical line)
#[derive(Clone, Copy, Debug)]
pub struct SnapGuide {
    /// Position of the guide (x for vertical, y for horizontal)
    pub position: f32,

    /// Whether this is a vertical guide (true) or horizontal (false)
    pub is_vertical: bool,

    /// Whether this is a temporary guide (shown during drag)
    pub is_temporary: bool,
}

impl SnapGuide {
    /// Create a vertical guide at x position
    pub fn vertical(x: f32) -> Self {
        Self {
            position: x,
            is_vertical: true,
            is_temporary: false,
        }
    }

    /// Create a horizontal guide at y position
    pub fn horizontal(y: f32) -> Self {
        Self {
            position: y,
            is_vertical: false,
            is_temporary: false,
        }
    }

    /// Create a temporary vertical guide
    pub fn temporary_vertical(x: f32) -> Self {
        Self {
            position: x,
            is_vertical: true,
            is_temporary: true,
        }
    }

    /// Create a temporary horizontal guide
    pub fn temporary_horizontal(y: f32) -> Self {
        Self {
            position: y,
            is_vertical: false,
            is_temporary: true,
        }
    }
}

/// Snap system for professional alignment behavior
///
/// The snapper provides grid snapping, entity snapping, and guide snapping
/// with Figma/tldraw-like UX.
///
/// # Example
///
/// ```rust
/// use archflow_sdk::snap::{Snapper, SnapConfig};
/// use archflow_core::Vec2;
///
/// let snapper = Snapper::new(
///     SnapConfig::new()
///         .with_grid_size(16.0)
///         .with_threshold(8.0),
/// );
///
/// let raw_pos = Vec2::new(123.4, 567.8);
/// let snapped_pos = snapper.snap_to_grid(raw_pos);
/// // Result: Vec2(128.0, 560.0) - aligned to 16px grid
/// ```
pub struct Snapper {
    config: SnapConfig,
    guides: Vec<SnapGuide>,
}

impl Snapper {
    /// Create a new snapper with the given configuration
    pub fn new(config: SnapConfig) -> Self {
        Self {
            config,
            guides: Vec::new(),
        }
    }

    /// Add a guide to the snapper
    pub fn add_guide(&mut self, guide: SnapGuide) {
        self.guides.push(guide);
    }

    /// Remove all temporary guides
    pub fn clear_temporary_guides(&mut self) {
        self.guides.retain(|g| !g.is_temporary);
    }

    /// Remove all guides
    pub fn clear_guides(&mut self) {
        self.guides.clear();
    }

    /// Snap a position to the grid
    ///
    /// If grid_size is 0, returns the position unchanged.
    /// Otherwise, aligns to the nearest grid intersection.
    ///
    /// # Arguments
    ///
    /// * `pos` - The position to snap
    ///
    /// # Returns
    ///
    /// The snapped position
    #[inline]
    pub fn snap_to_grid(&self, pos: Vec2) -> Vec2 {
        if self.config.grid_size == 0.0 {
            return pos;
        }

        Vec2::new(
            (pos.x / self.config.grid_size).round() * self.config.grid_size,
            (pos.y / self.config.grid_size).round() * self.config.grid_size,
        )
    }

    /// Snap a position to entity edges
    ///
    /// This requires spatial indexing for efficiency.
    /// For now, returns the position unchanged.
    ///
    /// TODO: Implement with SpatialHash integration
    #[inline]
    pub fn snap_to_entities(&self, pos: Vec2, _store: &archflow_engine::EntityStore) -> Vec2 {
        if !self.config.snap_to_edges {
            return pos;
        }

        // TODO: Use SpatialHash to find nearby entities
        // and check edges within threshold

        pos
    }

    /// Snap a position to guides
    ///
    /// Snaps to the nearest vertical or horizontal guide within threshold.
    #[inline]
    pub fn snap_to_guides(&self, pos: Vec2) -> Vec2 {
        if !self.config.snap_to_guides || self.guides.is_empty() {
            return pos;
        }

        let mut snapped_x = pos.x;
        let mut snapped_y = pos.y;

        for guide in &self.guides {
            if guide.is_vertical {
                // Check if we're close to this vertical guide
                if (pos.x - guide.position).abs() < self.config.threshold {
                    snapped_x = guide.position;
                }
            } else {
                // Check if we're close to this horizontal guide
                if (pos.y - guide.position).abs() < self.config.threshold {
                    snapped_y = guide.position;
                }
            }
        }

        Vec2::new(snapped_x, snapped_y)
    }

    /// Snap a position using all enabled snap modes
    ///
    /// This applies grid, entity, and guide snapping in order.
    /// The first snap found within threshold is used.
    #[inline]
    pub fn snap(&self, pos: Vec2, store: &archflow_engine::EntityStore) -> Vec2 {
        // Try grid snap first
        let snapped = self.snap_to_grid(pos);

        // If grid snap changed the position, use it
        if (snapped - pos).length() > 0.0 {
            return snapped;
        }

        // Try entity snap
        let snapped = self.snap_to_entities(snapped, store);
        if (snapped - pos).length() > 0.0 {
            return snapped;
        }

        // Try guide snap
        self.snap_to_guides(snapped)
    }

    /// Get the snapper's configuration
    pub fn config(&self) -> &SnapConfig {
        &self.config
    }

    /// Get all guides
    pub fn guides(&self) -> &[SnapGuide] {
        &self.guides
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_core::Vec2;

    #[test]
    fn test_snap_config_default() {
        let config = SnapConfig::default();
        assert_eq!(config.grid_size, 16.0);
        assert_eq!(config.threshold, 8.0);
        assert!(config.snap_to_edges);
        assert!(config.snap_to_centers);
        assert!(config.snap_to_guides);
    }

    #[test]
    fn test_snap_config_builder() {
        let config = SnapConfig::new()
            .with_grid_size(32.0)
            .with_threshold(16.0)
            .with_edges(false);

        assert_eq!(config.grid_size, 32.0);
        assert_eq!(config.threshold, 16.0);
        assert!(!config.snap_to_edges);
    }

    #[test]
    fn test_snapper_snap_to_grid() {
        let snapper = Snapper::new(SnapConfig::new().with_grid_size(16.0));

        // Test exact grid alignment
        let pos = Vec2::new(16.0, 32.0);
        let snapped = snapper.snap_to_grid(pos);
        assert_eq!(snapped.x, 16.0);
        assert_eq!(snapped.y, 32.0);

        // Test between grid points
        // 123.4 / 16 = 7.7125, rounds to 8, 8 * 16 = 128
        // 567.8 / 16 = 35.4875, rounds to 35, 35 * 16 = 560
        let pos = Vec2::new(123.4, 567.8);
        let snapped = snapper.snap_to_grid(pos);
        assert_eq!(snapped.x, 128.0); // Rounds to nearest 16
        assert_eq!(snapped.y, 560.0); // Rounds to nearest 16
    }

    #[test]
    fn test_snapper_snap_to_grid_disabled() {
        let snapper = Snapper::new(SnapConfig::new().with_grid_size(0.0));

        let pos = Vec2::new(123.4, 567.8);
        let snapped = snapper.snap_to_grid(pos);
        assert_eq!(snapped.x, 123.4);
        assert_eq!(snapped.y, 567.8);
    }

    #[test]
    fn test_snapper_snap_to_guides() {
        let mut snapper = Snapper::new(SnapConfig::new().with_threshold(10.0));
        snapper.add_guide(SnapGuide::vertical(100.0));
        snapper.add_guide(SnapGuide::horizontal(200.0));

        let pos = Vec2::new(105.0, 195.0);
        let snapped = snapper.snap_to_guides(pos);

        // Should snap to both guides
        assert_eq!(snapped.x, 100.0);
        assert_eq!(snapped.y, 200.0);
    }

    #[test]
    fn test_snapper_snap_to_guides_out_of_threshold() {
        let mut snapper = Snapper::new(SnapConfig::new().with_threshold(5.0));
        snapper.add_guide(SnapGuide::vertical(100.0));

        let pos = Vec2::new(110.0, 200.0);
        let snapped = snapper.snap_to_guides(pos);

        // Should NOT snap (10px away, threshold is 5px)
        assert_eq!(snapped.x, 110.0);
        assert_eq!(snapped.y, 200.0);
    }

    #[test]
    fn test_snap_guide_vertical() {
        let guide = SnapGuide::vertical(50.0);
        assert!(guide.is_vertical);
        assert_eq!(guide.position, 50.0);
        assert!(!guide.is_temporary);
    }

    #[test]
    fn test_snap_guide_horizontal() {
        let guide = SnapGuide::horizontal(100.0);
        assert!(!guide.is_vertical);
        assert_eq!(guide.position, 100.0);
        assert!(!guide.is_temporary);
    }

    #[test]
    fn test_snap_guide_temporary() {
        let guide = SnapGuide::temporary_vertical(50.0);
        assert!(guide.is_vertical);
        assert!(guide.is_temporary);
    }

    #[test]
    fn test_snapper_clear_temporary_guides() {
        let mut snapper = Snapper::new(SnapConfig::new());
        snapper.add_guide(SnapGuide::vertical(10.0));
        snapper.add_guide(SnapGuide::temporary_vertical(20.0));

        assert_eq!(snapper.guides().len(), 2);

        snapper.clear_temporary_guides();

        assert_eq!(snapper.guides().len(), 1);
        assert_eq!(snapper.guides()[0].position, 10.0);
    }

    #[test]
    fn test_snapper_clear_all_guides() {
        let mut snapper = Snapper::new(SnapConfig::new());
        snapper.add_guide(SnapGuide::vertical(10.0));
        snapper.add_guide(SnapGuide::horizontal(20.0));

        assert_eq!(snapper.guides().len(), 2);

        snapper.clear_guides();

        assert_eq!(snapper.guides().len(), 0);
    }
}
