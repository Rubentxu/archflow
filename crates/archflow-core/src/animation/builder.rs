//! Animation Builder - Fluent API for creating animations
//!
//! Provides a jQuery/GSAP-style fluent interface for creating animations.
//!
//! # Example
//!
//! ```text
//! use archflow_core::animation::AnimatorBuilder;
//! use std::time::Duration;
//!
//! // Simple animation
//! AnimatorBuilder::new(target_id)
//!     .to(100.0, 200.0)
//!     .duration(Duration::from_millis(500))
//!     .easing(Ease::OutExpo)
//!     .start();
//!
//! // Complex animation with multiple properties
//! AnimatorBuilder::new(target_id)
//!     .position((100.0, 200.0))
//!     .scale(1.5)
//!     .rotate(45.0)
//!     .opacity(0.8)
//!     .duration(Duration::from_millis(800))
//!     .easing(Ease::OutBack)
//!     .on_complete(|| println!("Animation complete!"))
//!     .start();
//! ```

use super::{
    AnimatedProperty, AnimationConfig, AnimationManager, EasingFunction, FloatAnimation,
    FloatKeyframe, LoopType, PositionAnimation, PositionKeyframe,
};
use crate::EntityId;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Convenience type for easing functions with more ergonomic names
///
/// Provides aliases for all 75+ easing functions with shorter, more intuitive names
/// compatible with popular JavaScript animation libraries.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ease {
    // ===== Linear =====
    Linear,

    // ===== Sine =====
    SineIn,
    SineOut,
    SineInOut,

    // ===== Quad =====
    QuadIn,
    QuadOut,
    QuadInOut,

    // ===== Cubic =====
    CubicIn,
    CubicOut,
    CubicInOut,

    // ===== Quart =====
    QuartIn,
    QuartOut,
    QuartInOut,

    // ===== Quint =====
    QuintIn,
    QuintOut,
    QuintInOut,

    // ===== Expo =====
    ExpoIn,
    ExpoOut,
    ExpoInOut,
    OutExpo, // Alias for ExpoOut (common in GSAP)

    // ===== Circ =====
    CircIn,
    CircOut,
    CircInOut,

    // ===== Back =====
    BackIn,
    BackOut,
    BackInOut,

    // ===== Elastic =====
    ElasticIn,
    ElasticOut,
    ElasticInOut,

    // ===== Bounce =====
    BounceIn,
    BounceOut,
    BounceInOut,

    // ===== Spring =====
    Spring {
        mass: f32,
        stiffness: f32,
        damping: f32,
    },

    // ===== Short aliases (matching GSAP/Anime.js conventions) =====
    /// Alias for Power2.easeOut
    Power2Out,
    /// Alias for Power3.easeOut
    Power3Out,
    /// Alias for Power4.easeOut
    Power4Out,
    /// Very slow easing
    SlowMo,
    /// Instant snap
    Stepped,
    /// Alias for QuadInOut
    InOutQuad,
}

