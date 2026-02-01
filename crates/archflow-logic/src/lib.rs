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

pub use actuators::{
    HighlightActuator, MoveActuator, Property, PropertyActuator, SelectActuator, SelectMode,
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
