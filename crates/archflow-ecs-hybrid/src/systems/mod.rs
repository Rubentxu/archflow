//! ECS Systems Module
//!
//! This module provides systems for synchronizing Records with ECS entities
//! and tracking changes for bidirectional sync.

pub mod dirty_tracking_system;
pub mod particles;
pub mod sync_record_to_ecs;

pub use dirty_tracking_system::{
    cleanup_dirty_system, clear_dirty_flags_system, dirty_tracking_system,
};
pub use particles::{
    Time, particle_animation_system, particle_cleanup_system, particle_emission_system,
    particle_lifetime_system, particle_physics_no_accel_system, particle_physics_system,
};
pub use sync_record_to_ecs::sync_records_to_ecs_system;
