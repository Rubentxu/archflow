//! RotationOperation - Handle-based rotation mathematics
//!
//! This module provides complete rotation operation support with:
//! - Angle calculation from center to mouse position
//! - Snap to increments (15° default)
//! - Visual guide rendering data
//! - Multi-entity rotation around unified center

use archflow_core::Vec2;
use serde::{Deserialize, Serialize};

/// Default rotation snap increment in degrees
pub const DEFAULT_SNAP_INCREMENT: f32 = 15.0;

/// Angle in degrees (0-360)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RotationAngle(pub f32);

impl RotationAngle {
    /// Create a new rotation angle
    #[inline]
    pub fn new(degrees: f32) -> Self {
        Self(degrees.rem_euclid(360.0))
    }

    /// Get the angle in radians
    #[inline]
    pub fn to_radians(&self) -> f32 {
        self.0.to_radians()
    }

    /// Get the raw degrees value
    #[inline]
    pub fn to_degrees(&self) -> f32 {
        self.0
    }

    /// Snap to nearest increment
    #[inline]
    pub fn snap_to(self, increment: f32) -> Self {
        if increment <= 0.0 {
            return self;
        }
        let snapped = (self.0 / increment).round() * increment;
        Self::new(snapped)
    }

    /// Normalize to 0-360 range
    #[inline]
    pub fn normalized(&self) -> Self {
        Self::new(self.0)
    }
}

impl Default for RotationAngle {
    fn default() -> Self {
        Self(0.0)
    }
}

/// Result of a rotation operation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RotationResult {
    /// New rotation angle in degrees
    pub angle: RotationAngle,
    /// Rotation delta from original
    pub delta: f32,
    /// Whether the rotation was snapped
    pub was_snapped: bool,
    /// Point on the rotation circle (for visual guide)
    pub guide_point: Vec2,
}

/// Configuration for rotation operations
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RotationConfig {
    /// Snap increment in degrees (0.0 = no snap)
    pub snap_increment: f32,
    /// Whether snap is enabled by default
    pub snap_enabled: bool,
    /// Radius of rotation handle from center
    pub handle_radius: f32,
    /// Minimum drag distance to start rotation
    pub min_drag_distance: f32,
}

impl Default for RotationConfig {
    fn default() -> Self {
        Self {
            snap_increment: DEFAULT_SNAP_INCREMENT,
            snap_enabled: true,
            handle_radius: 30.0,
            min_drag_distance: 5.0,
        }
    }
}

/// State marker for idle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdleRotationState;

/// State marker for dragging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraggingRotationState {
    /// Starting mouse position
    pub start_mouse_pos: Vec2,
    /// Current mouse position
    pub current_mouse_pos: Vec2,
    /// Angle at start of drag
    pub start_angle: RotationAngle,
}

/// State marker for completed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedRotationState {
    /// The final rotation result
    pub result: RotationResult,
}

/// Rotation operation with type-state pattern
///
/// The generic parameter S tracks the operation state:
/// - IdleRotationState: Operation created but not started
/// - DraggingRotationState: User is dragging the rotation handle
/// - CompletedRotationState: Drag completed, result available
#[derive(Debug, Clone)]
pub struct RotationOperation<S> {
    entity_id: archflow_core::EntityId,
    center: Vec2,
    original_angle: RotationAngle,
    handle_offset: Vec2,
    config: RotationConfig,
    state: S,
}

impl RotationOperation<IdleRotationState> {
    /// Create a new rotation operation
    #[inline]
    pub fn new(
        entity_id: archflow_core::EntityId,
        center: Vec2,
        original_angle: f32,
        handle_position: Vec2,
    ) -> Self {
        Self::with_config(
            entity_id,
            center,
            original_angle,
            handle_position,
            RotationConfig::default(),
        )
    }

    /// Create with custom configuration
    pub fn with_config(
        entity_id: archflow_core::EntityId,
        center: Vec2,
        original_angle: f32,
        handle_position: Vec2,
        config: RotationConfig,
    ) -> Self {
        let handle_offset = handle_position - center;

        Self {
            entity_id,
            center,
            original_angle: RotationAngle::new(original_angle),
            handle_offset,
            config,
            state: IdleRotationState,
        }
    }

    /// Start rotation drag
    pub fn start_drag(self, mouse_pos: Vec2) -> RotationOperation<DraggingRotationState> {
        let start_angle = self.calculate_angle(mouse_pos);

        RotationOperation {
            entity_id: self.entity_id,
            center: self.center,
            original_angle: self.original_angle,
            handle_offset: self.handle_offset,
            config: self.config,
            state: DraggingRotationState {
                start_mouse_pos: mouse_pos,
                current_mouse_pos: mouse_pos,
                start_angle,
            },
        }
    }
}

impl RotationOperation<DraggingRotationState> {
    /// Get start mouse position
    #[inline]
    pub fn start_mouse_pos(&self) -> Vec2 {
        self.state.start_mouse_pos
    }

    /// Get current mouse position
    #[inline]
    pub fn current_mouse_pos(&self) -> Vec2 {
        self.state.current_mouse_pos
    }

