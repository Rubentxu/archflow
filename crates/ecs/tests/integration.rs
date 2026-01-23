//! Integration test: ECS with Core
//!
//! Tests that ECS components work correctly with core geometry.

use archflow_core::geometry::Vec2;
use archflow_core::records::{FractionalIndex, Record, RecordId};
use archflow_ecs::{Color, Position, Shape, ShapeType, Transform, spawn_shape, spawn_text};
use std::sync::atomic::{AtomicU16, Ordering};

static COUNTER: AtomicU16 = AtomicU16::new(0);

fn make_record_id() -> RecordId {
    let suffix = COUNTER.fetch_add(1, Ordering::SeqCst);
    RecordId::new(format!("entity-id-{:04}", suffix))
}

/// A test record that can sync between Store and ECS
#[derive(Debug, Clone, PartialEq)]
struct EntityRecord {
    id: RecordId,
    index: FractionalIndex,
    type_name: String,
    position: Vec2,
    shape_type: String,
    width: f32,
    height: f32,
    color: [f32; 4],
}

impl Record for EntityRecord {
    fn id(&self) -> &RecordId {
        &self.id
    }

    fn type_name(&self) -> &str {
        &self.type_name
    }

    fn index(&self) -> &FractionalIndex {
        &self.index
    }

    fn with_index(&self, index: FractionalIndex) -> Self {
        Self {
            id: self.id.clone(),
            index,
            type_name: self.type_name.clone(),
            position: self.position,
            shape_type: self.shape_type.clone(),
            width: self.width,
            height: self.height,
            color: self.color,
        }
    }
}

#[test]
fn test_spawn_shape_creates_components() {
    let mut world = bevy_ecs::prelude::World::new();

    let entity = spawn_shape(
        &mut world,
        Vec2::new(100.0, 200.0),
        ShapeType::Rect,
        50.0,
        75.0,
        Color::new(1.0, 0.0, 0.0, 1.0),
    );

    // Verify Position component
    let position = world.get::<Position>(entity).unwrap();
    assert_eq!(position.x(), 100.0);
    assert_eq!(position.y(), 200.0);

    // Verify Shape component
    let shape = world.get::<Shape>(entity).unwrap();
    assert_eq!(shape.shape_type, ShapeType::Rect);
    assert_eq!(shape.width, 50.0);
    assert_eq!(shape.height, 75.0);

    // Verify Transform component
    let transform = world.get::<Transform>(entity).unwrap();
    assert_eq!(transform.position, Vec2::new(100.0, 200.0));

    // Verify Color component
    let color = world.get::<archflow_ecs::Color>(entity).unwrap();
    assert!((color.r - 1.0).abs() < 0.001);
    assert!((color.g).abs() < 0.001);
}

#[test]
fn test_spawn_text_creates_components() {
    let mut world = bevy_ecs::prelude::World::new();

    let entity = spawn_text(&mut world, Vec2::new(50.0, 50.0), "Hello ECS");

    // Verify Position component
    let position = world.get::<Position>(entity).unwrap();
    assert_eq!(position.x(), 50.0);
    assert_eq!(position.y(), 50.0);

    // Verify Text component
    let text = world.get::<archflow_ecs::Text>(entity).unwrap();
    assert_eq!(text.content, "Hello ECS");
    assert_eq!(text.font_size, 16.0);
}

#[test]
fn test_transform_operations() {
    let mut transform = Transform::new();

    // Test translation
    transform.translate(Vec2::new(10.0, 20.0));
    assert_eq!(transform.position.x(), 10.0);
    assert_eq!(transform.position.y(), 20.0);

    // Test rotation
    transform.rotate(std::f32::consts::PI / 2.0);
    assert!((transform.rotation - std::f32::consts::PI / 2.0).abs() < 0.001);

    // Test scale
    transform.scale_by(2.0);
    assert_eq!(transform.scale.x(), 2.0);
    assert_eq!(transform.scale.y(), 2.0);
}

#[test]
fn test_color_creation() {
    // Test RGBA creation
    let color = Color::new(0.5, 0.75, 1.0, 0.8);
    assert!((color.r - 0.5).abs() < 0.001);
    assert!((color.g - 0.75).abs() < 0.001);
    assert!((color.b - 1.0).abs() < 0.001);
    assert!((color.a - 0.8).abs() < 0.001);

    // Test hex creation
    let red = Color::hex("#FF0000");
    assert!((red.r - 1.0).abs() < 0.001);
    assert!((red.g).abs() < 0.001);
    assert!((red.b).abs() < 0.001);

    // Test hex with alpha (0x80 = 128, 128/255 ≈ 0.502)
    let transparent_blue = Color::hex("#0000FF80");
    assert!((transparent_blue.b - 1.0).abs() < 0.001);
    assert!((transparent_blue.a - 0.502).abs() < 0.01);
}

#[test]
fn test_shape_types() {
    let rect = Shape::rect(100.0, 50.0);
    assert_eq!(rect.shape_type, ShapeType::Rect);
    assert_eq!(rect.width, 100.0);
    assert_eq!(rect.height, 50.0);

    let circle = Shape::ellipse(25.0, 25.0);
    assert_eq!(circle.shape_type, ShapeType::Ellipse);
    assert_eq!(circle.width, 50.0); // radius * 2
    assert_eq!(circle.height, 50.0);

    let rotated = Shape::rect(100.0, 50.0).with_rotation(std::f32::consts::PI / 4.0);
    assert!((rotated.rotation - std::f32::consts::PI / 4.0).abs() < 0.001);
}

#[test]
fn test_record_to_ecs_sync() {
    use bevy_ecs::prelude::*;

    let mut world = World::new();

    // Create a record
    let record = EntityRecord {
        id: make_record_id(),
        index: FractionalIndex::new("a0".to_string()),
        type_name: "rect".to_string(),
        position: Vec2::new(150.0, 250.0),
        shape_type: "rect".to_string(),
        width: 80.0,
        height: 60.0,
        color: [1.0, 0.5, 0.0, 1.0],
    };

    // Spawn based on record
    let entity = spawn_shape(
        &mut world,
        record.position.clone(),
        ShapeType::Rect,
        record.width,
        record.height,
        Color::new(
            record.color[0],
            record.color[1],
            record.color[2],
            record.color[3],
        ),
    );

    // Verify sync
    let position = world.get::<Position>(entity).unwrap();
    assert_eq!(position.x(), record.position.x());
    assert_eq!(position.y(), record.position.y());

    let shape = world.get::<Shape>(entity).unwrap();
    assert_eq!(shape.width, record.width);
    assert_eq!(shape.height, record.height);
}

#[test]
fn test_vec2_operations_with_ecs() {
    // Test that Vec2 operations work with ECS Transform
    let pos = Vec2::new(100.0, 200.0);
    let delta = Vec2::new(50.0, -25.0);

    // Add vectors
    let new_pos = Vec2::new(pos.x() + delta.x(), pos.y() + delta.y());
    assert_eq!(new_pos.x(), 150.0);
    assert_eq!(new_pos.y(), 175.0);

    // Distance
    let dist = pos.distance_to(delta);
    assert!(dist > 0.0);

    // Dot product
    let dot = pos.dot(delta);
    assert!((dot - (100.0 * 50.0 + 200.0 * -25.0)).abs() < 0.001);
}
