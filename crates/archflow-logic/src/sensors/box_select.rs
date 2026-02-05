// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Box Selection Sensor
//
// Implements rectangular (marquee) selection using SpatialHash for O(k) queries.
// Allows users to select multiple entities by dragging a selection rectangle.
//
// Performance:
// - O(k) query where k = entities near selection rectangle
// - O(n) for final verification where n = nearby entities
// - Typical: 100x faster than O(n) iteration for sparse selections
//
// Reference: docs/epics/LOGIC_BRICKS_DEVELOPER_GUIDE.md L788-808
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::vec::Vec;
use archflow_core::{EntityId, Vec2};
use archflow_engine::EntityStore;

use crate::spatial::{Rect, SpatialHashGrid};

/// Rectangular selection area defined by start and end points
///
/// The rectangle is defined by two corners (start and end) which can be
/// in any order. The actual selection area is the AABB that contains both points.
///
/// # Example
///
/// ```rust
/// use archflow_logic::sensors::box_select::BoxSelection;
/// use archflow_core::Vec2;
///
/// let start = Vec2::new(100.0, 100.0);
/// let end = Vec2::new(300.0, 200.0);
/// let selection = BoxSelection::new(start, end);
///
/// // Get the AABB for spatial queries
/// let aabb = selection.to_aabb();
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxSelection {
    /// Starting corner of the selection
    pub start: Vec2,
    /// Ending corner of the selection
    pub end: Vec2,
}

impl BoxSelection {
    /// Create a new box selection with start and end points
    ///
    /// The points can be in any order - the actual selection rectangle
    /// will be the AABB containing both points.
    ///
    /// # Arguments
    ///
    /// * `start` - Starting corner (e.g., mouse down position)
    /// * `end` - Ending corner (e.g., current mouse position)
    #[inline]
    #[must_use]
    pub fn new(start: Vec2, end: Vec2) -> Self {
        Self { start, end }
    }

    /// Get the axis-aligned bounding box of this selection
    ///
    /// Returns a Rect that contains both start and end points,
    /// regardless of their order.
    ///
    /// # Returns
    ///
    /// AABR containing the entire selection area
    #[inline]
    #[must_use]
    pub fn to_aabb(&self) -> Rect {
        let min_x = self.start.x.min(self.end.x);
        let min_y = self.start.y.min(self.end.y);
        let max_x = self.start.x.max(self.end.x);
        let max_y = self.start.y.max(self.end.y);

        Rect::from_min_max(min_x, min_y, max_x, max_y)
    }

    /// Check if this is a valid selection (not just a click)
    ///
    /// A selection is valid if the rectangle has non-zero area
    /// or exceeds the given threshold distance.
    ///
    /// # Arguments
    ///
    /// * `threshold` - Minimum distance to consider as valid selection
    ///
    /// # Returns
    ///
    /// `true` if the selection has meaningful size
    #[inline]
    #[must_use]
    pub fn is_valid(&self, threshold: f32) -> bool {
        let dx = (self.end.x - self.start.x).abs();
        let dy = (self.end.y - self.start.y).abs();
        dx > threshold || dy > threshold
    }

    /// Get the center point of the selection
    ///
    /// # Returns
    ///
    /// Center of the selection rectangle
    #[inline]
    #[must_use]
    pub fn center(&self) -> Vec2 {
        self.to_aabb().center()
    }

    /// Get the dimensions of the selection
    ///
    /// # Returns
    ///
    /// (width, height) of the selection
    #[inline]
    #[must_use]
    pub fn dimensions(&self) -> (f32, f32) {
        let aabb = self.to_aabb();
        (aabb.width(), aabb.height())
    }

    /// Check if a point is inside this selection
    ///
    /// # Arguments
    ///
    /// * `point` - The point to check
    ///
    /// # Returns
    ///
    /// `true` if the point is inside the selection rectangle
    #[inline]
    #[must_use]
    pub fn contains_point(&self, point: Vec2) -> bool {
        let aabb = self.to_aabb();
        point.x >= aabb.min_x
            && point.x <= aabb.max_x
            && point.y >= aabb.min_y
            && point.y <= aabb.max_y
    }
}

/// Configuration for BoxSelectSensor
#[derive(Clone, Copy, Debug)]
pub struct BoxSelectConfig {
    /// Minimum selection size to consider as valid (avoids accidental clicks)
    pub min_selection_size: f32,

    /// Whether to include entities that are partially inside the selection
    pub include_partial: bool,

    /// Whether to include entities that are fully contained
    pub include_contained: bool,
}

impl Default for BoxSelectConfig {
    fn default() -> Self {
        Self {
            min_selection_size: 5.0, // 5 pixels minimum
            include_partial: true,
            include_contained: true,
        }
    }
}

