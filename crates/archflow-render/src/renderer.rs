// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Render - GpuRenderer with Multi-Phase Instancing
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 9
//
// Multi-Phase Instancing Renderer:
// - Phase 1: Shapes (SDF-based rectangles, circles, lines)
// - Phase 2: Icons (texture atlas lookup)
// - Phase 3: Images (texture2D array)
// - Phase 4: Text (MTSDF atlas)
//
// Why Multi-Phase vs Single Pipeline:
// - Single pipeline with branching causes SIMD divergence
// - Specialized pipelines = better cache coherency
// - 4 draw calls vs 1, but each is 4-8x faster
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(dead_code)]

use alloc::vec::Vec;

use archflow_core::{MAX_ENTITIES, Vec2f64};
use archflow_engine::EntityStore;

use crate::camera::Camera;

/// Maximum instances per draw call
pub const MAX_INSTANCES_PER_DRAW: u32 = 100_000;

/// Render phase for specialized pipelines
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderPhase {
    /// SDF-based shapes (rectangles, circles, lines, etc.)
    Shapes = 0,
    /// Icon textures from atlas
    Icons = 1,
    /// Images from texture2D array
    Images = 2,
    /// Text from MTSDF atlas
    Text = 3,
}

/// GPU instance data (48 bytes with 16-byte alignment)
///
/// Layout designed for WebGPU storage buffer access:
/// - All transforms in first 16 bytes (pos + size)
/// - Color and shape data in next 16 bytes
/// - UV rectangle in final 16 bytes
/// - Total: 48 bytes (3 x 16-byte aligned slots)
///
/// Layout is carefully organized to avoid padding:
/// - pos: [f32; 2] = 8 bytes
/// - size: [f32; 2] = 8 bytes (total 16)
/// - color: u32 = 4 bytes
/// - shape_type_or_texture_index: u32 = 4 bytes
/// - _padding: [u32; 2] = 8 bytes (total 16, aligned)
/// - uv_rect: [f32; 4] = 16 bytes (total 32)
///
/// Note: Fields are organized to ensure Pod compliance (no padding bytes).
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuInstance {
    /// Position [x, y] in world coordinates
    pub pos: [f32; 2],

    /// Size [w, h] in world coordinates
    pub size: [f32; 2],

    /// Packed color as 0xRRGGBBAA
    pub color: u32,

    /// Shape type (0-15) or texture index
    /// For shapes: 0=Rect, 1=Circle, 2=Ellipse, 3=Line, etc.
    /// For textures: Index into texture atlas/array
    pub shape_type_or_texture_index: u32,

    /// Padding to align to 16 bytes
    pub _padding: [u32; 2],

    /// UV rectangle for texture sampling [u, v, w, h]
    /// Normalized 0-1 coordinates in texture atlas
    pub uv_rect: [f32; 4],
}

/// Camera uniforms for vertex shader (64 bytes)
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniforms {
    /// View-projection matrix (column-major, 16 floats = 64 bytes)
    pub view_projection: [[f32; 4]; 4],
}

impl Default for CameraUniforms {
    fn default() -> Self {
        Self {
            view_projection: [[0.0; 4]; 4],
        }
    }
}

impl CameraUniforms {
    /// Create uniforms from camera
    pub fn from_camera(camera: &Camera) -> Self {
        // Camera already returns the matrix in [[f32; 4]; 4] format
        Self {
            view_projection: camera.build_view_projection_matrix(),
        }
    }
}

/// Draw batches for each render phase
///
/// Stores indices into the entity buffer for each phase.
/// Pre-allocated and reused every frame to avoid allocations.
#[derive(Default)]
struct DrawBatches {
    /// Entity indices for shape rendering
    shapes: Vec<u32>,

    /// Entity indices for icon rendering
    icons: Vec<u32>,

    /// Entity indices for image rendering
    images: Vec<u32>,

    /// Entity indices for text rendering
    text: Vec<u32>,
}

impl DrawBatches {
    /// Create new empty batches with pre-allocated capacity
    fn new() -> Self {
        Self {
            shapes: Vec::with_capacity(MAX_ENTITIES as usize),
            icons: Vec::with_capacity(MAX_ENTITIES as usize),
            images: Vec::with_capacity(MAX_ENTITIES as usize),
            text: Vec::with_capacity(MAX_ENTITIES as usize),
        }
    }

    /// Clear all batches for reuse
    fn clear(&mut self) {
        self.shapes.clear();
        self.icons.clear();
        self.images.clear();
        self.text.clear();
    }

    /// Get the batch for a given phase
    fn get_batch(&mut self, phase: RenderPhase) -> &mut Vec<u32> {
        match phase {
            RenderPhase::Shapes => &mut self.shapes,
            RenderPhase::Icons => &mut self.icons,
            RenderPhase::Images => &mut self.images,
            RenderPhase::Text => &mut self.text,
        }
    }

