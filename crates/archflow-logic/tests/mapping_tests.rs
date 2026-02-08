// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Logic Mapping Table Tests
//
// Epic 4: Logic Mapping Table with Controllers
// TDD Approach: Red → Green → Refactor
//
// These tests verify the LogicMappingTable which:
// - Stores connections between sensors and actuators per entity
// - Evaluates sensor signals through controllers (AND, OR, NOT)
// - Executes actuators when controller conditions are met
//
// Note: Integration tests run with std (not no_std)
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    // ═══════════════════════════════════════════════════════════════════════════════
    // RED PHASE: Tests are written FIRST (before implementation exists)
    // ═════════════════════════════════════════════════════════════════════════════

    use archflow_core::{EntityId, Vec2};
    use archflow_engine::EntityStore;
    use archflow_logic::SignalByte;
    use archflow_logic::mapping::{Controller, LogicMappingTable, SensorType};

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 4.1.1: add_connection(entity, sensor, actuator)
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_add_highlight_connection() {
        // AC4.1.1: Debe poder agregar una conexión sensor→actuator
        let mut table = LogicMappingTable::new();
        let entity = EntityId::new(1);

        table.add_highlight(entity, SensorType::MouseOver, Controller::Direct);

        assert!(table.has_connection(entity, SensorType::MouseOver));
    }

    #[test]
    fn test_add_multiple_connections_same_entity() {
        // AC4.1.1: Una entidad puede tener múltiples conexiones
        let mut table = LogicMappingTable::new();
        let entity = EntityId::new(1);

        table.add_highlight(entity, SensorType::MouseOver, Controller::Direct);

        table.add_select(entity, SensorType::MouseClick, Controller::Direct);

        assert!(table.has_connection(entity, SensorType::MouseOver));
        assert!(table.has_connection(entity, SensorType::MouseClick));
        assert_eq!(table.connection_count(entity), 2);
    }

    #[test]
    fn test_add_connections_different_entities() {
        // AC4.1.1: Diferentes entidades tienen conexiones independientes
        let mut table = LogicMappingTable::new();
        let entity1 = EntityId::new(1);
        let entity2 = EntityId::new(2);

        table.add_highlight(entity1, SensorType::MouseOver, Controller::Direct);

        table.add_select(entity2, SensorType::MouseClick, Controller::Direct);

        assert!(table.has_connection(entity1, SensorType::MouseOver));
        assert!(!table.has_connection(entity1, SensorType::MouseClick));
        assert!(table.has_connection(entity2, SensorType::MouseClick));
        assert!(!table.has_connection(entity2, SensorType::MouseOver));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 4.1.2: remove_connection(entity, sensor, actuator)
    // ═════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_remove_connection() {
        // AC4.1.2: Debe poder eliminar una conexión específica
        let mut table = LogicMappingTable::new();
        let entity = EntityId::new(1);

        table.add_highlight(entity, SensorType::MouseOver, Controller::Direct);

        assert!(table.has_connection(entity, SensorType::MouseOver));

        table.remove_connection(entity, SensorType::MouseOver);

        assert!(!table.has_connection(entity, SensorType::MouseOver));
        assert_eq!(table.connection_count(entity), 0);
    }

    #[test]
    fn test_remove_one_of_many_connections() {
        // AC4.1.2: Eliminar una conexión no afecta las otras
        let mut table = LogicMappingTable::new();
        let entity = EntityId::new(1);

        table.add_highlight(entity, SensorType::MouseOver, Controller::Direct);

        table.add_select(entity, SensorType::MouseClick, Controller::Direct);

        table.remove_connection(entity, SensorType::MouseOver);

        assert!(!table.has_connection(entity, SensorType::MouseOver));
        assert!(table.has_connection(entity, SensorType::MouseClick));
        assert_eq!(table.connection_count(entity), 1);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 4.1.3: evaluate_connections(entity) → execution
    // ═════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_evaluate_with_active_sensor_direct_controller() {
        // AC4.1.3: Con sensor activo y controller Direct, debe ejecutar actuator
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut table = LogicMappingTable::new();
        table.add_highlight(entity, SensorType::MouseOver, Controller::Direct);

        // Simular señal activa (6 ticks de true)
        let mut signal = SignalByte::default();
        for _ in 0..6 {
            signal.push(true);
        }

        // Evaluar debe ejecutar el actuador Highlight
        let executed = table.evaluate(&mut store, entity, &[(SensorType::MouseOver, signal)]);

        assert!(executed > 0);
    }

    #[test]
    fn test_evaluate_with_inactive_sensor_no_execution() {
        // AC4.1.3: Con sensor inactivo, no debe ejecutar actuator
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut table = LogicMappingTable::new();
        table.add_highlight(entity, SensorType::MouseOver, Controller::Direct);

        // Señal inactiva (todos false)
        let signal = SignalByte::from(0b00000000);

        let executed = table.evaluate(&mut store, entity, &[(SensorType::MouseOver, signal)]);

        assert_eq!(executed, 0);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 4.2.1: AND controller - todos los sensores activos
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_and_controller_all_active() {
        // AC4.2.1: AND con todos los sensores activos → ejecuta
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut table = LogicMappingTable::new();
        table.add_highlight(
            entity,
            SensorType::MouseOver,
            Controller::AND(SensorType::MouseClick),
        );

        // Ambos sensores activos
        let mouse_over = SignalByte::from(0b00111111); // Activo
        let mouse_click = SignalByte::from(0b00111111); // Activo

        let signals = &[
            (SensorType::MouseOver, mouse_over),
            (SensorType::MouseClick, mouse_click),
        ];

        let executed = table.evaluate(&mut store, entity, signals);

        assert!(executed > 0);
    }

    #[test]
    fn test_and_controller_one_inactive() {
        // AC4.2.1: AND con un sensor inactivo → NO ejecuta
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut table = LogicMappingTable::new();
        table.add_highlight(
            entity,
            SensorType::MouseOver,
            Controller::AND(SensorType::MouseClick),
        );

        // MouseOver activo, MouseClick inactivo
        let mouse_over = SignalByte::from(0b00111111);
        let mouse_click = SignalByte::from(0b00000000);

        let signals = &[
            (SensorType::MouseOver, mouse_over),
            (SensorType::MouseClick, mouse_click),
        ];

        let executed = table.evaluate(&mut store, entity, signals);

        assert_eq!(executed, 0);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 4.2.2: OR controller - al menos un sensor activo
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_or_controller_one_active() {
        // AC4.2.2: OR con un sensor activo → ejecuta
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut table = LogicMappingTable::new();
        table.add_highlight(
            entity,
            SensorType::MouseOver,
            Controller::OR(SensorType::MouseClick),
        );

        // Solo MouseOver activo
        let mouse_over = SignalByte::from(0b00111111);
        let mouse_click = SignalByte::from(0b00000000);

        let signals = &[
            (SensorType::MouseOver, mouse_over),
            (SensorType::MouseClick, mouse_click),
        ];

        let executed = table.evaluate(&mut store, entity, signals);

        assert!(executed > 0);
    }

    #[test]
    fn test_or_controller_all_inactive() {
        // AC4.2.2: OR con todos inactivos → NO ejecuta
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut table = LogicMappingTable::new();
        table.add_highlight(
            entity,
            SensorType::MouseOver,
            Controller::OR(SensorType::MouseClick),
        );

        // Ambos inactivos
        let mouse_over = SignalByte::from(0b00000000);
        let mouse_click = SignalByte::from(0b00000000);

        let signals = &[
            (SensorType::MouseOver, mouse_over),
            (SensorType::MouseClick, mouse_click),
        ];

        let executed = table.evaluate(&mut store, entity, signals);

        assert_eq!(executed, 0);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 4.2.3: NOT controller - invierte la señal
    // ═════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_not_controller_inverts_active() {
        // AC4.2.3: NOT con sensor activo → NO ejecuta (invertido)
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut table = LogicMappingTable::new();
        table.add_highlight(entity, SensorType::MouseOver, Controller::NOT);

        // Sensor activo, pero NOT lo invierte
        let signal = SignalByte::from(0b00111111);

        let executed = table.evaluate(&mut store, entity, &[(SensorType::MouseOver, signal)]);

        assert_eq!(executed, 0);
    }

    #[test]
    fn test_not_controller_inverts_inactive() {
        // AC4.2.3: NOT con sensor inactivo → ejecuta (invertido)
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut table = LogicMappingTable::new();
        table.add_highlight(entity, SensorType::MouseOver, Controller::NOT);

        // Sensor inactivo, NOT lo invierte a activo
        let signal = SignalByte::from(0b00000000);

        let executed = table.evaluate(&mut store, entity, &[(SensorType::MouseOver, signal)]);

        assert!(executed > 0);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // EDGE CASES
    // ═════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_clear_all_connections_for_entity() {
        let mut table = LogicMappingTable::new();
        let entity = EntityId::new(1);

        table.add_highlight(entity, SensorType::MouseOver, Controller::Direct);

        table.add_select(entity, SensorType::MouseClick, Controller::Direct);

        assert_eq!(table.connection_count(entity), 2);

        table.clear_entity(entity);

        assert_eq!(table.connection_count(entity), 0);
        assert!(!table.has_connection(entity, SensorType::MouseOver));
        assert!(!table.has_connection(entity, SensorType::MouseClick));
    }

    #[test]
    fn test_multiple_entities_independent() {
        let mut table = LogicMappingTable::new();
        let entity1 = EntityId::new(1);
        let entity2 = EntityId::new(2);

        table.add_highlight(entity1, SensorType::MouseOver, Controller::Direct);

        table.add_select(entity2, SensorType::MouseClick, Controller::Direct);

        assert_eq!(table.connection_count(entity1), 1);
        assert_eq!(table.connection_count(entity2), 1);

        table.clear_entity(entity1);

        assert_eq!(table.connection_count(entity1), 0);
        assert_eq!(table.connection_count(entity2), 1); // entity2 no afectado
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // GREEN PHASE CHECKLIST
    // ═══════════════════════════════════════════════════════════════════════════════
    //
    // After implementing LogicMappingTable and Controller in src/mapping/:
    //
    // 1. Run: cargo test --package archflow-logic --test mapping_tests
    // 2. Verify all tests pass
    //
    // ═══════════════════════════════════════════════════════════════════════════════
}
