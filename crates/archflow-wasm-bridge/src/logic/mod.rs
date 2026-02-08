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
// - Fluent Bricks API: BrickChainBuilder for ergonomic JS API
//
// All types are exposed to JavaScript via wasm-bindgen for use in web applications.
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]

pub mod actuator;
pub mod brick_chain_builder;
pub mod brick_handle;
pub mod callback_registry;
pub mod controller;
pub mod event_buffer;
pub mod factories;
pub mod logic_system;
pub mod mapping_table;
pub mod sensor_type;
pub mod signal_byte;

pub use actuator::{
    CameraConfig, ExtendedActuatorType, HighlightConfig, MoveConfig, PropertyConfig, PropertyValue,
    SelectModeWasm,
};
pub use brick_chain_builder::BrickChainBuilder;
pub use brick_handle::BrickHandle;
pub use controller::{Controller, ControllerType};
pub use event_buffer::{EventRingBufferWasm, EventType as EventEventType, JsLogicEvent};
pub use factories::{
    actuator_delete, actuator_emit_event, actuator_highlight, actuator_move, actuator_select_clear,
    actuator_select_multi, actuator_select_single, actuator_select_toggle, factory_and,
    factory_blinky, factory_custom, factory_debounce, factory_direct, factory_hysteresis,
    factory_nand, factory_nor, factory_not, factory_or, factory_pattern, factory_threshold,
    factory_xor, sensor_collision_detect, sensor_double_tap, sensor_keyboard_key,
    sensor_long_press, sensor_mouse_click, sensor_mouse_drag, sensor_mouse_hover,
    sensor_mouse_wheel, sensor_property_changed, sensor_timer_delay, sensor_timer_interval,
};
pub use logic_system::{JsLogicEventData, LogicSystemWasm, PulseWasm};
pub use mapping_table::{ActuatorType, LogicMappingTableWasm};
pub use sensor_type::SensorType;
pub use signal_byte::SignalByteWasm;
