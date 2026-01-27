//! Performance benchmarks for batch rendering system
//!
//! These benchmarks measure the performance characteristics of the batch renderer,
//! including batch preparation time, memory usage, and iteration performance.

use archflow_renderers::{BatchRenderer2D, Bounds, MaterialId, Renderable, RgbaColor};
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use glam::Vec2;

/// Benchmark renderable for performance testing
#[derive(Clone)]
struct BenchmarkRenderable {
    bounds: Bounds,
    color: RgbaColor,
    material_id: MaterialId,
}

impl BenchmarkRenderable {
    fn new(bounds: Bounds, material_id: u64) -> Self {
        Self {
            bounds,
            color: RgbaColor::red(),
            material_id: MaterialId(material_id),
        }
    }
}

impl Renderable for BenchmarkRenderable {
    fn bounds(&self) -> Option<Bounds> {
        Some(self.bounds)
    }

    fn contains_point(&self, _point: Vec2) -> bool {
        false
    }

    fn render_priority(&self) -> i32 {
        0
    }

    fn material_id(&self) -> MaterialId {
        self.material_id
    }

    fn color(&self) -> RgbaColor {
        self.color
    }
}

fn criterion() -> Criterion {
    Criterion::default()
        .sample_size(100)
        .measurement_time(std::time::Duration::from_secs(2))
        .warm_up_time(std::time::Duration::from_secs(1))
}

fn bench_batch_rendererCreation(c: &mut Criterion) {
    c.bench_function("batch_renderer_creation_10k", |b| {
        b.iter(|| BatchRenderer2D::new(black_box(10_000)))
    });
}

fn bench_add_single_instance(c: &mut Criterion) {
    c.bench_function("add_single_instance", |b| {
        b.iter_batched(
            || {
                let mut renderer = BatchRenderer2D::new(10_000);
                let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
                let renderable = BenchmarkRenderable::new(bounds, 1);
                (renderer, renderable)
            },
            |(mut renderer, renderable)| {
                renderer.add(black_box(&renderable));
                renderer
            },
            criterion::BatchSize::PerIteration,
        )
    });
}

fn bench_add_many_instances_same_material(c: &mut Criterion) {
    c.bench_function("add_10k_instances_same_material", |b| {
        b.iter_batched(
            || {
                let mut renderer = BatchRenderer2D::new(10_000);
                let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
                let renderable = BenchmarkRenderable::new(bounds, 1);
                (renderer, renderable)
            },
            |(mut renderer, renderable)| {
                for _ in 0..10_000 {
                    renderer.add(black_box(&renderable));
                }
                renderer
            },
            criterion::BatchSize::PerIteration,
        )
    });
}

fn bench_add_many_instances_different_materials(c: &mut Criterion) {
    c.bench_function("add_10k_instances_different_materials", |b| {
        b.iter_batched(
            || {
                let mut renderer = BatchRenderer2D::new(10_000);
                let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
                (renderer, bounds)
            },
            |(mut renderer, bounds)| {
                for i in 0..10_000 {
                    let renderable = BenchmarkRenderable::new(bounds, i as u64);
                    renderer.add(black_box(&renderable));
                }
                renderer
            },
            criterion::BatchSize::PerIteration,
        )
    });
}

fn bench_clear(c: &mut Criterion) {
    c.bench_function("clear_renderer", |b| {
        b.iter_batched(
            || {
                let mut renderer = BatchRenderer2D::new(10_000);
                let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
                for i in 0..10_000 {
                    let renderable = BenchmarkRenderable::new(bounds, i as u64);
                    renderer.add(&renderable);
                }
                renderer
            },
            |mut renderer| {
                renderer.clear();
                renderer
            },
            criterion::BatchSize::PerIteration,
        )
    });
}

fn bench_iter_batches(c: &mut Criterion) {
    c.bench_function("iter_batches_100_materials", |b| {
        b.iter_batched(
            || {
                let mut renderer = BatchRenderer2D::new(10_000);
                let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
                for material in 0..100 {
                    for _ in 0..100 {
                        let renderable = BenchmarkRenderable::new(bounds, material);
                        renderer.add(&renderable);
                    }
                }
                renderer
            },
            |renderer| {
                let mut count = 0;
                for (material_id, instances) in renderer.iter_batches() {
                    black_box(material_id);
                    black_box(instances);
                    count += 1;
                }
                count
            },
            criterion::BatchSize::PerIteration,
        )
    });
}

