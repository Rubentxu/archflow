//! Integration and stress tests for ArchFlow
//!
//! This crate contains end-to-end tests that verify the complete
//! workflow across all ArchFlow components.

use archflow_records::{Bounds, Record, RecordId, RecordStore};
use archflow_wasm_collab::{BinaryDeltaCodec, ShapeField, SharedBuffer};
use std::str::FromStr;
use std::time::Instant;

// ============================================================
// INTEGRATION TESTS
// ============================================================

/// Test record implementation - concrete type (not Box<dyn Record>)
#[derive(Debug, Clone)]
struct TestRecord {
    id: RecordId,
    bounds: Option<Bounds>,
    color: [u8; 4],
}

impl TestRecord {
    fn new(id: RecordId, x: f32, y: f32, width: f32, height: f32, color: [u8; 4]) -> Self {
        Self {
            id,
            bounds: Some(Bounds {
                min_x: x as f64,
                min_y: y as f64,
                max_x: (x + width) as f64,
                max_y: (y + height) as f64,
            }),
            color,
        }
    }
}

impl Record for TestRecord {
    fn id(&self) -> &RecordId {
        &self.id
    }

    fn type_name(&self) -> &'static str {
        "TestRecord"
    }

    fn bounds(&self) -> Option<Bounds> {
        self.bounds.clone()
    }
}

/// Test: Full workflow from Records to SharedBuffer
#[test]
fn test_full_workflow_records_to_buffer() {
    let mut store: RecordStore<TestRecord> = RecordStore::new();

    // Create 100 test records
    for i in 0..100 {
        let id = RecordId::from_u64(i as u64);
        let record = TestRecord::new(
            id.clone(),
            (i % 10) as f32 * 100.0,
            (i / 10) as f32 * 100.0,
            50.0,
            50.0,
            [
                (i % 256) as u8,
                (i * 2 % 256) as u8,
                (i * 3 % 256) as u8,
                255,
            ],
        );
        store.put(record);
    }

    assert_eq!(store.len(), 100);

    // Collect all IDs
    let ids: Vec<u64> = (0..100).map(|id| id as u64).collect();

    // Modify records and verify changeset
    for i in 0..10 {
        let id = RecordId::from_u64(i as u64);
        if let Some(record) = store.get(&id) {
            let updated = TestRecord {
                id: record.id().clone(),
                bounds: Some(Bounds {
                    min_x: i as f64 * 120.0,
                    min_y: i as f64 * 120.0,
                    max_x: i as f64 * 120.0 + 50.0,
                    max_y: i as f64 * 120.0 + 50.0,
                }),
                color: record.color,
            };
            store.put(updated);
        }
    }

    let changeset = store.drain_changes();
    assert_eq!(changeset.updated_indices().count(), 10);

    // SharedBuffer export
    let mut shared_buffer = SharedBuffer::new(100);
    let get_record = |id: u64| {
        store.get(&RecordId::from_u64(id)).and_then(|r| {
            let bounds = r.bounds()?;
            Some((bounds.min_x as f32, bounds.min_y as f32, r.color))
        })
    };
    shared_buffer.update(&ids, &get_record);
    assert!(!shared_buffer.is_empty());
}

/// Test: Binary delta encoding
#[test]
fn test_binary_delta_encoding() {
    let id = 42u64;
    let mask = (ShapeField::Position as u8) | (ShapeField::Color as u8);

    let mut encoded = Vec::new();
    BinaryDeltaCodec::encode_delta(
        &mut encoded,
        id,
        mask,
        Some((100.0, 200.0)),
        Some((255, 128, 64, 255)),
        None,
    );

    let decoded = BinaryDeltaCodec::decode_delta(&encoded).unwrap();
    assert_eq!(decoded.id, id);
    assert_eq!(decoded.position, Some((100.0, 200.0)));
    assert_eq!(decoded.color, Some((255, 128, 64, 255)));
}

/// Test: RecordStore CRUD
#[test]
fn test_record_store_crud() {
    let mut store: RecordStore<TestRecord> = RecordStore::new();

    // Create
    let id = RecordId::from_str("crud_test_001").unwrap();
    let record = TestRecord::new(id.clone(), 100.0, 200.0, 50.0, 50.0, [255, 128, 64, 255]);
    store.put(record);

    // Read
    assert!(store.get(&id).is_some());

    // Update
    let updated = TestRecord::new(id.clone(), 150.0, 250.0, 50.0, 50.0, [0, 255, 0, 255]);
    store.put(updated);

    let retrieved = store.get(&id).unwrap();
    assert_eq!(retrieved.bounds().unwrap().min_x, 150.0);
}

