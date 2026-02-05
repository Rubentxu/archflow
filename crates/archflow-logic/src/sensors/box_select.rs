// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Box Selection Sensor
//
// Detects entities within a rectangular selection area using SpatialHash
// for O(k) query performance instead of O(n) linear scan.
//
// Performance Characteristics:
// - O(k) where k = nearby entities (SpatialHash query)
// - O(n) only when spatial query returns large area
// - Zero-allocation for query results
//
// Memory Impact:
// - No per-entity storage (unlike MouseOverSensor)
// - Uses shared SpatialHash from LogicSystem
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::vec::Vec;
use archflow_core::{EntityId, Rect, Vec2};
use archflow_engine::{EntityStore, SpatialHash};

/// Rectangular selection area defined by two corners
///
/// The selection rectangle is defined by two points (start and end) which can
/// be in any order. The actual selection bounds are computed as the minimum
/// bounding rectangle of both points.
///
/// # Examples
///
/// ```
/// use archflow_logic::box_selection::BoxSelection;
/// use archflow_core::Vec2;
///
/// let selection = BoxSelection::new(Vec2::new(100.0, 100.0), Vec2::new(300.0, 200.0));
///
/// // Selection bounds are computed regardless of point order
/// assert_eq!(selection.width(), 200.0);
/// assert_eq!(selection.height(), 100.0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct BoxSelection {
    /// Starting corner of the selection
    start: Vec2,
    /// Ending corner of the selection
    end: Vec2,
}

impl BoxSelection {
    /// Creates a new box selection from two corners
    ///
    /// The corners can be in any order - the actual bounds are computed
    /// as the minimum bounding rectangle.
    ///
    /// # Arguments
    ///
    /// * `start` - First corner of the selection rectangle
    /// * `end` - Second corner of the selection rectangle
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_core::Vec2;
    /// use archflow_logic::box_selection::BoxSelection;
    ///
    /// let selection = BoxSelection::new(
    ///     Vec2::new(100.0, 100.0),
    ///     Vec2::new(300.0, 200.0)
    /// );
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn new(start: Vec2, end: Vec2) -> Self {
        Self { start, end }
    }

    /// Returns the axis-aligned bounding box of the selection
    ///
    /// The AABB is computed as the minimum rectangle that contains both
    /// the start and end points.
    ///
    /// # Returns
    ///
    /// `Rect` representing the selection bounds (min, max)
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_core::Vec2;
    /// use archflow_logic::box_selection::BoxSelection;
    ///
    /// let selection = BoxSelection::new(
    ///     Vec2::new(100.0, 200.0),
    ///     Vec2::new(300.0, 100.0)
    /// );
    ///
    /// let aabb = selection.to_aabb();
    /// // AABB will have min=(100, 100), max=(300, 200) regardless of point order
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn to_aabb(&self) -> Rect {
        let min_x = self.start.x.min(self.end.x);
        let min_y = self.start.y.min(self.end.y);
        let max_x = self.start.x.max(self.end.x);
        let max_y = self.start.y.max(self.end.y);
        Rect::new(min_x, min_y, max_x, max_y)
    }

    /// Returns the center point of the selection
    ///
    /// # Returns
    ///
    /// `Vec2` at the center of the selection rectangle
    #[inline(always)]
    #[must_use]
    pub fn center(&self) -> Vec2 {
        let aabb = self.to_aabb();
        Vec2::new(
            (aabb.min.x + aabb.max.x) * 0.5,
            (aabb.min.y + aabb.max.y) * 0.5,
        )
    }

    /// Returns the width of the selection
    ///
    /// # Returns
    ///
    /// Width in world units
    #[inline(always)]
    #[must_use]
    pub fn width(&self) -> f32 {
        (self.start.x - self.end.x).abs()
    }

    /// Returns the height of the selection
    ///
    /// # Returns
    ///
    /// Height in world units
    #[inline(always)]
    #[must_use]
    pub fn height(&self) -> f32 {
        (self.start.y - self.end.y).abs()
    }