    /// Update mouse position
    pub fn update(mut self, mouse_pos: Vec2) -> Self {
        self.state.current_mouse_pos = mouse_pos;
        self
    }

    /// Get start angle of drag
    #[inline]
    pub fn start_angle(&self) -> RotationAngle {
        self.state.start_angle
    }

    /// Calculate current rotation result
    pub fn current_result(&self) -> RotationResult {
        self.calculate_rotation(self.state.current_mouse_pos)
    }

    /// Complete the rotation
    pub fn complete(self) -> RotationOperation<CompletedRotationState> {
        let result = self.current_result();

        RotationOperation {
            entity_id: self.entity_id,
            center: self.center,
            original_angle: self.original_angle,
            handle_offset: self.handle_offset,
            config: self.config,
            state: CompletedRotationState { result },
        }
    }

    /// Calculate rotation from drag
    fn calculate_rotation(&self, mouse_pos: Vec2) -> RotationResult {
        let current_angle = self.calculate_angle(mouse_pos);

        let delta = current_angle.to_degrees() - self.state.start_angle.to_degrees();

        let mut new_angle = RotationAngle::new(self.original_angle.to_degrees() + delta);

        let mut was_snapped = false;
        if self.config.snap_enabled && self.config.snap_increment > 0.0 {
            let snapped_angle = new_angle.snap_to(self.config.snap_increment);
            if snapped_angle != new_angle {
                new_angle = snapped_angle;
                was_snapped = true;
            }
        }

        let guide_point = self.center
            + Vec2::new(
                self.config.handle_radius * (-new_angle.to_radians()).cos(),
                self.config.handle_radius * (-new_angle.to_radians()).sin(),
            );

        RotationResult {
            angle: new_angle,
            delta: (new_angle.to_degrees() - self.original_angle.to_degrees()),
            was_snapped,
            guide_point,
        }
    }
}

impl RotationOperation<CompletedRotationState> {
    /// Get the result
    #[inline]
    pub fn result(&self) -> RotationResult {
        self.state.result
    }
}

impl<S> RotationOperation<S> {
    /// Get entity ID
    #[inline]
    pub fn entity_id(&self) -> archflow_core::EntityId {
        self.entity_id
    }

    /// Get rotation center
    #[inline]
    pub fn center(&self) -> Vec2 {
        self.center
    }

    /// Get original angle
    #[inline]
    pub fn original_angle(&self) -> RotationAngle {
        self.original_angle
    }

    /// Get configuration
    #[inline]
    pub fn config(&self) -> RotationConfig {
        self.config
    }

    /// Calculate angle from center to a point
    fn calculate_angle(&self, point: Vec2) -> RotationAngle {
        let dx = point.x - self.center.x;
        let dy = point.y - self.center.y;
        let angle_rad = dx.atan2(dy);
        let angle_deg = angle_rad.to_degrees();

        RotationAngle::new(angle_deg)
    }
}

/// Calculate the position of a point after rotation around a center
#[inline]
pub fn rotate_point_around_center(point: Vec2, center: Vec2, angle: RotationAngle) -> Vec2 {
    let dx = point.x - center.x;
    let dy = point.y - center.y;

    let cos = (-angle.to_radians()).cos();
    let sin = (-angle.to_radians()).sin();

    Vec2::new(
        center.x + dx * cos - dy * sin,
        center.y + dx * sin + dy * cos,
    )
}

