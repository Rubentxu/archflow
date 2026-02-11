// ═══════════════════════════════════════════════════════════════════════════════════════
// ECS Benchmark Suite
//
// Benchmarks for Query API and ECS performance validation.
// Run with: cargo bench --package archflow-logic
//
// Targets:
// - Query 1 componente (10k entities): < 1ms
// - Query 2 componentes (10k entities): < 2ms
// - Query iter (100k entities): < 10ms
// - add_component: < 1μs
// ═══════════════════════════════════════════════════════════════════════════════════════

use archflow_logic::ecs::{Component, Query, QueryIterExt, System, World};
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

/// Test components for benchmarking
#[derive(Clone, Debug, PartialEq)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Clone, Debug, PartialEq)]
struct Velocity {
    dx: f32,
    dy: f32,
    dz: f32,
}

#[derive(Clone, Debug, PartialEq)]
struct Health {
    hp: f32,
    max_hp: f32,
}

#[derive(Clone, Debug, PartialEq)]
struct Damage {
    amount: f32,
}

impl Component for Position {
    type Storage = archflow_logic::ecs::VecStorage<Position>;
}

impl Component for Velocity {
    type Storage = archflow_logic::ecs::VecStorage<Velocity>;
}

impl Component for Health {
    type Storage = archflow_logic::ecs::VecStorage<Health>;
}

impl Component for Damage {
    type Storage = archflow_logic::ecs::VecStorage<Damage>;
}

/// Benchmark group: Query Operations
fn bench_query_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_operations");

    // Benchmark: Single component query (10k entities)
    group.bench_with_input(
        BenchmarkId::new("single_component", 10_000),
        &10_000,
        |b, &n| {
            let mut world = World::new();
            world.register_component::<Position>();

            for i in 0..n {
                let entity = world.create_entity();
                world.add_component(
                    entity,
                    Position {
                        x: i as f32,
                        y: i as f32 * 2.0,
                        z: i as f32 * 3.0,
                    },
                );
            }

            b.iter(|| {
                let mut count = 0;
                world.query::<&Position>().each(|_pos| {
                    black_box(_pos);
                    count += 1;
                });
                black_box(count);
            });
        },
    );

    // Benchmark: Two component query (10k entities)
    group.bench_with_input(
        BenchmarkId::new("two_components", 10_000),
        &10_000,
        |b, &n| {
            let mut world = World::new();
            world.register_component::<Position>();
            world.register_component::<Velocity>();

            for i in 0..n {
                let entity = world.create_entity();
                world.add_component(
                    entity,
                    Position {
                        x: i as f32,
                        y: i as f32 * 2.0,
                        z: i as f32 * 3.0,
                    },
                );
                world.add_component(
                    entity,
                    Velocity {
                        dx: i as f32 * 0.1,
                        dy: i as f32 * 0.2,
                        dz: i as f32 * 0.3,
                    },
                );
            }

            b.iter(|| {
                let mut sum = 0.0;
                world.query::<(&Position, &Velocity)>().each(|(pos, vel)| {
                    black_box(pos);
                    black_box(vel);
                    sum += pos.x + vel.dx;
                });
                black_box(sum);
            });
        },
    );

    // Benchmark: Four component query (10k entities)
    group.bench_with_input(
        BenchmarkId::new("four_components", 10_000),
        &10_000,
        |b, &n| {
            let mut world = World::new();
            world.register_component::<Position>();
            world.register_component::<Velocity>();
            world.register_component::<Health>();
            world.register_component::<Damage>();

            for i in 0..n {
                let entity = world.create_entity();
                world.add_component(
                    entity,
                    Position {
                        x: i as f32,
                        y: i as f32 * 2.0,
                        z: i as f32 * 3.0,
                    },
                );
                world.add_component(
                    entity,
                    Velocity {
                        dx: i as f32 * 0.1,
                        dy: i as f32 * 0.2,
                        dz: i as f32 * 0.3,
                    },
                );
                world.add_component(
                    entity,
                    Health {
                        hp: 100.0,
                        max_hp: 100.0,
                    },
                );
                world.add_component(
                    entity,
                    Damage {
                        amount: i as f32 % 50.0,
                    },
                );
            }

            b.iter(|| {
                let mut sum = 0.0;
                world
                    .query::<(&Position, &Velocity, &Health, &Damage)>()
                    .each(|(_pos, _vel, _health, dmg)| {
                        black_box(_pos);
                        black_box(_vel);
                        black_box(_health);
                        black_box(dmg);
                        sum += dmg.amount;
                    });
                black_box(sum);
            });
        },
    );

    // Benchmark: Query with iterator (lazy evaluation)
    group.bench_with_input(
        BenchmarkId::new("iterator_lazy", 10_000),
        &10_000,
        |b, &n| {
            let mut world = World::new();
            world.register_component::<Position>();
            world.register_component::<Velocity>();

            for i in 0..n {
                let entity = world.create_entity();
                world.add_component(
                    entity,
                    Position {
                        x: i as f32,
                        y: i as f32 * 2.0,
                        z: i as f32 * 3.0,
                    },
                );
                world.add_component(
                    entity,
                    Velocity {
                        dx: i as f32 * 0.1,
                        dy: i as f32 * 0.2,
                        dz: i as f32 * 0.3,
                    },
                );
            }

            b.iter(|| {
                let sum: f32 = world
                    .query::<(&Position, &Velocity)>()
                    .iter()
                    .map(|(pos, vel)| pos.x + vel.dx)
                    .sum();
                black_box(sum);
            });
        },
    );

    // Benchmark: Query is_empty check
    group.bench_with_input(BenchmarkId::new("is_empty", 10_000), &10_000, |b, &n| {
        let mut world = World::new();
        world.register_component::<Position>();

        for i in 0..n {
            let entity = world.create_entity();
            world.add_component(
                entity,
                Position {
                    x: i as f32,
                    y: i as f32 * 2.0,
                    z: i as f32 * 3.0,
                },
            );
        }

        b.iter(|| {
            let empty = black_box(world.query::<&Position>().is_empty());
            black_box(empty);
        });
    });

    group.finish();
}

