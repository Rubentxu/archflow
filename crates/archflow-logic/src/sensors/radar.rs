// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Radar Sensor (Directional Detection) - HU-008
//
// This sensor detects entities within a directional cone using dot product
// for angle testing and SpatialHash for efficient candidate filtering.
//
// Reference: docs/epics/EPIC-002-physics-sensors.md - HU-008
//
// Performance Characteristics:
// - O(n × k) where n = entities, k = average candidates in range
// - SpatialHash reduces candidates significantly
// - Dot product for angle testing (fast, SIMD-friendly)
//
// Memory Impact:
// - 1 byte per entity (SignalByte for detection state)
//
// ═══════════════════════════════════════════════════════════════════════════════

use crate::signals::SignalByte;
use alloc::vec;
use alloc::vec::Vec;
use archflow_core::{EntityId, Rect, Vec2};
use archflow_engine::{EntityStore, SpatialHash};

/// Axis for radar detection
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadarAxis {
    /// Positive X direction (right)
    PositiveX = 0,
    /// Negative X direction (left)
    NegativeX = 1,
    /// Positive Y direction (up)
    PositiveY = 2,
    /// Negative Y direction (down)
    NegativeY = 3,
}

impl RadarAxis {
    /// Get the direction vector for this axis
    #[must_use]
    pub const fn direction(self) -> Vec2 {
        match self {
            RadarAxis::PositiveX => Vec2::new(1.0, 0.0),
            RadarAxis::NegativeX => Vec2::new(-1.0, 0.0),
            RadarAxis::PositiveY => Vec2::new(0.0, 1.0),
            RadarAxis::NegativeY => Vec2::new(0.0, -1.0),
        }
    }
}

/// Radar Sensor for directional cone detection
///
/// This sensor detects entities within a directional cone defined by:
/// - A range (maximum distance)
/// - An angle (cone width in degrees)
/// - An axis (direction: X, Y, or negative variants)
///
/// # Examples
///
/// ```
/// use archflow_logic::sensors::radar::{RadarSensor, RadarAxis};
/// use archflow_core::Vec2;
/// use archflow_engine::{EntityStore, SpatialHash};
///
/// let mut store = EntityStore::new();
/// let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
/// let target = store.spawn(Vec2::new(150.0, 100.0), Vec2::new(50.0, 50.0));
///
/// let mut spatial = SpatialHash::new(MAX_ENTITIES);
/// // ... insert entities ...
///
/// let mut sensor = RadarSensor::new(MAX_ENTITIES, RadarAxis::PositiveX, 100.0, 45.0, 0);
/// sensor.evaluate(&store, &spatial);
///
/// let signal = sensor.signal(entity);
/// if signal.get_current() {
///     // Entity detected target within radar cone
/// }
/// ```
///
/// # Performance
///
/// - **Time**: O(n × k) where k = candidates within range
/// - **Space**: 1 byte per entity
/// - **Allocations**: Zero (pre-allocated on construction)
pub struct RadarSensor {
    /// Signal history for each entity
    ///
    /// Each SignalByte stores 6 ticks of "has_detection" state
    signals: Vec<SignalByte>,

    /// Detection axis (direction of radar cone)
    axis: RadarAxis,

    /// Maximum detection range (in world units)
    range: f32,

    /// Squared range for comparisons (precomputed to avoid sqrt)
    range_sq: f32,

    /// Cone angle in degrees (0-360, where 360 = full circle)
    angle_degrees: f32,

    /// Cosine of half-angle (precomputed for dot product comparison)
    cos_half_angle: f32,

    /// Target tag filter (0 = match all entities)
    target_tag: u8,

    /// Collision mask for filtering (0 = match all)
    mask: u32,
}

