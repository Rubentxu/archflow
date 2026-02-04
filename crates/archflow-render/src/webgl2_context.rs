// ═══════════════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Render - WebGL2 Context for WASM
//
// This module provides a WebGL2 rendering context implementation using
// web_sys bindings for WASM environments.
//
// Uses CanvasRenderingContext2D for maximum compatibility as the primary
// rendering backend, with optional raw WebGL2 for advanced features.
// ═══════════════════════════════════════════════════════════════════════════════════════════════

#![cfg(feature = "wasm-bindgen")]

use alloc::format;
use wasm_bindgen::prelude::JsValue;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use super::{WebGl2Context as WebGl2ContextTrait, WebGl2Program};

/// WebGL2 Context implementation using Canvas 2D
pub struct WebGl2Context2D {
    context: CanvasRenderingContext2d,
    width: u32,
    height: u32,
    current_program: Option<WebGl2Program>,
}

impl WebGl2Context2D {
    /// Create a new WebGL2 context from a canvas element
    pub fn try_from_canvas(canvas: &HtmlCanvasElement) -> Result<Self, JsValue> {
        // Try WebGL2 first for better performance
        if let Ok(Some(context)) = canvas.get_context("webgl2") {
            // For WebGL2, we'd need a WebGL2-specific implementation
            // For now, fall back to 2D
        }

        // Fall back to Canvas 2D
        let context = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("Canvas 2D context not available"))?
            .dyn_into::<CanvasRenderingContext2d>()?;

        let width = canvas.width();
        let height = canvas.height();

        Ok(Self {
            context,
            width,
            height,
            current_program: None,
        })
    }
}

impl WebGl2ContextTrait for WebGl2Context2D {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    fn clear(&self, red: f32, green: f32, blue: f32, alpha: f32) {
        // Clear with background color
        let _ = self
            .context
            .clear_rect(0.0, 0.0, self.width as f64, self.height as f64);
        let css_color = format!(
            "rgba({}, {}, {}, {})",
            (red * 255.0) as u8,
            (green * 255.0) as u8,
            (blue * 255.0) as u8,
            alpha
        );
        self.context.set_fill_style(&JsValue::from_str(&css_color));
        let _ = self
            .context
            .fill_rect(0.0, 0.0, self.width as f64, self.height as f64);
    }

    fn draw_instanced(&self, _mode: u32, _first: i32, _count: i32, _instance_count: i32) {
        // Canvas 2D doesn't support instancing
        web_sys::console::warn_1(&JsValue::from_str(
            "draw_instanced not supported in Canvas 2D",
        ));
    }

    fn use_program(&self, _program: &WebGl2Program) {
        // Canvas 2D doesn't use shader programs
    }

    fn set_viewport(&mut self, _x: i32, _y: i32, width: i32, height: i32) {
        self.width = width as u32;
        self.height = height as u32;
    }

    fn enable_blending(&self) {
        // Canvas 2D has alpha blending by default
    }

    fn disable_blending(&self) {
        // Canvas 2D blending cannot be disabled
    }
}
