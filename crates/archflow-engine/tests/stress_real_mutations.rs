//! Real stress tests with actual mutations using archflow-engine API
//!
//! Run with: cargo test -p archflow-engine --test stress_real_mutations --release -- --test-threads=1
//!
//! These tests verify performance with real mutations: position changes, size changes, hierarchy.

use archflow_core::Vec2;
use archflow_engine::EntityStore;
use rand::prelude::*;

/// Configuration
const STRESS_ENTITY_COUNT: usize = 100_000;
const SEED: u64 = 0xCAFEBABEDEADBEEF;

/// Helper to create seeded RNG
fn seeded_rng() -> StdRng {
    StdRng::seed_from_u64(SEED)
}

/// Helper to create deterministic entity positions
fn deterministic_position(index: usize) -> Vec2 {
    let row = (index / 500) as f32;
    let col = (index % 500) as f32;
    Vec2::new(col * 20.0, row * 20.0)
}

// ============================================================================
// Position Mutation Stress Tests
// ============================================================================

#[test]
#[ignore]
fn stress_position_mutation_100k() {
    // Test: Move ALL 100k entities every frame (simulates camera pan)
    let mut store = EntityStore::new();

    // Spawn entities
    let start = std::time::Instant::now();
    for i in 0..STRESS_ENTITY_COUNT {
        store.spawn(deterministic_position(i), Vec2::new(50.0, 50.0));
    }
    let spawn_time = start.elapsed();

    // Get all indices for batch mutation
    let indices: Vec<usize> = (0..STRESS_ENTITY_COUNT).collect();

    // Simulate 60 FPS camera pan - move all entities
    let move_start = std::time::Instant::now();
    let delta = Vec2::new(1.0, 0.5);

    for frame in 0..100 {
        for &idx in &indices {
            store.move_by(idx, delta);

            // Verify entity still valid after mutation
            debug_assert!(store.pos(idx).x >= 0.0);
        }
    }
    let move_time = move_start.elapsed();

    let total_time = spawn_time + move_time;
    let moves_per_sec = (STRESS_ENTITY_COUNT * 100) as f64 / move_time.as_secs_f64();

    println!("[STRESS POSITION 100k]");
    println!("  Spawn:     {:.2}s", spawn_time.as_secs_f64());
    println!("  100 frames x 100k moves: {:.2}s", move_time.as_secs_f64());
    println!("  Throughput: {:.0} moves/sec", moves_per_sec);
    println!("  Total:     {:.2}s", total_time.as_secs_f64());

    // Performance assertions
    assert!(
        moves_per_sec > 1_000_000.0,
        "Move throughput too low: {:.0}/sec",
        moves_per_sec
    );
    assert!(
        move_time.as_secs_f64() < 30.0,
        "Position mutations too slow: {}s",
        move_time.as_secs_f64()
    );
}

#[test]
#[ignore]
fn stress_set_position_100k() {
    // Test: Set absolute position for ALL entities (simulates viewport warp)
    let mut store = EntityStore::new();

    for i in 0..STRESS_ENTITY_COUNT {
        store.spawn(deterministic_position(i), Vec2::new(50.0, 50.0));
    }

    let start = std::time::Instant::now();
    let indices: Vec<usize> = (0..STRESS_ENTITY_COUNT).collect();
    let mut rng = seeded_rng();

    // Set random positions 10 times
    for frame in 0..10 {
        for &idx in &indices {
            let new_pos = Vec2::new(
                rng.gen_range(-10000.0..10000.0),
                rng.gen_range(-10000.0..10000.0),
            );
            store.set_pos(idx, new_pos);
        }
    }
    let elapsed = start.elapsed();

    let total_sets = STRESS_ENTITY_COUNT * 10;
    let rate = total_sets as f64 / elapsed.as_secs_f64();

    println!("[STRESS SET_POS 100k]");
    println!(
        "  {} set_pos calls in {:.2}s ({:.0}/sec)",
        total_sets,
        elapsed.as_secs_f64(),
        rate
    );

    assert!(rate > 500_000.0, "set_pos rate too low: {:.0}/sec", rate);
}

// ============================================================================
// Size Mutation Stress Tests
// ============================================================================

