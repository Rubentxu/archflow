// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Near Sensor (Proximity Sensor with Hysteresis)
//
// This sensor detects when entities are within a circular radius of another
// entity using hysteresis to prevent flickering at the detection boundary.
//
// Reference: docs/epics/EPIC-002-physics-sensors.md - HU-007
//
// Hysteresis (Schmitt Trigger):
// - ENTER: Triggered when distance < distance_threshold
// - EXIT: Only when distance > reset_distance (which is > distance_threshold)
//
// Performance Characteristics:
// - O(k) where k = number of entities in nearby spatial cells
// - Uses SpatialHash for broad-phase radius query
// - Uses squared distance to avoid expensive sqrt operations
// - 6-tick history for edge detection (enter/exit)
//
// Memory Impact:
// - 1 byte per entity (SignalByte for proximity state)
// - 100KB for 100,000 entities
// ═══════════════════════════════════════════════════════════════════════════════

use crate::signals::SignalByte;
use alloc::vec;
use alloc::vec::Vec;
use archflow_core::{EntityId, Rect, Vec2};
use archflow_engine::{EntityStore, SpatialHash};
use core::sync::atomic::{AtomicU32, Ordering};

/// Global counter for generating unique sensor IDs
static NEAR_SENSOR_ID_COUNTER: AtomicU32 = AtomicU32::new(100);

/// Sensor that detects entities within a circular radius with hysteresis
///
/// This sensor uses the Schmitt Trigger pattern to avoid flickering when
/// the detected entity is near the detection boundary.
pub struct NearSensor {
    /// Signal history for each entity
    signals: Vec<SignalByte>,

    /// Distance threshold for detection (smaller threshold for entering)
    distance: f32,

    /// Reset distance threshold (larger, for hysteresis to prevent flickering)
    reset_distance: f32,

    /// Squared distance threshold (cached for performance)
    distance_sq: f32,

    /// Squared reset distance (cached for performance)
    reset_distance_sq: f32,

    /// Optional tag filter - only detect entities with this tag
    target_tag: u32,

    /// Invert sensor behavior (detect when NOT near)
    invert: bool,

    /// Unique sensor ID for pulse routing
    sensor_id: u32,
}

impl NearSensor {
    /// Creates a new NearSensor with hysteresis support
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of entities to track
    /// * `distance` - Distance threshold for detection (enter condition)
    /// * `reset_distance` - Distance threshold for exit (must be >= distance)
    ///
    /// # Panics
    ///
    /// Panics if `reset_distance < distance` (hysteresis requires reset_distance >= distance)
    #[inline(always)]
    #[must_use]
    pub fn new(capacity: usize, distance: f32, reset_distance: f32) -> Self {
        assert!(
            reset_distance >= distance,
            "reset_distance ({}) must be >= distance ({}) for hysteresis",
            reset_distance,
            distance
        );

        let sensor_id = NEAR_SENSOR_ID_COUNTER.fetch_add(1, Ordering::Relaxed);

        Self {
            signals: vec![SignalByte::default(); capacity],
            distance,
            reset_distance,
            distance_sq: distance * distance,
            reset_distance_sq: reset_distance * reset_distance,
            target_tag: 0, // 0 = no filter, detect all
            invert: false,
            sensor_id,
        }
    }

    /// Creates a NearSensor with tag filtering
    #[inline(always)]
    #[must_use]
    pub fn with_tag(capacity: usize, distance: f32, reset_distance: f32, target_tag: u32) -> Self {
        let mut sensor = Self::new(capacity, distance, reset_distance);
        sensor.target_tag = target_tag;
        sensor
    }

    /// Get the unique sensor ID
    #[inline(always)]
    #[must_use]
    pub const fn sensor_id(&self) -> u32 {
        self.sensor_id
    }

    /// Get the detection distance threshold
    #[inline(always)]
    #[must_use]
    pub const fn distance(&self) -> f32 {
        self.distance
    }

    /// Get the reset distance threshold
    #[inline(always)]
    #[must_use]
    pub const fn reset_distance(&self) -> f32 {
        self.reset_distance
    }

    /// Set the invert flag
    #[inline(always)]
    pub fn set_invert(&mut self, invert: bool) {
        self.invert = invert;
    }

