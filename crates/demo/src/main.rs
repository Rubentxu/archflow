//! ArchFlow Engine Demo v2.0 - Showcases the new API modules
//!
//! This example demonstrates:
//! - Core types (Vec2, Color, Transform)
//! - Animation system (keyframe animations with easing functions)
//! - Zoom/LOD system (C4 model levels)
//! - High-level APIs (Scene, ShapeFactory, CanvasBuilder)
//! - Event Sourcing basics

use std::iter::repeat;
use std::time::Duration;

use archflow_core::{
    AnimatedProperty,
    // Animation types
    AnimationConfig,
    AnimationHelper,
    AnimationManager,
    // API types
    CanvasBuilder,
    // Core types
    Color,
    ColorPalette,
    Command,
    DocumentAggregate,
    DomainEvent,
    EasingFunction,
    // Entity ID
    EntityId,
    EventMetadata,
    FloatAnimation,
    FloatKeyframe,
    PositionAnimation,
    PositionKeyframe,
    Scene,
    ShapeFactory,
    SnapHelper,
    Transform,
    UndoRedoStack,
    Vec2,
    // Zoom types
    ZoomLevel,
    ZoomManager,
};

fn main() {
    println!("\n🧪 ArchFlow Engine v2.0 Demo\n");
    println!("{}", repeat('=').take(50).collect::<String>());

    // 1. Core Types Demo
    demo_core_types();

    // 2. Animation System Demo
    demo_animation_system();

    // 3. Zoom/LOD System Demo
    demo_zoom_system();

    // 4. High-Level API Demo
    demo_high_level_apis();

    // 5. Event Sourcing Demo
    demo_event_sourcing();

    println!("\n{}", repeat('=').take(50).collect::<String>());
    println!("✅ All demos completed successfully!");
    println!("🎉 ArchFlow Engine v2.0 is ready to use!\n");
}

fn demo_core_types() {
    println!("\n📦 Core Types Demo");
    println!("{}", repeat('-').take(30).collect::<String>());

    // Vec2 operations - using public fields directly
    let pos1 = Vec2::new(10.0, 20.0);
    let pos2 = Vec2::new(30.0, 40.0);
    let diff = pos2 - pos1;
    let distance = diff.length();
    println!(
        "  ✓ Vec2: ({}, {}) to ({}, {}) = distance {:.2}",
        pos1.x, pos1.y, pos2.x, pos2.y, distance
    );

    // Vector math
    let sum = pos1 + pos2;
    println!(
        "  ✓ Vec2: ({}, {}) + ({}, {}) = ({}, {})",
        pos1.x, pos1.y, pos2.x, pos2.y, sum.x, sum.y
    );

    // Color operations - using public fields
    let red = Color::rgb(1.0, 0.2, 0.2);
    let blue = Color::rgb(0.2, 0.4, 1.0);
    println!(
        "  ✓ Color: red=({}, {}, {}), blue=({}, {}, {})",
        red.r, red.g, red.b, blue.r, blue.g, blue.b
    );

    // Transform operations - using Default and public fields
    let mut transform = Transform::default();
    transform.translation = Vec2::new(100.0, 200.0);
    transform.rotation = std::f32::consts::PI / 4.0;
    transform.scale = Vec2::new(1.5, 1.5);

    println!(
        "  ✓ Transform: pos=({}, {}), rot={:.2}°, scale=({}x{})",
        transform.translation.x,
        transform.translation.y,
        transform.rotation.to_degrees(),
        transform.scale.x,
        transform.scale.y
    );

    println!("  ✓ Core types: Vec2, Color, Transform working correctly");
}

