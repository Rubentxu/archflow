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

pub use bridge::WasmBridge;
pub use engine::ArchFlowEngine;
pub use input::{
    Buttons, InputEventType, InputProcessor, InputRingBuffer, Modifiers, RawInputEvent,
    EVENT_CAPACITY, EVENT_SIZE, MAX_POINTERS,
};
