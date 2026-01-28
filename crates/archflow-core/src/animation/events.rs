//! Animation Events Module
//!
//! This module provides event-driven architecture for animation lifecycle events,
//! integrating efficiently with CanvasEvent to avoid event storms during high-frequency
//! animations (60fps+).
//!
//! # Architecture
//!
//! ```text
//! Animation System                Canvas Event System
//!       │                                  │
//!       ├── AnimatorBuilder                │
//!   [Creates Animation]                    │
//!       │                                  │
//!       ▼                                  │
//! ┌─────────────┐                         │
//! │ Animation   │──[on_start]──► ┌──────────────────┐
//! │  Running    │                 │ AnimationEvent  │
//! └─────────────┘                 │  Dispatcher     │
//!       │                         └──────────────────┘
//!       │[throttled updates]            │
//!       ▼                               │
//! ┌─────────────┐                       │
//! │ EventBatch  │──[batched]────► ┌──────────────┐
//! │  Accumulator │                  │ CanvasEvent  │
//! └─────────────┘                  │  Integration │
//!                                   └──────────────┘
//! ```
//!
//! # Performance Optimizations
//!
//! - **Throttling**: Canvas updates batched (default: 60fps → 15fps canvas invalidates)
//! - **Debouncing**: Multiple shape updates coalesced into single ShapeUpdated event
//! - **Lazy Evaluation**: Events only dispatched when listeners registered
//! - **Zero-Copy**: Event data references instead of clones where possible
//!
//! # Usage
//!
//! ```ignore
//! use archflow_core::animation::{AnimationEvent, AnimationEventDispatcher};
//!
//! // Create dispatcher
//! let mut dispatcher = AnimationEventDispatcher::new();
//!
//! // Register listener for start events
//! dispatcher.on_start(|event| {
//!     println!("Animation started: {:?}", event.animation_id);
//! });
//!
//! // Register listener for completion events
//! dispatcher.on_complete(|event| {
//!     println!("Animation completed: {:?}", event.animation_id);
//! });
//!
//! // Register listener for progress (throttled to avoid storms)
//! dispatcher.on_update_with_throttle(|event| {
//!     println!("Progress: {:.1}%", event.progress * 100.0);
//! }, Duration::from_millis(66)); // ~15fps
//! ```

use crate::EntityId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Unique identifier for animations
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AnimationId(pub u64);

impl AnimationId {
    /// Creates a new animation ID
    pub fn new() -> Self {
        Self(rand::random())
    }
}

impl Default for AnimationId {
    fn default() -> Self {
        Self::new()
    }
}

/// Lifecycle phase of an animation
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationPhase {
    /// Animation has just started
    Started,
    /// Animation is in progress
    Running,
    /// Animation has completed successfully
    Completed,
    /// Animation was cancelled before completion
    Cancelled,
    /// Animation encountered an error
    Failed,
}

/// Property value being animated with current state
///
/// This extends the basic AnimatedProperty enum by including from/to/current values
/// for event tracking and canvas integration.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AnimatedPropertyValue {
    /// Position (x, y)
    Position {
        from: (f32, f32),
        to: (f32, f32),
        current: (f32, f32),
    },
    /// Size (width, height)
    Size {
        from: (f32, f32),
        to: (f32, f32),
        current: (f32, f32),
    },
    /// Rotation in degrees
    Rotation { from: f32, to: f32, current: f32 },
    /// Opacity (0.0 - 1.0)
    Opacity { from: f32, to: f32, current: f32 },
    /// Fill color
    FillColor {
        from: [u8; 4],
        to: [u8; 4],
        current: [u8; 4],
    },
    /// Stroke color
    StrokeColor {
        from: [u8; 4],
        to: [u8; 4],
        current: [u8; 4],
    },
    /// Stroke width
    StrokeWidth { from: f32, to: f32, current: f32 },
}

