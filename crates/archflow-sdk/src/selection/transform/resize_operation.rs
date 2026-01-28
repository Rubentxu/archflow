//! ResizeOperation - Complete handle-based resize mathematics
//!
//! This module provides the complete resize operation implementation with support for:
//! - All 8 resize handles (corners and edges)
//! - Aspect ratio constraint (Shift key)
//! - Center-based resizing (Alt key)
//! - Minimum size enforcement
//! - Automatic edge swapping when dragging past opposite corner

use crate::selection::{HandleType, UnifiedBounds};
use archflow_core::Vec2;
use serde::{Deserialize, Serialize};

/// Minimum size for any resize operation (in pixels)
pub const MIN_HANDLE_SIZE: f32 = 5.0;

/// Result of a resize operation containing the new bounds
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ResizeResult {
    /// New minimum corner
    pub min: Vec2,
    /// New maximum corner
    pub max: Vec2,
    /// Delta from original position
    pub delta: Vec2,
    /// Whether the operation was clamped
    pub was_clamped: bool,
}

impl ResizeResult {
    /// Calculate width from min/max
    #[inline]
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    /// Calculate height from min/max
    #[inline]
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    /// Calculate center point
    #[inline]
    pub fn center(&self) -> Vec2 {
        Vec2::new(
            self.min.x + self.width() / 2.0,
            self.min.y + self.height() / 2.0,
        )
    }
}

/// Configuration for resize operations
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ResizeConfig {
    /// Minimum allowed size
    pub min_size: f32,
    /// Whether to constrain aspect ratio by default
    pub constrain_aspect: bool,
    /// Whether to resize from center by default
    pub from_center: bool,
    /// Snap increment in pixels (0.0 = no snap)
    pub snap_increment: f32,
}

impl Default for ResizeConfig {
    fn default() -> Self {
        Self {
            min_size: MIN_HANDLE_SIZE,
            constrain_aspect: false,
            from_center: false,
            snap_increment: 0.0,
        }
    }
}

/// State marker for idle operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdleState;

/// State marker for active dragging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraggingState {
    /// Current mouse position
    pub current_mouse_pos: Vec2,
    /// Starting mouse position
    pub start_mouse_pos: Vec2,
}

/// State marker for completed operation (ready for undo)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedState {
    /// The final result of the resize operation
    pub final_result: ResizeResult,
}

/// Resize operation with type-state pattern
///
/// The generic parameter S tracks the operation state:
/// - IdleState: Operation created but not started
/// - DraggingState: User is dragging a handle
/// - CompletedState: Drag completed, result available
#[derive(Debug, Clone)]
pub struct ResizeOperation<S> {
    entity_id: archflow_core::EntityId,
    original_bounds: UnifiedBounds,
    handle: HandleType,
    config: ResizeConfig,
    state: S,
}

