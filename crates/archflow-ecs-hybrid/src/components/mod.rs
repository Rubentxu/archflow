//! ECS Components Module
//!
//! This module provides components for linking ECS entities to Records
//! and managing synchronization state.

pub mod particles;
pub mod record_ref;
pub mod renderable_ecs;
pub mod transform;

pub use particles::{
    EmitterConfig, ParticleAcceleration, ParticleBundle, ParticleColor, ParticleEmitter,
    ParticleLifetime, ParticlePosition, ParticleSize, ParticleVelocity,
};
pub use record_ref::{Dirty, DirtyType, RecordRef};
pub use renderable_ecs::{RenderableBundle, RenderableEcs};
pub use transform::{Transform, TransformBundle};
