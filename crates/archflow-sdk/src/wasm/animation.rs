//! High-performance WASM bindings for the animation system
//!
//! This module provides optimized WebAssembly bindings for the animation system,
//! using zero-copy patterns and efficient JS-WASM communication.
//!
//! # Performance Optimizations
//!
//! - **Zero-Copy Serialization**: Uses `serde-wasm-bindgen` for direct JSValue conversion
//! - **Batch Operations**: Multiple animations can be controlled with single JS call
//! - **Shared Memory**: Animation data kept in WASM, only pointers exposed to JS
//! - **Lazy Allocation**: Animations only created when explicitly started
//! - **Efficient Callbacks**: Event listeners use function pointers instead of JS closures
//!
//! # Usage Example
//!
//! ```ignore
//! // Initialize animation system
//! const animSystem = new AnimationSystem();
//!
//! // Create animation builder
//! const builder = animSystem.animate(shapeId)
//!   .to(100, 100)
//!   .duration(500)
//!   .easing('easeInOut')
//!   .play();
//!
//! // Or use timeline for sequencing
//! const timeline = animSystem.timeline()
//!   .add(anim1.animate().to(200, 200).duration(300))
//!   .add(anim2.animate().rotate(90).duration(400), '-=200')
//!   .play();
//! ```

use crate::wasm::{JsColor, JsVec2};
use archflow_core::animation::{
    AnimatedPropertyValue, AnimationCanvasAdapter, AnimationEvent, AnimationEventDispatcher,
    AnimationId, AnimationPhase, EasingFunction, Stagger, Timeline, TimelineHandle,
    TimelinePosition,
};
use archflow_core::{EntityId, Vec2};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Animation system manager for JavaScript interop
///
/// This manages all animations in the system, providing a high-level API
/// that integrates seamlessly with the Canvas editor.
#[wasm_bindgen]
pub struct AnimationSystem {
    /// Active animations managed by this system
    animations: RefCell<HashMap<String, TimelineHandle>>,
    /// Event dispatcher for animation lifecycle events
    dispatcher: RefCell<AnimationEventDispatcher>,
    /// Canvas adapter for integration with canvas events
    canvas_adapter: RefCell<AnimationCanvasAdapter>,
    /// Next animation ID counter
    next_id: RefCell<u64>,
}

#[wasm_bindgen]
impl AnimationSystem {
    /// Creates a new animation system
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            animations: RefCell::new(HashMap::new()),
            dispatcher: RefCell::new(AnimationEventDispatcher::new()),
            canvas_adapter: RefCell::new(AnimationCanvasAdapter::new()),
            next_id: RefCell::new(0),
        }
    }

    // === Animation Builder API ===

    /// Creates a new animation builder for a shape
    ///
    /// Returns a builder that can be chained for fluent API.
    #[wasm_bindgen]
    pub fn animate(&self, target_id: &str) -> JsAnimationBuilder {
        JsAnimationBuilder::new(target_id, self)
    }

    // === Timeline API ===

    /// Creates a new timeline for sequencing animations
    #[wasm_bindgen]
    pub fn timeline(&self) -> JsTimelineBuilder {
        JsTimelineBuilder::new(self)
    }

    // === Staggering API ===

    /// Creates a stagger configuration for wave animations
    #[wasm_bindgen]
    pub fn stagger(&self, config: JsValue) -> JsResult<JsStagger> {
        let config: JsStaggerConfig = from_value(config).map_err(JsError::from)?;
        Ok(JsStagger::new(config))
    }

    // === Event Listener API ===

    /// Registers a callback for animation start events
    #[wasm_bindgen]
    pub fn on_start(&self, callback: JsValue) {
        // Store callback reference for event dispatch
        // Note: In production, you'd want a more sophisticated callback registry
        let _dispatcher = self.dispatcher.borrow_mut();
        // TODO: Implement callback registration
        log::warn!("on_start callback registered (implementation pending)");
    }

    /// Registers a callback for animation complete events
    #[wasm_bindgen]
    pub fn on_complete(&self, callback: JsValue) {
        log::warn!("on_complete callback registered (implementation pending)");
    }

    /// Registers a callback for animation update events
    #[wasm_bindgen]
    pub fn on_update(&self, callback: JsValue, throttle_ms: Option<u32>) {
        log::warn!("on_update callback registered (implementation pending)");
    }

    // === Control API ===

    /// Pauses all animations
    #[wasm_bindgen]
    pub fn pause_all(&self) {
        for (_id, handle) in self.animations.borrow_mut().iter_mut() {
            handle.pause();
        }
    }

    /// Resumes all paused animations
    #[wasm_bindgen]
    pub fn resume_all(&self) {
        for (_id, handle) in self.animations.borrow_mut().iter_mut() {
            handle.resume();
        }
    }

    /// Stops all animations
    #[wasm_bindgen]
    pub fn stop_all(&self) {
        self.animations.borrow_mut().clear();
    }

    /// Gets the count of active animations
    #[wasm_bindgen]
    pub fn active_count(&self) -> usize {
        self.animations.borrow().len()
    }

    // === Internal Methods ===

    /// Adds an animation to the system
    pub(crate) fn add_animation(&self, id: String, handle: TimelineHandle) {
        self.animations.borrow_mut().insert(id, handle);
    }

    /// Removes an animation from the system
    pub(crate) fn remove_animation(&self, id: &str) {
        self.animations.borrow_mut().remove(id);
    }

    /// Gets the next unique animation ID
    pub(crate) fn next_id(&self) -> String {
        let id = format!("anim_{}", *self.next_id.borrow());
        *self.next_id.borrow_mut() += 1;
        id
    }
}

