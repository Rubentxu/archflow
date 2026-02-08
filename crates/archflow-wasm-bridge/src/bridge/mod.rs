// ═══════════════════════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Web - Bridge Module
//
// This module provides the WASM bridge for JavaScript/WebAssembly communication.
// Architecture Reference: docs/analysis/ARCHITECTURE-CLEAN-BRIDGE.md
//
// Current Structure:
// - WasmBridge: Main facade for all WASM-exposed operations
// - Internal organization by concern: initialization, entity, selection, camera, input, history
//
// Note: Due to wasm_bindgen constraints, all WASM-exposed methods must be in a single impl block.
// The internal organization uses helper functions and organized code sections.
// ═══════════════════════════════════════════════════════════════════════════════════════════════════════

// Re-export WasmBridge as ArchFlowBridge for backward compatibility
pub use super::WasmBridge as ArchFlowBridge;