#[test]
#[ignore]
fn stress_size_mutation_100k() {
    // Test: Resize animation for ALL entities (simulates zoom effect)
    let mut store = EntityStore::new();

    for i in 0..STRESS_ENTITY_COUNT {
        store.spawn(deterministic_position(i), Vec2::new(50.0, 50.0));
    }

    let indices: Vec<usize> = (0..STRESS_ENTITY_COUNT).collect();
    let mut rng = seeded_rng();

    let start = std::time::Instant::now();

    // Zoom in/out animation
    for frame in 0..60 {
        let scale = 1.0 + (frame as f32 / 60.0).sin() * 0.5;
        let new_size = Vec2::new(50.0 * scale, 50.0 * scale);

        for &idx in &indices {
            store.set_size(idx, new_size);
        }

        if frame % 15 == 0 {
            let random_size = Vec2::new(rng.gen_range(10.0..100.0), rng.gen_range(10.0..100.0));
        }
    }
    let elapsed = start.elapsed();

    let total_changes = STRESS_ENTITY_COUNT * 60;
    let rate = total_changes as f64 / elapsed.as_secs_f64();

    println!("[STRESS SIZE 100k]");
    println!(
        "  {} size changes in {:.2}s ({:.0}/sec)",
        total_changes,
        elapsed.as_secs_f64(),
        rate
    );

    assert!(
        rate > 500_000.0,
        "Size mutation rate too low: {:.0}/sec",
        rate
    );
}

// ============================================================================
// Combined Position + Size Stress Tests
// ============================================================================

#[test]
#[ignore]
fn stress_position_and_size_100k() {
    // Test: Simultaneous position + size changes (simulates game loop update)
    let mut store = EntityStore::new();

    for i in 0..STRESS_ENTITY_COUNT {
        store.spawn(deterministic_position(i), Vec2::new(50.0, 50.0));
    }

    let indices: Vec<usize> = (0..STRESS_ENTITY_COUNT).collect();
    let mut rng = seeded_rng();

    let start = std::time::Instant::now();

    for frame in 0..30 {
        let delta = Vec2::new(rng.gen_range(-5.0..5.0), rng.gen_range(-3.0..3.0));
        let new_size = Vec2::new(rng.gen_range(20.0..80.0), rng.gen_range(20.0..80.0));

        for &idx in &indices {
            store.move_by(idx, delta);
            store.set_size(idx, new_size);
        }
    }
    let elapsed = start.elapsed();

    let total_ops = STRESS_ENTITY_COUNT * 30 * 2;
    let rate = total_ops as f64 / elapsed.as_secs_f64();

    println!("[STRESS POS+SIZE 100k]");
    println!(
        "  {} combined ops in {:.2}s ({:.0}/sec)",
        total_ops,
        elapsed.as_secs_f64(),
        rate
    );

    assert!(
        rate > 500_000.0,
        "Combined ops rate too low: {:.0}/sec",
        rate
    );
}

// ============================================================================
// Dirty Flag Tracking Stress Tests
// ============================================================================

#[test]
#[ignore]
fn stress_dirty_flags_100k() {
    let mut store = EntityStore::new();

    for i in 0..STRESS_ENTITY_COUNT {
        store.spawn(deterministic_position(i), Vec2::new(50.0, 50.0));
    }

    let indices: Vec<usize> = (0..STRESS_ENTITY_COUNT).collect();

    let start = std::time::Instant::now();
    for (i, &idx) in indices.iter().enumerate() {
        if i % 2 == 0 {
            store.move_by(idx, Vec2::new(1.0, 1.0));
        }
    }
    let elapsed = start.elapsed();

    let dirty_count = store.dirty_render_count();

    println!("[STRESS DIRTY FLAGS 100k]");
    println!(
        "  Modified {} entities in {:.2}s",
        STRESS_ENTITY_COUNT / 2,
        elapsed.as_secs_f64()
    );
    println!("  Dirty entities detected: {}", dirty_count);

    assert_eq!(dirty_count, STRESS_ENTITY_COUNT / 2);

    store.clear_dirty_flags();
    assert_eq!(store.dirty_render_count(), 0);

    println!("  Dirty flags cleared successfully");
}

#[test]
#[ignore]
fn stress_take_dirty_entities() {
    let mut store = EntityStore::new();

    for i in 0..50_000 {
        store.spawn(deterministic_position(i), Vec2::new(50.0, 50.0));
    }

    for idx in 0..50_000 {
        store.move_by(idx, Vec2::new(1.0, 1.0));
    }

    let start = std::time::Instant::now();
    let dirty_entities: Vec<usize> = store.take_dirty_render_entities().collect();
    let elapsed = start.elapsed();

    println!("[STRESS TAKE DIRTY]");
    println!(
        "  Extracted {} dirty entities in {:.4}s",
        dirty_entities.len(),
        elapsed.as_secs_f64()
    );

    assert_eq!(dirty_entities.len(), 50_000);
    assert!(
        elapsed.as_secs_f64() < 0.1,
        "Take dirty too slow: {:.4}s",
        elapsed.as_secs_f64()
    );
}

// ============================================================================
// Random Access Pattern Stress Tests
// ============================================================================

