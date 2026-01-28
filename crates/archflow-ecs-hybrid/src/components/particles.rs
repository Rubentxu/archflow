//! Particle Components for High-Performance Particle Systems
//!
//! This module provides Bevy ECS components for particle simulation using
//! Structure of Arrays (SoA) layout for maximum cache efficiency and SIMD
//! optimization.
//!
//! # Performance Characteristics
//!
//! - **SoA Layout**: Components stored in contiguous arrays for cache efficiency
//! - **SIMD-Friendly**: Compiler can auto-vectorize iteration over components
//! - **Batch Processing**: Update systems process thousands of particles in parallel
//! - **Memory Efficient**: Only alive particles consume memory and CPU
//!
//! # Example
//!
//! ```ignore
//! use bevy_ecs::prelude::*;
//! use archflow_ecs_hybrid::particles::*;
//!
//! fn spawn_particle_system(mut commands: Commands) {
//!     // Create particle emitter
//!     commands.spawn((
//!         ParticleEmitter::new(1000, 100.0), // max 1000 particles, 100 per second
//!         EmitterConfig {
//!             position: Vec3::new(0.0, 0.0, 0.0),
//!             velocity_range: Vec3::new(-50.0, 100.0, -50.0),
//!             lifetime: 2.0,
//!             size: 5.0,
//!             color: Color::rgba(1.0, 0.5, 0.0, 1.0),
//!         },
//!     ));
//! }
//!
//! fn update_particles(
//!     mut query: Query<(&mut ParticlePosition, &mut ParticleVelocity, &mut ParticleLifetime)>,
//!     time: Res<Time>,
//! ) {
//!     let delta = time.delta_seconds();
//!     for (mut pos, mut vel, mut lifetime) in query.iter_mut() {
//!         // Apply gravity
//!         vel.y -= 9.81 * delta;
//!         // Update position
//!         pos.x += vel.x * delta;
//!         pos.y += vel.y * delta;
//!         pos.z += vel.z * delta;
//!         // Update lifetime
//!         lifetime.age += delta;
//!     }
//! }
//! ```

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

/// 3D position component for particles
///
/// Stored as separate x, y, z fields for SoA layout
#[derive(Debug, Clone, Copy, Component, Serialize, Deserialize)]
pub struct ParticlePosition {
    /// X coordinate in world space
    pub x: f32,
    /// Y coordinate in world space
    pub y: f32,
    /// Z coordinate in world space
    pub z: f32,
}

impl ParticlePosition {
    /// Create a new particle position
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Create from a 3D tuple
    pub fn from_tuple(pos: (f32, f32, f32)) -> Self {
        Self {
            x: pos.0,
            y: pos.1,
            z: pos.2,
        }
    }

    /// Get position as a tuple
    pub fn as_tuple(&self) -> (f32, f32, f32) {
        (self.x, self.y, self.z)
    }
}

impl Default for ParticlePosition {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

/// 3D velocity component for particles
///
/// Separate from position for cache-efficient updates
#[derive(Debug, Clone, Copy, Component, Serialize, Deserialize)]
pub struct ParticleVelocity {
    /// X velocity
    pub x: f32,
    /// Y velocity
    pub y: f32,
    /// Z velocity
    pub z: f32,
}

impl ParticleVelocity {
    /// Create a new particle velocity
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Create zero velocity
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// Get velocity magnitude
    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Normalize velocity
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0001 {
            Self {
                x: self.x / mag,
                y: self.y / mag,
                z: self.z / mag,
            }
        } else {
            *self
        }
    }
}

impl Default for ParticleVelocity {
    fn default() -> Self {
        Self::zero()
    }
}

/// Acceleration component for particles
///
/// Used for forces like gravity, wind, attraction
#[derive(Debug, Clone, Copy, Component, Serialize, Deserialize)]
pub struct ParticleAcceleration {
    /// X acceleration
    pub x: f32,
    /// Y acceleration
    pub y: f32,
    /// Z acceleration
    pub z: f32,
}

impl ParticleAcceleration {
    /// Create gravity acceleration (9.81 m/s² downward)
    pub fn gravity() -> Self {
        Self {
            x: 0.0,
            y: -9.81,
            z: 0.0,
        }
    }

