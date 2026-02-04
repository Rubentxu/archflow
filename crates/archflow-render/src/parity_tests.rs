// ═══════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Render - Feature Parity Tests
//
// Tests to verify that rendering features work consistently across backends.
// These tests ensure no feature drift between WebGPU and WebGL2 implementations.
// ═══════════════════════════════════════════════════════════════════════════════

#![cfg(test)]

use alloc::vec;
use alloc::vec::Vec;

use archflow_core::{Vec2, Vec2f64};
use archflow_engine::EntityStore;

use crate::{
    camera::Camera,
    renderer::{GpuInstance, GpuRenderer, RenderPhase},
};

/// Test that entity synchronization produces consistent results across backends
///
/// Verifies that the sync_from_store logic produces the same visible entity count
/// and batch distribution regardless of the specific renderer implementation.
#[test]
fn test_sync_produces_consistent_batch_distribution() {
    let mut store = EntityStore::new();
    let mut camera = Camera::new(800.0, 600.0);

    // Position camera to see a known area
    camera.center = archflow_core::Vec2f64::new(0.5, 0.5);
    camera.zoom = 1.0;

    // Create entities in different phases
    // Shapes (texture_index = 0, text_glyph_count = 0)
    for i in 0..5 {
        let _ = store.spawn(Vec2::new(0.1 + i as f32 * 0.05, 0.1), Vec2::new(0.02, 0.02));
    }

    // Icons (texture_index = 1..=1000)
    for i in 0..3 {
        let idx = store.spawn(Vec2::new(0.1 + i as f32 * 0.05, 0.2), Vec2::new(0.02, 0.02));
        let entity_idx = idx.index().0 as usize;
        store.set_texture_index(entity_idx, 5); // Icon
    }

    // Images (texture_index > 1000)
    for i in 0..2 {
        let idx = store.spawn(Vec2::new(0.1 + i as f32 * 0.05, 0.3), Vec2::new(0.02, 0.02));
        let entity_idx = idx.index().0 as usize;
        store.set_texture_index(entity_idx, 2000); // Image
    }

    // Text (texture_index = 0, text_glyph_count > 0)
    for i in 0..4 {
        let idx = store.spawn(Vec2::new(0.1 + i as f32 * 0.05, 0.4), Vec2::new(0.02, 0.02));
        let entity_idx = idx.index().0 as usize;
        store.set_text_glyph_count(entity_idx, 3); // Text
    }

    // Sync with renderer
    let mut renderer = GpuRenderer::new();
    let visible_count = renderer.sync_from_store(&store, &camera);

    // Verify counts
    assert_eq!(visible_count, 14); // 5 + 3 + 2 + 4 = 14
    assert_eq!(renderer.batch_count(RenderPhase::Shapes), 5);
    assert_eq!(renderer.batch_count(RenderPhase::Icons), 3);
    assert_eq!(renderer.batch_count(RenderPhase::Images), 2);
    assert_eq!(renderer.batch_count(RenderPhase::Text), 4);
    assert_eq!(renderer.total_draw_calls(), 4);
}

/// Test viewport culling removes off-screen entities
///
/// Verifies that entities outside the camera viewport are not included
/// in the render batches.
#[test]
fn test_viewport_culling() {
    let mut store = EntityStore::new();
    let mut camera = Camera::new(800.0, 600.0);

    // Position camera at origin with zoom 1.0
    // Viewport at zoom 1.0 should cover approximately [-1, 1] in both axes
    camera.center = archflow_core::Vec2f64::ZERO;
    camera.zoom = 1.0;

    // Entity inside viewport
    let _inside = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(0.1, 0.1));

    // Entity outside viewport (far to the right)
    let _outside = store.spawn(Vec2::new(5.0, 0.0), Vec2::new(0.1, 0.1));

    // Entity outside viewport (far above)
    let _outside2 = store.spawn(Vec2::new(0.0, 5.0), Vec2::new(0.1, 0.1));

    let mut renderer = GpuRenderer::new();
    let visible_count = renderer.sync_from_store(&store, &camera);

    assert_eq!(visible_count, 1);
    assert_eq!(renderer.batch_count(RenderPhase::Shapes), 1);
}