fn demo_animation_system() {
    println!("\n🎬 Animation System Demo");
    println!("{}", repeat('-').take(30).collect::<String>());

    let mut manager = AnimationManager::new();

    // Create a position animation
    let target_id = EntityId::new();
    let position_anim = PositionAnimation::new(
        target_id,
        vec![
            PositionKeyframe::new(0.0, (0.0, 0.0), EasingFunction::EaseInOut),
            PositionKeyframe::new(0.5, (50.0, 50.0), EasingFunction::EaseInOut),
            PositionKeyframe::new(1.0, (100.0, 0.0), EasingFunction::EaseInOut),
        ],
    )
    .with_config(AnimationConfig {
        duration: Duration::from_millis(500),
        ..Default::default()
    });

    manager.add_position_animation(position_anim);
    println!("  ✓ Added position animation with {} keyframes", 3);

    // Create a fade animation
    let fade_target = EntityId::new();
    let fade_anim = FloatAnimation::new(
        fade_target,
        AnimatedProperty::Opacity,
        vec![
            FloatKeyframe::new(0.0, 0.0, EasingFunction::Linear),
            FloatKeyframe::new(1.0, 1.0, EasingFunction::Linear),
        ],
    )
    .with_config(AnimationConfig {
        duration: Duration::from_millis(300),
        ..Default::default()
    });

    manager.add_float_animation(fade_anim);
    println!("  ✓ Added fade animation (opacity 0→1)");

    // Simulate animation updates
    println!(
        "  ✓ Animation manager has {} active animations",
        manager.len()
    );

    // Use convenience function
    let _convenience_anim = AnimationHelper::animate_position((0.0, 0.0), (200.0, 100.0), 1000);
    println!("  ✓ Created convenience animation: 0→(200, 100) in 1s");

    // Test easing functions
    let eases: [EasingFunction; 6] = [
        EasingFunction::Linear,
        EasingFunction::EaseIn,
        EasingFunction::EaseOut,
        EasingFunction::EaseInOut,
        EasingFunction::Elastic,
        EasingFunction::Bounce,
    ];

    for easing in &eases {
        let t = easing.apply(0.5);
        println!("  ✓ Easing {:?}: t=0.5 → {:.3}", easing, t);
    }

    println!("  ✓ Animation system: {} animations active", manager.len());
}

fn demo_zoom_system() {
    println!("\n🔍 Zoom/LOD System Demo (C4 Model)");
    println!("{}", repeat('-').take(30).collect::<String>());

    let mut zoom_manager = ZoomManager::new(800.0, 600.0);

    // Test zoom levels
    let levels: [ZoomLevel; 4] = [
        ZoomLevel::System,
        ZoomLevel::Container,
        ZoomLevel::Component,
        ZoomLevel::Code,
    ];

    for level in &levels {
        println!("  ✓ Zoom Level {:?}: {}", level, level.name());
        println!("    └─ {}", level.description());
    }

    // Test scale-based level detection
    let scales = [50.0, 200.0, 700.0, 1500.0];
    for scale in &scales {
        let level = ZoomLevel::from_scale(*scale);
        println!("  ✓ Scale {:.0}px → {:?}", scale, level);
    }

    // Test zoom transitions
    zoom_manager.zoom_to_level(ZoomLevel::Component);
    println!(
        "  ✓ Zoomed to Component level (scale: {:.0})",
        zoom_manager.scale()
    );

    // Simulate transition
    zoom_manager.update(Duration::from_millis(600));
    assert_eq!(zoom_manager.zoom_level(), ZoomLevel::Component);

    // Zoom in
    zoom_manager.zoom_in();
    zoom_manager.update(Duration::from_millis(600));
    assert_eq!(zoom_manager.zoom_level(), ZoomLevel::Code);
    println!(
        "  ✓ Zoomed in to Code level (scale: {:.0})",
        zoom_manager.scale()
    );

    // Zoom out
    zoom_manager.zoom_out();
    zoom_manager.update(Duration::from_millis(600));
    assert_eq!(zoom_manager.zoom_level(), ZoomLevel::Component);

    println!("  ✓ Zoom system: {} levels available", 4);
}

