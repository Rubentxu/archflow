//! Batch Renderer for 2D Instanced Rendering
//!
//! This module provides high-performance batch rendering using WebGPU
//! instancing with bytemuck for zero-copy buffer operations.

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use std::collections::BTreeMap;

use crate::traits::{Bounds, MaterialId, Renderable};

/// Raw instance data for GPU upload.
///
/// This struct is carefully designed to be:
/// 1. **POD (Plain Old Data)**: Can be safely transmuted to bytes
/// 2. **Aligned**: Proper alignment for GPU consumption
/// 3. **Sized**: Fixed size for predictable buffer allocation
///
/// The matrix is stored in column-major order for direct GPU upload.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug, PartialEq)]
pub struct InstanceRaw {
    /// Model matrix 4x4 (column-major order for GPU)
    pub model_matrix: [[f32; 4]; 4],
    /// Color in RGBA float [0.0, 1.0]
    pub color: [f32; 4],
}

impl InstanceRaw {
    /// Creates an InstanceRaw from a Renderable's bounds and color.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The bounds of the renderable
    /// * `color` - The RGBA color as f32 array
    ///
    /// # Returns
    ///
    /// A new InstanceRaw with the model matrix computed from bounds
    #[inline]
    pub fn from_renderable(renderable: &dyn Renderable) -> Self {
        let bounds = renderable.bounds().unwrap_or_default();
        let color = renderable.color().to_f32_array();
        Self::from_bounds(bounds, color)
    }

    /// Creates an InstanceRaw from bounds and color.
    #[inline]
    pub fn from_bounds(bounds: Bounds, color: [f32; 4]) -> Self {
        Self {
            model_matrix: Self::compute_model_matrix(bounds),
            color,
        }
    }

    /// Computes the model matrix from bounds.
    #[inline]
    fn compute_model_matrix(bounds: Bounds) -> [[f32; 4]; 4] {
        if !bounds.is_valid() {
            return Mat4::IDENTITY.to_cols_array_2d();
        }

        let center = bounds.center();
        let size = bounds.size();

        // Create transform: translate to center, scale to size
        let transform = Mat4::from_translation(Vec3::new(center.x, center.y, 0.0))
            * Mat4::from_scale(Vec3::new(size.x, size.y, 1.0));

        transform.to_cols_array_2d()
    }
}

/// Batch renderer for 2D instanced rendering.
///
/// Maintains batches of instances organized by material ID for efficient
/// GPU rendering. Uses O(C) complexity where C is the number of changed records.
pub struct BatchRenderer2D {
    /// Batches organized by material ID (BTreeMap for deterministic iteration)
    batches: BTreeMap<MaterialId, Vec<InstanceRaw>>,
    /// Maximum instances supported
    max_instances: usize,
    /// Total instance count
    total_count: usize,
}

impl BatchRenderer2D {
    /// Creates a new BatchRenderer2D with the specified capacity.
    #[inline]
    pub fn new(max_instances: usize) -> Self {
        Self {
            batches: BTreeMap::new(),
            max_instances,
            total_count: 0,
        }
    }

    /// Clears all batches and resets state.
    #[inline]
    pub fn clear(&mut self) {
        self.batches.clear();
        self.total_count = 0;
    }

    /// Adds an instance from a renderable.
    ///
    /// This method uses Feature Envy reduction: the renderable provides
    /// its own instance data via `to_instance_data()`.
    #[inline]
    pub fn add(&mut self, renderable: &dyn Renderable) {
        if self.total_count >= self.max_instances {
            return;
        }

        // Feature Envy reduction: renderable encapsulates its instance data
        let instance = renderable.to_instance_data();
        let material_id = renderable.material_id();

        self.batches
            .entry(material_id)
            .or_insert_with(Vec::new)
            .push(instance);

        self.total_count += 1;
    }

    /// Returns the total size needed for the instance buffer.
    #[inline]
    pub fn total_instance_buffer_size(&self) -> usize {
        self.total_count * std::mem::size_of::<InstanceRaw>()
    }

    /// Returns the number of batches.
    #[inline]
    pub fn batch_count(&self) -> usize {
        self.batches.len()
    }

    /// Returns the total number of instances.
    #[inline]
    pub fn instance_count(&self) -> usize {
        self.total_count
    }