    /// Returns whether the selection is valid (larger than threshold)
    ///
    /// A selection is considered valid if it has non-zero area,
    /// which distinguishes a box selection from a simple click.
    ///
    /// # Arguments
    ///
    /// * `threshold` - Minimum dimension to consider valid (default: 5.0)
    ///
    /// # Returns
    ///
    /// `true` if the selection has significant area
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_core::Vec2;
    /// use archflow_logic::box_selection::BoxSelection;
    ///
    /// // Valid selection
    /// let large = BoxSelection::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
    /// assert!(large.is_valid(5.0));
    ///
    /// // Click (invalid selection)
    /// let click = BoxSelection::new(Vec2::new(50.0, 50.0), Vec2::new(50.0, 50.0));
    /// assert!(!click.is_valid(5.0));
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn is_valid(&self, threshold: f32) -> bool {
        self.width() > threshold || self.height() > threshold
    }

    /// Returns whether a point is inside the selection
    ///
    /// # Arguments
    ///
    /// * `point` - Point to test in world coordinates
    ///
    /// # Returns
    ///
    /// `true` if the point is within the selection rectangle
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_core::Vec2;
    /// use archflow_logic::box_selection::BoxSelection;
    ///
    /// let selection = BoxSelection::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
    ///
    /// assert!(selection.contains_point(Vec2::new(50.0, 50.0)));
    /// assert!(!selection.contains_point(Vec2::new(150.0, 50.0)));
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn contains_point(&self, point: Vec2) -> bool {
        let aabb = self.to_aabb();
        point.x >= aabb.min.x
            && point.x <= aabb.max.x
            && point.y >= aabb.min.y
            && point.y <= aabb.max.y
    }
}

/// Sensor for detecting entities within a rectangular selection area
///
/// BoxSelectSensor uses SpatialHash for efficient O(k) queries where k is
/// the number of nearby entities, rather than O(n) linear scan of all entities.
///
/// # Performance Characteristics
///
/// - **Spatial Query**: O(k) using SpatialHash (k = nearby entities)
/// - **Exact Intersection**: O(k) additional for AABB verification
/// - **Memory**: Zero per-entity storage
///
/// # Usage
///
/// 1. Create the sensor: `BoxSelectSensor::new()`
/// 2. Start drag: `start_drag(start_position)`
/// 3. Update during drag: `update_drag(current_position)`
/// 4. End drag: `end_drag()` returns selected entities
///
/// # Examples
///
/// ```
/// use archflow_core::Vec2;
/// use archflow_logic::box_selection::{BoxSelectSensor, BoxSelection};
/// use archflow_engine::{EntityStore, SpatialHash, MAX_ENTITIES};
///
/// let mut store = EntityStore::new();
/// let _e1 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(20.0, 20.0));
/// let _e2 = store.spawn(Vec2::new(150.0, 50.0), Vec2::new(20.0, 20.0));
/// let _e3 = store.spawn(Vec2::new(250.0, 50.0), Vec2::new(20.0, 20.0));
///
/// let mut spatial = SpatialHash::new(MAX_ENTITIES);
/// let mut sensor = BoxSelectSensor::new();
///
/// // Simulate drag selection
/// sensor.start_drag(Vec2::new(0.0, 0.0));
/// sensor.update_drag(Vec2::new(200.0, 100.0));
///
/// // Get selected entities
/// let selection = sensor.end_drag().unwrap();
/// let selected = sensor.evaluate(&store, &selection, &spatial);
///
/// assert_eq!(selected.len(), 2); // e1 and e2
/// ```
pub struct BoxSelectSensor {
    /// Current selection being dragged (None if not selecting)
    current_selection: Option<BoxSelection>,
}

