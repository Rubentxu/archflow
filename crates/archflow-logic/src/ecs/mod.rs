// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Entity Component System (ECS) Module
//
// This module provides a high-performance, no_std compatible Entity Component System
// for the ArchFlow Engine.
//
// Architecture Overview:
// ----------------------
// The ECS follows the Data-Oriented Design (DOD) paradigm, separating data from logic:
// - Entity: A unique identifier (usize) representing a game object
// - Component: Pure data structs (Position, Velocity, Health, etc.)
// - System: Logic that operates on entities with specific components
//
// Key Features:
// - Zero-cost abstraction: No virtual dispatch overhead
// - Cache-friendly: Component data stored contiguously in memory
// - Type-safe: Compile-time component type checking
// - Flexible storage: Support for dense (Vec) and sparse (SparseSet) storage
// - Dynamic registration: Components can be registered at runtime
// - Archetype-based storage: Groups entities with same components for optimal cache utilization
//
// Storage Strategies:
// -------------------
// - VecStorage: Simple, fast for components that most entities have
// - SparseSet: Memory-efficient for sparse components (cache-friendly iteration)
// - ArchetypeStorage: Data-Oriented Design storage with column-oriented component data
//
// Examples:
// ---------
// ```ignore
// use archflow_logic::ecs::{ComponentRegistry, Component, VecStorage};
//
// // Define a component
// #[derive(Clone, Debug)]
// struct Position {
//     x: f32,
//     y: f32,
// }
//
// impl Component for Position {
//     type Storage = VecStorage<Position>;
// }
//
// // Create registry and register component
// let mut registry = ComponentRegistry::new();
// registry.register::<Position>();
//
// // Add component to entity
// let positions = registry.get_storage_mut::<Position>().unwrap();
// positions.insert(0, Position { x: 10.0, y: 20.0 });
//
// // Access component
// let positions = registry.get_storage::<Position>().unwrap();
// assert_eq!(positions.get(0).unwrap().x, 10.0);
// ```
//
// Architecture Reference:
// -----------------------
// - Data-Oriented Design by Richard Fabian
// - Speck ECS SparseSet implementation
// - Unity DOTS architecture patterns
// - Archetype-based storage (similar to Bevy ECS)
// ═══════════════════════════════════════════════════════════════════════════════

#![no_std]

// Re-export all public types
pub mod archetype;
pub mod component;
pub mod components;
pub mod registry;
pub mod sparse_set;

// Core types
pub use archetype::{Archetype, ArchetypeId, ArchetypeStorage, BatchIter, ComponentColumn};
pub use component::{Component, ComponentId, ComponentStorage, VecStorage};
pub use components::{
    HighlightActuatorComponent, MouseSensorComponent, MoveActuatorComponent,
    SelectActuatorComponent, SignalStateComponent,
};
pub use registry::ComponentRegistry;
pub use sparse_set::SparseSet;

#[cfg(test)]
mod tests {
    use super::*;

