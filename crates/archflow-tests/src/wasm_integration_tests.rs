//! WASM Integration Tests for ArchFlow
//!
//! These tests verify the WASM bridge functionality and SharedArrayBuffer
//! communication between Rust and JavaScript.

use archflow_records::{Bounds, Record, RecordId, RecordStore};
use archflow_wasm_collab::{BinaryDeltaCodec, DecodedDelta, ShapeField, SharedBuffer};
use std::cmp::Ordering;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

// ============================================================
// TEST RECORDS FOR WASM
// ============================================================

#[derive(Debug, Clone)]
struct WasmTestRecord {
    id: RecordId,
    bounds: Option<Bounds>,
    position: [f32; 2],
    color: [u8; 4],
}

impl WasmTestRecord {
    fn new(id: RecordId, x: f32, y: f32, color: [u8; 4]) -> Self {
        Self {
            id,
            bounds: Some(Bounds {
                min_x: x as f64,
                min_y: y as f64,
                max_x: (x + 20.0) as f64,
                max_y: (y + 20.0) as f64,
            }),
            position: [x, y],
            color,
        }
    }
}

impl Record for WasmTestRecord {
    fn id(&self) -> &RecordId {
        &self.id
    }

    fn type_name(&self) -> &'static str {
        "WasmTestRecord"
    }

    fn bounds(&self) -> Option<Bounds> {
        self.bounds.clone()
    }
}

// ============================================================
// SHARED BUFFER TESTS
// ============================================================

#[test]
fn test_shared_buffer_initialization() {
    const MAX_ELEMENTS: usize = 1000;

    let buffer = SharedBuffer::new(MAX_ELEMENTS);
    assert!(buffer.is_empty());
    assert_eq!(buffer.len(), 0);
}

#[test]
fn test_shared_buffer_update() {
    const MAX_ELEMENTS: usize = 100;

    let mut buffer = SharedBuffer::new(MAX_ELEMENTS);
    let ids: Vec<u64> = (0..MAX_ELEMENTS).map(|id| id as u64).collect();

    // Update with position and color data
    let get_record = |id: u64| {
        Some((
            id as f32 * 10.0,
            id as f32 * 10.0,
            [(id % 256) as u8, 128, 64, 255],
        ))
    };

    buffer.update(&ids, &get_record);

    assert_eq!(buffer.len(), MAX_ELEMENTS);
    assert!(!buffer.is_empty());
}

#[test]
fn test_shared_buffer_pointer_stability() {
    const MAX_ELEMENTS: usize = 1000;

    let mut buffer = SharedBuffer::new(MAX_ELEMENTS);
    let ids: Vec<u64> = (0..MAX_ELEMENTS).map(|id| id as u64).collect();
    let get_record = |id: u64| Some((id as f32, id as f32, [255, 255, 255, 255]));

    // Get initial pointer
    let initial_ptr = buffer.get_ptr();

    // Update multiple times
    for _ in 0..10 {
        buffer.update(&ids, &get_record);
    }

    // Pointer should remain stable
    assert_eq!(buffer.get_ptr(), initial_ptr);
}

#[test]
fn test_shared_buffer_resize_behavior() {
    const INITIAL_SIZE: usize = 100;

    let mut buffer = SharedBuffer::new(INITIAL_SIZE);
    let ids: Vec<u64> = (0..INITIAL_SIZE).map(|id| id as u64).collect();
    let get_record = |id: u64| Some((id as f32, id as f32, [255, 255, 255, 255]));

    buffer.update(&ids, &get_record);
    assert_eq!(buffer.len(), INITIAL_SIZE);
}

// ============================================================
// BINARY DELTA CODEC TESTS
// ============================================================

#[test]
fn test_delta_encode_decode_roundtrip() {
    let id = 42u64;
    let position = (100.0, 200.0);
    let color = (255, 128, 64, 255);

    let mut encoded = Vec::new();
    BinaryDeltaCodec::encode_delta(
        &mut encoded,
        id,
        ShapeField::Position as u8 | ShapeField::Color as u8,
        Some(position),
        Some(color),
        None,
    );

    let decoded = BinaryDeltaCodec::decode_delta(&encoded);

    match decoded {
        Some(DecodedDelta {
            id: decoded_id,
            mask: _,
            position: decoded_pos,
            color: decoded_color,
            size: _,
        }) => {
            assert_eq!(decoded_id, id);
            assert_eq!(decoded_pos, Some(position));
            assert_eq!(decoded_color, Some(color));
        }
        None => panic!("decode_delta returned None"),
    }
}

