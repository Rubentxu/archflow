//! Benchmarks for core crate - geometry and records

use archflow_core::geometry::Vec2;
use archflow_core::records::{FractionalIndex, Record, RecordId, Store};
use criterion::{Criterion, black_box, criterion_group, criterion_main};

/// A simple benchmark record
#[derive(Debug, Clone, PartialEq)]
struct BenchmarkRecord {
    id: RecordId,
    index: FractionalIndex,
    type_name: String,
    position: Vec2,
}

impl Record for BenchmarkRecord {
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
        }
    }
}

fn make_record_id(i: u16) -> RecordId {
    RecordId::new(format!("bench-rec-{:010}", i))
}

fn criterion_config() -> Criterion {
    Criterion::default()
        .sample_size(100)
        .measurement_time(std::time::Duration::from_secs(2))
}

fn bench_vec2_operations(c: &mut Criterion) {
    let v1 = Vec2::new(100.0, 200.0);
    let v2 = Vec2::new(50.0, -25.0);

    c.bench_function("vec2_add", |b| b.iter(|| black_box(v1) + black_box(v2)));
    c.bench_function("vec2_sub", |b| b.iter(|| black_box(v1) - black_box(v2)));
    c.bench_function("vec2_distance", |b| {
        b.iter(|| black_box(v1).distance_to(black_box(v2)))
    });
    c.bench_function("vec2_dot", |b| b.iter(|| black_box(v1).dot(black_box(v2))));
    c.bench_function("vec2_normalize", |b| b.iter(|| black_box(v1).normalize()));
    c.bench_function("vec2_lerp", |b| {
        b.iter(|| Vec2::lerp(black_box(v1), black_box(v2), 0.5))
    });
    c.bench_function("vec2_length", |b| b.iter(|| black_box(v1).length()));
}

fn bench_store_insert(c: &mut Criterion) {
    let mut store = Store::new();
    let mut rng = fastrand::Rng::new();

    c.bench_function("store_insert_single", |b| {
        b.iter(|| {
            let id = make_record_id(rng.u16(..));
            let record = BenchmarkRecord {
                id,
                index: FractionalIndex::new("a0".to_string()),
                type_name: "rect".to_string(),
                position: Vec2::new(rng.f32() * 1000.0, rng.f32() * 1000.0),
            };
            store.put(record);
        })
    });
}

fn bench_store_get(c: &mut Criterion) {
    let mut store = Store::new();
    let mut rng = fastrand::Rng::new();
    let mut ids = Vec::new();

    // Pre-populate store
    for i in 0..1000 {
        let id = make_record_id(i);
        let record = BenchmarkRecord {
            id: id.clone(),
            index: FractionalIndex::new(format!("a{}", i)),
            type_name: "rect".to_string(),
            position: Vec2::new(i as f32, i as f32),
        };
        store.put(record);
        ids.push(id);
    }

    c.bench_function("store_get_random", |b| {
        b.iter(|| {
            let idx = rng.usize(..1000);
            black_box(store.get(black_box(&ids[idx])))
        })
    });
}

fn bench_store_iter(c: &mut Criterion) {
    let mut store = Store::new();

    // Pre-populate store
    for i in 0..1000 {
        let record = BenchmarkRecord {
            id: make_record_id(i),
            index: FractionalIndex::new(format!("a{}", i)),
            type_name: "rect".to_string(),
            position: Vec2::new(i as f32, i as f32),
        };
        store.put(record);
    }

    c.bench_function("store_iter_all", |b| {
        b.iter(|| {
            let mut count = 0;
            for record in black_box(&store).iter() {
                black_box(record);
                count += 1;
            }
            count
        })
    });
}

fn bench_store_undo(c: &mut Criterion) {
    let mut store = Store::new();

    // Pre-populate with changes
    for i in 0..100 {
        let id = make_record_id(i);
        store.put(BenchmarkRecord {
            id: id.clone(),
            index: FractionalIndex::new(format!("a{}", i)),
            type_name: "rect".to_string(),
            position: Vec2::new(i as f32, i as f32),
        });
        store.put(BenchmarkRecord {
            id,
            index: FractionalIndex::new(format!("a{}", i)),
            type_name: "rect".to_string(),
            position: Vec2::new(i as f32 * 2.0, i as f32 * 2.0),
        });
    }

    c.bench_function("store_undo", |b| {
        b.iter(|| {
            let mut result = true;
            while result {
                result = black_box(&mut store).undo();
            }
        })
    });
}

fn bench_fractional_index(c: &mut Criterion) {
    let idx1 = FractionalIndex::new("a0".to_string());
    let idx2 = FractionalIndex::new("a1".to_string());

    c.bench_function("fractional_index_between", |b| {
        b.iter(|| FractionalIndex::between(Some(black_box(&idx1)), Some(black_box(&idx2))))
    });
}

criterion_group!(
    name = geometry_benches;
    config = criterion_config();
    targets = bench_vec2_operations
);

criterion_group!(
    name = records_benches;
    config = criterion_config();
    targets = bench_store_insert, bench_store_get, bench_store_iter, bench_store_undo, bench_fractional_index
);

criterion_main!(geometry_benches, records_benches);