/// Benchmark group: Entity Operations
fn bench_entity_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_operations");

    // Benchmark: Create entity
    group.bench_function("create_entity", |b| {
        let mut world = World::new();
        world.register_component::<Position>();

        b.iter(|| {
            let entity = world.create_entity();
            black_box(entity);
        });
    });

    // Benchmark: Add component
    group.bench_function("add_component", |b| {
        let mut world = World::new();
        world.register_component::<Position>();

        let entity = world.create_entity();

        b.iter(|| {
            let result = world.add_component(
                entity,
                Position {
                    x: 1.0,
                    y: 2.0,
                    z: 3.0,
                },
            );
            black_box(result);
        });
    });

    // Benchmark: Destroy entity
    group.bench_function("destroy_entity", |b| {
        let mut world = World::new();

        let entity = world.create_entity();

        b.iter(|| {
            let result = world.destroy_entity(entity);
            black_box(result);
        });
    });

    // Benchmark: Get component
    group.bench_with_input(
        BenchmarkId::new("get_component", 1_000),
        &1_000,
        |b, &_n| {
            let mut world = World::new();
            world.register_component::<Position>();

            for i in 0..1_000 {
                let entity = world.create_entity();
                world.add_component(
                    entity,
                    Position {
                        x: i as f32,
                        y: i as f32 * 2.0,
                        z: i as f32 * 3.0,
                    },
                );
            }

            b.iter(|| {
                for i in 0..1_000 {
                    let entity = world.create_entity();
                    world.add_component(
                        entity,
                        Position {
                            x: i as f32,
                            y: i as f32 * 2.0,
                            z: i as f32 * 3.0,
                        },
                    );
                    let _pos = world.get_component::<Position>(entity);
                }
            });
        },
    );

    group.finish();
}

