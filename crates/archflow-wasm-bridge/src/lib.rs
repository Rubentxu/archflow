// ═══════════════════════════════════════════════════════════════════════════════════════
// ArchFlow WASM Bridge - Rust/JavaScript Communication Layer
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Sections 7, 21
//
// This crate provides WASM bindings for browser communication:
// - Lock-free input via SharedArrayBuffer
// - WASM bridge via wasm-bindgen
// - RequestAnimationFrame loop
// - JS bindings for browser APIs
//
// Modular Bridge Architecture (docs/analysis/ARCHITECTURE-CLEAN-BRIDGE.md):
// - WasmBridge: Main facade for all WASM-exposed operations
// - Organized by concern: initialization, entities, selection, camera, input, history
//
// Behavior JSON API (Developer Manual):
// - Declarative JSON API for behavior definitions
// - Following A-Frame pattern for developer-friendly configuration
// ═══════════════════════════════════════════════════════════════════════════════════════

#![no_std]
#![warn(missing_docs)]
#![warn(clippy::all)]

extern crate alloc;

pub mod behavior_json;
pub mod bridge;
pub mod bridge_wasm;
pub mod engine;
pub mod input;
pub mod logic;
pub mod logic_bricks_setup;

// Re-export bridge facade from bridge/ directory
pub use self::bridge::ArchFlowBridge;

// Re-export legacy WasmBridge from bridge_wasm.rs
pub use self::bridge_wasm::WasmBridge;

pub use engine::ArchFlowEngine;
pub use input::{
    Buttons, EVENT_CAPACITY, EVENT_SIZE, InputEventType, InputProcessor, InputRingBuffer,
    MAX_POINTERS, Modifiers, RawInputEvent,
};

// Re-export HU-003 InputSampler from archflow-logic
pub use archflow_logic::{
    InputEvent, InputSampler, InputSnapshotSAB, MAX_KEYS as INPUT_SAB_MAX_KEYS,
    MouseButton as InputMouseButton,
};

// Logic Bricks WASM exports (Epic 5)
pub use logic::{
    // Re-export core types
    ActuatorType,
    BrickChainBuilder,
    BrickHandle,
    CameraConfig,
    Controller,
    ControllerType,
    ExtendedActuatorType,
    HighlightConfig,
    LogicMappingTableWasm,
    LogicSystemWasm,
    MoveConfig,
    PropertyConfig,
    PropertyValue,
    SelectModeWasm,
    SensorType,
    SignalByteWasm,
    // Factory functions
    actuator_delete,
    actuator_emit_event,
    actuator_highlight,
    actuator_move,
    actuator_select_clear,
    actuator_select_multi,
    actuator_select_single,
    actuator_select_toggle,
    factory_and,
    factory_blinky,
    factory_custom,
    factory_debounce,
    factory_direct,
    factory_hysteresis,
    factory_nand,
    factory_nor,
    factory_not,
    factory_or,
    factory_pattern,
    factory_threshold,
    factory_xor,
    sensor_collision_detect,
    sensor_double_tap,
    sensor_keyboard_key,
    sensor_long_press,
    sensor_mouse_click,
    sensor_mouse_drag,
    sensor_mouse_hover,
    sensor_mouse_wheel,
    sensor_property_changed,
    sensor_timer_delay,
    sensor_timer_interval,
};

// Behavior JSON API exports
pub use behavior_json::{
    // Component configurations
    ActuatorDeleteConfig,
    ActuatorEventConfig,
    ActuatorHighlightConfig,
    ActuatorMoveConfig,
    ActuatorPropertyConfig,
    ActuatorSelectConfig,
    BehaviorDefinition,
    BehaviorError,
    BehaviorRegistry,
    ControllerDebounceConfig,
    ControllerDirectConfig,
    ControllerHysteresisConfig,
    SensorKeyboardConfig,
    SensorMouseConfig,
    SensorPropertyConfig,
    SensorTimerConfig,
};