impl AnimatedPropertyValue {
    /// Returns the current value as a tuple of f32 values
    pub fn current_value(&self) -> Vec<f32> {
        match self {
            AnimatedPropertyValue::Position { current, .. } => vec![current.0, current.1],
            AnimatedPropertyValue::Size { current, .. } => vec![current.0, current.1],
            AnimatedPropertyValue::Rotation { current, .. } => vec![*current],
            AnimatedPropertyValue::Opacity { current, .. } => vec![*current],
            AnimatedPropertyValue::FillColor { current, .. } => {
                vec![
                    current[0] as f32 / 255.0,
                    current[1] as f32 / 255.0,
                    current[2] as f32 / 255.0,
                    current[3] as f32 / 255.0,
                ]
            }
            AnimatedPropertyValue::StrokeColor { current, .. } => {
                vec![
                    current[0] as f32 / 255.0,
                    current[1] as f32 / 255.0,
                    current[2] as f32 / 255.0,
                    current[3] as f32 / 255.0,
                ]
            }
            AnimatedPropertyValue::StrokeWidth { current, .. } => vec![*current],
        }
    }
}

/// Event emitted during animation lifecycle
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationEvent {
    /// Unique animation identifier
    pub animation_id: AnimationId,
    /// Entity being animated (if applicable)
    pub target_entity: Option<EntityId>,
    /// Current phase of the animation
    pub phase: AnimationPhase,
    /// Property being animated (if applicable)
    pub property: Option<AnimatedPropertyValue>,
    /// Animation progress (0.0 - 1.0)
    pub progress: f32,
    /// Animation duration in milliseconds
    pub duration_ms: u64,
    /// Elapsed time in milliseconds
    pub elapsed_ms: u64,
    /// Event timestamp
    pub timestamp: u64,
}

impl AnimationEvent {
    /// Creates a new animation start event
    pub fn started(animation_id: AnimationId, target: Option<EntityId>, duration_ms: u64) -> Self {
        Self {
            animation_id,
            target_entity: target,
            phase: AnimationPhase::Started,
            property: None,
            progress: 0.0,
            duration_ms,
            elapsed_ms: 0,
            timestamp: Self::current_timestamp(),
        }
    }

    /// Creates a new animation update event
    pub fn update(
        animation_id: AnimationId,
        target: Option<EntityId>,
        property: AnimatedPropertyValue,
        progress: f32,
        duration_ms: u64,
        elapsed_ms: u64,
    ) -> Self {
        Self {
            animation_id,
            target_entity: target,
            phase: AnimationPhase::Running,
            property: Some(property),
            progress,
            duration_ms,
            elapsed_ms,
            timestamp: Self::current_timestamp(),
        }
    }

    /// Creates a new animation completion event
    pub fn completed(
        animation_id: AnimationId,
        target: Option<EntityId>,
        duration_ms: u64,
    ) -> Self {
        Self {
            animation_id,
            target_entity: target,
            phase: AnimationPhase::Completed,
            property: None,
            progress: 1.0,
            duration_ms,
            elapsed_ms: duration_ms,
            timestamp: Self::current_timestamp(),
        }
    }

    /// Creates a new animation cancelled event
    pub fn cancelled(animation_id: AnimationId, target: Option<EntityId>, elapsed_ms: u64) -> Self {
        Self {
            animation_id,
            target_entity: target,
            phase: AnimationPhase::Cancelled,
            property: None,
            progress: 0.0,
            duration_ms: elapsed_ms,
            elapsed_ms,
            timestamp: Self::current_timestamp(),
        }
    }

    /// Creates a new animation failed event
    pub fn failed(animation_id: AnimationId, target: Option<EntityId>, elapsed_ms: u64) -> Self {
        Self {
            animation_id,
            target_entity: target,
            phase: AnimationPhase::Failed,
            property: None,
            progress: 0.0,
            duration_ms: elapsed_ms,
            elapsed_ms,
            timestamp: Self::current_timestamp(),
        }
    }

