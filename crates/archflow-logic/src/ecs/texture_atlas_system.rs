// ═══════════════════════════════════════════════════════════════════════════════
// TextureAtlasSystem - ECS System for Sprite Rendering
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::vec::Vec;

use crate::ecs::{Component, EntityId, System, Transform, VecStorage, VisibilityComponent, World};

use super::{TextureAtlasComponent, Visibility};

// ═══════════════════════════════════════════════════════════════════════════════
// GpuSpriteInstance - GPU Instance Data for Sprite Rendering
// ═══════════════════════════════════════════════════════════════════════════════

/// GPU instance data for sprite rendering from texture atlas
/// Layout optimized for WebGPU/WebGL2
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct GpuSpriteInstance {
    /// Position [x, y] in world coordinates
    pub pos: [f32; 2],
    /// Size [w, h] in world coordinates
    pub size: [f32; 2],
    /// UV coordinates [u0, v0, u1, v1]
    pub uv: [f32; 4],
    /// Texture index into the atlas array
    pub texture_index: u32,
    /// Flip flags: bit 0 = flip_x, bit 1 = flip_y
    pub flip_flags: u32,
    /// Reserved padding
    pub _padding: [u32; 2],
}

impl GpuSpriteInstance {
    /// Create from components
    #[inline]
    #[must_use]
    pub fn from_components(
        transform: &Transform,
        atlas: &TextureAtlasComponent,
        width: f32,
        height: f32,
    ) -> Self {
        Self {
            pos: [transform.position_x, transform.position_y],
            size: [width, height],
            uv: atlas.current_uv(),
            texture_index: atlas.texture_index as u32,
            flip_flags: (atlas.flip_x as u32) | ((atlas.flip_y as u32) << 1),
            _padding: [0, 0],
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TextureAtlasSystem
// ═══════════════════════════════════════════════════════════════════════════════

/// Statistics for texture atlas rendering
#[derive(Clone, Debug, Default)]
pub struct TextureAtlasStats {
    /// Number of sprites queried
    pub queried: usize,
    /// Number of visible sprites rendered
    pub rendered: usize,
}

/// ECS System that queries texture atlas components and prepares GPU instances
///
/// This system:
/// 1. Queries entities with TextureAtlasComponent, Transform
/// 2. Filters out hidden entities via VisibilityComponent
/// 3. Builds GpuSpriteInstance array for GPU upload
#[derive(Clone, Debug)]
pub struct TextureAtlasSystem {
    /// Internal buffer for GPU instances
    instances: Vec<GpuSpriteInstance>,
    /// Statistics
    stats: TextureAtlasStats,
}

impl TextureAtlasSystem {
    /// Creates a new TextureAtlasSystem
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            instances: Vec::with_capacity(1024),
            stats: TextureAtlasStats::default(),
        }
    }

    /// Returns the GPU instances prepared for rendering
    #[inline]
    #[must_use]
    pub fn instances(&self) -> &[GpuSpriteInstance] {
        &self.instances
    }

    /// Returns rendering statistics
    #[inline]
    #[must_use]
    pub fn stats(&self) -> &TextureAtlasStats {
        &self.stats
    }

    /// Clears the internal buffer
    #[inline]
    pub fn clear(&mut self) {
        self.instances.clear();
        self.stats = TextureAtlasStats::default();
    }

    /// Reserves capacity for instances
    #[inline]
    pub fn reserve(&mut self, capacity: usize) {
        self.instances.reserve(capacity);
    }
}

impl Default for TextureAtlasSystem {
    fn default() -> Self {
        Self::new()
    }
}

// Implement System trait for TextureAtlasSystem
impl System for TextureAtlasSystem {
    /// Returns the system name
    #[inline]
    fn name(&self) -> &str {
        "TextureAtlasSystem"
    }

    /// Returns the system priority (runs after physics, before final render)
    #[inline]
    fn priority(&self) -> i32 {
        100
    }

    /// Runs the texture atlas render system
    ///
    /// Queries all entities with required components and prepares GPU instances
    fn run(&mut self, world: &mut World, _delta_time: f32) {
        // Reset buffer and stats
        self.instances.clear();
        self.stats = TextureAtlasStats::default();

        // Get all entity IDs that have TextureAtlasComponent + Transform
        let entity_ids: Vec<EntityId> = world
            .entities()
            .filter(|e| {
                world.has_component::<TextureAtlasComponent>(*e)
                    && world.has_component::<Transform>(*e)
            })
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

            // Get components
            if let (Some(atlas), Some(transform)) = (
                world.get_component::<TextureAtlasComponent>(entity_id),
                world.get_component::<Transform>(entity_id),
            ) {
                // Get size from RenderProperties if available, otherwise use defaults
                let (width, height) = world
                    .get_component::<super::RenderProperties>(entity_id)
                    .map(|p| (p.width, p.height))
                    .unwrap_or((32.0, 32.0));

                let instance = GpuSpriteInstance::from_components(transform, atlas, width, height);
                self.instances.push(instance);
                self.stats.rendered += 1;
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
    use crate::ecs::System;

    #[test]
    fn test_texture_atlas_system_creation() {
        let system = TextureAtlasSystem::new();
        assert_eq!(system.instances().len(), 0);
    }

    #[test]
    fn test_texture_atlas_system_name() {
        let system = TextureAtlasSystem::new();
        assert_eq!(system.name(), "TextureAtlasSystem");
    }

    #[test]
    fn test_texture_atlas_system_priority() {
        let system = TextureAtlasSystem::new();
        assert_eq!(system.priority(), 100);
    }

    #[test]
    fn test_gpu_sprite_instance_creation() {
        let transform = Transform::identity();
        let atlas = TextureAtlasComponent::new(0, 32, 32, 4, 4);

        let instance = GpuSpriteInstance::from_components(&transform, &atlas, 32.0, 32.0);

        assert_eq!(instance.pos, [0.0, 0.0]);
        assert_eq!(instance.size, [32.0, 32.0]);
        assert_eq!(instance.texture_index, 0);
    }
}
