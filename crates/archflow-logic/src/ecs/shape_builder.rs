// ═══════════════════════════════════════════════════════════════════════════════
// ShapeBuilder - Fluent API for Creating Entities with Shapes
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::string::String;
use archflow_core::EntityId;

use crate::ecs::{Component, EntityId as EcsEntityId, Transform, VecStorage, World};

use super::{
    Color, ColorComponent, RenderProperties, ShapeComponent, ShapeType, VisibilityComponent,
};

// ═══════════════════════════════════════════════════════════════════════════════
// ShapeBuilder
// ═══════════════════════════════════════════════════════════════════════════════

/// Fluent builder for creating entities with shape components
///
/// # Example
/// ```rust
/// let entity = ShapeBuilder::new(&mut world)
///     .circle()
///     .color(255, 0, 0)
///     .position(100.0, 200.0)
///     .size(50.0)
///     .layer(1)
///     .build();
/// ```
#[derive(Debug)]
pub struct ShapeBuilder<'a> {
    /// Reference to the ECS world
    world: &'a mut World,
    /// The created entity ID
    entity: Option<EcsEntityId>,
    /// Shape type to set
    shape: Option<ShapeType>,
    /// Radius for circle/ellipse
    radius: f32,
    /// Corner radius for rounded shapes
    corner_radius: f32,
    /// Fill color (RGB)
    fill_color: Option<(u8, u8, u8)>,
    /// Stroke color (RGB)
    stroke_color: Option<(u8, u8, u8)>,
    /// Stroke width
    stroke_width: f32,
    /// Position X
    position_x: f32,
    /// Position Y
    position_y: f32,
    /// Width
    width: f32,
    /// Height
    height: f32,
    /// Layer for rendering
    layer: i32,
    /// Visibility
    visible: bool,
}