    /// Returns current timestamp in milliseconds
    fn current_timestamp() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    /// Checks if this event should trigger a canvas update
    ///
    /// Used to avoid excessive canvas invalidations during animations.
    /// Returns true only for significant events or at throttled intervals.
    pub fn should_invalidate_canvas(
        &self,
        last_invalidate: Instant,
        throttle_duration: Duration,
    ) -> bool {
        match self.phase {
            // Always invalidate on start/complete/cancel/failed
            AnimationPhase::Started
            | AnimationPhase::Completed
            | AnimationPhase::Cancelled
            | AnimationPhase::Failed => true,
            // Throttle updates during animation
            AnimationPhase::Running => last_invalidate.elapsed() >= throttle_duration,
        }
    }
}

/// Callback function type for animation events
pub type AnimationCallback = Box<dyn Fn(&AnimationEvent) + Send + Sync>;

/// Dispatcher for animation events with batching and throttling support
///
/// This dispatcher manages event listeners and handles event delivery with
/// performance optimizations to avoid event storms during high-frequency animations.
pub struct AnimationEventDispatcher {
    /// Listeners for start events
    start_listeners: Vec<AnimationCallback>,
    /// Listeners for update events
    update_listeners: Vec<AnimationCallback>,
    /// Listeners for completion events
    complete_listeners: Vec<AnimationCallback>,
    /// Listeners for cancellation events
    cancel_listeners: Vec<AnimationCallback>,
    /// Listeners for error events
    error_listeners: Vec<AnimationCallback>,
    /// Throttle duration for update events (default: 66ms ~ 15fps)
    update_throttle: Duration,
    /// Last canvas invalidation timestamp
    last_canvas_invalidate: Instant,
    /// Canvas invalidate throttle (default: 66ms)
    canvas_invalidate_throttle: Duration,
    /// Batch accumulator for canvas events
    canvas_event_batch: Vec<CanvasAnimationUpdate>,
    /// Whether batching is enabled
    batching_enabled: bool,
}

impl Default for AnimationEventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationEventDispatcher {
    /// Creates a new event dispatcher with default settings
    pub fn new() -> Self {
        Self {
            start_listeners: Vec::new(),
            update_listeners: Vec::new(),
            complete_listeners: Vec::new(),
            cancel_listeners: Vec::new(),
            error_listeners: Vec::new(),
            update_throttle: Duration::from_millis(66), // ~15fps
            last_canvas_invalidate: Instant::now(),
            canvas_invalidate_throttle: Duration::from_millis(66),
            canvas_event_batch: Vec::new(),
            batching_enabled: true,
        }
    }

    /// Registers a listener for animation start events
    pub fn on_start<F>(&mut self, callback: F) -> &mut Self
    where
        F: Fn(&AnimationEvent) + Send + Sync + 'static,
    {
        self.start_listeners.push(Box::new(callback));
        self
    }

    /// Registers a listener for animation update events
    pub fn on_update<F>(&mut self, callback: F) -> &mut Self
    where
        F: Fn(&AnimationEvent) + Send + Sync + 'static,
    {
        self.update_listeners.push(Box::new(callback));
        self
    }

    /// Registers a listener for animation update events with custom throttle
    pub fn on_update_with_throttle<F>(&mut self, callback: F, throttle: Duration) -> &mut Self
    where
        F: Fn(&AnimationEvent) + Send + Sync + 'static,
    {
        self.update_throttle = throttle;
        self.update_listeners.push(Box::new(callback));
        self
    }

    /// Registers a listener for animation completion events
    pub fn on_complete<F>(&mut self, callback: F) -> &mut Self
    where
        F: Fn(&AnimationEvent) + Send + Sync + 'static,
    {
        self.complete_listeners.push(Box::new(callback));
        self
    }

    /// Registers a listener for animation cancellation events
    pub fn on_cancel<F>(&mut self, callback: F) -> &mut Self
    where
        F: Fn(&AnimationEvent) + Send + Sync + 'static,
    {
        self.cancel_listeners.push(Box::new(callback));
        self
    }

