// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Web - WASM Bridge & Main Loop
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 21
//
// This crate contains the WASM bindings and main loop:
// - WASM bridge via wasm-bindgen
// - RequestAnimationFrame loop
// - SharedArrayBuffer input bridge
// - JS bindings for browser APIs
// ═══════════════════════════════════════════════════════════════════════════════

// TODO: Implement ArchFlowEngine, tick(), WASM bindings
// See: ARQUITECTURA_FINAL_V3.md - Section 21

pub mod bridge;
pub mod engine;
pub mod input;

pub use bridge::WasmBridge;
pub use engine::ArchFlowEngine;
pub use input::InputRingBuffer;