fn bench_buffer_size_calculation(c: &mut Criterion) {
    c.bench_function("buffer_size_calculation", |b| {
        b.iter_batched(
            || {
                let mut renderer = BatchRenderer2D::new(10_000);
                let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
                for i in 0..10_000 {
                    let renderable = BenchmarkRenderable::new(bounds, i as u64);
                    renderer.add(&renderable);
                }
                renderer
            },
            |renderer| black_box(renderer.total_instance_buffer_size()),
            criterion::BatchSize::PerIteration,
        )
    });
}

fn bench_get_batch(c: &mut Criterion) {
    c.bench_function("get_batch", |b| {
        b.iter_batched(
            || {
                let mut renderer = BatchRenderer2D::new(10_000);
                let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
                for material in 0..100 {
                    for _ in 0..100 {
                        let renderable = BenchmarkRenderable::new(bounds, material);
                        renderer.add(&renderable);
                    }
                }
                renderer
            },
            |renderer| {
                for material in 0..100 {
                    black_box(renderer.get_batch(MaterialId(material)));
                }
                renderer
            },
            criterion::BatchSize::PerIteration,
        )
    });
}

fn bench_is_empty(c: &mut Criterion) {
    c.bench_function("is_empty_check", |b| {
        b.iter_batched(
            || {
                let mut renderer = BatchRenderer2D::new(10_000);
                let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
                for i in 0..10_000 {
                    let renderable = BenchmarkRenderable::new(bounds, i as u64);
                    renderer.add(&renderable);
                }
                renderer
            },
            |renderer| black_box(renderer.is_empty()),
            criterion::BatchSize::PerIteration,
        )
    });
}

fn bench_instance_count(c: &mut Criterion) {
    c.bench_function("instance_count", |b| {
        b.iter_batched(
            || {
                let mut renderer = BatchRenderer2D::new(10_000);
                let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
                for i in 0..10_000 {
                    let renderable = BenchmarkRenderable::new(bounds, i as u64);
                    renderer.add(&renderable);
                }
                renderer
            },
            |renderer| black_box(renderer.instance_count()),
            criterion::BatchSize::PerIteration,
        )
    });
}

fn bench_batch_count(c: &mut Criterion) {
    c.bench_function("batch_count", |b| {
        b.iter_batched(
            || {
                let mut renderer = BatchRenderer2D::new(10_000);
                let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
                for material in 0..100 {
                    for _ in 0..100 {
                        let renderable = BenchmarkRenderable::new(bounds, material);
                        renderer.add(&renderable);
                    }
                }
                renderer
            },
            |renderer| black_box(renderer.batch_count()),
            criterion::BatchSize::PerIteration,
        )
    });
}

// Memory benchmark - measures allocations
fn bench_memory_per_instance(c: &mut Criterion) {
    c.bench_function("memory_per_instance", |b| {
        b.iter_batched(
            || BatchRenderer2D::new(10_000),
            |mut renderer| {
                let bounds = Bounds::new(Vec2::ZERO, Vec2::new(100.0, 100.0));
                for i in 0..10_000 {
                    let renderable = BenchmarkRenderable::new(bounds, i as u64);
                    renderer.add(&renderable);
                }
                // Force allocation measurement
                let _size = renderer.total_instance_buffer_size();
                renderer
            },
            criterion::BatchSize::PerIteration,
        )
    });
}

criterion_group!(
    name = batch_renderer_benches;
    config = criterion();
    targets =
        bench_batch_rendererCreation,
        bench_add_single_instance,
        bench_add_many_instances_same_material,
        bench_add_many_instances_different_materials,
        bench_clear,
        bench_iter_batches,
        bench_buffer_size_calculation,
        bench_get_batch,
        bench_is_empty,
        bench_instance_count,
        bench_batch_count,
        bench_memory_per_instance,
);

criterion_main!(batch_renderer_benches);
