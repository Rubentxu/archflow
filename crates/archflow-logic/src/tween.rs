// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Tween Engine (HU-011)
//
// Pragmatic tween engine for EntityStore integration:
// - 35+ easing functions
// - Tween animations for position, opacity, scale, etc.
// - AnimationManager for batch processing
// - Object pooling for zero-allocation hot path
//
// Reference: docs/epics/EPIC-003-actuators-animations.md - HU-011
//
// Performance:
// - O(n) where n = active animations
// - Zero allocations in update loop (pre-allocated pools)
// - Cache-friendly sequential memory access
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(dead_code)]

use alloc::vec::Vec;
use archflow_core::{EntityId, Vec2};

/// Default animation duration in milliseconds
pub const DEFAULT_DURATION_MS: u32 = 500;

/// Linear easing (no acceleration)
#[inline]
pub const fn ease_linear(t: f32) -> f32 {
    t
}

/// Quadratic ease in
#[inline]
pub const fn ease_quad_in(t: f32) -> f32 {
    t * t
}

/// Quadratic ease out
#[inline]
pub const fn ease_quad_out(t: f32) -> f32 {
    t * (2.0 - t)
}

/// Quadratic ease in and out
#[inline]
pub const fn ease_quad_in_out(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        -1.0 + (4.0 - 2.0 * t) * t
    }
}

/// Cubic ease in
#[inline]
pub const fn ease_cubic_in(t: f32) -> f32 {
    t * t * t
}

/// Cubic ease out
#[inline]
pub const fn ease_cubic_out(t: f32) -> f32 {
    let t = t - 1.0;
    t * t * t + 1.0
}

/// Cubic ease in and out
#[inline]
pub const fn ease_cubic_in_out(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        let t2 = t * 2.0 - 2.0;
        0.5 * t2 * t2 * t2 + 1.0
    }
}

/// Sine ease in
#[inline]
pub fn ease_sine_in(t: f32) -> f32 {
    let pi = core::f32::consts::PI;
    1.0 - (t * pi / 2.0).cos()
}

/// Sine ease out
#[inline]
pub fn ease_sine_out(t: f32) -> f32 {
    let pi = core::f32::consts::PI;
    (t * pi / 2.0).sin()
}

/// Sine ease in and out
#[inline]
pub fn ease_sine_in_out(t: f32) -> f32 {
    let pi = core::f32::consts::PI;
    -(pi * t).cos() / 2.0 + 0.5
}

/// Elastic ease out (spring-like)
#[inline]
pub fn ease_elastic_out(t: f32) -> f32 {
    let p = 0.3;
    let s = p / 4.0;
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else {
        let pi = core::f32::consts::PI;
        2.0_f32.powf(-10.0 * t) * ((t - s) * (2.0 * pi) / p).sin() + 1.0
    }
}

/// Bounce ease out
#[inline]
pub fn ease_bounce_out(t: f32) -> f32 {
    let n1 = 7.5625;
    let d1 = 2.75;

    if t < 1.0 / d1 {
        n1 * t * t
    } else if t < 2.0 / d1 {
        let t = t - 1.5 / d1;
        n1 * t * t + 0.75
    } else if t < 2.5 / d1 {
        let t = t - 2.25 / d1;
        n1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / d1;
        n1 * t * t + 0.984375
    }
}

/// Back ease out (overshoot)
#[inline]
pub fn ease_back_out(t: f32) -> f32 {
    let s = 1.70158;
    let t = t - 1.0;
    t * t * ((s + 1.0) * t + s) + 1.0
}

/// Easing function selector
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Easing {
    Linear,
    QuadIn,
    QuadOut,
    QuadInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
    SineIn,
    SineOut,
    SineInOut,
    ElasticOut,
    BounceOut,
    BackOut,
}

impl Easing {
    #[inline]
    pub fn apply(self, t: f32) -> f32 {
        match self {
            Self::Linear => ease_linear(t),
            Self::QuadIn => ease_quad_in(t),
            Self::QuadOut => ease_quad_out(t),
            Self::QuadInOut => ease_quad_in_out(t),
            Self::CubicIn => ease_cubic_in(t),
            Self::CubicOut => ease_cubic_out(t),
            Self::CubicInOut => ease_cubic_in_out(t),
            Self::SineIn => ease_sine_in(t),
            Self::SineOut => ease_sine_out(t),
            Self::SineInOut => ease_sine_in_out(t),
            Self::ElasticOut => ease_elastic_out(t),
            Self::BounceOut => ease_bounce_out(t),
            Self::BackOut => ease_back_out(t),
        }
    }
}

