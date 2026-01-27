//! archflow-wasm-collab - Zero-Copy WASM Bridge for JavaScript Collaboration
//!
//! This crate provides high-performance shared memory communication between
//! Rust (compiled to WebAssembly) and JavaScript.

pub mod binary_delta_codec;
pub mod shared_buffer;
pub mod wasm_bridge;

pub use binary_delta_codec::{BinaryDeltaCodec, DecodedDelta, ShapeField};
pub use shared_buffer::{RenderAttribute, SharedBuffer};
pub use wasm_bridge::WasmBridge;

use wasm_bindgen::prelude::*;

/// Initializes the WASM module.
///
/// Must be called once at startup. Verifies cross-origin isolation
/// is available for SharedArrayBuffer support.
#[wasm_bindgen(start)]
pub fn init() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    // Check for cross-origin isolation using JavaScript
    let isolated =
        js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("crossOriginIsolated"));

    let is_isolated = match isolated {
        Ok(val) => val.as_bool().unwrap_or(false),
        Err(_) => false,
    };

    if !is_isolated {
        return Err(JsValue::from_str(
            "Cross-origin isolation required for SharedArrayBuffer. \
             Server must send COOP and COEP headers. \
             See: https://developer.mozilla.org/en-US/docs/Web/API/Window/crossOriginIsolated",
        ));
    }

    log("archflow-wasm-collab initialized (cross-origin isolated)");
    Ok(())
}

/// Logs a message to the browser console.
#[wasm_bindgen]
pub fn log(s: &str) {
    web_sys::console::log_1(&s.into());
}

/// Logs an error to the browser console.
#[wasm_bindgen]
pub fn log_error(s: &str) {
    web_sys::console::error_1(&s.into());
}
