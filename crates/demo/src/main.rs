//! ArchFlow Engine Demo - Shows how to use the core modules
//!
//! This example demonstrates the basic usage of ArchFlow Engine v2.0

use std::iter::repeat;

use archflow_core::geometry::Vec2;
use archflow_core::records::{FractionalIndex, Record, RecordId, Store};
use archflow_ecs::{Color, Position, Shape, ShapeType, Transform, spawn_shape, spawn_text};
use archflow_renderer::{FillStyle, FontManager, PathTessellator, StrokeStyle, TextRenderer};

/// A simple shape record for demonstration
#[derive(Debug, Clone, PartialEq)]
struct DemoShape {
    id: RecordId,
    index: FractionalIndex,
    name: String,
    position: Vec2,
    width: f32,
    height: f32,
    color: [f32; 4],
}

impl Record for DemoShape {
    fn id(&self) -> &RecordId {
        &self.id
    }

    fn type_name(&self) -> &str {
        "demo_shape"
    }

    fn index(&self) -> &FractionalIndex {
        &self.index
    }

    fn with_index(&self, index: FractionalIndex) -> Self {
        Self {
            id: self.id.clone(),
            index,
            name: self.name.clone(),
            position: self.position,
            width: self.width,
            height: self.height,
            color: self.color,
        }
    }
}

fn main() {
    println!("\n🧪 ArchFlow Engine v2.0 Demo\n");
    println!("{}", repeat('=').take(50).collect::<String>());

    // 1. Core Module Demo
    demo_core_module();

    // 2. ECS Module Demo
    demo_ecs_module();

    // 3. Renderer Module Demo
    demo_renderer_module();

    println!("\n{}", repeat('=').take(50).collect::<String>());
    println!("✅ All demos completed successfully!");
    println!("🎉 ArchFlow Engine is ready to use!\n");
}

fn demo_core_module() {
    println!("\n📦 Core Module Demo");
    println!("{}", repeat('-').take(30).collect::<String>());

    // Create a store with records
    let mut store = Store::new();

    // Create some demo shapes
    let shapes = vec![
        DemoShape {
            id: RecordId::new("shape-red-0001".to_string()),
            index: FractionalIndex::between(None, None),
            name: "Red Rectangle".to_string(),
            position: Vec2::new(100.0, 100.0),
            width: 150.0,
            height: 100.0,
            color: [1.0, 0.2, 0.2, 1.0],
        },
        DemoShape {
            id: RecordId::new("shape-blue-0001".to_string()),
            index: FractionalIndex::between(None, None),
            name: "Blue Circle".to_string(),
            position: Vec2::new(300.0, 200.0),
            width: 80.0,
            height: 80.0,
            color: [0.2, 0.4, 1.0, 1.0],
        },
        DemoShape {
            id: RecordId::new("shape-green-0001".to_string()),
            index: FractionalIndex::between(None, None),
            name: "Green Box".to_string(),
            position: Vec2::new(500.0, 150.0),
            width: 120.0,
            height: 120.0,
            color: [0.2, 0.8, 0.3, 1.0],
        },
    ];

    // Insert shapes into store
    for shape in &shapes {
        store.put(shape.clone());
    }

    println!("  ✓ Created store with {} shapes", store.len());

    // Test Vec2 operations
    let pos1 = Vec2::new(10.0, 20.0);
    let pos2 = Vec2::new(30.0, 40.0);
    let distance = pos1.distance_to(pos2);
    println!(
        "  ✓ Vec2: distance between ({}, {}) and ({}, {}) = {:.2}",
        pos1.x(),
        pos1.y(),
        pos2.x(),
        pos2.y(),
        distance
    );

    // Test fractional indexing
    let idx1 = FractionalIndex::between(None, None);
    let idx2 = FractionalIndex::between(Some(&idx1), None);
    println!(
        "  ✓ FractionalIndex: first='{}', second='{}'",
        idx1.as_str(),
        idx2.as_str()
    );

    // Test undo/redo
    let shape = store.get(&shapes[0].id).unwrap();
    store.put(DemoShape {
        id: shapes[0].id.clone(),
        index: shapes[0].index.clone(),
        name: "Modified Red".to_string(),
        position: Vec2::new(150.0, 150.0),
        width: shapes[0].width,
        height: shapes[0].height,
        color: shapes[0].color,
    });

    store.undo();
    let restored = store.get(&shapes[0].id).unwrap();
    println!("  ✓ Undo/Redo: name after undo = '{}'", restored.name);

    // Verify
    assert_eq!(store.len(), 3);
    assert_eq!(restored.name, "Red Rectangle");
    println!("  ✓ Core module: {} shapes in store", store.len());
}

