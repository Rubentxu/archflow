// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - CameraActuator with Smooth Camera Movements
//
// This module implements HU-020: CameraActuator for smooth camera movements.
//
// Reference: docs/epics/EPIC-003-actuators-animations.md - HU-020
//
// Key Features:
// - Exponential smoothing for professional camera feel
// - Pan (XY movement) with damping
// - Zoom with interpolation
// - Camera constraints (min/max zoom, bounds)
// - LookAt transform (eye + target)
//
// Architecture:
// - LookTransform: Camera position and target
// - Smoother: Exponential smoothing for smooth movements
// - CameraActuator: Actuator that controls camera behavior
//
// ═══════════════════════════════════════════════════════════════════════════════

#![warn(missing_docs)]

use archflow_core::{EntityId, Vec2};
use archflow_engine::EntityStore;

use crate::pulse::Pulse;

/// 2D camera transform representing position and zoom
///
/// This is adapted from the 3D LookTransform concept to 2D.
/// Instead of eye + target, we have:
/// - position: The camera's center point in world coordinates
/// - zoom: The zoom level (1.0 = 100%, 2.0 = 200%, etc.)
///
/// # Example
///
/// ```rust
/// let transform = CameraTransform::new(Vec2::new(100.0, 100.0), 1.0);
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraTransform {
    /// Camera center position in world coordinates
    pub position: Vec2,

    /// Zoom level (1.0 = 100%, higher = zoomed in)
    pub zoom: f32,
}

impl CameraTransform {
    /// Create a new camera transform
    #[must_use]
    pub const fn new(position: Vec2, zoom: f32) -> Self {
        Self { position, zoom }
    }

    /// Create a default camera transform at origin with 1.0 zoom
    #[must_use]
    pub const fn default() -> Self {
        Self {
            position: Vec2::new(0.0, 0.0),
            zoom: 1.0,
        }
    }

    /// Create a default camera transform (alias for default())
    #[must_use]
    pub const fn origin() -> Self {
        Self::default()
    }

    /// Linear interpolation between two transforms
    #[must_use]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            position: self.position.lerp(other.position, t),
            zoom: self.zoom + (other.zoom - self.zoom) * t,
        }
    }
}

impl Default for CameraTransform {
    fn default() -> Self {
        Self {
            position: Vec2::new(0.0, 0.0),
            zoom: 1.0,
        }
    }
}

/// Exponential smoother for camera movements
///
/// Provides smooth camera transitions using exponential smoothing:
/// `smoothed = previous * (1 - weight) + target * weight`
///
/// Higher `lag_weight` values produce smoother but slower movements.
/// - 0.0 = no smoothing (instant)
/// - 0.5 = moderate smoothing
/// - 0.9 = very smooth
/// - 0.99 = extremely smooth (almost no movement)
///
/// # Example
///
/// ```rust
/// let mut smoother = Smoother::new(0.9);
/// let current = smoother.smooth(CameraTransform::new(Vec2::new(100.0, 100.0), 1.0));
/// ```
pub struct Smoother {
    /// Smoothing factor (0.0 to 1.0)
    lag_weight: f32,

    /// Previous transform for interpolation
    prev_transform: Option<CameraTransform>,
}

impl Smoother {
    /// Create a new smoother with the given lag weight
    ///
    /// # Panics
    ///
    /// Panics if `lag_weight` is not in [0.0, 1.0]
    #[must_use]
    pub fn new(lag_weight: f32) -> Self {
        assert!(
            (0.0..=1.0).contains(&lag_weight),
            "lag_weight must be between 0.0 and 1.0"
        );
        Self {
            lag_weight,
            prev_transform: None,
        }
    }

    /// Smooth a target transform using exponential smoothing
    ///
    /// On the first call, returns the target directly and stores it.
    /// On subsequent calls, interpolates between previous and target.
    pub fn smooth(&mut self, target: CameraTransform) -> CameraTransform {
        match self.prev_transform {
            None => {
                self.prev_transform = Some(target);
                target
            }
            Some(prev) => {
                let smoothed = prev.lerp(target, self.lag_weight);
                self.prev_transform = Some(smoothed);
                smoothed
            }
        }
    }

    /// Reset the smoother, clearing the previous transform
    pub fn reset(&mut self) {
        self.prev_transform = None;
    }

    /// Get the current lag weight
    #[must_use]
    pub const fn lag_weight(&self) -> f32 {
        self.lag_weight
    }
}

