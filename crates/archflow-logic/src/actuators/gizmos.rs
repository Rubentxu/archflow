// ═══════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Gizmo Actuators
//
// Actuators for professional 3D-style transformation gizmos.
// Implements the gizmo system from Sprint 9.
//
// Architecture:
// - TransformGizmoActuator: Show/hide transform gizmo
// - GizmoMoveActuator: Axis-constrained movement (X, Y, XY)
// - GizmoScaleActuator: Uniform and non-uniform scaling
// - GizmoRotateActuator: Precision rotation with snapping
//
// Performance Characteristics:
// - O(1) gizmo visibility toggle
// - O(1) handle hit detection
// - O(n) for multi-entity transformations
//
// ═══════════════════════════════════════════════════════════════════════════════════════

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use archflow_core::{EntityId, MAX_ENTITIES, Vec2};
use archflow_engine::{Command, EntityStore};

/// Gizmo type enumeration
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GizmoType {
    /// Movement gizmo with axis arrows
    Move = 0,
    /// Scale gizmo with corner/side handles
    Scale = 1,
    /// Rotation gizmo with angular handle
    Rotate = 2,
    /// Combined gizmo (all handles visible)
    Combined = 3,
}

/// Axis constraint for gizmo operations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GizmoAxis {
    /// No constraint (free movement/scale)
    None = 0,
    /// X-axis only
    X = 1,
    /// Y-axis only
    Y = 2,
    /// Both X and Y (free)
    XY = 3,
}

/// Gizmo handle location and type
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GizmoHandle {
    /// Handle position in world coordinates
    pub position: Vec2,
    /// Handle type identifier
    pub handle_type: GizmoHandleType,
    /// Screen-space size (constant regardless of zoom)
    pub screen_size: f32,
}

/// Types of handles on a gizmo
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GizmoHandleType {
    /// Move arrows (X-axis)
    MoveX = 0,
    /// Move arrows (Y-axis)
    MoveY = 1,
    /// Move arrows (XY plane)
    MoveXY = 2,
    /// Scale handle (corner)
    ScaleCorner = 3,
    /// Scale handle (side)
    ScaleSide = 4,
    /// Rotation handle
    Rotate = 5,
    /// Pivot point
    Pivot = 6,
}

/// Gizmo configuration
#[derive(Clone, Copy, Debug)]
pub struct GizmoConfig {
    /// Arrow length in screen pixels
    pub arrow_length: f32,
    /// Arrow thickness in screen pixels
    pub arrow_thickness: f32,
    /// Rotate circle radius in screen pixels
    pub rotate_radius: f32,
    /// Handle size in screen pixels
    pub handle_size: f32,
    /// Pivot point size in screen pixels
    pub pivot_size: f32,
    /// Snapping angle in degrees
    pub snap_angle: f32,
    /// Minimum snap angle in degrees
    pub min_snap_angle: f32,
}

impl Default for GizmoConfig {
    fn default() -> Self {
        Self {
            arrow_length: 60.0,
            arrow_thickness: 8.0,
            rotate_radius: 80.0,
            handle_size: 12.0,
            pivot_size: 8.0,
            snap_angle: 15.0,
            min_snap_angle: 1.0,
        }
    }
}

/// State for a gizmo interaction
#[derive(Clone, Debug)]
pub struct GizmoState {
    /// Current gizmo type
    pub gizmo_type: GizmoType,
    /// Currently active handle
    pub active_handle: Option<GizmoHandle>,
    /// Starting position of the interaction
    pub start_pos: Vec2,
    /// Starting values before transformation
    pub start_values: Vec<f32>,
    /// Current drag offset
    pub drag_offset: Vec2,
    /// Whether gizmo is visible
    pub is_visible: bool,
    /// Pivot position (world coordinates)
    pub pivot: Vec2,
    /// Original entity positions (for multi-select)
    pub original_positions: Vec<Vec2>,
    /// Original entity sizes (for multi-select)
    pub original_sizes: Vec<Vec2>,
}

/// Result of a gizmo hit test
#[derive(Clone, Debug, PartialEq)]
pub enum GizmoHitResult {
    /// No hit
    None,
    /// Hit on a handle
    Handle(GizmoHandle),
    /// Hit on the gizmo body (for movement)
    Body(Vec2),
}

