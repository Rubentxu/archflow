//! ArchFlow Demo - Complete Workflow Demonstration
//!
//! This module demonstrates the complete ArchFlow workflow from
//! record creation through to WASM export and synchronization.
//!
//! Run with: cargo run --example demo

use archflow_records::{Bounds, Record, RecordId, RecordStore};
use archflow_wasm_collab::{BinaryDeltaCodec, DecodedDelta, ShapeField, SharedBuffer};
use std::str::FromStr;
use std::time::Instant;

// ============================================================
// DEMO RECORD TYPES
// ============================================================

/// A visual element record for the demo
#[derive(Debug, Clone)]
struct VisualElement {
    id: RecordId,
    bounds: Option<Bounds>,
    position: [f32; 2],
    size: [f32; 2],
    color: [u8; 4],
    name: String,
}

impl VisualElement {
    fn new(
        id: RecordId,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: [u8; 4],
        name: &str,
    ) -> Self {
        Self {
            id,
            bounds: Some(Bounds {
                min_x: x as f64,
                min_y: y as f64,
                max_x: (x + width) as f64,
                max_y: (y + height) as f64,
            }),
            position: [x, y],
            size: [width, height],
            color,
            name: name.to_string(),
        }
    }
}

impl Record for VisualElement {
    fn id(&self) -> &RecordId {
        &self.id
    }

    fn type_name(&self) -> &'static str {
        "VisualElement"
    }

    fn bounds(&self) -> Option<Bounds> {
        self.bounds.clone()
    }
}

// ============================================================
// DEMO FUNCTIONS
// ============================================================

/// Run the complete demo workflow
pub fn run_demo() {
    println!("\n{}", "=".repeat(70));
    println!("  ArchFlow Complete Workflow Demo");
    println!("{}", "=".repeat(70));

    // Phase 1: Record Store Setup
    println!("\n[Phase 1] Creating Record Store...");
    let mut store: RecordStore<VisualElement> = RecordStore::new();
    println!("  ✓ RecordStore created");

    // Phase 2: Create Visual Elements
    println!("\n[Phase 2] Creating visual elements...");
    let elements = [
        ("Background", 0.0, 0.0, 800.0, 600.0, [30, 30, 40, 255]),
        ("Header", 0.0, 0.0, 800.0, 80.0, [60, 60, 80, 255]),
        ("Sidebar", 0.0, 80.0, 200.0, 520.0, [45, 45, 55, 255]),
        ("Content Area", 200.0, 80.0, 600.0, 520.0, [35, 35, 45, 255]),
        ("Button 1", 220.0, 100.0, 120.0, 40.0, [70, 130, 180, 255]),
        ("Button 2", 350.0, 100.0, 120.0, 40.0, [70, 130, 180, 255]),
        ("Status Bar", 0.0, 580.0, 800.0, 20.0, [25, 25, 35, 255]),
    ];

    for (i, (name, x, y, w, h, color)) in elements.iter().enumerate() {
        let id = RecordId::from_u64(i as u64);
        let element = VisualElement::new(id.clone(), *x, *y, *w, *h, *color, name);
        store.put(element);
        println!("  ✓ Created: {} ({}, {} - {}x{})", name, x, y, w, h);
    }

    let changeset = store.drain_changes();
    println!(
        "  Total elements created: {}",
        changeset.created_indices().count()
    );

    // Phase 3: Simulate User Interactions
    println!("\n[Phase 3] Simulating user interactions...");
    let interactions = [
        (4, [230.0, 110.0]), // Move Button 1
        (5, [360.0, 110.0]), // Move Button 2
    ];

    for (idx, new_pos) in &interactions {
        let id = RecordId::from_u64(*idx as u64);
        if let Some(element) = store.get(&id) {
            // Clone needed values before mutable borrow
            let name = element.name.clone();
            let size = element.size;
            let color = element.color;

            let updated = VisualElement::new(
                id.clone(),
                new_pos[0],
                new_pos[1],
                size[0],
                size[1],
                color,
                &name,
            );
            store.put(updated);
            println!("  ✓ Moved '{}' to ({}, {})", name, new_pos[0], new_pos[1]);
        }
    }

    let changes = store.drain_changes();
    println!("  Elements updated: {}", changes.updated_indices().count());

    // Phase 4: Export to SharedBuffer
    println!("\n[Phase 4] Exporting to SharedBuffer...");
    let mut buffer = SharedBuffer::new(100);

    let ids: Vec<u64> = (0..elements.len()).map(|i| i as u64).collect();
    let get_element = |id: u64| {
        store
            .get(&RecordId::from_u64(id))
            .map(|e| (e.position[0], e.position[1], e.color))
    };

    buffer.update(&ids, &get_element);
    println!("  ✓ Exported {} elements to SharedBuffer", buffer.len());

    // Phase 5: Generate Binary Deltas
    println!("\n[Phase 5] Generating binary deltas for sync...");
    let mut delta_batch = Vec::new();

    for i in 0..elements.len() {
        if let Some(element) = store.get(&RecordId::from_u64(i as u64)) {
            BinaryDeltaCodec::encode_delta(
                &mut delta_batch,
                i as u64,
                ShapeField::Position as u8 | ShapeField::Color as u8,
                Some((element.position[0], element.position[1])),
                Some((
                    element.color[0],
                    element.color[1],
                    element.color[2],
                    element.color[3],
                )),
                None,
            );
        }
    }

    println!("  ✓ Generated {} bytes of delta data", delta_batch.len());

    // Phase 6: Simulate delta decoding (JavaScript side would do this)
    println!("\n[Phase 6] Simulating delta decoding...");
    let mut decoded_count = 0;
    let mut offset = 0;

    while offset < delta_batch.len() {
        if let Some(decoded) = BinaryDeltaCodec::decode_delta(&delta_batch[offset..]) {
            if decoded.position.is_some() || decoded.color.is_some() {
                decoded_count += 1;
            }
            // Calculate approximate length for next iteration
            let len = 1 + 8 + 1 + // id (varint) + mask
                if decoded.mask & ShapeField::Position as u8 != 0 { 8 } else { 0 } +
                if decoded.mask & ShapeField::Color as u8 != 0 { 4 } else { 0 } +
                if decoded.mask & ShapeField::Size as u8 != 0 { 8 } else { 0 };
            offset += len;
        } else {
            break;
        }
    }

    println!("  ✓ Decoded {} deltas", decoded_count);

    // Phase 7: Performance Summary
    println!("\n[Phase 7] Performance Summary");
    println!("  - Total records: {}", store.len());
    println!("  - SharedBuffer capacity: {}", 100);
    println!("  - Delta batch size: {} bytes", delta_batch.len());
    println!(
        "  - Updates generated: {}",
        changes.updated_indices().count()
    );

    // Summary
    println!("\n{}", "=".repeat(70));
    println!("  Demo completed successfully!");
    println!("  The workflow demonstrates:");
    println!("    1. RecordStore CRUD operations");
    println!("    2. ChangeSet tracking (created/updated)");
    println!("    3. SharedBuffer export for WASM");
    println!("    4. Binary delta encoding for sync");
    println!("{}", "=".repeat(70));
    println!();
}