/// Benchmark group: Mutable Query Operations
fn bench_mutable_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("mutable_query");

    // Benchmark: Mutable query (2 components)
    group.bench_with_input(
        BenchmarkId::new("mutable_two_components", 1_000),
        &1_000,
        |b, &n| {
            let mut world = World::new();
            world.register_component::<Position>();
            world.register_component::<Velocity>();

            for i in 0..n {
                let entity = world.create_entity();
                world.add_component(
                    entity,
                    Position {
                        x: i as f32,
                        y: i as f32 * 2.0,
                        z: i as f32 * 3.0,
                    },
                );
                world.add_component(
                    entity,
                    Velocity {
                        dx: i as f32 * 0.1,
                        dy: i as f32 * 0.2,
                        dz: i as f32 * 0.3,
                    },
                );
            }

            b.iter(|| {
                let dt = black_box(0.016);
                let mut updated = 0;
                world
                    .query_mut::<(&mut Position, &Velocity)>()
                    .each(|(pos, vel)| {
                        pos.x += vel.dx * dt;
                        pos.y += vel.dy * dt;
                        pos.z += vel.dz * dt;
                        updated += 1;
                    });
                black_box(updated);
            });
        },
    );

    group.finish();
}

/// Benchmark group: System Execution
fn bench_system_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("system_execution");

    // Benchmark: System with 2-component query
    group.bench_with_input(
        BenchmarkId::new("physics_system", 10_000),
        &10_000,
        |b, &n| {
            struct PhysicsSystem;

            impl System for PhysicsSystem {
                fn run(&mut self, world: &mut World, delta_time: f32) {
                    world
                        .query_mut::<(&mut Position, &Velocity)>()
                        .each(|(pos, vel)| {
                            pos.x += vel.dx * delta_time;
                            pos.y += vel.dy * delta_time;
                            pos.z += vel.dz * delta_time;
                        });
                }

                fn name(&self) -> &str {
                    "PhysicsSystem"
                }
            }

            let mut world = World::new();
            world.register_component::<Position>();
            world.register_component::<Velocity>();

            for i in 0..n {
                let entity = world.create_entity();
                world.add_component(
                    entity,
                    Position {
                        x: i as f32,
                        y: i as f32 * 2.0,
                        z: i as f32 * 3.0,
                    },
                );
                world.add_component(
                    entity,
                    Velocity {
                        dx: 1.0,
                        dy: 2.0,
                        dz: 3.0,
                    },
                );
            }

            let mut scheduler = archflow_logic::ecs::SystemScheduler::new();
            scheduler.add_system_type(PhysicsSystem);

            b.iter(|| {
                scheduler.run(&mut world, black_box(0.016));
            });
        },
    );

    group.finish();
}

/// Benchmark group: Large Scale Queries
fn bench_large_scale(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_scale");

    // Benchmark: 100k entities with 2 components
    group.bench_with_input(
        BenchmarkId::new("two_components_100k", 100_000),
        &100_000,
        |b, &n| {
            let mut world = World::new();
            world.register_component::<Position>();
            world.register_component::<Velocity>();

            for i in 0..n {
                let entity = world.create_entity();
                world.add_component(
                    entity,
                    Position {
                        x: i as f32,
                        y: i as f32 * 2.0,
                        z: i as f32 * 3.0,
                    },
                );
                world.add_component(
                    entity,
                    Velocity {
                        dx: i as f32 * 0.1,
                        dy: i as f32 * 0.2,
                        dz: i as f32 * 0.3,
                    },
                );
            }

            b.iter(|| {
                let mut sum = 0.0;
                world.query::<(&Position, &Velocity)>().each(|(pos, vel)| {
                    sum += pos.x + vel.dx;
                });
                black_box(sum);
            });
        },
    );

    group.finish();
}

// Register all benchmark groups
criterion_group!(
    benches,
    bench_query_operations,
    bench_entity_operations,
    bench_mutable_query,
    bench_system_execution,
    bench_large_scale
);

criterion_main!(benches);
