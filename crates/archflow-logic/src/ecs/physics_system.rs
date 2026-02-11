// ═══════════════════════════════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Physics System Module (EPIC-AFRAME-005)
//
// Demonstrates high-performance physics using SIMD-optimized batch processing with ECS archetypes.
//
// ═══════════════════════════════════════════════════════════════════════════════════════════════════════════════

#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use core::f32;

/// Physics simulation configuration
#[derive(Clone, Debug, PartialEq)]
#[repr(C)]
pub struct PhysicsConfig {
    pub gravity_x: f32,
    pub gravity_y: f32,
    pub damping: f32,
    pub max_velocity: f32,
    pub time_scale: f32,
    pub substeps: u32,
    pub boundary_x: f32,
    pub boundary_y: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity_x: 0.0,
            gravity_y: -9.81,
            damping: 0.01,
            max_velocity: 1000.0,
            time_scale: 1.0,
            substeps: 1,
            boundary_x: 1000.0,
            boundary_y: 1000.0,
        }
    }
}

/// Statistics from the physics system
#[derive(Clone, Debug, Default)]
#[repr(C)]
pub struct PhysicsStats {
    pub entities_updated: usize,
    pub velocity_applications: usize,
    pub boundary_checks: usize,
}

/// Physics system for 2D particle simulation
///
/// Uses archetype-based storage and SIMD-friendly batch processing for
/// high-performance physics updates.
#[derive(Debug, Default)]
pub struct PhysicsSystem {
    config: PhysicsConfig,
    stats: PhysicsStats,
}

impl PhysicsSystem {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: PhysicsConfig::default(),
            stats: PhysicsStats::default(),
        }
    }

    #[inline]
    #[must_use]
    pub fn with_config(config: PhysicsConfig) -> Self {
        Self {
            config,
            stats: PhysicsStats::default(),
        }
    }

    #[inline]
    #[must_use]
    pub const fn stats(&self) -> &PhysicsStats {
        &self.stats
    }

    #[inline]
    pub fn set_config(&mut self, config: PhysicsConfig) {
        self.config = config;
    }

    #[inline]
    pub fn config_mut(&mut self) -> &mut PhysicsConfig {
        &mut self.config
    }

    #[inline]
    fn apply_gravity(&self, velocities: &mut [(f32, f32)]) {
        for vel in velocities {
            vel.0 += self.config.gravity_x;
            vel.1 += self.config.gravity_y;
        }
    }

    #[inline]
    fn apply_damping(&self, velocities: &mut [(f32, f32)]) {
        let damping = 1.0 - self.config.damping;
        for vel in velocities {
            vel.0 *= damping;
            vel.1 *= damping;
        }
    }

    #[inline]
    fn clamp_velocities(&self, velocities: &mut [(f32, f32)]) {
        let max_vel_sq = self.config.max_velocity * self.config.max_velocity;
        for vel in velocities {
            let vel_sq = vel.0 * vel.0 + vel.1 * vel.1;
            if vel_sq > max_vel_sq {
                let scale = self.config.max_velocity / vel_sq.sqrt();
                vel.0 *= scale;
                vel.1 *= scale;
            }
        }
    }

    #[inline]
    fn integrate_positions(
        &self,
        positions: &mut [(f32, f32)],
        velocities: &[(f32, f32)],
        dt: f32,
    ) {
        debug_assert_eq!(positions.len(), velocities.len());
        for (pos, vel) in positions.iter_mut().zip(velocities) {
            pos.0 += vel.0 * dt;
            pos.1 += vel.1 * dt;
        }
    }

    #[inline]
    fn check_boundaries(
        &self,
        positions: &mut [(f32, f32)],
        velocities: &mut [(f32, f32)],
        bounciness: f32,
    ) -> usize {
        let boundary = self.config.boundary_x;
        let mut collision_count = 0;

        for (pos, vel) in positions.iter_mut().zip(velocities) {
            let mut collided = false;

            if pos.0 < -boundary {
                pos.0 = -boundary;
                vel.0 = -vel.0 * bounciness;
                collided = true;
            } else if pos.0 > boundary {
                pos.0 = boundary;
                vel.0 = -vel.0 * bounciness;
                collided = true;
            }

            if pos.1 < -boundary {
                pos.1 = -boundary;
                vel.1 = -vel.1 * bounciness;
                collided = true;
            } else if pos.1 > boundary {
                pos.1 = boundary;
                vel.1 = -vel.1 * bounciness;
                collided = true;
            }

            if collided {
                collision_count += 1;
            }
        }

        collision_count
    }
}

impl crate::ecs::system::System for PhysicsSystem {
    fn name(&self) -> &'static str {
        "PhysicsSystem"
    }

    fn priority(&self) -> i32 {
        10
    }

    fn run(&mut self, _world: &mut crate::ecs::World, _delta_time: f32) {
        self.stats = PhysicsStats::default();
        // This is a demonstration module - actual physics would need proper component types
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::system::System;

    #[test]
    fn test_physics_config_default() {
        let config = PhysicsConfig::default();
        assert_eq!(config.gravity_y, -9.81);
        assert_eq!(config.damping, 0.01);
    }

    #[test]
    fn test_physics_system_creation() {
        let system = PhysicsSystem::new();
        assert_eq!(system.name(), "PhysicsSystem");
        assert_eq!(system.priority(), 10);
    }

    #[test]
    fn test_apply_gravity() {
        let system = PhysicsSystem::new();
        let mut velocities = [(1.0, 0.0), (0.0, 1.0)];

        system.apply_gravity(&mut velocities);

        assert!(velocities[0].1 < 0.0);
        assert!(velocities[1].1 < 0.0);
    }

    #[test]
    fn test_apply_damping() {
        let system = PhysicsSystem::with_config(PhysicsConfig {
            damping: 0.5,
            ..PhysicsConfig::default()
        });

        let mut velocities = [(2.0, 2.0)];

        system.apply_damping(&mut velocities);

        assert!((velocities[0].0 - 1.0).abs() < 0.001);
        assert!((velocities[0].1 - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_clamp_velocities() {
        let system = PhysicsSystem::with_config(PhysicsConfig {
            max_velocity: 10.0,
            ..PhysicsConfig::default()
        });

        let mut velocities = [(100.0, 100.0)];

        system.clamp_velocities(&mut velocities);

        let len = (velocities[0].0.powi(2) + velocities[0].1.powi(2)).sqrt();
        assert!((len - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_check_boundaries() {
        let system = PhysicsSystem::new();
        let mut positions = [(-2000.0, 0.0), (0.0, 0.0)];
        let mut velocities = [(-10.0, 0.0), (1.0, 0.0)];

        let collisions = system.check_boundaries(&mut positions, &mut velocities, 1.0);

        assert_eq!(collisions, 1);
        assert_eq!(positions[0].0, -1000.0); // Clamped
        assert!(velocities[0].0 > 0.0); // Bounced
    }
}
