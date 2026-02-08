// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Collision Sensor Tests (TDD - Red Phase)
//
// These tests define the expected behavior of the CollisionSensor.
//
// Reference: docs/epics/EPIC-002-physics-sensors.md - HU-006
// ═══════════════════════════════════════════════════════════════════════════════

#![cfg(test)]

use archflow_core::{EntityId, Generation, Index, Rect, Vec2};
use archflow_engine::{EntityStore, MAX_ENTITIES, SpatialHash};

use archflow_logic::sensors::collision::CollisionSensor;

// Helper to create EntityId
fn make_id(idx: u32) -> EntityId {
    EntityId::from_parts(Index(idx), Generation(1))
}

/// Helper to get AABB from entity using correct API
#[inline(always)]
fn entity_aabb(store: &EntityStore, entity: EntityId) -> Rect {
    let idx = entity.index().0 as usize;
    // transforms[idx] = [x, y, width, height]
    let x = store.transforms[idx][0];
    let y = store.transforms[idx][1];
    let w = store.transforms[idx][2];
    let h = store.transforms[idx][3];
    Rect::new(x, y, x + w, y + h)
}

#[test]
fn test_collision_sensor_creation() {
    let sensor = CollisionSensor::new(MAX_ENTITIES);
    // Verify sensor was created with a valid ID
    assert!(sensor.sensor_id() > 0);
}

#[test]
fn test_collision_sensor_signals_initialized() {
    let sensor = CollisionSensor::new(MAX_ENTITIES);
    // Verify initial signal state by checking is_colliding returns false
    let entity = make_id(0);
    assert!(!sensor.is_colliding(entity));
}

#[test]
fn test_non_colliding_entities() {
    let mut store = EntityStore::new();
    let entity1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));
    let entity2 = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0));

    let mut spatial = SpatialHash::new(MAX_ENTITIES);
    spatial.insert(entity1, entity_aabb(&store, entity1));
    spatial.insert(entity2, entity_aabb(&store, entity2));

    let mut sensor = CollisionSensor::new(MAX_ENTITIES);
    sensor.evaluate(entity1, &store, &spatial);

    // No collision should be detected
    assert!(!sensor.is_colliding(entity1));
}

#[test]
fn test_colliding_entities() {
    let mut store = EntityStore::new();
    let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
    let entity2 = store.spawn(Vec2::new(110.0, 110.0), Vec2::new(50.0, 50.0));

    let mut spatial = SpatialHash::new(MAX_ENTITIES);
    spatial.insert(entity1, entity_aabb(&store, entity1));
    spatial.insert(entity2, entity_aabb(&store, entity2));

    let mut sensor = CollisionSensor::new(MAX_ENTITIES);
    sensor.evaluate(entity1, &store, &spatial);

    // These entities should be colliding
    assert!(sensor.is_colliding(entity1));
}

#[test]
fn test_edge_detection_rising_edge() {
    let mut store = EntityStore::new();
    let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
    let entity2 = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0));

    let mut spatial = SpatialHash::new(MAX_ENTITIES);
    spatial.insert(entity1, entity_aabb(&store, entity1));
    spatial.insert(entity2, entity_aabb(&store, entity2));

    let mut sensor = CollisionSensor::new(MAX_ENTITIES);

    // First frame: no collision
    sensor.evaluate(entity1, &store, &spatial);
    assert!(!sensor.is_colliding(entity1));

    // Move entity2 into collision range (simulate by removing and re-inserting)
    spatial.remove(entity2);
    store.transforms[1][0] = 110.0; // Move entity2 to (110, 110)
    store.transforms[1][1] = 110.0;
    spatial.insert(entity2, entity_aabb(&store, entity2));

    sensor.evaluate(entity1, &store, &spatial);

    // Should detect collision
    assert!(sensor.is_colliding(entity1));
}

#[test]
fn test_edge_detection_falling_edge() {
    let mut store = EntityStore::new();
    let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
    let entity2 = store.spawn(Vec2::new(110.0, 110.0), Vec2::new(50.0, 50.0));

    let mut spatial = SpatialHash::new(MAX_ENTITIES);
    spatial.insert(entity1, entity_aabb(&store, entity1));
    spatial.insert(entity2, entity_aabb(&store, entity2));

    let mut sensor = CollisionSensor::new(MAX_ENTITIES);

    // First frame: collision exists
    sensor.evaluate(entity1, &store, &spatial);
    assert!(sensor.is_colliding(entity1));

    // Move entity2 away
    spatial.remove(entity2);
    store.transforms[1][0] = 300.0; // Move entity2 far away
    store.transforms[1][1] = 300.0;
    spatial.insert(entity2, entity_aabb(&store, entity2));

    sensor.evaluate(entity1, &store, &spatial);

    // Should no longer be colliding
    assert!(!sensor.is_colliding(entity1));
}