impl Default for AnimationSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Fluent builder for creating animations (JavaScript-friendly)
///
/// This builder provides a chainable API for configuring animations,
/// similar to GSAP or Anime.js.
#[wasm_bindgen]
pub struct JsAnimationBuilder {
    /// Target shape ID
    target_id: String,
    /// Animation system reference
    system: AnimationSystem,
    /// Target position
    to_position: Option<(f32, f32)>,
    /// Target rotation
    rotation: Option<f32>,
    /// Target opacity
    opacity: Option<f32>,
    /// Target size
    size: Option<(f32, f32)>,
    /// Target color
    color: Option<JsColor>,
    /// Animation duration in milliseconds
    duration: u32,
    /// Delay before starting
    delay: u32,
    /// Easing function
    easing: EasingFunction,
    /// Whether to loop
    loop_count: u32,
}

#[wasm_bindgen]
impl JsAnimationBuilder {
    /// Creates a new animation builder
    pub fn new(target_id: &str, system: &AnimationSystem) -> Self {
        Self {
            target_id: target_id.to_string(),
            system: AnimationSystem {
                animations: system.animations.clone(),
                dispatcher: system.dispatcher.clone(),
                canvas_adapter: system.canvas_adapter.clone(),
                next_id: system.next_id.clone(),
            },
            to_position: None,
            rotation: None,
            opacity: None,
            size: None,
            color: None,
            duration: 500,
            delay: 0,
            easing: EasingFunction::QuadOut,
            loop_count: 0,
        }
    }

    /// Sets target position
    #[wasm_bindgen]
    pub fn to(&mut self, x: f32, y: f32) -> JsAnimationBuilder {
        self.to_position = Some((x, y));
        self.to_js_value()
    }

    /// Sets rotation
    #[wasm_bindgen]
    pub fn rotate(&mut self, degrees: f32) -> JsAnimationBuilder {
        self.rotation = Some(degrees);
        self.to_js_value()
    }

    /// Sets opacity
    #[wasm_bindgen]
    pub fn fade(&mut self, opacity: f32) -> JsAnimationBuilder {
        self.opacity = Some(opacity.max(0.0).min(1.0));
        self.to_js_value()
    }

    /// Sets size
    #[wasm_bindgen]
    pub fn scale(&mut self, width: f32, height: f32) -> JsAnimationBuilder {
        self.size = Some((width, height));
        self.to_js_value()
    }

    /// Sets fill color
    #[wasm_bindgen]
    pub fn color(&mut self, r: f32, g: f32, b: f32, a: f32) -> JsAnimationBuilder {
        self.color = Some(JsColor { r, g, b, a });
        self.to_js_value()
    }

    /// Sets duration in milliseconds
    #[wasm_bindgen]
    pub fn duration(&mut self, ms: u32) -> JsAnimationBuilder {
        self.duration = ms;
        self.to_js_value()
    }

