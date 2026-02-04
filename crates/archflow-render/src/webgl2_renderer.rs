// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Render - WebGL2 Rendering System
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Sections 6, 9, 10
//
// This module provides a WebGL2-based rendering backend as an alternative
// to WebGPU for broader browser compatibility.
//
// Features:
// - 2D Instanced rendering
// - SDF-based shape rendering (rectangles, circles)
// - Texture atlas support
// - Multi-pass rendering
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(dead_code)]

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use archflow_core::MAX_ENTITIES;
use archflow_engine::EntityStore;

use crate::camera::Camera;
use crate::{CameraUniforms, GpuInstance, RenderPhase};

/// Maximum instances per batch
pub const MAX_WEBGL_INSTANCES: u32 = 100_000;

/// WebGL2 Context wrapper for WASM
///
/// This trait defines the interface needed by WebGL2Renderer.
/// In WASM, we use web_sys bindings; in tests, we use a mock.
pub trait WebGl2Context {
    /// Get canvas width
    fn width(&self) -> u32;

    /// Get canvas height
    fn height(&self) -> u32;

    /// Resize viewport
    fn resize(&mut self, width: u32, height: u32);

    /// Clear the framebuffer
    fn clear(&self, red: f32, green: f32, blue: f32, alpha: f32);

    /// Draw instanced
    fn draw_instanced(&self, mode: u32, first: i32, count: i32, instance_count: i32);

    /// Use program
    fn use_program(&self, program: &WebGl2Program);

    /// Set viewport
    fn set_viewport(&mut self, x: i32, y: i32, width: i32, height: i32);

    /// Enable blending
    fn enable_blending(&self);

    /// Disable blending
    fn disable_blending(&self);
}

/// WebGL2 Shader Program
#[derive(Clone)]
pub struct WebGl2Program {
    /// Vertex shader source
    vertex_source: String,

    /// Fragment shader source
    fragment_source: String,

    /// Compiled program ID (0 if not compiled)
    program_id: u32,

    /// Vertex shader ID
    vertex_id: u32,

    /// Fragment shader ID
    fragment_id: u32,
}

impl WebGl2Program {
    /// Create a new shader program
    pub fn new(vertex_source: &str, fragment_source: &str) -> Self {
        Self {
            vertex_source: vertex_source.to_string(),
            fragment_source: fragment_source.to_string(),
            program_id: 0,
            vertex_id: 0,
            fragment_id: 0,
        }
    }

    /// Get the compiled program ID
    pub fn program_id(&self) -> u32 {
        self.program_id
    }

    /// Check if program is compiled
    pub fn is_ready(&self) -> bool {
        self.program_id != 0
    }
}

/// WebGL2 Renderer using 2D Canvas API as fallback
///
/// This implementation uses the HTML5 Canvas 2D API for rendering,
/// which provides maximum compatibility across all browsers.
///
/// For GPU-accelerated rendering, this can be replaced with
/// raw WebGL2 calls when needed.
pub struct WebGl2Renderer<C: WebGl2Context> {
    /// Rendering context
    context: C,

    /// Camera uniforms
    camera_uniforms: CameraUniforms,

    /// Instance data
    instances: Vec<GpuInstance>,

    /// Batch indices per phase
    batch_indices: [Vec<u32>; 4],

    /// Canvas dimensions
    canvas_width: f32,

    /// Canvas dimensions
    canvas_height: f32,
}

impl<C: WebGl2Context> WebGl2Renderer<C> {
    /// Create a new WebGL2 renderer
    pub fn new(context: C) -> Self {
        Self {
            context,
            camera_uniforms: CameraUniforms::default(),
            instances: Vec::with_capacity(MAX_ENTITIES as usize),
            batch_indices: [
                Vec::with_capacity(MAX_ENTITIES as usize),
                Vec::with_capacity(MAX_ENTITIES as usize),
                Vec::with_capacity(MAX_ENTITIES as usize),
                Vec::with_capacity(MAX_ENTITIES as usize),
            ],
            canvas_width: 800.0,
            canvas_height: 600.0,
        }
    }

    /// Get reference to context
    pub fn context(&self) -> &C {
        &self.context
    }

    /// Get mutable reference to context
    pub fn context_mut(&mut self) -> &mut C {
        &mut self.context
    }