    // Test component definitions
    #[derive(Clone, Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    impl Component for Position {
        type Storage = VecStorage<Position>;
    }

    #[derive(Clone, Debug, PartialEq)]
    struct Velocity {
        dx: f32,
        dy: f32,
    }

    impl Component for Velocity {
        type Storage = VecStorage<Velocity>;
    }

    #[derive(Clone, Debug, PartialEq)]
    struct Health {
        current: u32,
        max: u32,
    }

    impl Component for Health {
        type Storage = SparseSet<Health>;
    }

    #[test]
    fn test_ecs_basic_usage() {
        let mut registry = ComponentRegistry::new();

        // Register components
        registry.register::<Position>();
        registry.register::<Velocity>();
        registry.register::<Health>();

        assert!(registry.is_registered::<Position>());
        assert!(registry.is_registered::<Velocity>());
        assert!(registry.is_registered::<Health>());
        assert_eq!(registry.len(), 3);
    }

    #[test]
    fn test_ecs_entity_creation() {
        let mut registry = ComponentRegistry::new();

        registry.register::<Position>();
        registry.register::<Velocity>();

        // Create entity 0 with Position and Velocity
        {
            let positions = registry.get_storage_mut::<Position>().unwrap();
            positions.insert(0, Position { x: 0.0, y: 0.0 });

            let velocities = registry.get_storage_mut::<Velocity>().unwrap();
            velocities.insert(0, Velocity { dx: 1.0, dy: 1.0 });
        }

        // Verify entity 0 has both components
        let positions = registry.get_storage::<Position>().unwrap();
        let velocities = registry.get_storage::<Velocity>().unwrap();

        assert_eq!(positions.get(0), Some(&Position { x: 0.0, y: 0.0 }));
        assert_eq!(velocities.get(0), Some(&Velocity { dx: 1.0, dy: 1.0 }));
    }

    #[test]
    fn test_ecs_multiple_entities() {
        let mut registry = ComponentRegistry::new();

        registry.register::<Position>();

        // Create multiple entities
        let positions = registry.get_storage_mut::<Position>().unwrap();
        for i in 0..10 {
            positions.insert(
                i,
                Position {
                    x: i as f32,
                    y: i as f32 * 2.0,
                },
            );
        }

        // Verify all entities
        let positions = registry.get_storage::<Position>().unwrap();
        for i in 0..10 {
            assert_eq!(
                positions.get(i),
                Some(&Position {
                    x: i as f32,
                    y: i as f32 * 2.0
                })
            );
        }
    }

    #[test]
    fn test_ecs_sparse_component() {
        let mut registry = ComponentRegistry::new();

        registry.register::<Health>();

        // Add health to sparse entities
        let health = registry.get_storage_mut::<Health>().unwrap();
        health.insert(
            0,
            Health {
                current: 100,
                max: 100,
            },
        );
        health.insert(
            10,
            Health {
                current: 50,
                max: 100,
            },
        );
        health.insert(
            100,
            Health {
                current: 75,
                max: 100,
            },
        );

        // Verify sparse storage efficiency
        let health = registry.get_storage::<Health>().unwrap();
        assert_eq!(health.len(), 3);
        assert!(health.contains(0));
        assert!(!health.contains(5));
        assert!(health.contains(10));
        assert!(!health.contains(50));
        assert!(health.contains(100));
    }

    #[test]
    fn test_ecs_component_id() {
        let pos_id = ComponentId::of::<Position>();
        let vel_id = ComponentId::of::<Velocity>();
        let health_id = ComponentId::of::<Health>();

        assert_eq!(pos_id, ComponentId::of::<Position>());
        assert_ne!(pos_id, vel_id);
        assert_ne!(pos_id, health_id);
        assert_ne!(vel_id, health_id);
    }

    #[test]
    fn test_ecs_iteration() {
        let mut registry = ComponentRegistry::new();

        registry.register::<Position>();

        let positions = registry.get_storage_mut::<Position>().unwrap();
        positions.insert(0, Position { x: 0.0, y: 0.0 });
        positions.insert(1, Position { x: 1.0, y: 2.0 });
        positions.insert(2, Position { x: 2.0, y: 4.0 });

        // Iterate over positions
        let positions = registry.get_storage::<Position>().unwrap();
        let mut count = 0;
        for pos in positions.iter() {
            assert!(pos.x >= 0.0 && pos.x <= 2.0);
            count += 1;
        }
        assert_eq!(count, 3);
    }

    #[test]
    fn test_ecs_sparse_iteration() {
        let mut registry = ComponentRegistry::new();

        registry.register::<Health>();

        let health = registry.get_storage_mut::<Health>().unwrap();
        health.insert(
            0,
            Health {
                current: 100,
                max: 100,
            },
        );
        health.insert(
            10,
            Health {
                current: 50,
                max: 100,
            },
        );
        health.insert(
            100,
            Health {
                current: 75,
                max: 100,
            },
        );

        // Iterate over sparse set
        let health = registry.get_storage::<Health>().unwrap();
        let mut count = 0;
        for h in health.iter() {
            assert!(h.current <= h.max);
            count += 1;
        }
        assert_eq!(count, 3);
    }

    #[test]
    fn test_ecs_component_removal() {
        let mut registry = ComponentRegistry::new();

        registry.register::<Position>();

        let positions = registry.get_storage_mut::<Position>().unwrap();
        positions.insert(0, Position { x: 0.0, y: 0.0 });
        positions.insert(1, Position { x: 1.0, y: 2.0 });

        // Remove component from entity 0
        let removed = positions.remove(0);
        assert_eq!(removed, Some(Position { x: 0.0, y: 0.0 }));

        // Verify removal
        assert!(!positions.contains(0));
        assert!(positions.contains(1));
    }

    #[test]
    fn test_ecs_mixed_storage_types() {
        let mut registry = ComponentRegistry::new();

        registry.register::<Position>(); // VecStorage
        registry.register::<Velocity>(); // VecStorage
        registry.register::<Health>(); // SparseSet

        // Add components to entity 0
        {
            let positions = registry.get_storage_mut::<Position>().unwrap();
            positions.insert(0, Position { x: 0.0, y: 0.0 });

            let velocities = registry.get_storage_mut::<Velocity>().unwrap();
            velocities.insert(0, Velocity { dx: 1.0, dy: 1.0 });

            let health = registry.get_storage_mut::<Health>().unwrap();
            health.insert(
                0,
                Health {
                    current: 100,
                    max: 100,
                },
            );
        }

        // Verify all components are accessible
        let positions = registry.get_storage::<Position>().unwrap();
        let velocities = registry.get_storage::<Velocity>().unwrap();
        let health = registry.get_storage::<Health>().unwrap();

        assert!(positions.contains(0));
        assert!(velocities.contains(0));
        assert!(health.contains(0));
    }

    #[test]
    fn test_ecs_registry_clear() {
        let mut registry = ComponentRegistry::new();

        registry.register::<Position>();
        registry.register::<Velocity>();

        {
            let positions = registry.get_storage_mut::<Position>().unwrap();
            positions.insert(0, Position { x: 0.0, y: 0.0 });
        }

        registry.clear();

        assert!(!registry.is_registered::<Position>());
        assert!(!registry.is_registered::<Velocity>());
        assert!(registry.is_empty());
    }

    // Archetype storage tests
    #[test]
    fn test_archetype_storage_basic() {
        let mut storage = ArchetypeStorage::new();

        let mut types = BTreeMap::new();
        types.insert(
            ComponentId::of::<Position>(),
            core::mem::size_of::<Position>(),
        );
        types.insert(
            ComponentId::of::<Velocity>(),
            core::mem::size_of::<Velocity>(),
        );

        let arch_id = storage.add_entity(0, types);

        assert_eq!(storage.archetype_count(), 1);
        assert_eq!(storage.get_archetype_id(0), Some(arch_id));
    }

    #[test]
    fn test_archetype_id_stability() {
        let types = vec![ComponentId::of::<Position>(), ComponentId::of::<Velocity>()];

        let id1 = ArchetypeId::from_types(&types);
        let id2 = ArchetypeId::from_types(&types);

        assert_eq!(id1, id2);
    }
}
