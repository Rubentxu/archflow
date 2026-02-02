// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Collision Sensor (Touch Sensor) Implementation
//
// This sensor detects when entities are colliding using AABB (Axis-Aligned
// Bounding Box) intersection tests with 6-tick history tracking.
//
// Reference: docs/epics/EPIC-002-physics-sensors.md - HU-006
//
// Performance Characteristics:
// - O(n) where n = number of entities in nearby spatial cells
// - Uses SpatialHash for O(1) broad-phase queries
// - AABB narrow-phase intersection test
// - 6-tick history for edge detection (enter/exit)
//
// Memory Impact:
// - 1 byte per entity (SignalByte for collision state)
// - 100KB for 100,000 entities
// ═══════════════════════════════════════════════════════════════════════════════

use crate::signals::SignalByte;
use alloc::vec;
use alloc::vec::Vec;
use archflow_core::{EntityId, Rect, Vec2};
use archflow_engine::{EntityStore, SpatialHash};

/// Global counter for generating unique sensor IDs
/// Using u32 with manual increment (thread-safety not needed in single-threaded context)
static mut COLLISION_SENSOR_ID_COUNTER: u32 = 1;

/// Sensor that detects collisions between entities using AABB intersection
///
/// This sensor combines broad-phase (SpatialHash) and narrow-phase (AABB test)
/// collision detection to efficiently find colliding entity pairs.
///
/// # Architecture
///
/// ```
/// +-------------------+
/// | CollisionSensor   |
/// +-------------------+
///         |
///         v
/// +-------------------+     +------------------+
/// | SpatialHash Grid  | --> | Nearby Entities  | (Broad-phase O(1))
/// +-------------------+     +------------------+
///                                  |
///                                  v
///                         +-------------------+
///                         | AABB Intersection | (Narrow-phase)
///                         +-------------------+
///                                  |
///                                  v
///                         +-------------------+
///                         | SignalByte Update | (6-tick history)
///                         +-------------------+
/// ```
///
/// # Examples
///
/// ```
/// use archflow_logic::sensors::collision::CollisionSensor;
/// use archflow_engine::{EntityStore, SpatialHash, MAX_ENTITIES};
/// use archflow_core::Vec2;
///
/// let mut store = EntityStore::new();
/// let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
/// let entity2 = store.spawn(Vec2::new(110.0, 110.0), Vec2::new(50.0, 50.0));
///
/// let mut spatial = SpatialHash::new(MAX_ENTITIES);
/// // Insert entities into spatial hash...
///
/// let mut sensor = CollisionSensor::new(MAX_ENTITIES);
/// sensor.evaluate(entity1, &store, &spatial);
///
/// if sensor.is_colliding(entity1) {
///     // Entities are colliding
/// }
///
/// if sensor.on_collision_enter(entity1) {
///     // Just started colliding this frame
/// }
/// ```
///
/// # Performance
///
/// - **Time**: O(k) where k = entities in nearby spatial cells (typically << n)
/// - **Space**: 1 byte per entity
/// - **Allocations**: Zero (pre-allocated on construction)
pub struct CollisionSensor {
    /// Signal history for each entity
    ///
    /// Each SignalByte stores 6 ticks of collision state:
    /// - bit 0 (T0): current frame
    /// - bits 1-5 (T1-T5): previous 5 frames
    signals: Vec<SignalByte>,

    /// Unique sensor ID for pulse routing
    sensor_id: u32,
}

impl CollisionSensor {
    /// Creates a new CollisionSensor with capacity for the maximum number of entities
    ///
    /// # Examples
    ///
    /// ```
    /// let sensor = CollisionSensor::new(MAX_ENTITIES);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        // Safety: This is safe in single-threaded context
        // For multi-threaded use, would need atomic operations or Mutex
        let sensor_id = unsafe {
            let id = COLLISION_SENSOR_ID_COUNTER;
            COLLISION_SENSOR_ID_COUNTER = COLLISION_SENSOR_ID_COUNTER.wrapping_add(1);
            id
        };