    /// Create zero acceleration
    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

impl Default for ParticleAcceleration {
    fn default() -> Self {
        Self::zero()
    }
}

/// Lifetime component for particles
///
/// Tracks particle age and maximum lifetime for automatic cleanup
#[derive(Debug, Clone, Copy, Component, Serialize, Deserialize)]
pub struct ParticleLifetime {
    /// Current age in seconds
    pub age: f32,
    /// Maximum lifetime in seconds
    pub max_age: f32,
}

impl ParticleLifetime {
    /// Create a new particle lifetime
    pub fn new(max_age: f32) -> Self {
        Self { age: 0.0, max_age }
    }

    /// Check if particle is dead (age >= max_age)
    pub fn is_dead(&self) -> bool {
        self.age >= self.max_age
    }

    /// Get remaining lifetime
    pub fn remaining(&self) -> f32 {
        (self.max_age - self.age).max(0.0 as f32)
    }

    /// Get lifetime progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        if self.max_age > 0.0 {
            (self.age / self.max_age).min(1.0)
        } else {
            1.0
        }
    }
}

/// Size component for particles
///
/// Supports size animation over lifetime
#[derive(Debug, Clone, Copy, Component, Serialize, Deserialize)]
pub struct ParticleSize {
    /// Current size in pixels
    pub current: f32,
    /// Target size (for animation)
    pub target: f32,
}

impl ParticleSize {
    /// Create a new particle size
    pub fn new(size: f32) -> Self {
        Self {
            current: size,
            target: size,
        }
    }

    /// Create with animation
    pub fn with_animation(initial: f32, target: f32) -> Self {
        Self {
            current: initial,
            target,
        }
    }

    /// Update size towards target
    pub fn update(&mut self, delta: f32, speed: f32) {
        let diff = self.target - self.current;
        self.current += diff * speed * delta;
    }
}

impl Default for ParticleSize {
    fn default() -> Self {
        Self::new(1.0)
    }
}

/// Color component for particles
///
/// Supports RGBA color with alpha
#[derive(Debug, Clone, Copy, Component, Serialize, Deserialize)]
pub struct ParticleColor {
    /// Red channel (0.0 to 1.0)
    pub r: f32,
    /// Green channel (0.0 to 1.0)
    pub g: f32,
    /// Blue channel (0.0 to 1.0)
    pub b: f32,
    /// Alpha channel (0.0 to 1.0)
    pub a: f32,
}

impl ParticleColor {
    /// Create a new particle color
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create from RGBA tuple
    pub fn from_rgba(rgba: (f32, f32, f32, f32)) -> Self {
        Self {
            r: rgba.0,
            g: rgba.1,
            b: rgba.2,
            a: rgba.3,
        }
    }

    /// Create white color
    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0)
    }

    /// Fade color towards transparent
    pub fn fade(&mut self, delta: f32, speed: f32) {
        self.a = (self.a - speed * delta).max(0.0 as f32);
    }

    /// Get as RGBA tuple
    pub fn as_rgba(&self) -> (f32, f32, f32, f32) {
        (self.r, self.g, self.b, self.a)
    }
}

impl Default for ParticleColor {
    fn default() -> Self {
        Self::white()
    }
}

/// Particle emitter component
///
/// Attached to entities that spawn particles
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct ParticleEmitter {
    /// Maximum number of particles this emitter can have active
    pub max_particles: usize,
    /// Particles to emit per second
    pub emission_rate: f32,
    /// Time since last emission
    #[serde(skip)]
    emit_timer: f32,
    /// Number of active particles
    #[serde(skip)]
    active_count: usize,
}

impl ParticleEmitter {
    /// Create a new particle emitter
    pub fn new(max_particles: usize, emission_rate: f32) -> Self {
        Self {
            max_particles,
            emission_rate,
            emit_timer: 0.0,
            active_count: 0,
        }
    }

    /// Check if emitter can emit more particles
    pub fn can_emit(&self) -> bool {
        self.active_count < self.max_particles
    }

    /// Get number of active particles
    pub fn active_count(&self) -> usize {
        self.active_count
    }

    /// Increment active count (called when particle is spawned)
    pub fn increment_active(&mut self) {
        self.active_count = self.active_count.saturating_add(1);
    }

    /// Decrement active count (called when particle dies)
    pub fn decrement_active(&mut self) {
        self.active_count = self.active_count.saturating_sub(1);
    }

    /// Update emission timer
    pub fn update(&mut self, delta: f32) -> bool {
        self.emit_timer += delta;
        let emission_interval = 1.0 / self.emission_rate;

        if self.emit_timer >= emission_interval {
            self.emit_timer -= emission_interval;
            true
        } else {
            false
        }
    }
}