fn demo_high_level_apis() {
    println!("\n🎨 High-Level APIs Demo");
    println!("{}", repeat('-').take(30).collect::<String>());

    // CanvasBuilder
    let canvas_config = CanvasBuilder::new()
        .size(1920.0, 1080.0)
        .background_color(Color::rgb(0.1, 0.1, 0.15))
        .pixel_ratio(2.0)
        .build();

    println!(
        "  ✓ CanvasBuilder: {}x{} @ {:.1}x",
        canvas_config.width, canvas_config.height, canvas_config.pixel_ratio
    );

    // ShapeFactory
    let factory = ShapeFactory::new();

    let _pos = factory.create_position(100.0, 200.0);
    let _size = factory.create_size(150.0, 80.0);
    let _bounds = factory.create_bounds(100.0, 200.0, 150.0, 80.0);
    let _centered = factory.create_centered_bounds(400.0, 300.0, 100.0, 100.0);

    println!("  ✓ ShapeFactory: created position, size, and bounds");

    // Scene
    let mut scene = Scene::new();

    // Add shapes using the fluent API
    let _rect_id = scene.add_rectangle(50.0, 50.0, 120.0, 80.0);
    let _ellipse_id = scene.add_ellipse(300.0, 150.0, 60.0, 60.0);
    let _line_id = scene.add_line(500.0, 100.0, 600.0, 200.0);

    println!("  ✓ Scene: added {} shapes", scene.len());

    // ColorPalette
    let palette = ColorPalette::default();
    println!(
        "  ✓ ColorPalette: primary={:?}, accent={:?}",
        palette.primary, palette.accent
    );

    // SnapHelper
    let snap = SnapHelper::new()
        .enable()
        .with_grid_size(10.0)
        .with_threshold(5.0);

    let original = Vec2::new(13.7, 27.3);
    let snapped = snap.snap_to_grid(original);
    println!(
        "  ✓ SnapHelper: ({:.1}, {:.1}) → ({:.1}, {:.1})",
        original.x, original.y, snapped.x, snapped.y
    );

    println!("  ✓ High-level APIs: Scene with {} shapes", scene.len());
}

fn demo_event_sourcing() {
    println!("\n📝 Event Sourcing Demo");
    println!("{}", repeat('-').take(30).collect::<String>());

    // Create a document aggregate
    let doc_id = EntityId::new();
    let mut aggregate = DocumentAggregate::new(doc_id, "demo-document".to_string());

    // Apply some events
    let primitive_id = EntityId::new();
    let created_event = DomainEvent::PrimitiveCreated {
        primitive_id,
        primitive_type: "rectangle".to_string(),
        position: (100.0, 100.0),
        size: (200.0, 150.0),
        metadata: EventMetadata::new(EntityId::new(), 1, EntityId::new(), "test".to_string()),
    };
    aggregate.apply(&created_event);

    let move_event = DomainEvent::PrimitiveMoved {
        primitive_id,
        from: (100.0, 100.0),
        to: (150.0, 200.0),
        metadata: EventMetadata::new(EntityId::new(), 2, EntityId::new(), "test".to_string()),
    };
    aggregate.apply(&move_event);

    println!(
        "  ✓ DocumentAggregate: applied {} events",
        aggregate.version
    );

    // Test commands
    let create_cmd = Command::CreatePrimitive {
        primitive_id: EntityId::new(),
        primitive_type: "circle".to_string(),
        position: (300.0, 300.0),
        size: (50.0, 50.0),
    };
    println!("  ✓ Command: CreatePrimitive for circle");

    // Event metadata
    let _metadata = EventMetadata::new(EntityId::new(), 1, EntityId::new(), "demo".to_string());
    println!("  ✓ EventMetadata: created");

    // Undo/Redo stack
    let mut stack = UndoRedoStack::new(100);
    println!("  ✓ UndoRedoStack: created (capacity: 100)");

    println!(
        "  ✓ Event Sourcing: {} events in history",
        aggregate.version
    );
}