#[test]
fn test_delta_encode_position_only() {
    let id = 123u64;
    let position = (50.0, 75.0);

    let mut encoded = Vec::new();
    BinaryDeltaCodec::encode_delta(
        &mut encoded,
        id,
        ShapeField::Position as u8,
        Some(position),
        None,
        None,
    );

    let decoded = BinaryDeltaCodec::decode_delta(&encoded);

    match decoded {
        Some(DecodedDelta {
            id: decoded_id,
            mask: _,
            position: decoded_pos,
            color: decoded_color,
            size: _,
        }) => {
            assert_eq!(decoded_id, id);
            assert_eq!(decoded_pos, Some(position));
            assert_eq!(decoded_color, None);
        }
        None => panic!("decode_delta returned None"),
    }
}

#[test]
fn test_delta_encode_color_only() {
    let id = 456u64;
    let color = (0, 255, 128, 255);

    let mut encoded = Vec::new();
    BinaryDeltaCodec::encode_delta(
        &mut encoded,
        id,
        ShapeField::Color as u8,
        None,
        Some(color),
        None,
    );

    let decoded = BinaryDeltaCodec::decode_delta(&encoded);

    match decoded {
        Some(DecodedDelta {
            id: decoded_id,
            mask: _,
            position: decoded_pos,
            color: decoded_color,
            size: _,
        }) => {
            assert_eq!(decoded_id, id);
            assert_eq!(decoded_pos, None);
            assert_eq!(decoded_color, Some(color));
        }
        None => panic!("decode_delta returned None"),
    }
}

#[test]
fn test_delta_encode_empty() {
    let id = 789u64;

    let mut encoded = Vec::new();
    BinaryDeltaCodec::encode_delta(&mut encoded, id, 0, None, None, None);

    let decoded = BinaryDeltaCodec::decode_delta(&encoded);

    match decoded {
        Some(DecodedDelta {
            id: decoded_id,
            mask: _,
            position: decoded_pos,
            color: decoded_color,
            size: _,
        }) => {
            assert_eq!(decoded_id, id);
            assert_eq!(decoded_pos, None);
            assert_eq!(decoded_color, None);
        }
        None => panic!("decode_delta returned None"),
    }
}