/// Test that camera zoom affects visible area correctly
///
/// Verifies that zoom level changes the viewport bounds appropriately.
#[test]
fn test_camera_zoom_affects_visibility() {
    let mut store = EntityStore::new();
    let mut camera = Camera::new(800.0, 600.0);

    // Place entity close to origin (within viewport at zoom 10.0)
    // At zoom 10.0, viewport is ~0.27x0.2 units (very small)
    let entity_pos = Vec2::new(0.05, 0.0);
    let _ = store.spawn(entity_pos, Vec2::new(0.01, 0.01));

    // At zoom 1.0, entity should be visible
    camera.center = archflow_core::Vec2f64::ZERO;
    camera.zoom = 1.0;

    let mut renderer = GpuRenderer::new();
    let visible_zoom1 = renderer.sync_from_store(&store, &camera);

    // At zoom 10.0, entity should still be visible (position 0.05 is within viewport)
    camera.zoom = 10.0;
    let visible_zoom10 = renderer.sync_from_store(&store, &camera);

    // At zoom 50.0, entity should NOT be visible (viewport ~0.05x0.04, entity at 0.05 is at edge)
    camera.zoom = 50.0;
    let visible_zoom50 = renderer.sync_from_store(&store, &camera);

    assert_eq!(visible_zoom1, 1, "Entity should be visible at zoom 1.0");
    assert_eq!(visible_zoom10, 1, "Entity should be visible at zoom 10.0");
    assert_eq!(
        visible_zoom50, 0,
        "Entity should NOT be visible at zoom 50.0 (viewport too small)"
    );
}

/// Test that shape types are correctly mapped
///
/// Verifies that different shape types produce correct GpuInstance data.
#[test]
fn test_shape_type_mapping() {
    let mut store = EntityStore::new();

    // Spawn different shapes
    let id1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(0.1, 0.1));
    let idx1 = id1.index().0 as usize;

    // Set different shape types (0-4)
    store.set_shape_type(idx1, 0); // Rectangle

    let id2 = store.spawn(Vec2::new(0.2, 0.0), Vec2::new(0.1, 0.1));
    let idx2 = id2.index().0 as usize;
    store.set_shape_type(idx2, 1); // Circle

    let id3 = store.spawn(Vec2::new(0.4, 0.0), Vec2::new(0.1, 0.1));
    let idx3 = id3.index().0 as usize;
    store.set_shape_type(idx3, 4); // Rounded rectangle with radius

    let camera = Camera::new(800.0, 600.0);

    let mut renderer = GpuRenderer::new();
    renderer.sync_from_store(&store, &camera);

    let instances = renderer.instances();

    assert_eq!(instances.len(), 3);
    assert_eq!(instances[0].shape_type_or_texture_index, 0);
    assert_eq!(instances[1].shape_type_or_texture_index, 1);
    assert_eq!(instances[2].shape_type_or_texture_index, 4);
}

/// Test that UV rectangles are preserved during sync
///
/// Verifies that texture coordinates are correctly transferred to GpuInstance.
#[test]
fn test_uv_rect_preservation() {
    let mut store = EntityStore::new();

    let id = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(0.1, 0.1));
    let idx = id.index().0 as usize;

    // Set custom UV rectangle
    store.set_uv_rect(idx, [0.1, 0.2, 0.3, 0.4]);

    let camera = Camera::new(800.0, 600.0);

    let mut renderer = GpuRenderer::new();
    renderer.sync_from_store(&store, &camera);

    let instances = renderer.instances();

    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0].uv_rect, [0.1, 0.2, 0.3, 0.4]);
}

/// Test that colors are correctly packed
///
/// Verifies that RGBA colors are packed into u32 format.
#[test]
fn test_color_packing() {
    let mut store = EntityStore::new();

    let id = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(0.1, 0.1));
    let idx = id.index().0 as usize;

    // Set color (R=255, G=128, B=64, A=255)
    store.set_color(idx, 0xFF8040FF);

    let camera = Camera::new(800.0, 600.0);

    let mut renderer = GpuRenderer::new();
    renderer.sync_from_store(&store, &camera);

    let instances = renderer.instances();

    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0].color, 0xFF8040FF);
}

