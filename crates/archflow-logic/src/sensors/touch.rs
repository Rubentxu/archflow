// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Touch Sensor (Collision Detection) - HU-006
//
// This sensor detects AABB vs AABB collisions using SpatialHash for broad-phase
// and precise AABB testing for narrow-phase collision detection.
//
// Reference: docs/epics/EPIC-002-physics-sensors.md - HU-006
//
// Performance Characteristics:
// - O(n) where n = number of entities (single scan per frame)
// - SpatialHash broad-phase filters candidates
// - AABB narrow-phase for exact collision
// - Hit list tracking for enter/exit detection
//
// Memory Impact:
// - 1 byte per entity (SignalByte for collision state)
// - Additional hit list storage (dynamic, proportional to active collisions)
//
// ═══════════════════════════════════════════════════════════════════════════════

use crate::signals::SignalByte;
use alloc::collections::BTreeSet;
use alloc::vec;
use alloc::vec::Vec;
use archflow_core::{EntityId, Rect, Vec2};
use archflow_engine::{EntityStore, SpatialHash};

/// Touch Sensor for collision detection
///
/// This sensor detects when entities collide (AABB vs AABB) and maintains
/// a hit list of currently colliding entities for enter/exit detection.
///
/// # Examples
///
/// ```
/// use archflow_logic::sensors::touch::TouchSensor;
/// use archflow_core::Vec2;
/// use archflow_engine::{EntityStore, SpatialHash};
///
/// let mut store = EntityStore::new();
/// let entity1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));
/// let entity2 = store.spawn(Vec2::new(40.0, 0.0), Vec2::new(50.0, 50.0));
///
/// let mut spatial = SpatialHash::new(MAX_ENTITIES);
/// // ... insert entities into spatial hash ...
///
/// let mut sensor = TouchSensor::new(MAX_ENTITIES, 0);
/// sensor.evaluate(&store, &spatial);
///
/// let signal = sensor.signal(entity1);
/// if signal.is_rising_edge() {
///     // Entity1 just started colliding
/// }
/// ```
///
/// # Performance
///
/// - **Time**: O(n) single scan per `evaluate()` call
/// - **Space**: 1 byte per entity + hit list storage
/// - **Allocations**: Minimal (hit list grows/shrinks with active collisions)
pub struct TouchSensor {
    /// Signal history for each entity
    ///
    /// Each SignalByte stores 6 ticks of "is_colliding" state:
    /// - bit 0 (T0): current frame
    /// - bits 1-5 (T1-T5): previous 5 frames
    signals: Vec<SignalByte>,

    /// Hit list tracking currently colliding entities per entity
    ///
    /// For each entity, we maintain a set of entities it's currently colliding with.
    /// This allows us to detect enter (new collision) and exit (ended collision) events.
    hit_lists: Vec<BTreeSet<EntityId>>,

    /// Target tag filter (0 = match all entities)
    ///
    /// Only entities with this tag will trigger collision detection.
    /// Set to 0 to detect collisions with all entities.
    target_tag: u8,

    /// Invert output (BGE property)
    ///
    /// When true, the signal is inverted (collision = OFF, no collision = ON)
    invert: bool,
}

