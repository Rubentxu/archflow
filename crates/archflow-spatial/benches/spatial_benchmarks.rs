//! # Spatial Benchmarks
//!
//! Performance benchmarks for spatial indexing operations.

use archflow_records::{Bounds, Record, RecordId};
use archflow_spatial::queries::SpatialQueries;
use archflow_spatial::rtree::RTreeIndex;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkRecord {
    pub id: RecordId,
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

fn generate_test_data(count: usize) -> Vec<(RecordId, Bounds)> {
    (0..count)
        .map(|i| {
            let x = (i as f64 % 100.0) * 10.0;
            let y = (i as f64 / 100.0) * 10.0;
            let bounds = Bounds::new(x, y, x + 5.0, y + 5.0);
            (
                RecordId::from_str(&format!("bench_data_{:08}", i)).unwrap(),
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
        let mut index = RTreeIndex::new(16);

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
        let mut index = RTreeIndex::new(16);

        for (id, bounds) in items {
            index.insert(id, bounds);
        }

        let query = Bounds::new(0.0, 0.0, 100.0, 100.0);

        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = index.rect_query(query.clone());
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
        let mut index = RTreeIndex::new(16);

        for (id, bounds) in items {
            index.insert(id, bounds);
        }

        let viewport = Bounds::new(0.0, 0.0, 100.0, 100.0);
        let queries = SpatialQueries::<BenchmarkRecord>::new(index);

        let start = std::time::Instant::now();
        for _ in 0..100 {
            let _ = queries.selection_expanded(viewport.clone(), 0.0);
        }
        let elapsed = start.elapsed();

        // Selection debe ser rápida
        assert!(elapsed.as_millis() < 100);
    }
}