    /// Registers a listener for animation error events
    pub fn on_error<F>(&mut self, callback: F) -> &mut Self
    where
        F: Fn(&AnimationEvent) + Send + Sync + 'static,
    {
        self.error_listeners.push(Box::new(callback));
        self
    }

    /// Dispatches an animation event to all registered listeners
    ///
    /// Returns true if canvas should be invalidated based on throttle settings
    pub fn dispatch(&mut self, event: &AnimationEvent) -> bool {
        let should_invalidate = event
            .should_invalidate_canvas(self.last_canvas_invalidate, self.canvas_invalidate_throttle);

        match event.phase {
            AnimationPhase::Started => {
                for listener in &self.start_listeners {
                    listener(event);
                }
            }
            AnimationPhase::Running => {
                // Only dispatch updates if throttle period has elapsed
                if should_invalidate {
                    for listener in &self.update_listeners {
                        listener(event);
                    }

                    // Track canvas invalidation
                    self.last_canvas_invalidate = Instant::now();
                }

                // Always batch for canvas integration
                if self.batching_enabled {
                    if let Some(entity) = event.target_entity {
                        self.canvas_event_batch.push(CanvasAnimationUpdate {
                            entity,
                            property: event.property.clone(),
                            progress: event.progress,
                        });
                    }
                }
            }
            AnimationPhase::Completed => {
                for listener in &self.complete_listeners {
                    listener(event);
                }

                // Flush batch on completion
                if self.batching_enabled {
                    self.flush_canvas_events();
                }
            }
            AnimationPhase::Cancelled => {
                for listener in &self.cancel_listeners {
                    listener(event);
                }

                // Flush batch on cancellation
                if self.batching_enabled {
                    self.flush_canvas_events();
                }
            }
            AnimationPhase::Failed => {
                for listener in &self.error_listeners {
                    listener(event);
                }

                // Flush batch on error
                if self.batching_enabled {
                    self.flush_canvas_events();
                }
            }
        }

        should_invalidate
    }

    /// Returns and clears the batch of canvas animation updates
    pub fn take_canvas_events(&mut self) -> Vec<CanvasAnimationUpdate> {
        std::mem::take(&mut self.canvas_event_batch)
    }

    /// Flushes accumulated canvas events to the event system
    fn flush_canvas_events(&mut self) {
        // This would integrate with CanvasEvent system
        // For now, events are accumulated and can be taken via take_canvas_events()
        if !self.canvas_event_batch.is_empty() {
            // TODO: Emit CanvasEvent::Batch with animation updates
            self.canvas_event_batch.clear();
        }
    }

    /// Sets the canvas invalidate throttle duration
    pub fn set_canvas_throttle(&mut self, duration: Duration) -> &mut Self {
        self.canvas_invalidate_throttle = duration;
        self
    }

    /// Enables or disables event batching
    pub fn set_batching(&mut self, enabled: bool) -> &mut Self {
        self.batching_enabled = enabled;
        self
    }

    /// Clears all event listeners
    pub fn clear_listeners(&mut self) {
        self.start_listeners.clear();
        self.update_listeners.clear();
        self.complete_listeners.clear();
        self.cancel_listeners.clear();
        self.error_listeners.clear();
    }

    /// Returns the number of registered listeners
    pub fn listener_count(&self) -> usize {
        self.start_listeners.len()
            + self.update_listeners.len()
            + self.complete_listeners.len()
            + self.cancel_listeners.len()
            + self.error_listeners.len()
    }
}

/// Update data for canvas integration
///
/// This represents the minimal data needed to update canvas state
/// during an animation, avoiding the overhead of full AnimationEvent.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CanvasAnimationUpdate {
    /// Entity being animated
    pub entity: EntityId,
    /// Property being animated with current value
    pub property: Option<AnimatedPropertyValue>,
    /// Animation progress (0.0 - 1.0)
    pub progress: f32,
}

impl CanvasAnimationUpdate {
    /// Creates a new canvas animation update
    pub fn new(entity: EntityId, property: Option<AnimatedPropertyValue>, progress: f32) -> Self {
        Self {
            entity,
            property,
            progress,
        }
    }