impl TouchSensor {
    /// Creates a new Touch Sensor
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of entities to track
    /// * `target_tag` - Optional tag filter (0 = match all entities)
    ///
    /// # Examples
    ///
    /// ```
    /// let sensor = TouchSensor::new(MAX_ENTITIES, 0);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn new(capacity: usize, target_tag: u8) -> Self {
        Self {
            signals: vec![SignalByte::default(); capacity],
            hit_lists: vec![BTreeSet::new(); capacity],
            target_tag,
            invert: false,
        }
    }

    /// Returns the target tag filter
    #[inline(always)]
    #[must_use]
    pub const fn target_tag(&self) -> u8 {
        self.target_tag
    }

    /// Returns whether output is inverted
    #[inline(always)]
    #[must_use]
    pub const fn invert(&self) -> bool {
        self.invert
    }

    /// Set the invert property
    ///
    /// When true, collision detection output is inverted.
    pub fn set_invert(&mut self, invert: bool) {
        self.invert = invert;
    }

    /// Get the signal for a specific entity
    ///
    /// Returns the SignalByte which provides edge detection methods:
    /// - `is_rising_edge()` - entity just started colliding
    /// - `is_falling_edge()` - entity just stopped colliding
    /// - `get_current()` - entity currently colliding
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
    ///     // Entity just entered collision
    /// } else if signal.is_falling_edge() {
    ///     // Entity just exited collision
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

    /// Get the current hit list for an entity
    ///
    /// Returns the set of entities that the given entity is currently colliding with.
    ///
    /// # Arguments
    ///
    /// * `entity` - Entity ID to query
    ///
    /// # Returns
    ///
    /// Set of EntityId for entities in collision with the given entity
    ///
    /// # Examples
    ///
    /// ```
    /// sensor.evaluate(&store, &spatial);
    /// let hits = sensor.hit_list(entity_id);
    /// for &hit_entity in &hits {
    ///     println!("Colliding with: {:?}", hit_entity);
    /// }
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn hit_list(&self, entity: EntityId) -> &BTreeSet<EntityId> {
        let idx = entity.index().0 as usize;
        if idx < self.hit_lists.len() {
            &self.hit_lists[idx]
        } else {
            // Return empty set reference for out-of-bounds
            static EMPTY_SET: BTreeSet<EntityId> = BTreeSet::new();
            &EMPTY_SET
        }
    }

    /// Check if two entities are currently colliding
    ///
    /// This is a direct check (does not affect SignalByte state).
    ///
    /// # Arguments
    ///
    /// * `entity` - First entity
    /// * `target` - Second entity to check collision with
    ///
    /// # Returns
    ///
    /// `true` if entities are colliding
    ///
    /// # Examples
    ///
    /// ```
    /// sensor.evaluate(&store, &spatial);
    /// if sensor.is_colliding(entity1, entity2) {
    ///     // entity1 and entity2 are colliding
    /// }
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn is_colliding(&self, entity: EntityId, target: EntityId) -> bool {
        let idx = entity.index().0 as usize;
        if idx < self.hit_lists.len() {
            self.hit_lists[idx].contains(&target)
        } else {
            false
        }
    }

    /// Evaluate collision detection for all entities
    ///
    /// This performs broad-phase SpatialHash queries followed by narrow-phase
    /// AABB testing, updating SignalByte states with enter/exit detection.
    ///
    /// # Algorithm
    ///
    /// For each entity:
    /// 1. Query SpatialHash for potential colliders (broad-phase)
    /// 2. Filter by `target_tag` if set
    /// 3. Test AABB intersection for each candidate (narrow-phase)
    /// 4. Update hit list and detect enter/exit events
    /// 5. Apply `invert` property if set
    ///
    /// # Arguments
    ///
    /// * `store` - EntityStore with transforms and metadata
    /// * `spatial` - SpatialHash for O(1) spatial queries
    ///
    /// # Complexity
    ///
    /// O(n × k) where n = entities, k = average colliders per entity
    ///
    /// # Performance
    ///
    /// - Zero-allocation in signal updates
    /// - SpatialHash reduces candidate checks significantly
    /// - AABB testing is simple floating-point comparison
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
    ///         // Spawn collision effect
    ///     } else if signal.is_falling_edge() {
    ///         // Remove collision effect
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

            // Extract AABB from transform [x, y, width, height]
            let pos = Vec2::new(transform[0], transform[1]);
            let size = Vec2::new(transform[2], transform[3]);
            let entity_aabb = Rect {
                min: pos,
                max: pos + size,
            };

            // Query spatial hash for potential colliders (broad-phase)
            let candidates = spatial.query_rect(entity_aabb);

            // Track previous hit list for exit detection
            let previous_hits = core::mem::take(&mut self.hit_lists[idx]);

            // Test each candidate with AABB (narrow-phase)
            let mut current_hits = BTreeSet::new();
            for &candidate_id in &candidates {
                // Skip self
                let candidate_idx = candidate_id.index().0 as usize;
                if candidate_idx == idx {
                    continue;
                }

                // Filter by target tag if set
                if self.target_tag != 0 {
                    if candidate_idx < store.metadata.len() {
                        // Extract tag from metadata bits 16-23
                        let entity_tag = (store.metadata[candidate_idx] >> 16) & 0xFF;
                        if entity_tag as u8 != self.target_tag {
                            continue;
                        }
                    }
                }

                // AABB intersection test (narrow-phase)
                if self.test_aabb_collision(&entity_aabb, candidate_idx, store) {
                    current_hits.insert(candidate_id);
                }
            }

            // Detect enter/exit events
            let has_collision = !current_hits.is_empty();

            // Apply invert property
            let signal_value = if self.invert {
                !has_collision
            } else {
                has_collision
            };
            self.signals[idx].push(signal_value);

            // Store new hit list
            self.hit_lists[idx] = current_hits;
        }
    }

    /// Test AABB collision between entity and candidate
    #[inline(always)]
    fn test_aabb_collision(
        &self,
        entity_aabb: &Rect,
        candidate_idx: usize,
        store: &EntityStore,
    ) -> bool {
        if candidate_idx >= store.transforms.len() {
            return false;
        }

        let candidate_transform = store.transforms[candidate_idx];
        let candidate_pos = Vec2::new(candidate_transform[0], candidate_transform[1]);
        let candidate_size = Vec2::new(candidate_transform[2], candidate_transform[3]);

        let candidate_aabb = Rect {
            min: candidate_pos,
            max: candidate_pos + candidate_size,
        };

        entity_aabb.intersects(&candidate_aabb)
    }
}

