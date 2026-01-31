// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Signal Processing Module
//
// This crate provides the Logic Bricks system for ArchFlow Engine:
// - SignalByte: 6-tick history in 1 byte
// - Sensors: MouseOver, MouseClick, Proximity, KeyShortcut
// - Actuators: Highlight, Select, Move, etc.
// - Logic Mapping: Connect sensors to actuators with controllers
//
// Architecture Reference:
// - LOGIC_BRICKS_FEASIBILITY_STUDY.md
// - LOGIC_BRICKS_EPICS.md
// ═══════════════════════════════════════════════════════════════════════════════

#![no_std]
#![warn(missing_docs)]
#![warn(clippy::all)]

extern crate alloc;

pub mod actuators;
pub mod mapping;
pub mod sensors;
pub mod signals;

pub use actuators::{HighlightActuator, MoveActuator, SelectActuator, SelectMode};
pub use mapping::{Controller, LogicMappingTable, SensorType};
pub use sensors::mouse_over::MouseOverSensor;
pub use signals::SignalByte;