    /// Returns the maximum number of instances supported.
    #[inline]
    pub fn max_instances(&self) -> usize {
        self.max_instances
    }

    /// Returns true if there are no instances to render.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.total_count == 0
    }

    /// Iterates over all batches in deterministic order.
    ///
    /// Uses BTreeMap iteration which guarantees ascending key order.
    /// This ensures consistent frame rendering across runs.
    #[inline]
    pub fn iter_batches(&self) -> impl Iterator<Item = (&MaterialId, &[InstanceRaw])> {
        self.batches.iter().map(|(k, v)| (k, v.as_slice()))
    }

    /// Returns a reference to all batches.
    #[inline]
    pub fn batches(&self) -> &BTreeMap<MaterialId, Vec<InstanceRaw>> {
        &self.batches
    }

    /// Gets instances for a specific material.
    #[inline]
    pub fn get_batch(&self, material_id: MaterialId) -> &[InstanceRaw] {
        self.batches
            .get(&material_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{Bounds, MaterialId, Renderable, RgbaColor};
    use glam::Vec2;

    /// Test helper struct implementing Renderable
    #[derive(Clone, Debug, PartialEq)]
    struct TestRenderable {
        bounds: Bounds,
        priority: i32,
        material_id: MaterialId,
        color: RgbaColor,
    }

    impl TestRenderable {
        fn new(bounds: Bounds, material_id: u64, color: RgbaColor) -> Self {
            Self {
                bounds,
                priority: 0,
                material_id: MaterialId(material_id),
                color,
            }
        }
    }

    impl Renderable for TestRenderable {
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

        fn to_instance_data(&self) -> InstanceRaw {
            InstanceRaw::from_bounds(self.bounds, self.color.to_f32_array())
        }
    }

    // === MaterialId Tests ===

    #[test]
    fn test_material_id_newtype() {
        let id1 = MaterialId(1);
        let id2 = MaterialId(1);
        let id3 = MaterialId(2);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_material_id_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(MaterialId(1));
        set.insert(MaterialId(2));
        set.insert(MaterialId(1)); // Duplicate

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_material_id_ord() {
        let ids: Vec<MaterialId> = vec![MaterialId(3), MaterialId(1), MaterialId(2)];
        let mut sorted = ids.clone();
        sorted.sort();

        assert_eq!(sorted, vec![MaterialId(1), MaterialId(2), MaterialId(3)]);
    }

    // === InstanceRaw Tests ===

    #[test]
    fn test_instance_raw_pod() {
        let instance = InstanceRaw {
            model_matrix: Mat4::IDENTITY.to_cols_array_2d(),
            color: [1.0, 0.0, 0.0, 1.0],
        };

        // Verify that InstanceRaw is POD by converting to bytes
        let bytes = bytemuck::bytes_of(&instance);
        assert_eq!(bytes.len(), std::mem::size_of::<InstanceRaw>());
    }

    #[test]
    fn test_instance_raw_zeroable() {
        let instance = InstanceRaw::zeroed();
        // All zeros should be valid
        assert_eq!(instance.model_matrix, Mat4::ZERO.to_cols_array_2d());
        assert_eq!(instance.color, [0.0; 4]);
    }

    #[test]
    fn test_instance_raw_from_bounds() {
        let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
        let color = [1.0, 0.0, 0.0, 1.0];

        let instance = InstanceRaw::from_bounds(bounds, color);

        // Check color
        assert_eq!(instance.color, color);
    }

    #[test]
    fn test_instance_raw_from_invalid_bounds() {
        let bounds = Bounds::invalid();
        let color = [1.0, 0.0, 0.0, 1.0];

        let instance = InstanceRaw::from_bounds(bounds, color);

        // Should fall back to identity matrix
        assert_eq!(instance.model_matrix, Mat4::IDENTITY.to_cols_array_2d());
    }

    #[test]
    fn test_instance_raw_from_renderable() {
        let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
        let renderable = TestRenderable::new(bounds, 1, RgbaColor::red());

        let instance = InstanceRaw::from_renderable(&renderable);

        assert_eq!(instance.color, [1.0, 0.0, 0.0, 1.0]);
    }

    // === BatchRenderer2D Tests ===

    #[test]
    fn test_batch_renderer_new() {
        let renderer = BatchRenderer2D::new(1000);
        assert!(renderer.is_empty());
        assert_eq!(renderer.max_instances(), 1000);
        assert_eq!(renderer.batch_count(), 0);
        assert_eq!(renderer.instance_count(), 0);
    }

    #[test]
    fn test_batch_renderer_clear() {
        let mut renderer = BatchRenderer2D::new(1000);
        renderer.clear();

        assert!(renderer.is_empty());
        assert_eq!(renderer.batch_count(), 0);
    }

    #[test]
    fn test_batch_renderer_add_single() {
        let mut renderer = BatchRenderer2D::new(100);
        let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
        let renderable = TestRenderable::new(bounds, 1, RgbaColor::red());

        renderer.add(&renderable);

        assert_eq!(renderer.instance_count(), 1);
        assert_eq!(renderer.batch_count(), 1);
    }

    #[test]
    fn test_batch_renderer_add_multiple_same_material() {
        let mut renderer = BatchRenderer2D::new(100);
        let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));

        for i in 0..5 {
            let renderable = TestRenderable::new(bounds, 1, RgbaColor::red());
            renderer.add(&renderable);
        }

        assert_eq!(renderer.instance_count(), 5);
        assert_eq!(renderer.batch_count(), 1); // Same material
    }

    #[test]
    fn test_batch_renderer_add_multiple_different_materials() {
        let mut renderer = BatchRenderer2D::new(100);
        let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));

        for i in 0..5 {
            let renderable = TestRenderable::new(bounds, (i + 1) as u64, RgbaColor::red());
            renderer.add(&renderable);
        }

        assert_eq!(renderer.instance_count(), 5);
        assert_eq!(renderer.batch_count(), 5); // Different materials
    }

    #[test]
    fn test_batch_renderer_max_instances() {
        let mut renderer = BatchRenderer2D::new(3);
        let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));

        for i in 0..5 {
            let renderable = TestRenderable::new(bounds, 1, RgbaColor::red());
            renderer.add(&renderable);
        }

        assert_eq!(renderer.instance_count(), 3); // Capped at max
    }

    #[test]
    fn test_batch_renderer_buffer_size() {
        let mut renderer = BatchRenderer2D::new(100);
        let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
        let renderable = TestRenderable::new(bounds, 1, RgbaColor::red());

        renderer.add(&renderable);

        let expected_size = 1 * std::mem::size_of::<InstanceRaw>();
        assert_eq!(renderer.total_instance_buffer_size(), expected_size);
    }

    #[test]
    fn test_batch_renderer_iter_batches() {
        let mut renderer = BatchRenderer2D::new(100);
        let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));

        let r1 = TestRenderable::new(bounds, 1, RgbaColor::red());
        let r2 = TestRenderable::new(bounds, 2, RgbaColor::blue());
        renderer.add(&r1);
        renderer.add(&r2);

        let batches: Vec<_> = renderer.iter_batches().collect();
        assert_eq!(batches.len(), 2);
    }

    #[test]
    fn test_batch_renderer_get_batch_missing() {
        let renderer = BatchRenderer2D::new(1000);
        let batch = renderer.get_batch(MaterialId(999));
        assert!(batch.is_empty());
    }

    #[test]
    fn test_batch_renderer_get_batch_existing() {
        let mut renderer = BatchRenderer2D::new(100);
        let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
        let renderable = TestRenderable::new(bounds, 1, RgbaColor::red());

        renderer.add(&renderable);

        let batch = renderer.get_batch(MaterialId(1));
        assert_eq!(batch.len(), 1);
    }

    #[test]
    fn test_instance_raw_size() {
        // InstanceRaw should be exactly 80 bytes:
        // - 16 floats for matrix = 64 bytes
        // - 4 floats for color = 16 bytes
        assert_eq!(std::mem::size_of::<InstanceRaw>(), 80);
    }

    #[test]
    fn test_instance_raw_alignment() {
        // bytemuck requires 4-byte alignment minimum for basic types
        // The actual alignment depends on the largest field (mat4 is 16-byte aligned)
        let align = std::mem::align_of::<InstanceRaw>();
        assert!(
            align >= 4,
            "InstanceRaw should have at least 4-byte alignment"
        );
    }
}
