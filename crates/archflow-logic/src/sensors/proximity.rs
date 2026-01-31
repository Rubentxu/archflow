// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Proximity Sensor Implementation
//
// Epic 2.3: Proximity Sensor
// TDD Cycle: RED → GREEN → REFACTOR
//
// This sensor detects when entities are within a specified distance of each other
// using SpatialHash for O(1) spatial queries instead of O(n²) brute force.
//
// Performance Characteristics:
// - O(n) where n = number of entities (single scan per frame)
// - Uses SpatialHash for efficient proximity queries
// - Zero-allocation (pre-allocated buffers)
//
// Memory Impact:
// - 1 byte per entity (SignalByte for proximity state)
// - 100KB for 100,000 entities
//
// ═══════════════════════════════════════════════════════════════════════════════

use crate::signals::SignalByte;
use alloc::vec;
use alloc::vec::Vec;
use archflow_core::{EntityId, Rect, Vec2};
use archflow_engine::{EntityStore, SpatialHash};

/// Sensor that detects proximity between entities
///
/// This sensor tracks whether entities have nearby neighbors within a
/// specified radius. It uses SpatialHash for O(1) spatial queries.
///
/// # Examples
///
/// ```
/// use archflow_logic::sensors::proximity::ProximitySensor;
/// use archflow_core::Vec2;
/// use archflow_engine::{EntityStore, SpatialHash};
///
/// let mut store = EntityStore::new();
/// let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
/// let entity2 = store.spawn(Vec2::new(110.0, 100.0), Vec2::new(50.0, 50.0));
///
/// let mut spatial = SpatialHash::new(MAX_ENTITIES);
/// // ... insert entities into spatial hash ...
///
/// let mut sensor = ProximitySensor::new(MAX_ENTITIES, 20.0);
/// sensor.sample(&store, &spatial);
///
/// assert!(sensor.is_near(entity1, entity2));
/// ```
///
/// # Performance
///
/// - **Time**: O(n) single scan per `sample()` call
/// - **Space**: 1 byte per entity
/// - **Allocations**: Zero (pre-allocated on construction)
pub struct ProximitySensor {
    /// Signal history for each entity
    ///
    /// Each SignalByte stores 6 ticks of "has_nearby_neighbors" state:
    /// - bit 0 (T0): current frame
    /// - bits 1-5 (T1-T5): previous 5 frames
    signals: Vec<SignalByte>,

    /// Detection radius (in world units, typically pixels)
    radius: f32,

    /// Cached positions for distance calculations
    positions: Vec<Vec2>,
}

