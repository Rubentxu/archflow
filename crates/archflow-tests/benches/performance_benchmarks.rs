//! Performance benchmarks for ArchFlow components
//!
//! Run with: cargo bench --bench performance_benchmarks

use archflow_records::{Bounds, Record, RecordId, RecordStore};
use archflow_wasm_collab::{BinaryDeltaCodec, ShapeField, SharedBuffer};
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::time::Instant;

// ============================================================
// BENCHMARK TEST RECORDS
// ============================================================

#[derive(Debug, Clone)]
struct BenchRecord {
    id: RecordId,
    bounds: Option<Bounds>,
    value: f32,
}

impl BenchRecord {
    fn new(id: RecordId, x: f32, y: f32, value: f32) -> Self {
        Self {
            id,
            bounds: Some(Bounds {
                min_x: x as f64,
                min_y: y as f64,
                max_x: (x + 10.0) as f64,
                max_y: (y + 10.0) as f64,
            }),
            value,
        }
    }
}

impl Record for BenchRecord {
    fn id(&self) -> &RecordId {
        &self.id
    }

    fn type_name(&self) -> &'static str {
        "BenchRecord"
    }

    fn bounds(&self) -> Option<Bounds> {
        self.bounds.clone()
    }
}

// ============================================================
// RECORD STORE BENCHMARKS
// ============================================================

fn criterion_config() -> Criterion {
    Criterion::default()
        .sample_size(50)
        .measurement_time(std::time::Duration::from_secs(5))
        .warm_up_time(std::time::Duration::from_secs(2))
}

fn bench_record_insertion(c: &mut Criterion) {
    c.bench_function("record_insertion_1000", |b| {
        b.iter(|| {
            let mut store: RecordStore<BenchRecord> = RecordStore::new();
            for i in 0..black_box(1000) {
                let id = RecordId::from_u64(i as u64);
                let record = BenchRecord::new(id, i as f32, i as f32, i as f32);
                store.put(record);
            }
        });
    });

    c.bench_function("record_insertion_10000", |b| {
        b.iter(|| {
            let mut store: RecordStore<BenchRecord> = RecordStore::new();
            for i in 0..black_box(10000) {
                let id = RecordId::from_u64(i as u64);
                let record = BenchRecord::new(id, i as f32, i as f32, i as f32);
                store.put(record);
            }
        });
    });
}

fn bench_record_lookup(c: &mut Criterion) {
    // Setup: create a store with records
    let mut setup_store: RecordStore<BenchRecord> = RecordStore::new();
    for i in 0..10000 {
        let id = RecordId::from_u64(i as u64);
        let record = BenchRecord::new(id, i as f32, i as f32, i as f32);
        setup_store.put(record);
    }

    c.bench_function("record_lookup_sequential", |b| {
        b.iter(|| {
            for i in 0..black_box(1000) {
                let id = RecordId::from_u64(i as u64);
                black_box(setup_store.get(&id));
            }
        });
    });

    c.bench_function("record_lookup_random", |b| {
        b.iter(|| {
            for i in 0..black_box(1000) {
                let id = RecordId::from_u64((i * 7919) % 10000 as u64);
                black_box(setup_store.get(&id));
            }
        });
    });
}

fn bench_record_update(c: &mut Criterion) {
    // Setup: create a store with records
    let mut setup_store: RecordStore<BenchRecord> = RecordStore::new();
    for i in 0..10000 {
        let id = RecordId::from_u64(i as u64);
        let record = BenchRecord::new(id, i as f32, i as f32, i as f32);
        setup_store.put(record);
    }
    setup_store.drain_changes(); // Clear changes

    c.bench_function("record_update_100", |b| {
        b.iter(|| {
            for i in 0..black_box(100) {
                let id = RecordId::from_u64(i as u64);
                if let Some(record) = setup_store.get(&id) {
                    let updated = BenchRecord {
                        id: record.id().clone(),
                        bounds: record.bounds.clone(),
                        value: record.value + 1.0,
                    };
                    setup_store.put(updated);
                }
            }
            black_box(setup_store.drain_changes());
        });
    });
}

fn bench_change_set_drain(c: &mut Criterion) {
    // Setup: create a store with many records
    let mut setup_store: RecordStore<BenchRecord> = RecordStore::new();
    for i in 0..10000 {
        let id = RecordId::from_u64(i as u64);
        let record = BenchRecord::new(id, i as f32, i as f32, i as f32);
        setup_store.put(record);
    }

    c.bench_function("change_set_drain_10000", |b| {
        b.iter(|| {
            // Modify some records
            for i in 0..black_box(1000) {
                let id = RecordId::from_u64(i as u64);
                if let Some(record) = setup_store.get(&id) {
                    let updated = BenchRecord {
                        id: record.id().clone(),
                        bounds: record.bounds.clone(),
                        value: record.value + 1.0,
                    };
                    setup_store.put(updated);
                }
            }
            let _changeset = black_box(setup_store.drain_changes());
        });
    });
}

fn bench_record_iteration(c: &mut Criterion) {
    // Setup: create a store with records
    let mut setup_store: RecordStore<BenchRecord> = RecordStore::new();
    for i in 0..10000 {
        let id = RecordId::from_u64(i as u64);
        let record = BenchRecord::new(id, i as f32, i as f32, i as f32);
        setup_store.put(record);
    }

    c.bench_function("record_iteration_10000", |b| {
        b.iter(|| {
            let count = black_box(setup_store.iter().count());
            assert_eq!(count, 10000);
        });
    });
}

