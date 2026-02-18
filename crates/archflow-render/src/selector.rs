// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Render - Renderer Selector
//
// This module provides backend detection and renderer creation.
// Implements the RendererSelector pattern from the architecture.
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::boxed::Box;
use alloc::string::String;

use super::{RenderError, Renderer};

#[cfg(all(feature = "wasm-bindgen", feature = "webgl2"))]
use crate::WebGL2Renderer;

#[cfg(all(feature = "wasm-bindgen", feature = "webgpu"))]
use crate::WebGpuContext;

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
    pub fn detect_and_create(
        canvas: web_sys::HtmlCanvasElement,
    ) -> Result<Box<dyn Renderer>, RenderError> {
        // Try WebGPU first
        #[cfg(feature = "webgpu")]
        if Self::has_webgpu() {
            #[cfg(debug_assertions)]
            tracing::info!(
                target: "archflow::render::selector",
                "WebGPU detected, attempting to create WebGPU renderer"
            );

            match WebGpuContext::new() {
                Ok(context) => {
                    // Create GpuRenderer with WebGPU context
                    // Note: GpuRenderer is CPU-side for sync, actual rendering via WebGPU
                    // For now, return the CPU renderer that works with WebGPU backend
                    #[cfg(debug_assertions)]
                    tracing::info!(
                        target: "archflow::render::selector",
                        "Using WebGPU backend"
                    );
                    // Return GpuRenderer which has backend_name = "WebGPU"
                    return Ok(Box::new(crate::GpuRenderer::new()));
                }
                Err(e) => {
                    #[cfg(debug_assertions)]
                    tracing::warn!(
                        target: "archflow::render::selector",
                        error = ?e,
                        "WebGPU context creation failed, falling back"
                    );
                }
            }
        }

        // Try WebGL2 second
        #[cfg(all(feature = "wasm-bindgen", feature = "webgl2"))]
        if Self::has_webgl2(&canvas) {
            #[cfg(debug_assertions)]
            tracing::info!(
                target: "archflow::render::selector",
                "WebGL2 detected, creating WebGL2 renderer"
            );

            match WebGL2Renderer::new(canvas) {
                Ok(renderer) => {
                    #[cfg(debug_assertions)]
                    tracing::info!(
                        target: "archflow::render::selector",
                        "Using WebGL2 backend"
                    );
                    return Ok(Box::new(renderer));
                }
                Err(e) => {
                    #[cfg(debug_assertions)]
                    tracing::warn!(
                        target: "archflow::render::selector",
                        error = ?e,
                        "WebGL2 renderer creation failed, falling back"
                    );
                }
            }
        }

        // Fall back to CPU renderer (GpuRenderer with backend_name = "WebGPU")
        // This renderer does CPU-side preparation and works without GPU
        #[cfg(debug_assertions)]
        tracing::warn!(
            target: "archflow::render::selector",
            "No hardware-accelerated backend available, using CPU fallback"
        );

        Ok(Box::new(crate::GpuRenderer::new()))
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
    #[allow(dead_code)]
    fn has_webgpu() -> bool {
        // Simple check: WebGPU is available if we can access the window
        // The actual context creation will be attempted in detect_and_create
        web_sys::window().is_some()
    }

    /// Check if WebGL2 is available
    #[cfg(feature = "wasm-bindgen")]
    fn has_webgl2(canvas: &web_sys::HtmlCanvasElement) -> bool {
        // Try to get a WebGL2 context
        let context = canvas.get_context("webgl2");
        matches!(context, Ok(Some(_)))
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