impl Default for TouchSensor {
    fn default() -> Self {
        Self::new(100_000, 0)
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
    fn test_new_touch_sensor() {
        let sensor = TouchSensor::new(100, 5);
        assert_eq!(sensor.target_tag(), 5);
        assert!(!sensor.invert());
        assert_eq!(sensor.signals.len(), 100);
        assert_eq!(sensor.hit_lists.len(), 100);
    }

    #[test]
    fn test_default() {
        let sensor = TouchSensor::default();
        assert_eq!(sensor.target_tag(), 0);
        assert!(!sensor.invert());
    }

    #[test]
    fn test_set_invert() {
        let mut sensor = TouchSensor::new(100, 0);
        assert!(!sensor.invert());

        sensor.set_invert(true);
        assert!(sensor.invert());

        sensor.set_invert(false);
        assert!(!sensor.invert());
    }

    #[test]
    fn test_signal_method() {
        let sensor = TouchSensor::new(100, 0);
        let id = make_id(5);

        // Initial signal should be low (no collision yet)
        let signal = sensor.signal(id);
        assert!(!signal.get_current());
        assert!(!signal.is_rising_edge());
        assert!(!signal.is_falling_edge());
    }

    #[test]
    fn test_hit_list_empty_initially() {
        let sensor = TouchSensor::new(100, 0);
        let id = make_id(5);

        let hits = sensor.hit_list(id);
        assert!(hits.is_empty());
    }

    #[test]
    fn test_is_colliding_false_initially() {
        let sensor = TouchSensor::new(100, 0);
        let id1 = make_id(1);
        let id2 = make_id(2);

        assert!(!sensor.is_colliding(id1, id2));
    }

    #[test]
    fn test_signals_initialized_to_zero() {
        let sensor = TouchSensor::new(100, 0);
        for signal in &sensor.signals {
            assert_eq!(signal.as_u8(), 0);
        }
    }

    #[test]
    fn test_hit_lists_initialized_empty() {
        let sensor = TouchSensor::new(100, 0);
        for hit_list in &sensor.hit_lists {
            assert!(hit_list.is_empty());
        }
    }

    #[test]
    fn test_evaluate_detects_collision() {
        let mut store = EntityStore::new();
        let mut spatial = SpatialHash::new(100);

        // Spawn two overlapping entities
        let id1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));
        let id2 = store.spawn(Vec2::new(25.0, 0.0), Vec2::new(50.0, 50.0)); // Overlaps by 25px

