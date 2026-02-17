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
// - Query API: Type-safe multi-component queries
// - System execution: Prioritized, parallel-safe system scheduling
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
// use archflow_logic::ecs::{World, Component, VecStorage};
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
// // Create world and entity
// let mut world = World::new();
// let entity = world.create_entity();
// world.add_component(entity, Position { x: 10.0, y: 20.0 });
//
// // Query components
// world.query::<&Position>().each(|pos| {
//     println!("Position: {:?}", pos);
// });
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
pub mod archetype_query;
pub mod behavior_block;
pub mod component;
pub mod components;
pub mod entity_builder;
pub mod hybrid;
pub mod physics_components;
pub mod physics_system;
pub mod pool;
pub mod query;
pub mod registry;
pub mod scheduler;
pub mod simd;
pub mod simd_integration;
pub mod sparse_set;
pub mod system;
pub mod world;

#[cfg(test)]
mod entity_builder_tests;

#[cfg(test)]
mod integration_tests;

// Core types
pub use archetype::{Archetype, ArchetypeId, ArchetypeStorage, BatchIter, ComponentColumn};
pub use component::{Component, ComponentId, ComponentStorage, VecStorage};
pub use components::{
    AudioActuatorComponent, HighlightActuatorComponent, MouseSensorComponent,
    MoveActuatorComponent, NamedComponent, SelectActuatorComponent, SignalStateComponent,
};
pub use hybrid::{
    ActuatorComponent, BgeLogicConfig, BgeLogicStats, BgeLogicSystem, ClickType,
    ControllerComponent, SensorComponent, SensorComponentType, SensorConfig, SensorEvaluation,
    SensorRef,
};
pub use physics_components::{
    Acceleration, AnimationState, HighlightState, PhysicsMaterial, SelectionState, Transform,
    Velocity,
};
pub use physics_system::{PhysicsConfig, PhysicsStats, PhysicsSystem};
pub use pool::{ColumnPool, PoolStats};
pub use query::{
    EntityId, Query, QueryIter, QueryIterExt, QueryMut, QueryParameter, With, Without,
};
pub use registry::ComponentRegistry;
pub use scheduler::SystemScheduler;
pub use simd::{
    BatchPhysicsConfig, BatchPhysicsProcessor, BatchStats, MortonEncoder, SimdBatchIterator,
    SimdPhysicsConfig, SimdPhysicsProcessor, SimdStats,
};
pub use simd_integration::{
    Aabb2D, Aabb3D, CollisionSimdDetector, SIMD_F32_BATCH, SIMD_U8_BATCH, SpatialHash,
};
pub use sparse_set::SparseSet;
pub use system::{System, SystemInfo};
pub use world::World;