/// Configuration for particle emission
///
/// Defines initial properties of spawned particles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmitterConfig {
    /// Emitter position in world space
    pub position: (f32, f32, f32),
    /// Velocity range (min_x, min_y, min_z, max_x, max_y, max_z)
    pub velocity_range: (f32, f32, f32, f32, f32, f32),
    /// Particle lifetime in seconds
    pub lifetime: f32,
    /// Initial particle size
    pub size: f32,
    /// Initial particle color
    pub color: (f32, f32, f32, f32),
    /// Optional acceleration (e.g., gravity)
    pub acceleration: Option<(f32, f32, f32)>,
    /// Size animation (initial, target)
    pub size_animation: Option<(f32, f32)>,
}

impl Default for EmitterConfig {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0, 0.0),
            velocity_range: (-10.0, 50.0, -10.0, 10.0, 100.0, 10.0),
            lifetime: 2.0,
            size: 5.0,
            color: (1.0, 0.5, 0.0, 1.0),
            acceleration: None,
            size_animation: None,
        }
    }
}

/// Bundle of components for a new particle
///
/// Provides convenient spawning with all required components
#[derive(Debug, Clone)]
pub struct ParticleBundle {
    /// Position component
    pub position: ParticlePosition,
    /// Velocity component
    pub velocity: ParticleVelocity,
    /// Optional acceleration
    pub acceleration: Option<ParticleAcceleration>,
    /// Lifetime component
    pub lifetime: ParticleLifetime,
    /// Size component
    pub size: ParticleSize,
    /// Color component
    pub color: ParticleColor,
}

