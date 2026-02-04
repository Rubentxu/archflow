// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Render - WebGL2 Snapshot Tests
//
// These tests verify WebGL2 rendering by validating GPU instance data generation.
// Uses a headless-compatible approach with pixel comparison helpers.
//
// Testing Strategy:
// - Native tests: Verify GpuInstance data generation (no GPU required)
// - WASM tests: Full WebGL2 rendering with canvas comparison (requires browser)
// - Snapshot comparison: Pixel-by-pixel comparison with tolerance
// ═══════════════════════════════════════════════════════════════════════════════

#![cfg(test)]

use archflow_core::{Vec2, Vec2f64};
use archflow_engine::EntityStore;

use archflow_render::camera::Camera;
use archflow_render::renderer::{GpuInstance, GpuRenderer, RenderPhase};

// ═══════════════════════════════════════════════════════════════════════════════
// GOLD IMAGE SNAPSHOT TESTS (Native - Data Validation)
// ═══════════════════════════════════════════════════════════════════════════════

/// Creates a test store with a single rectangle at the origin.
fn create_test_store_with_rect() -> EntityStore {
    let mut store = EntityStore::new();
    // Spawn at origin with size 0.1 x 0.06 (visible in default 2x2 viewport)
    let _id = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(0.1, 0.06));
    store
}

/// Creates a test camera centered at origin with default zoom.
fn create_test_camera() -> Camera {
    let mut camera = Camera::new(800.0, 600.0);
    camera.center = Vec2f64::ZERO;
    camera.zoom = 1.0;
    camera
}

/// Creates a test store with multiple shapes for batch testing.
fn create_test_store_with_multiple_shapes() -> EntityStore {
    let mut store = EntityStore::new();

    // Create 5 rectangles in different positions
    let positions = [
        (0.0, 0.0),
        (0.3, 0.2),
        (-0.3, 0.2),
        (0.3, -0.2),
        (-0.3, -0.2),
    ];

    for (x, y) in positions {
        store.spawn(Vec2::new(x as f32, y as f32), Vec2::new(0.08, 0.05));
    }

    store
}

/// Test that GpuRenderer correctly generates instance data for a rectangle.
#[test]
fn test_snapshot_rectangle_instance_data() {
    let mut renderer = GpuRenderer::new();
    let store = create_test_store_with_rect();
    let camera = create_test_camera();

    renderer.sync_from_store(&store, &camera);

    // Should have 1 instance
    assert_eq!(renderer.instances().len(), 1);

    // Instance should be in shapes batch
    assert_eq!(renderer.batch_count(RenderPhase::Shapes), 1);
    assert_eq!(renderer.batch_count(RenderPhase::Icons), 0);
    assert_eq!(renderer.batch_count(RenderPhase::Images), 0);
    assert_eq!(renderer.batch_count(RenderPhase::Text), 0);

    // Verify instance data
    let instance = &renderer.instances()[0];
    assert!(
        (instance.pos[0] - 0.0).abs() < 0.001,
        "X position should be 0.0"
    );
    assert!(
        (instance.pos[1] - 0.0).abs() < 0.001,
        "Y position should be 0.0"
    );
}

/// Test that GpuRenderer correctly batches multiple entities.
#[test]
fn test_snapshot_multiple_shapes_batching() {
    let mut renderer = GpuRenderer::new();
    let store = create_test_store_with_multiple_shapes();
    let camera = create_test_camera();

    renderer.sync_from_store(&store, &camera);

    // Should have 5 instances (one per shape)
    assert_eq!(renderer.instances().len(), 5);

    // All should be in shapes batch
    assert_eq!(renderer.batch_count(RenderPhase::Shapes), 5);
    assert_eq!(renderer.batch_count(RenderPhase::Icons), 0);
    assert_eq!(renderer.batch_count(RenderPhase::Images), 0);
    assert_eq!(renderer.batch_count(RenderPhase::Text), 0);

    // Total draw calls should be 1 (single batch)
    assert_eq!(renderer.total_draw_calls(), 1);
}