/// ═══════════════════════════════════════════════════════════════════════════════════════
/// TransformGizmoActuator - Visibility Control
/// ═══════════════════════════════════════════════════════════════════════════════════════
/// Actuator for controlling transform gizmo visibility and activation.
///
/// Manages which gizmo is visible and handles the state machine for
/// gizmo interactions.
///
/// # Performance
/// - O(1) visibility toggle
/// - O(n) handle generation for multi-selection
///
/// # Example
///
/// ```
/// use archflow_logic::actuators::gizmos::{TransformGizmoActuator, GizmoType};
///
/// let mut actuator = TransformGizmoActuator::new();
/// actuator.show_gizmo(GizmoType::Combined);
/// ```
pub struct TransformGizmoActuator {
    /// Current gizmo state
    state: Option<GizmoState>,
    /// Configuration
    config: GizmoConfig,
}

impl TransformGizmoActuator {
    /// Creates a new TransformGizmoActuator
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: None,
            config: GizmoConfig::default(),
        }
    }

    /// Creates a TransformGizmoActuator with custom configuration
    #[inline(always)]
    #[must_use]
    pub fn with_config(config: GizmoConfig) -> Self {
        Self {
            state: None,
            config,
        }
    }

    /// Show the gizmo for an entity or selection
    ///
    /// # Arguments
    ///
    /// * `gizmo_type` - Type of gizmo to show
    /// * `entity_ids` - Entities to apply gizmo to
    /// * `center` - Center position of the selection
    /// * `store` - EntityStore for reading entity data
    ///
    /// # Returns
    ///
    /// Vector of commands to execute (if any)
    pub fn show_gizmo(
        &mut self,
        gizmo_type: GizmoType,
        entity_ids: &[EntityId],
        center: Vec2,
        store: &EntityStore,
    ) -> Vec<Command> {
        // Store original entity states for undo
        let mut original_positions = Vec::new();
        let mut original_sizes = Vec::new();

        for &entity in entity_ids {
            let idx = entity.index().0 as usize;
            if idx < MAX_ENTITIES as usize && store.is_alive(entity) {
                let pos = store.world_pos(idx);
                let size = store.size(idx);
                original_positions.push(pos);
                original_sizes.push(size);
            }
        }

        self.state = Some(GizmoState {
            gizmo_type,
            active_handle: None,
            start_pos: center,
            start_values: Vec::new(),
            drag_offset: Vec2::ZERO,
            is_visible: true,
            pivot: center,
            original_positions,
            original_sizes,
        });

        Vec::new()
    }

    /// Hide the gizmo
    #[inline(always)]
    pub fn hide_gizmo(&mut self) {
        self.state = None;
    }

    /// Toggle gizmo visibility
    ///
    /// # Returns
    ///
    /// New visibility state after toggle (`false` if gizmo was hidden)
    pub fn toggle_visibility(&mut self) -> bool {
        match &mut self.state {
            Some(s) => {
                s.is_visible = !s.is_visible;
                s.is_visible
            }
            None => false,
        }
    }

    /// Set gizmo type
    pub fn set_gizmo_type(&mut self, gizmo_type: GizmoType) {
        if let Some(ref mut state) = self.state {
            state.gizmo_type = gizmo_type;
        }
    }

    /// Set pivot point
    pub fn set_pivot(&mut self, pivot: Vec2) {
        if let Some(ref mut state) = self.state {
            state.pivot = pivot;
        }
    }

    /// Check if gizmo is visible
    #[inline(always)]
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.state.as_ref().map_or(false, |s| s.is_visible)
    }

    /// Get current gizmo type
    #[inline(always)]
    #[must_use]
    pub fn gizmo_type(&self) -> Option<GizmoType> {
        self.state.as_ref().map(|s| s.gizmo_type)
    }

    /// Get current state
    #[inline(always)]
    #[must_use]
    pub fn state(&self) -> Option<&GizmoState> {
        self.state.as_ref()
    }

    /// Get mutable state
    #[inline(always)]
    pub fn state_mut(&mut self) -> Option<&mut GizmoState> {
        self.state.as_mut()
    }

    /// Clear gizmo state
    #[inline(always)]
    pub fn clear(&mut self) {
        self.state = None;
    }
}