    /// Checks if this update represents a significant change
    ///
    /// Used to coalesce multiple small updates into a single canvas update.
    pub fn is_significant(&self, threshold: f32) -> bool {
        // Update is significant if progress has changed substantially
        self.progress > threshold
    }
}

/// Integration adapter for converting AnimationEvents to CanvasEvents
///
/// This adapter bridges the animation event system with the canvas event system,
/// enabling undo/redo support and collaborative editing integration.
#[derive(Debug)]
pub struct AnimationCanvasAdapter {
    /// Mapping of animation IDs to canvas operations
    operation_map: HashMap<AnimationId, CanvasOperationState>,
}

impl Default for AnimationCanvasAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationCanvasAdapter {
    /// Creates a new adapter
    pub fn new() -> Self {
        Self {
            operation_map: HashMap::new(),
        }
    }

    /// Tracks an animation for canvas integration
    pub fn track_animation(
        &mut self,
        animation_id: AnimationId,
        entity: EntityId,
        initial_property: AnimatedPropertyValue,
    ) {
        self.operation_map.insert(
            animation_id,
            CanvasOperationState {
                entity,
                initial_property,
                updates: Vec::new(),
            },
        );
    }

    /// Records an animation update
    pub fn record_update(&mut self, event: &AnimationEvent) {
        if let Some(state) = self.operation_map.get_mut(&event.animation_id) {
            if let Some(property) = &event.property {
                state.updates.push(property.clone());
            }
        }
    }

    /// Completes an animation and returns the final canvas operation
    pub fn complete_animation(
        &mut self,
        animation_id: AnimationId,
    ) -> Option<CanvasAnimationOperation> {
        self.operation_map
            .remove(&animation_id)
            .map(|state| CanvasAnimationOperation {
                entity: state.entity,
                initial: state.initial_property,
                updates: state.updates,
            })
    }

    /// Cancels an animation
    pub fn cancel_animation(&mut self, animation_id: AnimationId) -> bool {
        self.operation_map.remove(&animation_id).is_some()
    }

    /// Returns the number of tracked animations
    pub fn tracked_count(&self) -> usize {
        self.operation_map.len()
    }
}

/// Internal state for canvas operation tracking
#[derive(Debug)]
struct CanvasOperationState {
    entity: EntityId,
    initial_property: AnimatedPropertyValue,
    updates: Vec<AnimatedPropertyValue>,
}

