// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - EntityBuilder Tests
//
// Tests for the fluent EntityBuilder API (EPIC-ECS-001).
// This verifies world.spawn(), .insert(), .name(), and .build() methods.
//
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(unused)]

use alloc::string::String;

use crate::ecs::component::{Component, VecStorage};
use crate::ecs::entity_builder::WorldSpawnExt;
use crate::ecs::world::World;

// ═══════════════════════════════════════════════════════════════════════════════
// Test Components
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}

impl Component for Position {
    type Storage = VecStorage<Position>;
}

impl Default for Position {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Velocity {
    dx: f32,
    dy: f32,
}

impl Component for Velocity {
    type Storage = VecStorage<Velocity>;
}

impl Default for Velocity {
    fn default() -> Self {
        Self { dx: 0.0, dy: 0.0 }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests: world.spawn() - HU-ECS-001
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_spawn_returns_entity_builder() {
    let mut world = World::new();

    // world.spawn() should return EntityBuilder
    let _builder = world.spawn();
}

#[test]
fn test_spawn_build_creates_entity() {
    let mut world = World::new();

    // Spawn and build should create a valid entity
    let entity = world.spawn().build();

    // Entity should be valid (index 0)
    assert_eq!(entity.index(), 0);
}

#[test]
fn test_multiple_spawns_increment_entity_ids() {
    let mut world = World::new();

    let e1 = world.spawn().build();
    let e2 = world.spawn().build();
    let e3 = world.spawn().build();

    // Each entity should have a unique ID
    assert_ne!(e1, e2);
    assert_ne!(e2, e3);
    assert_ne!(e1, e3);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests: .insert() - HU-ECS-002
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_insert_single_component() {
    let mut world = World::new();

    let entity = world.spawn().insert(Position { x: 10.0, y: 20.0 }).build();

    // Verify component was added
    let pos = world.get_component::<Position>(entity);
    assert!(pos.is_some());
    assert_eq!(pos.unwrap().x, 10.0);
    assert_eq!(pos.unwrap().y, 20.0);
}

#[test]
fn test_insert_multiple_components() {
    let mut world = World::new();

    let entity = world
        .spawn()
        .insert(Position { x: 100.0, y: 200.0 })
        .insert(Velocity { dx: 1.0, dy: 2.0 })
        .build();

    // Verify both components were added
    let pos = world.get_component::<Position>(entity);
    let vel = world.get_component::<Velocity>(entity);

    assert!(pos.is_some());
    assert!(vel.is_some());
    assert_eq!(pos.unwrap().x, 100.0);
    assert_eq!(vel.unwrap().dx, 1.0);
}

#[test]
fn test_insert_returns_self_for_chaining() {
    let mut world = World::new();

    // If insert() returns Self, we can chain multiple inserts
    // This is verified at compile time
    let _entity = world
        .spawn()
        .insert(Position::default())
        .insert(Velocity::default())
        .insert(Position { x: 50.0, y: 60.0 })
        .build();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests: .name() - HU-ECS-004
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_name_creates_valid_entity() {
    let mut world = World::new();

    let entity = world.spawn().name("Player").build();

    // The entity should be created
    assert!(world.is_entity_alive(entity));
}

#[test]
fn test_name_allows_chaining() {
    let mut world = World::new();

    // Multiple .name() calls should work (last one wins)
    let entity = world
        .spawn()
        .name("First")
        .name("Second")
        .name("Final")
        .build();

    assert!(world.is_entity_alive(entity));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests: Combined API
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_full_fluent_api() {
    let mut world = World::new();

    // Full API: spawn().insert().name().build()
    let entity = world
        .spawn()
        .insert(Position { x: 42.0, y: 84.0 })
        .insert(Velocity { dx: 10.0, dy: 20.0 })
        .name("TestEntity")
        .build();

    // Verify everything worked
    let pos = world.get_component::<Position>(entity);
    let vel = world.get_component::<Velocity>(entity);

    assert!(pos.is_some());
    assert!(vel.is_some());
    assert_eq!(pos.unwrap().x, 42.0);
    assert_eq!(vel.unwrap().dy, 20.0);
}

#[test]
fn test_spawn_with_only_components() {
    let mut world = World::new();

    // Can create entity with just components, no name
    let e1 = world.spawn().insert(Position::default()).build();
    let e2 = world.spawn().build();

    assert_ne!(e1, e2);
}

#[test]
fn test_spawn_with_only_name() {
    let mut world = World::new();

    // Can create entity with just a name
    let entity = world.spawn().name("OnlyName").build();

    assert!(world.is_entity_alive(entity));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests: Edge Cases
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_entity_reuse_after_destroy() {
    let mut world = World::new();

    let e1 = world.spawn().build();
    let e1_index = e1.index();

    // Destroy the entity
    world.destroy_entity(e1);

    // Spawning a new entity should work
    let e2 = world.spawn().build();

    // The new entity should be valid
    assert!(world.is_entity_alive(e2));
}

#[test]
fn test_empty_spawn_creates_valid_entity() {
    let mut world = World::new();

    // Spawning with no components should still create a valid entity
    let entity = world.spawn().build();

    assert!(world.is_entity_alive(entity));
}