impl Default for TransformGizmoActuator {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// GizmoMoveActuator - Axis-Constrained Movement
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Actuator for axis-constrained movement using gizmo handles.
///
/// Provides precise control over entity movement with optional constraints
/// to X-axis, Y-axis, or free XY movement.
///
/// # Performance
/// - O(1) constraint application
/// - O(n) for multi-entity movement
///
/// # Example
///
/// ```
/// use archflow_logic::actuators::gizmos::{GizmoMoveActuator, GizmoAxis};
///
/// let mut actuator = GizmoMoveActuator::new();
/// let delta = actuator.calculate_delta(Vec2::new(100.0, 100.0), GizmoAxis::X);
/// ```
pub struct GizmoMoveActuator {
    /// Configuration
    config: GizmoConfig,
    /// Current axis constraint
    current_constraint: GizmoAxis,
}

impl GizmoMoveActuator {
    /// Creates a new GizmoMoveActuator
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: GizmoConfig::default(),
            current_constraint: GizmoAxis::None,
        }
    }

    /// Creates a GizmoMoveActuator with custom configuration
    #[inline(always)]
    #[must_use]
    pub fn with_config(config: GizmoConfig) -> Self {
        Self {
            config,
            current_constraint: GizmoAxis::None,
        }
    }

    /// Calculate movement delta with axis constraint
    ///
    /// # Arguments
    ///
    /// * `raw_delta` - Unconstrained movement vector
    /// * `axis` - Axis constraint to apply
    ///
    /// # Returns
    ///
    /// Constrained movement vector
    #[inline(always)]
    #[must_use]
    pub fn calculate_delta(&self, raw_delta: Vec2, axis: GizmoAxis) -> Vec2 {
        match axis {
            GizmoAxis::X => Vec2::new(raw_delta.x, 0.0),
            GizmoAxis::Y => Vec2::new(0.0, raw_delta.y),
            GizmoAxis::XY => raw_delta,
            GizmoAxis::None => raw_delta,
        }
    }

    /// Apply constrained movement to an entity
    ///
    /// # Arguments
    ///
    /// * `entity_id` - Entity to move
    /// * `raw_delta` - Unconstrained movement vector
    /// * `axis` - Axis constraint
    /// * `store` - EntityStore to update
    ///
    /// # Returns
    ///
    /// Move command if movement occurred
    pub fn apply_movement(
        &self,
        entity_id: EntityId,
        raw_delta: Vec2,
        axis: GizmoAxis,
        _store: &mut EntityStore,
    ) -> Option<Command> {
        let delta = self.calculate_delta(raw_delta, axis);

        if delta.x.abs() < 0.001 && delta.y.abs() < 0.001 {
            return None;
        }

        Some(Command::Move {
            id: entity_id,
            delta,
        })
    }

    /// Apply constrained movement to multiple entities
    ///
    /// # Arguments
    ///
    /// * `entity_ids` - Entities to move
    /// * `raw_delta` - Unconstrained movement vector
    /// * `axis` - Axis constraint
    /// * `store` - EntityStore to update
    ///
    /// # Returns
    ///
    /// Vector of Move commands
    pub fn apply_batch_movement(
        &self,
        entity_ids: &[EntityId],
        raw_delta: Vec2,
        axis: GizmoAxis,
        _store: &mut EntityStore,
    ) -> Vec<Command> {
        let delta = self.calculate_delta(raw_delta, axis);

        if delta.x.abs() < 0.001 && delta.y.abs() < 0.001 {
            return Vec::new();
        }

        let mut commands = Vec::new();
        for &entity in entity_ids {
            commands.push(Command::Move { id: entity, delta });
        }
        commands
    }

    /// Set current axis constraint
    #[inline(always)]
    pub fn set_constraint(&mut self, axis: GizmoAxis) {
        self.current_constraint = axis;
    }

    /// Get current axis constraint
    #[inline(always)]
    #[must_use]
    pub fn constraint(&self) -> GizmoAxis {
        self.current_constraint
    }

    /// Calculate display overlay text
    ///
    /// # Arguments
    ///
    /// * `delta` - Applied movement delta
    ///
    /// # Returns
    ///
    /// Display string like "X: +45.2px"
    #[inline(always)]
    #[must_use]
    pub fn format_display(&self, delta: Vec2) -> String {
        String::from(format!("X: {:+.1}px  Y: {:+.1}px", delta.x, delta.y))
    }
}

