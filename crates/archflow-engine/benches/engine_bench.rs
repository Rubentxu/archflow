//! Benchmarks for archflow-engine performance
//!
//! Run with: cargo bench -p archflow-engine
//!
//! These benchmarks measure core engine operations.

use archflow_core::Vec2;
use archflow_engine::EntityStore;
use criterion::{BatchSize, BenchmarkId, Criterion, black_box};
use rand::prelude::*;

/// Helper for seeded random
fn seeded_rng() -> StdRng {
    StdRng::seed_from_u64(0xDEADBEEF)
}

/// Helper for deterministic positions
fn deterministic_pos(index: usize) -> Vec2 {
    let row = (index / 100) as f32;
    let col = (index % 100) as f32;
    Vec2::new(col * 20.0, row * 20.0)
}

// ═══════════════════════════════════════════════════════════════════════════════════════════════
// Entity Store Benchmarks
// ═══════════════════════════════════════════════════════════════════════════════════════════════

fn entity_store_spawn(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_store::spawn");

    // Single spawn
    group.bench_function("single", |b| {
        b.iter(|| {
            let mut store = EntityStore::new();
            black_box(store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0)));
        });
    });

    // Batch spawn
    for &count in &[1000, 5000, 10000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter_batched(
                || EntityStore::new(),
                |mut store| {
                    for i in 0..count {
                        store.spawn(deterministic_pos(i), Vec2::new(50.0, 50.0));
                    }
                    black_box(store.alive_count())
                },
                BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

fn entity_store_despawn(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_store::despawn");

    group.bench_function("single", |b| {
        b.iter(|| {
            let mut store = EntityStore::new();
            let id = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
            black_box(store.despawn(id));
        });
    });

    group.bench_function("with_cleanup", |b| {
        b.iter(|| {
            let mut store = EntityStore::new();
            let id = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
            let cleaned = std::cell::Cell::new(0);
            store.despawn_with_cleanup(id, |_| {
                cleaned.set(cleaned.get() + 1);
            });
            cleaned.get()
        });
    });

    group.finish();
}

fn entity_store_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_store::query");

    let setup = || {
        let mut store = EntityStore::new();
        for i in 0..10_000 {
            store.spawn(deterministic_pos(i), Vec2::new(50.0, 50.0));
        }
        store
    };

    group.bench_function("alive_count", |b| {
        let store = setup();
        b.iter(|| black_box(store.alive_count()));
    });

    group.finish();
}

// ═══════════════════════════════════════════════════════════════════════════════════════════════
// Mutation Benchmarks
// ═══════════════════════════════════════════════════════════════════════════════════════════════

fn mutation_move_by(c: &mut Criterion) {
    let mut group = c.benchmark_group("mutation::move_by");

    for &count in &[1000, 10000, 100000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter_batched(
                || {
                    let mut store = EntityStore::new();
                    for i in 0..count {
                        store.spawn(deterministic_pos(i), Vec2::new(50.0, 50.0));
                    }
                    store
                },
                |mut store| {
                    for i in 0..count {
                        store.move_by(i, Vec2::new(1.0, 0.5));
                    }
                    black_box(())
                },
                BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

fn mutation_set_pos(c: &mut Criterion) {
    let mut group = c.benchmark_group("mutation::set_pos");

    for &count in &[1000, 10000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter_batched(
                || {
                    let mut store = EntityStore::new();
                    for i in 0..count {
                        store.spawn(deterministic_pos(i), Vec2::new(50.0, 50.0));
                    }
                    store
                },
                |mut store| {
                    let mut rng = seeded_rng();
                    for i in 0..count {
                        store.set_pos(
                            i,
                            Vec2::new(rng.gen_range(0.0..10000.0), rng.gen_range(0.0..10000.0)),
                        );
                    }
                    black_box(())
                },
                BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

fn mutation_set_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("mutation::set_size");

    for &count in &[1000, 10000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter_batched(
                || {
                    let mut store = EntityStore::new();
                    for i in 0..count {
                        store.spawn(deterministic_pos(i), Vec2::new(50.0, 50.0));
                    }
                    store
                },
                |mut store| {
                    for i in 0..count {
                        store.set_size(i, Vec2::new(100.0, 100.0));
                    }
                    black_box(())
                },
                BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

// ═══════════════════════════════════════════════════════════════════════════════════════════════
// Dirty Flag Benchmarks
// ═══════════════════════════════════════════════════════════════════════════════════════════════

fn dirty_flags(c: &mut Criterion) {
    let mut group = c.benchmark_group("dirty_flags");

    let setup = || {
        let mut store = EntityStore::new();
        for i in 0..10000 {
            store.spawn(deterministic_pos(i), Vec2::new(50.0, 50.0));
            store.move_by(i, Vec2::new(1.0, 0.5));
        }
        store
    };

    group.bench_function("dirty_count", |b| {
        let store = setup();
        b.iter(|| black_box(store.dirty_render_count()));
    });

    group.bench_function("take_dirty", |b| {
        b.iter_batched(
            || setup(),
            |mut store| {
                let dirty: Vec<_> = store.take_dirty_render_entities().collect();
                black_box(dirty.len())
            },
            BatchSize::LargeInput,
        );
    });

    group.bench_function("clear", |b| {
        b.iter_batched(
            || setup(),
            |mut store| {
                store.clear_dirty_flags();
                black_box(store.dirty_render_count())
            },
            BatchSize::LargeInput,
        );
    });

    group.finish();
}

// ═══════════════════════════════════════════════════════════════════════════════════════════════
// Throughput Benchmarks
// ═══════════════════════════════════════════════════════════════════════════════════════════════

fn throughput_sustained(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput::sustained");

    group.measurement_time(std::time::Duration::from_secs(5));

    for &count in &[100_000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter_batched(
                || {
                    let mut store = EntityStore::new();
                    for i in 0..count {
                        store.spawn(deterministic_pos(i), Vec2::new(50.0, 50.0));
                    }
                    store
                },
                |mut store| {
                    let delta = Vec2::new(0.1, 0.05);
                    for i in 0..count {
                        store.move_by(i, delta);
                    }
                    black_box(())
                },
                BatchSize::PerIteration,
            );
        });
    }

    group.finish();
}

fn throughput_mixed(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput::mixed");

    for &count in &[10_000, 50_000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter_batched(
                || {
                    let mut store = EntityStore::new();
                    for i in 0..count {
                        store.spawn(deterministic_pos(i), Vec2::new(50.0, 50.0));
                    }
                    store
                },
                |mut store| {
                    let delta = Vec2::new(1.0, 0.5);
                    let new_size = Vec2::new(75.0, 75.0);
                    for i in 0..count {
                        let _ = store.pos(i);
                        let _ = store.size(i);
                        store.move_by(i, delta);
                        store.set_size(i, new_size);
                    }
                    black_box(())
                },
                BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

// ═══════════════════════════════════════════════════════════════════════════════════════════════
// Criterion 0.5 requires explicit benchmark registration
// ═══════════════════════════════════════════════════════════════════════════════════════════════

criterion::criterion_group!(
    benches,
    entity_store_spawn,
    entity_store_despawn,
    entity_store_query,
    mutation_move_by,
    mutation_set_pos,
    mutation_set_size,
    dirty_flags,
    throughput_sustained,
    throughput_mixed,
);

criterion::criterion_main!(benches);