/// Test: ChangeSet optimization
#[test]
fn test_change_set_optimization() {
    let mut store: RecordStore<TestRecord> = RecordStore::new();

    for i in 0..10000 {
        let id = RecordId::from_u64(i as u64);
        let record = TestRecord::new(
            id,
            (i % 100) as f32 * 10.0,
            (i / 100) as f32 * 10.0,
            5.0,
            5.0,
            [255, 255, 255, 255],
        );
        store.put(record);
    }

    let changeset = store.drain_changes();
    // Initially created all records, so created_indices should have 10000
    assert_eq!(changeset.created_indices().count(), 10000);
    assert_eq!(changeset.updated_indices().count(), 0);

    // Modify 10 records - these should be marked as updated (not created)
    for i in 0..10 {
        let id = RecordId::from_u64(i as u64);
        if let Some(record) = store.get(&id) {
            let updated = TestRecord {
                id: record.id().clone(),
                bounds: record.bounds.clone(),
                color: [255, 0, 0, 255],
            };
            store.put(updated);
        }
    }

    let changeset2 = store.drain_changes();
    // After modifications, created should be 0 (records already exist)
    assert_eq!(changeset2.created_indices().count(), 0);
    // Should have 10 updated records
    assert_eq!(changeset2.updated_indices().count(), 10);

    // Modify same 10 records again - should still be updates
    for i in 0..10 {
        let id = RecordId::from_u64(i as u64);
        if let Some(record) = store.get(&id) {
            let updated = TestRecord {
                id: record.id().clone(),
                bounds: record.bounds.clone(),
                color: [0, 255, 0, 255],
            };
            store.put(updated);
        }
    }

    let changeset3 = store.drain_changes();
    assert_eq!(changeset3.created_indices().count(), 0);
    assert_eq!(changeset3.updated_indices().count(), 10);
}

/// Test: Bounds center
#[test]
fn test_bounds_center() {
    let bounds = Bounds {
        min_x: 100.0,
        min_y: 100.0,
        max_x: 200.0,
        max_y: 200.0,
    };

    let center_x = (bounds.min_x + bounds.max_x) / 2.0;
    let center_y = (bounds.min_y + bounds.max_y) / 2.0;

    assert!((center_x - 150.0).abs() < 0.001);
    assert!((center_y - 150.0).abs() < 0.001);
}

/// Test: RecordId validation
#[test]
fn test_record_id_validation() {
    let valid = RecordId::from_str("record_1234567890");
    assert!(valid.is_ok());
    assert_eq!(valid.unwrap().len(), 17);

    let short = RecordId::from_str("short");
    assert!(short.is_err());

    let special = RecordId::from_str("invalid@chars!");
    assert!(special.is_err());
}

// ============================================================
// STRESS TESTS
// ============================================================

#[derive(Debug, Clone)]
struct StressRecord {
    id: RecordId,
    bounds: Option<Bounds>,
    value: f32,
}

impl StressRecord {
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

impl Record for StressRecord {
    fn id(&self) -> &RecordId {
        &self.id
    }

    fn type_name(&self) -> &'static str {
        "StressRecord"
    }

    fn bounds(&self) -> Option<Bounds> {
        self.bounds.clone()
    }
}

/// Stress test: Large dataset insertion
#[test]
fn test_large_dataset_insertion() {
    const NUM_RECORDS: usize = 20000;

    let start = Instant::now();
    let mut store: RecordStore<StressRecord> = RecordStore::new();

    for i in 0..NUM_RECORDS {
        let id = RecordId::from_u64(i as u64);
        let record = StressRecord::new(id, (i % 1000) as f32, (i / 1000) as f32, i as f32);
        store.put(record);
    }

    let elapsed = start.elapsed();
    assert!(elapsed.as_secs() < 30, "Insert took {:?}", elapsed);
    assert_eq!(store.len(), NUM_RECORDS);
}

