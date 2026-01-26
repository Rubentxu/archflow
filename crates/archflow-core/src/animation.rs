//! Animation System - Keyframe animations with easing functions
//!
//! Provides:
//! - Animation trait for animated properties
//! - KeyframeAnimation with multiple keyframes
//! - Easing functions (linear, ease-in, ease-out, bezier)
//! - AnimationManager for running animations

use crate::EntityId;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Timing function type for easing
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EasingFunction {
    /// Linear easing (no acceleration)
    Linear,
    /// Ease in - starts slow, accelerates
    EaseIn,
    /// Ease out - starts fast, decelerates
    EaseOut,
    /// Ease in and out - slow start, fast middle, slow end
    EaseInOut,
    /// Cubic bezier with custom control points
    CubicBezier(f32, f32, f32, f32),
    /// Elastic bounce effect
    Elastic,
    /// Bounce effect at the end
    Bounce,
}

impl Default for EasingFunction {
    fn default() -> Self {
        Self::EaseInOut
    }
}

impl EasingFunction {
    /// Apply the easing function to a normalized time value (0.0 to 1.0)
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            Self::Linear => t,
            Self::EaseIn => t * t,
            Self::EaseOut => t * (2.0 - t),
            Self::EaseInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    let t = t * 2.0 - 2.0;
                    0.5 * t * t * t + 1.0
                }
            }
            Self::CubicBezier(x1, y1, x2, y2) => Self::cubic_bezier(*x1, *y1, *x2, *y2, t),
            Self::Elastic => {
                // Elastic easing out
                let p = 0.3;
                let s = p / 4.0;
                (2.0_f32).powf(-10.0 * t) * ((t - s) * (2.0 * std::f32::consts::PI) / p).sin() + 1.0
            }
            Self::Bounce => {
                // Bounce easing out
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
        }
    }

    /// Calculate cubic bezier value
    fn cubic_bezier(_x1: f32, y1: f32, _x2: f32, y2: f32, t: f32) -> f32 {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        mt3 + 3.0 * mt2 * t * y1 + 3.0 * mt * t2 * y2 + t3
    }
}

/// Animated property type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnimatedProperty {
    /// Position (x, y)
    Position,
    /// Size (width, height)
    Size,
    /// Rotation in degrees
    Rotation,
    /// Opacity (0.0 to 1.0)
    Opacity,
    /// Scale (x, y)
    Scale,
    /// Color (r, g, b, a)
    Color,
    /// Stroke width
    StrokeWidth,
    /// Custom property by name
    Custom(String),
}

/// Animation loop type
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LoopType {
    /// Play once and stop
    None,
    /// Loop indefinitely
    Infinite,
    /// Loop a specific number of times
    Count(u32),
    /// Play forward then backward (ping-pong)
    PingPong,
    /// Ping-pong with specific count
    PingPongCount(u32),
}

impl Default for LoopType {
    fn default() -> Self {
        Self::None
    }
}

/// Direction of animation playback
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AnimationDirection {
    /// Play forward
    Forward,
    /// Play backward (for ping-pong)
    Backward,
}

/// State of an animation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AnimationState {
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

/// Configuration for animation behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationConfig {
    /// Duration of the animation
    pub duration: Duration,
    /// Delay before starting
    pub delay: Duration,
    /// Loop behavior
    pub loop_type: LoopType,
    /// Playback speed (1.0 = normal)
    pub speed: f32,
    /// Whether to start automatically
    pub auto_start: bool,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            duration: Duration::from_millis(500),
            delay: Duration::ZERO,
            loop_type: LoopType::None,
            speed: 1.0,
            auto_start: true,
        }
    }
}

/// A keyframe with position value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionKeyframe {
    /// Time offset from animation start (0.0 to 1.0 normalized)
    pub time: f32,
    /// Value at this keyframe
    pub value: (f32, f32),
    /// Easing function to use from this keyframe to the next
    pub easing: EasingFunction,
}

impl PositionKeyframe {
    /// Create a new keyframe
    pub fn new(time: f32, value: (f32, f32), easing: EasingFunction) -> Self {
        Self {
            time,
            value,
            easing,
        }
    }
}

/// A keyframe with float value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloatKeyframe {
    /// Time offset from animation start (0.0 to 1.0 normalized)
    pub time: f32,
    /// Value at this keyframe
    pub value: f32,
    /// Easing function to use from this keyframe to the next
    pub easing: EasingFunction,
}