// ============================================================
// PERFORMANCE BENCHMARK DEMO
// ============================================================

/// Run a performance benchmark
pub fn run_performance_benchmark() {
    println!("\n{}", "=".repeat(70));
    println!("  ArchFlow Performance Benchmark");
    println!("{}", "=".repeat(70));

    const RECORD_COUNTS: [usize; 3] = [1000, 5000, 10000];
    const UPDATE_RATIOS: [f64; 3] = [0.01, 0.05, 0.10];

    for &record_count in &RECORD_COUNTS {
        println!("\n📊 Testing with {} records:", record_count);

        for &update_ratio in &UPDATE_RATIOS {
            let update_count = (record_count as f64 * update_ratio) as usize;

            // Benchmark: Insert
            let insert_start = Instant::now();
            let mut store: RecordStore<VisualElement> = RecordStore::new();
            for i in 0..record_count {
                let id = RecordId::from_u64(i as u64);
                let element = VisualElement::new(
                    id,
                    i as f32 % 100.0,
                    i as f32 / 100.0,
                    20.0,
                    20.0,
                    [128, 128, 128, 255],
                    &format!("Element {}", i),
                );
                store.put(element);
            }
            let insert_time = insert_start.elapsed();

            // Benchmark: Update
            let update_start = Instant::now();
            for i in 0..update_count {
                let id = RecordId::from_u64(i as u64);
                if let Some(element) = store.get(&id) {
                    let updated = VisualElement::new(
                        id.clone(),
                        element.position[0] + 1.0,
                        element.position[1],
                        element.size[0],
                        element.size[1],
                        element.color,
                        &element.name,
                    );
                    store.put(updated);
                }
            }
            let update_time = update_start.elapsed();

            // Benchmark: Drain Changes
            let drain_start = Instant::now();
            let _changeset = store.drain_changes();
            let drain_time = drain_start.elapsed();

            // Benchmark: Export to Buffer
            let export_start = Instant::now();
            let mut buffer = SharedBuffer::new(record_count);
            let ids: Vec<u64> = (0..record_count).map(|i| i as u64).collect();
            let get_element = |id: u64| {
                store
                    .get(&RecordId::from_u64(id))
                    .map(|e| (e.position[0], e.position[1], e.color))
            };
            buffer.update(&ids, &get_element);
            let export_time = export_start.elapsed();

            println!(
                "  [{:>5.1}% updates] Insert: {:>6.2}ms | Update: {:>6.2}ms | Drain: {:>5.2}ms | Export: {:>6.2}ms",
                update_ratio * 100.0,
                insert_time.as_secs_f64() * 1000.0,
                update_time.as_secs_f64() * 1000.0,
                drain_time.as_secs_f64() * 1000.0,
                export_time.as_secs_f64() * 1000.0,
            );
        }
    }

    println!("\n{}", "=".repeat(70));
    println!("  Benchmark completed!");
    println!("{}", "=".repeat(70));
    println!();
}

// ============================================================
// ENTRY POINT
// ============================================================

fn main() {
    // Run the main demo
    run_demo();

    // Run performance benchmark
    run_performance_benchmark();
}