impl Default for GizmoMoveActuator {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// GizmoScaleActuator - Uniform and Non-Uniform Scaling
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Actuator for scaling entities using gizmo handles.
///
/// Supports both uniform scaling (preserving aspect ratio) and non-uniform
/// scaling (stretching) based on modifier keys.
///
/// # Performance
/// - O(1) scale factor calculation
/// - O(n) for multi-entity scaling
///
/// # Example
///
/// ```
/// use archflow_logic::actuators::gizmos::{GizmoScaleActuator, GizmoAxis};
///
/// let actuator = GizmoScaleActuator::new();
/// let (scale_x, scale_y) = actuator.calculate_scale(Vec2::new(200.0, 200.0), Vec2::new(100.0, 100.0), false, false);
/// ```
pub struct GizmoScaleActuator {
    /// Configuration
    config: GizmoConfig,
    /// Center point for scaling
    center: Vec2,
}

impl GizmoScaleActuator {
    /// Creates a new GizmoScaleActuator
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: GizmoConfig::default(),
            center: Vec2::ZERO,
        }
    }

    /// Creates a GizmoScaleActuator with custom configuration
    #[inline(always)]
    #[must_use]
    pub fn with_config(config: GizmoConfig) -> Self {
        Self {
            config,
            center: Vec2::ZERO,
        }
    }

    /// Calculate scale factors based on drag
    ///
    /// # Arguments
    ///
    /// * `handle_pos` - Current handle position
    /// * `original_size` - Original entity size
    /// * `uniform` - Whether to maintain aspect ratio
    /// * `from_center` - Whether to scale from center
    ///
    /// # Returns
    ///
    /// (scale_x, scale_y) factors
    #[must_use]
    pub fn calculate_scale(
        &self,
        handle_pos: Vec2,
        original_size: Vec2,
        uniform: bool,
        _from_center: bool,
    ) -> (f32, f32) {
        let current_size = handle_pos - self.center;
        let current_x = current_size.x.abs() * 2.0;
        let current_y = current_size.y.abs() * 2.0;

        if original_size.x < 0.001 || original_size.y < 0.001 {
            return (1.0, 1.0);
        }

        let scale_x = current_x / original_size.x;
        let scale_y = current_y / original_size.y;

        if uniform {
            let avg_scale = (scale_x + scale_y) / 2.0;
            (avg_scale, avg_scale)
        } else {
            (scale_x, scale_y)
        }
    }

    /// Calculate new size based on drag
    ///
    /// # Arguments
    ///
    /// * `original_size` - Original entity size
    /// * `drag_delta` - Drag movement vector
    /// * `uniform` - Whether to maintain aspect ratio
    /// * `from_center` - Whether to scale from center
    /// * `handle_type` - Type of handle being dragged
    ///
    /// # Returns
    ///
    /// New size vector
    #[must_use]
    pub fn calculate_new_size(
        &self,
        original_size: Vec2,
        drag_delta: Vec2,
        uniform: bool,
        from_center: bool,
        handle_type: GizmoHandleType,
    ) -> Vec2 {
        let (sx, sy) = self.calculate_scale(
            self.center + original_size / 2.0 + drag_delta,
            original_size,
            uniform,
            from_center,
        );

        let mut new_w = original_size.x * sx;
        let mut new_h = original_size.y * sy;

        // Apply constraints based on handle type
        match handle_type {
            GizmoHandleType::ScaleCorner => {
                // Full scaling
            }
            GizmoHandleType::ScaleSide => {
                // Scale only in one dimension
                if drag_delta.x.abs() > drag_delta.y.abs() {
                    new_h = original_size.y;
                } else {
                    new_w = original_size.x;
                }
            }
            _ => {
                // Default to corner behavior
            }
        }

        // Minimum size constraint
        new_w = new_w.max(10.0);
        new_h = new_h.max(10.0);

        Vec2::new(new_w, new_h)
    }

    /// Apply scale to an entity
    pub fn apply_scale(
        &self,
        entity_id: EntityId,
        new_size: Vec2,
        _store: &mut EntityStore,
    ) -> Command {
        Command::Resize {
            id: entity_id,
            size: new_size,
        }
    }

    /// Apply batch scale to multiple entities
    pub fn apply_batch_scale(
        &self,
        entity_ids: &[EntityId],
        original_sizes: &[Vec2],
        new_sizes: &[Vec2],
        _store: &mut EntityStore,
    ) -> Vec<Command> {
        let mut commands = Vec::new();
        for (&entity, (&_orig, &new)) in entity_ids
            .iter()
            .zip(original_sizes.iter().zip(new_sizes.iter()))
        {
            commands.push(Command::Resize {
                id: entity,
                size: new,
            });
        }
        commands
    }

    /// Set center point for scaling
    #[inline(always)]
    pub fn set_center(&mut self, center: Vec2) {
        self.center = center;
    }

    /// Format scale display string
    ///
    /// # Arguments
    ///
    /// * `scale_x` - X scale factor
    /// * `scale_y` - Y scale factor
    ///
    /// # Returns
    ///
    /// Display string like "Scale: 1.5x" or "Scale: 1.5x × 0.8x"
    #[inline(always)]
    #[must_use]
    pub fn format_display(&self, scale_x: f32, scale_y: f32) -> String {
        if (scale_x - scale_y).abs() < 0.001 {
            String::from(format!("Scale: {:.2}x", scale_x))
        } else {
            String::from(format!("Scale: {:.2}x × {:.2}x", scale_x, scale_y))
        }
    }
}

