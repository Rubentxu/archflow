// ═══════════════════════════════════════════════════════════════════════════════
// AnimationSystem - ECS System for Sprite Animation
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::vec::Vec;

use crate::ecs::{System, World};

use super::AnimationComponent;

// ═══════════════════════════════════════════════════════════════════════════════
// AnimationSystem
// ═══════════════════════════════════════════════════════════════════════════════

/// Statistics for animation system
#[derive(Clone, Debug, Default)]
pub struct AnimationStats {
    /// Number of entities with animations
    pub animated: usize,
    /// Number of frame changes this tick
    pub frame_changes: usize,
}

/// ECS System that updates animations based on delta time
///
/// This system:
/// 1. Queries entities with AnimationComponent
/// 2. Updates current_frame based on elapsed time
/// 3. Handles looping and single-shot animations
#[derive(Clone, Debug)]
pub struct AnimationSystem {
    /// Statistics
    stats: AnimationStats,
}

impl AnimationSystem {
    /// Creates a new AnimationSystem
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            stats: AnimationStats::default(),
        }
    }

    /// Returns statistics
    #[inline]
    #[must_use]
    pub fn stats(&self) -> &AnimationStats {
        &self.stats
    }

    /// Reset statistics
    #[inline]
    pub fn reset_stats(&mut self) {
        self.stats = AnimationStats::default();
    }
}

impl Default for AnimationSystem {
    fn default() -> Self {
        Self::new()
    }
}

// Implement System trait for AnimationSystem
impl System for AnimationSystem {
    /// Returns the system name
    #[inline]
    fn name(&self) -> &str {
        "AnimationSystem"
    }

    /// Returns the system priority (runs before render, after physics)
    /// Priority 50 = before ShapeRenderSystem (100)
    #[inline]
    fn priority(&self) -> i32 {
        50
    }

    /// Runs the animation system
    ///
    /// Updates all animations based on delta time
    fn run(&mut self, world: &mut World, delta_time: f32) {
        // Reset stats
        self.stats = AnimationStats::default();

        // Convert delta_time (seconds) to milliseconds
        let delta_ms = (delta_time * 1000.0) as u64;

        if delta_ms == 0 {
            return;
        }

        // Get all entity IDs with AnimationComponent
        let entity_ids: Vec<_> = world
            .entities()
            .filter(|e| world.has_component::<AnimationComponent>(*e))
            .collect();

        self.stats.animated = entity_ids.len();

        // Update each animation
        for entity_id in entity_ids {
            if let Some(anim) = world.get_component_mut::<AnimationComponent>(entity_id) {
                if let Some(new_frame) = anim.tick(delta_ms) {
                    // Frame changed
                    self.stats.frame_changes += 1;
                    // In a full implementation, this would trigger a frame update event
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::{Component, World};

    #[test]
    fn test_animation_system_creation() {
        let system = AnimationSystem::new();
        assert_eq!(system.stats().animated, 0);
    }

    #[test]
    fn test_animation_system_name() {
        let system = AnimationSystem::new();
        assert_eq!(system.name(), "AnimationSystem");
    }

    #[test]
    fn test_animation_system_priority() {
        let system = AnimationSystem::new();
        assert_eq!(system.priority(), 50);
    }

    #[test]
    fn test_animation_tick() {
        let mut anim = AnimationComponent::new(4, 100); // 4 frames, 100ms each

        // Initially not playing
        assert_eq!(anim.tick(50), None);

        // Start playing
        anim.play();

        // First frame should advance after 100ms
        assert_eq!(anim.tick(100), Some(1));
        assert_eq!(anim.current(), 1);

        // Second frame after another 100ms
        assert_eq!(anim.tick(100), Some(2));
    }

    #[test]
    fn test_animation_loop() {
        let mut anim = AnimationComponent::new(3, 50); // 3 frames, 50ms each
        anim.play();

        // Progress through frames
        assert_eq!(anim.tick(50), Some(1));
        assert_eq!(anim.tick(50), Some(2));
        assert_eq!(anim.tick(50), Some(0)); // Loop back to 0
        assert_eq!(anim.tick(50), Some(1));
    }

    #[test]
    fn test_animation_single_shot() {
        let mut anim = AnimationComponent::new_single_shot(3, 50);
        anim.play();

        // Progress through frames
        assert_eq!(anim.tick(50), Some(1));
        assert_eq!(anim.tick(50), Some(2));

        // At end - should stop playing
        assert_eq!(anim.tick(50), Some(2)); // Stays at 2
        assert!(!anim.is_playing); // Stopped
    }
}
