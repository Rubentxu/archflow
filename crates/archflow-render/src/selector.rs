// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Render - Renderer Selector
//
// This module provides backend detection and renderer creation.
// Implements the RendererSelector pattern from the architecture.
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::boxed::Box;
use alloc::string::String;

use super::{RenderError, Renderer};

/// Available rendering backends
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// WebGPU - hardware accelerated, modern API
    WebGPU,

    /// WebGL2 - hardware accelerated, widely supported
    WebGL2,

    /// Canvas 2D - software rendering, universal support
    Canvas2D,
}

impl Backend {
    /// Get backend name as static string
    pub fn name(&self) -> &'static str {
        match self {
            Backend::WebGPU => "WebGPU",
            Backend::WebGL2 => "WebGL2",
            Backend::Canvas2D => "Canvas2D",
        }
    }
}

/// Renderer selector - detects and creates appropriate renderer
///
/// This struct implements the Strategy pattern for renderer selection.
/// It detects available backends and creates the best possible renderer.
pub struct RendererSelector;

impl RendererSelector {
    /// Detect best available backend and create renderer
    ///
    /// Tries backends in order: WebGPU → WebGL2 → Canvas2D
    /// Each fallback is attempted if previous backend initialization fails.
    ///
    /// # Returns
    ///
    /// `Result<Box<dyn Renderer>, RenderError>` - The created renderer or error
    #[cfg(feature = "wasm-bindgen")]
    pub fn detect_and_create() -> Result<Box<dyn Renderer>, RenderError> {
        // Default to Canvas2D for now (fallback only)
        // WebGL2 will be implemented in HU-RENDER-002
        // WebGPU will be refactored in HU-RENDER-001

        #[cfg(debug_assertions)]
        tracing::warn!(
            target: "archflow::render::selector",
            "Using Canvas2D fallback - renderer selection not fully implemented yet"
        );

        Err(RenderError::BackendNotAvailable(String::from(
            "Renderer selection not implemented",
        )))
    }

    /// Detect best available backend and create renderer (non-WASM)
    ///
    /// Always returns an error on non-WASM targets since rendering requires canvas.
    #[cfg(not(feature = "wasm-bindgen"))]
    pub fn detect_and_create() -> Result<Box<dyn Renderer>, RenderError> {
        Err(RenderError::BackendNotAvailable(String::from(
            "Rendering only available on WASM targets",
        )))
    }

    /// Check if WebGPU is available
    #[cfg(feature = "wasm-bindgen")]
    fn has_webgpu() -> bool {
        // WebGPU detection will be implemented with proper navigator.gpu check
        false
    }

    /// Check if WebGL2 is available
    #[cfg(feature = "wasm-bindgen")]
    fn has_webgl2() -> bool {
        // WebGL2 detection will be implemented with context check
        false
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_name() {
        assert_eq!(Backend::WebGPU.name(), "WebGPU");
        assert_eq!(Backend::WebGL2.name(), "WebGL2");
        assert_eq!(Backend::Canvas2D.name(), "Canvas2D");
    }

    #[test]
    fn test_backend_equality() {
        assert_eq!(Backend::WebGPU, Backend::WebGPU);
        assert_ne!(Backend::WebGPU, Backend::WebGL2);
        assert_ne!(Backend::WebGL2, Backend::Canvas2D);
    }

    #[test]
    fn test_selector_non_wasm_returns_error() {
        #[cfg(not(feature = "wasm-bindgen"))]
        {
            let result = RendererSelector::detect_and_create();
            assert!(result.is_err());
            match result {
                Err(RenderError::BackendNotAvailable(msg)) => {
                    assert!(msg.contains("WASM"));
                }
                _ => panic!("Expected BackendNotAvailable error"),
            }
        }
    }
}
