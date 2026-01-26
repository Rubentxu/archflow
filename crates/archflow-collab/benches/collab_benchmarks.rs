use archflow_collab::crdt::CRDT;
use archflow_collab::merge::LwwStrategy;
use archflow_collab::types::SiteId;
use archflow_records::{FractionalIndex, Record, RecordId};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkRecord {
    pub id: RecordId,
    pub index: Option<FractionalIndex>,
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

    fn index(&self) -> Option<&FractionalIndex> {
        self.index.as_ref()
    }

    fn with_index(mut self, index: FractionalIndex) -> Self {
        self.index = Some(index);
        self
    }
}

pub fn bench_crdt_apply_local(c: &mut Criterion) {
    let site_id = SiteId::new();
    let mut crdt = CRDT::<BenchmarkRecord>::new(site_id);
    let id = RecordId::from_str("bench_001").unwrap();

    c.bench_function("crdt_apply_local", |b| {
        b.iter(|| {
            let record = BenchmarkRecord {
                id: id.clone(),
                index: None,
                name: "Bench".into(),
                value: 42,
            };
            crdt.apply_local(black_box(record)).unwrap();
        })
    });
}

pub fn bench_crdt_merge(c: &mut Criterion) {
    let site_a = SiteId::new();
    let site_b = SiteId::new();
    let mut crdt_a = CRDT::<BenchmarkRecord>::new(site_a);
    let id = RecordId::from_str("bench_merge").unwrap();

    let record_b = BenchmarkRecord {
        id,
        index: None,
        name: "From B".into(),
        value: 100,
    };

    let mut crdt_b = CRDT::<BenchmarkRecord>::new(site_b);
    crdt_b.apply_local(record_b.clone()).unwrap();
    let clock_b = crdt_b.vector_clock().clone();

    c.bench_function("crdt_merge", |b| {
        b.iter(|| {
            crdt_a
                .merge(black_box(&clock_b), black_box(vec![record_b.clone()]))
                .unwrap();
        })
    });
}

pub fn bench_lww_strategy(c: &mut Criterion) {
    let site_id = SiteId::new();
    let strategy = LwwStrategy::new(site_id);
    let id = RecordId::from_str("bench_lww").unwrap();
    let r1 = BenchmarkRecord {
        id: id.clone(),
        index: None,
        name: "R1".into(),
        value: 1,
    };
    let r2 = BenchmarkRecord {
        id: id.clone(),
        index: None,
        name: "R2".into(),
        value: 2,
    };

    c.bench_function("lww_merge", |b| {
        b.iter(|| {
            use archflow_collab::merge::MergeStrategy;
            strategy.merge(black_box(&r1), black_box(&r2)).unwrap();
        })
    });
}

criterion_group!(
    benches,
    bench_crdt_apply_local,
    bench_crdt_merge,
    bench_lww_strategy
);
criterion_main!(benches);
