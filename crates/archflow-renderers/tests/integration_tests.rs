//! Integration tests for batch rendering system
//!
//! These tests verify the end-to-end behavior of the batch rendering pipeline,
//! including multiple batches, different materials, and performance characteristics.

use archflow_renderers::{BatchRenderer2D, Bounds, InstanceRaw, MaterialId, Renderable, RgbaColor};
use glam::Vec2;

/// Test renderable for integration tests
#[derive(Clone, Debug)]
struct IntegrationTestRenderable {
    bounds: Bounds,
    color: RgbaColor,
    priority: i32,
    material_id: MaterialId,
}

impl IntegrationTestRenderable {
    fn new(bounds: Bounds, color: RgbaColor, material_id: u64) -> Self {
        Self {
            bounds,
            color,
            priority: 0,
            material_id: MaterialId(material_id),
        }
    }
}

impl Renderable for IntegrationTestRenderable {
    fn bounds(&self) -> Option<Bounds> {
        Some(self.bounds)
    }

    fn contains_point(&self, point: Vec2) -> bool {
        self.bounds.contains(point)
    }

    fn render_priority(&self) -> i32 {
        self.priority
    }

    fn material_id(&self) -> MaterialId {
        self.material_id
    }

    fn color(&self) -> RgbaColor {
        self.color
    }
}

/// Verifies deterministic batch iteration across multiple runs
#[test]
fn test_batch_deterministic_iteration() {
    let mut renderer = BatchRenderer2D::new(10_000);

    // Add renderables with different material IDs
    for i in 0..10 {
        let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
        let color = RgbaColor::new((i * 25) as u8, 0, 0, 255);
        let renderable = IntegrationTestRenderable::new(bounds, color, (10 - i) as u64);
        renderer.add(&renderable);
    }

    // Get iteration order
    let first_run: Vec<MaterialId> = renderer.iter_batches().map(|(id, _)| *id).collect();
    let second_run: Vec<MaterialId> = renderer.iter_batches().map(|(id, _)| *id).collect();

    assert_eq!(
        first_run, second_run,
        "Batch iteration must be deterministic"
    );

    // Verify sorted order
    let mut sorted = first_run.clone();
    sorted.sort();
    assert_eq!(
        first_run, sorted,
        "Batches should be in ascending material ID order"
    );
}

/// Tests batch rendering with maximum instance capacity
#[test]
fn test_batch_renderer_max_capacity() {
    let max_capacity = 100;
    let mut renderer = BatchRenderer2D::new(max_capacity);

    // Add more than max capacity
    for i in 0..(max_capacity * 2) {
        let bounds = Bounds::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        let renderable = IntegrationTestRenderable::new(bounds, RgbaColor::red(), 1);
        renderer.add(&renderable);
    }

    assert_eq!(
        renderer.instance_count(),
        max_capacity,
        "Should be capped at max instances"
    );
    assert_eq!(renderer.batch_count(), 1);
}

/// Tests material batching behavior
#[test]
fn test_material_batching() {
    let mut renderer = BatchRenderer2D::new(1000);
    let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));

    // Add 10 instances per material for 3 materials
    for material in 1..=3u64 {
        for _ in 0..10 {
            let renderable = IntegrationTestRenderable::new(bounds, RgbaColor::red(), material);
            renderer.add(&renderable);
        }
    }

    assert_eq!(renderer.instance_count(), 30);
    assert_eq!(renderer.batch_count(), 3);

    // Verify each batch has correct count
    for material in 1..=3u64 {
        let batch = renderer.get_batch(MaterialId(material));
        assert_eq!(
            batch.len(),
            10,
            "Material {} should have 10 instances",
            material
        );
    }
}

/// Tests empty renderer behavior
#[test]
fn test_empty_renderer() {
    let renderer = BatchRenderer2D::new(1000);

    assert!(renderer.is_empty());
    assert_eq!(renderer.batch_count(), 0);
    assert_eq!(renderer.instance_count(), 0);
    assert_eq!(renderer.total_instance_buffer_size(), 0);
}

/// Tests clear functionality
#[test]
fn test_clear_functionality() {
    let mut renderer = BatchRenderer2D::new(1000);
    let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));

    // Add some renderables
    for i in 0..10 {
        let renderable = IntegrationTestRenderable::new(bounds, RgbaColor::red(), i as u64);
        renderer.add(&renderable);
    }

    assert_eq!(renderer.instance_count(), 10);

    renderer.clear();

    assert!(renderer.is_empty());
    assert_eq!(renderer.batch_count(), 0);
    assert_eq!(renderer.instance_count(), 0);
}

/// Tests InstanceRaw POD properties
#[test]
fn test_instance_raw_pod_properties() {
    use std::mem;

    // Verify POD requirements
    assert!(
        mem::size_of::<InstanceRaw>() > 0,
        "InstanceRaw must have non-zero size"
    );
    assert!(
        mem::align_of::<InstanceRaw>() >= 4,
        "InstanceRaw must have at least 4-byte alignment"
    );

    // Verify repr(C)
    let size = mem::size_of::<InstanceRaw>();
    assert_eq!(
        size, 80,
        "InstanceRaw should be exactly 80 bytes (16 floats for matrix + 4 floats for color)"
    );

    // Verify can be transmuted to bytes
    let instance = InstanceRaw::from_bounds(
        Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0)),
        [1.0, 0.0, 0.0, 1.0],
    );
    let bytes: &[u8] = bytemuck::bytes_of(&instance);
    assert_eq!(bytes.len(), size);
}