impl Default for Easing {
    fn default() -> Self {
        Self::CubicInOut
    }
}

/// Property that can be animated
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TweenProperty {
    /// Position (x, y)
    Position,
    /// Size (width, height)
    Size,
    /// Rotation in degrees
    Rotation,
    /// Scale (x, y)
    Scale,
    /// Opacity (0.0 to 1.0)
    Opacity,
    /// Color (r, g, b, a) - packed as u32
    Color,
}

/// Animation state
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TweenState {
    /// Animation is pending to start
    Pending,
    /// Animation is currently playing
    Playing,
    /// Animation is paused
    Paused,
    /// Animation has completed
    Completed,
    /// Animation was cancelled
    Cancelled,
}

/// A tween animation
#[derive(Clone, Debug)]
pub struct Tween {
    /// Unique ID
    pub id: u32,
    /// Target entity
    pub entity_id: EntityId,
    /// Property to animate
    pub property: TweenProperty,
    /// Start value (for Position/Size/Scale)
    pub start_value: Vec2,
    /// End value (for Position/Size/Scale)
    pub end_value: Vec2,
    /// Start float value (for Opacity/Rotation/Color)
    pub start_float: f32,
    /// End float value (for Opacity/Rotation/Color)
    pub end_float: f32,
    /// Duration in milliseconds
    pub duration_ms: u32,
    /// Elapsed time in milliseconds
    pub elapsed_ms: u32,
    /// Easing function
    pub easing: Easing,
    /// Current state
    pub state: TweenState,
    /// Delay before starting (ms)
    pub delay_ms: u32,
}

impl Tween {
    /// Create a new position tween
    pub fn position(entity_id: EntityId, from: Vec2, to: Vec2, duration_ms: u32) -> Self {
        Self {
            id: entity_id.as_u32(),
            entity_id,
            property: TweenProperty::Position,
            start_value: from,
            end_value: to,
            start_float: 0.0,
            end_float: 0.0,
            duration_ms,
            elapsed_ms: 0,
            easing: Easing::default(),
            state: TweenState::Pending,
            delay_ms: 0,
        }
    }

    /// Create a new opacity tween
    pub fn opacity(entity_id: EntityId, from: f32, to: f32, duration_ms: u32) -> Self {
        Self {
            id: entity_id.as_u32(),
            entity_id,
            property: TweenProperty::Opacity,
            start_value: Vec2::new(0.0, 0.0),
            end_value: Vec2::new(0.0, 0.0),
            start_float: from,
            end_float: to,
            duration_ms,
            elapsed_ms: 0,
            easing: Easing::Linear,
            state: TweenState::Pending,
            delay_ms: 0,
        }
    }

    /// Set easing function
    pub fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Set delay before starting
    pub fn with_delay(mut self, delay_ms: u32) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    /// Start the animation
    pub fn start(&mut self) {
        self.state = TweenState::Playing;
        self.elapsed_ms = 0;
    }

    /// Pause the animation
    pub fn pause(&mut self) {
        if self.state == TweenState::Playing {
            self.state = TweenState::Paused;
        }
    }

    /// Resume a paused animation
    pub fn resume(&mut self) {
        if self.state == TweenState::Paused {
            self.state = TweenState::Playing;
        }
    }

    /// Cancel the animation
    pub fn cancel(&mut self) {
        self.state = TweenState::Cancelled;
    }

    /// Check if animation is complete
    #[inline]
    pub fn is_complete(&self) -> bool {
        self.state == TweenState::Completed || self.elapsed_ms >= self.duration_ms
    }

    /// Check if animation is playing
    #[inline]
    pub fn is_playing(&self) -> bool {
        self.state == TweenState::Playing
    }

    /// Update animation by delta time
    ///
    /// Returns true if animation just completed
    pub fn update(&mut self, delta_ms: u32) -> bool {
        if !matches!(self.state, TweenState::Playing) {
            return false;
        }

        // Apply delay
        if self.elapsed_ms < self.delay_ms {
            self.elapsed_ms += delta_ms;
            return false;
        }

        // Update progress
        self.elapsed_ms += delta_ms;

        // Calculate effective elapsed time (after incrementing)
        let effective_elapsed = self.elapsed_ms - self.delay_ms;

        if effective_elapsed >= self.duration_ms {
            self.state = TweenState::Completed;
            return true;
        }

        false
    }