        // Insert into spatial hash
        let bounds1 = Rect::from_origin_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));
        let bounds2 = Rect::from_origin_size(Vec2::new(25.0, 0.0), Vec2::new(50.0, 50.0));
        spatial.insert(id1, bounds1);
        spatial.insert(id2, bounds2);

        let mut sensor = TouchSensor::new(100, 0);

        // First evaluation - should detect collision
        sensor.evaluate(&store, &spatial);

        // Check collision signal
        let actual_idx1 = id1.index().0 as usize;
        if actual_idx1 < sensor.signals.len() {
            let signal1 = sensor.signals[actual_idx1];
            assert!(signal1.get_current()); // Entity 1 is colliding
            assert!(signal1.is_rising_edge()); // Just entered collision
        }
    }

    #[test]
    fn test_evaluate_no_collision() {
        let mut store = EntityStore::new();
        let mut spatial = SpatialHash::new(100);

        // Spawn two non-overlapping entities
        let id1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));
        let id2 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0)); // No overlap

        spatial.insert(
            id1,
            Rect::from_origin_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0)),
        );
        spatial.insert(
            id2,
            Rect::from_origin_size(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0)),
        );

        let mut sensor = TouchSensor::new(100, 0);

        sensor.evaluate(&store, &spatial);

        // Should NOT be colliding
        let actual_idx1 = id1.index().0 as usize;
        if actual_idx1 < sensor.signals.len() {
            let signal1 = sensor.signals[actual_idx1];
            assert!(!signal1.get_current());
        }
    }

    #[test]
    fn test_hit_list_contains_colliding_entity() {
        let mut store = EntityStore::new();
        let mut spatial = SpatialHash::new(100);

        let id1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));
        let id2 = store.spawn(Vec2::new(25.0, 0.0), Vec2::new(50.0, 50.0));

        spatial.insert(
            id1,
            Rect::from_origin_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0)),
        );
        spatial.insert(
            id2,
            Rect::from_origin_size(Vec2::new(25.0, 0.0), Vec2::new(50.0, 50.0)),
        );

        let mut sensor = TouchSensor::new(100, 0);
        sensor.evaluate(&store, &spatial);

        let hits = sensor.hit_list(id1);
        assert!(hits.contains(&id2));
    }

    #[test]
    fn test_is_colliding_returns_true() {
        let mut store = EntityStore::new();
        let mut spatial = SpatialHash::new(100);

        let id1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));
        let id2 = store.spawn(Vec2::new(25.0, 0.0), Vec2::new(50.0, 50.0));

        spatial.insert(
            id1,
            Rect::from_origin_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0)),
        );
        spatial.insert(
            id2,
            Rect::from_origin_size(Vec2::new(25.0, 0.0), Vec2::new(50.0, 50.0)),
        );

        let mut sensor = TouchSensor::new(100, 0);
        sensor.evaluate(&store, &spatial);

        assert!(sensor.is_colliding(id1, id2));
        assert!(sensor.is_colliding(id2, id1));
    }

    #[test]
    fn test_invert_property() {
        let mut store = EntityStore::new();
        let mut spatial = SpatialHash::new(100);

        let id1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));
        let id2 = store.spawn(Vec2::new(25.0, 0.0), Vec2::new(50.0, 50.0));

        spatial.insert(
            id1,
            Rect::from_origin_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0)),
        );
        spatial.insert(
            id2,
            Rect::from_origin_size(Vec2::new(25.0, 0.0), Vec2::new(50.0, 50.0)),
        );

        let mut sensor = TouchSensor::new(100, 0);
        sensor.set_invert(true);

        sensor.evaluate(&store, &spatial);

        // With invert=true, collision should produce false signal
        let actual_idx1 = id1.index().0 as usize;
        if actual_idx1 < sensor.signals.len() {
            let signal1 = sensor.signals[actual_idx1];
            assert!(!signal1.get_current()); // Inverted: collision = false
        }
    }
}