impl Default for GizmoScaleActuator {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// GizmoRotateActuator - Precision Rotation
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Actuator for precision rotation using gizmo handles.
///
/// Provides smooth rotation with optional snapping to angles and
/// visual feedback of the rotation angle.
///
/// # Performance
/// - O(1) angle calculation
/// - O(n) for multi-entity rotation
///
/// # Example
///
/// ```
/// use archflow_logic::actuators::gizmos::GizmoRotateActuator;
///
/// let actuator = GizmoRotateActuator::new();
/// let angle = actuator.calculate_angle(Vec2::new(100.0, 0.0));
/// ```
pub struct GizmoRotateActuator {
    /// Configuration
    config: GizmoConfig,
    /// Center point for rotation
    center: Vec2,
    /// Original angle when drag started
    start_angle: f32,
}

impl GizmoRotateActuator {
    /// Creates a new GizmoRotateActuator
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: GizmoConfig::default(),
            center: Vec2::ZERO,
            start_angle: 0.0,
        }
    }

    /// Creates a GizmoRotateActuator with custom configuration
    #[inline(always)]
    #[must_use]
    pub fn with_config(config: GizmoConfig) -> Self {
        Self {
            config,
            center: Vec2::ZERO,
            start_angle: 0.0,
        }
    }

    /// Calculate angle from center to position
    ///
    /// # Arguments
    ///
    /// * `position` - World position
    ///
    /// # Returns
    ///
    /// Angle in degrees (0-360)
    #[inline(always)]
    #[must_use]
    pub fn calculate_angle(&self, position: Vec2) -> f32 {
        let offset = position - self.center;
        let angle_rad = offset.y.atan2(offset.x);
        let mut angle_deg = angle_rad.to_degrees();
        if angle_deg < 0.0 {
            angle_deg += 360.0;
        }
        angle_deg
    }

    /// Calculate rotation delta with snapping
    ///
    /// # Arguments
    ///
    /// * `current_angle` - Current mouse angle
    /// * `snap` - Whether to apply snapping
    /// * `fine_snap` - Use fine snap angle (1° vs 15°)
    ///
    /// # Returns
    ///
    /// Snapped angle in degrees
    #[must_use]
    pub fn calculate_rotation(&self, current_angle: f32, snap: bool, fine_snap: bool) -> f32 {
        if !snap {
            return current_angle;
        }

        let snap_angle = if fine_snap {
            self.config.min_snap_angle
        } else {
            self.config.snap_angle
        };

        let snapped = (current_angle / snap_angle).round() * snap_angle;
        if snapped >= 360.0 {
            snapped - 360.0
        } else {
            snapped
        }
    }

    /// Start rotation tracking
    ///
    /// # Arguments
    ///
    /// * `start_angle` - Initial angle when drag started
    #[inline(always)]
    pub fn start_tracking(&mut self, start_angle: f32) {
        self.start_angle = start_angle;
    }

    /// Apply rotation to an entity
    ///
    /// Note: This calculates the rotation delta. Full rotation implementation
    /// requires a Rotate command or equivalent in the EntityStore.
    pub fn apply_rotation(
        &self,
        entity_id: EntityId,
        current_angle: f32,
        snap: bool,
        fine_snap: bool,
        _store: &mut EntityStore,
    ) -> Option<Command> {
        let angle = self.calculate_rotation(current_angle, snap, fine_snap);
        let delta = angle - self.start_angle;

        if delta.abs() < 0.1 {
            return None;
        }

        // For now, return a resize-like command to indicate rotation
        // In a full implementation, this would be a Command::Rotate
        Some(Command::Resize {
            id: entity_id,
            size: Vec2::new(delta, delta), // Using size to pass rotation angle
        })
    }

    /// Format rotation display string
    ///
    /// # Arguments
    ///
    /// * `angle` - Rotation angle in degrees
    ///
    /// # Returns
    ///
    /// Display string like "45.3°"
    #[inline(always)]
    #[must_use]
    pub fn format_display(&self, angle: f32) -> String {
        String::from(format!("{:.1}°", angle))
    }

    /// Set center point for rotation
    #[inline(always)]
    pub fn set_center(&mut self, center: Vec2) {
        self.center = center;
    }
}