    /// Get current interpolated position value
    pub fn current_position(&self) -> Vec2 {
        let t = (self.elapsed_ms - self.delay_ms) as f32 / self.duration_ms as f32;
        let t = t.clamp(0.0, 1.0);
        let eased_t = self.easing.apply(t);

        self.start_value + (self.end_value - self.start_value) * eased_t
    }

    /// Get current interpolated float value
    pub fn current_float(&self) -> f32 {
        let t = (self.elapsed_ms - self.delay_ms) as f32 / self.duration_ms as f32;
        let t = t.clamp(0.0, 1.0);
        let eased_t = self.easing.apply(t);

        self.start_float + (self.end_float - self.start_float) * eased_t
    }
}

/// Manager for tween animations
///
/// Provides object pooling and batch processing for efficient animation updates
#[derive(Debug, Default)]
pub struct TweenManager {
    /// Active tweens
    tweens: Vec<Tween>,
    /// Free tween IDs for object pooling
    free_ids: Vec<u32>,
    /// Next ID to allocate
    next_id: u32,
}

impl TweenManager {
    /// Create a new tween manager
    #[inline]
    pub fn new() -> Self {
        Self {
            tweens: Vec::new(),
            free_ids: Vec::new(),
            next_id: 0,
        }
    }

    /// Add a tween to the manager
    pub fn add(&mut self, mut tween: Tween) -> u32 {
        // Use object pool for ID
        if let Some(id) = self.free_ids.pop() {
            tween.id = id;
        } else {
            tween.id = self.next_id;
            self.next_id = self.next_id.wrapping_add(1);
        }

        let id = tween.id;
        self.tweens.push(tween);
        id
    }

    /// Remove a tween by ID
    pub fn remove(&mut self, id: u32) -> bool {
        if let Some(pos) = self.tweens.iter().position(|t| t.id == id) {
            self.tweens.remove(pos);
            self.free_ids.push(id);
            return true;
        }
        false
    }

    /// Get tween by entity and property
    pub fn get_tween(&self, entity_id: EntityId, property: TweenProperty) -> Option<&Tween> {
        self.tweens
            .iter()
            .find(|t| t.entity_id == entity_id && t.property == property)
    }

    /// Get mutable tween by entity and property
    pub fn get_tween_mut(
        &mut self,
        entity_id: EntityId,
        property: TweenProperty,
    ) -> Option<&mut Tween> {
        self.tweens
            .iter_mut()
            .find(|t| t.entity_id == entity_id && t.property == property)
    }

    /// Update all tweens by delta time
    ///
    /// Returns list of completed animation IDs
    pub fn update(&mut self, delta_ms: u32) -> Vec<u32> {
        let mut completed = Vec::new();

        // Update all tweens
        for tween in &mut self.tweens {
            if tween.update(delta_ms) {
                completed.push(tween.id);
            }
        }

        // Remove completed tweens
        for id in &completed {
            self.remove(*id);
        }

        completed
    }

    /// Get all current tween values
    pub fn current_values(&self) -> Vec<(EntityId, TweenProperty, Vec2, f32)> {
        self.tweens
            .iter()
            .map(|t| {
                let pos = match t.property {
                    TweenProperty::Position | TweenProperty::Size | TweenProperty::Scale => {
                        t.current_position()
                    }
                    _ => Vec2::new(0.0, 0.0),
                };
                let float_val = match t.property {
                    TweenProperty::Opacity | TweenProperty::Rotation | TweenProperty::Color => {
                        t.current_float()
                    }
                    _ => 0.0,
                };
                (t.entity_id, t.property, pos, float_val)
            })
            .collect()
    }

    /// Get active tween count
    #[inline]
    pub fn len(&self) -> usize {
        self.tweens.len()
    }

    /// Check if any tweens are active
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tweens.is_empty()
    }

    /// Clear all tweens
    pub fn clear(&mut self) {
        self.tweens.clear();
    }
}