/// Test that draw order is respected
///
/// Verifies that entities are processed in draw order.
#[test]
fn test_draw_order_respected() {
    let mut store = EntityStore::new();

    // Spawn entities that will be at different indices
    let id3 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(0.1, 0.1));
    let id1 = store.spawn(Vec2::new(0.1, 0.0), Vec2::new(0.1, 0.1));
    let id2 = store.spawn(Vec2::new(0.2, 0.0), Vec2::new(0.1, 0.1));

    // Modify order
    store.draw_order = vec![id3.index().0, id1.index().0, id2.index().0];

    let camera = Camera::new(800.0, 600.0);

    let mut renderer = GpuRenderer::new();
    renderer.sync_from_store(&store, &camera);

    let instances = renderer.instances();

    // Instances should be in draw order
    assert_eq!(instances.len(), 3);

    // Verify positions match draw order
    assert_eq!(instances[0].pos, [0.0, 0.0]);
    assert_eq!(instances[1].pos, [0.1, 0.0]);
    assert_eq!(instances[2].pos, [0.2, 0.0]);
}

/// Test entity visibility toggle
///
/// Verifies that invisible entities are excluded from rendering.
#[test]
fn test_entity_visibility_toggle() {
    let mut store = EntityStore::new();

    let visible = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(0.1, 0.1));
    let invisible = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(0.1, 0.1));

    // Set one invisible
    let idx = invisible.index().0 as usize;
    store.set_visible(idx, false);

    let camera = Camera::new(800.0, 600.0);

    let mut renderer = GpuRenderer::new();
    let visible_count = renderer.sync_from_store(&store, &camera);

    assert_eq!(visible_count, 1);
    assert_eq!(renderer.instances().len(), 1);
}

/// Test batch indices are computed correctly
///
/// Verifies that batch_indices returns correct offset ranges.
#[test]
fn test_batch_indices_computation() {
    let mut store = EntityStore::new();

    // Create entities in different phases
    // 3 shapes
    for i in 0..3 {
        let _ = store.spawn(Vec2::new(i as f32 * 0.1, 0.0), Vec2::new(0.05, 0.05));
    }

    // 2 icons
    for i in 0..2 {
        let idx = store.spawn(Vec2::new(i as f32 * 0.1, 0.1), Vec2::new(0.05, 0.05));
        let entity_idx = idx.index().0 as usize;
        store.set_texture_index(entity_idx, 5);
    }

    // 4 text
    for i in 0..4 {
        let idx = store.spawn(Vec2::new(i as f32 * 0.1, 0.2), Vec2::new(0.05, 0.05));
        let entity_idx = idx.index().0 as usize;
        store.set_text_glyph_count(entity_idx, 2);
    }

    let camera = Camera::new(800.0, 600.0);

    let mut renderer = GpuRenderer::new();
    renderer.sync_from_store(&store, &camera);

    // Verify batch counts
    assert_eq!(renderer.batch_count(RenderPhase::Shapes), 3);
    assert_eq!(renderer.batch_count(RenderPhase::Icons), 2);
    assert_eq!(renderer.batch_count(RenderPhase::Images), 0);
    assert_eq!(renderer.batch_count(RenderPhase::Text), 4);

    // Verify total draw calls
    assert_eq!(renderer.total_draw_calls(), 3);
}

/// Test that instance size is 48 bytes as expected
///
/// Verifies memory layout for GPU compatibility.
#[test]
fn test_instance_size_layout() {
    // GpuInstance should be exactly 48 bytes
    assert_eq!(core::mem::size_of::<GpuInstance>(), 48);
    assert_eq!(core::mem::align_of::<GpuInstance>(), 16);
}

/// Test camera uniforms are updated correctly
///
/// Verifies that camera produces valid view-projection matrix.
#[test]
fn test_camera_uniforms_update() {
    let mut camera = Camera::new(800.0, 600.0);

    // Set non-identity camera state
    camera.center = archflow_core::Vec2f64::new(100.0, 200.0);
    camera.zoom = 2.0;

    let mut renderer = GpuRenderer::new();
    renderer.sync_from_store(&EntityStore::new(), &camera);

    let uniforms = renderer.camera_uniforms();

    // Verify matrix is not identity
    let is_identity = uniforms
        .view_projection
        .iter()
        .flatten()
        .enumerate()
        .all(|(i, &v)| {
            let expected = if i % 5 == 0 { 1.0 } else { 0.0 };
            (v - expected).abs() < 0.0001
        });

    assert!(
        !is_identity,
        "Camera matrix should not be identity when zoom/center changed"
    );
}
