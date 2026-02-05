// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Actuators Module
//
// Actuators transform sensor signals into Commands that modify EntityStore state.
//
// Architecture:
// - Sensors (input) → Actuators (transformation) → Commands (output)
// - Actuators maintain state for undo/restore operations
// - Each actuator generates zero or more commands per update
// ═══════════════════════════════════════════════════════════════════════════════

pub mod alignment;
pub mod batch_select;
pub mod camera;
pub mod clipboard;
pub mod connections;
pub mod gizmos;
pub mod highlight;
pub mod message;
pub mod move_;
pub mod property;
pub mod state;
pub mod z_order;

pub use alignment::{Alignment, AlignmentActuator, DistributionActuator, DistributionAxis};
pub use batch_select::{BatchSelectActuator, DeltaMask, SelectMode};
pub use camera::{
    CameraActuator, CameraActuatorConfig, CameraConstraints, CameraTransform, Smoother,
};
pub use clipboard::{
    ClipboardData, ClipboardEntity, ClipboardState, CopyActuator, DeleteActuator,
    DuplicateActuator, PasteActuator,
};
pub use connections::{
    AnchorConfig, AnchorPoint, ArrowBindActuator, AutoRouteActuator, ConnectionLabelActuator,
    ConnectionStyle, ElbowConfig, ElbowRoutingActuator,
};
pub use gizmos::{
    GizmoAxis, GizmoConfig, GizmoHandle, GizmoHandleType, GizmoHitResult, GizmoHitTest,
    GizmoMoveActuator, GizmoRotateActuator, GizmoScaleActuator, GizmoState, GizmoType,
    TransformGizmoActuator,
};
pub use highlight::HighlightActuator;
pub use message::{Message, MessageActuator, MessageBus, MessagePayload};
pub use move_::MoveActuator;
pub use property::{Property, PropertyActuator};
pub use state::{
    EntityState, StateActuator, StateBitset, StateId, StateMachine, StateManager, StateTransition,
    StateTransitionTable,
};
pub use z_order::{ZOrderActuator, ZOrderDirection, ZOrderOp};