impl FloatKeyframe {
    /// Create a new keyframe
    pub fn new(time: f32, value: f32, easing: EasingFunction) -> Self {
        Self {
            time,
            value,
            easing,
        }
    }
}

/// Position-based animation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionAnimation {
    /// Unique animation ID
    pub id: EntityId,
    /// Entity this animation applies to
    pub target_id: EntityId,
    /// Keyframes for the animation
    pub keyframes: Vec<PositionKeyframe>,
    /// Animation configuration
    pub config: AnimationConfig,
    /// Current state
    pub state: AnimationState,
    /// Current playback direction
    pub direction: AnimationDirection,
    /// Current progress (0.0 to 1.0)
    pub progress: f32,
    /// Number of loops completed
    pub loop_count: u32,
    /// Elapsed time since start
    pub elapsed: Duration,
}

impl PositionAnimation {
    /// Create a new position animation
    pub fn new(target_id: EntityId, keyframes: Vec<PositionKeyframe>) -> Self {
        Self {
            id: EntityId::new(),
            target_id,
            keyframes,
            config: AnimationConfig::default(),
            state: AnimationState::Pending,
            direction: AnimationDirection::Forward,
            progress: 0.0,
            loop_count: 0,
            elapsed: Duration::ZERO,
        }
    }

    /// Set the animation configuration
    pub fn with_config(mut self, config: AnimationConfig) -> Self {
        self.config = config;
        self
    }

    /// Start the animation
    pub fn start(&mut self) {
        self.state = AnimationState::Playing;
        self.elapsed = Duration::ZERO;
        self.progress = 0.0;
    }

    /// Pause the animation
    pub fn pause(&mut self) {
        if self.state == AnimationState::Playing {
            self.state = AnimationState::Paused;
        }
    }

    /// Resume a paused animation
    pub fn resume(&mut self) {
        if self.state == AnimationState::Paused {
            self.state = AnimationState::Playing;
        }
    }

    /// Cancel the animation
    pub fn cancel(&mut self) {
        self.state = AnimationState::Cancelled;
        self.progress = 0.0;
    }

    /// Reset the animation to initial state
    pub fn reset(&mut self) {
        self.state = AnimationState::Pending;
        self.progress = 0.0;
        self.loop_count = 0;
        self.elapsed = Duration::ZERO;
    }

    /// Get the current interpolated value
    pub fn current_value(&self) -> (f32, f32) {
        if self.keyframes.is_empty() {
            return (0.0, 0.0);
        }

        if self.keyframes.len() == 1 {
            return self.keyframes[0].value;
        }

        let mut prev_keyframe = &self.keyframes[0];
        let mut next_keyframe = &self.keyframes[1];

        for (i, keyframe) in self.keyframes.iter().enumerate() {
            if keyframe.time >= self.progress {
                if i == 0 {
                    return keyframe.value;
                }
                prev_keyframe = &self.keyframes[i - 1];
                next_keyframe = keyframe;
                break;
            }
            if i == self.keyframes.len() - 1 {
                prev_keyframe = &self.keyframes[i - 1];
                next_keyframe = keyframe;
                break;
            }
        }

        let time_range = next_keyframe.time - prev_keyframe.time;
        if time_range.abs() < 1e-6 {
            return next_keyframe.value;
        }

        let local_t = (self.progress - prev_keyframe.time) / time_range;
        let eased_t = prev_keyframe.easing.apply(local_t);

        (
            prev_keyframe.value.0 + (next_keyframe.value.0 - prev_keyframe.value.0) * eased_t,
            prev_keyframe.value.1 + (next_keyframe.value.1 - prev_keyframe.value.1) * eased_t,
        )
    }

    /// Check if the animation is complete
    pub fn is_complete(&self) -> bool {
        self.progress >= 1.0 && matches!(self.config.loop_type, LoopType::None | LoopType::Count(0))
    }