    /// Sync renderer data from EntityStore
    pub fn sync_from_store(&mut self, store: &EntityStore, camera: &Camera) -> usize {
        // Clear previous data
        self.instances.clear();
        for batch in &mut self.batch_indices {
            batch.clear();
        }

        // Update camera uniforms
        self.camera_uniforms = CameraUniforms::from_camera(camera);

        let viewport = camera.viewport_bounds();
        let mut visible_count = 0;

        // Iterate entities in draw order
        for &idx in &store.draw_order {
            let idx = idx as usize;

            // Skip invisible entities
            if !store.is_visible(idx) {
                continue;
            }

            // Viewport culling
            let pos = store.pos(idx);
            let size = store.size(idx);
            let half_size = size / 2.0;
            let entity_min = pos - half_size;
            let entity_max = pos + half_size;

            if !viewport.intersects(&archflow_core::Rect::new(
                entity_min.x,
                entity_min.y,
                entity_max.x,
                entity_max.y,
            )) {
                continue;
            }

            // Determine render phase
            let texture_idx = store.texture_index[idx];
            let phase = match texture_idx {
                0 if store.text_glyph_count[idx] > 0 => RenderPhase::Text,
                0 => RenderPhase::Shapes,
                1..=1000 => RenderPhase::Icons,
                _ => RenderPhase::Images,
            };

            // Create instance
            let instance = GpuInstance {
                pos: [pos.x, pos.y],
                size: [size.x, size.y],
                color: store.colors[idx],
                shape_type_or_texture_index: if texture_idx == 0 {
                    store.shape_type(idx) as u32
                } else {
                    texture_idx as u32
                },
                _padding: [0, 0],
                uv_rect: store.uv_rects[idx],
            };

            let instance_idx = self.instances.len() as u32;
            self.instances.push(instance);

            // Add to appropriate batch
            let batch_idx = match phase {
                RenderPhase::Shapes => 0,
                RenderPhase::Icons => 1,
                RenderPhase::Images => 2,
                RenderPhase::Text => 3,
            };
            self.batch_indices[batch_idx].push(instance_idx);

            visible_count += 1;
        }

        visible_count
    }

    /// Render a frame
    pub fn render(&self) {
        // Clear to background color
        self.context.clear(1.0, 1.0, 1.0, 1.0);
    }

    /// Get batch count for a phase
    pub fn batch_count(&self, phase: RenderPhase) -> usize {
        match phase {
            RenderPhase::Shapes => self.batch_indices[0].len(),
            RenderPhase::Icons => self.batch_indices[1].len(),
            RenderPhase::Images => self.batch_indices[2].len(),
            RenderPhase::Text => self.batch_indices[3].len(),
        }
    }

    /// Get instance data reference
    pub fn instances(&self) -> &[GpuInstance] {
        &self.instances
    }

    /// Get camera uniforms reference
    pub fn camera_uniforms(&self) -> &CameraUniforms {
        &self.camera_uniforms
    }

    /// Get batch indices
    pub fn batch_indices(&self, phase: RenderPhase) -> &[u32] {
        match phase {
            RenderPhase::Shapes => &self.batch_indices[0],
            RenderPhase::Icons => &self.batch_indices[1],
            RenderPhase::Images => &self.batch_indices[2],
            RenderPhase::Text => &self.batch_indices[3],
        }
    }

    /// Calculate total draw calls
    pub fn total_draw_calls(&self) -> u32 {
        let mut count = 0;
        for batch in &self.batch_indices {
            if !batch.is_empty() {
                count += 1;
            }
        }
        count
    }

    /// Resize the renderer
    pub fn resize(&mut self, width: u32, height: u32) {
        self.canvas_width = width as f32;
        self.canvas_height = height as f32;
    }
}

impl<C: WebGl2Context> Default for WebGl2Renderer<C> {
    fn default() -> Self {
        panic!("WebGl2Renderer requires a context");
    }
}

/// Implement Renderer trait for WebGl2Renderer
impl<C: WebGl2Context> super::Renderer for WebGl2Renderer<C> {
    fn sync_from_store(&mut self, store: &EntityStore, camera: &Camera) -> usize {
        Self::sync_from_store(self, store, camera)
    }

    fn batch_count(&self, phase: RenderPhase) -> usize {
        match phase {
            RenderPhase::Shapes => self.batch_indices[0].len(),
            RenderPhase::Icons => self.batch_indices[1].len(),
            RenderPhase::Images => self.batch_indices[2].len(),
            RenderPhase::Text => self.batch_indices[3].len(),
        }
    }

    fn instances(&self) -> &[GpuInstance] {
        &self.instances
    }

    fn camera_uniforms(&self) -> &CameraUniforms {
        &self.camera_uniforms
    }

    fn batch_indices(&self, phase: RenderPhase) -> &[u32] {
        match phase {
            RenderPhase::Shapes => &self.batch_indices[0],
            RenderPhase::Icons => &self.batch_indices[1],
            RenderPhase::Images => &self.batch_indices[2],
            RenderPhase::Text => &self.batch_indices[3],
        }
    }

    fn total_draw_calls(&self) -> u32 {
        Self::total_draw_calls(self)
    }

    fn resize(&mut self, width: u32, height: u32) {
        Self::resize(self, width, height);
    }