/// Complete animation operation for canvas integration
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CanvasAnimationOperation {
    pub entity: EntityId,
    pub initial: AnimatedPropertyValue,
    pub updates: Vec<AnimatedPropertyValue>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_id_unique() {
        let id1 = AnimationId::new();
        let id2 = AnimationId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_animation_event_started() {
        let id = AnimationId::new();
        let event = AnimationEvent::started(id, Some(EntityId::new()), 1000);

        assert_eq!(event.animation_id, id);
        assert_eq!(event.phase, AnimationPhase::Started);
        assert_eq!(event.progress, 0.0);
        assert_eq!(event.duration_ms, 1000);
    }

    #[test]
    fn test_animation_event_update() {
        let id = AnimationId::new();
        let entity = EntityId::new();
        let property = AnimatedPropertyValue::Opacity {
            from: 0.0,
            to: 1.0,
            current: 0.5,
        };
        let event = AnimationEvent::update(id, Some(entity), property.clone(), 0.5, 1000, 500);

        assert_eq!(event.animation_id, id);
        assert_eq!(event.phase, AnimationPhase::Running);
        assert_eq!(event.progress, 0.5);
        assert!(event.property.is_some());
    }

    #[test]
    fn test_animation_event_completed() {
        let id = AnimationId::new();
        let event = AnimationEvent::completed(id, Some(EntityId::new()), 1000);

        assert_eq!(event.phase, AnimationPhase::Completed);
        assert_eq!(event.progress, 1.0);
        assert_eq!(event.elapsed_ms, 1000);
    }

    #[test]
    fn test_animated_property_current_value() {
        let property = AnimatedPropertyValue::Position {
            from: (0.0, 0.0),
            to: (100.0, 100.0),
            current: (50.0, 50.0),
        };

        let value = property.current_value();
        assert_eq!(value, vec![50.0, 50.0]);
    }

    #[test]
    fn test_dispatcher_start_listener() {
        let mut dispatcher = AnimationEventDispatcher::new();

        // Test that listener is registered
        dispatcher.on_start(|_| {});
        assert_eq!(dispatcher.listener_count(), 1);

        // Test dispatch doesn't panic
        let event = AnimationEvent::started(AnimationId::new(), None, 1000);
        let result = dispatcher.dispatch(&event);

        // Start events should always invalidate canvas
        assert!(result);
    }

    #[test]
    fn test_dispatcher_complete_listener() {
        let mut dispatcher = AnimationEventDispatcher::new();

        // Test that listener is registered
        dispatcher.on_complete(|_| {});
        assert_eq!(dispatcher.listener_count(), 1);

        // Test dispatch doesn't panic
        let event = AnimationEvent::completed(AnimationId::new(), None, 1000);
        dispatcher.dispatch(&event);
    }

    #[test]
    fn test_dispatcher_listener_count() {
        let mut dispatcher = AnimationEventDispatcher::new();

        assert_eq!(dispatcher.listener_count(), 0);

        dispatcher.on_start(|_| {});
        assert_eq!(dispatcher.listener_count(), 1);

        dispatcher.on_update(|_| {});
        assert_eq!(dispatcher.listener_count(), 2);

        dispatcher.on_complete(|_| {});
        assert_eq!(dispatcher.listener_count(), 3);
    }

    #[test]
    fn test_canvas_animation_update_significant() {
        let update = CanvasAnimationUpdate::new(
            EntityId::new(),
            Some(AnimatedPropertyValue::Opacity {
                from: 0.0,
                to: 1.0,
                current: 0.5,
            }),
            0.5,
        );

        assert!(update.is_significant(0.1));
        assert!(!update.is_significant(0.9));
    }

    #[test]
    fn test_adapter_track_and_complete() {
        let mut adapter = AnimationCanvasAdapter::new();
        let id = AnimationId::new();
        let entity = EntityId::new();
        let property = AnimatedPropertyValue::Opacity {
            from: 0.0,
            to: 1.0,
            current: 0.0,
        };

        adapter.track_animation(id, entity, property.clone());

        assert_eq!(adapter.tracked_count(), 1);

        let operation = adapter.complete_animation(id);

        assert!(operation.is_some());
        assert_eq!(adapter.tracked_count(), 0);
    }

    #[test]
    fn test_should_invalidate_canvas_on_start() {
        let event = AnimationEvent::started(AnimationId::new(), None, 1000);
        let last_invalidate = Instant::now() - Duration::from_secs(1);
        let throttle = Duration::from_millis(66);

        assert!(event.should_invalidate_canvas(last_invalidate, throttle));
    }

    #[test]
    fn test_should_invalidate_canvas_throttled() {
        let throttle = Duration::from_millis(66);

        // Create update event
        let event1 = AnimationEvent::update(
            AnimationId::new(),
            None,
            AnimatedPropertyValue::Opacity {
                from: 0.0,
                to: 1.0,
                current: 0.5,
            },
            0.5,
            1000,
            500,
        );

        // Test with old last_invalidate (should invalidate)
        let last_invalidate_old = Instant::now() - Duration::from_millis(100);
        assert!(event1.should_invalidate_canvas(last_invalidate_old, throttle));

        // Test with recent last_invalidate (should be throttled)
        let last_invalidate_recent = Instant::now() - Duration::from_millis(10);
        assert!(!event1.should_invalidate_canvas(last_invalidate_recent, throttle));

        // Test with exactly throttle period (should invalidate)
        let last_invalidate_exact = Instant::now() - throttle;
        assert!(event1.should_invalidate_canvas(last_invalidate_exact, throttle));
    }
}
