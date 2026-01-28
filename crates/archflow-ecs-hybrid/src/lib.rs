//! ArchFlow ECS Hybrid - Record-ECS Synchronization Layer
//!
//! This crate provides bidirectional synchronization between Records
//! and ECS entities using bevy_ecs for maximum performance.
//!
//! ## Features
//!
//! - **RecordRef Component**: Links ECS entities to Records
//! - **Transform Component**: 2D position, rotation, scale
//! - **Renderable Component**: Rendering visibility and layering
//! - **Dirty Tracking**: O(C) change detection and processing
//! - **Optimized Sync**: Uses ChangeSet to process only changed records
//! - **Bidirectional**: Records ↔ ECS synchronization systems
//! - **Particle Systems**: High-performance particle simulation with SoA layout
//!
//! ## Architecture
//!
//! ```text
//! RecordStore ←→ ChangeSet ←→ ECS World
//!       ↓              ↓              ↓
//!   Records         Changes       Entities
//! ```
//!
//! ## Quick Start
//!
//! ```ignore
//! use archflow_ecs_hybrid::{RecordRef, Transform, sync_records_to_ecs_system};
//! use archflow_records::RecordStore;
//! use bevy_ecs::prelude::*;
//!
//! #[derive(Record)]
//! struct MyRecord {
//!     // fields
//! }
//!
//! fn main() {
//!     let mut world = World::new();
//!     world.insert_resource(RecordStore::<MyRecord>::new());
//!
//!     let mut schedule = Schedule::default();
//!     schedule.add_systems((
//!         sync_records_to_ecs_system::<MyRecord>,
//!         dirty_tracking_system,
//!     ));
//!
//!     loop {
//!         schedule.run(&mut world);
//!     }
//! }
//! ```

pub mod components;
pub mod systems;

pub use components::{
    Dirty, DirtyType, RecordRef, RenderableBundle, RenderableEcs, Transform, TransformBundle,
};
pub use systems::{
    cleanup_dirty_system, clear_dirty_flags_system, dirty_tracking_system,
    sync_records_to_ecs_system,
};

// Re-export particle components and systems
pub use components::particles::{
    EmitterConfig, ParticleAcceleration, ParticleBundle, ParticleColor, ParticleEmitter,
    ParticleLifetime, ParticlePosition, ParticleSize, ParticleVelocity,
};
pub use systems::particles::{
    Time, particle_animation_system, particle_cleanup_system, particle_emission_system,
    particle_lifetime_system, particle_physics_no_accel_system, particle_physics_system,
};

/// ECS Hybrid Error type
#[derive(Debug)]
pub enum EcsHybridError {
    /// Error converting RecordId to Entity
    InvalidEntityConversion(String),
}

impl std::fmt::Display for EcsHybridError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EcsHybridError::InvalidEntityConversion(msg) => {
                write!(f, "Failed to convert RecordId to Entity: {}", msg)
            }
        }
    }
}

impl std::error::Error for EcsHybridError {}

/// Result type for ECS Hybrid operations
pub type Result<T> = std::result::Result<T, EcsHybridError>;