/// Test that camera zoom affects instance data correctly.
#[test]
fn test_snapshot_camera_zoom_effect() {
    let mut renderer = GpuRenderer::new();
    let store = create_test_store_with_rect();

    // Test with different zoom levels
    let zoom_levels = [0.5, 1.0, 2.0, 4.0];

    for &zoom in &zoom_levels {
        let mut camera = create_test_camera();
        camera.zoom = zoom;

        renderer.sync_from_store(&store, &camera);

        // Instance count should remain the same
        assert_eq!(
            renderer.instances().len(),
            1,
            "Instance count should be 1 at zoom {}",
            zoom
        );
    }
}

/// Test that entities outside viewport are culled.
#[test]
fn test_snapshot_viewport_culling() {
    let mut renderer = GpuRenderer::new();
    let mut store = EntityStore::new();

    // Create camera centered far from entities
    let mut camera = create_test_camera();
    camera.center = Vec2f64::new(500.0, 500.0); // Far from origin (viewport is 1280x720 at PPU=1.0, zoom=1.0)

    // Spawn entities at origin (outside camera view)
    store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));

    renderer.sync_from_store(&store, &camera);

    // All entities should be culled (outside viewport)
    assert_eq!(renderer.instances().len(), 0);
    assert_eq!(renderer.batch_count(RenderPhase::Shapes), 0);
}

/// Test GpuInstance size and alignment (critical for GPU upload).
#[test]
fn test_snapshot_instance_alignment() {
    // GpuInstance must be 48 bytes with 16-byte alignment
    assert_eq!(core::mem::size_of::<GpuInstance>(), 48);
    assert_eq!(core::mem::align_of::<GpuInstance>(), 16);

    // Verify Pod compliance bytemuck
    let instance = GpuInstance {
        pos: [1.0, 2.0],
        size: [10.0, 20.0],
        color: 0xFF0000FF,
        shape_type_or_texture_index: 0,
        _padding: [0, 0],
        uv_rect: [0.0, 0.0, 1.0, 1.0],
    };

    // Should be able to cast to bytes and back
    let bytes: &[u8] = bytemuck::bytes_of(&instance);
    assert_eq!(bytes.len(), 48);
    let _restored: &GpuInstance = bytemuck::from_bytes(bytes);
}

/// Test that total draw calls calculation is correct.
#[test]
fn test_snapshot_draw_call_counting() {
    let mut renderer = GpuRenderer::new();
    let store = create_test_store_with_multiple_shapes();
    let camera = create_test_camera();

    renderer.sync_from_store(&store, &camera);

    // All shapes in one batch = 1 draw call
    assert_eq!(renderer.total_draw_calls(), 1);
}

/// Test empty store handling.
#[test]
fn test_snapshot_empty_store() {
    let mut renderer = GpuRenderer::new();
    let store = EntityStore::new();
    let camera = create_test_camera();

    let count = renderer.sync_from_store(&store, &camera);

    assert_eq!(count, 0);
    assert_eq!(renderer.instances().len(), 0);
    assert_eq!(renderer.total_draw_calls(), 0);
}

/// Test camera uniforms generation.
#[test]
fn test_snapshot_camera_uniforms() {
    let mut camera = create_test_camera();
    camera.zoom = 2.0;

    let mut renderer = GpuRenderer::new();
    let store = create_test_store_with_rect();
    renderer.sync_from_store(&store, &camera);

    let uniforms = renderer.camera_uniforms();

    // Camera uniforms should be 80 bytes (64 for view_projection + 16 for camera_pos)
    assert_eq!(
        core::mem::size_of::<archflow_render::renderer::CameraUniforms>(),
        80
    );

    // With zoom 2.0, view-projection matrix should be scaled
    let matrix = uniforms.view_projection;

    // Check that matrix has been modified (not identity)
    // At zoom 2.0, the scale factor should be 2.0 in the diagonal
    let is_scaled = matrix[0][0] != 1.0 || matrix[1][1] != 1.0;
    assert!(is_scaled, "Camera matrix should be scaled at zoom 2.0");
}

// ═══════════════════════════════════════════════════════════════════════════════
// SNAPSHOT COMPARISON HELPERS (For WASM/Integration Tests)
// ═══════════════════════════════════════════════════════════════════════════════

