// ═══════════════════════════════════════════════════════════════════════════════
// ShapeRenderSystem - ECS System for Rendering Shapes
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::vec::Vec;

use crate::ecs::{Component, EntityId, Transform, VecStorage, World};

use super::{ColorComponent, RenderProperties, ShapeComponent, VisibilityComponent};

// ═══════════════════════════════════════════════════════════════════════════════
// GpuShapeInstance - GPU Instance Data for Shape Rendering
// ═══════════════════════════════════════════════════════════════════════════════

/// GPU instance data for shape rendering
/// Matches the shader layout for efficient GPU upload
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct GpuShapeInstance {
    /// Position [x, y] in world coordinates
    pub pos: [f32; 2],
    /// Size [w, h] in world coordinates
    pub size: [f32; 2],
    /// Packed fill color as 0xAABBGGRR (ABGR for WebGL compatibility)
    pub color: u32,
    /// Shape type (0-15): 0=Rect, 1=Circle, 2=Ellipse, 3=Triangle, 4=Diamond, 5=Cylinder, 6=Line, 7=Arc
    pub shape_type: u32,
    /// Packed stroke color as 0xAABBGGRR
    pub stroke_color: u32,
    /// Stroke width in world units
    pub stroke_width: f32,
    /// Reserved padding
    pub _padding: [u32; 1],
}

impl GpuShapeInstance {
    /// Creates a new GPU shape instance from ECS components
    #[inline]
    #[must_use]
    pub fn from_components(
        transform: &Transform,
        shape: &ShapeComponent,
        color: &ColorComponent,
        render_props: &RenderProperties,
    ) -> Self {
        Self {
            pos: [transform.position_x, transform.position_y],
            size: [render_props.width, render_props.height],
            color: Self::color_to_abgr(color.fill),
            shape_type: shape.shape_type as u32,
            stroke_color: Self::color_to_abgr(color.stroke),
            stroke_width: color.stroke_width,
            _padding: [0],
        }
    }

    /// Convert Color to ABGR packed u32 (WebGL format)
    #[inline]
    fn color_to_abgr(color: crate::ecs::components::Color) -> u32 {
        let a = color.a as u32;
        let r = color.r as u32;
        let g = color.g as u32;
        let b = color.b as u32;
        (a << 24) | (b << 16) | (g << 8) | r
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ShapeRenderSystem
// ═══════════════════════════════════════════════════════════════════════════════

/// Statistics for shape rendering
#[derive(Clone, Debug, Default)]
pub struct ShapeRenderStats {
    /// Number of entities queried
    pub queried: usize,
    /// Number of visible entities rendered
    pub rendered: usize,
    /// Number of entities filtered (hidden)
    pub filtered: usize,
}

/// ECS System that queries shape components and prepares GPU instances
///
/// This system:
/// 1. Queries entities with ShapeComponent, ColorComponent, Transform, RenderProperties
/// 2. Gets VisibilityComponent separately and filters out hidden entities
/// 3. Sorts by layer for correct render order
/// 4. Builds GpuShapeInstance array for GPU upload
#[derive(Clone, Debug)]
pub struct ShapeRenderSystem {
    /// Internal buffer for GPU instances
    instances: Vec<GpuShapeInstance>,
}

impl ShapeRenderSystem {
    /// Creates a new ShapeRenderSystem
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            instances: Vec::with_capacity(1024),
        }
    }

    /// Returns the GPU instances prepared for rendering
    #[inline]
    #[must_use]
    pub fn instances(&self) -> &[GpuShapeInstance] {
        &self.instances
    }

    /// Clears the internal buffer
    #[inline]
    pub fn clear(&mut self) {
        self.instances.clear();
    }

    /// Reserves capacity for instances
    #[inline]
    pub fn reserve(&mut self, capacity: usize) {
        self.instances.reserve(capacity);
    }
}

impl Default for ShapeRenderSystem {
    fn default() -> Self {
        Self::new()
    }
}

// Implement System trait for ShapeRenderSystem
impl crate::ecs::System for ShapeRenderSystem {
    /// Returns the system name
    #[inline]
    fn name(&self) -> &str {
        "ShapeRenderSystem"
    }

    /// Returns the system priority (runs after physics, before final render)
    #[inline]
    fn priority(&self) -> i32 {
        100
    }

    /// Runs the shape render system
    ///
    /// Queries all entities with required components and prepares GPU instances
    fn run(&mut self, world: &mut World, _delta_time: f32) {
        // Reset buffer
        self.instances.clear();

        // Get all entity IDs that have all required components + visibility
        let entity_ids: Vec<EntityId> = world
            .entities()
            .filter(|e| {
                world.has_component::<ShapeComponent>(*e)
                    && world.has_component::<ColorComponent>(*e)
                    && world.has_component::<Transform>(*e)
                    && world.has_component::<RenderProperties>(*e)
                    && world.has_component::<VisibilityComponent>(*e)
            })
            .collect();

        // Process each entity
        let mut pending: Vec<(i32, GpuShapeInstance)> = Vec::with_capacity(entity_ids.len());

        for entity_id in entity_ids {
            // Get visibility - skip hidden entities
            if let Some(visibility) = world.get_component::<VisibilityComponent>(entity_id) {
                if !visibility.is_visible() {
                    continue;
                }
            }

            // Get all components - safe to unwrap since we checked has_component
            let shape = world.get_component::<ShapeComponent>(entity_id).unwrap();
            let color = world.get_component::<ColorComponent>(entity_id).unwrap();
            let transform = world.get_component::<Transform>(entity_id).unwrap();
            let render_props = world.get_component::<RenderProperties>(entity_id).unwrap();

            // Build GPU instance
            let instance = GpuShapeInstance::from_components(transform, shape, color, render_props);

            pending.push((render_props.layer, instance));
        }

        // Sort by layer (stable sort for same layer)
        pending.sort_by_key(|(layer, _)| *layer);

        // Extract sorted instances
        self.instances.reserve(pending.len());
        for (_, instance) in pending {
            self.instances.push(instance);
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
    use crate::ecs::components::Color;

    #[test]
    fn test_shape_render_system_creation() {
        let system = ShapeRenderSystem::new();
        assert_eq!(system.instances().len(), 0);
    }

    #[test]
    fn test_shape_render_system_name() {
        let system = ShapeRenderSystem::new();
        assert_eq!(system.name(), "ShapeRenderSystem");
    }

    #[test]
    fn test_shape_render_system_priority() {
        let system = ShapeRenderSystem::new();
        assert_eq!(system.priority(), 100);
    }
}
