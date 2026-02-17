// ═══════════════════════════════════════════════════════════════════════════════
// MaterialSystem - ECS System for Material Processing
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::vec::Vec;

use crate::ecs::{Component, EntityId, System, VisibilityComponent, World};

use super::{GpuMaterialInstance, MaterialComponent};

// ═══════════════════════════════════════════════════════════════════════════════
// MaterialStats
// ═══════════════════════════════════════════════════════════════════════════════

/// Statistics for material system processing
#[derive(Clone, Debug, Default)]
pub struct MaterialStats {
    /// Number of entities with materials queried
    pub queried: usize,
    /// Number of visible materials prepared for rendering
    pub prepared: usize,
    /// Number of materials using custom shaders
    pub custom_shaders: usize,
    /// Number of transparent materials (non-opaque blend mode)
    pub transparent: usize,
}

// ═══════════════════════════════════════════════════════════════════════════════
// MaterialSystem
// ═══════════════════════════════════════════════════════════════════════════════

/// ECS System that queries material components and prepares GPU instances
///
/// This system:
/// 1. Queries entities with MaterialComponent
/// 2. Filters out hidden entities via VisibilityComponent
/// 3. Converts MaterialComponent to GpuMaterialInstance for GPU upload
/// 4. Tracks statistics for debugging and profiling
///
/// Priority 110 = runs after TextureAtlasSystem (100), before ShapeRenderSystem (150)
#[derive(Clone, Debug)]
pub struct MaterialSystem {
    /// Internal buffer for GPU material instances
    instances: Vec<GpuMaterialInstance>,
    /// Entity IDs corresponding to each instance (for correlation)
    entity_ids: Vec<EntityId>,
    /// Statistics
    stats: MaterialStats,
}

impl MaterialSystem {
    /// Creates a new MaterialSystem with default capacity
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            instances: Vec::with_capacity(256),
            entity_ids: Vec::with_capacity(256),
            stats: MaterialStats::default(),
        }
    }

    /// Creates a new MaterialSystem with specified capacity
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            instances: Vec::with_capacity(capacity),
            entity_ids: Vec::with_capacity(capacity),
            stats: MaterialStats::default(),
        }
    }

    /// Returns the GPU material instances prepared for rendering
    #[inline]
    #[must_use]
    pub fn instances(&self) -> &[GpuMaterialInstance] {
        &self.instances
    }

    /// Returns the entity IDs corresponding to each material instance
    #[inline]
    #[must_use]
    pub fn entity_ids(&self) -> &[EntityId] {
        &self.entity_ids
    }

    /// Returns processing statistics
    #[inline]
    #[must_use]
    pub fn stats(&self) -> &MaterialStats {
        &self.stats
    }

    /// Clears the internal buffers
    #[inline]
    pub fn clear(&mut self) {
        self.instances.clear();
        self.entity_ids.clear();
        self.stats = MaterialStats::default();
    }

    /// Reserves capacity for instances
    #[inline]
    pub fn reserve(&mut self, capacity: usize) {
        self.instances.reserve(capacity);
        self.entity_ids.reserve(capacity);
    }

    /// Returns the number of prepared instances
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.instances.len()
    }

    /// Returns true if no instances are prepared
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    /// Returns the capacity of the internal instance buffer
    #[inline]
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.instances.capacity()
    }
}

impl Default for MaterialSystem {
    fn default() -> Self {
        Self::new()
    }
}

// Implement System trait for MaterialSystem
impl System for MaterialSystem {
    /// Returns the system name
    #[inline]
    fn name(&self) -> &str {
        "MaterialSystem"
    }

    /// Returns the system priority
    ///
    /// Priority 110 = runs after TextureAtlasSystem (100), before ShapeRenderSystem (150)
    /// This ensures materials are prepared before final rendering but after texture atlas
    #[inline]
    fn priority(&self) -> i32 {
        110
    }

