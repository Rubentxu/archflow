//! Transform Operations - Complete handle-based transformation mathematics
//!
//! This module provides comprehensive transformation support for:
//! - **ResizeOperation**: Handle-based resizing with all 8 resize handles
//! - **RotationOperation**: Rotation with snap-to-angle support
//! - **MultiTransform**: Multi-entity transformation with preserved offsets
//!
//! # Type State Pattern
//!
//! The operations use Rust's type state pattern to ensure operations
//! progress through valid states: Idle -> Dragging -> Completed
//!
//! # Examples
//!
//! ```rust
//! use archflow_sdk::selection::transform::{ResizeOperation, RotationOperation};
//! use archflow_sdk::selection::{HandleType, UnifiedBounds};
//! use archflow_core::{EntityId, Vec2};
//!
//! // Resize operation
//! let entity_id = EntityId::new();
//! let bounds = UnifiedBounds {
//!     min: Vec2::new(100.0, 100.0),
//!     max: Vec2::new(200.0, 200.0),
//!     center: Vec2::new(150.0, 150.0),
//!     width: 100.0,
//!     height: 100.0,
//! };
//!
//! let mut resize_op = ResizeOperation::new(entity_id, HandleType::ResizeSouthEast, bounds);
//! let dragging = resize_op.start_drag(Vec2::new(200.0, 200.0));
//! let result = dragging.update(Vec2::new(250.0, 250.0)).current_result();
//!
//! // Rotation operation
//! let mut rotation_op = RotationOperation::new(
//!     entity_id,
//!     Vec2::new(150.0, 150.0),  // center
//!     0.0,                       // original angle
//!     Vec2::new(150.0, 120.0),   // handle position
//! );
//! let dragging = rotation_op.start_drag(Vec2::new(150.0, 120.0));
//! let rotation_result = dragging.update(Vec2::new(180.0, 150.0)).current_result();
//! ```

pub mod multi_transform;
pub mod resize_operation;
pub mod rotation_operation;
pub mod transform_result;

pub use multi_transform::{CanvasAdapter, EntityTransform, MultiTransform, MultiTransformResult};
pub use resize_operation::{
    CompletedState, DraggingState, IdleState, MIN_HANDLE_SIZE, ResizeConfig, ResizeOperation,
    ResizeResult,
};
pub use rotation_operation::{
    CompletedRotationState, DEFAULT_SNAP_INCREMENT, DraggingRotationState, IdleRotationState,
    RotationAngle, RotationConfig, RotationOperation, RotationResult, rotate_bounds,
    rotate_point_around_center,
};
pub use transform_result::{TransformResult, TransformType};
