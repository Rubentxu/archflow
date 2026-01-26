//! # Spatial Benchmarks
//!
//! Performance benchmarks for spatial indexing operations.

use archflow_records::{Record, RecordId, RecordStore};
use archflow_spatial::rtree::RTreeIndex;
use archflow_spatial::trait_spatial_index::{SpatialBounds, AABB};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkRecord {
    pub id: RecordId,
    pub bounds: Option<AABB<[f32; 2]>>,
    pub index: Option<archflow_records::FractionalIndex>,
    pub name: String,
    pub value: i32,
}

impl Record for BenchmarkRecord {
    fn id(&self) -> &RecordId {
        &self.id
    }

    fn type_name(&self) -> &'static str {
        "BenchmarkRecord"
    }

    fn index(&self) -> Option<&archflow_records::FractionalIndex> {
        self.index.as_ref()
    }

    fn with_index(mut self, _index: archflow_records::FractionalIndex) -> Self {
        self
    }
}

impl archflow_spatial::trait_spatial_index::SpatialBounds for AABB<[f32; 2]> {
    fn from_record(_record: &impl archflow_spatial::trait_spatial_index::HasBounds) -> Self {
        Self {
            min: [0.0, 0.0],
            max: [1.0, 1.0],
        }
    }

    fn contains(&self, point: [f32; 2]) -> bool {
        point[0] >= self.min[0]
            && point[0] <= self.max[0]
            && point[1] >= self.min[1]
            && point[1] <= self.max[1]
    }

    fn intersects(&self, other: &Self) -> bool {
        !(self.max[0] < other.min[0]
            || self.min[0] > other.max[0]
            || self.max[1] < other.min[1]
            || self.min[1] > other.max[1])
    }

    fn center(&self) -> [f32; 2] {
        [
            (self.min[0] + self.max[0]) / 2.0,
            (self.min[1] + self.max[1]) / 2.0,
        ]
    }

    fn area(&self) -> f32 {
        (self.max[0] - self.min[0]) * (self.max[1] - self.min[1])
    }

    fn grow(&self, amount: f32) -> Self {
        Self {
            min: [self.min[0] - amount, self.min[1] - amount],
            max: [self.max[0] + amount, self.max[1] + amount],
        }
    }

    fn to_aabb(&self) -> AABB<[f32; 2]> {
        AABB::from_corners(self.min, self.max)
    }
}

fn generate_test_data(count: usize) -> Vec<(RecordId, AABB<[f32; 2]>)> {
    (0..count)
        .map(|i| {
            let x = (i as f32 % 100.0) * 10.0;
            let y = (i as f32 / 100.0) * 10.0;
            let bounds = AABB::from_corners([x, y], [x + 5.0, y + 5.0]);
            (
                RecordId::from_str(&format!("bench_{:08}", i)).unwrap(),
                bounds,
            )
        })
        .collect()
}

#[cfg(test)]
mod benchmarks {
    use super::*;

    #[test]
    fn bench_rtree_insert_performance() {
        let items = generate_test_data(10_000);
        let mut index = RTreeIndex::<BenchmarkRecord>::new(16);

        let start = std::time::Instant::now();
        for (id, bounds) in items {
            index.insert(id, bounds);
        }
        let elapsed = start.elapsed();

        // F.11: < 100ms para 10k inserts
        assert!(elapsed.as_millis() < 100, "Insert took {:?}", elapsed);
    }

    #[test]
    fn bench_rtree_query_performance() {
        let items = generate_test_data(100_000);
        let mut index = RTreeIndex::<BenchmarkRecord>::new(16);

        for (id, bounds) in items {
            index.insert(id, bounds);
        }

        let query = AABB::from_corners([0.0, 0.0], [100.0, 100.0]);

        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = index.rect_query(&query);
        }
        let elapsed = start.elapsed();

        // F.11: < 1ms por query (1000 queries < 1000ms)
        assert!(
            elapsed.as_millis() < 1000,
            "1000 queries took {:?}",
            elapsed
        );
    }

    #[test]
    fn bench_viewport_culling() {
        let items = generate_test_data(50_000);
        let mut index = RTreeIndex::<BenchmarkRecord>::new(16);

        for (id, bounds) in items {
            index.insert(id, bounds);
        }

        let viewport = AABB::from_corners([0.0, 0.0], [100.0, 100.0]);

        let start = std::time::Instant::now();
        for _ in 0..100 {
            let _ = index.get_visible_elements(&viewport);
        }
        let elapsed = start.elapsed();

        // Caché debe hacer esto muy rápido
        assert!(elapsed.as_millis() < 10);
    }
}

criterion_group!(
    benches,
    bench_rtree_insert_performance,
    bench_rtree_query_performance,
    bench_viewport_culling
);
criterion_main!(benches);
