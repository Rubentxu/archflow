// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Near Sensor with Hysteresis (HU-007)
//
// This sensor detects when entities are within a specified distance using
// Schmitt Trigger pattern to avoid flickering at boundaries.
//
// Reference: docs/epics/EPIC-002-physics-sensors.md - HU-007
//
// Performance Characteristics:
// - O(n) where n = number of entities (single scan per frame)
// - Uses SpatialHash for O(1) spatial queries
// - Distance squared comparisons (avoids sqrt)
// - Hysteresis prevents boundary flickering
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

/// Near Sensor with hysteresis (Schmitt Trigger)
///
/// This sensor tracks proximity using two thresholds:
/// - `distance`: Trigger threshold (enter active state)
/// - `reset_distance`: Reset threshold (exit active state, must be > distance)
///
/// The hysteresis gap prevents flickering when entities hover near the boundary.
///
/// # Schmitt Trigger Pattern
///
/// ```text
/// Active
///    ↑
///    │              ┌─────────────────
///    │              │
///    │    ┌─────────┘
///    │    │
///    └────┴────────────────────────────→
///         D   RD
///
/// D = distance (trigger point)
/// RD = reset_distance (clear point, RD > D)
/// ```
///
/// # Examples
///
/// ```
/// use archflow_logic::sensors::ProximitySensor;
/// use archflow_core::Vec2;
/// use archflow_engine::{EntityStore, SpatialHash};
///
/// let mut store = EntityStore::new();
/// let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
/// let entity2 = store.spawn(Vec2::new(110.0, 100.0), Vec2::new(50.0, 50.0));
///
/// // Create spatial hash and insert entities
/// let mut spatial = SpatialHash::new(MAX_ENTITIES);
/// // ... insert entities ...
///
/// let mut sensor = ProximitySensor::with_hysteresis(MAX_ENTITIES, 20.0, 25.0, 0);
/// sensor.evaluate(&store, &spatial);
///
/// // Check if entity2 is near entity1 using SignalByte edge detection
/// let signal = sensor.signal(entity1);
/// if signal.is_rising_edge() {
///     // Entity just entered proximity - activate
/// }
/// ```
///
/// # Performance
///
/// - **Time**: O(n) single scan per `evaluate()` call
/// - **Space**: 1 byte per entity (SignalByte)
/// - **Allocations**: Zero (pre-allocated on construction)
pub struct ProximitySensor {
    /// Signal history for each entity
    ///
    /// Each SignalByte stores 6 ticks of "has_nearby_neighbors" state:
    /// - bit 0 (T0): current frame
    /// - bits 1-5 (T1-T5): previous 5 frames
    signals: Vec<SignalByte>,

    /// Detection distance (trigger threshold - entities within this distance are "near")
    distance: f32,

    /// Reset distance (clear threshold - must be > distance for hysteresis)
    ///
    /// Once triggered, proximity state persists until entities move beyond
    /// this greater distance, preventing boundary flickering.
    reset_distance: f32,

    /// Target tag filter (0 = match all entities)
    ///
    /// Only entities with this tag will be detected. Set to 0 to detect all entities.
    target_tag: u8,

    /// Squared distances for comparisons (precomputed to avoid sqrt)
    distance_sq: f32,
    reset_distance_sq: f32,
}