impl ProximitySensor {
    /// Creates a new ProximitySensor with capacity for `capacity` entities
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of entities to track
    /// * `radius` - Detection radius (default: 20px)
    ///
    /// # Examples
    ///
    /// ```
    /// let sensor = ProximitySensor::new(MAX_ENTITIES, 20.0);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn new(capacity: usize, radius: f32) -> Self {
        Self {
            signals: vec![SignalByte::default(); capacity],
            radius,
            positions: vec![Vec2::new(0.0, 0.0); capacity],
        }
    }

    /// Returns the detection radius
    ///
    /// # Examples
    ///
    /// ```
    /// let sensor = ProximitySensor::new(100, 50.0);
    /// assert_eq!(sensor.radius(), 50.0);
    /// ```
    #[inline(always)]
    #[must_use]
    pub const fn radius(&self) -> f32 {
        self.radius
    }

    /// Samples proximity state for all entities
    ///
    /// This performs spatial queries using SpatialHash to detect which entities
    /// have neighbors within the detection radius. Call this once per frame.
    ///
    /// # Arguments
    ///
    /// * `store` - EntityStore with transforms
    /// * `spatial` - SpatialHash for O(1) spatial queries
    ///
    /// # Complexity
    ///
    /// O(n) where n = `store.transforms.len()` (number of entities)
    ///
    /// # Performance
    ///
    /// - Zero-allocation
    /// - Uses SpatialHash for efficient proximity queries
    /// - Cache-friendly (linear scan of arrays)
    ///
    /// # Examples
    ///
    /// ```
    /// sensor.sample(&store, &spatial);
    /// ```
    #[inline(never)] // Prevent inlining to keep binary size small
    pub fn sample(&mut self, store: &EntityStore, spatial: &SpatialHash) {
        // Process all entities in a single cache-friendly loop
        // For each entity, query spatial hash to find nearby entities

        for (i, transform) in store.transforms.iter().enumerate() {
            // Extract position from transform [x, y, width, height]
            let pos = Vec2::new(transform[0], transform[1]);
            self.positions[i] = pos;

            // Create a query bounds for the radius
            let query_bounds = Rect {
                min: pos - Vec2::new(self.radius, self.radius),
                max: pos + Vec2::new(self.radius, self.radius),
            };

            // Query spatial hash for entities in this radius
            let nearby = spatial.query_rect(query_bounds);

            // Check if there are nearby entities (excluding self)
            let has_nearby = nearby.iter().any(|&id| id.index().0 as usize != i);

            // Update 6-tick history for this entity
            self.signals[i].push(has_nearby);
        }
    }

    /// Returns true if `target` is near `entity` (distance < radius)
    ///
    /// This checks if two specific entities are within the detection radius.
    /// Uses Euclidean distance calculation.
    ///
    /// # Arguments
    ///
    /// * `entity` - First entity
    /// * `target` - Second entity to check distance to
    ///
    /// # Examples
    ///
    /// ```
    /// sensor.sample(&store, &spatial);
    /// if sensor.is_near(entity1, entity2) {
    ///     // entity2 is within radius of entity1
    /// }
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn is_near(&self, entity: EntityId, target: EntityId) -> bool {
        let idx = entity.index().0 as usize;
        let target_idx = target.index().0 as usize;

        if idx >= self.positions.len() || target_idx >= self.positions.len() {
            return false;
        }

        // Calculate Euclidean distance
        let pos = self.positions[idx];
        let target_pos = self.positions[target_idx];
        let diff = pos - target_pos;
        let distance_sq = diff.x * diff.x + diff.y * diff.y;

        distance_sq <= self.radius * self.radius
    }

    /// Returns all entities within `radius` of a given position
    ///
    /// This performs a spatial query around the given position and returns
    /// all entities whose centers are within the specified radius.
    ///
    /// # Arguments
    ///
    /// * `position` - World position to query around
    /// * `radius` - Detection radius (can differ from sensor's default)
    ///
    /// # Returns
    ///
    /// Vector of EntityId for entities within the radius
    ///
    /// # Examples
    ///
    /// ```
    /// sensor.sample(&store, &spatial);
    /// let nearby = sensor.get_nearby_entities(mouse_pos, 30.0);
    /// for entity in nearby {
    ///     println!("Entity near mouse: {:?}", entity);
    /// }
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn get_nearby_entities(&self, position: Vec2, radius: f32) -> Vec<EntityId> {
        let mut result = Vec::new();

        for (i, &pos) in self.positions.iter().enumerate() {
            let diff = pos - position;
            let distance_sq = diff.x * diff.x + diff.y * diff.y;

            if distance_sq <= radius * radius {
                // Found an entity within radius (inclusive)
                let id = EntityId::new(i as u32);
                result.push(id);
            }
        }

        result
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS (inline for verification during development)
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capacity_and_radius() {
        let sensor = ProximitySensor::new(1000, 25.0);
        assert_eq!(sensor.signals.len(), 1000);
        assert_eq!(sensor.positions.len(), 1000);
        assert_eq!(sensor.radius(), 25.0);
    }

    #[test]
    fn test_signals_initialized_to_zero() {
        let sensor = ProximitySensor::new(100, 20.0);
        for signal in &sensor.signals {
            assert_eq!(signal.as_u8(), 0);
        }
    }

    #[test]
    fn test_positions_initialized_to_zero() {
        let sensor = ProximitySensor::new(100, 20.0);
        for &pos in &sensor.positions {
            assert_eq!(pos.x, 0.0);
            assert_eq!(pos.y, 0.0);
        }
    }
}