fn demo_ecs_module() {
    println!("\n🎮 ECS Module Demo");
    println!("{}", repeat('-').take(30).collect::<String>());

    let mut world = archflow_ecs::World::new();

    // Spawn shapes using ECS
    let rect_entity = spawn_shape(
        &mut world,
        Vec2::new(100.0, 100.0),
        ShapeType::Rect,
        150.0,
        100.0,
        Color::new(1.0, 0.3, 0.3, 1.0),
    );

    let _ellipse_entity = spawn_shape(
        &mut world,
        Vec2::new(300.0, 200.0),
        ShapeType::Ellipse,
        80.0,
        80.0,
        Color::new(0.3, 0.5, 1.0, 1.0),
    );

    let _text_entity = spawn_text(&mut world, Vec2::new(500.0, 100.0), "Hello ECS!");

    println!("  ✓ Spawned {} entities", 3);

    // Query all positions
    let positions: Vec<(f32, f32)> = world
        .query::<&Position>()
        .iter(&world)
        .map(|p: &Position| (p.x(), p.y()))
        .collect();

    println!("  ✓ Query found {} positions", positions.len());

    // Verify entities
    let rect_pos = world.get::<Position>(rect_entity).unwrap();
    let rect_shape = world.get::<Shape>(rect_entity).unwrap();
    let rect_transform = world.get::<Transform>(rect_entity).unwrap();

    println!(
        "  ✓ Rectangle: pos=({}, {}), size=({}x{}), scale=({}x{})",
        rect_pos.x(),
        rect_pos.y(),
        rect_shape.width,
        rect_shape.height,
        rect_transform.scale.x(),
        rect_transform.scale.y()
    );

    // Test transform operations
    let mut transform = Transform::new();
    transform.translate(Vec2::new(50.0, 50.0));
    transform.rotate(std::f32::consts::PI / 4.0);
    transform.scale_by(1.5);

    println!(
        "  ✓ Transform: pos=({}, {}), rot={:.2} rad, scale=({}x{})",
        transform.position.x(),
        transform.position.y(),
        transform.rotation,
        transform.scale.x(),
        transform.scale.y()
    );

    println!("  ✓ ECS module: 3 entities created");
}

fn demo_renderer_module() {
    println!("\n🎨 Renderer Module Demo");
    println!("{}", repeat('-').take(30).collect::<String>());

    // Font Manager
    let font_manager = FontManager::new();
    let font_count = font_manager.font_db().faces().count();
    println!("  ✓ FontManager: {} system fonts loaded", font_count);

    // Text Renderer
    let mut text_renderer = TextRenderer::new();

    // Create text buffers
    let buffer1 = text_renderer.create_text_buffer("Hello, ArchFlow!");
    let buffer2 = text_renderer.create_text_buffer_with_style(
        "Styled Text",
        archflow_renderer::TextStyle {
            font_size: 24.0,
            color: [255, 100, 50, 255],
            ..archflow_renderer::TextStyle::default()
        },
    );

    println!(
        "  ✓ TextBuffer '{}': {}x{} px",
        buffer1.text(),
        buffer1.width(),
        buffer1.height()
    );
    println!(
        "  ✓ TextBuffer '{}': {}x{} px",
        buffer2.text(),
        buffer2.width(),
        buffer2.height()
    );

    // Path Tessellator
    let tessellator = PathTessellator::new();

    // Tessellate a rectangle
    let rect_mesh = tessellator.tessellate_rect(
        0.0,
        0.0,
        100.0,
        100.0,
        Some(FillStyle::new(Color::new(1.0, 0.2, 0.2, 1.0))),
        None,
    );
    println!(
        "  ✓ Rectangle tessellation: {} vertices, {} indices",
        rect_mesh.vertices.len(),
        rect_mesh.indices.len()
    );

    // Tessellate an ellipse
    let ellipse_mesh = tessellator.tessellate_ellipse(
        200.0,
        200.0,
        50.0,
        50.0,
        Some(FillStyle::new(Color::new(0.2, 0.5, 1.0, 1.0))),
        None,
    );
    println!(
        "  ✓ Ellipse tessellation: {} vertices, {} indices",
        ellipse_mesh.vertices.len(),
        ellipse_mesh.indices.len()
    );

    // Tessellate a line
    let line_mesh = tessellator.tessellate_line(
        0.0,
        0.0,
        200.0,
        50.0,
        StrokeStyle::new(3.0, Color::new(0.0, 0.0, 0.0, 1.0)),
    );
    println!(
        "  ✓ Line tessellation: {} vertices, {} indices",
        line_mesh.vertices.len(),
        line_mesh.indices.len()
    );

    // Tessellate a shape
    let shape = Shape::rect(150.0, 100.0);
    let position = Position::new(400.0, 300.0);
    let fill = FillStyle::new(Color::new(0.3, 0.8, 0.3, 1.0));

    let (fill_mesh, _) = tessellator.tessellate_shape(position, &shape, Some(fill), None);
    println!(
        "  ✓ Shape tessellation: {} vertices, {} indices",
        fill_mesh.vertices.len(),
        fill_mesh.indices.len()
    );

    println!("  ✓ Renderer module ready for GPU rendering");
}