    /// Update animation state based on elapsed time
    /// Returns true if animation just completed
    pub fn update(&mut self, delta: Duration) -> bool {
        if !matches!(self.state, AnimationState::Playing) {
            return false;
        }

        self.elapsed += delta;

        // Handle delay
        if self.elapsed < self.config.delay {
            return false;
        }

        let effective_elapsed = self.elapsed - self.config.delay;
        let duration_secs = self.config.duration.as_secs_f32();
        let speed = self.config.speed;

        let raw_progress = if duration_secs > 0.0 {
            effective_elapsed.as_secs_f32() / duration_secs * speed
        } else {
            1.0
        };

        match self.config.loop_type {
            LoopType::None | LoopType::Count(0) => {
                self.progress = raw_progress.min(1.0);
                if self.progress >= 1.0 {
                    self.state = AnimationState::Completed;
                    return true;
                }
            }
            LoopType::Infinite => {
                self.progress = raw_progress % 1.0;
            }
            LoopType::Count(n) => {
                let loops_completed = (raw_progress as u32).min(n);
                self.progress = raw_progress - loops_completed as f32;
                self.loop_count = loops_completed;
                if loops_completed >= n && raw_progress >= n as f32 {
                    self.progress = 1.0;
                    self.state = AnimationState::Completed;
                    return true;
                }
            }
            LoopType::PingPong => {
                let cycle_progress = raw_progress % 2.0;
                if cycle_progress < 1.0 {
                    self.progress = cycle_progress;
                    self.direction = AnimationDirection::Forward;
                } else {
                    self.progress = 2.0 - cycle_progress;
                    self.direction = AnimationDirection::Backward;
                }
            }
            LoopType::PingPongCount(n) => {
                let full_cycles = (raw_progress / 2.0) as u32;
                self.loop_count = full_cycles;
                if full_cycles >= n {
                    self.progress = 1.0;
                    self.state = AnimationState::Completed;
                    return true;
                }
                let cycle_progress = raw_progress % 2.0;
                if cycle_progress < 1.0 {
                    self.progress = cycle_progress;
                    self.direction = AnimationDirection::Forward;
                } else {
                    self.progress = 2.0 - cycle_progress;
                    self.direction = AnimationDirection::Backward;
                }
            }
        }

        false
    }

    /// Check if animation loops
    pub fn is_looped(&self) -> bool {
        matches!(
            self.config.loop_type,
            LoopType::Infinite | LoopType::PingPong
        )
    }
}

/// Float value animation (for opacity, rotation, scale, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloatAnimation {
    /// Unique animation ID
    pub id: EntityId,
    /// Entity this animation applies to
    pub target_id: EntityId,
    /// Property being animated
    pub property: AnimatedProperty,
    /// Keyframes for the animation
    pub keyframes: Vec<FloatKeyframe>,
    /// Animation configuration
    pub config: AnimationConfig,
    /// Current state
    pub state: AnimationState,
    /// Current playback direction
    pub direction: AnimationDirection,
    /// Current progress (0.0 to 1.0)
    pub progress: f32,
    /// Number of loops completed
    pub loop_count: u32,
    /// Elapsed time since start
    pub elapsed: Duration,
}

impl FloatAnimation {
    /// Create a new float animation
    pub fn new(
        target_id: EntityId,
        property: AnimatedProperty,
        keyframes: Vec<FloatKeyframe>,
    ) -> Self {
        Self {
            id: EntityId::new(),
            target_id,
            property,
            keyframes,
            config: AnimationConfig::default(),
            state: AnimationState::Pending,
            direction: AnimationDirection::Forward,
            progress: 0.0,
            loop_count: 0,
            elapsed: Duration::ZERO,
        }
    }

    /// Set the animation configuration
    pub fn with_config(mut self, config: AnimationConfig) -> Self {
        self.config = config;
        self
    }

    /// Start the animation
    pub fn start(&mut self) {
        self.state = AnimationState::Playing;
        self.elapsed = Duration::ZERO;
        self.progress = 0.0;
    }

    /// Pause the animation
    pub fn pause(&mut self) {
        if self.state == AnimationState::Playing {
            self.state = AnimationState::Paused;
        }
    }

    /// Resume a paused animation
    pub fn resume(&mut self) {
        if self.state == AnimationState::Paused {
            self.state = AnimationState::Playing;
        }
    }

    /// Cancel the animation
    pub fn cancel(&mut self) {
        self.state = AnimationState::Cancelled;
        self.progress = 0.0;
    }

