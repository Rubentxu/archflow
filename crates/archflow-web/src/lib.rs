// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Web - WASM Bridge & Main Loop
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Sections 7, 21
//
// This crate contains the WASM bindings and main loop:
// - Lock-free input via SharedArrayBuffer
// - WASM bridge via wasm-bindgen
// - RequestAnimationFrame loop
// - JS bindings for browser APIs
// ═══════════════════════════════════════════════════════════════════════════════

#![no_std]
#![warn(missing_docs)]
#![warn(clippy::all)]

extern crate alloc;

pub mod bridge;
pub mod engine;
pub mod input;
pub mod logic;

pub use bridge::WasmBridge;
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
};