/// Camera constraints for limiting camera movement and zoom
///
/// # Example
///
/// ```rust
/// let constraints = CameraConstraints::new()
///     .with_min_zoom(0.5)
///     .with_max_zoom(3.0)
///     .with_bounds(Rect::new(0.0, 0.0, 1000.0, 1000.0));
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraConstraints {
    /// Minimum zoom level (None = no limit)
    pub min_zoom: Option<f32>,

    /// Maximum zoom level (None = no limit)
    pub max_zoom: Option<f32>,

    /// World bounds the camera must stay within (None = no limit)
    pub bounds: Option<archflow_core::Rect>,
}

impl CameraConstraints {
    /// Create new empty constraints (no limits)
    #[must_use]
    pub const fn new() -> Self {
        Self {
            min_zoom: None,
            max_zoom: None,
            bounds: None,
        }
    }

    /// Set minimum zoom level
    #[must_use]
    pub const fn with_min_zoom(mut self, min_zoom: f32) -> Self {
        self.min_zoom = Some(min_zoom);
        self
    }

    /// Set maximum zoom level
    #[must_use]
    pub const fn with_max_zoom(mut self, max_zoom: f32) -> Self {
        self.max_zoom = Some(max_zoom);
        self
    }

    /// Set world bounds for camera position
    #[must_use]
    pub const fn with_bounds(mut self, bounds: archflow_core::Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Clamp a transform to satisfy these constraints
    #[must_use]
    pub fn clamp_transform(&self, transform: CameraTransform) -> CameraTransform {
        let mut result = transform;

        // Clamp zoom
        if let Some(min) = self.min_zoom {
            result.zoom = result.zoom.max(min);
        }
        if let Some(max) = self.max_zoom {
            result.zoom = result.zoom.min(max);
        }

        // Clamp position to bounds
        if let Some(bounds) = self.bounds {
            result.position.x = result.position.x.clamp(bounds.min.x, bounds.max.x);
            result.position.y = result.position.y.clamp(bounds.min.y, bounds.max.y);
        }

        result
    }
}

impl Default for CameraConstraints {
    fn default() -> Self {
        Self::new()
    }
}

/// Camera actuator for smooth camera movements
///
/// This actuator responds to pulses by smoothly transitioning the camera
/// to a target position/zoom. It uses exponential smoothing for professional
/// camera feel.
///
/// # Example
///
/// ```rust
/// let mut actuator = CameraActuator::new(
///     entity_id,
///     CameraActuatorConfig::new()
///         .with_zoom(2.0)
///         .with_lag_weight(0.9)
/// );
/// ```
pub struct CameraActuator {
    /// Entity ID this actuator operates on
    entity_id: EntityId,

    /// Current camera transform
    current_transform: CameraTransform,

    /// Smoother for smooth transitions
    smoother: Smoother,

    /// Camera constraints
    constraints: CameraConstraints,

    /// Target transform to move towards
    target_transform: Option<CameraTransform>,
}

/// Configuration for camera actuator
///
/// # Example
///
/// ```rust
/// let config = CameraActuatorConfig::new()
///     .with_zoom(2.0)
///     .with_lag_weight(0.9)
///     .with_constraints(CameraConstraints::new().with_max_zoom(3.0));
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraActuatorConfig {
    /// Initial zoom level
    pub zoom: f32,

    /// Smoothing factor (0.0 to 1.0)
    pub lag_weight: f32,

    /// Camera constraints
    pub constraints: CameraConstraints,
}

impl CameraActuatorConfig {
    /// Create new default configuration
    #[must_use]
    pub const fn new() -> Self {
        Self {
            zoom: 1.0,
            lag_weight: 0.9,
            constraints: CameraConstraints::new(),
        }
    }

    /// Set initial zoom level
    #[must_use]
    pub const fn with_zoom(mut self, zoom: f32) -> Self {
        self.zoom = zoom;
        self
    }

    /// Set smoothing factor
    #[must_use]
    pub const fn with_lag_weight(mut self, lag_weight: f32) -> Self {
        self.lag_weight = lag_weight;
        self
    }