impl ResizeOperation<IdleState> {
    /// Create a new idle resize operation
    #[inline]
    pub fn new(
        entity_id: archflow_core::EntityId,
        handle: HandleType,
        bounds: UnifiedBounds,
    ) -> Self {
        Self::with_config(entity_id, handle, bounds, ResizeConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(
        entity_id: archflow_core::EntityId,
        handle: HandleType,
        bounds: UnifiedBounds,
        config: ResizeConfig,
    ) -> Self {
        Self {
            entity_id,
            original_bounds: bounds,
            handle,
            config,
            state: IdleState,
        }
    }

    /// Start the drag operation
    pub fn start_drag(self, mouse_pos: Vec2) -> ResizeOperation<DraggingState> {
        ResizeOperation {
            entity_id: self.entity_id,
            original_bounds: self.original_bounds,
            handle: self.handle,
            config: self.config,
            state: DraggingState {
                current_mouse_pos: mouse_pos,
                start_mouse_pos: mouse_pos,
            },
        }
    }
}

impl ResizeOperation<DraggingState> {
    /// Get current mouse position
    #[inline]
    pub fn current_mouse_pos(&self) -> Vec2 {
        self.state.current_mouse_pos
    }

    /// Get start mouse position
    #[inline]
    pub fn start_mouse_pos(&self) -> Vec2 {
        self.state.start_mouse_pos
    }

    /// Update the drag position
    pub fn update(mut self, mouse_pos: Vec2) -> Self {
        self.state.current_mouse_pos = mouse_pos;
        self
    }

    /// Calculate current resize result
    pub fn current_result(&self) -> ResizeResult {
        let constrain_aspect = self.config.constrain_aspect;
        let from_center = self.config.from_center;
        self.calculate_resize(self.state.current_mouse_pos, constrain_aspect, from_center)
    }

    /// Complete the resize operation and transition to completed state
    pub fn complete(self) -> ResizeOperation<CompletedState> {
        let result = self.current_result();

        ResizeOperation {
            entity_id: self.entity_id,
            original_bounds: self.original_bounds,
            handle: self.handle,
            config: self.config,
            state: CompletedState {
                final_result: result,
            },
        }
    }

    /// Calculate the resize bounds with all modifiers applied
    fn calculate_resize(
        &self,
        mouse_pos: Vec2,
        constrain_aspect: bool,
        from_center: bool,
    ) -> ResizeResult {
        let original = self.original_bounds;
        let (mut new_min, mut new_max) = (original.min, original.max);
        let mut was_clamped = false;

        // Apply snap if configured
        let snapped_pos = if self.config.snap_increment > 0.0 {
            snap_to_grid(mouse_pos, self.config.snap_increment)
        } else {
            mouse_pos
        };

        match self.handle {
            HandleType::ResizeNorthWest => {
                if from_center {
                    let delta = snapped_pos - original.center;
                    let abs_delta_x = delta.x.abs();
                    let abs_delta_y = delta.y.abs();

                    if constrain_aspect {
                        let max_delta = abs_delta_x.max(abs_delta_y);
                        let sign_x = if delta.x >= 0.0 { 1.0 } else { -1.0 };
                        let sign_y = if delta.y >= 0.0 { 1.0 } else { -1.0 };
                        let adjusted_delta = Vec2::new(max_delta * sign_x, max_delta * sign_y);

                        new_min = original.center - adjusted_delta;
                        new_max = original.center + adjusted_delta;
                    } else {
                        new_min = original.center - delta;
                        new_max = original.center + delta;
                    }
                } else {
                    new_min = snapped_pos;
                }
            }

            HandleType::ResizeNorth => {
                if from_center {
                    let delta = snapped_pos.y - original.center.y;
                    new_min.y = original.center.y - delta;
                    new_max.y = original.center.y + delta;
                } else {
                    new_min.y = snapped_pos.y;
                }
            }

            HandleType::ResizeNorthEast => {
                if from_center {
                    let delta = Vec2::new(
                        snapped_pos.x - original.center.x,
                        original.center.y - snapped_pos.y,
                    );

                    if constrain_aspect {
                        let abs_delta_x = delta.x.abs();
                        let abs_delta_y = delta.y.abs();
                        let max_delta = abs_delta_x.max(abs_delta_y);
                        let sign_x = if delta.x >= 0.0 { 1.0 } else { -1.0 };
                        let sign_y = if delta.y >= 0.0 { 1.0 } else { -1.0 };

                        new_min = Vec2::new(
                            original.center.x - max_delta * sign_x,
                            original.center.y - max_delta * sign_y,
                        );
                        new_max = Vec2::new(
                            original.center.x + max_delta * sign_x,
                            original.center.y + max_delta * sign_y,
                        );
                    } else {
                        new_min =
                            Vec2::new(original.center.x - delta.x, original.center.y - delta.y);
                        new_max =
                            Vec2::new(original.center.x + delta.x, original.center.y + delta.y);
                    }
                } else {
                    new_min.y = snapped_pos.y;
                    new_max.x = snapped_pos.x;
                }
            }

            HandleType::ResizeEast => {
                if from_center {
                    let delta = snapped_pos.x - original.center.x;
                    new_min.x = original.center.x - delta;
                    new_max.x = original.center.x + delta;
                } else {
                    new_max.x = snapped_pos.x;
                }
            }

            HandleType::ResizeSouthEast => {
                if from_center {
                    let delta = snapped_pos - original.center;

                    if constrain_aspect {
                        let abs_delta_x = delta.x.abs();
                        let abs_delta_y = delta.y.abs();
                        let max_delta = abs_delta_x.max(abs_delta_y);
                        let sign_x = if delta.x >= 0.0 { 1.0 } else { -1.0 };
                        let sign_y = if delta.y >= 0.0 { 1.0 } else { -1.0 };
                        let adjusted_delta = Vec2::new(max_delta * sign_x, max_delta * sign_y);

                        new_min = original.center - adjusted_delta;
                        new_max = original.center + adjusted_delta;
                    } else {
                        new_min = original.center - delta;
                        new_max = original.center + delta;
                    }
                } else {
                    new_max = snapped_pos;
                }
            }

            HandleType::ResizeSouth => {
                if from_center {
                    let delta = snapped_pos.y - original.center.y;
                    new_min.y = original.center.y - delta;
                    new_max.y = original.center.y + delta;
                } else {
                    new_max.y = snapped_pos.y;
                }
            }

            HandleType::ResizeSouthWest => {
                if from_center {
                    let delta = Vec2::new(
                        original.center.x - snapped_pos.x,
                        snapped_pos.y - original.center.y,
                    );

                    if constrain_aspect {
                        let abs_delta_x = delta.x.abs();
                        let abs_delta_y = delta.y.abs();
                        let max_delta = abs_delta_x.max(abs_delta_y);
                        let sign_x = if delta.x >= 0.0 { 1.0 } else { -1.0 };
                        let sign_y = if delta.y >= 0.0 { 1.0 } else { -1.0 };

                        new_min = Vec2::new(
                            original.center.x - max_delta * sign_x,
                            original.center.y - max_delta * sign_y,
                        );
                        new_max = Vec2::new(
                            original.center.x + max_delta * sign_x,
                            original.center.y + max_delta * sign_y,
                        );
                    } else {
                        new_min =
                            Vec2::new(original.center.x - delta.x, original.center.y - delta.y);
                        new_max =
                            Vec2::new(original.center.x + delta.x, original.center.y + delta.y);
                    }
                } else {
                    new_min.x = snapped_pos.x;
                    new_max.y = snapped_pos.y;
                }
            }

            HandleType::ResizeWest => {
                if from_center {
                    let delta = snapped_pos.x - original.center.x;
                    new_min.x = original.center.x - delta;
                    new_max.x = original.center.x + delta;
                } else {
                    new_min.x = snapped_pos.x;
                }
            }

            HandleType::Rotate => {
                return ResizeResult {
                    min: original.min,
                    max: original.max,
                    delta: Vec2::ZERO,
                    was_clamped: false,
                };
            }
        }

        // Ensure min is always less than max
        if new_min.x > new_max.x {
            std::mem::swap(&mut new_min.x, &mut new_max.x);
        }
        if new_min.y > new_max.y {
            std::mem::swap(&mut new_min.y, &mut new_max.y);
        }

        // Enforce minimum size
        let width = new_max.x - new_min.x;
        let height = new_max.y - new_min.y;

        if width < self.config.min_size {
            let center_x = (new_min.x + new_max.x) / 2.0;
            new_min.x = center_x - self.config.min_size / 2.0;
            new_max.x = center_x + self.config.min_size / 2.0;
            was_clamped = true;
        }

        if height < self.config.min_size {
            let center_y = (new_min.y + new_max.y) / 2.0;
            new_min.y = center_y - self.config.min_size / 2.0;
            new_max.y = center_y + self.config.min_size / 2.0;
            was_clamped = true;
        }

        if constrain_aspect {
            let current_width = new_max.x - new_min.x;
            let current_height = new_max.y - new_min.y;

            if current_width > current_height {
                let center_y = (new_min.y + new_max.y) / 2.0;
                let new_height = current_width;
                new_min.y = center_y - new_height / 2.0;
                new_max.y = center_y + new_height / 2.0;
            } else {
                let center_x = (new_min.x + new_max.x) / 2.0;
                let new_width = current_height;
                new_min.x = center_x - new_width / 2.0;
                new_max.x = center_x + new_width / 2.0;
            }
            was_clamped = true;
        }

        let delta = Vec2::new(
            (new_min.x + new_max.x) / 2.0 - original.center.x,
            (new_min.y + new_max.y) / 2.0 - original.center.y,
        );

        ResizeResult {
            min: new_min,
            max: new_max,
            delta,
            was_clamped,
        }
    }
}

impl ResizeOperation<CompletedState> {
    /// Get the final result
    #[inline]
    pub fn result(&self) -> ResizeResult {
        self.state.final_result
    }
}

impl<S> ResizeOperation<S> {
    /// Get the entity ID being resized
    #[inline]
    pub fn entity_id(&self) -> archflow_core::EntityId {
        self.entity_id
    }

    /// Get the handle type
    #[inline]
    pub fn handle(&self) -> HandleType {
        self.handle
    }

    /// Get the original bounds
    #[inline]
    pub fn original_bounds(&self) -> UnifiedBounds {
        self.original_bounds
    }

    /// Get the resize configuration
    #[inline]
    pub fn config(&self) -> ResizeConfig {
        self.config
    }
}

/// Snap a position to a grid
#[inline]
fn snap_to_grid(pos: Vec2, increment: f32) -> Vec2 {
    Vec2::new(
        (pos.x / increment).round() * increment,
        (pos.y / increment).round() * increment,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selection::HandleType;
    use archflow_core::EntityId;

    fn create_test_bounds() -> UnifiedBounds {
        UnifiedBounds {
            min: Vec2::new(100.0, 100.0),
            max: Vec2::new(200.0, 200.0),
            center: Vec2::new(150.0, 150.0),
            width: 100.0,
            height: 100.0,
        }
    }

    #[test]
    fn test_new_operation() {
        let entity_id = EntityId::new();
        let bounds = create_test_bounds();

        let operation = ResizeOperation::new(entity_id, HandleType::ResizeSouthEast, bounds);

        assert_eq!(operation.entity_id(), entity_id);
        assert_eq!(operation.handle(), HandleType::ResizeSouthEast);
        assert_eq!(operation.original_bounds().min, bounds.min);
    }

    #[test]
    fn test_south_east_resize() {
        let entity_id = EntityId::new();
        let bounds = create_test_bounds();

        let operation = ResizeOperation::new(entity_id, HandleType::ResizeSouthEast, bounds);
        let dragging = operation.start_drag(Vec2::new(200.0, 200.0));
        let result = dragging.update(Vec2::new(250.0, 280.0)).current_result();

        assert_eq!(result.max.x, 250.0);
        assert_eq!(result.max.y, 280.0);
        assert_eq!(result.min, bounds.min);
    }

    #[test]
    fn test_north_west_resize() {
        let entity_id = EntityId::new();
        let bounds = create_test_bounds();

        let operation = ResizeOperation::new(entity_id, HandleType::ResizeNorthWest, bounds);
        let dragging = operation.start_drag(Vec2::new(100.0, 100.0));
        let result = dragging.update(Vec2::new(80.0, 80.0)).current_result();

        assert_eq!(result.min.x, 80.0);
        assert_eq!(result.min.y, 80.0);
        assert_eq!(result.max, bounds.max);
    }

    #[test]
    fn test_edge_swap_on_cross() {
        let entity_id = EntityId::new();
        let bounds = create_test_bounds();

        let operation = ResizeOperation::new(entity_id, HandleType::ResizeNorthWest, bounds);
        let dragging = operation.start_drag(Vec2::new(100.0, 100.0));
        let result = dragging.update(Vec2::new(250.0, 250.0)).current_result();

        assert!(result.min.x > bounds.min.x || result.min.y > bounds.min.y);
    }

    #[test]
    fn test_center_resize() {
        let entity_id = EntityId::new();
        let bounds = create_test_bounds();

        let config = ResizeConfig {
            min_size: MIN_HANDLE_SIZE,
            constrain_aspect: false,
            from_center: true,
            snap_increment: 0.0,
        };

        let operation =
            ResizeOperation::with_config(entity_id, HandleType::ResizeSouthEast, bounds, config);
        let dragging = operation.start_drag(Vec2::new(200.0, 200.0));
        let result = dragging.update(Vec2::new(250.0, 250.0)).current_result();

        let new_center = result.center();
        assert!((new_center.x - 150.0).abs() < 0.001);
        assert!((new_center.y - 150.0).abs() < 0.001);
    }

    #[test]
    fn test_aspect_ratio_constraint() {
        let entity_id = EntityId::new();
        let bounds = create_test_bounds();

        let config = ResizeConfig {
            min_size: MIN_HANDLE_SIZE,
            constrain_aspect: true,
            from_center: false,
            snap_increment: 0.0,
        };

        let operation =
            ResizeOperation::with_config(entity_id, HandleType::ResizeEast, bounds, config);
        let dragging = operation.start_drag(Vec2::new(200.0, 150.0));
        let result = dragging.update(Vec2::new(300.0, 150.0)).current_result();

        assert!((result.width() - result.height()).abs() < 0.001);
    }

    #[test]
    fn test_minimum_size_enforcement() {
        let entity_id = EntityId::new();
        let bounds = create_test_bounds();

        let operation = ResizeOperation::new(entity_id, HandleType::ResizeSouthEast, bounds);
        let dragging = operation.start_drag(Vec2::new(200.0, 200.0));
        let result = dragging.update(Vec2::new(102.0, 102.0)).current_result();

        assert!(result.width() >= MIN_HANDLE_SIZE);
        assert!(result.height() >= MIN_HANDLE_SIZE);
        assert!(result.was_clamped);
    }

    #[test]
    fn test_complete_operation() {
        let entity_id = EntityId::new();
        let bounds = create_test_bounds();

        let operation = ResizeOperation::new(entity_id, HandleType::ResizeSouthEast, bounds);
        let dragging = operation.start_drag(Vec2::new(200.0, 200.0));
        let completed = dragging.update(Vec2::new(250.0, 250.0)).complete();

        let result = completed.result();
        assert_eq!(result.max.x, 250.0);
        assert_eq!(result.max.y, 250.0);
    }

    #[test]
    fn test_snap_to_grid() {
        let pos = Vec2::new(13.7, 27.3);
        let snapped = snap_to_grid(pos, 10.0);

        assert!((snapped.x - 10.0).abs() < 0.001);
        assert!((snapped.y - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_rotate_handle_returns_original() {
        let entity_id = EntityId::new();
        let bounds = create_test_bounds();

        let operation = ResizeOperation::new(entity_id, HandleType::Rotate, bounds);
        let dragging = operation.start_drag(Vec2::new(150.0, 80.0));
        let result = dragging.update(Vec2::new(160.0, 70.0)).current_result();

        assert_eq!(result.min, bounds.min);
        assert_eq!(result.max, bounds.max);
    }
}