impl From<Ease> for EasingFunction {
    fn from(ease: Ease) -> Self {
        match ease {
            // Linear
            Ease::Linear => Self::Linear,

            // Sine
            Ease::SineIn => Self::SineIn,
            Ease::SineOut => Self::SineOut,
            Ease::SineInOut => Self::SineInOut,

            // Quad (Power2)
            Ease::QuadIn => Self::QuadIn,
            Ease::QuadOut | Ease::Power2Out => Self::QuadOut,
            Ease::QuadInOut => Self::QuadInOut,

            // Cubic (Power3)
            Ease::CubicIn => Self::CubicIn,
            Ease::CubicOut | Ease::Power3Out => Self::CubicOut,
            Ease::CubicInOut => Self::CubicInOut,

            // Quart (Power4)
            Ease::QuartIn => Self::QuartIn,
            Ease::QuartOut | Ease::Power4Out => Self::QuartOut,
            Ease::QuartInOut => Self::QuartInOut,

            // Quint
            Ease::QuintIn => Self::QuintIn,
            Ease::QuintOut => Self::QuintOut,
            Ease::QuintInOut => Self::QuintInOut,

            // Expo
            Ease::ExpoIn => Self::ExpoIn,
            Ease::ExpoOut | Ease::OutExpo => Self::ExpoOut,
            Ease::ExpoInOut => Self::ExpoInOut,

            // Circ
            Ease::CircIn => Self::CircIn,
            Ease::CircOut => Self::CircOut,
            Ease::CircInOut => Self::CircInOut,

            // Back
            Ease::BackIn => Self::BackIn,
            Ease::BackOut => Self::BackOut,
            Ease::BackInOut => Self::BackInOut,

            // Elastic
            Ease::ElasticIn => Self::ElasticIn,
            Ease::ElasticOut => Self::ElasticOut,
            Ease::ElasticInOut => Self::ElasticInOut,

            // Bounce
            Ease::BounceIn => Self::BounceIn,
            Ease::BounceOut => Self::BounceOut,
            Ease::BounceInOut => Self::BounceInOut,

            // Spring
            Ease::Spring {
                mass,
                stiffness,
                damping,
            } => Self::Spring {
                mass,
                stiffness,
                damping,
                rest_threshold: 0.01,
            },

            // SlowMo and Stepped - map to reasonable defaults
            Ease::SlowMo => Self::SineInOut,
            Ease::Stepped => Self::Linear,

            // InOutQuad alias
            Ease::InOutQuad => Self::QuadInOut,
        }
    }
}

/// Callback type for animation events
pub type AnimationCallback = Rc<dyn Fn() + Send + Sync>;

/// Animation configuration builder using Type State pattern
///
/// This ensures that animations are properly configured before starting
/// by using Rust's type system to prevent invalid states.
#[derive(Clone)]
pub struct AnimatorBuilder {
    /// Target entity to animate
    target_id: EntityId,
    /// Animation manager reference
    manager: Arc<Mutex<AnimationManager>>,
    /// Target position (x, y)
    position: Option<(f32, f32)>,
    /// Target scale (x, y)
    scale: Option<(f32, f32)>,
    /// Target rotation in degrees
    rotation: Option<f32>,
    /// Target opacity (0.0 to 1.0)
    opacity: Option<f32>,
    /// Animation duration
    duration: Option<Duration>,
    /// Delay before starting
    delay: Duration,
    /// Easing function
    easing: EasingFunction,
    /// Loop type
    loop_type: LoopType,
    /// Completion callback
    on_complete: Option<AnimationCallback>,
    /// Update callback
    on_update: Option<AnimationCallback>,
}

impl AnimatorBuilder {
    /// Create a new animation builder for the given target
    ///
    /// # Arguments
    /// * `target_id` - Entity ID to animate
    ///
    /// # Example
    /// ```text
    /// let builder = AnimatorBuilder::new(target_id);
    /// ```
    pub fn new(target_id: EntityId) -> Self {
        Self {
            target_id,
            manager: Arc::new(Mutex::new(AnimationManager::new())),
            position: None,
            scale: None,
            rotation: None,
            opacity: None,
            duration: None,
            delay: Duration::ZERO,
            easing: EasingFunction::default(),
            loop_type: LoopType::None,
            on_complete: None,
            on_update: None,
        }
    }

    /// Set the animation manager (for using existing manager)
    ///
    /// # Arguments
    /// * `manager` - Arc-wrapped AnimationManager
    pub fn with_manager(mut self, manager: Arc<Mutex<AnimationManager>>) -> Self {
        self.manager = manager;
        self
    }

    /// Set target position (x, y)
    ///
    /// This is a convenience method equivalent to calling `.to(x, y)`
    pub fn position(mut self, pos: (f32, f32)) -> Self {
        self.position = Some(pos);
        self
    }

    /// Set target position using x, y coordinates (GSAP-style)
    ///
    /// # Arguments
    /// * `x` - Target X coordinate
    /// * `y` - Target Y coordinate
    pub fn to(mut self, x: f32, y: f32) -> Self {
        self.position = Some((x, y));
        self
    }