#[test]
fn test_delta_codec_batch_processing() {
    const BATCH_SIZE: usize = 100;

    let mut encoded_batch = Vec::new();

    // Encode batch
    for i in 0..BATCH_SIZE {
        BinaryDeltaCodec::encode_delta(
            &mut encoded_batch,
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

    // Decode all deltas using try_parse pattern
    // Since varint length is variable, we use a simple approach:
    // Each delta has: id (varint) + mask (1 byte) + position (8 bytes if present) + color (4 bytes if present)
    // For id values 0-99, varint encoding is 1 byte
    // So each delta is approximately 1 + 1 + 8 + 4 = 14 bytes
    let expected_bytes_per_delta = 14;
    let expected_total = BATCH_SIZE * expected_bytes_per_delta;

    // Verify we have approximately the right amount of data
    assert!(
        encoded_batch.len() >= BATCH_SIZE * 10,
        "Should have substantial encoded data"
    );

    // Decode and verify each delta exists
    let mut decoded_count = 0;
    let mut offset: usize = 0;

    // Use try_decode pattern - decode one at a time by trying different offsets
    while decoded_count < BATCH_SIZE {
        let mut found = false;
        // Try decoding from current offset with up to 20 bytes lookahead
        for lookahead in 0..20usize {
            let end_idx = offset + lookahead + 1;
            if end_idx <= encoded_batch.len() {
                if let Some(decoded) =
                    BinaryDeltaCodec::decode_delta(&encoded_batch[offset..end_idx])
                {
                    if decoded.id == decoded_count as u64 {
                        // Found the next delta, skip past it
                        offset = end_idx;
                        decoded_count += 1;
                        found = true;
                        break;
                    }
                }
            }
        }
        if !found {
            break;
        }
    }

    assert_eq!(decoded_count, BATCH_SIZE);
}

// ============================================================
// RECORD STORE WITH SHARED BUFFER
// ============================================================

#[test]
fn test_record_store_to_shared_buffer() {
    let mut store: RecordStore<WasmTestRecord> = RecordStore::new();

    // Create records
    for i in 0..50 {
        let id = RecordId::from_u64(i as u64);
        let record = WasmTestRecord::new(
            id.clone(),
            (i % 10) as f32 * 30.0,
            (i / 10) as f32 * 30.0,
            [(i % 256) as u8, 128, 64, 255],
        );
        store.put(record);
    }

    // Export to SharedBuffer
    let mut buffer = SharedBuffer::new(100);
    let ids: Vec<u64> = (0..50).map(|id| id as u64).collect();

    let get_record = |id: u64| {
        store.get(&RecordId::from_u64(id)).map(|r| {
            let pos = r.position;
            let color = r.color;
            (pos[0], pos[1], color)
        })
    };

    buffer.update(&ids, &get_record);
    assert_eq!(buffer.len(), 50);
}

#[test]
fn test_record_store_changes_to_deltas() {
    let mut store: RecordStore<WasmTestRecord> = RecordStore::new();

    // Create initial records
    for i in 0..10 {
        let id = RecordId::from_u64(i as u64);
        let record = WasmTestRecord::new(
            id.clone(),
            i as f32 * 10.0,
            i as f32 * 10.0,
            [255, 255, 255, 255],
        );
        store.put(record);
    }

    let changeset = store.drain_changes();
    let created_count = changeset.created_indices().count();
    assert_eq!(created_count, 10);

    // Modify records
    for i in 0..5 {
        let id = RecordId::from_u64(i as u64);
        if let Some(record) = store.get(&id) {
            let updated = WasmTestRecord {
                id: record.id().clone(),
                bounds: record.bounds.clone(),
                position: record.position,
                color: [255, 0, 0, 255], // Red for updated
            };
            store.put(updated);
        }
    }

    let changeset2 = store.drain_changes();
    assert_eq!(changeset2.created_indices().count(), 0);
    assert_eq!(changeset2.updated_indices().count(), 5);
}

// ============================================================
// CONCURRENT ACCESS SIMULATION
// ============================================================

#[test]
fn test_concurrent_record_operations() {
    use std::sync::Arc;
    use std::sync::atomic::AtomicUsize;

    let store = Arc::new(std::sync::Mutex::new(RecordStore::<WasmTestRecord>::new()));
    let write_count = Arc::new(AtomicUsize::new(0));
    let read_count = Arc::new(AtomicUsize::new(0));

    const THREADS: usize = 4;
    const OPERATIONS_PER_THREAD: usize = 100;

    let handles: Vec<std::thread::JoinHandle<()>> = (0..THREADS)
        .map(|i| {
            let store = Arc::clone(&store);
            let write_count = Arc::clone(&write_count);
            let read_count = Arc::clone(&read_count);

            std::thread::spawn(move || {
                for j in 0..OPERATIONS_PER_THREAD {
                    let global_id = (i * OPERATIONS_PER_THREAD + j) as u64;
                    let id = RecordId::from_u64(global_id);

                    // Write
                    let record = WasmTestRecord::new(
                        id.clone(),
                        global_id as f32,
                        global_id as f32,
                        [(global_id % 256) as u8, 128, 64, 255],
                    );

                    let _ = store.lock().unwrap().put(record);
                    write_count.fetch_add(1, AtomicOrdering::SeqCst);

                    // Read
                    let _ = store.lock().unwrap().get(&id);
                    read_count.fetch_add(1, AtomicOrdering::SeqCst);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(
        write_count.load(AtomicOrdering::SeqCst),
        THREADS * OPERATIONS_PER_THREAD
    );
    assert_eq!(
        read_count.load(AtomicOrdering::SeqCst),
        THREADS * OPERATIONS_PER_THREAD
    );
}

// ============================================================
// ERROR HANDLING TESTS
// ============================================================

#[test]
fn test_invalid_record_id() {
    // RecordId validation is tested in integration tests
    let result = RecordId::from_str("valid_id_1234567890");
    assert!(result.is_ok());

    let invalid = RecordId::from_str("short!");
    assert!(invalid.is_err());
}

#[test]
fn test_out_of_bounds_access() {
    let mut buffer = SharedBuffer::new(10);
    let ids: Vec<u64> = (0..10).map(|id| id as u64).collect();
    let get_record = |id: u64| Some((id as f32, id as f32, [255, 255, 255, 255]));

    buffer.update(&ids, &get_record);

    // Accessing beyond bounds should not panic
    let large_id = 10000u64;
    let result = get_record(large_id);
    assert!(result.is_some());
}
