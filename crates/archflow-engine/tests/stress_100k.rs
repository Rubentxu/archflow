//! Stress tests with 100k entities for archflow-engine
//!
//! Run with: cargo test -p archflow-engine --test stress_100k --release -- --test-threads=1
//!
//! These tests verify performance and stability under heavy load.

use archflow_core::Vec2;
use archflow_engine::store::EntityStore;
use rand::prelude::*;

/// Configuration for stress tests
const STRESS_ENTITY_COUNT: usize = 100_000;
const SEED: u64 = 0xDEADBEEFCAFEBABE;

/// Helper to create seeded RNG
fn seeded_rng() -> StdRng {
    StdRng::seed_from_u64(SEED)
}

/// Helper to create deterministic entity positions
fn deterministic_position(index: usize) -> Vec2 {
    let row = (index / 1000) as f32;
    let col = (index % 1000) as f32;
    Vec2::new(col * 15.0, row * 15.0)
}

// ============================================================================
// Entity Store Stress Tests
// ============================================================================

#[test]
#[ignore]
fn stress_spawn_100k_entities() {
    let start = std::time::Instant::now();

    let mut store = EntityStore::new();
    let mut ids = Vec::with_capacity(STRESS_ENTITY_COUNT);

    for i in 0..STRESS_ENTITY_COUNT {
        let pos = deterministic_position(i);
        let id = store.spawn(pos, Vec2::new(50.0, 50.0));
        ids.push(id);
        assert!(store.is_alive(id), "Entity {} should be alive", i);
    }

    let elapsed = start.elapsed();
    let rate = STRESS_ENTITY_COUNT as f64 / elapsed.as_secs_f64();

    println!(
        "[STRESS] Spawned {} entities in {:.2}s ({:.0} entities/sec)",
        STRESS_ENTITY_COUNT,
        elapsed.as_secs_f64(),
        rate
    );

    assert!(rate > 10_000.0, "Spawn rate too low: {:.0}/sec", rate);
    assert_eq!(store.alive_count(), STRESS_ENTITY_COUNT);
}

#[test]
#[ignore]
fn stress_spawn_and_despawn_100k_entities() {
    let mut store = EntityStore::new();
    let mut ids = Vec::with_capacity(STRESS_ENTITY_COUNT);

    let spawn_start = std::time::Instant::now();
    for i in 0..STRESS_ENTITY_COUNT {
        let id = store.spawn(deterministic_position(i), Vec2::new(50.0, 50.0));
        ids.push(id);
    }
    let spawn_time = spawn_start.elapsed();

    let access_start = std::time::Instant::now();
    let mut rng = seeded_rng();
    let mut hits = 0;
    for _ in 0..100_000 {
        let idx = rng.gen_range(0..STRESS_ENTITY_COUNT);
        if store.is_alive(ids[idx]) {
            hits += 1;
        }
    }
    let access_time = access_start.elapsed();

    let despawn_start = std::time::Instant::now();
    let mut cleaned = 0;
    for id in ids {
        store.despawn_with_cleanup(id, |_| {
            cleaned += 1;
        });
    }
    let despawn_time = despawn_start.elapsed();

    assert_eq!(cleaned, STRESS_ENTITY_COUNT);
    assert!(store.is_empty());

    println!(
        "[STRESS] Spawn: {:.2}s, Access: {:.2}s ({} hits), Cleanup: {:.2}s",
        spawn_time.as_secs_f64(),
        access_time.as_secs_f64(),
        hits,
        despawn_time.as_secs_f64()
    );
}