// ============================================================
// SHARED BUFFER BENCHMARKS
// ============================================================

fn bench_shared_buffer_creation(c: &mut Criterion) {
    c.bench_function("shared_buffer_creation_1000", |b| {
        b.iter(|| {
            let mut buffer = SharedBuffer::new(black_box(1000));
            let ids: Vec<u64> = (0..1000).map(|id| id as u64).collect();
            let get_record = |id: u64| Some((id as f32, id as f32 * 1.5, [255, 255, 255, 255]));
            buffer.update(&ids, &get_record);
        });
    });

    c.bench_function("shared_buffer_creation_10000", |b| {
        b.iter(|| {
            let mut buffer = SharedBuffer::new(black_box(10000));
            let ids: Vec<u64> = (0..10000).map(|id| id as u64).collect();
            let get_record = |id: u64| Some((id as f32, id as f32 * 1.5, [255, 255, 255, 255]));
            buffer.update(&ids, &get_record);
        });
    });
}

fn bench_shared_buffer_update(c: &mut Criterion) {
    // Setup: create a buffer with initial data
    let mut setup_buffer = SharedBuffer::new(10000);
    let setup_ids: Vec<u64> = (0..10000).map(|id| id as u64).collect();
    let get_record = |id: u64| Some((id as f32, id as f32 * 1.5, [255, 255, 255, 255]));
    setup_buffer.update(&setup_ids, &get_record);

    c.bench_function("shared_buffer_update_1000", |b| {
        b.iter(|| {
            let ids: Vec<u64> = (0..1000).map(|id| id as u64).collect();
            let get_record =
                |id: u64| Some((id as f32, id as f32 * 2.0, [(id % 256) as u8, 128, 64, 255]));
            setup_buffer.update(&ids, &get_record);
        });
    });
}

// ============================================================
// DELTA ENCODING BENCHMARKS
// ============================================================

fn bench_delta_encoding(c: &mut Criterion) {
    c.bench_function("delta_encode_1000", |b| {
        b.iter(|| {
            for i in 0..black_box(1000) {
                let mut encoded = Vec::new();
                BinaryDeltaCodec::encode_delta(
                    &mut encoded,
                    i as u64,
                    ShapeField::Position as u8 | ShapeField::Color as u8,
                    Some((i as f32, i as f32 * 1.5)),
                    Some((
                        (i % 256) as u8,
                        ((i * 2) % 256) as u8,
                        ((i * 3) % 256) as u8,
                        255,
                    )),
                    None,
                );
            }
        });
    });

    c.bench_function("delta_decode_1000", |b| {
        // Pre-encode deltas
        let encoded_deltas: Vec<Vec<u8>> = (0..1000)
            .map(|i| {
                let mut encoded = Vec::new();
                BinaryDeltaCodec::encode_delta(
                    &mut encoded,
                    i as u64,
                    ShapeField::Position as u8 | ShapeField::Color as u8,
                    Some((i as f32, i as f32 * 1.5)),
                    Some((
                        (i % 256) as u8,
                        ((i * 2) % 256) as u8,
                        ((i * 3) % 256) as u8,
                        255,
                    )),
                    None,
                );
                encoded
            })
            .collect();

        b.iter(|| {
            for encoded in &encoded_deltas {
                let _decoded = black_box(BinaryDeltaCodec::decode_delta(encoded));
            }
        });
    });
}

// ============================================================
// THROUGHPUT BENCHMARKS
// ============================================================

fn bench_throughput_insert(c: &mut Criterion) {
    c.bench_function("throughput_insert_100k", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = std::time::Duration::ZERO;

            for _ in 0..iters {
                let start = Instant::now();
                let mut store: RecordStore<BenchRecord> = RecordStore::new();
                for i in 0..100000 {
                    let id = RecordId::from_u64(i as u64);
                    let record = BenchRecord::new(id, i as f32, i as f32, i as f32);
                    store.put(record);
                }
                total_duration += start.elapsed();
            }

            total_duration
        });
    });
}

fn bench_throughput_batch_operations(c: &mut Criterion) {
    c.bench_function("throughput_batch_update_100k", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = std::time::Duration::ZERO;

            for _ in 0..iters {
                // Setup
                let mut store: RecordStore<BenchRecord> = RecordStore::new();
                for i in 0..100000 {
                    let id = RecordId::from_u64(i as u64);
                    let record = BenchRecord::new(id, i as f32, i as f32, i as f32);
                    store.put(record);
                }
                store.drain_changes();

                // Batch update
                let start = Instant::now();
                for i in 0..10000 {
                    let id = RecordId::from_u64(i as u64);
                    if let Some(record) = store.get(&id) {
                        let updated = BenchRecord {
                            id: record.id().clone(),
                            bounds: record.bounds.clone(),
                            value: record.value + 1.0,
                        };
                        store.put(updated);
                    }
                }
                let _changeset = store.drain_changes();
                total_duration += start.elapsed();
            }

            total_duration
        });
    });
}

criterion_group!(
    benches,
    bench_record_insertion,
    bench_record_lookup,
    bench_record_update,
    bench_change_set_drain,
    bench_record_iteration,
    bench_shared_buffer_creation,
    bench_shared_buffer_update,
    bench_delta_encoding,
    bench_throughput_insert,
    bench_throughput_batch_operations
);

criterion_main!(benches);
