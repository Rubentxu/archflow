// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Animation Actuator
//
// This actuator manages entity animations using easing functions:
// - Connects sensors to animations (click → animate)
// - Generates Command::Teleport, Command::Resize based on animation results
//
// Performance Characteristics:
// - O(1) animation lookup by entity ID
// - Batch processing for multiple animations
//
// Memory Impact:
// - One HashMap entry per animated entity
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::vec;
use alloc::vec::Vec;

use archflow_core::{EntityId, Vec2};
use archflow_engine::{Command, EntityStore};

use crate::tween::Easing;

/// Animation actuator that manages entity animations in response to sensor signals
pub struct AnimationActuator {
    /// Active animations by entity ID
    animations: hashbrown::HashMap<EntityId, AnimationState>,
}

/// State for an entity's animation
#[derive(Clone, Debug)]
struct AnimationState {
    /// Target position (if animating position)
    target_pos: Vec2,
    /// Target size (if animating size)
    target_size: Option<Vec2>,
    /// Target opacity (if animating opacity)
    target_opacity: Option<f32>,
    /// Duration in milliseconds
    duration_ms: u32,
    /// Elapsed time in milliseconds
    elapsed_ms: u32,
    /// Start position (cached for interpolation)
    start_pos: Vec2,
    /// Start size (cached for interpolation)
    start_size: Vec2,
    /// Easing function
    easing: Easing,
    /// Animation type
    anim_type: AnimationType,
}

/// Type of animation
#[derive(Clone, Copy, Debug, PartialEq)]
enum AnimationType {
    /// Move to position
    Move,
    /// Resize to size
    Resize,
    /// Fade opacity
    Fade,
    /// Move + Fade
    MoveAndFade,
}

impl AnimationActuator {
    /// Creates a new AnimationActuator
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            animations: hashbrown::HashMap::new(),
        }
    }

    /// Animate an entity to a target position
    pub fn animate_to(
        &mut self,
        entity_id: EntityId,
        to: Vec2,
        duration_ms: u32,
        easing: Easing,
        store: &EntityStore,
    ) -> Vec<Command> {
        let idx = entity_id.index().0 as usize;
        let from = store.pos(idx);

        self.animations.insert(
            entity_id,
            AnimationState {
                target_pos: to,
                target_size: None,
                target_opacity: None,
                duration_ms,
                elapsed_ms: 0,
                start_pos: from,
                start_size: store.size(idx),
                easing,
                anim_type: AnimationType::Move,
            },
        );

        vec![Command::Teleport {
            id: entity_id,
            pos: from,
        }]
    }

    /// Animate an entity's size
    pub fn animate_size(
        &mut self,
        entity_id: EntityId,
        to: Vec2,
        duration_ms: u32,
        easing: Easing,
        store: &EntityStore,
    ) -> Vec<Command> {
        let idx = entity_id.index().0 as usize;
        let from = store.size(idx);

        self.animations.insert(
            entity_id,
            AnimationState {
                target_pos: Vec2::ZERO,
                target_size: Some(to),
                target_opacity: None,
                duration_ms,
                elapsed_ms: 0,
                start_pos: Vec2::ZERO,
                start_size: from,
                easing,
                anim_type: AnimationType::Resize,
            },
        );

        vec![Command::Resize {
            id: entity_id,
            size: from,
        }]
    }

    /// Stop animation for an entity
    pub fn stop(&mut self, entity_id: EntityId) -> bool {
        self.animations.remove(&entity_id).is_some()
    }

    /// Check if entity is animating
    #[inline(always)]
    #[must_use]
    pub fn is_animating(&self, entity_id: EntityId) -> bool {
        self.animations.contains_key(&entity_id)
    }

    /// Get number of active animations
    #[inline(always)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.animations.len()
    }

    /// Check if no animations are active
    #[inline(always)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.animations.is_empty()
    }

    /// Update all animations - call this every frame
    ///
    /// Returns commands to apply to entity store
    pub fn update(&mut self, delta_ms: u32, _store: &mut EntityStore) -> Vec<Command> {
        let mut commands = Vec::new();
        let mut completed: Vec<EntityId> = Vec::new();

        for (entity_id, state) in &mut self.animations {
            // Update elapsed time
            state.elapsed_ms = state.elapsed_ms.saturating_add(delta_ms);

            // Calculate progress
            let progress = if state.duration_ms > 0 {
                (state.elapsed_ms as f32 / state.duration_ms as f32).min(1.0)
            } else {
                1.0
            };

            // Apply easing
            let eased = state.easing.apply(progress);

            // Generate commands based on animation type
            match state.anim_type {
                AnimationType::Move => {
                    let current_x =
                        state.start_pos.x + (state.target_pos.x - state.start_pos.x) * eased;
                    let current_y =
                        state.start_pos.y + (state.target_pos.y - state.start_pos.y) * eased;
                    commands.push(Command::Teleport {
                        id: *entity_id,
                        pos: Vec2::new(current_x, current_y),
                    });
                }
                AnimationType::Resize => {
                    if let Some(target_size) = state.target_size {
                        let current_x =
                            state.start_size.x + (target_size.x - state.start_size.x) * eased;
                        let current_y =
                            state.start_size.y + (target_size.y - state.start_size.y) * eased;
                        commands.push(Command::Resize {
                            id: *entity_id,
                            size: Vec2::new(current_x, current_y),
                        });
                    }
                }
                AnimationType::Fade | AnimationType::MoveAndFade => {
                    // Opacity not yet implemented
                }
            }

            // Mark complete when progress reaches 1.0
            if progress >= 1.0 {
                completed.push(*entity_id);
            }
        }

        // Remove completed animations
        for entity_id in completed {
            self.animations.remove(&entity_id);
        }

        commands
    }
}

impl Default for AnimationActuator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_actuator_basic() {
        let actuator = AnimationActuator::new();
        assert!(actuator.is_empty());
    }

    #[test]
    fn test_animate_to() {
        let mut actuator = AnimationActuator::new();
        let entity_id = EntityId::new(1);

        let store = EntityStore::new();
        let _ = actuator.animate_to(
            entity_id,
            Vec2::new(100.0, 200.0),
            500,
            Easing::QuadInOut,
            &store,
        );

        assert!(actuator.is_animating(entity_id));
        assert!(actuator.stop(entity_id));
        assert!(!actuator.is_animating(entity_id));
    }
}
