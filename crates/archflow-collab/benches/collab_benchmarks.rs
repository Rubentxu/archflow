use archflow_collab::crdt::CRDT;
use archflow_collab::merge::LwwStrategy;
use archflow_collab::types::{SiteId, VectorClock};
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

pub fn bench_crdt_merge_concurrent(c: &mut Criterion) {
    let site_a = SiteId::new();
    let site_b = SiteId::new();

    let mut crdt_a = CRDT::<BenchmarkRecord>::new(site_a);
    let mut crdt_b = CRDT::<BenchmarkRecord>::new(site_b);

    let records_a: Vec<_> = (0..1000)
        .map(|i| {
            let id = RecordId::from_str(&format!("merge_a_{:06}", i)).unwrap();
            BenchmarkRecord {
                id: id.clone(),
                index: None,
                name: format!("a_{}", i),
                value: i,
            }
        })
        .collect();

    let records_b: Vec<_> = (0..1000)
        .map(|i| {
            let id = RecordId::from_str(&format!("merge_b_{:06}", i)).unwrap();
            BenchmarkRecord {
                id: id.clone(),
                index: None,
                name: format!("b_{}", i),
                value: i,
            }
        })
        .collect();

    for record in &records_a {
        let _ = crdt_a.apply_local(record.clone());
    }

    for record in &records_b {
        let _ = crdt_b.apply_local(record.clone());
    }

    let clock_b = crdt_b.vector_clock().clone();
    let mut crdt_a_clone = CRDT::<BenchmarkRecord>::new(site_a);

    for record in &records_a {
        let _ = crdt_a_clone.apply_local(record.clone());
    }

    c.bench_function("crdt_merge_1000_concurrent", |b| {
        b.iter(|| {
            let mut test_crdt = CRDT::<BenchmarkRecord>::new(site_a);
            for record in &records_a {
                let _ = test_crdt.apply_local(record.clone());
            }

            let _ = test_crdt
                .merge(black_box(&clock_b), black_box(records_b.clone()))
                .unwrap();
        })
    });
}

pub fn bench_vector_clock_relation(c: &mut Criterion) {
    let mut clock_a = VectorClock::new();
    let mut clock_b = VectorClock::new();

    let sites: Vec<SiteId> = (0..100).map(|_| SiteId::new()).collect();

    for i in 0..50 {
        clock_a.increment(sites[i]);
    }

    for i in 50..100 {
        clock_b.increment(sites[i]);
    }

    c.bench_function("vector_clock_relation_10k", |b| {
        b.iter(|| {
            let _ = black_box(clock_a.relation(black_box(&clock_b)));
        })
    });
}

criterion_group!(
    benches,
    bench_crdt_apply_local,
    bench_crdt_merge,
    bench_lww_strategy,
    bench_crdt_merge_concurrent,
    bench_vector_clock_relation
);
criterion_main!(benches);