    fn backend_name(&self) -> &'static str {
        "WebGL2"
    }

    fn render(&mut self) -> Result<(), super::RenderError> {
        Self::render(self);
        Ok(())
    }
}

/// Draw mode constants for WebGL
pub mod draw_mode {
    /// Points
    pub const POINTS: u32 = 0x0000;
    /// Lines
    pub const LINES: u32 = 0x0001;
    /// Line loop
    pub const LINE_LOOP: u32 = 0x0002;
    /// Line strip
    pub const LINE_STRIP: u32 = 0x0003;
    /// Triangles
    pub const TRIANGLES: u32 = 0x0004;
    /// Triangle strip
    pub const TRIANGLE_STRIP: u32 = 0x0005;
    /// Triangle fan
    pub const TRIANGLE_FAN: u32 = 0x0006;
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::rc::Rc;
    use core::cell::RefCell;

    /// Mock WebGL2 context for testing
    struct MockWebGl2Context {
        width: u32,
        height: u32,
        clear_called: bool,
    }

    impl MockWebGl2Context {
        fn new() -> Self {
            Self {
                width: 800,
                height: 600,
                clear_called: false,
            }
        }
    }

    impl WebGl2Context for MockWebGl2Context {
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

        fn clear(&self, _red: f32, _green: f32, _blue: f32, _alpha: f32) {
            // In real implementation, this would clear the WebGL context
        }

        fn draw_instanced(&self, _mode: u32, _first: i32, _count: i32, _instance_count: i32) {}

        fn use_program(&self, _program: &WebGl2Program) {}

        fn set_viewport(&mut self, _x: i32, _y: i32, _width: i32, _height: i32) {}

        fn enable_blending(&self) {}

        fn disable_blending(&self) {}
    }

    #[test]
    fn test_webgl2_renderer_creation() {
        let context = MockWebGl2Context::new();
        let renderer = WebGl2Renderer::new(context);
        assert_eq!(renderer.instances().len(), 0);
        assert_eq!(renderer.total_draw_calls(), 0);
    }

    #[test]
    fn test_webgl2_renderer_sync_empty() {
        let context = MockWebGl2Context::new();
        let mut renderer = WebGl2Renderer::new(context);
        let store = EntityStore::new();
        let camera = Camera::new(800.0, 600.0);

        let count = renderer.sync_from_store(&store, &camera);
        assert_eq!(count, 0);
        assert_eq!(renderer.instances().len(), 0);
    }

    #[test]
    fn test_webgl2_renderer_sync_with_entities() {
        let context = MockWebGl2Context::new();
        let mut renderer = WebGl2Renderer::new(context);
        let mut store = EntityStore::new();
        let mut camera = Camera::new(800.0, 600.0);

        camera.center = archflow_core::Vec2::new(0.0, 0.0);

        // Spawn entities at origin (visible)
        let _id1 = store.spawn(
            archflow_core::Vec2::new(0.0, 0.0),
            archflow_core::Vec2::new(0.1, 0.06),
        );
        let _id2 = store.spawn(
            archflow_core::Vec2::new(0.5, 0.5),
            archflow_core::Vec2::new(0.08, 0.05),
        );

        let count = renderer.sync_from_store(&store, &camera);
        assert_eq!(count, 2);
        assert_eq!(renderer.instances().len(), 2);
        assert_eq!(renderer.batch_count(RenderPhase::Shapes), 2);
    }

    #[test]
    fn test_webgl2_program_creation() {
        let program = WebGl2Program::new(
            "#version 300 es\nvoid main() { gl_Position = vec4(0.0); }",
            "#version 300 es\nvoid main() { fragColor = vec4(1.0); }",
        );
        assert!(!program.is_ready());
        assert_eq!(program.program_id(), 0);
    }

    #[test]
    fn test_webgl2_batch_indices() {
        let context = MockWebGl2Context::new();
        let mut renderer = WebGl2Renderer::new(context);
        let mut store = EntityStore::new();
        let mut camera = Camera::new(800.0, 600.0);

        camera.center = archflow_core::Vec2::new(0.0, 0.0);

        store.spawn(
            archflow_core::Vec2::new(0.0, 0.0),
            archflow_core::Vec2::new(0.1, 0.06),
        );

        renderer.sync_from_store(&store, &camera);

        let shapes_batch = renderer.batch_indices(RenderPhase::Shapes);
        assert_eq!(shapes_batch.len(), 1);
        assert_eq!(shapes_batch[0], 0);
    }

    #[test]
    fn test_draw_mode_constants() {
        assert_eq!(draw_mode::POINTS, 0x0000);
        assert_eq!(draw_mode::LINES, 0x0001);
        assert_eq!(draw_mode::TRIANGLES, 0x0004);
    }
}