    /// Get the current interpolated value
    pub fn current_value(&self) -> f32 {
        if self.keyframes.is_empty() {
            return 0.0;
        }

        if self.keyframes.len() == 1 {
            return self.keyframes[0].value;
        }

        let mut prev_keyframe = &self.keyframes[0];
        let mut next_keyframe = &self.keyframes[1];

        for (i, keyframe) in self.keyframes.iter().enumerate() {
            if keyframe.time >= self.progress {
                if i == 0 {
                    return keyframe.value;
                }
                prev_keyframe = &self.keyframes[i - 1];
                next_keyframe = keyframe;
                break;
            }
            if i == self.keyframes.len() - 1 {
                prev_keyframe = &self.keyframes[i - 1];
                next_keyframe = keyframe;
                break;
            }
        }

        let time_range = next_keyframe.time - prev_keyframe.time;
        if time_range.abs() < 1e-6 {
            return next_keyframe.value;
        }

        let local_t = (self.progress - prev_keyframe.time) / time_range;
        let eased_t = prev_keyframe.easing.apply(local_t);

        prev_keyframe.value + (next_keyframe.value - prev_keyframe.value) * eased_t
    }

    /// Update animation state
    pub fn update(&mut self, delta: Duration) -> bool {
        if !matches!(self.state, AnimationState::Playing) {
            return false;
        }

        self.elapsed += delta;

        if self.elapsed < self.config.delay {
            return false;
        }

        let effective_elapsed = self.elapsed - self.config.delay;
        let duration_secs = self.config.duration.as_secs_f32();
        let speed = self.config.speed;

        let raw_progress = if duration_secs > 0.0 {
            effective_elapsed.as_secs_f32() / duration_secs * speed
        } else {
            1.0
        };

        match self.config.loop_type {
            LoopType::None | LoopType::Count(0) => {
                self.progress = raw_progress.min(1.0);
                if self.progress >= 1.0 {
                    self.state = AnimationState::Completed;
                    return true;
                }
            }
            LoopType::Infinite => {
                self.progress = raw_progress % 1.0;
            }
            LoopType::Count(n) => {
                let loops_completed = (raw_progress as u32).min(n);
                self.progress = raw_progress - loops_completed as f32;
                self.loop_count = loops_completed;
                if loops_completed >= n && raw_progress >= n as f32 {
                    self.progress = 1.0;
                    self.state = AnimationState::Completed;
                    return true;
                }
            }
            LoopType::PingPong => {
                let cycle_progress = raw_progress % 2.0;
                if cycle_progress < 1.0 {
                    self.progress = cycle_progress;
                    self.direction = AnimationDirection::Forward;
                } else {
                    self.progress = 2.0 - cycle_progress;
                    self.direction = AnimationDirection::Backward;
                }
            }
            LoopType::PingPongCount(n) => {
                let full_cycles = (raw_progress / 2.0) as u32;
                self.loop_count = full_cycles;
                if full_cycles >= n {
                    self.progress = 1.0;
                    self.state = AnimationState::Completed;
                    return true;
                }
                let cycle_progress = raw_progress % 2.0;
                if cycle_progress < 1.0 {
                    self.progress = cycle_progress;
                    self.direction = AnimationDirection::Forward;
                } else {
                    self.progress = 2.0 - cycle_progress;
                    self.direction = AnimationDirection::Backward;
                }
            }
        }

        false
    }

    /// Check if the animation is complete
    pub fn is_complete(&self) -> bool {
        self.progress >= 1.0 && matches!(self.config.loop_type, LoopType::None | LoopType::Count(0))
    }

    /// Check if animation loops
    pub fn is_looped(&self) -> bool {
        matches!(
            self.config.loop_type,
            LoopType::Infinite | LoopType::PingPong
        )
    }
}

/// Result of an animation update
#[derive(Debug, Clone)]
pub struct AnimationUpdate {
    /// Animation ID
    pub animation_id: EntityId,
    /// Target entity
    pub target_id: EntityId,
    /// Property that changed
    pub property: AnimatedProperty,
    /// New float value (if applicable)
    pub float_value: Option<f32>,
    /// New position value (if applicable)
    pub position_value: Option<(f32, f32)>,
    /// Whether this is the final update
    pub is_complete: bool,
}

/// Manager for running animations
#[derive(Debug, Default)]
pub struct AnimationManager {
    /// Active position animations
    position_animations: Vec<PositionAnimation>,
    /// Active float animations
    float_animations: Vec<FloatAnimation>,
    /// Global time scale
    time_scale: f32,
    /// Whether animations are globally paused
    paused: bool,
}