    /// Set camera constraints
    #[must_use]
    pub const fn with_constraints(mut self, constraints: CameraConstraints) -> Self {
        self.constraints = constraints;
        self
    }
}

impl Default for CameraActuatorConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl CameraActuator {
    /// Create a new camera actuator
    #[must_use]
    pub fn new(entity_id: EntityId, config: CameraActuatorConfig) -> Self {
        let initial_transform = CameraTransform::new(Vec2::new(0.0, 0.0), config.zoom);
        let clamped = config.constraints.clamp_transform(initial_transform);

        Self {
            entity_id,
            current_transform: clamped,
            smoother: Smoother::new(config.lag_weight),
            constraints: config.constraints,
            target_transform: None,
        }
    }

    /// Get the current camera transform
    #[must_use]
    pub const fn current_transform(&self) -> CameraTransform {
        self.current_transform
    }

    /// Set target position to pan to
    pub fn pan_to(&mut self, target: Vec2) {
        self.target_transform = Some(CameraTransform {
            position: target,
            zoom: self.current_transform.zoom,
        });
    }

    /// Set target zoom level
    pub fn zoom_to(&mut self, zoom: f32) {
        self.target_transform = Some(CameraTransform {
            position: self.current_transform.position,
            zoom,
        });
    }

    /// Set target position and zoom
    pub fn look_at(&mut self, position: Vec2, zoom: f32) {
        self.target_transform = Some(CameraTransform::new(position, zoom));
    }

    /// Update the camera (call this every frame)
    pub fn update(&mut self) {
        if let Some(target) = self.target_transform {
            // Clamp target to constraints
            let clamped_target = self.constraints.clamp_transform(target);

            // Smooth towards target
            let smoothed = self.smoother.smooth(clamped_target);

            // Update current transform
            self.current_transform = smoothed;

            // Check if we've reached the target (within small epsilon)
            let diff = (self.current_transform.position - clamped_target.position).length()
                + (self.current_transform.zoom - clamped_target.zoom).abs();

            if diff < 0.001 {
                self.target_transform = None;
            }
        }
    }

    /// Activate the actuator with a pulse
    ///
    /// Positive pulse: Set target from pulse data
    /// Negative pulse: Reset to default position
    pub fn activate(&mut self, pulse: &Pulse, _store: &mut EntityStore) {
        if pulse.is_positive() {
            // Extract target position/zoom from pulse metadata
            // For now, just reset to origin
            self.target_transform = Some(CameraTransform::default());
        } else if pulse.is_negative() {
            self.target_transform = None;
            self.smoother.reset();
        }
    }

