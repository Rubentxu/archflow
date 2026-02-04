// ═════════════════════════════════════════════════════════════════════════════
// ArchFlow Render - WebGL2 Context (Real WebGL2)
//
// This module provides actual WebGL2 rendering using glow bindings.
// Replaces the Canvas 2D fallback with proper GPU-accelerated rendering.
// ═══════════════════════════════════════════════════════════════════════════

use alloc::string::{String, ToString};
use glow::HasContext;

use crate::{RenderError, WebGl2Program};

/// WebGL2 context using glow for GPU-accelerated rendering
///
/// This is a production-ready WebGL2 context implementation that uses actual GPU
/// resources for rendering, not the Canvas 2D API.
#[cfg(feature = "wasm-bindgen")]
pub struct WebGL2Context {
    /// Glow GL context
    gl: glow::Context,

    /// Canvas element
    canvas: web_sys::HtmlCanvasElement,

    /// Current program
    current_program: Option<WebGl2Program>,

    /// Instance buffer ID
    instance_buffer: glow::Buffer,

    /// Vertex array object ID
    vao: glow::VertexArray,

    /// Canvas width
    width: u32,

    /// Canvas height
    height: u32,
}

#[cfg(feature = "wasm-bindgen")]
impl WebGL2Context {
    /// Create a new WebGL2 context from canvas
    ///
    /// # Errors
    ///
    /// Returns `RenderError::WebGL2` if context creation fails
    pub fn new(canvas: web_sys::HtmlCanvasElement) -> Result<Self, RenderError> {
        use wasm_bindgen::JsCast;

        // Get WebGL2 context
        let context = canvas
            .get_context("webgl2")
            .map_err(|e| RenderError::WebGL2(format!("Failed to get WebGL2 context: {:?}", e)))?
            .ok_or_else(|| RenderError::WebGL2(String::from("WebGL2 not supported")))?
            .dyn_into::<web_sys::WebGl2RenderingContext>()
            .map_err(|e| {
                RenderError::WebGL2(format!("Failed to cast to WebGL2 context: {:?}", e))
            })?;

        // Create glow context from web_sys WebGL2 context
        let gl = glow::Context::from_webgl2_context(context)
            .map_err(|e| RenderError::WebGL2(format!("Failed to create glow context: {:?}", e)))?;

        // Get canvas dimensions
        let width = canvas.width();
        let height = canvas.height();

        Ok(Self {
            gl,
            canvas,
            current_program: None,
            instance_buffer: unsafe { gl.create_buffer() }.map_err(|e| {
                RenderError::WebGL2(format!("Failed to create instance buffer: {:?}", e))
            })?,
            vao: unsafe { gl.create_vertex_array() }
                .map_err(|e| RenderError::WebGL2(format!("Failed to create VAO: {:?}", e)))?,
            width,
            height,
        })
    }

    /// Compile and link a shader program from GLSL sources
    pub fn create_program(
        &self,
        vertex_source: &str,
        fragment_source: &str,
    ) -> Result<WebGl2Program, RenderError> {
        // Compile vertex shader
        let vertex_shader = unsafe { self.gl.create_shader(glow::VERTEX_SHADER) }.map_err(|e| {
            RenderError::ShaderCompilation(format!("Failed to create vertex shader: {:?}", e))
        })?;

        unsafe { self.gl.shader_source(vertex_shader, vertex_source) }

        unsafe { self.gl.compile_shader(vertex_shader) }
        let compile_status = unsafe { self.gl.get_shader_compile_status(vertex_shader) };
        if !compile_status {
            let info_log = unsafe { self.gl.get_shader_info_log(vertex_shader) };
            return Err(RenderError::ShaderCompilation(format!(
                "Vertex shader compilation failed: {}",
                info_log
            )));
        }

        // Compile fragment shader
        let fragment_shader =
            unsafe { self.gl.create_shader(glow::FRAGMENT_SHADER) }.map_err(|e| {
                RenderError::ShaderCompilation(format!("Failed to create fragment shader: {:?}", e))
            })?;

        unsafe { self.gl.shader_source(fragment_shader, fragment_source) }

        unsafe { self.gl.compile_shader(fragment_shader) }
        let compile_status = unsafe { self.gl.get_shader_compile_status(fragment_shader) };
        if !compile_status {
            let info_log = unsafe { self.gl.get_shader_info_log(fragment_shader) };
            return Err(RenderError::ShaderCompilation(format!(
                "Fragment shader compilation failed: {}",
                info_log
            )));
        }

        // Create and link program
        let program = unsafe { self.gl.create_program() }
            .map_err(|e| RenderError::WebGL2(format!("Failed to create program: {:?}", e)))?;

        unsafe { self.gl.attach_shader(program, vertex_shader) }

        unsafe { self.gl.attach_shader(program, fragment_shader) }

        unsafe { self.gl.link_program(program) }

        let link_status = unsafe { self.gl.get_program_link_status(program) };
        if !link_status {
            let info_log = unsafe { self.gl.get_program_info_log(program) };
            return Err(RenderError::ShaderCompilation(format!(
                "Program linking failed: {}",
                info_log
            )));
        }

        // Clean up shaders
        unsafe { self.gl.delete_shader(vertex_shader) }
        unsafe { self.gl.delete_shader(fragment_shader) }

        Ok(WebGl2Program::new(program))
    }

    /// Use a shader program
    pub fn use_program(&mut self, program: &WebGl2Program) {
        self.current_program = Some(program.clone());
        unsafe {
            self.gl.use_program(Some(program.program_id()));
        }
    }

    /// Resize the viewport
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        unsafe { self.gl.viewport(0, 0, width as i32, height as i32) }
    }

    /// Clear framebuffer with given color
    pub fn clear(&self, red: f32, green: f32, blue: f32, alpha: f32) {
        unsafe {
            self.gl.clear_color(red, green, blue, alpha);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }
    }

    /// Enable alpha blending
    pub fn enable_blending(&self) {
        unsafe {
            self.gl.enable(glow::BLEND);
            self.gl
                .blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }
    }

    /// Disable alpha blending
    pub fn disable_blending(&self) {
        unsafe {
            self.gl.disable(glow::BLEND);
        }
    }

    /// Get canvas width
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get canvas height
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Upload instance data to GPU
    pub fn upload_instances(&self, data: &[u8]) {
        unsafe {
            self.gl
                .bind_buffer(glow::ARRAY_BUFFER, Some(self.instance_buffer));
            self.gl
                .buffer_data_u8_slice(glow::ARRAY_BUFFER, data, glow::STATIC_DRAW);
        }
    }

    /// Draw instanced
    pub fn draw_instanced(&self, mode: u32, count: i32, instance_count: i32) {
        unsafe {
            self.gl
                .draw_arrays_instanced(mode, 0, count, instance_count)
        }
    }

    /// Delete WebGL2 resources
    pub fn cleanup(&self) {
        unsafe {
            if self.instance_buffer != glow::Buffer::default() {
                self.gl.delete_buffer(self.instance_buffer);
            }
            if self.vao != glow::VertexArray::default() {
                self.gl.delete_vertex_array(self.vao);
            }
        }
    }
}