    /// Runs the material system
    ///
    /// Queries all entities with MaterialComponent and prepares GPU instances.
    /// Filters out hidden entities via VisibilityComponent.
    fn run(&mut self, world: &mut World, _delta_time: f32) {
        // Reset buffers and stats
        self.instances.clear();
        self.entity_ids.clear();
        self.stats = MaterialStats::default();

        // Get all entity IDs that have MaterialComponent
        let entity_ids: Vec<EntityId> = world
            .entities()
            .filter(|e| world.has_component::<MaterialComponent>(*e))
            .collect();

        self.stats.queried = entity_ids.len();

        // Process each entity
        for entity_id in entity_ids {
            // Check visibility if component exists
            let is_visible = world
                .get_component::<VisibilityComponent>(entity_id)
                .map_or(true, |v| v.is_visible());

            if !is_visible {
                continue;
            }

            // Get the material component
            if let Some(material) = world.get_component::<MaterialComponent>(entity_id) {
                // Track custom shaders
                if material.shader_id != 0 {
                    self.stats.custom_shaders += 1;
                }

                // Track transparent materials
                if material.blend_mode != super::BlendMode::Opaque {
                    self.stats.transparent += 1;
                }

                // Convert to GPU instance (material is already &MaterialComponent)
                let gpu_instance = GpuMaterialInstance::from(material);
                self.instances.push(gpu_instance);
                self.entity_ids.push(entity_id);
                self.stats.prepared += 1;
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::{BlendMode, World};

    #[test]
    fn test_material_system_creation() {
        let system = MaterialSystem::new();
        assert_eq!(system.instances().len(), 0);
        assert_eq!(system.entity_ids().len(), 0);
        assert!(system.is_empty());
    }

    #[test]
    fn test_material_system_name() {
        let system = MaterialSystem::new();
        assert_eq!(system.name(), "MaterialSystem");
    }

    #[test]
    fn test_material_system_priority() {
        let system = MaterialSystem::new();
        assert_eq!(system.priority(), 110);
    }

    #[test]
    fn test_material_system_with_capacity() {
        let system = MaterialSystem::with_capacity(512);
        assert!(system.capacity() >= 512);
    }

    #[test]
    fn test_material_system_clear() {
        let mut system = MaterialSystem::new();
        system.instances.push(GpuMaterialInstance {
            color_multiply: [1.0, 1.0, 1.0, 1.0],
            emission: [0.0, 0.0, 0.0],
            _padding: 0.0,
            blend_mode: 0,
            shader_id: 0,
            _reserved: [0, 0],
        });
        system.stats.prepared = 1;

        system.clear();

        assert_eq!(system.instances().len(), 0);
        assert_eq!(system.stats().prepared, 0);
    }

    #[test]
    fn test_material_system_run_empty_world() {
        let mut system = MaterialSystem::new();
        let mut world = World::new();

        system.run(&mut world, 0.016);

        assert_eq!(system.stats().queried, 0);
        assert_eq!(system.stats().prepared, 0);
    }

    #[test]
    fn test_material_system_run_with_materials() {
        let mut system = MaterialSystem::new();
        let mut world = World::new();

        // Create entity with material
        let entity = world.create_entity();
        let material = MaterialComponent::new(
            [1.0, 0.5, 0.25, 0.75],
            [0.1, 0.2, 0.3],
            BlendMode::AlphaBlend,
        );
        world.add_component(entity, material);

        system.run(&mut world, 0.016);

        assert_eq!(system.stats().queried, 1);
        assert_eq!(system.stats().prepared, 1);
        assert_eq!(system.stats().transparent, 1);
        assert_eq!(system.instances().len(), 1);
        assert_eq!(system.entity_ids().len(), 1);
    }

    #[test]
    fn test_material_system_run_with_custom_shader() {
        let mut system = MaterialSystem::new();
        let mut world = World::new();

        let entity = world.create_entity();
        let material = MaterialComponent::default_material().with_shader(42);
        world.add_component(entity, material);

        system.run(&mut world, 0.016);

        assert_eq!(system.stats().custom_shaders, 1);
    }

    #[test]
    fn test_material_system_respects_visibility() {
        let mut system = MaterialSystem::new();
        let mut world = World::new();

        // Create visible entity
        let visible_entity = world.create_entity();
        let visible_material = MaterialComponent::default_material();
        world.add_component(visible_entity, visible_material);
        world.add_component(visible_entity, VisibilityComponent::visible());

        // Create hidden entity
        let hidden_entity = world.create_entity();
        let hidden_material = MaterialComponent::default_material();
        world.add_component(hidden_entity, hidden_material);
        world.add_component(hidden_entity, VisibilityComponent::hidden());

        system.run(&mut world, 0.016);

        assert_eq!(system.stats().queried, 2);
        assert_eq!(system.stats().prepared, 1); // Only visible entity
    }

    #[test]
    fn test_gpu_instance_from_material() {
        let material =
            MaterialComponent::new([0.5, 0.6, 0.7, 0.8], [0.1, 0.2, 0.3], BlendMode::Add);

        let gpu_instance = GpuMaterialInstance::from(&material);

        assert_eq!(gpu_instance.color_multiply, [0.5, 0.6, 0.7, 0.8]);
        assert_eq!(gpu_instance.emission, [0.1, 0.2, 0.3]);
        assert_eq!(gpu_instance.blend_mode, 2); // Add = 2
    }

    #[test]
    fn test_material_system_multiple_entities() {
        let mut system = MaterialSystem::new();
        let mut world = World::new();

        // Create multiple entities with different materials
        for i in 0..5 {
            let entity = world.create_entity();
            let blend_mode = if i % 2 == 0 {
                BlendMode::Opaque
            } else {
                BlendMode::AlphaBlend
            };
            let material =
                MaterialComponent::new([1.0, 1.0, 1.0, 1.0], [0.0, 0.0, 0.0], blend_mode);
            world.add_component(entity, material);
        }

        system.run(&mut world, 0.016);

        assert_eq!(system.stats().queried, 5);
        assert_eq!(system.stats().prepared, 5);
        assert_eq!(system.stats().transparent, 2); // 2 AlphaBlend
        assert_eq!(system.len(), 5);
    }
}