impl AnimationManager {
    /// Create a new animation manager
    pub fn new() -> Self {
        Self {
            position_animations: Vec::new(),
            float_animations: Vec::new(),
            time_scale: 1.0,
            paused: false,
        }
    }

    /// Add a position animation
    pub fn add_position_animation(&mut self, animation: PositionAnimation) {
        self.position_animations.push(animation);
    }

    /// Add a float animation
    pub fn add_float_animation(&mut self, animation: FloatAnimation) {
        self.float_animations.push(animation);
    }

    /// Remove an animation by ID
    pub fn remove_animation(&mut self, id: EntityId) -> bool {
        let pos_removed = self.position_animations.iter().any(|a| a.id == id);
        self.position_animations.retain(|a| a.id != id);

        let float_removed = self.float_animations.iter().any(|a| a.id == id);
        self.float_animations.retain(|a| a.id != id);

        pos_removed || float_removed
    }

    /// Get animations for a specific target entity
    pub fn get_animations_for_target(
        &self,
        target_id: EntityId,
    ) -> (Vec<&PositionAnimation>, Vec<&FloatAnimation>) {
        (
            self.position_animations
                .iter()
                .filter(|a| a.target_id == target_id)
                .collect(),
            self.float_animations
                .iter()
                .filter(|a| a.target_id == target_id)
                .collect(),
        )
    }

    /// Get all active position values
    pub fn get_active_positions(&self) -> Vec<(EntityId, (f32, f32))> {
        self.position_animations
            .iter()
            .map(|a| (a.target_id, a.current_value()))
            .collect()
    }

    /// Get all active float values
    pub fn get_active_floats(&self) -> Vec<(EntityId, AnimatedProperty, f32)> {
        self.float_animations
            .iter()
            .map(|a| (a.target_id, a.property.clone(), a.current_value()))
            .collect()
    }

    /// Update all active animations
    pub fn update(&mut self, delta: Duration) -> Vec<AnimationUpdate> {
        if self.paused {
            return Vec::new();
        }

        let scaled_delta = Duration::from_secs_f64(delta.as_secs_f64() * self.time_scale as f64);
        let mut updates = Vec::new();

        // Update position animations
        for animation in &mut self.position_animations {
            if animation.update(scaled_delta) {
                updates.push(AnimationUpdate {
                    animation_id: animation.id,
                    target_id: animation.target_id,
                    property: AnimatedProperty::Position,
                    float_value: None,
                    position_value: Some(animation.current_value()),
                    is_complete: true,
                });
            }
        }

        // Remove completed non-looped animations
        self.position_animations.retain(|a| {
            !(a.is_complete() && matches!(a.config.loop_type, LoopType::None | LoopType::Count(0)))
        });

        // Update float animations
        for animation in &mut self.float_animations {
            if animation.update(scaled_delta) {
                updates.push(AnimationUpdate {
                    animation_id: animation.id,
                    target_id: animation.target_id,
                    property: animation.property.clone(),
                    float_value: Some(animation.current_value()),
                    position_value: None,
                    is_complete: true,
                });
            }
        }

        // Remove completed non-looped animations
        self.float_animations.retain(|a| {
            !(a.is_complete() && matches!(a.config.loop_type, LoopType::None | LoopType::Count(0)))
        });

        updates
    }

    /// Pause all animations
    pub fn pause_all(&mut self) {
        self.paused = true;
        for animation in &mut self.position_animations {
            animation.pause();
        }
        for animation in &mut self.float_animations {
            animation.pause();
        }
    }

    /// Resume all animations
    pub fn resume_all(&mut self) {
        self.paused = false;
        for animation in &mut self.position_animations {
            animation.resume();
        }
        for animation in &mut self.float_animations {
            animation.resume();
        }
    }

    /// Stop all animations
    pub fn stop_all(&mut self) {
        self.position_animations.clear();
        self.float_animations.clear();
    }

    /// Set global time scale
    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale.max(0.0);
    }

    /// Get global time scale
    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }

    /// Check if any animations are running
    pub fn is_animating(&self) -> bool {
        !self.position_animations.is_empty() || !self.float_animations.is_empty()
    }

    /// Get count of active animations
    pub fn len(&self) -> usize {
        self.position_animations.len() + self.float_animations.len()
    }

    /// Check if no animations are running
    pub fn is_empty(&self) -> bool {
        self.position_animations.is_empty() && self.float_animations.is_empty()
    }
}

