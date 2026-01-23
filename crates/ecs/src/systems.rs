//! ECS Systems for ArchFlow
//!
//! Provides systems for transform updates and spatial synchronization.

use crate::{Color, EcsSyncState, Fill, Position, Shape, ShapeType, Stroke};
use archflow_core::geometry::{Bounds, Vec2};
use archflow_core::records::Record;
use bevy_ecs::prelude::*;

/// Tag component for entities that need transform updates
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransformDirty;

/// Tag component for entities that need spatial sync
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpatialDirty;

/// Transform component with full transformation data
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    /// Position in world space
    pub position: Vec2,
    /// Rotation in radians
    pub rotation: f32,
    /// Scale factors
    pub scale: Vec2,
}

impl Transform {
    /// Creates a new transform at origin with no rotation and unit scale
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }

    /// Creates a transform with position only
    pub fn from_position(position: Vec2) -> Self {
        Self {
            position,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }

    /// Sets the position
    pub fn with_position(mut self, position: Vec2) -> Self {
        self.position = position;
        self
    }

    /// Sets the rotation
    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    /// Sets the scale
    pub fn with_scale(mut self, scale: Vec2) -> Self {
        self.scale = scale;
        self
    }

    /// Translates the transform
    pub fn translate(&mut self, delta: Vec2) {
        self.position += delta;
    }

    /// Rotates the transform
    pub fn rotate(&mut self, angle: f32) {
        self.rotation += angle;
    }

    /// Scales the transform uniformly
    pub fn scale_uniformly(&mut self, factor: f32) {
        self.scale *= factor;
    }

    /// Scales the transform by vector
    pub fn scale_by(&mut self, factor: f32) {
        self.scale *= factor;
    }

    /// Computes the transformation matrix (2D)
    pub fn to_mat3(&self) -> [[f32; 3]; 3] {
        let (sin, cos) = self.rotation.sin_cos();
        let scale_x = self.scale.x();
        let scale_y = self.scale.y();

        [
            [cos * scale_x, sin * scale_y, 0.0],
            [-sin * scale_x, cos * scale_y, 0.0],
            [self.position.x(), self.position.y(), 1.0],
        ]
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

/// Scale component for ECS entities
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Scale(pub Vec2);

impl Scale {
    /// Creates a new uniform scale
    pub fn new(scale: f32) -> Self {
        Self(Vec2::new(scale, scale))
    }

    /// Creates a new scale from components
    pub fn from_vec2(scale: Vec2) -> Self {
        Self(scale)
    }
}

impl Default for Scale {
    fn default() -> Self {
        Self(Vec2::ONE)
    }
}

/// Z-index component for layering
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZIndex(pub i32);

impl ZIndex {
    /// Creates a new z-index
    pub fn new(index: i32) -> Self {
        Self(index)
    }
}

impl Default for ZIndex {
    fn default() -> Self {
        Self(0)
    }
}

/// Text component for text entities
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Text {
    /// Text content
    pub content: String,
    /// Font size in pixels
    pub font_size: f32,
    /// Font family name
    pub font_family: String,
}

impl Text {
    /// Creates a new text component
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
            font_size: 16.0,
            font_family: "sans-serif".to_string(),
        }
    }

    /// Creates text with font size
    pub fn with_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Creates text with font family
    pub fn with_font(mut self, family: &str) -> Self {
        self.font_family = family.to_string();
        self
    }
}

impl Default for Text {
    fn default() -> Self {
        Self::new("Text")
    }
}

/// Updates Transform from Position and Shape components
///
/// This system runs every frame and updates the Transform component
/// based on Position, Shape, and Scale components.
pub fn transform_update_system(
    mut transform_query: Query<(&Position, &mut Transform, Option<&Scale>)>,
) {
    for (position, mut transform, scale) in transform_query.iter_mut() {
        // Update position
        transform.position = position.0;

        // Update scale from Scale component or derive from Shape
        if let Some(scale_comp) = scale {
            transform.scale = scale_comp.0;
        }
    }
}

/// Syncs spatial data from ECS to Store
///
/// This system runs periodically to sync entity positions and shapes
/// to the external Store for spatial queries.
pub fn spatial_sync_system<R: Record>(
    spatial_query: Query<(Entity, &Position, &Shape, Option<&ZIndex>)>,
    mut sync_state: ResMut<EcsSyncState<R>>,
) where
    R: Record + Send + Sync + 'static,
{
    for (entity, position, shape, _z_index) in spatial_query.iter() {
        // Create bounds from position and shape
        let min_x = position.x() - shape.width / 2.0;
        let min_y = position.y() - shape.height / 2.0;
        let max_x = position.x() + shape.width / 2.0;
        let max_y = position.y() + shape.height / 2.0;

        let bounds = Bounds::new(min_x, min_y, max_x - min_x, max_y - min_y);

        // Record entity in sync state
        let entity_str = format!("{:?}", entity);
        sync_state.id_to_entity.insert(entity_str, entity);
    }
}

/// Marks an entity as needing transform update
///
/// Call this when position, shape, or scale changes.
pub fn mark_transform_dirty(entity: Entity, world: &mut World) {
    if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
        entity_mut.insert(TransformDirty);
    }
}

