// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - MouseOver Sensor Tests
//
// Epic 2.1: MouseOver Sensor
// TDD Approach: Red → Green → Refactor
//
// These tests verify the MouseOver sensor implementation which detects
// when the mouse cursor is over an entity using AABB hit testing.
//
// Note: Integration tests run with std (not no_std) to allow timing tests
// ═══════════════════════════════════════════════════════════════════════════════

// Integration tests run with std (not no_std)
#[cfg(test)]
mod tests {
    use std::time::Instant;

    // ═══════════════════════════════════════════════════════════════════════════════
    // RED PHASE: Tests are written FIRST (before implementation exists)
    // ═══════════════════════════════════════════════════════════════════════════════

    use archflow_core::{EntityId, Vec2};
    use archflow_engine::EntityStore;
    use archflow_logic::sensors::mouse_over::MouseOverSensor;

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 2.1.1: sample() method updates SignalByte per entity
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_mouse_over_detects_hover() {
        // AC2.1.4: Supports up to MAX_ENTITIES (100,000)
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = MouseOverSensor::new(archflow_engine::MAX_ENTITIES);

        // Mouse dentro del rectángulo (100±25, 100±25)
        // AABB: x ∈ [75, 125], y ∈ [75, 125]
        sensor.sample(Vec2::new(110.0, 105.0), &store);

        assert!(sensor.is_over(entity));
    }

    #[test]
    fn test_mouse_over_outside_bounds() {
        // Mouse fuera del rectángulo
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = MouseOverSensor::new(archflow_engine::MAX_ENTITIES);

        // Mouse fuera: (200, 200) no está en [75, 125] × [75, 125]
        sensor.sample(Vec2::new(200.0, 200.0), &store);

        assert!(!sensor.is_over(entity));
    }