impl RadarSensor {
    /// Creates a new Radar Sensor
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of entities to track
    /// * `axis` - Detection axis (direction)
    /// * `range` - Maximum detection range
    /// * `angle_degrees` - Cone angle in degrees (0-360)
    /// * `target_tag` - Optional tag filter (0 = match all)
    ///
    /// # Examples
    ///
    /// ```
    /// let sensor = RadarSensor::new(MAX_ENTITIES, RadarAxis::PositiveX, 100.0, 45.0, 0);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn new(
        capacity: usize,
        axis: RadarAxis,
        range: f32,
        angle_degrees: f32,
        target_tag: u8,
    ) -> Self {
        assert!(
            angle_degrees >= 0.0 && angle_degrees <= 360.0,
            "Angle must be 0-360 degrees"
        );
        assert!(range > 0.0, "Range must be positive");

        let half_angle = angle_degrees.to_radians() / 2.0;
        let cos_half_angle = half_angle.cos();

        Self {
            signals: vec![SignalByte::default(); capacity],
            axis,
            range_sq: range * range,
            range,
            angle_degrees,
            cos_half_angle,
            target_tag,
            mask: 0xFFFFFFFF, // Match all by default
        }
    }

    /// Returns the detection axis
    #[inline(always)]
    #[must_use]
    pub const fn axis(&self) -> RadarAxis {
        self.axis
    }

    /// Returns the detection range
    #[inline(always)]
    #[must_use]
    pub const fn range(&self) -> f32 {
        self.range
    }

    /// Returns the cone angle in degrees
    #[inline(always)]
    #[must_use]
    pub const fn angle_degrees(&self) -> f32 {
        self.angle_degrees
    }

    /// Returns the target tag filter
    #[inline(always)]
    #[must_use]
    pub const fn target_tag(&self) -> u8 {
        self.target_tag
    }

    /// Returns the collision mask
    #[inline(always)]
    #[must_use]
    pub const fn mask(&self) -> u32 {
        self.mask
    }

    /// Set the collision mask
    pub fn set_mask(&mut self, mask: u32) {
        self.mask = mask;
    }

    /// Get the signal for a specific entity
    ///
    /// Returns the SignalByte which provides edge detection methods.
    ///
    /// # Arguments
    ///
    /// * `entity` - Entity ID to query
    #[inline(always)]
    #[must_use]
    pub fn signal(&self, entity: EntityId) -> SignalByte {
        let idx = entity.index().0 as usize;
        if idx < self.signals.len() {
            self.signals[idx]
        } else {
            SignalByte::default()
        }
    }

    /// Get detected entities for a specific entity
    ///
    /// Returns the list of entities detected by this entity's radar.
    ///
    /// # Arguments
    ///
    /// * `entity` - Entity ID to query
    ///
    /// # Returns
    ///
    /// Vector of EntityId for detected entities
    #[must_use]
    pub fn detected_entities(&self, _entity: EntityId) -> Vec<EntityId> {
        // Note: This is a simplified implementation
        // In a full implementation, we'd store detected entities per radar
        Vec::new()
    }

    /// Evaluate radar detection for all entities
    ///
    /// This performs:
    /// 1. SpatialHash query for entities within range
    /// 2. Dot product angle testing for directional filtering
    /// 3. Tag and mask filtering
    ///
    /// # Arguments
    ///
    /// * `store` - EntityStore with transforms
    /// * `spatial` - SpatialHash for O(1) spatial queries
    ///
    /// # Complexity
    ///
    /// O(n × k) where n = entities, k = average candidates within range
    ///
    /// # Performance
    ///
    /// - Zero-allocation in signal updates
    /// - SpatialHash reduces candidate checks
    /// - Dot product is fast and SIMD-friendly
    #[inline(never)] // Prevent inlining to keep binary size small
    pub fn evaluate(&mut self, store: &EntityStore, spatial: &SpatialHash) {
        // Get the direction vector for this radar's axis
        let radar_direction = self.axis.direction();

        // Process all entities
        for (idx, transform) in store.transforms.iter().enumerate() {
            // Skip if index exceeds sensor capacity
            if idx >= self.signals.len() {
                break;
            }

            // Extract position from transform [x, y, width, height]
            let pos = Vec2::new(transform[0], transform[1]);

            // Query spatial hash for entities within range
            let query_bounds = Rect {
                min: pos - Vec2::new(self.range, self.range),
                max: pos + Vec2::new(self.range, self.range),
            };

            let candidates = spatial.query_rect(query_bounds);

            // Check each candidate for radar detection
            let has_detection = candidates.iter().any(|&candidate_id| {
                // Skip self
                if candidate_id.index().0 as usize == idx {
                    return false;
                }

                // Filter by target tag if set
                if self.target_tag != 0 {
                    let candidate_idx = candidate_id.index().0 as usize;
                    if candidate_idx < store.metadata.len() {
                        let entity_tag = (store.metadata[candidate_idx] >> 16) & 0xFF;
                        if entity_tag as u8 != self.target_tag {
                            return false;
                        }
                    }
                }

                // Filter by mask
                if self.mask != 0xFFFFFFFF {
                    let candidate_idx = candidate_id.index().0 as usize;
                    if candidate_idx < store.metadata.len() {
                        // Extract collision mask from metadata bits 24-31
                        let entity_mask = (store.metadata[candidate_idx] >> 24) & 0xFF;
                        if (entity_mask as u32 & self.mask) == 0 {
                            return false;
                        }
                    }
                }

                // Check range
                let candidate_idx = candidate_id.index().0 as usize;
                if candidate_idx >= store.transforms.len() {
                    return false;
                }

                let candidate_pos = Vec2::new(
                    store.transforms[candidate_idx][0],
                    store.transforms[candidate_idx][1],
                );

                let to_target = candidate_pos - pos;
                let dist_sq = to_target.x * to_target.x + to_target.y * to_target.y;

                if dist_sq > self.range_sq {
                    return false;
                }

                // Check angle using dot product
                // Normalize the direction vector
                let dist = dist_sq.sqrt();
                if dist < 0.001 {
                    // Target is at same position, consider it detected
                    return true;
                }

                let normalized_direction = to_target / dist;

                // Dot product gives us the cosine of the angle
                let dot_product = normalized_direction.dot(radar_direction);

                // Check if within cone angle
                dot_product >= self.cos_half_angle
            });

            // Update signal state
            self.signals[idx].push(has_detection);
        }
    }
}