impl ProximitySensor {
    /// Creates a new Near Sensor with default settings (no hysteresis)
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of entities to track
    /// * `distance` - Detection radius in world units (pixels)
    ///
    /// # Examples
    ///
    /// ```
    /// let sensor = ProximitySensor::new(MAX_ENTITIES, 20.0);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn new(capacity: usize, distance: f32) -> Self {
        Self::with_hysteresis(capacity, distance, distance, 0)
    }

    /// Creates a new Near Sensor with hysteresis (Schmitt Trigger)
    ///
    /// This is the recommended constructor for production use as it prevents
    /// flickering at proximity boundaries.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of entities to track
    /// * `distance` - Trigger threshold (entities within this distance trigger)
    /// * `reset_distance` - Reset threshold (state clears beyond this distance)
    /// * `target_tag` - Optional tag filter (0 = match all)
    ///
    /// # Hysteresis
    ///
    /// For smooth behavior, `reset_distance` should typically be 20-30% larger
    /// than `distance`:
    ///
    /// ```text
    /// distance = 20px
    /// reset_distance = 25px (25% hysteresis gap)
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// // 20px detection, 25% hysteresis gap
    /// let sensor = ProximitySensor::with_hysteresis(MAX_ENTITIES, 20.0, 25.0, 0);
    ///
    /// // Tag-filtered: only detect entities with tag=5
    /// let sensor = ProximitySensor::with_hysteresis(MAX_ENTITIES, 20.0, 25.0, 5);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn with_hysteresis(
        capacity: usize,
        distance: f32,
        reset_distance: f32,
        target_tag: u8,
    ) -> Self {
        assert!(
            reset_distance >= distance,
            "reset_distance must be >= distance"
        );

        Self {
            signals: vec![SignalByte::default(); capacity],
            distance_sq: distance * distance,
            reset_distance_sq: reset_distance * reset_distance,
            distance,
            reset_distance,
            target_tag,
        }
    }

    /// Returns the detection distance (trigger threshold)
    #[inline(always)]
    #[must_use]
    pub const fn distance(&self) -> f32 {
        self.distance
    }

    /// Returns the reset distance (clear threshold)
    #[inline(always)]
    #[must_use]
    pub const fn reset_distance(&self) -> f32 {
        self.reset_distance
    }

    /// Returns the target tag filter (0 = match all)
    #[inline(always)]
    #[must_use]
    pub const fn target_tag(&self) -> u8 {
        self.target_tag
    }

    /// Get the signal for a specific entity
    ///
    /// Returns the SignalByte which provides edge detection methods:
    /// - `is_rising_edge()` - entity just entered proximity
    /// - `is_falling_edge()` - entity just exited proximity
    /// - `is_high()` - entity currently in proximity
    ///
    /// # Arguments
    ///
    /// * `entity` - Entity ID to query
    ///
    /// # Examples
    ///
    /// ```
    /// sensor.evaluate(&store, &spatial);
    /// let signal = sensor.signal(entity_id);
    ///
    /// if signal.is_rising_edge() {
    ///     // Entity just entered proximity
    /// } else if signal.is_falling_edge() {
    ///     // Entity just exited proximity
    /// }
    /// ```
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

    /// Evaluate proximity for all entities using SpatialHash
    ///
    /// This performs spatial queries and updates SignalByte states with hysteresis.
    /// Call this once per frame before checking signals.
    ///
    /// # Algorithm
    ///
    /// For each entity:
    /// 1. Query SpatialHash for entities within `reset_distance` (max threshold)
    /// 2. Filter by `target_tag` if set
    /// 3. Calculate squared distance to each candidate (no sqrt)
    /// 4. Apply Schmitt Trigger logic:
    ///    - If currently inactive: trigger if dist < distance_sq
    ///    - If currently active: stay active until dist > reset_distance_sq
    ///
    /// # Arguments
    ///
    /// * `store` - EntityStore with transforms and metadata
    /// * `spatial` - SpatialHash for O(1) spatial queries
    ///
    /// # Complexity
    ///
    /// O(n) where n = `store.transforms.len()` (number of entities)
    ///
    /// # Performance
    ///
    /// - Zero-allocation
    /// - Uses squared distances (avoids sqrt)
    /// - Cache-friendly (linear scan of arrays)
    /// - Hysteresis prevents rapid state changes
    ///
    /// # Examples
    ///
    /// ```
    /// // In game loop
    /// sensor.evaluate(&store, &spatial);
    ///
    /// // Check for rising/falling edges
    /// for (id, transform) in store.transforms.iter().enumerate() {
    ///     let entity_id = EntityId::new(id as u32);
    ///     let signal = sensor.signal(entity_id);
    ///
    ///     if signal.is_rising_edge() {
    ///         // Spawn proximity effect
    ///     } else if signal.is_falling_edge() {
    ///         // Remove proximity effect
    ///     }
    /// }
    /// ```
    #[inline(never)] // Prevent inlining to keep binary size small
    pub fn evaluate(&mut self, store: &EntityStore, spatial: &SpatialHash) {
        // Process all entities in a single cache-friendly loop
        for (idx, transform) in store.transforms.iter().enumerate() {
            // Skip if index exceeds sensor capacity
            if idx >= self.signals.len() {
                break;
            }

            // Extract position from transform [x, y, width, height]
            let pos = Vec2::new(transform[0], transform[1]);

            // Create query bounds using reset_distance (larger threshold)
            // This ensures we don't miss entities that should keep state active
            let query_bounds = Rect {
                min: pos - Vec2::new(self.reset_distance, self.reset_distance),
                max: pos + Vec2::new(self.reset_distance, self.reset_distance),
            };

            // Query spatial hash for candidate entities
            let candidates = spatial.query_rect(query_bounds);

            // Check current state for hysteresis
            let current_signal = self.signals[idx];
            let is_currently_active = current_signal.get_current();

            // Determine threshold based on current state (Schmitt Trigger)
            let threshold_sq = if is_currently_active {
                self.reset_distance_sq // Use larger threshold to maintain active state
            } else {
                self.distance_sq // Use smaller threshold to trigger
            };

            // Find closest candidate within threshold
            let has_nearby = candidates.iter().any(|&candidate_id| {
                // Skip self
                if candidate_id.index().0 as usize == idx {
                    return false;
                }

                // Filter by target tag if set
                if self.target_tag != 0 {
                    let candidate_idx = candidate_id.index().0 as usize;
                    if candidate_idx < store.metadata.len() {
                        // Extract tag from metadata bits 16-23
                        let entity_tag = (store.metadata[candidate_idx] >> 16) & 0xFF;
                        if entity_tag as u8 != self.target_tag {
                            return false;
                        }
                    }
                }

                // Calculate squared distance (avoids sqrt)
                let candidate_idx = candidate_id.index().0 as usize;
                if candidate_idx < store.transforms.len() {
                    let candidate_pos = Vec2::new(
                        store.transforms[candidate_idx][0],
                        store.transforms[candidate_idx][1],
                    );
                    let diff = candidate_pos - pos;
                    let dist_sq = diff.x * diff.x + diff.y * diff.y;
                    dist_sq <= threshold_sq
                } else {
                    false
                }
            });

            // Update 6-tick history for this entity
            self.signals[idx].push(has_nearby);
        }
    }

    /// Check if two specific entities are within detection range
    ///
    /// This is a direct distance check (no hysteresis, no SignalByte update).
    /// Useful for one-off queries.
    ///
    /// # Arguments
    ///
    /// * `entity` - First entity
    /// * `target` - Second entity to check distance to
    ///
    /// # Returns
    ///
    /// `true` if distance < `self.distance`
    ///
    /// # Examples
    ///
    /// ```
    /// if sensor.is_near(entity1, entity2) {
    ///     // entity2 is within detection range of entity1
    /// }
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn is_near(&self, entity: EntityId, target: EntityId, store: &EntityStore) -> bool {
        let idx = entity.index().0 as usize;
        let target_idx = target.index().0 as usize;

        if idx >= store.transforms.len() || target_idx >= store.transforms.len() {
            return false;
        }

        // Calculate squared distance
        let pos = Vec2::new(store.transforms[idx][0], store.transforms[idx][1]);
        let target_pos = Vec2::new(
            store.transforms[target_idx][0],
            store.transforms[target_idx][1],
        );
        let diff = target_pos - pos;
        let dist_sq = diff.x * diff.x + diff.y * diff.y;

        dist_sq <= self.distance_sq
    }

    /// Get all entities within a given radius of a position
    ///
    /// This is a utility spatial query (does not affect SignalByte state).
    ///
    /// # Arguments
    ///
    /// * `position` - World position to query around
    /// * `radius` - Detection radius (can differ from sensor's default)
    /// * `spatial` - SpatialHash for spatial query
    ///
    /// # Returns
    ///
    /// Vector of EntityId for entities within the radius
    ///
    /// # Examples
    ///
    /// ```
    /// let nearby = sensor.get_nearby_entities(mouse_pos, 30.0, &spatial);
    /// for entity in nearby {
    ///     println!("Entity near mouse: {:?}", entity);
    /// }
    /// ```
    #[inline(always)]
    pub fn get_nearby_entities(
        &self,
        position: Vec2,
        radius: f32,
        spatial: &SpatialHash,
    ) -> Vec<EntityId> {
        let mut result = Vec::new();

        let query_bounds = Rect {
            min: position - Vec2::new(radius, radius),
            max: position + Vec2::new(radius, radius),
        };

        let candidates = spatial.query_rect(query_bounds);

        for &candidate_id in &candidates {
            result.push(candidate_id);
        }

        result
    }

    /// Reset the sensor state for a specific entity
    ///
    /// Called when an entity is destroyed to clean up sensor state.
    /// This prevents stale signals from destroyed entities.
    ///
    /// # Arguments
    ///
    /// * `entity_idx` - Index of the entity to reset
    ///
    /// # Examples
    ///
    /// ```
    /// // When entity is destroyed
    /// sensor.reset_entity(entity_idx);
    /// ```
    #[inline(always)]
    pub fn reset_entity(&mut self, entity_idx: usize) {
        if entity_idx < self.signals.len() {
            self.signals[entity_idx] = SignalByte::default();
        }
    }
}