    /// Get the count of entities in a batch
    fn batch_count(&self, phase: RenderPhase) -> usize {
        match phase {
            RenderPhase::Shapes => self.shapes.len(),
            RenderPhase::Icons => self.icons.len(),
            RenderPhase::Images => self.images.len(),
            RenderPhase::Text => self.text.len(),
        }
    }
}

/// GPU Renderer with Multi-Phase Instancing
///
/// This is a simplified, testable version that focuses on:
/// 1. Proper data structure layout for WebGPU
/// 2. Batch organization by render phase
/// 3. Instance data preparation from EntityStore
///
/// The actual WebGPU rendering will be added in a future update.
pub struct GpuRenderer {
    /// Instance data buffer (CPU-side, ready for GPU upload)
    instances: Vec<GpuInstance>,

    /// Draw batches organized by render phase
    batches: DrawBatches,

    /// Camera uniforms (updated every frame)
    camera_uniforms: CameraUniforms,
}

impl GpuRenderer {
    /// Create a new GPU renderer
    pub fn new() -> Self {
        Self {
            instances: Vec::with_capacity(MAX_ENTITIES as usize),
            batches: DrawBatches::new(),
            camera_uniforms: CameraUniforms::default(),
        }
    }

    /// Sync renderer data from EntityStore
    ///
    /// This is the main preparation step before rendering:
    /// 1. Clear previous batches
    /// 2. Iterate through visible entities in draw order
    /// 3. Sort entities into phase-specific batches
    /// 4. Prepare instance data for GPU upload
    ///
    /// Returns the number of visible entities prepared
    pub fn sync_from_store(&mut self, store: &EntityStore, camera: &Camera) -> usize {
        // Clear previous frame data
        self.batches.clear();
        self.instances.clear();

        // Update camera uniforms
        self.camera_uniforms = CameraUniforms::from_camera(camera);

        let viewport = camera.viewport_bounds();
        let mut visible_count = 0;

        // Iterate entities in draw order (back-to-front for proper z-layering)
        for &idx in &store.draw_order {
            let idx = idx as usize;

            // Skip if not visible
            if !store.is_visible(idx) {
                continue;
            }

            // Skip if outside viewport (culling)
            let pos = store.pos(idx);
            let size = store.size(idx);
            let entity_min = pos - size / 2.0;
            let entity_max = pos + size / 2.0;

            if !viewport.intersects(&archflow_core::Rect::new(
                entity_min.x,
                entity_min.y,
                entity_max.x,
                entity_max.y,
            )) {
                continue;
            }

            // Determine render phase based on texture index
            let texture_idx = store.texture_index[idx];
            let phase = match texture_idx {
                0 => {
                    // Solid color - check if it's text
                    if store.text_glyph_count[idx] > 0 {
                        RenderPhase::Text
                    } else {
                        RenderPhase::Shapes
                    }
                }
                1..=1000 => RenderPhase::Icons,
                _ => RenderPhase::Images,
            };

            // Create instance data
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

            // Add to instances and batch
            let instance_idx = self.instances.len() as u32;
            self.instances.push(instance);
            self.batches.get_batch(phase).push(instance_idx);

            visible_count += 1;
        }

        visible_count
    }

    /// Get the number of entities in a specific render phase batch
    pub fn batch_count(&self, phase: RenderPhase) -> usize {
        self.batches.batch_count(phase)
    }

    /// Get a reference to the instance data buffer
    pub fn instances(&self) -> &[GpuInstance] {
        &self.instances
    }

    /// Get a reference to the camera uniforms
    pub fn camera_uniforms(&self) -> &CameraUniforms {
        &self.camera_uniforms
    }

    /// Get the indices for a specific render phase
    pub fn batch_indices(&self, phase: RenderPhase) -> &[u32] {
        match phase {
            RenderPhase::Shapes => &self.batches.shapes,
            RenderPhase::Icons => &self.batches.icons,
            RenderPhase::Images => &self.batches.images,
            RenderPhase::Text => &self.batches.text,
        }
    }

    /// Calculate total draw calls (one per non-empty phase)
    pub fn total_draw_calls(&self) -> u32 {
        let mut count = 0;
        if !self.batches.shapes.is_empty() {
            count += 1;
        }
        if !self.batches.icons.is_empty() {
            count += 1;
        }
        if !self.batches.images.is_empty() {
            count += 1;
        }
        if !self.batches.text.is_empty() {
            count += 1;
        }
        count
    }
}

impl Default for GpuRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Renderer trait for polymorphic rendering backends
///
/// This trait defines the interface that all rendering backends must implement.
/// It allows the engine to work with different renderers (WebGPU, WebGL2, Canvas 2D)
/// without knowing the specific implementation details.
pub trait Renderer {
    /// Sync renderer data from EntityStore
    ///
    /// Called every frame before rendering to prepare instance data.
    /// Returns the number of visible entities prepared for rendering.
    fn sync_from_store(&mut self, store: &EntityStore, camera: &Camera) -> usize;

