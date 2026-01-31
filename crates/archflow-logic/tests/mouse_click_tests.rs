// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - MouseClick Sensor Tests
//
// Epic 2.2: MouseClick Sensor
// TDD Approach: Red → Green → Refactor
//
// These tests verify the MouseClick sensor implementation which detects
// mouse button clicks on entities using AABB hit testing combined with
// button state tracking.
//
// Note: Integration tests run with std (not no_std)
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    // ═══════════════════════════════════════════════════════════════════════════════
    // RED PHASE: Tests are written FIRST (before implementation exists)
    // ═══════════════════════════════════════════════════════════════════════════════

    use archflow_core::Vec2;
    use archflow_engine::EntityStore;
    use archflow_logic::sensors::mouse_click::{MouseClickSensor, PointerButtons};

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 2.2.1: Combina MouseOver + button_state
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_click_only_when_over_and_pressed() {
        // AC2.2.1: Click debe detectarse solo cuando mouse está sobre la entidad Y botón presionado
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = MouseClickSensor::new(archflow_engine::MAX_ENTITIES);

        // Mouse dentro + botón primario presionado
        sensor.sample(
            Vec2::new(100.0, 100.0),
            PointerButtons::from_u8(PointerButtons::PRIMARY),
            &store,
        );

        assert!(sensor.on_click(entity));
    }

    #[test]
    fn test_no_click_when_mouse_outside() {
        // AC2.2.1: NO debe detectar click si mouse está fuera, aunque botón esté presionado
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = MouseClickSensor::new(archflow_engine::MAX_ENTITIES);

        // Mouse fuera + botón presionado
        sensor.sample(
            Vec2::new(200.0, 200.0),
            PointerButtons::from_u8(PointerButtons::PRIMARY),
            &store,
        );

        assert!(!sensor.on_click(entity));
    }

    #[test]
    fn test_no_click_when_not_pressed() {
        // AC2.2.1: NO debe detectar click si mouse está sobre entidad pero botón NO está presionado
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = MouseClickSensor::new(archflow_engine::MAX_ENTITIES);

        // Mouse dentro + sin botón presionado
        sensor.sample(Vec2::new(100.0, 100.0), PointerButtons::from_u8(0), &store);

        assert!(!sensor.on_click(entity));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 2.2.2: Soporta botones Primary, Secondary, Middle
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_primary_button_detection() {
        // AC2.2.2: Debe detectar click primario (botón izquierdo)
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = MouseClickSensor::new(archflow_engine::MAX_ENTITIES);

        sensor.sample(
            Vec2::new(100.0, 100.0),
            PointerButtons::from_u8(PointerButtons::PRIMARY),
            &store,
        );

        assert!(sensor.on_click(entity));
        assert!(!sensor.on_right_click(entity));
    }

    #[test]
    fn test_secondary_button_detection() {
        // AC2.2.2: Debe detectar click secundario (botón derecho)
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = MouseClickSensor::new(archflow_engine::MAX_ENTITIES);

        sensor.sample(
            Vec2::new(100.0, 100.0),
            PointerButtons::from_u8(PointerButtons::SECONDARY),
            &store,
        );

        assert!(!sensor.on_click(entity));
        assert!(sensor.on_right_click(entity));
    }

    #[test]
    fn test_middle_button_detection() {
        // AC2.2.2: Debe detectar click medio (rueda del mouse)
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = MouseClickSensor::new(archflow_engine::MAX_ENTITIES);

        sensor.sample(
            Vec2::new(100.0, 100.0),
            PointerButtons::from_u8(PointerButtons::MIDDLE),
            &store,
        );

        // Middle click no tiene método específico, pero se puede verificar que no afecta otros botones
        assert!(!sensor.on_click(entity));
        assert!(!sensor.on_right_click(entity));
    }

    #[test]
    fn test_multiple_buttons_simultaneously() {
        // AC2.2.2: Debe manejar múltiples botones presionados simultáneamente
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = MouseClickSensor::new(archflow_engine::MAX_ENTITIES);

        // Primary + Secondary presionados
        sensor.sample(
            Vec2::new(100.0, 100.0),
            PointerButtons::from_u8(PointerButtons::PRIMARY | PointerButtons::SECONDARY),
            &store,
        );

        assert!(sensor.on_click(entity));
        assert!(sensor.on_right_click(entity));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 2.2.3: on_click() detecta rising edge (press, not hold)
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_on_click_detects_rising_edge_only() {
        // AC2.2.3: on_click() debe detectar solo el rising edge (0 → 1)
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = MouseClickSensor::new(archflow_engine::MAX_ENTITIES);

        // Frame 1: Botón presionado (rising edge)
        sensor.sample(
            Vec2::new(100.0, 100.0),
            PointerButtons::from_u8(PointerButtons::PRIMARY),
            &store,
        );
        assert!(sensor.on_click(entity));

        // Frame 2: Botón sigue presionado (steady, NOT rising edge)
        sensor.sample(
            Vec2::new(100.0, 100.0),
            PointerButtons::from_u8(PointerButtons::PRIMARY),
            &store,
        );
        assert!(!sensor.on_click(entity), "Should not detect hold as click");

        // Frame 3: Botón liberado
        sensor.sample(Vec2::new(100.0, 100.0), PointerButtons::from_u8(0), &store);
        assert!(!sensor.on_click(entity));

        // Frame 4: Botón presionado de nuevo (rising edge otra vez)
        sensor.sample(
            Vec2::new(100.0, 100.0),
            PointerButtons::from_u8(PointerButtons::PRIMARY),
            &store,
        );
        assert!(sensor.on_click(entity));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 2.2.4: on_double_click() con pattern detection
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_double_click_pattern() {
        // AC2.2.4: Debe detectar double-click pattern: click-pause-click
        // Patrón esperado: 0b00100101 (click, pause, click)
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = MouseClickSensor::new(archflow_engine::MAX_ENTITIES);

        // Simular secuencia de double-click
        // Tick 1: Click (0 → 1)
        sensor.sample(
            Vec2::new(100.0, 100.0),
            PointerButtons::from_u8(PointerButtons::PRIMARY),
            &store,
        );

        // Tick 2-3: Release (1 → 0)
        sensor.sample(Vec2::new(100.0, 100.0), PointerButtons::from_u8(0), &store);
        sensor.sample(Vec2::new(100.0, 100.0), PointerButtons::from_u8(0), &store);

        // Tick 4: Click de nuevo (0 → 1)
        sensor.sample(
            Vec2::new(100.0, 100.0),
            PointerButtons::from_u8(PointerButtons::PRIMARY),
            &store,
        );

        // Tick 5-6: Release
        sensor.sample(Vec2::new(100.0, 100.0), PointerButtons::from_u8(0), &store);
        sensor.sample(Vec2::new(100.0, 100.0), PointerButtons::from_u8(0), &store);

        // Después de 6 ticks, debe detectar double-click
        assert!(sensor.on_double_click(entity));
    }

    #[test]
    fn test_no_double_click_with_single_click() {
        // Un solo click NO debe ser detectado como double-click
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = MouseClickSensor::new(archflow_engine::MAX_ENTITIES);

        // Solo un click
        sensor.sample(
            Vec2::new(100.0, 100.0),
            PointerButtons::from_u8(PointerButtons::PRIMARY),
            &store,
        );
        sensor.sample(Vec2::new(100.0, 100.0), PointerButtons::from_u8(0), &store);
        sensor.sample(Vec2::new(100.0, 100.0), PointerButtons::from_u8(0), &store);
        sensor.sample(Vec2::new(100.0, 100.0), PointerButtons::from_u8(0), &store);
        sensor.sample(Vec2::new(100.0, 100.0), PointerButtons::from_u8(0), &store);
        sensor.sample(Vec2::new(100.0, 100.0), PointerButtons::from_u8(0), &store);

        assert!(!sensor.on_double_click(entity));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // EDGE CASES
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_pointer_buttons_constants() {
        // Verificar que las constantes de botones son correctas
        assert_eq!(PointerButtons::PRIMARY, 0b00000001);
        assert_eq!(PointerButtons::SECONDARY, 0b00000010);
        assert_eq!(PointerButtons::MIDDLE, 0b00000100);
        assert_eq!(PointerButtons::BACK, 0b00001000);
        assert_eq!(PointerButtons::FORWARD, 0b00010000);
    }

    #[test]
    fn test_pointer_buttons_is_methods() {
        let primary = PointerButtons::from_u8(PointerButtons::PRIMARY);
        assert!(primary.is_primary());
        assert!(!primary.is_secondary());

        let secondary = PointerButtons::from_u8(PointerButtons::SECONDARY);
        assert!(!secondary.is_primary());
        assert!(secondary.is_secondary());

        let both = PointerButtons::from_u8(PointerButtons::PRIMARY | PointerButtons::SECONDARY);
        assert!(both.is_primary());
        assert!(both.is_secondary());
    }

    #[test]
    fn test_multiple_entities_different_clicks() {
        // Varias entidades, cada una con su propio estado de click
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(30.0, 30.0));
        let e2 = store.spawn(Vec2::new(150.0, 50.0), Vec2::new(30.0, 30.0));
        let e3 = store.spawn(Vec2::new(100.0, 150.0), Vec2::new(30.0, 30.0));

        let mut sensor = MouseClickSensor::new(archflow_engine::MAX_ENTITIES);

        // Click en e1
        sensor.sample(
            Vec2::new(50.0, 50.0),
            PointerButtons::from_u8(PointerButtons::PRIMARY),
            &store,
        );
        assert!(sensor.on_click(e1));
        assert!(!sensor.on_click(e2));
        assert!(!sensor.on_click(e3));

        // Click en e2
        sensor.sample(
            Vec2::new(150.0, 50.0),
            PointerButtons::from_u8(PointerButtons::PRIMARY),
            &store,
        );
        assert!(!sensor.on_click(e1)); // e1: steady (no rising edge)
        assert!(sensor.on_click(e2)); // e2: rising edge
        assert!(!sensor.on_click(e3));
    }

    #[test]
    fn test_zero_entities() {
        // Debe manejar EntityStore vacío (pre-allocado pero sin spawneados)
        let store = EntityStore::new();
        let mut sensor = MouseClickSensor::new(archflow_engine::MAX_ENTITIES);

        // No debe panic
        sensor.sample(
            Vec2::new(100.0, 100.0),
            PointerButtons::from_u8(PointerButtons::PRIMARY),
            &store,
        );
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // GREEN PHASE CHECKLIST
    // ═══════════════════════════════════════════════════════════════════════════════
    //
    // After implementing MouseClickSensor in src/sensors/mouse_click.rs:
    //
    // 1. Run: cargo test --package archflow-logic --test mouse_click_tests
    // 2. Verify all tests pass
    //
    // ═══════════════════════════════════════════════════════════════════════════════
}