/// Rotate a bounding box around its center
pub fn rotate_bounds(bounds: (Vec2, Vec2), center: Vec2, angle: RotationAngle) -> (Vec2, Vec2) {
    let corners = [
        Vec2::new(bounds.0.x, bounds.0.y),
        Vec2::new(bounds.0.x, bounds.1.y),
        Vec2::new(bounds.1.x, bounds.0.y),
        Vec2::new(bounds.1.x, bounds.1.y),
    ];

    let rotated: Vec<Vec2> = corners
        .into_iter()
        .map(|p| rotate_point_around_center(p, center, angle))
        .collect();

    let min_x = rotated.iter().map(|p| p.x).fold(f32::MAX, f32::min);
    let min_y = rotated.iter().map(|p| p.y).fold(f32::MAX, f32::min);
    let max_x = rotated.iter().map(|p| p.x).fold(f32::MIN, f32::max);
    let max_y = rotated.iter().map(|p| p.y).fold(f32::MIN, f32::max);

    (Vec2::new(min_x, min_y), Vec2::new(max_x, max_y))
}

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_core::EntityId;

    fn create_test_operation() -> RotationOperation<IdleRotationState> {
        let entity_id = EntityId::new();
        RotationOperation::new(
            entity_id,
            Vec2::new(150.0, 150.0), // center
            0.0,                     // original angle
            Vec2::new(150.0, 120.0), // handle position (above center)
        )
    }

    #[test]
    fn test_new_rotation_operation() {
        let op = create_test_operation();

        assert_eq!(op.center(), Vec2::new(150.0, 150.0));
        assert_eq!(op.original_angle().to_degrees(), 0.0);
    }

    #[test]
    fn test_calculate_angle_right() {
        let op = create_test_operation();

        // Point at 0° (right of center)
        let angle = op.calculate_angle(Vec2::new(200.0, 150.0));
        assert!((angle.to_degrees() - 0.0).abs() < 0.1);
    }

    #[test]
    fn test_calculate_angle_up() {
        let op = create_test_operation();

        // Point at 90° (above center)
        let angle = op.calculate_angle(Vec2::new(150.0, 100.0));
        assert!((angle.to_degrees() - 90.0).abs() < 0.1);
    }

    #[test]
    fn test_calculate_angle_left() {
        let op = create_test_operation();

        // Point at 180° (left of center)
        let angle = op.calculate_angle(Vec2::new(100.0, 150.0));
        assert!((angle.to_degrees() - 180.0).abs() < 0.1);
    }

    #[test]
    fn test_calculate_angle_down() {
        let op = create_test_operation();

        // Point at 270° (below center)
        let angle = op.calculate_angle(Vec2::new(150.0, 200.0));
        assert!((angle.to_degrees() - 270.0).abs() < 0.1);
    }

    #[test]
    fn test_rotation_drag_90_degrees() {
        let op = create_test_operation();
        let dragging = op.start_drag(Vec2::new(150.0, 120.0));

        // Drag to right position (0°) -> should be 90° rotation
        let result = dragging.update(Vec2::new(180.0, 150.0)).current_result();

        assert!((result.angle.to_degrees() - 90.0).abs() < 1.0);
    }

    #[test]
    fn test_rotation_snap_to_45() {
        let op = create_test_operation();
        let dragging = op.start_drag(Vec2::new(150.0, 120.0));

        // Drag to ~45° position
        let result = dragging.update(Vec2::new(171.0, 129.0)).current_result();

        // Should snap to 45°
        assert!((result.angle.to_degrees() - 45.0).abs() < 1.0);
        assert!(result.was_snapped);
    }

    #[test]
    fn test_rotation_no_snap() {
        let config = RotationConfig {
            snap_increment: DEFAULT_SNAP_INCREMENT,
            snap_enabled: false,
            handle_radius: 30.0,
            min_drag_distance: 5.0,
        };

        let op = RotationOperation::with_config(
            EntityId::new(),
            Vec2::new(150.0, 150.0),
            0.0,
            Vec2::new(150.0, 120.0),
            config,
        );

        let dragging = op.start_drag(Vec2::new(150.0, 120.0));
        let result = dragging.update(Vec2::new(171.0, 129.0)).current_result();

        assert!(!result.was_snapped);
    }

    #[test]
    fn test_guide_point_calculation() {
        let op = create_test_operation();
        let dragging = op.start_drag(Vec2::new(150.0, 120.0));

        // At 90°, guide point should be directly above center at handle_radius
        let result = dragging.update(Vec2::new(150.0, 120.0)).current_result();

        let expected_guide = Vec2::new(150.0, 120.0); // Same as handle position
        assert!((result.guide_point.x - expected_guide.x).abs() < 0.1);
        assert!((result.guide_point.y - expected_guide.y).abs() < 0.1);
    }

    #[test]
    fn test_rotation_result_delta() {
        let op = create_test_operation();
        let dragging = op.start_drag(Vec2::new(150.0, 120.0));

        // Rotate 45°
        let result = dragging.update(Vec2::new(171.0, 129.0)).current_result();

        assert!((result.delta - 45.0).abs() < 1.0);
    }

    #[test]
    fn test_complete_rotation() {
        let op = create_test_operation();
        let dragging = op.start_drag(Vec2::new(150.0, 120.0));
        let completed = dragging.update(Vec2::new(171.0, 129.0)).complete();

        let result = completed.result();
        assert!((result.angle.to_degrees() - 45.0).abs() < 1.0);
    }

    #[test]
    fn test_rotation_angle_normalization() {
        let angle = RotationAngle::new(450.0);
        assert!((angle.to_degrees() - 90.0).abs() < 0.001);

        let angle2 = RotationAngle::new(-90.0);
        assert!((angle2.to_degrees() - 270.0).abs() < 0.001);
    }

    #[test]
    fn test_rotate_point_around_center() {
        let point = Vec2::new(200.0, 150.0); // 50px right of center
        let center = Vec2::new(150.0, 150.0);
        let angle = RotationAngle::new(90.0);

        let rotated = rotate_point_around_center(point, center, angle);

        assert!((rotated.x - 150.0).abs() < 0.1);
        assert!((rotated.y - 100.0).abs() < 0.1);
    }

    #[test]
    fn test_rotate_bounds() {
        let bounds = (Vec2::new(100.0, 100.0), Vec2::new(200.0, 200.0));
        let center = Vec2::new(150.0, 150.0);
        let angle = RotationAngle::new(90.0);

        let rotated = rotate_bounds(bounds, center, angle);

        assert!((rotated.0.x - 100.0).abs() < 1.0);
        assert!((rotated.1.x - 200.0).abs() < 1.0);
    }
}