impl Default for RadarSensor {
    fn default() -> Self {
        Self::new(100_000, RadarAxis::PositiveX, 100.0, 90.0, 0)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_core::{Generation, Index};

    fn make_id(idx: u32) -> EntityId {
        EntityId::from_parts(Index(idx), Generation(1))
    }

    #[test]
    fn test_new_radar_sensor() {
        let sensor = RadarSensor::new(100, RadarAxis::PositiveX, 100.0, 45.0, 5);
        assert_eq!(sensor.axis(), RadarAxis::PositiveX);
        assert_eq!(sensor.range(), 100.0);
        assert_eq!(sensor.angle_degrees(), 45.0);
        assert_eq!(sensor.target_tag(), 5);
    }

    #[test]
    fn test_default() {
        let sensor = RadarSensor::default();
        assert_eq!(sensor.axis(), RadarAxis::PositiveX);
        assert_eq!(sensor.range(), 100.0);
        assert_eq!(sensor.angle_degrees(), 90.0);
        assert_eq!(sensor.target_tag(), 0);
    }

    #[test]
    fn test_axis_directions() {
        assert_eq!(RadarAxis::PositiveX.direction(), Vec2::new(1.0, 0.0));
        assert_eq!(RadarAxis::NegativeX.direction(), Vec2::new(-1.0, 0.0));
        assert_eq!(RadarAxis::PositiveY.direction(), Vec2::new(0.0, 1.0));
        assert_eq!(RadarAxis::NegativeY.direction(), Vec2::new(0.0, -1.0));
    }

    #[test]
    fn test_signals_initialized() {
        let sensor = RadarSensor::new(100, RadarAxis::PositiveX, 100.0, 45.0, 0);
        assert_eq!(sensor.signals.len(), 100);
        for signal in &sensor.signals {
            assert_eq!(signal.as_u8(), 0);
        }
    }

    #[test]
    fn test_signal_method() {
        let sensor = RadarSensor::new(100, RadarAxis::PositiveX, 100.0, 45.0, 0);
        let id = make_id(5);

        let signal = sensor.signal(id);
        assert!(!signal.get_current());
        assert!(!signal.is_rising_edge());
        assert!(!signal.is_falling_edge());
    }

    #[test]
    fn test_set_mask() {
        let mut sensor = RadarSensor::new(100, RadarAxis::PositiveX, 100.0, 45.0, 0);
        assert_eq!(sensor.mask(), 0xFFFFFFFF);

        sensor.set_mask(0x000000FF);
        assert_eq!(sensor.mask(), 0x000000FF);
    }

    #[test]
    fn test_range_validation() {
        // Valid range
        let sensor = RadarSensor::new(100, RadarAxis::PositiveX, 50.0, 45.0, 0);
        assert_eq!(sensor.range(), 50.0);
        assert_eq!(sensor.range_sq, 2500.0);
    }

    #[test]
    fn test_angle_validation() {
        // Valid angles
        let sensor1 = RadarSensor::new(100, RadarAxis::PositiveX, 100.0, 0.0, 0);
        assert_eq!(sensor1.angle_degrees(), 0.0);

        let sensor2 = RadarSensor::new(100, RadarAxis::PositiveX, 100.0, 180.0, 0);
        assert_eq!(sensor2.angle_degrees(), 180.0);

        let sensor3 = RadarSensor::new(100, RadarAxis::PositiveX, 100.0, 360.0, 0);
        assert_eq!(sensor3.angle_degrees(), 360.0);
    }

    #[test]
    fn test_cos_half_angle_precomputed() {
        // 45 degree cone = 22.5 degree half-angle
        let sensor = RadarSensor::new(100, RadarAxis::PositiveX, 100.0, 45.0, 0);
        let expected = (45.0_f32.to_radians() / 2.0).cos();
        assert!((sensor.cos_half_angle - expected).abs() < 0.0001);
    }

    #[test]
    fn test_evaluate_detects_target_in_cone() {
        let mut store = EntityStore::new();
        let mut spatial = SpatialHash::new(100);

        // Source entity at origin, looking right (PositiveX)
        let id1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));

