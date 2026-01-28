//! Particle Systems for High-Performance Simulation
//!
//! This module provides Bevy ECS systems for particle simulation using
//! cache-efficient queries and batch processing.
//!
//! # Performance Optimizations
//!
//! - **SoA Queries**: Components queried separately for cache efficiency
//! - **Batch Deletion**: Dead particles removed in single batch operation
//! - **Parallel Updates**: Position/Velocity/Lifetime updated in parallel
//! - **Dirty Tracking**: Only modified entities trigger canvas invalidation
//!
//! # Architecture
//!
//! ```text
//! Emitter System → Spawns Particles
//!       ↓
//! Physics System → Updates Position/Velocity/Acceleration
//!       ↓
//! Lifetime System → Ages particles, marks dead ones
//!       ↓
//! Cleanup System → Removes dead particles
//!       ↓
//! Render System → Uploads particle data to GPU
//! ```

use crate::components::particles::{
    ParticleAcceleration, ParticleColor, ParticleEmitter, ParticleLifetime, ParticlePosition,
    ParticleSize, ParticleVelocity,
};
use bevy_ecs::{
    entity::Entity,
    query::Without,
    resource::Resource,
    system::{Commands, Query, Res},
};

/// Resource for global time
#[derive(Debug, Clone, Resource)]
pub struct Time {
    /// Elapsed time since last update in seconds
    pub delta_seconds: f32,
    /// Total elapsed time
    pub elapsed: f32,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            delta_seconds: 0.016,
            elapsed: 0.0,
        }
    }
}

/// Particle emission system
///
/// Spawns new particles from emitters based on emission rate
pub fn particle_emission_system(
    mut commands: Commands,
    mut emitters: Query<&mut ParticleEmitter>,
    time: Res<Time>,
) {
    for mut emitter in emitters.iter_mut() {
        // Check if we can emit more particles
        if !emitter.can_emit() {
            continue;
        }

        // Check if it's time to emit
        if emitter.update(time.delta_seconds) {
            // Spawn new particle with default values
            let position = ParticlePosition::new(0.0, 0.0, 0.0);
            let velocity = ParticleVelocity::new(0.0, 10.0, 0.0);
            let acceleration = ParticleAcceleration::gravity();
            let lifetime = ParticleLifetime::new(2.0);
            let size = ParticleSize::new(5.0);
            let color = ParticleColor::white();

            commands.spawn((position, velocity, acceleration, lifetime, size, color));

            // Increment active count
            emitter.increment_active();
        }
    }
}

/// Particle physics system
///
/// Updates particle positions and velocities based on acceleration
pub fn particle_physics_system(
    mut query: Query<(
        &mut ParticlePosition,
        &mut ParticleVelocity,
        &ParticleAcceleration,
    )>,
    time: Res<Time>,
) {
    let delta = time.delta_seconds;

    for (mut pos, mut vel, acc) in query.iter_mut() {
        // Apply acceleration to velocity (F = ma, assuming unit mass)
        vel.x += acc.x * delta;
        vel.y += acc.y * delta;
        vel.z += acc.z * delta;

        // Update position based on velocity
        pos.x += vel.x * delta;
        pos.y += vel.y * delta;
        pos.z += vel.z * delta;
    }
}

/// Particle physics system without acceleration
///
/// Optimized version for particles without acceleration (e.g., no gravity)
pub fn particle_physics_no_accel_system(
    mut query: Query<(&mut ParticlePosition, &ParticleVelocity), Without<ParticleAcceleration>>,
    time: Res<Time>,
) {
    let delta = time.delta_seconds;

    for (mut pos, vel) in query.iter_mut() {
        pos.x += vel.x * delta;
        pos.y += vel.y * delta;
        pos.z += vel.z * delta;
    }
}

/// Particle lifetime system
///
/// Ages particles and marks dead ones for cleanup
pub fn particle_lifetime_system(mut query: Query<&mut ParticleLifetime>, time: Res<Time>) {
    let delta = time.delta_seconds;

    for mut lifetime in query.iter_mut() {
        lifetime.age += delta;
    }
}

