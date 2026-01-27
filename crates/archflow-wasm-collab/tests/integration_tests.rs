//! Integration tests for archflow-wasm-collab
//!
//! These tests verify the complete workflow between SharedBuffer,
//! BinaryDeltaCodec, and WasmBridge components.

use archflow_wasm_collab::{
    SharedBuffer,
    binary_delta_codec::{BinaryDeltaCodec, ShapeField},
    wasm_bridge::WasmBridge,
};

/// Integration test: Full round-trip from WasmBridge to SharedBuffer
#[test]
fn test_bridge_to_shared_buffer_roundtrip() {
    let mut bridge = WasmBridge::new(100);

    // Update several records
    bridge.update_position(1, 100.0, 200.0);
    bridge.update_color(1, 255, 128, 64, 255);
    bridge.update_position(2, 150.0, 250.0);
    bridge.update_color(2, 0, 255, 0, 255);
    bridge.update_position(3, 300.0, 400.0);

    // Update the shared buffer
    bridge.update_render_buffer();

    // Verify buffer is populated
    assert_eq!(bridge.render_buffer_len(), 3);
    assert_eq!(bridge.record_count(), 3);
}

/// Integration test: Binary delta codec with WasmBridge
#[test]
fn test_delta_codec_with_bridge() {
    let mut bridge = WasmBridge::new(100);

    // Create deltas for multiple records
    let mut all_data = Vec::new();

    for i in 0..10 {
        let mut delta = Vec::new();
        BinaryDeltaCodec::encode_delta(
            &mut delta,
            i as u64,
            (ShapeField::Position as u8) | (ShapeField::Color as u8),
            Some((i as f32 * 10.0, i as f32 * 20.0)),
            Some(((i * 25) as u8, (i * 15) as u8, (i * 5) as u8, 255)),
            None,
        );
        all_data.extend(delta);
    }

    // Apply all deltas
    let count = bridge.apply_deltas(&all_data);
    assert_eq!(count, 10);
    assert_eq!(bridge.record_count(), 10);

    // Serialize and verify changes
    let changes = bridge.serialize_changes();
    assert!(!changes.is_empty());
    assert_eq!(bridge.dirty_count(), 0);
}

/// Integration test: Shared buffer pointer stability with updates
#[test]
fn test_shared_buffer_pointer_stability() {
    let mut buffer = SharedBuffer::new(50);

    let initial_ptr = buffer.get_ptr();

    // Perform multiple updates within capacity
    for batch in 0..5 {
        let ids: Vec<u64> = (0..10).map(|i| (batch * 10 + i) as u64).collect();
        // Truncate to capacity
        let ids = if ids.len() > 50 {
            ids[..50].to_vec()
        } else {
            ids
        };
        let get_record = |id: u64| Some((id as f32, id as f32 * 2.0, [255, 255, 255, 255]));
        buffer.update(&ids, &get_record);
    }

    // Pointer should remain stable
    assert_eq!(buffer.get_ptr(), initial_ptr);
    assert!(buffer.len() <= 50); // Capped at capacity
}

/// Integration test: Delta encoding efficiency
#[test]
fn test_delta_encoding_efficiency() {
    // Test encoding and decoding individual deltas (simpler test)
    for i in 0..50 {
        let mut encoded = Vec::new();
        BinaryDeltaCodec::encode_delta(
            &mut encoded,
            i as u64,
            ShapeField::Position as u8,
            Some((i as f32, i as f32 * 1.5)),
            None,
            None,
        );

        // Each encoded delta should be compact (~11 bytes)
        assert!(
            encoded.len() <= 15,
            "Delta {} has {} bytes, too large",
            i,
            encoded.len()
        );

        // Decode and verify
        let decoded = BinaryDeltaCodec::decode_delta(&encoded);
        assert!(decoded.is_some(), "Should decode delta {}", i);
        assert_eq!(decoded.unwrap().id, i as u64);
    }
}

/// Integration test: Concurrent-looking operations simulation
#[test]
fn test_concurrent_operations_simulation() {
    let mut bridge = WasmBridge::new(1000);

    // Simulate rapid updates from multiple "threads"
    for i in 0..100 {
        bridge.update_position(i, i as f32, i as f32);
        bridge.update_color(
            i,
            (i % 256) as u8,
            ((i * 2) % 256) as u8,
            ((i * 3) % 256) as u8,
            255,
        );
    }

    // Process deltas
    let changes = bridge.serialize_changes();
    assert!(!changes.is_empty());

    // Apply changes back
    let count = bridge.apply_deltas(&changes);
    assert_eq!(count, 100);

    // Update render buffer
    bridge.update_render_buffer();
    assert_eq!(bridge.render_buffer_len(), 100);
}