        Self {
            signals: vec![SignalByte::default(); capacity],
            sensor_id,
        }
    }

    /// Get the unique sensor ID
    ///
    /// This ID is used to route pulses to connected actuators.
    #[inline(always)]
    #[must_use]
    pub const fn sensor_id(&self) -> u32 {
        self.sensor_id
    }

    /// Evaluates collision state for a single entity against all nearby entities
    ///
    /// This method performs broad-phase query via SpatialHash to find nearby
    /// entities, then performs narrow-phase AABB intersection tests.
    ///
    /// # Arguments
    ///
    /// * `entity` - Entity to check collisions for
    /// * `store` - EntityStore containing all entity transforms
    /// * `spatial` - SpatialHash for efficient nearby entity queries
    ///
    /// # Complexity
    ///
    /// O(k) where k = entities in nearby spatial cells (typically << total entities)
    ///
    /// # Performance
    ///
    /// - Zero-allocation
    /// - Cache-friendly (spatial query returns contiguous memory)
    /// - Early exit on first collision found
    ///
    /// # Examples
    ///
    /// ```
    /// sensor.evaluate(entity1, &store, &spatial);
    /// ```
    #[inline(never)] // Prevent inlining to keep binary size small
    pub fn evaluate(&mut self, entity: EntityId, store: &EntityStore, spatial: &SpatialHash) {
        let entity_idx = entity.index().0 as usize;

        // Bounds check for safety
        if entity_idx >= self.signals.len() {
            return;
        }

        // Get entity AABB
        let entity_aabb = self.entity_aabb(store, entity);

        // Broad-phase: query spatial hash for potentially colliding entities
        let nearby = spatial.query_rect(entity_aabb);

        // Narrow-phase: check AABB intersection with nearby entities
        let mut is_colliding = false;

        for &other in &nearby {
            // Skip self
            if other == entity {
                continue;
            }

            // Get other entity AABB
            let other_aabb = self.entity_aabb(store, other);

            // AABB intersection test (narrow-phase)
            if entity_aabb.intersects(&other_aabb) {
                is_colliding = true;
                break; // Early exit on first collision
            }
        }

        // Update 6-tick history for this entity
        self.signals[entity_idx].push(is_colliding);
    }

    /// Returns true if the entity is currently colliding
    ///
    /// This checks the current frame (tick T0) only.
    ///
    /// # Examples
    ///
    /// ```
    /// sensor.evaluate(entity, &store, &spatial);
    /// if sensor.is_colliding(entity) {
    ///     // Entity is colliding this frame
    /// }
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn is_colliding(&self, entity: EntityId) -> bool {
        let idx = entity.index().0 as usize;
        if idx < self.signals.len() {
            self.signals[idx].get_current()
        } else {
            false
        }
    }

    /// Detects the moment when entity starts colliding (rising edge)
    ///
    /// Returns true only on the frame when collision transitions from
    /// no collision (0) to collision (1).
    ///
    /// # Examples
    ///
    /// ```
    /// if sensor.on_collision_enter(entity) {
    ///     // Entity just started colliding
    ///     // Play sound, trigger event, etc.
    /// }
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn on_collision_enter(&self, entity: EntityId) -> bool {
        let idx = entity.index().0 as usize;
        if idx < self.signals.len() {
            self.signals[idx].is_rising_edge()
        } else {
            false
        }
    }

    /// Detects the moment when entity stops colliding (falling edge)
    ///
    /// Returns true only on the frame when collision transitions from
    /// collision (1) to no collision (0).
    ///
    /// # Examples
    ///
    /// ```
    /// if sensor.on_collision_exit(entity) {
    ///     // Entity just stopped colliding
    ///     // End collision effects, etc.
    /// }
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn on_collision_exit(&self, entity: EntityId) -> bool {
        let idx = entity.index().0 as usize;
        if idx < self.signals.len() {
            self.signals[idx].is_falling_edge()
        } else {
            false
        }
    }

    /// Returns true if entity has been colliding for N consecutive ticks
    ///
    /// This is useful for debouncing and detecting intentional collisions.
    /// For example, `is_steady_colliding(entity, 6)` means "entity has been
    /// colliding for 6 consecutive frames (100ms at 60 FPS)".
    ///
    /// # Arguments
    ///
    /// * `entity` - Entity to check
    /// * `ticks` - Number of consecutive ticks required (1-6)
    ///
    /// # Examples
    ///
    /// ```
    /// // Only trigger after 100ms of sustained contact
    /// if sensor.is_steady_colliding(entity, 6) {
    ///     apply_damage(entity);
    /// }
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn is_steady_colliding(&self, entity: EntityId, ticks: u8) -> bool {
        let idx = entity.index().0 as usize;
        if idx < self.signals.len() {
            self.signals[idx].is_steady(ticks)
        } else {
            false
        }
    }

    /// Get the SignalByte for an entity (advanced usage)
    ///
    /// Returns a copy of the signal history for the entity.
    /// This is useful for advanced pattern matching.
    ///
    /// # Examples
    ///
    /// ```
    /// let signal = sensor.signal(entity);
    /// if signal.has_noise() {
    ///     // Collision is flickering, entity may be on boundary
    /// }
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn signal(&self, entity: EntityId) -> SignalByte {
        let idx = entity.index().0 as usize;
        if idx < self.signals.len() {
            self.signals[idx]
        } else {
            SignalByte::new()
        }
    }

    /// Extract the AABB for an entity from the EntityStore
    #[inline(always)]
    fn entity_aabb(&self, store: &EntityStore, entity: EntityId) -> Rect {
        let idx = entity.index().0 as usize;
        // transforms[idx] = [x, y, width, height]
        let x = store.transforms[idx][0];
        let y = store.transforms[idx][1];
        let w = store.transforms[idx][2];
        let h = store.transforms[idx][3];
        Rect::new(x, y, x + w, y + h)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS (inline for documentation examples)
// ═════════════════════════════════════════════════════════════════════════════==

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full integration tests are in tests/collision_tests.rs
    // These are just quick sanity checks for the implementation

    #[test]
    fn test_capacity() {
        let sensor = CollisionSensor::new(100);
        assert_eq!(sensor.signals.len(), 100);
    }

    #[test]
    fn test_signals_initialized_to_zero() {
        let sensor = CollisionSensor::new(10);
        assert_eq!(sensor.signals[0].as_u8(), 0);
    }

    #[test]
    fn test_sensor_id_is_unique() {
        let sensor1 = CollisionSensor::new(10);
        let sensor2 = CollisionSensor::new(10);
        assert_ne!(sensor1.sensor_id(), sensor2.sensor_id());
    }

    #[test]
    fn test_sensor_id_starts_greater_than_zero() {
        let sensor = CollisionSensor::new(10);
        assert!(sensor.sensor_id() > 0);
    }
}