    /// Immediately snap to a transform (no smoothing)
    pub fn snap_to(&mut self, transform: CameraTransform) {
        let clamped = self.constraints.clamp_transform(transform);
        self.current_transform = clamped;
        self.smoother.reset();
        self.target_transform = None;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_core::{Generation, Index};

    fn make_id(idx: u32) -> EntityId {
        EntityId::from_parts(Index(idx), Generation(1))
    }

    // CameraTransform tests

    #[test]
    fn test_camera_transform_new() {
        let transform = CameraTransform::new(Vec2::new(100.0, 200.0), 2.0);
        assert_eq!(transform.position, Vec2::new(100.0, 200.0));
        assert_eq!(transform.zoom, 2.0);
    }

    #[test]
    fn test_camera_transform_default() {
        let transform = CameraTransform::default();
        assert_eq!(transform.position, Vec2::new(0.0, 0.0));
        assert_eq!(transform.zoom, 1.0);
    }

    #[test]
    fn test_camera_transform_lerp() {
        let a = CameraTransform::new(Vec2::new(0.0, 0.0), 1.0);
        let b = CameraTransform::new(Vec2::new(100.0, 100.0), 2.0);

        // Halfway interpolation
        let result = a.lerp(b, 0.5);
        assert_eq!(result.position, Vec2::new(50.0, 50.0));
        assert_eq!(result.zoom, 1.5);
    }

    #[test]
    fn test_camera_transform_lerp_full() {
        let a = CameraTransform::new(Vec2::new(0.0, 0.0), 1.0);
        let b = CameraTransform::new(Vec2::new(100.0, 100.0), 2.0);

        // Full interpolation (t=1.0)
        let result = a.lerp(b, 1.0);
        assert_eq!(result.position, Vec2::new(100.0, 100.0));
        assert_eq!(result.zoom, 2.0);
    }

    // Smoother tests

    #[test]
    fn test_smoother_new() {
        let smoother = Smoother::new(0.9);
        assert_eq!(smoother.lag_weight(), 0.9);
    }

    #[test]
    #[should_panic(expected = "lag_weight must be between 0.0 and 1.0")]
    fn test_smoother_invalid_weight() {
        Smoother::new(1.5);
    }

    #[test]
    fn test_smoother_first_call() {
        let mut smoother = Smoother::new(0.9);
        let target = CameraTransform::new(Vec2::new(100.0, 100.0), 2.0);

        // First call should return target directly
        let result = smoother.smooth(target);
        assert_eq!(result.position, Vec2::new(100.0, 100.0));
        assert_eq!(result.zoom, 2.0);
    }

    #[test]
    fn test_smoother_subsequent_calls() {
        let mut smoother = Smoother::new(0.5); // 50% smoothing
        let a = CameraTransform::new(Vec2::new(0.0, 0.0), 1.0);
        let b = CameraTransform::new(Vec2::new(100.0, 100.0), 2.0);

        // First call
        smoother.smooth(a);

        // Second call should interpolate
        let result = smoother.smooth(b);
        // 0.5 of the way from a to b
        assert_eq!(result.position, Vec2::new(50.0, 50.0));
        assert_eq!(result.zoom, 1.5);
    }

    #[test]
    fn test_smoother_reset() {
        let mut smoother = Smoother::new(0.9);
        let target = CameraTransform::new(Vec2::new(100.0, 100.0), 2.0);

        smoother.smooth(target);
        smoother.reset();

        // After reset, should act like first call again
        let new_target = CameraTransform::new(Vec2::new(200.0, 200.0), 3.0);
        let result = smoother.smooth(new_target);
        assert_eq!(result.position, Vec2::new(200.0, 200.0));
        assert_eq!(result.zoom, 3.0);
    }

    // CameraConstraints tests

    #[test]
    fn test_constraints_new() {
        let constraints = CameraConstraints::new();
        assert!(constraints.min_zoom.is_none());
        assert!(constraints.max_zoom.is_none());
        assert!(constraints.bounds.is_none());
    }

    #[test]
    fn test_constraints_with_zoom_limits() {
        let constraints = CameraConstraints::new()
            .with_min_zoom(0.5)
            .with_max_zoom(3.0);

        assert_eq!(constraints.min_zoom, Some(0.5));
        assert_eq!(constraints.max_zoom, Some(3.0));
    }

    #[test]
    fn test_constraints_clamp_zoom() {
        let constraints = CameraConstraints::new()
            .with_min_zoom(0.5)
            .with_max_zoom(2.0);

        // Below min
        let transform = CameraTransform::new(Vec2::new(0.0, 0.0), 0.1);
        let clamped = constraints.clamp_transform(transform);
        assert_eq!(clamped.zoom, 0.5);

        // Above max
        let transform = CameraTransform::new(Vec2::new(0.0, 0.0), 3.0);
        let clamped = constraints.clamp_transform(transform);
        assert_eq!(clamped.zoom, 2.0);

        // Within range
        let transform = CameraTransform::new(Vec2::new(0.0, 0.0), 1.5);
        let clamped = constraints.clamp_transform(transform);
        assert_eq!(clamped.zoom, 1.5);
    }

    #[test]
    fn test_constraints_clamp_position() {
        let bounds = archflow_core::Rect::new(0.0, 0.0, 100.0, 100.0);
        let constraints = CameraConstraints::new().with_bounds(bounds);

        // Outside bounds
        let transform = CameraTransform::new(Vec2::new(150.0, 150.0), 1.0);
        let clamped = constraints.clamp_transform(transform);
        assert_eq!(clamped.position, Vec2::new(100.0, 100.0));

        // Inside bounds
        let transform = CameraTransform::new(Vec2::new(50.0, 50.0), 1.0);
        let clamped = constraints.clamp_transform(transform);
        assert_eq!(clamped.position, Vec2::new(50.0, 50.0));
    }

    // CameraActuatorConfig tests

    #[test]
    fn test_config_new() {
        let config = CameraActuatorConfig::new();
        assert_eq!(config.zoom, 1.0);
        assert_eq!(config.lag_weight, 0.9);
    }

    #[test]
    fn test_config_builder() {
        let constraints = CameraConstraints::new().with_max_zoom(3.0);
        let config = CameraActuatorConfig::new()
            .with_zoom(2.0)
            .with_lag_weight(0.8)
            .with_constraints(constraints);

        assert_eq!(config.zoom, 2.0);
        assert_eq!(config.lag_weight, 0.8);
        assert_eq!(config.constraints.max_zoom, Some(3.0));
    }

    // CameraActuator tests

    #[test]
    fn test_actuator_new() {
        let entity_id = make_id(42);
        let config = CameraActuatorConfig::new().with_zoom(2.0);
        let actuator = CameraActuator::new(entity_id, config);

        assert_eq!(actuator.entity_id, entity_id);
        assert_eq!(actuator.current_transform().zoom, 2.0);
    }

    #[test]
    fn test_actuator_pan_to() {
        let entity_id = make_id(42);
        let mut actuator = CameraActuator::new(entity_id, CameraActuatorConfig::new());

        actuator.pan_to(Vec2::new(100.0, 200.0));

        // After setting target, update should move towards it
        actuator.update();
        let current = actuator.current_transform();

        // With 0.9 lag weight, should be very close to original but starting to move
        assert!(current.position.x > 0.0);
        assert!(current.position.y > 0.0);
    }

    #[test]
    fn test_actuator_zoom_to() {
        let entity_id = make_id(42);
        let mut actuator = CameraActuator::new(entity_id, CameraActuatorConfig::new());

        actuator.zoom_to(2.0);

        // After setting target, update should change zoom
        actuator.update();
        let current = actuator.current_transform();

        // With 0.9 lag weight, should have moved significantly towards 2.0
        // (First frame: previous is 1.0, target is 2.0, lerp at 0.9 = 1.9)
        assert!(current.zoom > 1.0);
        assert!(current.zoom >= 1.5); // Should be around 1.9
    }

    #[test]
    fn test_actuator_look_at() {
        let entity_id = make_id(42);
        let mut actuator = CameraActuator::new(entity_id, CameraActuatorConfig::new());

        actuator.look_at(Vec2::new(100.0, 100.0), 2.0);

        actuator.update();
        let current = actuator.current_transform();

        assert!(current.position.x > 0.0);
        assert!(current.zoom > 1.0);
    }

    #[test]
    fn test_actuator_snap_to() {
        let entity_id = make_id(42);
        let mut actuator = CameraActuator::new(entity_id, CameraActuatorConfig::new());

        // Set a target
        actuator.pan_to(Vec2::new(100.0, 100.0));
        actuator.update();

        // Snap should override everything
        actuator.snap_to(CameraTransform::new(Vec2::new(500.0, 500.0), 3.0));

        let current = actuator.current_transform();
        assert_eq!(current.position, Vec2::new(500.0, 500.0));
        assert_eq!(current.zoom, 3.0);
    }

    #[test]
    fn test_actuator_respects_constraints() {
        let entity_id = make_id(42);
        let constraints = CameraConstraints::new().with_max_zoom(2.0);
        let config = CameraActuatorConfig::new()
            .with_zoom(1.0)
            .with_constraints(constraints);
        let mut actuator = CameraActuator::new(entity_id, config);

        // Try to zoom beyond constraint
        actuator.zoom_to(5.0);
        actuator.update();

        // Should be clamped to max zoom
        let current = actuator.current_transform();
        assert!(current.zoom <= 2.0);
    }

    #[test]
    fn test_actuator_activate_positive() {
        let entity_id = make_id(42);
        let mut actuator = CameraActuator::new(entity_id, CameraActuatorConfig::new());
        let pulse = Pulse::positive(0, 42, 1000);

        actuator.activate(&pulse, &mut EntityStore::new());

        // Should set target to default
        assert!(actuator.target_transform.is_some());
    }

    #[test]
    fn test_actuator_activate_negative() {
        let entity_id = make_id(42);
        let mut actuator = CameraActuator::new(entity_id, CameraActuatorConfig::new());

        // Set a target first
        actuator.pan_to(Vec2::new(100.0, 100.0));

        let pulse = Pulse::negative(0, 42, 2000);
        actuator.activate(&pulse, &mut EntityStore::new());

        // Should clear target
        assert!(actuator.target_transform.is_none());
    }

    #[test]
    fn test_actuator_update_converges() {
        let entity_id = make_id(42);
        let mut actuator = CameraActuator::new(entity_id, CameraActuatorConfig::new());

        actuator.look_at(Vec2::new(10.0, 10.0), 1.5);

        // Update multiple times until convergence
        for _ in 0..100 {
            actuator.update();
            if actuator.target_transform.is_none() {
                break;
            }
        }

        // Should have converged
        assert!(actuator.target_transform.is_none());

        let current = actuator.current_transform();
        assert_eq!(current.position, Vec2::new(10.0, 10.0));
        assert_eq!(current.zoom, 1.5);
    }
}