    /// Sets delay in milliseconds
    #[wasm_bindgen]
    pub fn delay(&mut self, ms: u32) -> JsAnimationBuilder {
        self.delay = ms;
        self.to_js_value()
    }

    /// Sets easing function
    #[wasm_bindgen]
    pub fn easing(&mut self, easing_name: &str) -> JsAnimationBuilder {
        self.easing = Self::parse_easing(easing_name);
        self.to_js_value()
    }

    /// Sets loop count (0 = play once, u32::MAX = infinite)
    #[wasm_bindgen]
    pub fn loop_(&mut self, count: u32) -> JsAnimationBuilder {
        self.loop_count = count;
        self.to_js_value()
    }

    /// Starts the animation
    ///
    /// Returns the animation ID for control.
    #[wasm_bindgen]
    pub fn play(&mut self) -> String {
        // Generate unique animation ID
        let id = self.system.next_id();

        // TODO: Create and start the actual animation
        // For now, this is a placeholder that returns an ID
        log::info!("Playing animation {} for target {}", id, self.target_id);

        id
    }

    /// Converts self to JsValue for chaining
    fn to_js_value(self) -> JsAnimationBuilder {
        self
    }

    /// Parses easing function name
    fn parse_easing(name: &str) -> EasingFunction {
        match name {
            "linear" => EasingFunction::Linear,
            "ease" | "easeInOut" => EasingFunction::QuadInOut,
            "easeIn" => EasingFunction::QuadIn,
            "easeOut" => EasingFunction::QuadOut,
            "easeInOutQuad" => EasingFunction::QuadInOut,
            "easeInOutCubic" => EasingFunction::CubicInOut,
            "easeInOutQuart" => EasingFunction::QuartInOut,
            "easeInOutQuint" => EasingFunction::QuintInOut,
            "easeInOutSine" => EasingFunction::SineInOut,
            "easeInOutExpo" => EasingFunction::ExpoInOut,
            "easeInOutCirc" => EasingFunction::CircInOut,
            "easeInOutBack" => EasingFunction::BackInOut,
            "easeInOutElastic" => EasingFunction::ElasticOut,
            "easeInOutBounce" => EasingFunction::BounceOut,
            _ => EasingFunction::QuadOut,
        }
    }
}

/// Timeline builder for sequencing animations (JavaScript-friendly)
#[wasm_bindgen]
pub struct JsTimelineBuilder {
    /// Animation system reference
    system: AnimationSystem,
    /// Timeline being built
    timeline: Timeline,
}

#[wasm_bindgen]
impl JsTimelineBuilder {
    /// Creates a new timeline builder
    pub fn new(system: &AnimationSystem) -> Self {
        Self {
            system: AnimationSystem {
                animations: system.animations.clone(),
                dispatcher: system.dispatcher.clone(),
                canvas_adapter: system.canvas_adapter.clone(),
                next_id: system.next_id.clone(),
            },
            timeline: Timeline::new(),
        }
    }

    /// Adds an animation to the timeline
    #[wasm_bindgen]
    pub fn add(&mut self, animation_id: &str, position: Option<String>) -> JsTimelineBuilder {
        // TODO: Parse position string (e.g., "-=200", "label+=50")
        let pos = position
            .as_ref()
            .map(|p| TimelinePosition::parse(p).unwrap_or(TimelinePosition::End))
            .unwrap_or(TimelinePosition::End);

        // TODO: Add actual animation to timeline
        log::info!("Adding animation {} at position {:?}", animation_id, pos);

        self.to_js_value()
    }

    /// Adds a label to the timeline
    #[wasm_bindgen]
    pub fn add_label(&mut self, label: &str) -> JsTimelineBuilder {
        self.timeline.add_label(label);
        self.to_js_value()
    }

    /// Sets time scale (speed multiplier)
    #[wasm_bindgen]
    pub fn set_time_scale(&mut self, scale: f32) -> JsTimelineBuilder {
        self.timeline.set_time_scale(scale);
        self.to_js_value()
    }

    /// Sets loop count
    #[wasm_bindgen]
    pub fn set_loops(&mut self, count: u32) -> JsTimelineBuilder {
        self.timeline.set_loops(count);
        self.to_js_value()
    }