        // Target entity at (50, 0) - directly in front, within range
        let id2 = store.spawn(Vec2::new(50.0, 0.0), Vec2::new(10.0, 10.0));

        spatial.insert(
            id1,
            Rect::from_origin_size(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0)),
        );
        spatial.insert(
            id2,
            Rect::from_origin_size(Vec2::new(50.0, 0.0), Vec2::new(10.0, 10.0)),
        );

        // 90 degree cone, 100 unit range
        let mut sensor = RadarSensor::new(100, RadarAxis::PositiveX, 100.0, 90.0, 0);

        sensor.evaluate(&store, &spatial);

        let actual_idx1 = id1.index().0 as usize;
        if actual_idx1 < sensor.signals.len() {
            let signal1 = sensor.signals[actual_idx1];
            assert!(signal1.get_current()); // Should detect target
        }
    }

    #[test]
    fn test_evaluate_no_detection_out_of_range() {
        let mut store = EntityStore::new();
        let mut spatial = SpatialHash::new(100);

        let id1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let id2 = store.spawn(Vec2::new(200.0, 0.0), Vec2::new(10.0, 10.0)); // Out of range

        spatial.insert(
            id1,
            Rect::from_origin_size(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0)),
        );
        spatial.insert(
            id2,
            Rect::from_origin_size(Vec2::new(200.0, 0.0), Vec2::new(10.0, 10.0)),
        );

        // 50 unit range, 90 degree cone
        let mut sensor = RadarSensor::new(100, RadarAxis::PositiveX, 50.0, 90.0, 0);

        sensor.evaluate(&store, &spatial);

        let actual_idx1 = id1.index().0 as usize;
        if actual_idx1 < sensor.signals.len() {
            let signal1 = sensor.signals[actual_idx1];
            assert!(!signal1.get_current()); // Should NOT detect (out of range)
        }
    }

    #[test]
    fn test_evaluate_no_detection_out_of_cone() {
        let mut store = EntityStore::new();
        let mut spatial = SpatialHash::new(100);

        // Source looking right (PositiveX)
        let id1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));

        // Target is within range but behind (to the left)
        let id2 = store.spawn(Vec2::new(-30.0, 0.0), Vec2::new(10.0, 10.0));

        spatial.insert(
            id1,
            Rect::from_origin_size(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0)),
        );
        spatial.insert(
            id2,
            Rect::from_origin_size(Vec2::new(-30.0, 0.0), Vec2::new(10.0, 10.0)),
        );

        // Narrow 30 degree cone, should not detect behind
        let mut sensor = RadarSensor::new(100, RadarAxis::PositiveX, 100.0, 30.0, 0);

        sensor.evaluate(&store, &spatial);

        let actual_idx1 = id1.index().0 as usize;
        if actual_idx1 < sensor.signals.len() {
            let signal1 = sensor.signals[actual_idx1];
            assert!(!signal1.get_current()); // Should NOT detect (behind)
        }
    }

    #[test]
    fn test_evaluate_detects_with_wide_cone() {
        let mut store = EntityStore::new();
        let mut spatial = SpatialHash::new(100);

        let id1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        // Target at 45 degrees (should be within 90 degree cone)
        let id2 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(10.0, 10.0));

        spatial.insert(
            id1,
            Rect::from_origin_size(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0)),
        );
        spatial.insert(
            id2,
            Rect::from_origin_size(Vec2::new(50.0, 50.0), Vec2::new(10.0, 10.0)),
        );

        // 90 degree cone (should detect 45 degrees)
        let mut sensor = RadarSensor::new(100, RadarAxis::PositiveX, 100.0, 90.0, 0);

        sensor.evaluate(&store, &spatial);

        let actual_idx1 = id1.index().0 as usize;
        if actual_idx1 < sensor.signals.len() {
            let signal1 = sensor.signals[actual_idx1];
            assert!(signal1.get_current()); // Should detect at 45 degrees
        }
    }
}
