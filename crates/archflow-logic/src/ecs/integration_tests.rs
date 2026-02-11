// ═══════════════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - ECS Integration Tests
//
// Tests for ECS Query API, World management, and component operations.
// These tests verify core ECS functionality.
//
// ═══════════════════════════════════════════════════════════════════════════════════════════════

use alloc::vec;
use alloc::vec::Vec;

use crate::ecs::component::{Component, VecStorage};
use crate::ecs::world::World;

// ═══════════════════════════════════════════════════════════════════════════════════════════════
// Test Components
// ═══════════════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

impl Component for Position {
    type Storage = VecStorage<Position>;
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Velocity {
    dx: f32,
    dy: f32,
    dz: f32,
}

impl Component for Velocity {
    type Storage = VecStorage<Velocity>;
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Health {
    current: u32,
    max: u32,
}

impl Component for Health {
    type Storage = VecStorage<Health>;
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Damage {
    amount: u32,
}

impl Component for Damage {
    type Storage = VecStorage<Damage>;
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// Tests: Query with Components
// ═══════════════════════════════════════════════════════════════════════════════════════

#[test]
fn test_query_uses_component_storage() {
    let mut world = World::new();

    // Create entities with different component combinations
    let e1 = world.create_entity();
    let e2 = world.create_entity();
    let e3 = world.create_entity();
    let e4 = world.create_entity();

    // e1: Position + Velocity
    world.add_component(
        e1,
        Position {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
    );
    world.add_component(
        e1,
        Velocity {
            dx: 0.1,
            dy: 0.2,
            dz: 0.3,
        },
    );

    // e2: Position + Velocity + Health
    world.add_component(
        e2,
        Position {
            x: 4.0,
            y: 5.0,
            z: 6.0,
        },
    );
    world.add_component(
        e2,
        Velocity {
            dx: 0.4,
            dy: 0.5,
            dz: 0.6,
        },
    );
    world.add_component(
        e2,
        Health {
            current: 100,
            max: 100,
        },
    );

    // e3: Position only
    world.add_component(
        e3,
        Position {
            x: 7.0,
            y: 8.0,
            z: 9.0,
        },
    );

    // e4: Position + Health
    world.add_component(
        e4,
        Position {
            x: 10.0,
            y: 11.0,
            z: 12.0,
        },
    );
    world.add_component(
        e4,
        Health {
            current: 50,
            max: 100,
        },
    );

    // Query for Position + Velocity should return e1 and e2 only
    let mut count = 0;
    let mut sum_x = 0.0f32;

    world.query::<(&Position, &Velocity)>().each(|(pos, _vel)| {
        count += 1;
        sum_x += pos.x;
    });

    assert_eq!(
        count, 2,
        "Should find exactly 2 entities with Position + Velocity"
    );
    assert!(
        (sum_x - 5.0).abs() < 0.001,
        "Sum of x positions should be 1.0 + 4.0 = 5.0"
    );
}

#[test]
fn test_query_with_multiple_entities() {
    let mut world = World::new();

    // Create multiple entities
    for i in 0..10 {
        let entity = world.create_entity();
        world.add_component(
            entity,
            Position {
                x: i as f32,
                y: i as f32 * 2.0,
                z: i as f32 * 3.0,
            },
        );
        world.add_component(
            entity,
            Velocity {
                dx: i as f32 * 0.1,
                dy: i as f32 * 0.2,
                dz: i as f32 * 0.3,
            },
        );
    }

    // Query should return all 10 entities
    let mut count = 0;
    world
        .query::<(&Position, &Velocity)>()
        .each(|(_pos, _vel)| {
            count += 1;
        });

    assert_eq!(count, 10, "Should find all 10 entities");
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// Tests: Mutable Queries
// ═══════════════════════════════════════════════════════════════════════════════════════

#[test]
fn test_query_mut_modifies_components() {
    let mut world = World::new();

    let e1 = world.create_entity();
    let e2 = world.create_entity();

    world.add_component(
        e1,
        Position {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
    );
    world.add_component(
        e1,
        Velocity {
            dx: 0.1,
            dy: 0.2,
            dz: 0.3,
        },
    );
    world.add_component(
        e2,
        Position {
            x: 4.0,
            y: 5.0,
            z: 6.0,
        },
    );
    world.add_component(
        e2,
        Velocity {
            dx: 0.4,
            dy: 0.5,
            dz: 0.6,
        },
    );

    // Modify all positions using query_mut
    let mut query = world.query_mut::<(&mut Position, &Velocity)>();
    query.each(|(pos, vel)| {
        pos.x += vel.dx;
        pos.y += vel.dy;
        pos.z += vel.dz;
    });

    // Verify modifications
    let p1 = world.get_component::<Position>(e1).unwrap();
    let p2 = world.get_component::<Position>(e2).unwrap();

    assert!((p1.x - 1.1).abs() < 0.001, "e1.x should be 1.1");
    assert!((p1.y - 2.2).abs() < 0.001, "e1.y should be 2.2");
    assert!((p1.z - 3.3).abs() < 0.001, "e1.z should be 3.3");

    assert!((p2.x - 4.4).abs() < 0.001, "e2.x should be 4.4");
    assert!((p2.y - 5.5).abs() < 0.001, "e2.y should be 5.5");
    assert!((p2.z - 6.6).abs() < 0.001, "e2.z should be 6.6");
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// Tests: Entity Lifecycle with Queries
// ═══════════════════════════════════════════════════════════════════════════════════════

#[test]
fn test_query_after_entity_deletion() {
    let mut world = World::new();

    let e1 = world.create_entity();
    let e2 = world.create_entity();
    let e3 = world.create_entity();

    world.add_component(
        e1,
        Position {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
    );
    world.add_component(
        e2,
        Position {
            x: 4.0,
            y: 5.0,
            z: 6.0,
        },
    );
    world.add_component(
        e3,
        Position {
            x: 7.0,
            y: 8.0,
            z: 9.0,
        },
    );

    // Delete e2
    world.destroy_entity(e2);

    // Query should return only e1 and e3
    let mut count = 0;
    let mut sum_x = 0.0f32;

    world.query::<&Position>().each(|pos| {
        count += 1;
        sum_x += pos.x;
    });

    assert_eq!(count, 2, "Should find only 2 entities");
    assert!((sum_x - 8.0).abs() < 0.001, "Sum should be 1.0 + 7.0 = 8.0");
}

#[test]
fn test_query_with_stale_entity_reference() {
    let mut world = World::new();

    let e1 = world.create_entity();
    let e2 = world.create_entity();

    world.add_component(
        e1,
        Position {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
    );
    world.add_component(
        e2,
        Position {
            x: 4.0,
            y: 5.0,
            z: 6.0,
        },
    );

    // Delete e1 and create new entity (will reuse index)
    world.destroy_entity(e1);
    let e3 = world.create_entity();

    // e3 has different generation than e1
    assert_ne!(e1.generation(), e3.generation());

    // Query should only find e2 and e3 (not stale e1)
    let mut count = 0;

    world.query::<&Position>().each(|_pos| {
        count += 1;
    });

    assert_eq!(count, 2, "Should find exactly 2 entities (e2 and e3)");
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// Tests: Complex Queries with Multiple Components
// ═══════════════════════════════════════════════════════════════════════════════════════

#[test]
fn test_query_four_components() {
    let mut world = World::new();

    let e1 = world.create_entity();
    let e2 = world.create_entity();

    world.add_component(
        e1,
        Position {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
    );
    world.add_component(
        e1,
        Velocity {
            dx: 0.1,
            dy: 0.2,
            dz: 0.3,
        },
    );
    world.add_component(
        e1,
        Health {
            current: 100,
            max: 100,
        },
    );
    world.add_component(e1, Damage { amount: 25 });

    world.add_component(
        e2,
        Position {
            x: 4.0,
            y: 5.0,
            z: 6.0,
        },
    );
    world.add_component(
        e2,
        Velocity {
            dx: 0.4,
            dy: 0.5,
            dz: 0.6,
        },
    );
    world.add_component(
        e2,
        Health {
            current: 50,
            max: 100,
        },
    );
    world.add_component(e2, Damage { amount: 10 });

    let mut count = 0;
    let mut total_damage = 0u32;

    world
        .query::<(&Position, &Velocity, &Health, &Damage)>()
        .each(|(_pos, _vel, _health, dmg)| {
            count += 1;
            total_damage += dmg.amount;
        });

    assert_eq!(count, 2);
    assert_eq!(total_damage, 35, "Total damage should be 25 + 10");
}

#[test]
fn test_query_is_empty() {
    let world = World::new();

    let query = world.query::<&Position>();
    assert!(query.is_empty(), "Query should be empty with no entities");

    let mut world = World::new();
    let entity = world.create_entity();
    world.add_component(
        entity,
        Position {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    );

    let query = world.query::<&Position>();
    assert!(!query.is_empty(), "Query should have entities");

    let query = world.query::<(&Position, &Velocity)>();
    assert!(query.is_empty(), "Query should be empty (no Velocity)");
}

#[test]
fn test_world_create_destroy_entities() {
    let mut world = World::new();

    let e1 = world.create_entity();
    let e2 = world.create_entity();
    let e3 = world.create_entity();

    assert_eq!(world.entity_count(), 3);

    world.destroy_entity(e2);
    assert_eq!(world.entity_count(), 2);

    let e4 = world.create_entity();
    assert_ne!(e2, e4, "New entity should have different generation");

    assert_eq!(world.entity_count(), 3);
}