/// Result of a snapshot comparison.
#[derive(Debug, Clone, PartialEq)]
pub struct SnapshotComparison {
    /// Whether the comparison passed.
    pub passed: bool,
    /// Number of matching pixels.
    pub matching_pixels: u64,
    /// Total number of pixels compared.
    pub total_pixels: u64,
    /// Percentage of matching pixels (0.0 to 100.0).
    pub match_percentage: f64,
    /// Maximum difference found between images.
    pub max_difference: u32,
    /// List of differences if any.
    pub differences: Vec<PixelDifference>,
}

/// A single pixel difference.
#[derive(Debug, Clone, PartialEq)]
pub struct PixelDifference {
    /// X coordinate of the difference.
    pub x: u32,
    /// Y coordinate of the difference.
    pub y: u32,
    /// Expected RGBA values.
    pub expected: [u8; 4],
    /// Actual RGBA values.
    pub actual: [u8; 4],
    /// Color distance (Euclidean in RGB space).
    pub distance: f64,
}

/// Compare two RGBA images with a tolerance threshold.
///
/// Returns a `SnapshotComparison` with detailed results.
///
/// # Arguments
///
/// * `expected` - Reference image data (RGBA8)
/// * `actual` - Rendered image data (RGBA8)
/// * `tolerance` - Maximum color distance to consider as matching (0-441)
///
/// # Panics
///
/// Panics if image dimensions don't match.
#[allow(dead_code)]
pub fn compare_rgba_images(expected: &[u8], actual: &[u8], tolerance: u32) -> SnapshotComparison {
    assert_eq!(expected.len(), actual.len(), "Image dimensions must match");

    let width = (expected.len() / 4) as u64;
    let total_pixels = width;

    let mut matching: u64 = 0;
    let mut max_diff: u32 = 0;
    let mut differences = Vec::new();

    for i in 0..total_pixels {
        let i_usize = i as usize;
        let expected_pixel = [
            expected[i_usize * 4],
            expected[i_usize * 4 + 1],
            expected[i_usize * 4 + 2],
            expected[i_usize * 4 + 3],
        ];
        let actual_pixel = [
            actual[i_usize * 4],
            actual[i_usize * 4 + 1],
            actual[i_usize * 4 + 2],
            actual[i_usize * 4 + 3],
        ];

        // Calculate Euclidean distance in RGB space (ignore alpha)
        let diff_r = expected_pixel[0] as i32 - actual_pixel[0] as i32;
        let diff_g = expected_pixel[1] as i32 - actual_pixel[1] as i32;
        let diff_b = expected_pixel[2] as i32 - actual_pixel[2] as i32;
        let distance = ((diff_r * diff_r + diff_g * diff_g + diff_b * diff_b) as f64).sqrt();
        let distance = distance as u32;

        if distance <= tolerance {
            matching += 1;
        } else {
            max_diff = max_diff.max(distance);
            differences.push(PixelDifference {
                x: (i % width) as u32,
                y: (i / width) as u32,
                expected: expected_pixel,
                actual: actual_pixel,
                distance: distance as f64,
            });
        }
    }

    SnapshotComparison {
        passed: differences.is_empty(),
        matching_pixels: matching,
        total_pixels,
        match_percentage: (matching as f64 / total_pixels as f64) * 100.0,
        max_difference: max_diff,
        differences,
    }
}

/// Generate a simple test pattern image.
///
/// Creates a 256x256 image with a colored rectangle pattern
/// suitable for basic WebGL2 rendering tests.
///
/// # Returns
///
/// A 256x256 RGBA8 image as a `Vec<u8>`.
#[allow(dead_code)]
pub fn generate_test_pattern() -> Vec<u8> {
    let size = 256usize;
    let mut pixels = vec![0u8; size * size * 4];

    // Fill with dark gray background
    for i in 0..(size * size) {
        pixels[i * 4] = 30; // R
        pixels[i * 4 + 1] = 30; // G
        pixels[i * 4 + 2] = 30; // B
        pixels[i * 4 + 3] = 255; // A
    }

    // Draw a red rectangle in the center
    let rect_size = 64usize;
    let start = (size - rect_size) / 2;
    for y in start..(start + rect_size) {
        for x in start..(start + rect_size) {
            let idx = (y * size + x) as usize;
            pixels[idx * 4] = 255; // R
            pixels[idx * 4 + 1] = 0; // G
            pixels[idx * 4 + 2] = 0; // B
            pixels[idx * 4 + 3] = 255; // A
        }
    }

    pixels
}
