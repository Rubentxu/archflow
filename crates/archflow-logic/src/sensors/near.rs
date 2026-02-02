// ArchFlow Logic - Near Sensor (Proximity Sensor with Hysteresis)
//
// This sensor detects when entities are within a circular radius of another
// entity using hysteresis to prevent flickering at the detection boundary.
//
// Reference: docs/epics/EPIC-002-physics-sensors.md - HU-007

use crate::signals::SignalByte;
use alloc::vec;
use alloc::vec::Vec;
use archflow_core::{EntityId, Rect, Vec2};
use archflow_engine::{EntityStore, SpatialHash};
use core::sync::atomic::{AtomicU32, Ordering};

static NEAR_SENSOR_ID_COUNTER: AtomicU32 = AtomicU32::new(100);

pub struct NearSensor {
    signals: Vec<SignalByte>,
    distance: f32,
    reset_distance: f32,
    distance_sq: f32,
    reset_distance_sq: f32,
    target_tag: u32,
    invert: bool,
    sensor_id: u32,
}

impl NearSensor {
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
            target_tag: 0,
            invert: false,
            sensor_id,
        }
    }

    #[inline(always)]
    #[must_use]
    pub fn with_tag(capacity: usize, distance: f32, reset_distance: f32, target_tag: u32) -> Self {
        let mut sensor = Self::new(capacity, distance, reset_distance);
        sensor.target_tag = target_tag;
        sensor
    }

    #[inline(always)]
    #[must_use]
    pub const fn sensor_id(&self) -> u32 {
        self.sensor_id
    }

    #[inline(always)]
    #[must_use]
    pub const fn distance(&self) -> f32 {
        self.distance
    }

    #[inline(always)]
    #[must_use]
    pub const fn reset_distance(&self) -> f32 {
        self.reset_distance
    }

    #[inline(always)]
    pub fn set_invert(&mut self, invert: bool) {
        self.invert = invert;
    }

    #[inline(never)]
    pub fn evaluate(&mut self, entity: EntityId, store: &EntityStore, spatial: &SpatialHash) {
        let entity_idx = entity.index().0 as usize;

        if entity_idx >= self.signals.len() {
            return;
        }

        let pos = store.pos(entity_idx);
        let currently_near = self.signals[entity_idx].get_current();

        // Broad-phase: query spatial hash with reset_radius (larger for hysteresis)
        // Query rect is centered on entity, with side length = reset_distance * 2
        // to ensure we catch entities up to reset_distance away
        let query_side = self.reset_distance * 2.0;
        let half_side = self.reset_distance;
        let query_rect = Rect::from_origin_size(
            Vec2::new(pos.x - half_side, pos.y - half_side),
            Vec2::new(query_side, query_side),
        );
        let nearby = spatial.query_rect(query_rect);

        let threshold_sq = if currently_near {
            self.reset_distance_sq
        } else {
            self.distance_sq
        };

        let mut is_near_any = false;

        for &other in &nearby {
            if other.index().0 as usize == entity_idx {
                continue;
            }

            let other_pos = store.pos(other.index().0 as usize);
            let dx = pos.x - other_pos.x;
            let dy = pos.y - other_pos.y;
            let dist_sq = dx * dx + dy * dy;

            if dist_sq <= threshold_sq {
                is_near_any = true;
                break;
            }
        }

        let result = if self.invert {
            !is_near_any
        } else {
            is_near_any
        };
        self.signals[entity_idx].push(result);
    }

    #[inline(always)]
    #[must_use]
    pub fn is_near(&self, entity: EntityId) -> bool {
        let idx = entity.index().0 as usize;
        if idx >= self.signals.len() {
            return false;
        }
        self.signals[idx].get_current()
    }

    #[inline(always)]
    #[must_use]
    pub fn on_proximity_enter(&self, entity: EntityId) -> bool {
        let idx = entity.index().0 as usize;
        if idx >= self.signals.len() {
            return false;
        }
        self.signals[idx].is_rising_edge()
    }

    #[inline(always)]
    #[must_use]
    pub fn on_proximity_exit(&self, entity: EntityId) -> bool {
        let idx = entity.index().0 as usize;
        if idx >= self.signals.len() {
            return false;
        }
        self.signals[idx].is_falling_edge()
    }

    #[inline(always)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.signals.len()
    }

    #[inline(always)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.signals.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_core::{Rect, Vec2};
    use archflow_engine::MAX_ENTITIES;

    #[test]
    fn test_new_validates_hysteresis() {
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

    /// Integration test that uses the full SpatialHash correctly
    #[test]
    fn test_near_sensor_integration() {
        use archflow_core::Vec2;

        let mut store = EntityStore::new();
        let mut spatial = SpatialHash::new(MAX_ENTITIES);

        // Create entities at positions aligned with spatial cells
        // SpatialHash uses 64x64 cells (CELL_SIZE = 64.0)
        // Position (200, 200) is at cell (3, 3): 200/64 = 3.125 → floor = 3
        let entity_a = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(10.0, 10.0));
        let bounds_a = Rect::from_origin_size(Vec2::new(195.0, 195.0), Vec2::new(10.0, 10.0));
        spatial.insert(entity_a, bounds_a);

        // Entity B at 50 units: (250, 200) - same row of cells (row 3)
        // 250/64 = 3.9 → floor = 3, so both entities in cell (3, 3)
        let entity_b = store.spawn(Vec2::new(250.0, 200.0), Vec2::new(10.0, 10.0));
        let bounds_b = Rect::from_origin_size(Vec2::new(245.0, 195.0), Vec2::new(10.0, 10.0));
        spatial.insert(entity_b, bounds_b);

        let mut sensor = NearSensor::new(MAX_ENTITIES, 60.0, 80.0);

        // Entity B is 50 units away from entity A, which is < 60, so should be detected
        sensor.evaluate(entity_a, &store, &spatial);
        assert!(
            sensor.is_near(entity_a),
            "Entity at 50 units should be detected"
        );

        // Now test hysteresis by moving entity B
        spatial.remove(entity_b);
        store.set_pos(entity_b.index().0 as usize, Vec2::new(270.0, 200.0)); // 70 units away
        let bounds_b = Rect::from_origin_size(Vec2::new(265.0, 195.0), Vec2::new(10.0, 10.0));
        spatial.insert(entity_b, bounds_b);

        sensor.evaluate(entity_a, &store, &spatial);
        assert!(
            sensor.is_near(entity_a),
            "Entity at 70 units should still be detected (hysteresis)"
        );

        // Move beyond hysteresis band
        spatial.remove(entity_b);
        store.set_pos(entity_b.index().0 as usize, Vec2::new(300.0, 200.0)); // 100 units away
        let bounds_b = Rect::from_origin_size(Vec2::new(295.0, 195.0), Vec2::new(10.0, 10.0));
        spatial.insert(entity_b, bounds_b);

        sensor.evaluate(entity_a, &store, &spatial);
        assert!(
            !sensor.is_near(entity_a),
            "Entity at 100 units should not be detected"
        );
    }

    /// Test that verifies rising and falling edges work correctly
    #[test]
    fn test_hysteresis_edges_with_proper_spatial() {
        use archflow_core::Vec2;

        let mut store = EntityStore::new();
        let mut spatial = SpatialHash::new(MAX_ENTITIES);

        let entity_a = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(10.0, 10.0));
        let bounds_a = Rect::from_origin_size(Vec2::new(195.0, 195.0), Vec2::new(10.0, 10.0));
        spatial.insert(entity_a, bounds_a);

        let entity_b = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(10.0, 10.0));
        let bounds_b = Rect::from_origin_size(Vec2::new(195.0, 195.0), Vec2::new(10.0, 10.0));
        spatial.insert(entity_b, bounds_b);

        let mut sensor = NearSensor::new(MAX_ENTITIES, 60.0, 80.0);

        assert!(!sensor.is_near(entity_a));
        assert!(!sensor.on_proximity_enter(entity_a));

        // Move B close (50 units < 60)
        spatial.remove(entity_b);
        store.set_pos(entity_b.index().0 as usize, Vec2::new(250.0, 200.0));
        let bounds_b = Rect::from_origin_size(Vec2::new(245.0, 195.0), Vec2::new(10.0, 10.0));
        spatial.insert(entity_b, bounds_b);

        sensor.evaluate(entity_a, &store, &spatial);
        assert!(sensor.is_near(entity_a));
        assert!(
            sensor.on_proximity_enter(entity_a),
            "Should detect enter edge"
        );

        // Move to hysteresis band (70 units, between 60 and 80)
        spatial.remove(entity_b);
        store.set_pos(entity_b.index().0 as usize, Vec2::new(270.0, 200.0));
        let bounds_b = Rect::from_origin_size(Vec2::new(265.0, 195.0), Vec2::new(10.0, 10.0));
        spatial.insert(entity_b, bounds_b);

        sensor.evaluate(entity_a, &store, &spatial);
        assert!(sensor.is_near(entity_a));
        assert!(
            !sensor.on_proximity_enter(entity_a),
            "No enter edge in hysteresis band"
        );

        // Move far (100 units > 80)
        spatial.remove(entity_b);
        store.set_pos(entity_b.index().0 as usize, Vec2::new(300.0, 200.0));
        let bounds_b = Rect::from_origin_size(Vec2::new(295.0, 195.0), Vec2::new(10.0, 10.0));
        spatial.insert(entity_b, bounds_b);

        sensor.evaluate(entity_a, &store, &spatial);
        assert!(!sensor.is_near(entity_a));
        assert!(
            sensor.on_proximity_exit(entity_a),
            "Should detect exit edge"
        );
    }
}
