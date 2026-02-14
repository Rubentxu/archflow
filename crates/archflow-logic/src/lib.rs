// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Signal Processing Module
//
// This crate provides the Logic Bricks system for ArchFlow Engine:
// - Pulse System: SensorState, Pulse, PulseBus (Blender BGE pattern)
// - SignalByte: 6-tick history in 1 byte
// - Sensors: MouseOver, MouseClick, Proximity, KeyShortcut
// - Actuators: Highlight, Select, Move, etc.
// - Logic Mapping: Connect sensors to actuators with controllers
// - ECS: Entity Component System for flexible component management
//
// Architecture Reference:
// - docs/integration/LOGIC_BRICKS_MIGRATION_PLAN.md
// - Blender Game Engine: KX_ISensor::Evaluate(), SCA_ILogicController
//
// Key Pattern: Sensors are PULSE PRODUCERS, not state stores.
// They emit SensorState::Positive/Negative/None which flows through controllers.
// ═══════════════════════════════════════════════════════════════════════════════

#![no_std]
#![warn(missing_docs)]
#![warn(clippy::all)]

extern crate alloc;

pub mod actuators;
pub mod api;
pub mod audio;
pub mod command;
pub mod ecs;
pub mod events;
pub mod input;
pub mod logic_driver;
pub mod logic_system;
pub mod mapping;
pub mod physics_pulse;
pub mod pulse;
pub mod sensors;
pub mod signals;
pub mod simd;
pub mod snap;
pub mod spatial;
pub mod tool;
pub mod tween;
pub mod visibility;

// NOTE: archflow_wasm_bridge::BehaviorEngine import removed to avoid circular dependency
// The wasm-bridge crate depends on archflow-logic, not the other way around

pub use audio::{AudioSystem, SoundInfo};
pub use events::{EventData, EventRingBuffer, LogicEvent, LogicEventType};

pub use actuators::{
    // Alignment actuators
    Alignment,
    AlignmentActuator,
    // Smart guides
    AlignmentType,
    // Audio actuator
    AudioActuator,
    // Batch selection (replaces legacy SelectActuator)
    BatchSelectActuator,
    // Camera types
    CameraActuator,
    CameraActuatorConfig,
    CameraConstraints,
    CameraTransform,
    // Clipboard operations
    ClipboardData,
    ClipboardEntity,
    ClipboardState,
    // Container actuator
    ContainerActuator,
    ContainerOp,
    ContainerOpData,
    CopyActuator,
    DeleteActuator,
    DeltaMask,
    DistributionActuator,
    DistributionAxis,
    // Movement
    DragAxis,
    DuplicateActuator,
    // State machine types
    EntityState,
    // Gizmo transforms
    GizmoAxis,
    GizmoConfig,
    GizmoHandle,
    GizmoHandleType,
    GizmoHitResult,
    GizmoHitTest,
    GizmoMoveActuator,
    GizmoRotateActuator,
    GizmoScaleActuator,
    GizmoState,
    GizmoType,
    // Grouping
    GroupActuator,
    GroupConfig,
    GroupOp,
    GroupOpType,
    GroupResult,
    // Selection highlight
    HighlightActuator,
    // Message types
    Message,
    MessageActuator,
    MessageBus,
    MessagePayload,
    MoveActuator,
    PasteActuator,
    // Property modification
    Property,
    PropertyActuator,
    // Selection
    SelectActuator,
    SelectMode,
    SelectionConfig,
    SelectionMode,
    SelectionResult,
    SelectionState,
    SmartGuide,
    SmartGuidesActuator,
    SmartGuidesConfig,
    SmartGuidesResult,
    Smoother,
    // Snap to grid
    SnapToGridActuator,
    StateActuator,
    StateBitset,
    StateId,
    StateMachine,
    StateManager,
    StateTransition,
    StateTransitionTable,
    // Swimlane
    SwimlaneActuator,
    SwimlaneConfig,
    SwimlaneOp,
    SwimlaneOrientation,
    TransformGizmoActuator,
    // Z-order
    ZOrderActuator,
    ZOrderDirection,
    ZOrderOp,
};