/// Marks an entity as needing spatial sync
///
/// Call this when spatial properties change.
pub fn mark_spatial_dirty(entity: Entity, world: &mut World) {
    if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
        entity_mut.insert(SpatialDirty);
    }
}

/// Spawns an entity with all required shape components
pub fn spawn_shape(
    world: &mut World,
    position: Vec2,
    shape_type: ShapeType,
    width: f32,
    height: f32,
    color: Color,
) -> Entity {
    world
        .spawn((
            Position::new(position.x(), position.y()),
            Transform::from_position(position),
            Shape {
                shape_type,
                width,
                height,
                rotation: 0.0,
            },
            Color::new(color.r, color.g, color.b, color.a),
            Stroke::default(),
            Fill::new(color, 1.0),
        ))
        .id()
}

/// Spawns an entity with text components
pub fn spawn_text(world: &mut World, position: Vec2, content: &str) -> Entity {
    world
        .spawn((
            Position::new(position.x(), position.y()),
            Transform::from_position(position),
            Text::new(content),
            Color::default(),
        ))
        .id()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_creation() {
        let transform = Transform::new();
        assert_eq!(transform.position, Vec2::ZERO);
        assert_eq!(transform.rotation, 0.0);
        assert_eq!(transform.scale, Vec2::ONE);
    }

    #[test]
    fn test_transform_with_position() {
        let transform = Transform::new().with_position(Vec2::new(100.0, 200.0));
        assert_eq!(transform.position.x(), 100.0);
        assert_eq!(transform.position.y(), 200.0);
    }

    #[test]
    fn test_transform_with_rotation() {
        let transform = Transform::new().with_rotation(std::f32::consts::PI / 2.0);
        assert!((transform.rotation - std::f32::consts::PI / 2.0).abs() < 0.001);
    }

    #[test]
    fn test_transform_with_scale() {
        let transform = Transform::new().with_scale(Vec2::new(2.0, 3.0));
        assert_eq!(transform.scale.x(), 2.0);
        assert_eq!(transform.scale.y(), 3.0);
    }

    #[test]
    fn test_transform_translate() {
        let mut transform = Transform::new();
        transform.translate(Vec2::new(10.0, 20.0));
        assert_eq!(transform.position.x(), 10.0);
        assert_eq!(transform.position.y(), 20.0);
    }

    #[test]
    fn test_transform_rotate() {
        let mut transform = Transform::new();
        transform.rotate(std::f32::consts::PI);
        assert!((transform.rotation - std::f32::consts::PI).abs() < 0.001);
    }

    #[test]
    fn test_transform_scale_by() {
        let mut transform = Transform::new();
        transform.scale_by(2.0);
        assert_eq!(transform.scale, Vec2::new(2.0, 2.0));
    }

    #[test]
    fn test_transform_to_mat3() {
        let transform = Transform::new();
        let mat = transform.to_mat3();
        // Identity transform should give identity-like matrix
        assert_eq!(mat[0][0], 1.0); // cos(0) * 1
        assert_eq!(mat[1][1], 1.0); // cos(0) * 1
    }

    #[test]
    fn test_scale_creation() {
        let scale = Scale::new(2.0);
        assert_eq!(scale.0.x(), 2.0);
        assert_eq!(scale.0.y(), 2.0);
    }

    #[test]
    fn test_z_index_creation() {
        let z = ZIndex::new(5);
        assert_eq!(z.0, 5);
    }

    #[test]
    fn test_text_creation() {
        let text = Text::new("Hello");
        assert_eq!(text.content, "Hello");
        assert_eq!(text.font_size, 16.0);
        assert_eq!(text.font_family, "sans-serif");
    }

    #[test]
    fn test_text_with_size() {
        let text = Text::new("Hello").with_size(24.0);
        assert_eq!(text.font_size, 24.0);
    }

    #[test]
    fn test_text_with_font() {
        let text = Text::new("Hello").with_font("serif");
        assert_eq!(text.font_family, "serif");
    }

    #[test]
    fn test_spawn_shape() {
        let mut world = World::new();
        let entity = spawn_shape(
            &mut world,
            Vec2::new(100.0, 100.0),
            ShapeType::Rect,
            50.0,
            50.0,
            Color::new(1.0, 0.0, 0.0, 1.0),
        );

        let position = world.get::<Position>(entity).unwrap();
        assert_eq!(position.x(), 100.0);
        assert_eq!(position.y(), 100.0);
    }

    #[test]
    fn test_spawn_text() {
        let mut world = World::new();
        let entity = spawn_text(&mut world, Vec2::new(100.0, 100.0), "Hello World");

        let text = world.get::<Text>(entity).unwrap();
        assert_eq!(text.content, "Hello World");
    }

    #[test]
    fn test_transform_dirty_tag() {
        let transform = TransformDirty;
        assert_eq!(transform, TransformDirty);
    }

    #[test]
    fn test_spatial_dirty_tag() {
        let spatial = SpatialDirty;
        assert_eq!(spatial, SpatialDirty);
    }
}