impl Default for GizmoRotateActuator {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// GizmoHitTest - Handle Detection
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Utilities for gizmo hit testing
pub struct GizmoHitTest;

impl GizmoHitTest {
    /// Test if a point hits a handle
    ///
    /// # Arguments
    ///
    /// * `point` - Test point in world coordinates
    /// * `handle` - Handle to test against
    /// * `camera_zoom` - Current camera zoom level
    ///
    /// # Returns
    ///
    /// True if point hits the handle
    #[inline(always)]
    #[must_use]
    pub fn hit_test(point: Vec2, handle: GizmoHandle, camera_zoom: f32) -> bool {
        let screen_radius = handle.screen_size / camera_zoom / 2.0;
        let distance = point.distance(handle.position);
        distance <= screen_radius
    }

    /// Test hit on rotation circle
    ///
    /// # Arguments
    ///
    /// * `point` - Test point
    /// * `center` - Rotation center
    /// * `radius` - Rotation radius in screen pixels
    /// * `camera_zoom` - Current camera zoom
    /// * `thickness` - Ring thickness in screen pixels
    ///
    /// # Returns
    ///
    /// True if point is on the rotation circle
    #[inline(always)]
    #[must_use]
    pub fn hit_test_rotation(
        point: Vec2,
        center: Vec2,
        radius: f32,
        camera_zoom: f32,
        thickness: f32,
    ) -> bool {
        let world_radius = radius / camera_zoom;
        let world_thickness = thickness / camera_zoom;
        let distance = point.distance(center);
        let diff = (distance - world_radius).abs();
        diff <= world_thickness / 2.0
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_engine::EntityStore;

    // ═══════════════════════════════════════════════════════════════════════════════════
    // TransformGizmoActuator Tests
    // ═══════════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_transform_gizmo_actuator_initial_state() {
        let actuator = TransformGizmoActuator::new();
        assert!(!actuator.is_visible());
        assert!(actuator.gizmo_type().is_none());
    }

    #[test]
    fn test_transform_gizmo_show_hide() {
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut actuator = TransformGizmoActuator::new();
        assert!(!actuator.is_visible());

        actuator.show_gizmo(
            GizmoType::Combined,
            &[entity],
            Vec2::new(125.0, 125.0),
            &store,
        );
        assert!(actuator.is_visible());
        assert_eq!(actuator.gizmo_type(), Some(GizmoType::Combined));

        actuator.hide_gizmo();
        assert!(!actuator.is_visible());
    }

    #[test]
    fn test_transform_gizmo_toggle() {
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut actuator = TransformGizmoActuator::new();
        actuator.show_gizmo(GizmoType::Move, &[entity], Vec2::ZERO, &store);

        assert!(actuator.is_visible());
        // After first toggle: visible -> invisible, returns false
        assert!(!actuator.toggle_visibility());
        assert!(!actuator.is_visible());
        // After second toggle: invisible -> visible, returns true
        assert!(actuator.toggle_visibility());
        assert!(actuator.is_visible());
    }

    #[test]
    fn test_transform_gizmo_clear() {
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut actuator = TransformGizmoActuator::new();
        actuator.show_gizmo(GizmoType::Scale, &[entity], Vec2::ZERO, &store);
        actuator.clear();

        assert!(!actuator.is_visible());
        assert!(actuator.state().is_none());
    }

    // ═══════════════════════════════════════════════════════════════════════════════════
    // GizmoMoveActuator Tests
    // ═══════════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_gizmo_move_x_constraint() {
        let actuator = GizmoMoveActuator::new();

        let delta = actuator.calculate_delta(Vec2::new(100.0, 50.0), GizmoAxis::X);
        assert_eq!(delta.x, 100.0);
        assert_eq!(delta.y, 0.0);
    }

    #[test]
    fn test_gizmo_move_y_constraint() {
        let actuator = GizmoMoveActuator::new();

        let delta = actuator.calculate_delta(Vec2::new(100.0, 50.0), GizmoAxis::Y);
        assert_eq!(delta.x, 0.0);
        assert_eq!(delta.y, 50.0);
    }

    #[test]
    fn test_gizmo_move_xy_no_constraint() {
        let actuator = GizmoMoveActuator::new();

        let delta = actuator.calculate_delta(Vec2::new(100.0, 50.0), GizmoAxis::XY);
        assert_eq!(delta.x, 100.0);
        assert_eq!(delta.y, 50.0);
    }