    /// Get the number of entities in a specific render phase
    fn batch_count(&self, phase: RenderPhase) -> usize;

    /// Get a reference to the instance data buffer
    fn instances(&self) -> &[GpuInstance];

    /// Get camera uniforms
    fn camera_uniforms(&self) -> &CameraUniforms;

    /// Get the indices for a render phase
    fn batch_indices(&self, phase: RenderPhase) -> &[u32];

    /// Calculate total draw calls
    fn total_draw_calls(&self) -> u32;

    /// Resize renderer
    fn resize(&mut self, width: u32, height: u32);

    /// Get the backend name
    ///
    /// Returns a static string identifier for the rendering backend.
    /// Used for logging, debugging, and telemetry.
    fn backend_name(&self) -> &'static str;

    /// Render a frame
    ///
    /// Executes the actual draw calls using the prepared instance data.
    /// Should be called after sync_from_store.
    ///
    /// Returns an error if rendering fails.
    fn render(&mut self) -> Result<(), super::RenderError>;
}

// Implement Renderer for GpuRenderer
impl Renderer for GpuRenderer {
    fn sync_from_store(&mut self, store: &EntityStore, camera: &Camera) -> usize {
        Self::sync_from_store(self, store, camera)
    }

    fn batch_count(&self, phase: RenderPhase) -> usize {
        Self::batch_count(self, phase)
    }

    fn instances(&self) -> &[GpuInstance] {
        Self::instances(self)
    }

    fn camera_uniforms(&self) -> &CameraUniforms {
        Self::camera_uniforms(self)
    }

    fn batch_indices(&self, phase: RenderPhase) -> &[u32] {
        Self::batch_indices(self, phase)
    }

    fn total_draw_calls(&self) -> u32 {
        Self::total_draw_calls(self)
    }

    fn resize(&mut self, _width: u32, _height: u32) {
        // GpuRenderer doesn't need resize in this implementation
        // In a full implementation, this would update framebuffers
    }

