// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Web - Logic Bricks WASM Bindings
//
// Epic 5: SDK TypeScript con WASM bindings
//
// This module provides WASM bindings for the Logic Bricks system:
// - SignalByte: Binary signal processing with 6-tick history
// - Sensors: MouseOver, MouseClick, Proximity, KeyShortcut
// - Actuators: Highlight, Select, Move
// - Logic Mapping Table: Sensor-Actuator connections with controllers
//
// All types are exposed to JavaScript via wasm-bindgen for use in web applications.
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]

pub mod actuator;
pub mod controller;
pub mod logic_system;
pub mod mapping_table;
pub mod sensor_type;
pub mod signal_byte;

pub use actuator::{
    CameraConfig, ExtendedActuatorType, HighlightConfig, MoveConfig, PropertyConfig, PropertyValue,
    SelectModeWasm,
};
pub use controller::{Controller, ControllerType};
pub use logic_system::{LogicSystemWasm, PulseWasm};
pub use mapping_table::{ActuatorType, LogicMappingTableWasm};
pub use sensor_type::SensorType;
pub use signal_byte::SignalByteWasm;