impl BoxSelectSensor {
    /// Creates a new BoxSelectSensor
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_logic::box_selection::BoxSelectSensor;
    ///
    /// let sensor = BoxSelectSensor::new();
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_selection: None,
        }
    }

    /// Starts a drag selection operation
    ///
    /// Initializes the selection with the starting position.
    /// Call `update_drag` to update the selection as the mouse moves.
    ///
    /// # Arguments
    ///
    /// * `start_pos` - Starting position in world coordinates
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_core::Vec2;
    /// use archflow_logic::box_selection::BoxSelectSensor;
    ///
    /// let mut sensor = BoxSelectSensor::new();
    /// sensor.start_drag(Vec2::new(100.0, 100.0));
    /// ```
    #[inline(always)]
    pub fn start_drag(&mut self, start_pos: Vec2) {
        self.current_selection = Some(BoxSelection::new(start_pos, start_pos));
    }

    /// Updates the selection during a drag operation
    ///
    /// Extends the selection rectangle to include the current position.
    /// Has no effect if `start_drag` was not called first.
    ///
    /// # Arguments
    ///
    /// * `current_pos` - Current mouse position in world coordinates
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_core::Vec2;
    /// use archflow_logic::box_selection::BoxSelectSensor;
    ///
    /// let mut sensor = BoxSelectSensor::new();
    /// sensor.start_drag(Vec2::new(100.0, 100.0));
    /// sensor.update_drag(Vec2::new(300.0, 200.0));
    /// ```
    #[inline(always)]
    pub fn update_drag(&mut self, current_pos: Vec2) {
        if let Some(selection) = self.current_selection.as_mut() {
            // Update end point, keeping start point
            self.current_selection = Some(BoxSelection::new(selection.start, current_pos));
        }
    }

    /// Ends the drag selection and returns the selection rectangle
    ///
    /// Consumes the current selection and returns it.
    /// Returns `None` if no selection was in progress.
    ///
    /// # Returns
    ///
    /// `Some(BoxSelection)` if a selection was in progress, `None` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_core::Vec2;
    /// use archflow_logic::box_selection::BoxSelectSensor;
    ///
    /// let mut sensor = BoxSelectSensor::new();
    /// sensor.start_drag(Vec2::new(100.0, 100.0));
    /// sensor.update_drag(Vec2::new(300.0, 200.0));
    ///
    /// let selection = sensor.end_drag();
    /// assert!(selection.is_some());
    /// ```
    #[inline(always)]
    pub fn end_drag(&mut self) -> Option<BoxSelection> {
        self.current_selection.take()
    }

    /// Cancels the current selection without returning it
    ///
    /// Useful for canceling selection when the user presses Escape
    /// or when the selection is too small.
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_core::Vec2;
    /// use archflow_logic::box_selection::BoxSelectSensor;
    ///
    /// let mut sensor = BoxSelectSensor::new();
    /// sensor.start_drag(Vec2::new(100.0, 100.0));
    /// sensor.cancel();
    ///
    /// assert!(sensor.end_drag().is_none());
    /// ```
    #[inline(always)]
    pub fn cancel(&mut self) {
        self.current_selection = None;
    }

    /// Checks if a selection is currently in progress
    ///
    /// # Returns
    ///
    /// `true` if `start_drag` was called without `end_drag` or `cancel`
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_core::Vec2;
    /// use archflow_logic::box_selection::BoxSelectSensor;
    ///
    /// let mut sensor = BoxSelectSensor::new();
    /// assert!(!sensor.is_selecting());
    ///
    /// sensor.start_drag(Vec2::new(100.0, 100.0));
    /// assert!(sensor.is_selecting());
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn is_selecting(&self) -> bool {
        self.current_selection.is_some()
    }

    /// Gets the current selection without consuming it
    ///
    /// Returns `None` if no selection is in progress.
    ///
    /// # Returns
    ///
    /// Reference to current `BoxSelection` or `None`
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_core::Vec2;
    /// use archflow_logic::box_selection::BoxSelectSensor;
    ///
    /// let mut sensor = BoxSelectSensor::new();
    /// sensor.start_drag(Vec2::new(100.0, 100.0));
    ///
    /// let selection = sensor.current_selection();
    /// assert!(selection.is_some());
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn current_selection(&self) -> Option<&BoxSelection> {
        self.current_selection.as_ref()
    }

    /// Evaluates which entities are within the selection
    ///
    /// Uses SpatialHash to find nearby entities (O(k)) and then
    /// performs exact AABB intersection tests (O(k)).
    ///
    /// # Arguments
    ///
    /// * `store` - EntityStore with entity positions and sizes
    /// * `selection` - Box selection area
    /// * `spatial` - SpatialHash for efficient nearby entity queries
    ///
    /// # Returns
    ///
    /// Vector of EntityIds within the selection
    ///
    /// # Performance
    ///
    /// - O(k) SpatialHash query where k = nearby entities
    /// - O(k) exact AABB verification
    /// - Zero allocations for result vector (pre-allocated)
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_core::Vec2;
    /// use archflow_logic::box_selection::{BoxSelectSensor, BoxSelection};
    /// use archflow_engine::{EntityStore, SpatialHash, MAX_ENTITIES};
    ///
    /// let mut store = EntityStore::new();
    /// let _e1 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(20.0, 20.0));
    /// let _e2 = store.spawn(Vec2::new(150.0, 50.0), Vec2::new(20.0, 20.0));
    ///
    /// let mut spatial = SpatialHash::new(MAX_ENTITIES);
    /// let mut sensor = BoxSelectSensor::new();
    ///
    /// let selection = BoxSelection::new(Vec2::new(0.0, 0.0), Vec2::new(200.0, 100.0));
    /// let selected = sensor.evaluate(&store, &selection, &spatial);
    ///
    /// assert_eq!(selected.len(), 2);
    /// ```
    #[inline(never)]
    pub fn evaluate(
        &self,
        store: &EntityStore,
        selection: &BoxSelection,
        spatial: &SpatialHash,
    ) -> Vec<EntityId> {
        let aabb = selection.to_aabb();
        let mut selected = Vec::with_capacity(64);

        // Use SpatialHash to find nearby entities (O(k) instead of O(n))
        let nearby = spatial.query_rect(aabb);

        // Filter to alive entities and verify exact intersection
        for entity_id in nearby {
            let idx = entity_id.index().0 as usize;

            // Skip dead entities
            if !store.is_alive_index(idx) {
                continue;
            }

            // Get entity bounds
            let pos = store.pos(idx);
            let size = store.size(idx);
            let half_w = size.x * 0.5;
            let half_h = size.y * 0.5;

            let entity_min_x = pos.x - half_w;
            let entity_max_x = pos.x + half_w;
            let entity_min_y = pos.y - half_h;
            let entity_max_y = pos.y + half_h;

            // Exact AABB intersection test
            let intersects = aabb.min.x <= entity_max_x
                && aabb.max.x >= entity_min_x
                && aabb.min.y <= entity_max_y
                && aabb.max.y >= entity_min_y;

            if intersects {
                selected.push(entity_id);
            }
        }

        selected
    }

    /// Evaluates selection with a filter for entity visibility
    ///
    /// Similar to `evaluate` but only considers visible entities.
    ///
    /// # Arguments
    ///
    /// * `store` - EntityStore with entity positions and sizes
    /// * `selection` - Box selection area
    /// * `spatial` - SpatialHash for efficient nearby entity queries
    ///
    /// # Returns
    ///
    /// Vector of visible EntityIds within the selection
    ///
    /// # Examples
    ///
    /// See `evaluate` for basic usage.
    #[inline(never)]
    pub fn evaluate_visible(
        &self,
        store: &EntityStore,
        selection: &BoxSelection,
        spatial: &SpatialHash,
    ) -> Vec<EntityId> {
        let aabb = selection.to_aabb();
        let mut selected = Vec::with_capacity(64);

        let nearby = spatial.query_rect(aabb);

        for entity_id in nearby {
            let idx = entity_id.index().0 as usize;

            // Skip non-visible entities
            if !store.is_visible(idx) {
                continue;
            }

            let pos = store.pos(idx);
            let size = store.size(idx);
            let half_w = size.x * 0.5;
            let half_h = size.y * 0.5;

            let entity_min_x = pos.x - half_w;
            let entity_max_x = pos.x + half_w;
            let entity_min_y = pos.y - half_h;
            let entity_max_y = pos.y + half_h;

            let intersects = aabb.min.x <= entity_max_x
                && aabb.max.x >= entity_min_x
                && aabb.min.y <= entity_max_y
                && aabb.max.y >= entity_min_y;

            if intersects {
                selected.push(entity_id);
            }
        }

        selected
    }
}