impl<'a> ShapeBuilder<'a> {
    /// Creates a new ShapeBuilder
    #[inline]
    #[must_use]
    pub fn new(world: &'a mut World) -> Self {
        Self {
            world,
            entity: None,
            shape: Some(ShapeType::Rectangle),
            radius: 0.0,
            corner_radius: 0.0,
            fill_color: Some((204, 221, 238)), // Default light blue
            stroke_color: Some((0, 0, 0)),
            stroke_width: 1.0,
            position_x: 0.0,
            position_y: 0.0,
            width: 100.0,
            height: 100.0,
            layer: 0,
            visible: true,
        }
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // Shape Type Methods
    // ═════════════════════════════════════════════════════════════════════════════

    /// Sets shape to rectangle (default)
    #[inline]
    #[must_use]
    pub fn rectangle(mut self) -> Self {
        self.shape = Some(ShapeType::Rectangle);
        self
    }

    /// Sets shape to circle
    #[inline]
    #[must_use]
    pub fn circle(mut self) -> Self {
        self.shape = Some(ShapeType::Circle);
        self.radius = 0.5;
        self
    }

    /// Sets shape to ellipse
    #[inline]
    #[must_use]
    pub fn ellipse(mut self) -> Self {
        self.shape = Some(ShapeType::Ellipse);
        self
    }

    /// Sets shape to triangle
    #[inline]
    #[must_use]
    pub fn triangle(mut self) -> Self {
        self.shape = Some(ShapeType::Triangle);
        self
    }

    /// Sets shape to diamond
    #[inline]
    #[must_use]
    pub fn diamond(mut self) -> Self {
        self.shape = Some(ShapeType::Diamond);
        self
    }

    /// Sets shape to cylinder
    #[inline]
    #[must_use]
    pub fn cylinder(mut self) -> Self {
        self.shape = Some(ShapeType::Cylinder);
        self
    }

    /// Sets shape to line
    #[inline]
    #[must_use]
    pub fn line(mut self) -> Self {
        self.shape = Some(ShapeType::Line);
        self
    }

    /// Sets shape to arc
    #[inline]
    #[must_use]
    pub fn arc(mut self) -> Self {
        self.shape = Some(ShapeType::Arc);
        self
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // Color Methods
    // ═════════════════════════════════════════════════════════════════════════════

    /// Sets the fill color (RGB)
    #[inline]
    #[must_use]
    pub fn color(mut self, r: u8, g: u8, b: u8) -> Self {
        self.fill_color = Some((r, g, b));
        self
    }

    /// Sets the fill color using a Color
    #[inline]
    #[must_use]
    pub fn fill(mut self, color: Color) -> Self {
        self.fill_color = Some((color.r, color.g, color.b));
        self
    }

    /// Sets the stroke color (RGB)
    #[inline]
    #[must_use]
    pub fn stroke(mut self, r: u8, g: u8, b: u8) -> Self {
        self.stroke_color = Some((r, g, b));
        self
    }

    /// Sets the stroke width
    #[inline]
    #[must_use]
    pub fn stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = width;
        self
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // Transform Methods (Position)
    // ═════════════════════════════════════════════════════════════════════════════

    /// Sets the position (x, y)
    #[inline]
    #[must_use]
    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.position_x = x;
        self.position_y = y;
        self
    }

    /// Sets the X position
    #[inline]
    #[must_use]
    pub fn x(mut self, x: f32) -> Self {
        self.position_x = x;
        self
    }

    /// Sets the Y position
    #[inline]
    #[must_use]
    pub fn y(mut self, y: f32) -> Self {
        self.position_y = y;
        self
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // Size Methods (RenderProperties)
    // ═════════════════════════════════════════════════════════════════════════════

    /// Sets the size (width, height)
    #[inline]
    #[must_use]
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Sets a square size (width = height)
    #[inline]
    #[must_use]
    pub fn square_size(mut self, size: f32) -> Self {
        self.width = size;
        self.height = size;
        self
    }

    /// Sets a square size
    #[inline]
    #[must_use]
    pub fn square(mut self, size: f32) -> Self {
        self.width = size;
        self.height = size;
        self
    }

    /// Sets the width
    #[inline]
    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Sets the height
    #[inline]
    #[must_use]
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // Layer Methods
    // ═════════════════════════════════════════════════════════════════════════════

    /// Sets the render layer
    #[inline]
    #[must_use]
    pub fn layer(mut self, layer: i32) -> Self {
        self.layer = layer;
        self
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // Visibility Methods
    // ═════════════════════════════════════════════════════════════════════════════

    /// Sets visibility to hidden
    #[inline]
    #[must_use]
    pub fn hidden(mut self) -> Self {
        self.visible = false;
        self
    }

    /// Sets visibility to visible (default)
    #[inline]
    #[must_use]
    pub fn visible(mut self) -> Self {
        self.visible = true;
        self
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // Build Method
    // ═════════════════════════════════════════════════════════════════════════════

    /// Builds the entity with all configured components
    ///
    /// Returns the EntityId of the created entity
    pub fn build(&mut self) -> EcsEntityId {
        // Create entity
        let entity = self.world.create_entity();
        self.entity = Some(entity);

        // Add ShapeComponent
        let shape = ShapeComponent {
            shape_type: self.shape.unwrap_or(ShapeType::Rectangle),
            radius: self.radius,
            corner_radius: self.corner_radius,
        };
        let _ = self.world.add_component(entity, shape);

        // Add ColorComponent
        let fill = self
            .fill_color
            .map(|(r, g, b)| Color::rgb(r, g, b))
            .unwrap_or(Color::rgb(204, 221, 238));
        let stroke = self
            .stroke_color
            .map(|(r, g, b)| Color::rgb(r, g, b))
            .unwrap_or(Color::rgb(0, 0, 0));
        let mut color = ColorComponent::new(fill);
        color.stroke = stroke;
        color.stroke_width = self.stroke_width;
        let _ = self.world.add_component(entity, color);

        // Add Transform
        let mut transform = Transform::identity();
        transform.set_position(self.position_x, self.position_y);
        let _ = self.world.add_component(entity, transform);

        // Add RenderProperties
        let mut render_props = RenderProperties::new(self.width, self.height);
        render_props.layer = self.layer;
        let _ = self.world.add_component(entity, render_props);

        // Add VisibilityComponent
        let visibility = if self.visible {
            VisibilityComponent::visible()
        } else {
            VisibilityComponent::hidden()
        };
        let _ = self.world.add_component(entity, visibility);

        entity
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_builder_default() {
        let world = &mut World::new();

        let entity = ShapeBuilder::new(world).build();

        // Verify entity was created
        assert!(world.is_entity_alive(entity));

        // Verify components were added
        assert!(world.has_component::<ShapeComponent>(entity));
        assert!(world.has_component::<ColorComponent>(entity));
        assert!(world.has_component::<Transform>(entity));
        assert!(world.has_component::<RenderProperties>(entity));
        assert!(world.has_component::<VisibilityComponent>(entity));
    }

    #[test]
    fn test_shape_builder_circle() {
        let world = &mut World::new();

        let entity = ShapeBuilder::new(world).circle().build();

        let shape = world.get_component::<ShapeComponent>(entity).unwrap();
        assert!(matches!(shape.shape_type, ShapeType::Circle));
    }

    #[test]
    fn test_shape_builder_color() {
        let world = &mut World::new();

        let entity = ShapeBuilder::new(world).color(255, 128, 64).build();

        let color = world.get_component::<ColorComponent>(entity).unwrap();
        assert_eq!(color.fill.r, 255);
        assert_eq!(color.fill.g, 128);
        assert_eq!(color.fill.b, 64);
    }

    #[test]
    fn test_shape_builder_position() {
        let world = &mut World::new();

        let entity = ShapeBuilder::new(world).position(100.0, 200.0).build();

        let transform = world.get_component::<Transform>(entity).unwrap();
        assert_eq!(transform.position_x, 100.0);
        assert_eq!(transform.position_y, 200.0);
    }

    #[test]
    fn test_shape_builder_size() {
        let world = &mut World::new();

        let entity = ShapeBuilder::new(world).size(50.0, 75.0).build();

        let props = world.get_component::<RenderProperties>(entity).unwrap();
        assert_eq!(props.width, 50.0);
        assert_eq!(props.height, 75.0);
    }

    #[test]
    fn test_shape_builder_layer() {
        let world = &mut World::new();

        let entity = ShapeBuilder::new(world).layer(5).build();

        let props = world.get_component::<RenderProperties>(entity).unwrap();
        assert_eq!(props.layer, 5);
    }

    #[test]
    fn test_shape_builder_hidden() {
        let world = &mut World::new();

        let entity = ShapeBuilder::new(world).hidden().build();

        let visibility = world.get_component::<VisibilityComponent>(entity).unwrap();
        assert!(!visibility.is_visible());
    }

    #[test]
    fn test_shape_builder_fluent_chain() {
        let world = &mut World::new();

        let entity = ShapeBuilder::new(world)
            .circle()
            .color(255, 0, 0)
            .position(10.0, 20.0)
            .square(30.0)
            .layer(2)
            .hidden()
            .build();

        // Verify all components
        let shape = world.get_component::<ShapeComponent>(entity).unwrap();
        assert!(matches!(shape.shape_type, ShapeType::Circle));

        let color = world.get_component::<ColorComponent>(entity).unwrap();
        assert_eq!(color.fill.r, 255);

        let transform = world.get_component::<Transform>(entity).unwrap();
        assert_eq!(transform.position_x, 10.0);

        let props = world.get_component::<RenderProperties>(entity).unwrap();
        assert_eq!(props.width, 30.0);
        assert_eq!(props.layer, 2);

        let visibility = world.get_component::<VisibilityComponent>(entity).unwrap();
        assert!(!visibility.is_visible());
    }

    #[test]
    fn test_shape_builder_square() {
        let world = &mut World::new();

        let entity = ShapeBuilder::new(world).square(50.0).build();

        let props = world.get_component::<RenderProperties>(entity).unwrap();
        assert_eq!(props.width, 50.0);
        assert_eq!(props.height, 50.0);
    }

    #[test]
    fn test_shape_builder_all_shapes() {
        let world = &mut World::new();

        // Test all shape types
        for shape_type in [
            ShapeType::Rectangle,
            ShapeType::Circle,
            ShapeType::Ellipse,
            ShapeType::Triangle,
            ShapeType::Diamond,
            ShapeType::Cylinder,
            ShapeType::Line,
            ShapeType::Arc,
        ] {
            let entity = match shape_type {
                ShapeType::Rectangle => ShapeBuilder::new(world).rectangle().build(),
                ShapeType::Circle => ShapeBuilder::new(world).circle().build(),
                ShapeType::Ellipse => ShapeBuilder::new(world).ellipse().build(),
                ShapeType::Triangle => ShapeBuilder::new(world).triangle().build(),
                ShapeType::Diamond => ShapeBuilder::new(world).diamond().build(),
                ShapeType::Cylinder => ShapeBuilder::new(world).cylinder().build(),
                ShapeType::Line => ShapeBuilder::new(world).line().build(),
                ShapeType::Arc => ShapeBuilder::new(world).arc().build(),
            };

            let shape = world.get_component::<ShapeComponent>(entity).unwrap();
            assert_eq!(shape.shape_type, shape_type);
        }
    }
}