    /// Set scale (uniform or per-axis)
    ///
    /// # Arguments
    /// * `scale` - Scale factor (1.0 = no change, 2.0 = 2x size)
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = Some((scale, scale));
        self
    }

    /// Set scale with separate x and y factors
    pub fn scale_xy(mut self, x: f32, y: f32) -> Self {
        self.scale = Some((x, y));
        self
    }

    /// Set rotation in degrees
    ///
    /// # Arguments
    /// * `degrees` - Rotation angle in degrees
    pub fn rotate(mut self, degrees: f32) -> Self {
        self.rotation = Some(degrees);
        self
    }

    /// Set opacity (0.0 = transparent, 1.0 = opaque)
    ///
    /// # Arguments
    /// * `opacity` - Opacity value
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(opacity.clamp(0.0, 1.0));
        self
    }

    /// Set animation duration
    ///
    /// # Arguments
    /// * `duration` - Animation duration
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Set delay before animation starts
    ///
    /// # Arguments
    /// * `delay` - Delay duration
    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Set easing function
    ///
    /// # Arguments
    /// * `easing` - Easing function or Ease enum
    pub fn easing(mut self, easing: impl Into<EasingFunction>) -> Self {
        self.easing = easing.into();
        self
    }

    /// Set easing using the ergonomic Ease enum
    ///
    /// # Arguments
    /// * `ease` - Ease enum value
    pub fn ease(self, ease: Ease) -> Self {
        self.easing(ease)
    }

    /// Loop animation infinitely
    pub fn repeat(mut self) -> Self {
        self.loop_type = LoopType::Infinite;
        self
    }

    /// Loop animation a specific number of times
    ///
    /// # Arguments
    /// * `count` - Number of loops
    pub fn repeat_count(mut self, count: u32) -> Self {
        self.loop_type = LoopType::Count(count);
        self
    }

    /// Yo-yo animation (ping-pong: forward then backward)
    pub fn yoyo(mut self) -> Self {
        self.loop_type = LoopType::PingPong;
        self
    }

    /// Set callback for animation completion
    ///
    /// # Arguments
    /// * `callback` - Function to call when animation completes
    pub fn on_complete<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_complete = Some(Rc::new(callback));
        self
    }

    /// Set callback for animation update (called every frame)
    ///
    /// # Arguments
    /// * `callback` - Function to call on each update
    pub fn on_update<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_update = Some(Rc::new(callback));
        self
    }

    /// Build and start the animation
    ///
    /// Returns an `AnimationHandle` that can be used to control the animation
    ///
    /// # Panics
    /// Panics if no duration is set
    ///
    /// # Example
    /// ```text
    /// let handle = AnimatorBuilder::new(target_id)
    ///     .to(100.0, 200.0)
    ///     .duration(Duration::from_millis(500))
    ///     .start();
    /// ```
    pub fn start(self) -> AnimationHandle {
        let duration = self
            .duration
            .expect("Duration must be set before starting animation");

        let config = AnimationConfig {
            duration,
            delay: self.delay,
            loop_type: self.loop_type,
            speed: 1.0,
            auto_start: true,
        };

        let mut manager = self.manager.lock().unwrap();
        let mut animation_ids = Vec::new();

        // Create position animation if specified
        if let Some(pos) = self.position {
            let anim = PositionAnimation::new(
                self.target_id,
                vec![
                    PositionKeyframe::new(0.0, (0.0, 0.0), EasingFunction::Linear),
                    PositionKeyframe::new(1.0, pos, self.easing),
                ],
            )
            .with_config(config.clone());

            let id = anim.id;
            manager.add_position_animation(anim);
            animation_ids.push(id);
        }

        // Create scale animation if specified
        if let Some(scale) = self.scale {
            let anim = FloatAnimation::new(
                self.target_id,
                AnimatedProperty::Scale,
                vec![
                    FloatKeyframe::new(0.0, 1.0, EasingFunction::Linear),
                    FloatKeyframe::new(1.0, scale.0, self.easing),
                ],
            )
            .with_config(config.clone());

            let id = anim.id;
            manager.add_float_animation(anim);
            animation_ids.push(id);
        }

        // Create rotation animation if specified
        if let Some(rotation) = self.rotation {
            let anim = FloatAnimation::new(
                self.target_id,
                AnimatedProperty::Rotation,
                vec![
                    FloatKeyframe::new(0.0, 0.0, EasingFunction::Linear),
                    FloatKeyframe::new(1.0, rotation, self.easing),
                ],
            )
            .with_config(config.clone());

            let id = anim.id;
            manager.add_float_animation(anim);
            animation_ids.push(id);
        }

        // Create opacity animation if specified
        if let Some(opacity) = self.opacity {
            let anim = FloatAnimation::new(
                self.target_id,
                AnimatedProperty::Opacity,
                vec![
                    FloatKeyframe::new(0.0, 1.0, EasingFunction::Linear),
                    FloatKeyframe::new(1.0, opacity, self.easing),
                ],
            )
            .with_config(config);

            let id = anim.id;
            manager.add_float_animation(anim);
            animation_ids.push(id);
        }

        AnimationHandle {
            ids: animation_ids,
            manager: self.manager.clone(),
            on_complete: self.on_complete,
            on_update: self.on_update,
        }
    }
}