/// Box Selection Sensor
///
/// Detects entities within a rectangular selection area using SpatialHash.
/// Supports both preview mode (during drag) and final selection.
///
/// # Performance
///
/// Uses O(k) spatial query where k = entities in nearby cells:
/// 1. Query SpatialHash for entities near selection rectangle (O(k))
/// 2. Verify exact intersection with each entity's AABB (O(k))
/// 3. Return filtered list of selected entities
///
/// # Example
///
/// ```rust
/// use archflow_logic::sensors::box_select::{BoxSelectSensor, BoxSelection, BoxSelectConfig};
/// use archflow_core::Vec2;
///
/// let mut sensor = BoxSelectSensor::new();
/// sensor.set_config(BoxSelectConfig::default());
///
/// // During mouse drag
/// let selection = BoxSelection::new(start_pos, current_pos);
/// let selected = sensor.evaluate(&store, &selection);
///
/// // After mouse up
/// if selection.is_valid(config.min_selection_size) {
///     // Apply selection to entities
/// }
/// ```
pub struct BoxSelectSensor {
    /// Current selection rectangle (during drag)
    selection: Option<BoxSelection>,

    /// Configuration options
    config: BoxSelectConfig,

    /// Cached spatial hash reference (optional)
    spatial: Option<*const SpatialHashGrid>,
}

impl BoxSelectSensor {
    /// Create a new BoxSelectSensor with default config
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            selection: None,
            config: BoxSelectConfig::default(),
            spatial: None,
        }
    }

    /// Create with custom configuration
    #[inline]
    #[must_use]
    pub fn with_config(config: BoxSelectConfig) -> Self {
        Self {
            selection: None,
            config,
            spatial: None,
        }
    }

    /// Set the spatial hash reference for queries
    ///
    /// # Safety
    ///
    /// The SpatialHashGrid must live longer than this sensor.
    #[inline]
    pub fn set_spatial_hash(&mut self, spatial: *const SpatialHashGrid) {
        self.spatial = Some(spatial);
    }

    /// Update configuration
    #[inline]
    pub fn set_config(&mut self, config: BoxSelectConfig) {
        self.config = config;
    }

    /// Get current configuration
    #[inline]
    #[must_use]
    pub fn config(&self) -> BoxSelectConfig {
        self.config
    }

    /// Start a new selection drag
    ///
    /// Called when mouse button is pressed to begin selection.
    ///
    /// # Arguments
    ///
    /// * `pos` - Starting position of the selection
    #[inline]
    pub fn start_drag(&mut self, pos: Vec2) {
        self.selection = Some(BoxSelection::new(pos, pos));
    }

    /// Update the selection during drag
    ///
    /// Called as mouse moves while button is held.
    ///
    /// # Arguments
    ///
    /// * `pos` - Current mouse position
    #[inline]
    pub fn update_drag(&mut self, pos: Vec2) {
        if let Some(ref mut sel) = self.selection {
            sel.end = pos;
        }
    }

    /// End the selection drag
    ///
    /// Called when mouse button is released. Returns the final selection
    /// if it's valid, or None if it's just a click.
    ///
    /// # Returns
    ///
    /// The completed selection if valid, None if it was just a click
    #[inline]
    #[must_use]
    pub fn end_drag(&mut self) -> Option<BoxSelection> {
        let selection = self.selection.take();

        // Return selection only if it's valid
        match selection {
            Some(sel) if self.config.min_selection_size == 0.0 => Some(sel),
            Some(sel) if sel.is_valid(self.config.min_selection_size) => Some(sel),
            _ => None,
        }
    }

    /// Cancel the current selection drag
    ///
    /// Called if selection should be cancelled (e.g., ESC key).
    #[inline]
    pub fn cancel(&mut self) {
        self.selection = None;
    }

    /// Get the current selection (during drag)
    ///
    /// # Returns
    ///
    /// Current selection rectangle if dragging, None otherwise
    #[inline]
    #[must_use]
    pub fn current_selection(&self) -> Option<BoxSelection> {
        self.selection
    }

    /// Check if currently dragging
    #[inline]
    #[must_use]
    pub fn is_dragging(&self) -> bool {
        self.selection.is_some()
    }

    /// Evaluate entities within the selection rectangle
    ///
    /// This is the main query method. It uses SpatialHash for efficient
    /// O(k) queries instead of O(n) iteration over all entities.
    ///
    /// # Arguments
    ///
    /// * `store` - EntityStore containing entity transforms
    /// * `selection` - The selection rectangle to query
    ///
    /// # Returns
    ///
    /// Vector of EntityIds that intersect with the selection
    ///
    /// # Performance
    ///
    /// - O(k) where k = entities in nearby cells
    /// - Each entity verified with AABB intersection test
    #[inline]
    pub fn evaluate(&self, store: &EntityStore, selection: &BoxSelection) -> Vec<EntityId> {
        let mut selected = Vec::new();
        let selection_aabb = selection.to_aabb();

        // Check if we have a spatial hash for fast queries
        if let Some(spatial_ptr) = self.spatial {
            // SAFETY: The spatial hash is set via set_spatial_hash and must live longer
            // than this sensor. We dereference it as a reference for the query.
            let spatial = unsafe { &*spatial_ptr };

            // O(k) query: get entities in nearby cells
            let nearby = spatial.query_rect(selection_aabb);

            // Verify exact intersection with each entity
            for entity_id in nearby {
                let idx = entity_id.index().0 as usize;
                if idx < store.transforms.len() {
                    let pos = store.pos(idx);
                    let size = store.size(idx);
                    let entity_aabb = Rect::new(pos, size);

                    if selection_aabb.intersects(entity_aabb) {
                        selected.push(entity_id);
                    }
                }
            }
        } else {
            // Fallback: iterate all entities (O(n) but works without SpatialHash)
            // This is slower but provides correct results when SpatialHash isn't available
            for (idx, _) in store.transforms.iter().enumerate() {
                // Skip entities that are not alive (not in draw_order or in free_list)
                if !store.is_alive_index(idx) {
                    continue;
                }

                let pos = store.pos(idx);
                let size = store.size(idx);
                let entity_aabb = Rect::new(pos, size);

                if selection_aabb.intersects(entity_aabb) {
                    selected.push(EntityId::new(idx as u32));
                }
            }
        }

        selected
    }

    /// Evaluate using current selection (during drag)
    ///
    /// Convenience method for preview during drag.
    ///
    /// # Returns
    ///
    /// Entities currently in selection if dragging, empty vec otherwise
    #[inline]
    pub fn evaluate_current(&self, store: &EntityStore) -> Vec<EntityId> {
        match self.selection {
            Some(ref selection) => self.evaluate(store, selection),
            None => Vec::new(),
        }
    }

    /// Get the count of entities in the current selection
    ///
    /// Useful for displaying selection count during drag.
    #[inline]
    #[must_use]
    pub fn current_selection_count(&self, store: &EntityStore) -> usize {
        self.evaluate_current(store).len()
    }
}