impl Default for ProximitySensor {
    fn default() -> Self {
        Self::new(100_000, 20.0)
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
    fn test_new_without_hysteresis() {
        let sensor = ProximitySensor::new(100, 20.0);
        assert_eq!(sensor.distance(), 20.0);
        assert_eq!(sensor.reset_distance(), 20.0);
        assert_eq!(sensor.target_tag(), 0);
    }

    #[test]
    fn test_with_hysteresis() {
        let sensor = ProximitySensor::with_hysteresis(100, 20.0, 25.0, 5);
        assert_eq!(sensor.distance(), 20.0);
        assert_eq!(sensor.reset_distance(), 25.0);
        assert_eq!(sensor.target_tag(), 5);
    }

    #[test]
    fn test_hysteresis_validation() {
        // Valid: reset_distance >= distance
        let sensor = ProximitySensor::with_hysteresis(100, 20.0, 25.0, 0);
        assert_eq!(sensor.reset_distance(), 25.0);

        // Valid: reset_distance == distance (no hysteresis)
        let sensor = ProximitySensor::with_hysteresis(100, 20.0, 20.0, 0);
        assert_eq!(sensor.reset_distance(), 20.0);
    }

    #[test]
    fn test_signals_initialized() {
        let sensor = ProximitySensor::new(100, 20.0);
        assert_eq!(sensor.signals.len(), 100);
        for signal in &sensor.signals {
            assert_eq!(signal.as_u8(), 0);
        }
    }

    #[test]
    fn test_signal_method() {
        let sensor = ProximitySensor::new(100, 20.0);
        let id = make_id(5);

        // Initial signal should be low (no proximity yet)
        let signal = sensor.signal(id);
        assert!(!signal.get_current());
        assert!(!signal.is_rising_edge());
        assert!(!signal.is_falling_edge());
    }

    #[test]
    fn test_default() {
        let sensor = ProximitySensor::default();
        assert_eq!(sensor.distance(), 20.0);
        assert_eq!(sensor.reset_distance(), 20.0);
        assert_eq!(sensor.target_tag(), 0);
    }

    #[test]
    fn test_squared_distance_precomputed() {
        let sensor = ProximitySensor::with_hysteresis(100, 20.0, 25.0, 0);
        // distance_sq = 20^2 = 400
        assert_eq!(sensor.distance_sq, 400.0);
        // reset_distance_sq = 25^2 = 625
        assert_eq!(sensor.reset_distance_sq, 625.0);
    }

    #[test]
    fn test_is_near_direct_check() {
        let mut store = EntityStore::new();

        // Spawn two entities 15 units apart
        let id1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let id2 = store.spawn(Vec2::new(15.0, 0.0), Vec2::new(10.0, 10.0));

        // Sensor with 20px detection radius
        let sensor = ProximitySensor::new(100, 20.0);

        // Should be near (15 < 20)
        assert!(sensor.is_near(id1, id2, &store));
        assert!(sensor.is_near(id2, id1, &store));
    }

    #[test]
    fn test_is_not_near() {
        let mut store = EntityStore::new();

        // Spawn two entities 30 units apart
        let id1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let id2 = store.spawn(Vec2::new(30.0, 0.0), Vec2::new(10.0, 10.0));

        // Sensor with 20px detection radius
        let sensor = ProximitySensor::new(100, 20.0);

        // Should NOT be near (30 > 20)
        assert!(!sensor.is_near(id1, id2, &store));
        assert!(!sensor.is_near(id2, id1, &store));
    }

    #[test]
    fn test_diagonal_distance() {
        let mut store = EntityStore::new();

        // Spawn entities at (0,0) and (14,14)
        // Distance = sqrt(14^2 + 14^2) ≈ 19.8
        let id1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let id2 = store.spawn(Vec2::new(14.0, 14.0), Vec2::new(10.0, 10.0));

        // Sensor with 20px detection radius
        let sensor = ProximitySensor::new(100, 20.0);

        // Should be near (19.8 < 20)
        assert!(sensor.is_near(id1, id2, &store));
    }

    #[test]
    fn test_evaluate_updates_signals() {
        let mut store = EntityStore::new();
        let mut spatial = SpatialHash::new(100);

        // Create entities with known indices
        let _idx1 = 0;
        let _idx2 = 1;

        let id1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let id2 = store.spawn(Vec2::new(15.0, 0.0), Vec2::new(10.0, 10.0));

        // Insert into spatial hash
        let bounds1 = Rect::from_origin_size(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let bounds2 = Rect::from_origin_size(Vec2::new(15.0, 0.0), Vec2::new(10.0, 10.0));
        spatial.insert(id1, bounds1);
        spatial.insert(id2, bounds2);

        let mut sensor = ProximitySensor::new(100, 20.0);

        // First evaluation - entities should become "near"
        sensor.evaluate(&store, &spatial);

        // Check signal using actual entity index from store
        let actual_idx1 = id1.index().0 as usize;
        if actual_idx1 < sensor.signals.len() {
            let signal1 = sensor.signals[actual_idx1];
            assert!(signal1.get_current()); // Entity 1 is near entity 2
        }
    }

    #[test]
    fn test_evaluate_detects_rising_edge() {
        let mut store = EntityStore::new();
        let mut spatial = SpatialHash::new(100);

        // Spawn two entities
        let id1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let id2 = store.spawn(Vec2::new(15.0, 0.0), Vec2::new(10.0, 10.0));

        spatial.insert(
            id1,
            Rect::from_origin_size(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0)),
        );
        spatial.insert(
            id2,
            Rect::from_origin_size(Vec2::new(15.0, 0.0), Vec2::new(10.0, 10.0)),
        );

        let mut sensor = ProximitySensor::new(100, 20.0);

        // First frame: detect rising edge
        sensor.evaluate(&store, &spatial);

        let actual_idx1 = id1.index().0 as usize;
        if actual_idx1 < sensor.signals.len() {
            let signal1 = sensor.signals[actual_idx1];
            assert!(signal1.is_rising_edge()); // Just entered proximity
        }
    }

    #[test]
    fn test_evaluate_no_proximity() {
        let mut store = EntityStore::new();
        let mut spatial = SpatialHash::new(100);

        // Spawn two entities far apart
        let id1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let id2 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(10.0, 10.0));

        spatial.insert(
            id1,
            Rect::from_origin_size(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0)),
        );
        spatial.insert(
            id2,
            Rect::from_origin_size(Vec2::new(100.0, 100.0), Vec2::new(10.0, 10.0)),
        );

        let mut sensor = ProximitySensor::new(100, 20.0);

        sensor.evaluate(&store, &spatial);

        // Entities should not be near each other
        let actual_idx1 = id1.index().0 as usize;
        if actual_idx1 < sensor.signals.len() {
            let signal1 = sensor.signals[actual_idx1];
            assert!(!signal1.get_current());
        }
    }
}