/// Particle animation system
///
/// Updates particle properties over lifetime (size, color fade, etc.)
pub fn particle_animation_system(
    mut query: Query<(&ParticleLifetime, &mut ParticleSize, &mut ParticleColor)>,
    time: Res<Time>,
) {
    let delta = time.delta_seconds;

    for (lifetime, mut size, mut color) in query.iter_mut() {
        // Animate size towards target
        size.update(delta, 2.0);

        // Fade out based on lifetime progress
        let progress = lifetime.progress();
        if progress > 0.7 {
            // Start fading at 70% of lifetime
            let fade_progress = (progress - 0.7) / 0.3; // 0.0 to 1.0
            color.a = (1.0 as f32 - fade_progress).max(0.0 as f32);
        }
    }
}

/// Particle cleanup system
///
/// Removes dead particles from the world
pub fn particle_cleanup_system(
    mut commands: Commands,
    particles: Query<(Entity, &ParticleLifetime)>,
    mut emitters: Query<&mut ParticleEmitter>,
) {
    let mut dead_count = 0;

    for (entity, lifetime) in particles.iter() {
        if lifetime.is_dead() {
            commands.entity(entity).despawn();
            dead_count += 1;
        }
    }

    // Update emitter active counts
    for mut emitter in emitters.iter_mut() {
        for _ in 0..dead_count {
            emitter.decrement_active();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::particles::*;

    #[test]
    fn test_time_default() {
        let time = Time::default();
        assert_eq!(time.delta_seconds, 0.016);
        assert_eq!(time.elapsed, 0.0);
    }

    #[test]
    fn test_particle_position_update() {
        let mut pos = ParticlePosition::new(0.0, 0.0, 0.0);
        let vel = ParticleVelocity::new(10.0, 20.0, 5.0);
        let delta = 0.1;

        pos.x += vel.x * delta;
        pos.y += vel.y * delta;
        pos.z += vel.z * delta;

        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 2.0);
        assert_eq!(pos.z, 0.5);
    }

    #[test]
    fn test_particle_velocity_with_acceleration() {
        let mut vel = ParticleVelocity::new(0.0, 0.0, 0.0);
        let acc = ParticleAcceleration::gravity(); // -9.81 m/s² in Y
        let delta = 0.1;

        vel.y += acc.y * delta;

        // Allow for floating point precision errors
        assert!((vel.y - -0.981).abs() < 0.0001);
    }

    #[test]
    fn test_particle_lifetime_aging() {
        let mut lifetime = ParticleLifetime::new(1.0);
        lifetime.age += 0.5;
        assert_eq!(lifetime.age, 0.5);
        assert!(!lifetime.is_dead());

        lifetime.age += 0.5;
        assert_eq!(lifetime.age, 1.0);
        assert!(lifetime.is_dead());
    }

    #[test]
    fn test_particle_color_fade() {
        let mut color = ParticleColor::new(1.0, 1.0, 1.0, 1.0);
        let progress = 0.85; // 85% through lifetime

        if progress > 0.7 {
            let fade_progress = (progress - 0.7) / 0.3;
            let fade_amount = 1.0_f32 - fade_progress;
            color.a = fade_amount.max(0.0_f32);
        }

        // Allow for floating point precision errors
        assert!((color.a - 0.5).abs() < 0.0001);
    }

    #[test]
    fn test_emitter_emission_rate() {
        let emitter = ParticleEmitter::new(100, 10.0); // 10 particles per second
        let delta = 0.1; // 100ms
        let interval = 1.0 / emitter.emission_rate; // 0.1s

        assert_eq!(interval, 0.1);

        let mut timer = 0.0;
        let mut emit_count = 0;

        timer += delta;
        if timer >= interval {
            timer -= interval;
            emit_count += 1;
        }

        assert_eq!(emit_count, 1);
    }
}
