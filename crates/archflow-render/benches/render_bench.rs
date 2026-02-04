// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Render - Performance Benchmarks
//
// Benchmarks for renderer sync performance with large entity counts.
// Focus on sync_from_store operations, not actual GPU rendering.
// ═══════════════════════════════════════════════════════════════════════════════

#![cfg(feature = "std")]

use archflow_core::Vec2;
use archflow_engine::EntityStore;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::time::Duration;

use archflow_render::{Camera, GpuRenderer};

/// Default benchmark configuration
fn config() -> Criterion {
    Criterion::default()
        .measurement_time(Duration::from_secs(2))
        .sample_size(50)
        .warm_up_time(Duration::from_secs(1))
}

/// Create an EntityStore with the specified number of entities
fn create_entity_store(entity_count: usize) -> EntityStore {
    let mut store = EntityStore::new();

    // Spawn entities in a grid pattern for realistic distribution
    let grid_size = (entity_count as f64).sqrt().ceil() as usize;
    let spacing = 1.0 / grid_size as f32;

    for i in 0..entity_count {
        let row = i / grid_size;
        let col = i % grid_size;
        let x = col as f32 * spacing;
        let y = row as f32 * spacing;

        // Spawn with small size to fit in viewport
        let _ = store.spawn(Vec2::new(x, y), Vec2::new(spacing * 0.9, spacing * 0.9));
    }

    store
}

/// Benchmark sync_from_store with 1,000 entities
fn bench_sync_1k(c: &mut Criterion) {
    let store = create_entity_store(1_000);
    let mut renderer = GpuRenderer::new();
    let mut camera = Camera::new(800.0, 600.0);
    camera.zoom = 1.0;

    c.bench_function("sync_1k_entities", |b| {
        b.iter(|| {
            renderer.sync_from_store(black_box(&store), black_box(&camera));
        });
    });
}

/// Benchmark sync_from_store with 10,000 entities
fn bench_sync_10k(c: &mut Criterion) {
    let store = create_entity_store(10_000);
    let mut renderer = GpuRenderer::new();
    let mut camera = Camera::new(800.0, 600.0);
    camera.zoom = 1.0;

    c.bench_function("sync_10k_entities", |b| {
        b.iter(|| {
            renderer.sync_from_store(black_box(&store), black_box(&camera));
        });
    });
}

/// Benchmark sync_from_store with 100,000 entities
fn bench_sync_100k(c: &mut Criterion) {
    let store = create_entity_store(100_000);
    let mut renderer = GpuRenderer::new();
    let mut camera = Camera::new(800.0, 600.0);
    camera.zoom = 1.0;

    c.bench_function("sync_100k_entities", |b| {
        b.iter(|| {
            renderer.sync_from_store(black_box(&store), black_box(&camera));
        });
    });
}

/// Benchmark batch creation for different phase distributions
fn bench_batch_distribution(c: &mut Criterion) {
    let mut store = EntityStore::new();

    // Create balanced distribution: 25% shapes, 25% icons, 25% images, 25% text
    let base_count = 2_500;

    for i in 0..base_count {
        let pos = Vec2::new((i % 100) as f32 * 0.01, (i / 100) as f32 * 0.01);
        let _ = store.spawn(pos, Vec2::new(0.005, 0.005));
    }

    // Add icons (texture_index = 1)
    for i in 0..base_count {
        let idx = store.spawn(Vec2::new(i as f32 * 0.01, 0.5), Vec2::new(0.005, 0.005));
        let entity_idx = idx.index().0 as usize;
        store.texture_index[entity_idx] = 1;
    }

    // Add images (texture_index = 2000)
    for i in 0..base_count {
        let idx = store.spawn(Vec2::new(i as f32 * 0.01, 1.0), Vec2::new(0.005, 0.005));
        let entity_idx = idx.index().0 as usize;
        store.texture_index[entity_idx] = 2000;
    }

    // Add text (texture_index = 0, text_glyph_count > 0)
    for i in 0..base_count {
        let idx = store.spawn(Vec2::new(i as f32 * 0.01, 1.5), Vec2::new(0.005, 0.005));
        let entity_idx = idx.index().0 as usize;
        store.text_glyph_count[entity_idx] = 3;
    }

    let mut renderer = GpuRenderer::new();
    let camera = Camera::new(800.0, 600.0);

    c.bench_function("sync_balanced_distribution", |b| {
        b.iter(|| {
            renderer.sync_from_store(black_box(&store), black_box(&camera));
        });
    });
}

/// Measure memory allocation patterns during sync
fn bench_memory_allocations(c: &mut Criterion) {
    let store = create_entity_store(50_000);
    let mut renderer = GpuRenderer::new();
    let camera = Camera::new(800.0, 600.0);

    // First run warms up allocations
    renderer.sync_from_store(&store, &camera);

    c.bench_function("sync_with_preallocated", |b| {
        b.iter(|| {
            renderer.sync_from_store(black_box(&store), black_box(&camera));
        });
    });
}

criterion_group!(
    name = render_benches;
    config = config();
    targets =
        bench_sync_1k,
        bench_sync_10k,
        bench_sync_100k,
        bench_batch_distribution,
        bench_memory_allocations,
);

criterion_main!(render_benches);