#[test]
#[ignore]
fn stress_random_spawn_despawn_pattern() {
    let mut store = EntityStore::new();
    let mut rng = seeded_rng();
    let mut active_ids = Vec::new();

    let start = std::time::Instant::now();
    let mut total_operations = 0;

    for round in 0..100 {
        if rng.gen_bool(0.7) || active_ids.is_empty() {
            let pos = Vec2::new(rng.gen_range(0.0..10000.0), rng.gen_range(0.0..10000.0));
            let id = store.spawn(pos, Vec2::new(50.0, 50.0));
            active_ids.push(id);
        } else {
            let idx = rng.gen_range(0..active_ids.len());
            let id = active_ids.remove(idx);
            store.despawn(id);
        }

        total_operations += 1;

        if round % 10 == 0 {
            assert_eq!(store.alive_count(), active_ids.len());
        }
    }

    let elapsed = start.elapsed();
    let rate = total_operations as f64 / elapsed.as_secs_f64();

    println!(
        "[STRESS] {} operations in {:.2}s ({:.0} ops/sec)",
        total_operations,
        elapsed.as_secs_f64(),
        rate
    );

    for id in active_ids {
        store.despawn(id);
    }
}

// ============================================================================
// Memory Stress Tests
// ============================================================================

#[test]
#[ignore]
fn stress_memory_cleanup_verification() {
    let mut store = EntityStore::new();
    let cleanup_count = std::cell::Cell::new(0);
    let mut ids = Vec::new();

    for i in 0..10_000 {
        let id = store.spawn(deterministic_position(i), Vec2::new(50.0, 50.0));
        ids.push(id);
    }

    for id in ids {
        store.despawn_with_cleanup(id, |_| {
            cleanup_count.set(cleanup_count.get() + 1);
        });
    }

    assert_eq!(cleanup_count.get(), 10_000);
    assert!(store.is_empty());

    println!(
        "[STRESS] Cleanup verified: {} entities",
        cleanup_count.get()
    );
}

#[test]
#[ignore]
fn stress_id_reuse_pattern() {
    let mut store = EntityStore::new();
    let mut first_ids = Vec::new();

    for _ in 0..3 {
        for i in 0..1_000 {
            let id = store.spawn(Vec2::new(i as f32 * 10.0, 0.0), Vec2::new(50.0, 50.0));
            first_ids.push(id);
        }
        for id in first_ids.drain(..) {
            store.despawn(id);
        }
    }

    let new_id = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));

    assert_eq!(store.alive_count(), 1);
    assert!(store.is_alive(new_id));

    println!("[STRESS] ID reuse: system maintains integrity");
}

// ============================================================================
// Combined Stress Tests
// ============================================================================

#[test]
#[ignore]
fn stress_full_system_100k() {
    let mut store = EntityStore::new();
    let mut ids = Vec::with_capacity(STRESS_ENTITY_COUNT);

    let spawn_start = std::time::Instant::now();
    for i in 0..STRESS_ENTITY_COUNT {
        let pos = deterministic_position(i);
        let id = store.spawn(pos, Vec2::new(50.0, 50.0));
        ids.push(id);
    }
    let spawn_time = spawn_start.elapsed();

    let access_start = std::time::Instant::now();
    let mut rng = seeded_rng();
    let mut selected_count = 0;
    for _ in 0..10_000 {
        let idx = rng.gen_range(0..STRESS_ENTITY_COUNT);
        if store.is_alive(ids[idx]) {
            selected_count += 1;
        }
    }
    let access_time = access_start.elapsed();

    let cleanup_start = std::time::Instant::now();
    let mut cleaned = 0;
    for id in ids {
        store.despawn_with_cleanup(id, |_| {
            cleaned += 1;
        });
    }
    let cleanup_time = cleanup_start.elapsed();

    assert_eq!(cleaned, STRESS_ENTITY_COUNT);
    assert!(store.is_empty());

    println!("[STRESS FULL SYSTEM 100k]");
    println!("  Spawn:   {:.2}s", spawn_time.as_secs_f64());
    println!(
        "  Access: {:.2}s ({})",
        access_time.as_secs_f64(),
        selected_count
    );
    println!("  Cleanup: {:.2}s", cleanup_time.as_secs_f64());

    assert!(spawn_time.as_secs_f64() < 30.0, "Spawn too slow");
    assert!(cleanup_time.as_secs_f64() < 30.0, "Cleanup too slow");
}