/// Convenience function to create a position animation
pub fn position_animation(
    target_id: EntityId,
    from: (f32, f32),
    to: (f32, f32),
    duration: Duration,
) -> PositionAnimation {
    PositionAnimation::new(
        target_id,
        vec![
            PositionKeyframe::new(0.0, from, EasingFunction::EaseInOut),
            PositionKeyframe::new(1.0, to, EasingFunction::EaseInOut),
        ],
    )
    .with_config(AnimationConfig {
        duration,
        ..Default::default()
    })
}

/// Convenience function to create a fade animation
pub fn fade_animation(
    target_id: EntityId,
    from: f32,
    to: f32,
    duration: Duration,
) -> FloatAnimation {
    FloatAnimation::new(
        target_id,
        AnimatedProperty::Opacity,
        vec![
            FloatKeyframe::new(0.0, from, EasingFunction::Linear),
            FloatKeyframe::new(1.0, to, EasingFunction::Linear),
        ],
    )
    .with_config(AnimationConfig {
        duration,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easing_functions() {
        // Linear should return same value
        assert!((EasingFunction::Linear.apply(0.5) - 0.5).abs() < 1e-6);

        // EaseIn at 0.5 should be less than 0.5
        assert!(EasingFunction::EaseIn.apply(0.5) < 0.5);

        // EaseOut at 0.5 should be greater than 0.5
        assert!(EasingFunction::EaseOut.apply(0.5) > 0.5);

        // EaseInOut at 0.5 should be 0.5 (inflection point)
        assert!((EasingFunction::EaseInOut.apply(0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_easing_bounds() {
        for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let linear = EasingFunction::Linear.apply(t);
            assert!(linear >= 0.0 && linear <= 1.0);

            let ease_in = EasingFunction::EaseIn.apply(t);
            assert!(ease_in >= 0.0 && ease_in <= 1.0);

            let ease_out = EasingFunction::EaseOut.apply(t);
            assert!(ease_out >= 0.0 && ease_out <= 1.0);

            let ease_in_out = EasingFunction::EaseInOut.apply(t);
            assert!(ease_in_out >= 0.0 && ease_in_out <= 1.0);
        }
    }

    #[test]
    fn test_position_animation() {
        let target_id = EntityId::from_u128(1);
        let animation = PositionAnimation::new(
            target_id,
            vec![
                PositionKeyframe::new(0.0, (0.0, 0.0), EasingFunction::Linear),
                PositionKeyframe::new(1.0, (100.0, 100.0), EasingFunction::Linear),
            ],
        );

        assert_eq!(animation.keyframes.len(), 2);
        assert_eq!(animation.state, AnimationState::Pending);
    }

    #[test]
    fn test_position_animation_update() {
        let target_id = EntityId::from_u128(1);
        let mut animation = PositionAnimation::new(
            target_id,
            vec![
                PositionKeyframe::new(0.0, (0.0, 0.0), EasingFunction::Linear),
                PositionKeyframe::new(1.0, (100.0, 100.0), EasingFunction::Linear),
            ],
        );
        animation.config.duration = Duration::from_millis(100);

        // Initial state
        assert_eq!(animation.state, AnimationState::Pending);
        assert!((animation.progress - 0.0).abs() < 1e-6);

        // Start animation
        animation.start();
        assert_eq!(animation.state, AnimationState::Playing);

        // After update, progress should increase
        animation.update(Duration::from_millis(50));
        assert!(animation.progress >= 0.0);
    }

    #[test]
    fn test_position_animation_pause_resume() {
        let target_id = EntityId::from_u128(1);
        let mut animation = PositionAnimation::new(
            target_id,
            vec![
                PositionKeyframe::new(0.0, (0.0, 0.0), EasingFunction::Linear),
                PositionKeyframe::new(1.0, (100.0, 100.0), EasingFunction::Linear),
            ],
        );
        animation.config.duration = Duration::from_secs(10);

        animation.start();
        assert_eq!(animation.state, AnimationState::Playing);

        animation.pause();
        assert_eq!(animation.state, AnimationState::Paused);

        animation.resume();
        assert_eq!(animation.state, AnimationState::Playing);
    }

    #[test]
    fn test_position_animation_cancel() {
        let target_id = EntityId::from_u128(1);
        let mut animation = PositionAnimation::new(
            target_id,
            vec![
                PositionKeyframe::new(0.0, (0.0, 0.0), EasingFunction::Linear),
                PositionKeyframe::new(1.0, (100.0, 100.0), EasingFunction::Linear),
            ],
        );

        animation.start();
        animation.cancel();

        assert_eq!(animation.state, AnimationState::Cancelled);
        assert!((animation.progress - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_float_animation() {
        let target_id = EntityId::from_u128(1);
        let animation = FloatAnimation::new(
            target_id,
            AnimatedProperty::Opacity,
            vec![
                FloatKeyframe::new(0.0, 0.0, EasingFunction::Linear),
                FloatKeyframe::new(1.0, 1.0, EasingFunction::Linear),
            ],
        );

        assert_eq!(animation.keyframes.len(), 2);
    }

    #[test]
    fn test_animation_manager() {
        let mut manager = AnimationManager::new();

        assert!(manager.is_empty());
        assert!(!manager.is_animating());

        let target_id = EntityId::from_u128(1);
        let mut animation = PositionAnimation::new(
            target_id,
            vec![
                PositionKeyframe::new(0.0, (0.0, 0.0), EasingFunction::Linear),
                PositionKeyframe::new(1.0, (100.0, 100.0), EasingFunction::Linear),
            ],
        );

        let animation_id = animation.id;
        manager.add_position_animation(animation);

        assert_eq!(manager.len(), 1);
        assert!(manager.is_animating());

        let removed = manager.remove_animation(animation_id);
        assert!(removed);
        assert!(manager.is_empty());
    }

    #[test]
    fn test_animation_manager_pause_all() {
        let mut manager = AnimationManager::new();
        let target_id = EntityId::from_u128(1);

        let mut animation = PositionAnimation::new(
            target_id,
            vec![
                PositionKeyframe::new(0.0, (0.0, 0.0), EasingFunction::Linear),
                PositionKeyframe::new(1.0, (100.0, 100.0), EasingFunction::Linear),
            ],
        );
        animation.start();

        manager.add_position_animation(animation);

        manager.pause_all();
        assert!(manager.paused);
    }

    #[test]
    fn test_position_interpolation() {
        let animation = PositionAnimation::new(
            EntityId::from_u128(1),
            vec![
                PositionKeyframe::new(0.0, (0.0, 0.0), EasingFunction::Linear),
                PositionKeyframe::new(0.5, (50.0, 50.0), EasingFunction::Linear),
                PositionKeyframe::new(1.0, (100.0, 100.0), EasingFunction::Linear),
            ],
        );

        let value = animation.current_value();
        assert_eq!(value, (0.0, 0.0));
    }

    #[test]
    fn test_loop_types() {
        let target_id = EntityId::from_u128(1);

        // Infinite loop
        let mut infinite = PositionAnimation::new(
            target_id,
            vec![
                PositionKeyframe::new(0.0, (0.0, 0.0), EasingFunction::Linear),
                PositionKeyframe::new(1.0, (100.0, 100.0), EasingFunction::Linear),
            ],
        );
        infinite.config.loop_type = LoopType::Infinite;
        assert!(infinite.is_looped());

        // No loop
        let none = PositionAnimation::new(
            target_id,
            vec![
                PositionKeyframe::new(0.0, (0.0, 0.0), EasingFunction::Linear),
                PositionKeyframe::new(1.0, (100.0, 100.0), EasingFunction::Linear),
            ],
        );
        assert!(!none.is_looped());
    }

    #[test]
    fn test_convenience_functions() {
        let target_id = EntityId::from_u128(1);

        // Position animation
        let pos_anim = position_animation(
            target_id,
            (0.0, 0.0),
            (100.0, 100.0),
            Duration::from_millis(500),
        );
        assert_eq!(pos_anim.target_id, target_id);
        assert_eq!(pos_anim.keyframes.len(), 2);

        // Fade animation
        let fade_anim = fade_animation(target_id, 0.0, 1.0, Duration::from_millis(300));
        assert_eq!(fade_anim.target_id, target_id);
        assert_eq!(fade_anim.property, AnimatedProperty::Opacity);
        assert_eq!(fade_anim.keyframes.len(), 2);
    }
}