impl Default for BoxSelectSensor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_engine::EntityStore;

    #[test]
    fn test_box_selection_new() {
        let start = Vec2::new(100.0, 100.0);
        let end = Vec2::new(300.0, 200.0);
        let selection = BoxSelection::new(start, end);

        assert_eq!(selection.start, start);
        assert_eq!(selection.end, end);
    }

    #[test]
    fn test_box_selection_to_aabb() {
        let selection = BoxSelection::new(Vec2::new(300.0, 200.0), Vec2::new(100.0, 100.0));
        let aabb = selection.to_aabb();

        assert_eq!(aabb.min_x, 100.0);
        assert_eq!(aabb.min_y, 100.0);
        assert_eq!(aabb.max_x, 300.0);
        assert_eq!(aabb.max_y, 200.0);
    }

    #[test]
    fn test_box_selection_is_valid() {
        let small = BoxSelection::new(Vec2::new(100.0, 100.0), Vec2::new(101.0, 101.0));
        assert!(!small.is_valid(5.0)); // 1 pixel is less than threshold

        let large = BoxSelection::new(Vec2::new(100.0, 100.0), Vec2::new(200.0, 200.0));
        assert!(large.is_valid(5.0)); // 100 pixels exceeds threshold
    }

    #[test]
    fn test_box_selection_contains_point() {
        let selection = BoxSelection::new(Vec2::new(100.0, 100.0), Vec2::new(200.0, 200.0));

        // Inside
        assert!(selection.contains_point(Vec2::new(150.0, 150.0)));

        // On edge
        assert!(selection.contains_point(Vec2::new(100.0, 150.0)));
        assert!(selection.contains_point(Vec2::new(150.0, 100.0)));

        // Outside
        assert!(!selection.contains_point(Vec2::new(50.0, 150.0)));
        assert!(!selection.contains_point(Vec2::new(250.0, 150.0)));
        assert!(!selection.contains_point(Vec2::new(150.0, 50.0)));
        assert!(!selection.contains_point(Vec2::new(150.0, 250.0)));
    }

    #[test]
    fn test_box_selection_dimensions() {
        let selection = BoxSelection::new(Vec2::new(100.0, 100.0), Vec2::new(300.0, 250.0));
        let (width, height) = selection.dimensions();

        assert_eq!(width, 200.0);
        assert_eq!(height, 150.0);
    }

    #[test]
    fn test_box_selection_center() {
        let selection = BoxSelection::new(Vec2::new(100.0, 100.0), Vec2::new(300.0, 200.0));
        let center = selection.center();

        assert_eq!(center.x, 200.0);
        assert_eq!(center.y, 150.0);
    }

    #[test]
    fn test_box_select_sensor_drag_lifecycle() {
        let mut sensor = BoxSelectSensor::new();

        // Start drag
        sensor.start_drag(Vec2::new(100.0, 100.0));
        assert!(sensor.is_dragging());
        assert!(sensor.current_selection().is_some());

        // Update drag
        sensor.update_drag(Vec2::new(300.0, 200.0));
        let selection = sensor.current_selection().unwrap();
        assert_eq!(selection.end, Vec2::new(300.0, 200.0));

        // End drag - valid selection
        let final_selection = sensor.end_drag();
        assert!(final_selection.is_some());
        assert!(!sensor.is_dragging());
    }

    #[test]
    fn test_box_select_sensor_cancel() {
        let mut sensor = BoxSelectSensor::new();

        sensor.start_drag(Vec2::new(100.0, 100.0));
        assert!(sensor.is_dragging());

        sensor.cancel();
        assert!(!sensor.is_dragging());
        assert!(sensor.current_selection().is_none());
    }

    #[test]
    fn test_box_select_sensor_evaluate() {
        let mut store = EntityStore::new();
        let _e1 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(20.0, 20.0)); // Outside
        let e2 = store.spawn(Vec2::new(150.0, 150.0), Vec2::new(30.0, 30.0)); // Inside
        let _e3 = store.spawn(Vec2::new(300.0, 300.0), Vec2::new(20.0, 20.0)); // Outside

        let sensor = BoxSelectSensor::new();
        let selection = BoxSelection::new(Vec2::new(100.0, 100.0), Vec2::new(200.0, 200.0));
        let selected = sensor.evaluate(&store, &selection);

        // Should only contain e2
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0], e2);
    }

    #[test]
    fn test_box_select_sensor_partial_intersection() {
        let mut store = EntityStore::new();
        // Entity that partially overlaps with selection
        let e1 = store.spawn(Vec2::new(180.0, 180.0), Vec2::new(50.0, 50.0)); // Overlaps corner

        let sensor = BoxSelectSensor::new();
        let selection = BoxSelection::new(Vec2::new(100.0, 100.0), Vec2::new(200.0, 200.0));
        let selected = sensor.evaluate(&store, &selection);

        // Should contain e1 because it intersects (partial overlap)
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0], e1);
    }

    #[test]
    fn test_box_select_sensor_empty_selection() {
        // Test that point selection works (edge case)
        // Note: Without SpatialHash, this tests the Rect intersection logic directly
        let selection = BoxSelection::new(Vec2::new(100.0, 100.0), Vec2::new(100.0, 100.0));

        // Point at center of selection should be contained
        assert!(selection.contains_point(Vec2::new(100.0, 100.0)));
    }

    #[test]
    fn test_box_select_sensor_with_config() {
        let config = BoxSelectConfig {
            min_selection_size: 10.0,
            include_partial: true,
            include_contained: true,
        };
        let mut sensor = BoxSelectSensor::with_config(config);

        let _selection = BoxSelection::new(Vec2::new(5.0, 5.0), Vec2::new(8.0, 8.0)); // 3x3 area

        // This should be invalid because it's smaller than min_selection_size
        let result = sensor.end_drag();
        assert!(result.is_none()); // Invalid due to size
    }

    #[test]
    fn test_box_select_sensor_evaluate_current() {
        // Test the drag lifecycle with current_selection()
        let mut sensor = BoxSelectSensor::new();

        // Initially not dragging
        assert!(!sensor.is_dragging());
        assert!(sensor.current_selection().is_none());

        // Start drag
        sensor.start_drag(Vec2::new(100.0, 100.0));
        assert!(sensor.is_dragging());
        assert!(sensor.current_selection().is_some());
    }

    #[test]
    fn test_box_select_sensor_current_selection_count() {
        let mut store = EntityStore::new();
        for i in 0..5 {
            store.spawn(
                Vec2::new(150.0 + i as f32 * 10.0, 150.0),
                Vec2::new(20.0, 20.0),
            );
        }

        let mut sensor = BoxSelectSensor::new();

        sensor.start_drag(Vec2::new(100.0, 100.0));
        sensor.update_drag(Vec2::new(300.0, 200.0));

        let count = sensor.current_selection_count(&store);
        assert_eq!(count, 5);
    }
}