/// Tests Bounds edge cases
#[test]
fn test_bounds_edge_cases() {
    // Zero-size bounds
    let zero = Bounds::new(Vec2::ZERO, Vec2::ZERO);
    assert!(!zero.is_valid());
    assert_eq!(zero.width(), 0.0);
    assert_eq!(zero.height(), 0.0);

    // Negative-size bounds (min > max)
    let negative = Bounds::new(Vec2::new(100.0, 100.0), Vec2::ZERO);
    assert!(!negative.is_valid());
    assert_eq!(negative.width(), -100.0);
    assert_eq!(negative.height(), -100.0);

    // Very large bounds
    let large = Bounds::new(Vec2::new(-1e10, -1e10), Vec2::new(1e10, 1e10));
    assert!(large.is_valid());
    assert!((large.width() - 2e10).abs() < 1.0);
}

/// Tests MaterialId type safety
#[test]
fn test_material_id_type_safety() {
    let id1 = MaterialId(1);
    let id2 = MaterialId(1);
    let id3 = MaterialId(2);

    // Equality
    assert_eq!(id1, id2);
    assert_ne!(id1, id3);

    // Ordering
    assert!(id1 < id3);
    assert!(id3 > id1);

    // Hash
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(id1);
    set.insert(id3);
    assert_eq!(set.len(), 2);

    // From/Into
    let val: u64 = id1.into();
    assert_eq!(val, 1);
    let new_id: MaterialId = 42.into();
    assert_eq!(new_id.0, 42);
}

/// Tests RgbaColor conversion
#[test]
fn test_rgba_color_conversion() {
    // Full intensity
    let white = RgbaColor::white();
    let [r, g, b, a] = white.to_f32_array();
    assert_eq!((r, g, b, a), (1.0, 1.0, 1.0, 1.0));

    // Half intensity
    let half = RgbaColor::new(127, 127, 127, 127);
    let [r, g, b, a] = half.to_f32_array();
    assert!((r - 0.498).abs() < 0.01);
    assert!((g - 0.498).abs() < 0.01);
    assert!((b - 0.498).abs() < 0.01);
    assert!((a - 0.498).abs() < 0.01);

    // Zero intensity
    let black = RgbaColor::black();
    let [r, g, b, a] = black.to_f32_array();
    assert_eq!((r, g, b, a), (0.0, 0.0, 0.0, 1.0));
}

/// Tests Renderable trait with different bounds
#[test]
fn test_renderable_with_different_bounds() {
    let bounds = Bounds::new(Vec2::new(10.0, 20.0), Vec2::new(100.0, 200.0));
    let renderable = IntegrationTestRenderable::new(bounds, RgbaColor::blue(), 42);

    // Verify bounds
    let returned_bounds = renderable.bounds().unwrap();
    assert_eq!(returned_bounds.min, Vec2::new(10.0, 20.0));
    assert_eq!(returned_bounds.max, Vec2::new(100.0, 200.0));

    // Verify contains_point
    assert!(renderable.contains_point(Vec2::new(50.0, 100.0)));
    assert!(!renderable.contains_point(Vec2::new(0.0, 0.0)));

    // Verify material_id
    assert_eq!(renderable.material_id(), MaterialId(42));

    // Verify color
    assert_eq!(renderable.color(), RgbaColor::blue());

    // Verify to_instance_data
    let instance = renderable.to_instance_data();
    assert_eq!(instance.color, [0.0, 0.0, 1.0, 1.0]);
}

/// Tests batch iteration with many materials
#[test]
fn test_many_materials_iteration() {
    let mut renderer = BatchRenderer2D::new(10_000);
    let bounds = Bounds::new(Vec2::ZERO, Vec2::new(10.0, 10.0));

    // Add 100 different materials with 10 instances each
    for material in 0..100 {
        for _ in 0..10 {
            let renderable = IntegrationTestRenderable::new(bounds, RgbaColor::red(), material);
            renderer.add(&renderable);
        }
    }

    assert_eq!(renderer.instance_count(), 1000);
    assert_eq!(renderer.batch_count(), 100);

    // Verify sorted iteration
    let ids: Vec<MaterialId> = renderer.iter_batches().map(|(id, _)| *id).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted);

    // Verify no duplicates and complete range
    for (i, id) in ids.iter().enumerate() {
        assert_eq!(id.0, i as u64);
    }
}

/// Tests that instance data is correctly computed
#[test]
fn test_instance_data_computation() {
    let bounds = Bounds::from_center_size(Vec2::new(100.0, 100.0), Vec2::new(50.0, 30.0));
    let renderable = IntegrationTestRenderable::new(bounds, RgbaColor::green(), 1);

    let instance = renderable.to_instance_data();

    // Color should match
    assert_eq!(instance.color, [0.0, 1.0, 0.0, 1.0]);

    // Model matrix should not be identity for non-default bounds
    // Verify matrix is different from identity (scaling/transform applied)
    let is_identity = instance
        .model_matrix
        .iter()
        .flatten()
        .enumerate()
        .all(|(i, &v)| if i % 5 == 0 { v == 1.0 } else { v == 0.0 });
    assert!(
        !is_identity,
        "Instance matrix should have transform applied"
    );
}