    fn backend_name(&self) -> &'static str {
        "WebGPU"
    }

    fn render(&mut self) -> Result<(), super::RenderError> {
        // GpuRenderer needs WebGPU context to render
        // This will be implemented in HU-RENDER-001 continuation
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_core::Vec2;

    #[test]
    fn test_gpu_instance_size() {
        // GpuInstance should be exactly 48 bytes (3 x 16-byte aligned slots)
        // This is due to 16-byte alignment padding the struct
        assert_eq!(core::mem::size_of::<GpuInstance>(), 48);
        assert_eq!(core::mem::align_of::<GpuInstance>(), 16);
    }

    #[test]
    fn test_camera_uniforms_size() {
        // CameraUniforms should be 64 bytes (one cache line)
        assert_eq!(core::mem::size_of::<CameraUniforms>(), 64);
        assert_eq!(core::mem::align_of::<CameraUniforms>(), 16);
    }

    #[test]
    fn test_renderer_creation() {
        let renderer = GpuRenderer::new();
        assert_eq!(renderer.instances().len(), 0);
        assert_eq!(renderer.total_draw_calls(), 0);
    }

    #[test]
    fn test_renderer_default() {
        let renderer = GpuRenderer::default();
        assert_eq!(renderer.instances().len(), 0);
    }

    #[test]
    fn test_draw_batches_clear() {
        let mut batches = DrawBatches::new();
        batches.shapes.push(1);
        batches.icons.push(2);
        batches.images.push(3);
        batches.text.push(4);

        assert_eq!(batches.shapes.len(), 1);
        assert_eq!(batches.icons.len(), 1);
        assert_eq!(batches.images.len(), 1);
        assert_eq!(batches.text.len(), 1);

        batches.clear();
        assert_eq!(batches.shapes.len(), 0);
        assert_eq!(batches.icons.len(), 0);
        assert_eq!(batches.images.len(), 0);
        assert_eq!(batches.text.len(), 0);
    }

    #[test]
    fn test_draw_batches_get_batch() {
        let mut batches = DrawBatches::new();

        batches.get_batch(RenderPhase::Shapes).push(1);
        batches.get_batch(RenderPhase::Icons).push(2);
        batches.get_batch(RenderPhase::Images).push(3);
        batches.get_batch(RenderPhase::Text).push(4);

        assert_eq!(batches.batch_count(RenderPhase::Shapes), 1);
        assert_eq!(batches.batch_count(RenderPhase::Icons), 1);
        assert_eq!(batches.batch_count(RenderPhase::Images), 1);
        assert_eq!(batches.batch_count(RenderPhase::Text), 1);
    }

    #[test]
    fn test_render_phase_values() {
        assert_eq!(RenderPhase::Shapes as u8, 0);
        assert_eq!(RenderPhase::Icons as u8, 1);
        assert_eq!(RenderPhase::Images as u8, 2);
        assert_eq!(RenderPhase::Text as u8, 3);
    }

    #[test]
    fn test_gpu_instance_bytemuck_compatible() {
        // GpuInstance must be Pod for safe bytemuck casting
        let instance = GpuInstance {
            pos: [1.0, 2.0],
            size: [10.0, 20.0],
            color: 0xFF0000FF,
            shape_type_or_texture_index: 1,
            _padding: [0, 0],
            uv_rect: [0.0, 0.0, 1.0, 1.0],
        };

        // Verify we can cast bytes to GpuInstance (Pod guarantee)
        let bytes: [u8; 48] = unsafe { core::mem::transmute(instance) };
        assert_eq!(bytes.len(), 48);

        // And back again
        let _restored: GpuInstance = unsafe { core::mem::transmute(bytes) };
    }

    #[test]
    fn test_camera_uniforms_from_camera() {
        use archflow_core::Vec2f64;

        let mut camera = Camera::new(800.0, 600.0);
        camera.center = Vec2f64::new(100.0, 200.0);
        camera.zoom = 2.0;

        let uniforms = CameraUniforms::from_camera(&camera);

        // Check that the matrix is not identity (we set center and zoom)
        // Identity matrix would have 1s on diagonal
        let identity = [[1.0, 0.0, 0.0, 0.0]; 4];
        // At least one element should differ from identity
        let is_identity = uniforms
            .view_projection
            .iter()
            .zip(identity.iter())
            .all(|(a, b)| a.iter().zip(b.iter()).all(|(x, &y)| (x - y).abs() < 0.001));
        assert!(
            !is_identity,
            "Camera matrix should not be identity with offset center"
        );
    }

    #[test]
    fn test_sync_from_store_empty() {
        let mut renderer = GpuRenderer::new();
        let store = EntityStore::new();
        let camera = Camera::new(800.0, 600.0);

        let count = renderer.sync_from_store(&store, &camera);
        assert_eq!(count, 0);
        assert_eq!(renderer.instances().len(), 0);
        assert_eq!(renderer.total_draw_calls(), 0);
    }

    #[test]
    fn test_sync_from_store_with_entities() {
        let mut renderer = GpuRenderer::new();
        let mut store = EntityStore::new();
        let mut camera = Camera::new(800.0, 600.0);

        // Position camera to see entities around origin
        camera.center = Vec2f64::ZERO;
        camera.zoom = 1.0;

        // Spawn some entities at origin (visible to default camera)
        // At zoom 1.0, viewport is 2x2 units (from -1 to +1)
        let _id1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(0.1, 0.06));
        let _id2 = store.spawn(Vec2::new(0.5, 0.5), Vec2::new(0.08, 0.05));

        let count = renderer.sync_from_store(&store, &camera);
        assert_eq!(count, 2);
        assert_eq!(renderer.instances().len(), 2);

        // Should have 2 entities in the shapes batch (texture_index = 0)
        assert_eq!(renderer.batch_count(RenderPhase::Shapes), 2);
    }

    #[test]
    fn test_batch_indices() {
        let mut renderer = GpuRenderer::new();
        let mut store = EntityStore::new();
        let mut camera = Camera::new(800.0, 600.0);

        // Position camera to see entities around origin
        camera.center = Vec2f64::ZERO;

        // Entity at origin, small size to be visible in 2x2 viewport
        store.spawn(Vec2::new(0.0, 0.0), Vec2::new(0.1, 0.06));

        renderer.sync_from_store(&store, &camera);

        let shapes_batch = renderer.batch_indices(RenderPhase::Shapes);
        assert_eq!(shapes_batch.len(), 1);
        assert_eq!(shapes_batch[0], 0); // First instance index
    }

    #[test]
    fn test_total_draw_calls() {
        let mut renderer = GpuRenderer::new();
        let mut store = EntityStore::new();
        let mut camera = Camera::new(800.0, 600.0);

        // Position camera to see entities around origin
        camera.center = Vec2f64::ZERO;

        // Spawn entities that will go to different batches
        // Use small sizes to fit in 2x2 viewport at zoom 1.0
        let id1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(0.05, 0.05));
        let _id2 = store.spawn(Vec2::new(0.5, 0.5), Vec2::new(0.05, 0.05));

        // Set one entity to have text (goes to text batch)
        let idx1 = id1.index().0 as usize;
        store.text_glyph_count[idx1] = 5;

        renderer.sync_from_store(&store, &camera);

        // Should have 2 draw calls: shapes + text
        assert_eq!(renderer.total_draw_calls(), 2);
    }
}
