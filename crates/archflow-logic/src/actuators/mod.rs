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
pub mod animation;
pub mod audio;
pub mod batch_select;
pub mod camera;
pub mod clipboard;
pub mod connections;
pub mod container;
pub mod gizmos;
pub mod highlight;
pub mod message;
pub mod move_;
pub mod property;
pub mod selection;
pub mod state;
pub mod swimlane;
pub mod z_order;

pub use alignment::{Alignment, AlignmentActuator, DistributionActuator, DistributionAxis};
pub use animation::AnimationActuator;
pub use audio::{AudioActuator, AudioCommand};
pub use batch_select::{BatchSelectActuator, DeltaMask, SelectMode};
pub use camera::{
    CameraActuator, CameraActuatorConfig, CameraConstraints, CameraTransform, Smoother,
};
pub use clipboard::{
    ClipboardData, ClipboardEntity, ClipboardState, CopyActuator, DeleteActuator,
    DuplicateActuator, PasteActuator,
};
pub use connections::{
    AnchorConfig, AnchorPoint, AnchorVisibilityActuator, AnchorVisibilityState, AnchorVisualConfig,
    ArrowBindActuator, AutoRouteActuator, ConnectionLabelActuator, ElbowConfig,
    ElbowRoutingActuator, LineStyleActuator, LineStyleChange, LineStyleConfig,
    PathOptimizationActuator, PathOptimizationConfig, PathOptimizationResult,
    PathOptimizationState,
};
pub use container::{ContainerActuator, ContainerOp, ContainerOpData};
pub use gizmos::{
    GizmoAxis, GizmoConfig, GizmoHandle, GizmoHandleType, GizmoHitResult, GizmoHitTest,
    GizmoMoveActuator, GizmoRotateActuator, GizmoScaleActuator, GizmoState, GizmoType,
    TransformGizmoActuator,
};
pub mod group;
pub use group::{GroupActuator, GroupConfig, GroupOp, GroupOpType, GroupResult};
pub use highlight::HighlightActuator;
pub use message::{Message, MessageActuator, MessageBus, MessagePayload};
pub use move_::{DragAxis, MoveActuator};
pub use property::{Property, PropertyActuator};
pub mod smart_guides;
pub mod snap;
pub use selection::{
    SelectActuator, SelectionConfig, SelectionMode, SelectionResult, SelectionState,
};
pub use smart_guides::{
    AlignmentType, SmartGuide, SmartGuidesActuator, SmartGuidesConfig, SmartGuidesResult,
};
pub use snap::{SnapConfig, SnapResult, SnapToGridActuator};
pub use state::{
    EntityState, StateActuator, StateBitset, StateId, StateMachine, StateManager, StateTransition,
    StateTransitionTable,
};
pub use swimlane::{SwimlaneActuator, SwimlaneConfig, SwimlaneOp, SwimlaneOrientation};
pub use z_order::{ZOrderActuator, ZOrderDirection, ZOrderOp};