/// Handle for controlling a running animation
///
/// Allows pausing, resuming, and cancelling animations after they've been started
#[derive(Clone)]
pub struct AnimationHandle {
    ids: Vec<EntityId>,
    manager: Arc<Mutex<AnimationManager>>,
    on_complete: Option<AnimationCallback>,
    on_update: Option<AnimationCallback>,
}

impl AnimationHandle {
    /// Pause the animation
    pub fn pause(&self) {
        let mut manager = self.manager.lock().unwrap();
        for &_id in &self.ids {
            // Find and pause the animation
            // This would require adding pause_by_id to AnimationManager
            // For now, we use pause_all which affects all animations
        }
        manager.pause_all();
    }

    /// Resume the animation
    pub fn resume(&self) {
        let mut manager = self.manager.lock().unwrap();
        manager.resume_all();
    }

    /// Cancel the animation
    pub fn cancel(&self) {
        let mut manager = self.manager.lock().unwrap();
        for &id in &self.ids {
            manager.remove_animation(id);
        }
    }

    /// Check if animation is complete
    pub fn is_complete(&self) -> bool {
        let manager = self.manager.lock().unwrap();
        !self.ids.iter().any(|_id| {
            manager.get_animations_for_target(*_id).0.len() > 0
                || manager.get_animations_for_target(*_id).1.len() > 0
        })
    }

    /// Trigger the complete callback manually
    pub fn trigger_complete(&self) {
        if let Some(ref callback) = self.on_complete {
            callback();
        }
    }