#[test]
fn test_multiple_entities_collision() {
    let mut store = EntityStore::new();
    let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
    let entity2 = store.spawn(Vec2::new(105.0, 105.0), Vec2::new(50.0, 50.0)); // Collides
    let entity3 = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0)); // No collision
    let entity4 = store.spawn(Vec2::new(110.0, 110.0), Vec2::new(50.0, 50.0)); // Collides

    let mut spatial = SpatialHash::new(MAX_ENTITIES);
    spatial.insert(entity1, entity_aabb(&store, entity1));
    spatial.insert(entity2, entity_aabb(&store, entity2));
    spatial.insert(entity3, entity_aabb(&store, entity3));
    spatial.insert(entity4, entity_aabb(&store, entity4));

    let mut sensor = CollisionSensor::new(MAX_ENTITIES);
    sensor.evaluate(entity1, &store, &spatial);

    // entity1 should be colliding (with entity2 and entity4)
    assert!(sensor.is_colliding(entity1));
    // entity3 should not be colliding
    assert!(!sensor.is_colliding(entity3));
}

#[test]
fn test_boundary_collision() {
    let mut store = EntityStore::new();
    // Entity at (100, 100) with size (50, 50) -> bounds [75, 125] × [75, 125]
    let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
    // Overlapping: center (120, 100) -> bounds [95, 145] × [75, 125]
    // Overlaps with entity1 from x=95 to x=125
    let entity2 = store.spawn(Vec2::new(120.0, 100.0), Vec2::new(50.0, 50.0));

    let mut spatial = SpatialHash::new(MAX_ENTITIES);
    spatial.insert(entity1, entity_aabb(&store, entity1));
    spatial.insert(entity2, entity_aabb(&store, entity2));

    let mut sensor = CollisionSensor::new(MAX_ENTITIES);
    sensor.evaluate(entity1, &store, &spatial);

    // Overlapping rects count as collision
    assert!(sensor.is_colliding(entity1));
}

#[test]
fn test_steady_state_detection() {
    let mut store = EntityStore::new();
    let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
    let entity2 = store.spawn(Vec2::new(105.0, 105.0), Vec2::new(50.0, 50.0));

    let mut spatial = SpatialHash::new(MAX_ENTITIES);
    spatial.insert(entity1, entity_aabb(&store, entity1));
    spatial.insert(entity2, entity_aabb(&store, entity2));

    let mut sensor = CollisionSensor::new(MAX_ENTITIES);

    // Simulate multiple frames of collision
    for _ in 0..6 {
        sensor.evaluate(entity1, &store, &spatial);
    }

    // Should be steady high for 6 ticks
    assert!(sensor.is_steady_colliding(entity1, 6));
}

#[test]
fn test_invalid_entity_returns_false() {
    let sensor = CollisionSensor::new(MAX_ENTITIES);
    let invalid_id = make_id(u32::MAX);

    assert!(!sensor.is_colliding(invalid_id));
    assert!(!sensor.on_collision_enter(invalid_id));
    assert!(!sensor.on_collision_exit(invalid_id));
}

#[test]
fn test_sensor_id_assignment() {
    let sensor = CollisionSensor::new(MAX_ENTITIES);
    // Sensor should have a valid ID
    assert!(sensor.sensor_id() > 0);
}

#[test]
fn test_entity_just_entered_collision() {
    let mut store = EntityStore::new();
    let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
    let entity2 = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0));

    let mut spatial = SpatialHash::new(MAX_ENTITIES);
    spatial.insert(entity1, entity_aabb(&store, entity1));
    spatial.insert(entity2, entity_aabb(&store, entity2));

    let mut sensor = CollisionSensor::new(MAX_ENTITIES);

    // Frame 1: no collision
    sensor.evaluate(entity1, &store, &spatial);

    // Frame 2: now colliding
    spatial.remove(entity2);
    store.transforms[1][0] = 110.0;
    store.transforms[1][1] = 110.0;
    spatial.insert(entity2, entity_aabb(&store, entity2));

    sensor.evaluate(entity1, &store, &spatial);

    // Should detect rising edge (just entered)
    assert!(sensor.on_collision_enter(entity1));
}

#[test]
fn test_entity_just_exited_collision() {
    let mut store = EntityStore::new();
    let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
    let entity2 = store.spawn(Vec2::new(105.0, 105.0), Vec2::new(50.0, 50.0));

    let mut spatial = SpatialHash::new(MAX_ENTITIES);
    spatial.insert(entity1, entity_aabb(&store, entity1));
    spatial.insert(entity2, entity_aabb(&store, entity2));

    let mut sensor = CollisionSensor::new(MAX_ENTITIES);

    // Frame 1: collision exists
    sensor.evaluate(entity1, &store, &spatial);

    // Frame 2: no collision (entity2 moved away)
    spatial.remove(entity2);
    store.transforms[1][0] = 300.0;
    store.transforms[1][1] = 300.0;
    spatial.insert(entity2, entity_aabb(&store, entity2));

    sensor.evaluate(entity1, &store, &spatial);

    // Should detect falling edge (just exited)
    assert!(sensor.on_collision_exit(entity1));
}

#[test]
fn test_sensor_id_is_unique() {
    let sensor1 = CollisionSensor::new(10);
    let sensor2 = CollisionSensor::new(10);
    assert_ne!(sensor1.sensor_id(), sensor2.sensor_id());
}
