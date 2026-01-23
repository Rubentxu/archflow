//! Benchmarks for ECS crate

use archflow_core::geometry::Vec2;
use archflow_ecs::{Color, Position, Shape, ShapeType, Transform, spawn_shape, spawn_text};
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn criterion_config() -> Criterion {
    Criterion::default()
        .sample_size(50)
        .measurement_time(std::time::Duration::from_secs(2))
}

fn bench_spawn_entities(c: &mut Criterion) {
    c.bench_function("spawn_shape", |b| {
        b.iter(|| {
            let mut world = bevy_ecs::prelude::World::new();
            spawn_shape(
                &mut world,
                Vec2::new(100.0, 200.0),
                ShapeType::Rect,
                50.0,
                75.0,
                Color::new(1.0, 0.0, 0.0, 1.0),
            )
        })
    });

    c.bench_function("spawn_text", |b| {
        b.iter(|| {
            let mut world = bevy_ecs::prelude::World::new();
            spawn_text(&mut world, Vec2::new(50.0, 50.0), "Hello World")
        })
    });
}

fn bench_entity_queries(c: &mut Criterion) {
    let mut world = bevy_ecs::prelude::World::new();

    // Spawn entities with various components
    for i in 0..500 {
        world.spawn((
            Position::new(i as f32, i as f32),
            Transform::new(),
            Shape::rect(10.0, 10.0),
            Color::new(1.0, 0.0, 0.0, 1.0),
        ));
    }

    c.bench_function("query_position_500", |b| {
        b.iter(|| {
            let mut query = world.query::<&Position>();
            let mut count = 0;
            for _ in query.iter(&world) {
                count += 1;
            }
            count
        })
    });

    c.bench_function("query_transform_500", |b| {
        b.iter(|| {
            let mut query = world.query::<&Transform>();
            let mut count = 0;
            for _ in query.iter(&world) {
                count += 1;
            }
            count
        })
    });
}

fn bench_component_insert(c: &mut Criterion) {
    c.bench_function("insert_position", |b| {
        b.iter(|| {
            let mut world = bevy_ecs::prelude::World::new();
            for i in 0..100 {
                world.spawn((Position::new(i as f32, i as f32),));
            }
        })
    });

    c.bench_function("insert_transform", |b| {
        b.iter(|| {
            let mut world = bevy_ecs::prelude::World::new();
            for _ in 0..100 {
                world.spawn((Transform::new(),));
            }
        })
    });
}

fn bench_transform_access(c: &mut Criterion) {
    let mut world = bevy_ecs::prelude::World::new();

    // Spawn entities with Position and Transform
    for i in 0..1000 {
        world.spawn((Position::new(i as f32, i as f32), Transform::new()));
    }

    c.bench_function("transform_query_1000", |b| {
        b.iter(|| {
            let mut query = world.query::<(&Position, &mut Transform)>();
            let mut count = 0;
            for (_, _) in query.iter_mut(&mut world) {
                count += 1;
            }
            count
        })
    });
}

criterion_group!(
    name = ecs_benches;
    config = criterion_config();
    targets = bench_spawn_entities, bench_entity_queries, bench_component_insert, bench_transform_access
);

criterion_main!(ecs_benches);
