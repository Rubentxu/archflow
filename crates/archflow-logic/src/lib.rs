// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Signal Processing Module
//
// This crate provides the Logic Bricks system for ArchFlow Engine:
// - Pulse System: SensorState, Pulse, PulseBus (Blender BGE pattern)
// - SignalByte: 6-tick history in 1 byte
// - Sensors: MouseOver, MouseClick, Proximity, KeyShortcut
// - Actuators: Highlight, Select, Move, etc.
// - Logic Mapping: Connect sensors to actuators with controllers
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
pub mod input;
pub mod logic_system;
pub mod mapping;
pub mod pulse;
pub mod sensors;
pub mod signals;
pub mod snap;
pub mod spatial;
pub mod tween;

pub use actuators::{
    HighlightActuator, Message, MessageActuator, MessageBus, MessagePayload, MoveActuator,
    Property, PropertyActuator, SelectActuator, SelectMode,
};
pub use input::{InputEvent, InputSampler, InputSnapshotSAB, MouseButton, MAX_KEYS};
pub use logic_system::LogicSystem;
pub use mapping::{Controller, LogicMappingTable, SensorType};
pub use pulse::{Pulse, PulseBus, SensorState};
pub use sensors::{
    DoubleTapSensor, KeyShortcutSensor, LongPressSensor, MouseClickSensor, MouseConfig, MouseMode,
    MouseOverSensor, MouseSensor, PointerButtons, ProximitySensor, RightClickSensor, DOUBLE_TAP_MS,
    LONG_PRESS_MS, TAP_TIMEOUT_MS,
};
pub use signals::SignalByte;
pub use snap::{
    EntityEdge, SnapConfig, SnapPoint, SnapResult, SnapTarget, Snapper,
    DEFAULT_GRID_SIZE as SNAP_DEFAULT_GRID_SIZE, DEFAULT_THRESHOLD as SNAP_DEFAULT_THRESHOLD,
};
pub use spatial::{GridCoord, Rect, SpatialHashGrid, DEFAULT_GRID_SIZE};
pub use tween::{
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
    // Types
    Easing,
    Tween,
    TweenManager,
    TweenProperty,
    TweenState,
    DEFAULT_DURATION_MS as TWEEN_DEFAULT_DURATION_MS,
};