#[test]
#[ignore]
fn stress_random_access_pattern() {
    let mut store = EntityStore::new();

    for i in 0..STRESS_ENTITY_COUNT {
        store.spawn(deterministic_position(i), Vec2::new(50.0, 50.0));
    }

    let mut rng = seeded_rng();
    let start = std::time::Instant::now();

    let mut operations = 0;
    let mut mutations = 0;

    for frame in 0..60 {
        let query_count = rng.gen_range(100..1000);
        let mut queried_indices = Vec::new();

        for _ in 0..query_count {
            let idx = rng.gen_range(0..STRESS_ENTITY_COUNT);
            let pos = store.pos(idx);

            if pos.x > 5000.0 && pos.y > 5000.0 {
                store.set_pos(idx, Vec2::new(pos.x + 1.0, pos.y + 1.0));
                mutations += 1;
            }

            queried_indices.push(idx);
            operations += 1;
        }

        for &idx in &queried_indices {
            let _ = store.pos(idx);
        }
    }

    let elapsed = start.elapsed();
    let rate = operations as f64 / elapsed.as_secs_f64();

    println!("[STRESS RANDOM ACCESS 100k]");
    println!(
        "  {} operations in {:.2}s ({:.0}/sec)",
        operations,
        elapsed.as_secs_f64(),
        rate
    );
    println!("  {} mutations performed", mutations);

    assert!(
        rate > 50_000.0,
        "Random access rate too low: {:.0}/sec",
        rate
    );
}

// ============================================================================
// Throughput Benchmark
// ============================================================================

#[test]
#[ignore]
fn stress_throughput_benchmark() {
    let mut store = EntityStore::new();

    for i in 0..STRESS_ENTITY_COUNT {
        store.spawn(deterministic_position(i), Vec2::new(50.0, 50.0));
    }

    let indices: Vec<usize> = (0..STRESS_ENTITY_COUNT).collect();

    let start = std::time::Instant::now();
    let mut total_ops = 0u64;

    let deadline = start.elapsed() + std::time::Duration::from_secs(5);

    let mut frame = 0;
    while start.elapsed() < deadline {
        let op = frame % 4;

        match op {
            0 => {
                for &idx in &indices[..1000] {
                    store.move_by(idx, Vec2::new(1.0, 0.5));
                    total_ops += 1;
                }
            }
            1 => {
                for &idx in &indices[..1000] {
                    store.set_size(idx, Vec2::new(50.0, 50.0));
                    total_ops += 1;
                }
            }
            2 => {
                for &idx in &indices[..1000] {
                    let _ = store.pos(idx);
                    total_ops += 1;
                }
            }
            _ => {
                for (i, &idx) in indices[..500].iter().enumerate() {
                    if i % 2 == 0 {
                        store.move_by(idx, Vec2::new(0.5, 0.25));
                    } else {
                        store.set_size(idx, Vec2::new(40.0, 40.0));
                    }
                    total_ops += 1;
                }
            }
        }

        frame += 1;
    }

    let elapsed = start.elapsed();
    let throughput = total_ops as f64 / elapsed.as_secs_f64();

    println!("[STRESS THROUGHPUT BENCHMARK]");
    println!("  Duration:   {:.2}s", elapsed.as_secs_f64());
    println!("  Total ops: {}", total_ops);
    println!("  Throughput: {:.0} ops/sec", throughput);
    println!(
        "  Per entity: {:.1} ops/sec per entity",
        throughput / STRESS_ENTITY_COUNT as f64
    );

    assert!(
        throughput > 1_000_000.0,
        "Throughput too low: {:.0}/sec",
        throughput
    );
}

// ============================================================================
// Query Performance Tests
// ============================================================================

#[test]
#[ignore]
fn stress_query_performance() {
    // Test: Position queries under heavy load
    let mut store = EntityStore::new();

    for i in 0..STRESS_ENTITY_COUNT {
        store.spawn(deterministic_position(i), Vec2::new(50.0, 50.0));
    }

    let mut rng = seeded_rng();
    let start = std::time::Instant::now();

    // Query random entities 100k times
    let mut total_reads = 0;
    for _ in 0..100_000 {
        let idx = rng.gen_range(0..STRESS_ENTITY_COUNT);
        let _ = store.pos(idx);
        let _ = store.size(idx);
        total_reads += 2;
    }

    let elapsed = start.elapsed();
    let rate = total_reads as f64 / elapsed.as_secs_f64();

    println!("[STRESS QUERY PERFORMANCE]");
    println!(
        "  {} position+size reads in {:.2}s ({:.0}/sec)",
        total_reads,
        elapsed.as_secs_f64(),
        rate
    );

    assert!(rate > 1_000_000.0, "Query rate too low: {:.0}/sec", rate);
}
