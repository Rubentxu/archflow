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
pub mod tween;
pub mod visibility;

pub use events::{EventData, EventRingBuffer, LogicEvent, LogicEventType};

pub use actuators::{
    // Alignment actuators
    AlignmentActuator,
    AlignmentTarget,
    // Batch selection (replaces legacy SelectActuator)
    BatchSelectActuator,
    // Camera types
    CameraActuator,
    CameraActuatorConfig,
    CameraConstraints,
    CameraTransform,
    // Container actuator
    ContainerActuator,
    ContainerConfig,
    // Clipboard operations
    CopyActuator,
    DeleteActuator,
    DeltaMask,
    // Movement
    DragAxis,
    DuplicateActuator,
    // State machine types
    EntityState,
    // Gizmo transforms
    GizmoType,
    // Grouping
    GroupCreateMode,
    GroupingActuator,
    GroupingMode,
    // Selection highlight
    HighlightActuator,
    HighlightConfig,
    HighlightStyle,
    HoverConfig,
    // Property modification
    PropertyActuator,
    PropertyOperation,
    // Selection
    SelectMode,
    // State actuator
    StateActuator,
    VisibilityActuator,
    // Z-order
    ZOrderActuator,
    ZOrderOperation,
};

pub use command::LogicCommand;
pub use ecs::{
    Component, ComponentId, ComponentRegistry, ComponentStorage, HighlightActuatorComponent,
    MouseSensorComponent, MoveActuatorComponent, SelectActuatorComponent, SignalStateComponent,
    SparseSet, VecStorage,
};

pub use input::{InputState, Key, KeyboardState, MouseButton, MouseState};

pub use logic_driver::{LogicDriver, LogicDriverConfig};

pub use logic_system::LogicSystem;

pub use mapping::{
    controller::{Controller, ControllerMode, LogicController, PulseCondition},
    mapping_table::LogicMappingTable,
    sensor_type::SensorType,
};

pub use physics_pulse::{PhysicsPulseEmitter, PhysicsPulseReceiver};

pub use pulse::{Pulse, PulseMode, PulseReceiver};

pub use sensors::{
    box_select::BoxSelectSensor, collision::CollisionSensor, key_shortcut::KeyShortcutSensor,
    mouse::MouseOverSensor, near::NearSensor, proximity::ProximitySensor, radar::RadarSensor,
    touch::TouchSensor,
};

pub use signals::{SensorOutput, SignalByte, SignalState};

pub use simd::{
    POSITION_BATCH_SIZE, SIGNAL_BATCH_SIZE, SIMD_SUPPORT, can_use_simd, has_simd_support,
    process_signals, process_signals_scalar, process_signals_simd, update_positions,
    update_positions_scalar, update_positions_simd,
};

pub use snap::{SnapActuator, SnapConfig, SnapGrid};

pub use spatial::SpatialIndex;

pub use tween::{EasingFunction, Tween, TweenActuator, TweenConfig, TweenState, TweenType};

pub use visibility::VisibilityChange;

// Declarative JSON API (requires std)
// Note: These types use serde_json and are primarily intended for non-WASM builds
pub use api::json::{
    BehaviorDefinition, BehaviorRegistry, ComponentCreator, ComponentDefinition, ComponentFactory,
    ComponentFactoryError,
};