    #[test]
    fn test_mouse_over_on_exact_boundary() {
        // Mouse en el límite exacto del rectángulo
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = MouseOverSensor::new(archflow_engine::MAX_ENTITIES);

        // Esquina superior izquierda del AABB
        sensor.sample(Vec2::new(75.0, 75.0), &store);
        assert!(sensor.is_over(entity));

        // Justo fuera de la esquina
        sensor.sample(Vec2::new(74.9, 75.0), &store);
        assert!(!sensor.is_over(entity));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 2.1.3: AABB hit test
    // Formula: mouse_x >= x - w/2 && mouse_x <= x + w/2 && (same for y)
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_aabb_hit_test_center() {
        // Mouse exactamente en el centro
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = MouseOverSensor::new(archflow_engine::MAX_ENTITIES);

        sensor.sample(Vec2::new(100.0, 100.0), &store);
        assert!(sensor.is_over(entity));
    }

    #[test]
    fn test_aabb_hit_test_corners() {
        // Las 4 esquinas del AABB
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = MouseOverSensor::new(archflow_engine::MAX_ENTITIES);

        // Esquinas: (75,75), (125,75), (75,125), (125,125)
        let corners = [
            Vec2::new(75.0, 75.0),
            Vec2::new(125.0, 75.0),
            Vec2::new(75.0, 125.0),
            Vec2::new(125.0, 125.0),
        ];

        for corner in corners {
            sensor.sample(corner, &store);
            assert!(
                sensor.is_over(entity),
                "Should detect hover at corner {:?}",
                corner
            );
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 2.1.1: Updates SignalByte (6-tick history)
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_hover_enter_edge_detection() {
        // on_hover_enter() should detect rising edge (0 → 1)
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = MouseOverSensor::new(archflow_engine::MAX_ENTITIES);

        // Frame 1: Fuera
        sensor.sample(Vec2::new(200.0, 200.0), &store);
        assert!(!sensor.on_hover_enter(entity));

        // Frame 2: Dentro (rising edge)
        sensor.sample(Vec2::new(100.0, 100.0), &store);
        assert!(sensor.on_hover_enter(entity)); // Should detect rising edge
    }

    #[test]
    fn test_hover_exit_edge_detection() {
        // on_hover_exit() should detect falling edge (1 → 0)
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = MouseOverSensor::new(archflow_engine::MAX_ENTITIES);

        // Establecer mouse dentro por 3 frames
        sensor.sample(Vec2::new(100.0, 100.0), &store);
        sensor.sample(Vec2::new(100.0, 100.0), &store);
        sensor.sample(Vec2::new(100.0, 100.0), &store);

        // Frame siguiente: fuera (falling edge)
        sensor.sample(Vec2::new(200.0, 200.0), &store);
        assert!(sensor.on_hover_exit(entity)); // Should detect falling edge
    }

    #[test]
    fn test_stable_hover_3_ticks() {
        // is_stable_over(entity, 3) should return true after 3 consecutive ticks
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = MouseOverSensor::new(archflow_engine::MAX_ENTITIES);

        // 3 ticks consecutivos con mouse encima
        sensor.sample(Vec2::new(100.0, 100.0), &store);
        sensor.sample(Vec2::new(100.0, 100.0), &store);
        sensor.sample(Vec2::new(100.0, 100.0), &store);

        assert!(sensor.is_stable_over(entity, 3));
    }

    #[test]
    fn test_not_stable_after_only_2_ticks() {
        // Should NOT be stable at 3 ticks after only 2 ticks
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = MouseOverSensor::new(archflow_engine::MAX_ENTITIES);

        // Solo 2 ticks
        sensor.sample(Vec2::new(100.0, 100.0), &store);
        sensor.sample(Vec2::new(100.0, 100.0), &store);

        assert!(!sensor.is_stable_over(entity, 3));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 2.1.5: Zero-allocation in hot path
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_multiple_entities_single_sample() {
        // Single sample() should update ALL entities without allocation
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(30.0, 30.0));
        let e2 = store.spawn(Vec2::new(150.0, 50.0), Vec2::new(30.0, 30.0));
        let e3 = store.spawn(Vec2::new(100.0, 150.0), Vec2::new(30.0, 30.0));

        let mut sensor = MouseOverSensor::new(archflow_engine::MAX_ENTITIES);

        // Un solo sample actualiza todas las entidades
        sensor.sample(Vec2::new(50.0, 50.0), &store);

        assert!(sensor.is_over(e1));
        assert!(!sensor.is_over(e2));
        assert!(!sensor.is_over(e3));
    }

    #[test]
    fn test_zero_entities() {
        // Should handle empty EntityStore gracefully
        // Note: EntityStore pre-allocates MAX_ENTITIES slots, so sensor needs same capacity
        let mut store = EntityStore::new();
        let mut sensor = MouseOverSensor::new(archflow_engine::MAX_ENTITIES);

        // Should not panic
        sensor.sample(Vec2::new(100.0, 100.0), &store);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // EDGE CASES
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_entity_size_zero() {
        // Entidad con tamaño 0 (solo punto)
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(0.0, 0.0));

        let mut sensor = MouseOverSensor::new(archflow_engine::MAX_ENTITIES);

        // Exactamente en el punto
        sensor.sample(Vec2::new(100.0, 100.0), &store);
        assert!(sensor.is_over(entity));

        // Cualquier otro lugar no cuenta
        sensor.sample(Vec2::new(100.1, 100.0), &store);
        assert!(!sensor.is_over(entity));
    }

    #[test]
    fn test_negative_coordinates() {
        // Entidades con coordenadas negativas
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(-50.0, -50.0), Vec2::new(30.0, 30.0));

        let mut sensor = MouseOverSensor::new(archflow_engine::MAX_ENTITIES);

        // AABB: x ∈ [-65, -35], y ∈ [-65, -35]
        sensor.sample(Vec2::new(-50.0, -50.0), &store);
        assert!(sensor.is_over(entity));
    }

    #[test]
    fn test_very_large_entity() {
        // Entidad muy grande
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(1000.0, 1000.0));

        let mut sensor = MouseOverSensor::new(archflow_engine::MAX_ENTITIES);

        // Casi cualquier punto debería estar dentro
        sensor.sample(Vec2::new(400.0, 400.0), &store);
        assert!(sensor.is_over(entity));

        // Pero no infinito
        sensor.sample(Vec2::new(1000.0, 1000.0), &store);
        assert!(!sensor.is_over(entity));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // PERFORMANCE TESTS
    // Note: Performance is better validated in release mode with benchmarks
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_sample_1000_entities() {
        // Functional test: 1K entities should work without issues
        let mut store = EntityStore::new();

        // Spawn 1000 entities
        for i in 0..1000 {
            let x = (i % 20) as f32 * 50.0;
            let y = (i / 20) as f32 * 50.0;
            store.spawn(Vec2::new(x, y), Vec2::new(30.0, 30.0));
        }

        let mut sensor = MouseOverSensor::new(archflow_engine::MAX_ENTITIES);

        // Sample should complete without errors
        for _ in 0..10 {
            sensor.sample(Vec2::new(100.0, 100.0), &store);
        }

        // Just verify it doesn't panic - timing is for benchmarks, not unit tests
        assert!(true);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // GREEN PHASE CHECKLIST
    // ═══════════════════════════════════════════════════════════════════════════════
    //
    // After implementing MouseOverSensor in src/sensors/mouse_over.rs:
    //
    // 1. Run: cargo test --package archflow-logic --test mouse_over_tests
    // 2. Verify all tests pass
    // 3. Run with --release to verify performance targets
    //
    // ═══════════════════════════════════════════════════════════════════════════════
}