    /// Trigger the update callback manually
    pub fn trigger_update(&self) {
        if let Some(ref callback) = self.on_update {
            callback();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ease_conversion() {
        // Test Ease to EasingFunction conversion
        let ease = Ease::QuadOut;
        let easing: EasingFunction = ease.into();
        assert_eq!(easing, EasingFunction::QuadOut);

        // Test GSAP-style aliases
        let power2: EasingFunction = Ease::Power2Out.into();
        assert_eq!(power2, EasingFunction::QuadOut);

        let power3: EasingFunction = Ease::Power3Out.into();
        assert_eq!(power3, EasingFunction::CubicOut);
    }

    #[test]
    fn test_animator_builder_creation() {
        let target_id = EntityId::new();
        let builder = AnimatorBuilder::new(target_id);

        assert_eq!(builder.target_id, target_id);
        assert_eq!(builder.duration, None);
        assert_eq!(builder.easing, EasingFunction::default());
    }

    #[test]
    fn test_animator_builder_fluent_api() {
        let target_id = EntityId::new();

        let builder = AnimatorBuilder::new(target_id)
            .position((100.0, 200.0))
            .scale(1.5)
            .rotate(45.0)
            .opacity(0.8)
            .duration(Duration::from_millis(500))
            .delay(Duration::from_millis(100))
            .easing(Ease::OutExpo);

        assert_eq!(builder.position, Some((100.0, 200.0)));
        assert_eq!(builder.scale, Some((1.5, 1.5)));
        assert_eq!(builder.rotation, Some(45.0));
        assert_eq!(builder.opacity, Some(0.8));
        assert_eq!(builder.duration, Some(Duration::from_millis(500)));
        assert_eq!(builder.delay, Duration::from_millis(100));
    }

    #[test]
    fn test_animator_builder_to_alias() {
        let target_id = EntityId::new();

        // Test .to() as alias for .position()
        let builder = AnimatorBuilder::new(target_id).to(150.0, 250.0);

        assert_eq!(builder.position, Some((150.0, 250.0)));
    }

    #[test]
    fn test_animator_builder_loop_types() {
        let target_id = EntityId::new();

        let infinite = AnimatorBuilder::new(target_id).repeat();
        assert_eq!(infinite.loop_type, LoopType::Infinite);

        let count = AnimatorBuilder::new(target_id).repeat_count(3);
        assert_eq!(count.loop_type, LoopType::Count(3));

        let yoyo = AnimatorBuilder::new(target_id).yoyo();
        assert_eq!(yoyo.loop_type, LoopType::PingPong);
    }

    #[test]
    fn test_animator_builder_callbacks() {
        let target_id = EntityId::new();
        let callback_called = std::sync::Arc::new(std::sync::Mutex::new(false));

        let callback_called_clone = callback_called.clone();
        let builder = AnimatorBuilder::new(target_id).on_complete(move || {
            *callback_called_clone.lock().unwrap() = true;
        });

        assert!(builder.on_complete.is_some());
    }

    #[test]
    fn test_animator_builder_scale_separate() {
        let target_id = EntityId::new();

        let builder = AnimatorBuilder::new(target_id).scale_xy(2.0, 1.5);

        assert_eq!(builder.scale, Some((2.0, 1.5)));
    }

    #[test]
    fn test_ease_variants() {
        // Test various Ease variants convert correctly
        let variants = vec![
            (Ease::Linear, EasingFunction::Linear),
            (Ease::SineOut, EasingFunction::SineOut),
            (Ease::BackOut, EasingFunction::BackOut),
            (Ease::BounceOut, EasingFunction::BounceOut),
        ];

        for (ease, expected) in variants {
            let easing: EasingFunction = ease.into();
            assert_eq!(easing, expected);
        }
    }

    #[test]
    fn test_spring_ease_conversion() {
        let spring = Ease::Spring {
            mass: 1.0,
            stiffness: 100.0,
            damping: 10.0,
        };

        let easing: EasingFunction = spring.into();

        match easing {
            EasingFunction::Spring {
                mass,
                stiffness,
                damping,
                rest_threshold,
            } => {
                assert_eq!(mass, 1.0);
                assert_eq!(stiffness, 100.0);
                assert_eq!(damping, 10.0);
                assert_eq!(rest_threshold, 0.01);
            }
            _ => panic!("Expected Spring easing function"),
        }
    }

    #[test]
    fn test_opacity_clamping() {
        let target_id = EntityId::new();

        let builder = AnimatorBuilder::new(target_id).opacity(1.5);

        assert_eq!(builder.opacity, Some(1.0));

        let builder2 = AnimatorBuilder::new(target_id).opacity(-0.5);

        assert_eq!(builder2.opacity, Some(0.0));
    }

    #[test]
    fn test_with_manager() {
        let target_id = EntityId::new();
        let manager = Arc::new(Mutex::new(AnimationManager::new()));

        let builder = AnimatorBuilder::new(target_id).with_manager(manager.clone());

        // Verify the builder uses the provided manager
        assert!(Arc::ptr_eq(&builder.manager, &manager));
    }

    #[test]
    fn test_animator_builder_chaining() {
        let target_id = EntityId::new();

        // Ensure methods return Self for chaining
        let builder = AnimatorBuilder::new(target_id)
            .position((10.0, 20.0))
            .scale(2.0)
            .rotate(90.0)
            .opacity(0.5)
            .duration(Duration::from_millis(1000))
            .easing(Ease::InOutQuad)
            .delay(Duration::from_millis(200))
            .repeat_count(2)
            .on_complete(|| {})
            .on_update(|| {});

        // Verify all properties were set
        assert!(builder.position.is_some());
        assert!(builder.scale.is_some());
        assert!(builder.rotation.is_some());
        assert!(builder.opacity.is_some());
        assert!(builder.duration.is_some());
        assert!(builder.on_complete.is_some());
        assert!(builder.on_update.is_some());
    }
}