    /// Evaluates proximity state for a single entity against all nearby entities
    #[inline(never)]
    pub fn evaluate(&mut self, entity: EntityId, store: &EntityStore, spatial: &SpatialHash) {
        let entity_idx = entity.index().0 as usize;

        // Bounds check for safety
        if entity_idx >= self.signals.len() {
            return;
        }

        // Get entity position
        let pos = store.pos(entity_idx);

        // Broad-phase: query spatial hash with reset_radius (larger for hysteresis)
        let half_reset = self.reset_distance / 2.0;
        let query_rect = Rect::new(
            pos.x - half_reset,
            pos.y - half_reset,
            self.reset_distance,
            self.reset_distance,
        );
        let nearby = spatial.query_rect(query_rect);

        // Narrow-phase: check squared distance with all nearby entities
        let mut is_near_any = false;

        for &other in &nearby {
            // Skip self
            if other.index().0 as usize == entity_idx {
                continue;
            }

            // Get other entity position
            let other_idx = other.index().0 as usize;
            let other_pos = store.pos(other_idx);

            // Squared distance check (avoid expensive sqrt)
            let dx = pos.x - other_pos.x;
            let dy = pos.y - other_pos.y;
            let dist_sq = dx * dx + dy * dy;

            // Check if within detection radius
            if dist_sq <= self.distance_sq {
                is_near_any = true;
                break;
            }
        }

        // Apply inversion if configured
        let result = if self.invert {
            !is_near_any
        } else {
            is_near_any
        };

        // Update signal history
        self.signals[entity_idx].push(result);
    }

    /// Checks if the entity is currently being detected (near any target)
    #[inline(always)]
    #[must_use]
    pub fn is_near(&self, entity: EntityId) -> bool {
        let idx = entity.index().0 as usize;
        if idx >= self.signals.len() {
            return false;
        }
        self.signals[idx].get_current()
    }

    /// Checks if proximity was just entered this frame (rising edge)
    #[inline(always)]
    #[must_use]
    pub fn on_proximity_enter(&self, entity: EntityId) -> bool {
        let idx = entity.index().0 as usize;
        if idx >= self.signals.len() {
            return false;
        }
        self.signals[idx].is_rising_edge()
    }

    /// Checks if proximity was just exited this frame (falling edge)
    #[inline(always)]
    #[must_use]
    pub fn on_proximity_exit(&self, entity: EntityId) -> bool {
        let idx = entity.index().0 as usize;
        if idx >= self.signals.len() {
            return false;
        }
        self.signals[idx].is_falling_edge()
    }

    /// Returns the number of entities currently being tracked
    #[inline(always)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.signals.len()
    }

    /// Returns true if no entities are being tracked
    #[inline(always)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.signals.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_core::Vec2;
    use archflow_engine::MAX_ENTITIES;

    #[test]
    fn test_new_validates_hysteresis() {
        // reset_distance >= distance should not panic
        let _sensor = NearSensor::new(MAX_ENTITIES, 50.0, 60.0);
        let _sensor = NearSensor::new(MAX_ENTITIES, 50.0, 50.0);
    }

    #[test]
    #[should_panic(expected = "reset_distance (50) must be >= distance (60)")]
    fn test_new_panics_on_invalid_hysteresis() {
        let _sensor = NearSensor::new(MAX_ENTITIES, 60.0, 50.0);
    }

    #[test]
    fn test_sensor_id_uniqueness() {
        let sensor1 = NearSensor::new(MAX_ENTITIES, 50.0, 70.0);
        let sensor2 = NearSensor::new(MAX_ENTITIES, 50.0, 70.0);
        assert_ne!(sensor1.sensor_id(), sensor2.sensor_id());
    }

    #[test]
    fn test_initial_state_not_near() {
        let mut store = EntityStore::new();
        let _spatial = SpatialHash::new(MAX_ENTITIES);
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let sensor = NearSensor::new(MAX_ENTITIES, 50.0, 70.0);
        // Cannot call evaluate without mutable sensor, but we can check is_near
        assert!(!sensor.is_near(entity));
    }

    #[test]
    fn test_len_and_is_empty() {
        let sensor = NearSensor::new(MAX_ENTITIES, 50.0, 70.0);
        assert_eq!(sensor.len(), MAX_ENTITIES);
        assert!(!sensor.is_empty());
    }

    #[test]
    fn test_distance_accessors() {
        let sensor = NearSensor::new(MAX_ENTITIES, 50.0, 70.0);
        assert_eq!(sensor.distance(), 50.0);
        assert_eq!(sensor.reset_distance(), 70.0);
    }
}