impl ParticleBundle {
    /// Create a new particle bundle from emitter config
    pub fn from_config(config: &EmitterConfig, rng: &mut impl rand::Rng) -> Self {
        // Random velocity within range
        let vx = rng.gen_range(config.velocity_range.0..config.velocity_range.3);
        let vy = rng.gen_range(config.velocity_range.1..config.velocity_range.4);
        let vz = rng.gen_range(config.velocity_range.2..config.velocity_range.5);

        let size = if let Some((initial, target)) = config.size_animation {
            ParticleSize::with_animation(initial, target)
        } else {
            ParticleSize::new(config.size)
        };

        Self {
            position: ParticlePosition::from_tuple(config.position),
            velocity: ParticleVelocity::new(vx, vy, vz),
            acceleration: config
                .acceleration
                .map(|(x, y, z)| ParticleAcceleration { x, y, z }),
            lifetime: ParticleLifetime::new(config.lifetime),
            size,
            color: ParticleColor::from_rgba(config.color),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === ParticlePosition Tests ===

    #[test]
    fn test_particle_position_new() {
        let pos = ParticlePosition::new(1.0, 2.0, 3.0);
        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 2.0);
        assert_eq!(pos.z, 3.0);
    }

    #[test]
    fn test_particle_position_from_tuple() {
        let pos = ParticlePosition::from_tuple((1.0, 2.0, 3.0));
        assert_eq!(pos.as_tuple(), (1.0, 2.0, 3.0));
    }

    #[test]
    fn test_particle_position_default() {
        let pos = ParticlePosition::default();
        assert_eq!(pos.x, 0.0);
        assert_eq!(pos.y, 0.0);
        assert_eq!(pos.z, 0.0);
    }

    // === ParticleVelocity Tests ===

    #[test]
    fn test_particle_velocity_new() {
        let vel = ParticleVelocity::new(1.0, 2.0, 3.0);
        assert_eq!(vel.x, 1.0);
        assert_eq!(vel.y, 2.0);
        assert_eq!(vel.z, 3.0);
    }

    #[test]
    fn test_particle_velocity_zero() {
        let vel = ParticleVelocity::zero();
        assert_eq!(vel.magnitude(), 0.0);
    }

    #[test]
    fn test_particle_velocity_magnitude() {
        let vel = ParticleVelocity::new(3.0, 4.0, 0.0);
        assert!((vel.magnitude() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_particle_velocity_normalize() {
        let vel = ParticleVelocity::new(3.0, 4.0, 0.0);
        let normalized = vel.normalize();
        assert!((normalized.magnitude() - 1.0).abs() < 0.001);
    }

    // === ParticleAcceleration Tests ===

    #[test]
    fn test_particle_acceleration_gravity() {
        let acc = ParticleAcceleration::gravity();
        assert_eq!(acc.x, 0.0);
        assert_eq!(acc.y, -9.81);
        assert_eq!(acc.z, 0.0);
    }

    #[test]
    fn test_particle_acceleration_zero() {
        let acc = ParticleAcceleration::zero();
        assert_eq!(acc.x, 0.0);
        assert_eq!(acc.y, 0.0);
        assert_eq!(acc.z, 0.0);
    }

    // === ParticleLifetime Tests ===

    #[test]
    fn test_particle_lifetime_new() {
        let lifetime = ParticleLifetime::new(5.0);
        assert_eq!(lifetime.age, 0.0);
        assert_eq!(lifetime.max_age, 5.0);
    }

    #[test]
    fn test_particle_lifetime_is_dead() {
        let mut lifetime = ParticleLifetime::new(1.0);
        assert!(!lifetime.is_dead());
        lifetime.age = 1.0;
        assert!(lifetime.is_dead());
    }

    #[test]
    fn test_particle_lifetime_remaining() {
        let mut lifetime = ParticleLifetime::new(5.0);
        assert_eq!(lifetime.remaining(), 5.0);
        lifetime.age = 2.0;
        assert_eq!(lifetime.remaining(), 3.0);
    }

    #[test]
    fn test_particle_lifetime_progress() {
        let mut lifetime = ParticleLifetime::new(2.0);
        assert_eq!(lifetime.progress(), 0.0);
        lifetime.age = 1.0;
        assert_eq!(lifetime.progress(), 0.5);
        lifetime.age = 2.0;
        assert_eq!(lifetime.progress(), 1.0);
    }

    // === ParticleSize Tests ===

    #[test]
    fn test_particle_size_new() {
        let size = ParticleSize::new(10.0);
        assert_eq!(size.current, 10.0);
        assert_eq!(size.target, 10.0);
    }

    #[test]
    fn test_particle_size_with_animation() {
        let size = ParticleSize::with_animation(5.0, 10.0);
        assert_eq!(size.current, 5.0);
        assert_eq!(size.target, 10.0);
    }

    #[test]
    fn test_particle_size_update() {
        let mut size = ParticleSize::with_animation(0.0, 10.0);
        size.update(0.5, 2.0);
        assert_eq!(size.current, 10.0); // Full interpolation in 0.5s at speed 2.0
    }

    // === ParticleColor Tests ===

    #[test]
    fn test_particle_color_new() {
        let color = ParticleColor::new(1.0, 0.5, 0.0, 1.0);
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.5);
        assert_eq!(color.b, 0.0);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn test_particle_color_white() {
        let color = ParticleColor::white();
        assert_eq!(color.as_rgba(), (1.0, 1.0, 1.0, 1.0));
    }

    #[test]
    fn test_particle_color_fade() {
        let mut color = ParticleColor::new(1.0, 1.0, 1.0, 1.0);
        color.fade(0.5, 1.0);
        assert_eq!(color.a, 0.5);
    }

    // === ParticleEmitter Tests ===

    #[test]
    fn test_particle_emitter_new() {
        let emitter = ParticleEmitter::new(1000, 100.0);
        assert_eq!(emitter.max_particles, 1000);
        assert_eq!(emitter.emission_rate, 100.0);
        assert_eq!(emitter.active_count(), 0);
    }

    #[test]
    fn test_particle_emitter_can_emit() {
        let mut emitter = ParticleEmitter::new(10, 100.0);
        assert!(emitter.can_emit());

        for _ in 0..10 {
            emitter.increment_active();
        }
        assert!(!emitter.can_emit());
    }

    #[test]
    fn test_particle_emitter_active_count() {
        let mut emitter = ParticleEmitter::new(100, 100.0);
        emitter.increment_active();
        emitter.increment_active();
        emitter.increment_active();
        assert_eq!(emitter.active_count(), 3);

        emitter.decrement_active();
        assert_eq!(emitter.active_count(), 2);
    }

    #[test]
    fn test_particle_emitter_update() {
        let mut emitter = ParticleEmitter::new(100, 10.0); // 10 particles per second
        assert!(!emitter.update(0.05)); // 50ms - not enough
        assert!(emitter.update(0.05)); // 100ms more - should emit
    }

    // === EmitterConfig Tests ===

    #[test]
    fn test_emitter_config_default() {
        let config = EmitterConfig::default();
        assert_eq!(config.position, (0.0, 0.0, 0.0));
        assert_eq!(config.lifetime, 2.0);
        assert_eq!(config.size, 5.0);
    }
}