    #[test]
    fn test_gizmo_move_apply() {
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::ZERO, Vec2::new(50.0, 50.0));

        let actuator = GizmoMoveActuator::new();
        let cmd = actuator.apply_movement(entity, Vec2::new(100.0, 50.0), GizmoAxis::X, &mut store);

        match cmd {
            Some(Command::Move { id, delta }) => {
                assert_eq!(id, entity);
                assert_eq!(delta.x, 100.0);
                assert_eq!(delta.y, 0.0);
            }
            _ => panic!("Expected Move command"),
        }
    }

    #[test]
    fn test_gizmo_move_ignores_small_delta() {
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::ZERO, Vec2::new(50.0, 50.0));

        let actuator = GizmoMoveActuator::new();
        let cmd = actuator.apply_movement(entity, Vec2::new(0.0005, 0.0), GizmoAxis::X, &mut store);

        assert!(cmd.is_none());
    }

    #[test]
    fn test_gizmo_move_batch() {
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::ZERO, Vec2::new(50.0, 50.0));
        let e2 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let actuator = GizmoMoveActuator::new();
        let cmds = actuator.apply_batch_movement(
            &[e1, e2],
            Vec2::new(10.0, 20.0),
            GizmoAxis::XY,
            &mut store,
        );

        assert_eq!(cmds.len(), 2);
        match cmds[0] {
            Command::Move { id, delta } => {
                assert_eq!(id, e1);
                assert_eq!(delta, Vec2::new(10.0, 20.0));
            }
            _ => panic!("Expected Move command"),
        }
    }

    #[test]
    fn test_gizmo_move_display_format() {
        let actuator = GizmoMoveActuator::new();
        let display = actuator.format_display(Vec2::new(45.2, -30.5));

        assert!(display.contains("X: +45.2px"));
        assert!(display.contains("Y: -30.5px"));
    }

    // ═══════════════════════════════════════════════════════════════════════════════════
    // GizmoScaleActuator Tests
    // ═══════════════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_gizmo_scale_uniform() {
        let mut actuator = GizmoScaleActuator::new();
        actuator.set_center(Vec2::ZERO);

        let (sx, sy) = actuator.calculate_scale(
            Vec2::new(100.0, 100.0), // handle pos
            Vec2::new(50.0, 50.0),   // original size
            true,                    // uniform
            false,                   // from center
        );

        // With center at (0,0) and handle at (100,100), scale is:
        // current_size = (100, 100), current_x/y = 200 each, scale = 200/50 = 4.0
        assert!((sx - 4.0).abs() < 0.01);
        assert!((sy - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_gizmo_scale_non_uniform() {
        let mut actuator = GizmoScaleActuator::new();
        actuator.set_center(Vec2::ZERO);

        let (sx, sy) = actuator.calculate_scale(
            Vec2::new(100.0, 75.0), // handle pos (stretching X more)
            Vec2::new(50.0, 50.0),  // original size
            false,                  // non-uniform
            false,
        );

        // current_x = 200, current_y = 150, scale_x = 4.0, scale_y = 3.0
        assert!((sx - 4.0).abs() < 0.01);
        assert!((sy - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_gizmo_scale_apply() {
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::ZERO, Vec2::new(50.0, 50.0));

        let actuator = GizmoScaleActuator::new();
        let cmd = actuator.apply_scale(entity, Vec2::new(100.0, 100.0), &mut store);

        match cmd {
            Command::Resize { id, size } => {
                assert_eq!(id, entity);
                assert_eq!(size, Vec2::new(100.0, 100.0));
            }
            _ => panic!("Expected Resize command"),
        }
    }

    #[test]
    fn test_gizmo_scale_minimum_size() {
        let mut actuator = GizmoScaleActuator::new();
        actuator.set_center(Vec2::ZERO);

        let new_size = actuator.calculate_new_size(
            Vec2::new(50.0, 50.0),
            Vec2::new(-100.0, -100.0), // Would result in negative size
            false,
            false,
            GizmoHandleType::ScaleCorner,
        );

        assert!(new_size.x >= 10.0);
        assert!(new_size.y >= 10.0);
    }

    #[test]
    fn test_gizmo_scale_display_uniform() {
        let actuator = GizmoScaleActuator::new();
        let display = actuator.format_display(1.5, 1.5);

        assert!(display.contains("Scale: 1.50x"));
    }

    #[test]
    fn test_gizmo_scale_display_non_uniform() {
        let actuator = GizmoScaleActuator::new();
        let display = actuator.format_display(1.5, 0.8);

        assert!(display.contains("Scale: 1.50"));
        assert!(display.contains("×"));
    }

    // ═══════════════════════════════════════════════════════════════════════════════════
    // GizmoRotateActuator Tests
    // ═══════════════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_gizmo_rotate_angle_calculation() {
        let mut actuator = GizmoRotateActuator::new();
        actuator.set_center(Vec2::ZERO);

        // Point at (1, 0) should be 0 degrees
        let angle = actuator.calculate_angle(Vec2::new(1.0, 0.0));
        assert!((angle - 0.0).abs() < 0.1);

        // Point at (0, 1) should be 90 degrees
        let angle = actuator.calculate_angle(Vec2::new(0.0, 1.0));
        assert!((angle - 90.0).abs() < 0.1);

        // Point at (-1, 0) should be 180 degrees
        let angle = actuator.calculate_angle(Vec2::new(-1.0, 0.0));
        assert!((angle - 180.0).abs() < 0.1);

        // Point at (0, -1) should be 270 degrees
        let angle = actuator.calculate_angle(Vec2::new(0.0, -1.0));
        assert!((angle - 270.0).abs() < 0.1);
    }

    #[test]
    fn test_gizmo_rotate_snap() {
        let actuator = GizmoRotateActuator::new();

        // Without snapping
        let angle = actuator.calculate_rotation(16.0, false, false);
        assert!((angle - 16.0).abs() < 0.1);

        // With 15° snapping (should snap to 15)
        let angle = actuator.calculate_rotation(16.0, true, false);
        assert!((angle - 15.0).abs() < 0.1);

        // With 1° snapping (should snap to 16)
        let angle = actuator.calculate_rotation(16.0, true, true);
        assert!((angle - 16.0).abs() < 0.1);
    }

    #[test]
    fn test_gizmo_rotate_display() {
        let actuator = GizmoRotateActuator::new();
        let display = actuator.format_display(45.3);

        assert!(display.contains("45.3°"));
    }

    // ═══════════════════════════════════════════════════════════════════════════════════
    // GizmoHitTest Tests
    // ═══════════════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_hit_test_handle() {
        let handle = GizmoHandle {
            position: Vec2::new(100.0, 100.0),
            handle_type: GizmoHandleType::MoveX,
            screen_size: 12.0,
        };

        // Point inside handle radius (6 world units with zoom 1.0)
        assert!(GizmoHitTest::hit_test(Vec2::new(103.0, 100.0), handle, 1.0));

        // Point outside handle radius
        assert!(!GizmoHitTest::hit_test(
            Vec2::new(120.0, 100.0),
            handle,
            1.0
        ));
    }

    #[test]
    fn test_hit_test_rotation_circle() {
        let center = Vec2::ZERO;

        // Inside the ring
        assert!(GizmoHitTest::hit_test_rotation(
            Vec2::new(80.0, 0.0),
            center,
            80.0, // radius
            1.0,  // zoom
            8.0,  // thickness
        ));

        // Too far from ring
        assert!(!GizmoHitTest::hit_test_rotation(
            Vec2::new(100.0, 0.0),
            center,
            80.0,
            1.0,
            8.0,
        ));
    }

    // ═══════════════════════════════════════════════════════════════════════════════════
    // GizmoType Tests
    // ═══════════════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_gizmo_type_values() {
        assert_eq!(GizmoType::Move as u8, 0);
        assert_eq!(GizmoType::Scale as u8, 1);
        assert_eq!(GizmoType::Rotate as u8, 2);
        assert_eq!(GizmoType::Combined as u8, 3);
    }

    #[test]
    fn test_gizmo_axis_values() {
        assert_eq!(GizmoAxis::None as u8, 0);
        assert_eq!(GizmoAxis::X as u8, 1);
        assert_eq!(GizmoAxis::Y as u8, 2);
        assert_eq!(GizmoAxis::XY as u8, 3);
    }

    // ═══════════════════════════════════════════════════════════════════════════════════
    // GizmoConfig Tests
    // ═══════════════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_gizmo_config_defaults() {
        let config = GizmoConfig::default();

        assert_eq!(config.arrow_length, 60.0);
        assert_eq!(config.arrow_thickness, 8.0);
        assert_eq!(config.rotate_radius, 80.0);
        assert_eq!(config.handle_size, 12.0);
        assert_eq!(config.pivot_size, 8.0);
        assert_eq!(config.snap_angle, 15.0);
        assert_eq!(config.min_snap_angle, 1.0);
    }
}