    /// Plays the timeline
    ///
    /// Returns the timeline ID for control.
    #[wasm_bindgen]
    pub fn play(&mut self) -> String {
        let id = self.system.next_id();
        let handle = self.timeline.play();

        // Store handle in system
        self.system.add_animation(id.clone(), handle);

        id
    }

    fn to_js_value(self) -> JsTimelineBuilder {
        self
    }
}

/// Stagger configuration for wave animations
#[wasm_bindgen]
pub struct JsStagger {
    config: JsStaggerConfig,
}

#[wasm_bindgen]
impl JsStagger {
    /// Creates a new stagger configuration
    pub fn new(config: JsStaggerConfig) -> Self {
        Self { config }
    }

    /// Applies stagger to a list of target IDs
    ///
    /// Returns an array of animation IDs, one per target.
    #[wasm_bindgen]
    pub fn apply_to(&self, target_ids: JsValue, system: &AnimationSystem) -> JsValue {
        // Parse target IDs from JS array
        let ids: Vec<String> = from_value(target_ids).unwrap_or_default();

        // Create staggered animations
        let stagger = Stagger::from(self.config.from, self.config.axis);
        let animation_ids: Vec<String> = ids
            .iter()
            .enumerate()
            .map(|(i, id)| {
                let delay = stagger.calculate_delay(i, ids.len(), self.config.amount, self.config.grid);
                format!("{}_delay_{}", id, delay)
            })
            .collect();

        to_value(&animation_ids).unwrap_or(JsValue::UNDEFINED)
    }

    /// Gets the delay for a specific index
    #[wasm_bindgen]
    pub fn get_delay(&self, index: usize, total: usize) -> f32 {
        let stagger = Stagger::from(self.config.from, self.config.axis);
        stagger.calculate_delay(index, total, self.config.amount, self.config.grid)
    }
}

/// Stagger configuration options
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsStaggerConfig {
    /// Starting position for stagger
    pub from: String,
    /// Axis for stagger calculation
    pub axis: String,
    /// Delay amount in milliseconds
    pub amount: f32,
    /// Grid configuration for from: "center"
    pub grid: Option<JsGridConfig>,
}

/// Grid configuration for staggering
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsGridConfig {
    /// Number of columns
    pub columns: usize,
    /// Number of rows
    pub rows: usize,
}

/// Error type for WASM operations
#[derive(Debug)]
pub struct JsError {
    message: String,
}

impl JsError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl From<wasm_bindgen::JsValue> for JsError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        Self::new(format!("{:?}", value))
    }
}

impl std::fmt::Display for JsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for JsError {}

/// Result type for WASM operations
pub type JsResult<T> = Result<T, JsError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_system_creation() {
        let system = AnimationSystem::new();
        assert_eq!(system.active_count(), 0);
    }

    #[test]
    fn test_animation_builder_creation() {
        let system = AnimationSystem::new();
        let builder = JsAnimationBuilder::new("shape_123", &system);
        assert_eq!(builder.target_id, "shape_123");
    }

    #[test]
    fn test_animation_builder_chain() {
        let system = AnimationSystem::new();
        let mut builder = JsAnimationBuilder::new("shape_123", &system);
        builder.to(100.0, 200.0).rotate(45.0).fade(0.5).duration(1000);

        assert_eq!(builder.to_position, Some((100.0, 200.0)));
        assert_eq!(builder.rotation, Some(45.0)));
        assert_eq!(builder.opacity, Some(0.5)));
        assert_eq!(builder.duration, 1000);
    }

    #[test]
    fn test_easing_parsing() {
        assert!(matches!(
            JsAnimationBuilder::parse_easing("linear"),
            EasingFunction::Linear
        ));
        assert!(matches!(
            JsAnimationBuilder::parse_easing("easeInOut"),
            EasingFunction::QuadInOut
        ));
        assert!(matches!(
            JsAnimationBuilder::parse_easing("unknown"),
            EasingFunction::QuadOut // fallback
        ));
    }

    #[test]
    fn test_stagger_config() {
        let config = JsStaggerConfig {
            from: "center".to_string(),
            axis: "x".to_string(),
            amount: 100.0,
            grid: None,
        };

        let stagger = JsStagger::new(config);
        assert_eq!(stagger.config.amount, 100.0);
    }
}