/// Integration test: Delete and recreate records
#[test]
fn test_delete_and_recreate() {
    let mut bridge = WasmBridge::new(100);

    // Create records
    bridge.update_position(1, 100.0, 200.0);
    bridge.update_position(2, 150.0, 250.0);
    bridge.update_position(3, 300.0, 400.0);
    assert_eq!(bridge.record_count(), 3);

    // Delete middle record
    bridge.delete(2);
    assert_eq!(bridge.record_count(), 2);

    // Recreate with different data
    bridge.update_position(2, 999.0, 888.0);
    assert_eq!(bridge.record_count(), 3);

    // Verify state
    bridge.update_render_buffer();
    assert_eq!(bridge.render_buffer_len(), 3);
}

/// Integration test: Partial delta application
#[test]
fn test_partial_delta_application() {
    let mut bridge = WasmBridge::new(100);

    // Create initial state
    bridge.update_position(1, 100.0, 200.0);
    bridge.update_color(1, 255, 255, 255, 255);

    // Apply position-only delta
    let mut pos_delta = Vec::new();
    BinaryDeltaCodec::encode_delta(
        &mut pos_delta,
        1,
        ShapeField::Position as u8,
        Some((500.0, 600.0)),
        None,
        None,
    );
    bridge.apply_delta(&pos_delta).unwrap();

    // Apply color-only delta
    let mut color_delta = Vec::new();
    BinaryDeltaCodec::encode_delta(
        &mut color_delta,
        1,
        ShapeField::Color as u8,
        None,
        Some((255, 0, 0, 255)),
        None,
    );
    bridge.apply_delta(&color_delta).unwrap();

    // Verify both changes are applied
    let serialized = bridge.serialize_record(1);
    let decoded = BinaryDeltaCodec::decode_delta(&serialized).unwrap();

    assert_eq!(decoded.position, Some((500.0, 600.0)));
    assert_eq!(decoded.color, Some((255, 0, 0, 255)));
}

/// Integration test: Boundary conditions
#[test]
fn test_boundary_conditions() {
    let mut bridge = WasmBridge::new(100);

    // Test with larger IDs that work with the buffer
    bridge.update_position(50, 1.0, 2.0);
    bridge.update_position(0, 3.0, 4.0);
    bridge.update_position(99, f32::MAX, f32::MIN);
    bridge.update_color(99, 0, 0, 0, 0);

    // Serialize and verify
    let changes = bridge.serialize_changes();
    assert!(!changes.is_empty());

    // Apply to new bridge
    let mut bridge2 = WasmBridge::new(100);
    let count = bridge2.apply_deltas(&changes);
    assert_eq!(count, 3, "Should apply 3 deltas");
}

/// Integration test: Memory efficiency with large batches
#[test]
fn test_large_batch_processing() {
    let mut bridge = WasmBridge::new(10000);

    // Process 1000 records in batches
    let batch_size = 100;
    for batch in 0..10 {
        let mut deltas = Vec::new();
        for i in 0..batch_size {
            let id = (batch * batch_size + i) as u64;
            BinaryDeltaCodec::encode_delta(
                &mut deltas,
                id,
                ShapeField::Position as u8,
                Some((id as f32, id as f32 * 1.5)),
                None,
                None,
            );
        }
        bridge.apply_deltas(&deltas);
    }

    assert_eq!(bridge.record_count(), 1000);
    assert_eq!(bridge.dirty_count(), 1000);

    // Serialize should produce compact output
    let serialized = bridge.serialize_changes();
    // Each position delta: ~11 bytes (varint + mask + 2 floats)
    let expected_max = 1000 * 15;
    assert!(
        serialized.len() <= expected_max,
        "Serialized size {} exceeds expected {}",
        serialized.len(),
        expected_max
    );

    // Clear dirty tracking
    assert_eq!(bridge.dirty_count(), 0);
}

/// Integration test: Clear and reuse bridge
#[test]
fn test_clear_and_reuse() {
    let mut bridge = WasmBridge::new(100);

    // Add many records
    for i in 0..50 {
        bridge.update_position(i, i as f32, i as f32);
    }
    assert_eq!(bridge.record_count(), 50);

    // Clear all
    bridge.clear();
    assert_eq!(bridge.record_count(), 0);
    assert_eq!(bridge.dirty_count(), 0);

    // Reuse bridge
    bridge.update_position(0, 1.0, 2.0);
    assert_eq!(bridge.record_count(), 1);

    // Apply deltas from another source
    let mut deltas = Vec::new();
    for i in 1..10 {
        BinaryDeltaCodec::encode_delta(
            &mut deltas,
            i,
            ShapeField::Position as u8,
            Some((i as f32, i as f32)),
            None,
            None,
        );
    }
    bridge.apply_deltas(&deltas);
    assert_eq!(bridge.record_count(), 10);
}