/// Stress test: Memory usage
#[test]
fn test_memory_usage() {
    const NUM_RECORDS: usize = 5000;

    let before = get_memory_usage_kb();
    let mut store: RecordStore<StressRecord> = RecordStore::new();

    for i in 0..NUM_RECORDS {
        let id = RecordId::from_u64(i as u64);
        let record = StressRecord::new(id, i as f32, i as f32, i as f32);
        store.put(record);
    }

    let after = get_memory_usage_kb();
    let per_record = (after.saturating_sub(before)) / NUM_RECORDS as u64;

    assert!(per_record < 1024, "Memory per record: {} bytes", per_record);
}

fn get_memory_usage_kb() -> u64 {
    #[cfg(target_os = "linux")]
    {
        let pid = std::process::id();
        let path = format!("/proc/{}/statm", pid);
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Some(first) = content.split_whitespace().next() {
                return first.parse::<u64>().unwrap() * 4;
            }
        }
    }
    0
}

/// Stress test: Rapid updates
#[test]
fn test_rapid_updates() {
    const UPDATES: usize = 500;
    const BATCH_SIZE: usize = 50;

    let mut store: RecordStore<StressRecord> = RecordStore::new();

    for i in 0..BATCH_SIZE {
        let id = RecordId::from_u64(i as u64);
        let record = StressRecord::new(id, 0.0, 0.0, 0.0);
        store.put(record);
    }

    let start = Instant::now();
    for _ in 0..UPDATES {
        for i in 0..BATCH_SIZE {
            let id = RecordId::from_u64(i as u64);
            if let Some(record) = store.get(&id) {
                let updated = StressRecord {
                    id: record.id().clone(),
                    bounds: record.bounds.clone(),
                    value: record.value + 1.0,
                };
                store.put(updated);
            }
        }
        store.drain_changes();
    }
    let elapsed = start.elapsed();

    assert!(elapsed.as_secs() < 30, "Updates took {:?}", elapsed);
}

/// Stress test: SharedBuffer large dataset
#[test]
fn test_shared_buffer_large() {
    const MAX_ELEMENTS: usize = 10000;

    let mut buffer = SharedBuffer::new(MAX_ELEMENTS);
    let ids: Vec<u64> = (0..MAX_ELEMENTS).map(|id| id as u64).collect();
    let get_record = |id: u64| Some((id as f32, id as f32 * 1.5, [255, 255, 255, 255]));

    buffer.update(&ids, &get_record);
    assert_eq!(buffer.len(), MAX_ELEMENTS);

    // Pointer stability
    let ptr1 = buffer.get_ptr();
    buffer.update(&ids, &get_record);
    assert_eq!(buffer.get_ptr(), ptr1);
}

/// Stress test: Delta encoding throughput
#[test]
fn test_delta_encoding_throughput() {
    const DELTAS: usize = 2000;

    let start = Instant::now();
    for i in 0..DELTAS {
        let mut encoded = Vec::new();
        BinaryDeltaCodec::encode_delta(
            &mut encoded,
            i as u64,
            ShapeField::Position as u8,
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
    let elapsed = start.elapsed();

    assert!(elapsed.as_secs() < 2, "Delta encoding took {:?}", elapsed);
}

/// Stress test: Iterating over records
#[test]
fn test_record_iteration() {
    const NUM_RECORDS: usize = 10000;

    let mut store: RecordStore<StressRecord> = RecordStore::new();
    for i in 0..NUM_RECORDS {
        let id = RecordId::from_u64(i as u64);
        let record = StressRecord::new(id, (i % 100) as f32, (i / 100) as f32, i as f32);
        store.put(record);
    }

    let start = Instant::now();
    let count = store.iter().count();
    let elapsed = start.elapsed();

    assert_eq!(count, NUM_RECORDS);
    assert!(elapsed.as_millis() < 100, "Iteration took {:?}", elapsed);
}

/// Stress test: Version tracking
#[test]
fn test_version_tracking() {
    let mut store: RecordStore<TestRecord> = RecordStore::new();

    let v0 = store.version();
    assert_eq!(v0, 0);

    let id = RecordId::from_str("version_test_001").unwrap();
    let record = TestRecord::new(id.clone(), 100.0, 200.0, 50.0, 50.0, [255, 128, 64, 255]);
    store.put(record);

    let v1 = store.version();
    assert_eq!(v1, 1);

    store.drain_changes();

    let updated = TestRecord::new(id.clone(), 150.0, 250.0, 50.0, 50.0, [0, 255, 0, 255]);
    store.put(updated);

    let v2 = store.version();
    assert_eq!(v2, 2);
}