pub use command::{Command, CommandHistory, DEFAULT_MAX_HISTORY};
pub use input::{InputEvent, InputSampler, InputSnapshotSAB, MAX_KEYS, MouseButton};
pub use logic_driver::LogicDriver;
pub use logic_system::LogicSystem;
pub use mapping::{ActuatorType, Controller, LogicMappingTable, SensorType};
pub use pulse::Pulse;
pub use signals::{SensorOutput, SignalByte, SignalState};
pub use snap::{
    DEFAULT_GRID_SIZE as SNAP_DEFAULT_GRID_SIZE, DEFAULT_THRESHOLD as SNAP_DEFAULT_THRESHOLD,
    EntityEdge, SnapConfig, SnapPoint, SnapResult, SnapTarget, Snapper,
};
pub use spatial::{DEFAULT_GRID_SIZE, GridCoord, Rect, SpatialHashGrid};
pub use tool::{DEFAULT_TOOL, ToolActuator, ToolConfig, ToolState, ToolType};
pub use tween::{
    DEFAULT_DURATION_MS as TWEEN_DEFAULT_DURATION_MS,
    // Types
    Easing,
    Tween,
    TweenManager,
    TweenProperty,
    TweenState,
    // Easing functions (re-exported for convenience)
    ease_back_out as tween_ease_back_out,
    ease_bounce_out as tween_ease_bounce_out,
    ease_cubic_in as tween_ease_cubic_in,
    ease_cubic_in_out as tween_ease_cubic_in_out,
    ease_cubic_out as tween_ease_cubic_out,
    ease_elastic_out as tween_ease_elastic_out,
    ease_linear as tween_ease_linear,
    ease_quad_in as tween_ease_quad_in,
    ease_quad_in_out as tween_ease_quad_in_out,
    ease_quad_out as tween_ease_quad_out,
    ease_sine_in as tween_ease_sine_in,
    ease_sine_in_out as tween_ease_sine_in_out,
    ease_sine_out as tween_ease_sine_out,
    // Convenience functions
    tween_opacity,
    tween_position,
};
pub use visibility::{VisibilityActuator, VisibilityBitset, VisibilityConfig, VisibilityManager};

// ECS module - Entity Component System for flexible component management
pub use ecs::{
    // Hybrid BGE integration
    ActuatorComponent,
    // Core ECS types
    Archetype,
    ArchetypeId,
    ArchetypeStorage,
    // Audio component for entities
    AudioActuatorComponent,
    BatchIter,
    // Systems
    BgeLogicConfig,
    BgeLogicStats,
    BgeLogicSystem,
    ClickType,
    Component,
    ComponentColumn,
    ComponentId,
    // Registry and storage
    ComponentRegistry,
    ComponentStorage,
    ControllerComponent,
    EntityId,
    // Component implementations
    HighlightActuatorComponent,
    MoveActuatorComponent,
    // Query types
    Query,
    QueryMut,
    QueryParameter,
    SelectActuatorComponent,
    SensorComponent,
    SensorComponentType,
    SensorConfig,
    SensorEvaluation,
    SensorRef,
    SignalStateComponent,
    SimdBatchIterator,
    // Storage
    SparseSet,
    // System execution
    System,
    SystemInfo,
    SystemScheduler,
    VecStorage,
    // Filters
    With,
    Without,
    // World
    World,
};

// SIMD module exports (WASM-only)
pub use simd::{
    POSITION_BATCH_SIZE, SIGNAL_BATCH_SIZE, SIMD_SUPPORT, can_use_simd, has_simd_support,
    process_signals, process_signals_scalar, update_positions, update_positions_scalar,
};

// Declarative JSON API (requires std feature)
// Provides A-Frame compatible declarative entity/component definitions
#[cfg(feature = "std")]
pub use api::json::{
    BehaviorDefinition, ComponentDefinition, ComponentFactory, DefaultComponentFactory,
    EntityDefinition, FogSettings, Scene, SceneLoadResult, SceneLoader, SceneLoaderError,
    SceneMetadata,
};