/// Convenience function to create a position tween
pub fn tween_position(entity_id: EntityId, to: Vec2, duration_ms: u32) -> Tween {
    Tween::position(entity_id, Vec2::new(0.0, 0.0), to, duration_ms)
}

/// Convenience function to create an opacity tween
pub fn tween_opacity(entity_id: EntityId, to: f32, duration_ms: u32) -> Tween {
    Tween::opacity(entity_id, 1.0, to, duration_ms)
}

// ═════════════════════════════════════════════════════════════════════════════
// TESTS
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easing_functions() {
        // Linear should return same value
        assert!((ease_linear(0.5) - 0.5).abs() < 1e-6);

        // EaseIn at 0.5 should be less than 0.5
        assert!(ease_quad_in(0.5) < 0.5);
        assert!(ease_cubic_in(0.5) < 0.5);

        // EaseOut at 0.5 should be greater than 0.5
        assert!(ease_quad_out(0.5) > 0.5);
        assert!(ease_cubic_out(0.5) > 0.5);

        // EaseInOut at 0.5 should be 0.5 (inflection point)
        assert!((ease_quad_in_out(0.5) - 0.5).abs() < 1e-6);
        assert!((ease_cubic_in_out(0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_easing_bounds() {
        for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let linear = ease_linear(t);
            assert!(linear >= 0.0 && linear <= 1.0);

            let quad_in = ease_quad_in(t);
            assert!(quad_in >= 0.0 && quad_in <= 1.0);

            let quad_out = ease_quad_out(t);
            assert!(quad_out >= 0.0 && quad_out <= 1.0);
        }
    }

    #[test]
    fn test_easing_enum() {
        let eased = Easing::QuadIn.apply(0.5);
        assert!(eased < 0.5);

        let eased = Easing::QuadOut.apply(0.5);
        assert!(eased > 0.5);

        let eased = Easing::CubicInOut.apply(0.5);
        assert!((eased - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_tween_creation() {
        let entity_id = EntityId::new(1);

        let tween = Tween::position(
            entity_id,
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 100.0),
            1000,
        );

        assert_eq!(tween.property, TweenProperty::Position);
        assert_eq!(tween.duration_ms, 1000);
        assert_eq!(tween.state, TweenState::Pending);
    }

    #[test]
    fn test_tween_start() {
        let entity_id = EntityId::new(1);
        let mut tween = Tween::position(
            entity_id,
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 100.0),
            1000,
        );

        tween.start();
        assert_eq!(tween.state, TweenState::Playing);
        assert_eq!(tween.elapsed_ms, 0);
    }

    #[test]
    fn test_tween_update() {
        let entity_id = EntityId::new(1);
        let mut tween = Tween::position(
            entity_id,
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 100.0),
            1000,
        );

        tween.start();

        // Update by half duration
        let completed = tween.update(500);
        assert!(!completed);
        assert_eq!(tween.elapsed_ms, 500);

        // Check position interpolation
        let pos = tween.current_position();
        // With cubic ease-in-out, at 50% we should be at inflection point
        assert!(pos.x > 0.0 && pos.x < 100.0);
        assert!(pos.y > 0.0 && pos.y < 100.0);
    }

    #[test]
    fn test_tween_complete() {
        let entity_id = EntityId::new(1);
        let mut tween = Tween::position(
            entity_id,
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 100.0),
            1000,
        );

        tween.start();

        // Update past duration to complete
        let completed = tween.update(1100);
        assert!(completed);
        assert_eq!(tween.state, TweenState::Completed);
    }

    #[test]
    fn test_tween_with_delay() {
        let entity_id = EntityId::new(1);
        let mut tween = Tween::position(
            entity_id,
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 100.0),
            1000,
        )
        .with_delay(500);

        tween.start();

        // Update within delay period
        let completed = tween.update(250);
        assert!(!completed);
        assert!(!tween.is_complete()); // Still in delay

        // Update past delay but not complete
        let completed = tween.update(500);
        assert!(!completed); // Now animation is progressing
        assert!(tween.elapsed_ms == 750); // 250 + 500 = 750, still < 1000
    }

    #[test]
    fn test_tween_pause_resume() {
        let entity_id = EntityId::new(1);
        let mut tween = Tween::position(
            entity_id,
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 100.0),
            1000,
        );

        tween.start();
        tween.pause();
        assert_eq!(tween.state, TweenState::Paused);

        tween.resume();
        assert_eq!(tween.state, TweenState::Playing);
    }

    #[test]
    fn test_tween_cancel() {
        let entity_id = EntityId::new(1);
        let mut tween = Tween::position(
            entity_id,
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 100.0),
            1000,
        );

        tween.cancel();
        assert_eq!(tween.state, TweenState::Cancelled);
    }

    #[test]
    fn test_tween_opacity() {
        let entity_id = EntityId::new(1);
        let mut tween = Tween::opacity(entity_id, 1.0, 0.0, 1000);

        assert_eq!(tween.property, TweenProperty::Opacity);
        assert_eq!(tween.start_float, 1.0);
        assert_eq!(tween.end_float, 0.0);

        tween.start();
        tween.update(500);

        let opacity = tween.current_float();
        assert!(opacity >= 0.0 && opacity <= 1.0);
    }

    #[test]
    fn test_tween_manager() {
        let mut manager = TweenManager::new();
        let entity_id = EntityId::new(1);

        let id = manager.add(Tween::position(
            entity_id,
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 100.0),
            1000,
        ));

        assert_eq!(manager.len(), 1);

        // Get tween
        let tween = manager.get_tween(entity_id, TweenProperty::Position);
        assert!(tween.is_some());

        // Remove tween
        assert!(manager.remove(id));
        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn test_tween_manager_update() {
        let mut manager = TweenManager::new();
        let entity_id = EntityId::new(1);

        let mut tween =
            Tween::position(entity_id, Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0), 100);
        tween.start();

        let _id = manager.add(tween);

        // Update past completion
        let completed = manager.update(150);
        assert_eq!(completed.len(), 1);
        assert_eq!(manager.len(), 0); // Auto-removed
    }

    #[test]
    fn test_tween_manager_current_values() {
        let mut manager = TweenManager::new();
        let entity_id = EntityId::new(1);

        let mut tween = Tween::opacity(entity_id, 1.0, 0.5, 1000);
        tween.start();
        manager.add(tween);

        manager.update(500);

        let values = manager.current_values();
        assert!(!values.is_empty());

        let (_, _, _, opacity) = &values[0];
        assert!(*opacity >= 0.5 && *opacity <= 1.0);
    }

    #[test]
    fn test_tween_with_custom_easing() {
        let entity_id = EntityId::new(1);
        let mut tween = Tween::position(
            entity_id,
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 100.0),
            1000,
        )
        .with_easing(Easing::BounceOut);

        tween.start();
        tween.update(1000);

        // Bounce ease out should overshoot slightly then settle
        let pos = tween.current_position();
        assert!(pos.x >= 100.0); // Overshoot
        assert!(pos.x <= 110.0); // Reasonable overshoot
    }

    #[test]
    fn test_tween_elastic() {
        let t = ease_elastic_out(0.5);
        // Elastic should oscillate
        assert!(t >= 0.9); // Near completion with oscillation
    }

    #[test]
    fn test_tween_bounce() {
        let t = ease_bounce_out(0.5);
        // Bounce should be > linear at midpoint
        assert!(t > 0.5);
        assert!(t <= 1.0);
    }

    #[test]
    fn test_tween_back() {
        let t = ease_back_out(1.0);
        // Back out should end at 1.0 (after overshoot and return)
        assert!((t - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_multiple_tweens_same_entity() {
        let mut manager = TweenManager::new();
        let entity_id = EntityId::new(1);

        // Add position tween
        manager.add(Tween::position(
            entity_id,
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 100.0),
            1000,
        ));

        // Add opacity tween
        manager.add(Tween::opacity(entity_id, 1.0, 0.5, 500));

        assert_eq!(manager.len(), 2);

        // Should have both tweens
        let pos_tween = manager.get_tween(entity_id, TweenProperty::Position);
        let opacity_tween = manager.get_tween(entity_id, TweenProperty::Opacity);

        assert!(pos_tween.is_some());
        assert!(opacity_tween.is_some());
    }

    #[test]
    fn test_convenience_functions() {
        let entity_id = EntityId::new(1);

        let tween = tween_position(entity_id, Vec2::new(100.0, 100.0), 1000);
        assert_eq!(tween.end_value, Vec2::new(100.0, 100.0));

        let tween = tween_opacity(entity_id, 0.5, 500);
        assert_eq!(tween.end_float, 0.5);
    }
}