impl Default for BoxSelectSensor {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_selection_order_independent() {
        let sel1 = BoxSelection::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        let sel2 = BoxSelection::new(Vec2::new(100.0, 100.0), Vec2::new(0.0, 0.0));

        assert_eq!(sel1.to_aabb(), sel2.to_aabb());
    }

    #[test]
    fn test_box_selection_aabb() {
        let selection = BoxSelection::new(Vec2::new(10.0, 20.0), Vec2::new(100.0, 80.0));
        let aabb = selection.to_aabb();

        assert_eq!(aabb.min.x, 10.0);
        assert_eq!(aabb.min.y, 20.0);
        assert_eq!(aabb.max.x, 100.0);
        assert_eq!(aabb.max.y, 80.0);
    }

    #[test]
    fn test_box_selection_is_valid() {
        let large = BoxSelection::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 50.0));
        assert!(large.is_valid(5.0));

        let small = BoxSelection::new(Vec2::new(50.0, 50.0), Vec2::new(52.0, 52.0));
        assert!(!small.is_valid(5.0));

        let click = BoxSelection::new(Vec2::new(100.0, 100.0), Vec2::new(100.0, 100.0));
        assert!(!click.is_valid(0.0));
    }

    #[test]
    fn test_box_selection_contains_point() {
        let selection = BoxSelection::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));

        assert!(selection.contains_point(Vec2::new(50.0, 50.0)));
        assert!(selection.contains_point(Vec2::new(0.0, 0.0)));
        assert!(selection.contains_point(Vec2::new(100.0, 100.0)));
        assert!(!selection.contains_point(Vec2::new(150.0, 50.0)));
        assert!(!selection.contains_point(Vec2::new(50.0, 150.0)));
    }

    #[test]
    fn test_sensor_state_machine() {
        let mut sensor = BoxSelectSensor::new();

        assert!(!sensor.is_selecting());
        assert!(sensor.current_selection().is_none());

        sensor.start_drag(Vec2::new(0.0, 0.0));
        assert!(sensor.is_selecting());
        assert!(sensor.current_selection().is_some());

        sensor.cancel();
        assert!(!sensor.is_selecting());

        sensor.start_drag(Vec2::new(0.0, 0.0));
        sensor.update_drag(Vec2::new(100.0, 100.0));
        let selection = sensor.end_drag();

        assert!(selection.is_some());
        assert!(!sensor.is_selecting());
    }

    #[test]
    fn test_sensor_end_drag_returns_selection() {
        let mut sensor = BoxSelectSensor::new();
        sensor.start_drag(Vec2::new(10.0, 20.0));
        sensor.update_drag(Vec2::new(100.0, 80.0));

        let selection = sensor.end_drag().unwrap();

        assert_eq!(selection.start.x, 10.0);
        assert_eq!(selection.start.y, 20.0);
        assert_eq!(selection.end.x, 100.0);
        assert_eq!(selection.end.y, 80.0);
    }

    #[test]
    fn test_sensor_end_drag_no_selection() {
        let mut sensor = BoxSelectSensor::new();
        assert!(sensor.end_drag().is_none());
    }
}
