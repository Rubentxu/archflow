// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Move Actuator Tests
//
// Epic 3.3: Move Actuator with Hysteresis
// TDD Approach: Red → Green → Refactor
//
// These tests verify the MoveActuator which:
// - Initiates drag only after 6 ticks of steady signal (hysteresis)
// - Requires 6 ticks of 0 to release (prevents accidental release)
// - Generates Command::Move with delta accumulated
//
// Note: Integration tests run with std (not no_std)
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    // ═══════════════════════════════════════════════════════════════════════════════
    // RED PHASE: Tests are written FIRST (before implementation exists)
    // ═══════════════════════════════════════════════════════════════════════════════

    use archflow_core::Vec2;
    use archflow_engine::{Command, EntityStore};
    use archflow_logic::actuators::MoveActuator;
    use archflow_logic::SignalByte;

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 3.3.1: Move { entity, delta } solo cuando señal estable por 6 ticks
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_drag_starts_after_6_ticks_stable() {
        // AC3.3.1: Drag debe iniciar solo después de 6 ticks estables en true
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut actuator = MoveActuator::new();
        let mut signal = SignalByte::default();

        // Simular 6 ticks de click
        for _ in 0..6 {
            signal.push(true);
        }

        let commands = actuator.update(entity, signal, Vec2::new(110.0, 105.0), &store);

        // Drag iniciado - debe haber un comando Move
        assert_eq!(commands.len(), 1);
        match commands[0] {
            Command::Move { id, delta } => {
                assert_eq!(id, entity);
                // Delta desde posición inicial (100, 100) hasta (110, 105)
                assert_eq!(delta.x, 10.0);
                assert_eq!(delta.y, 5.0);
            }
            _ => panic!("Expected Move command, got {:?}", commands[0]),
        }

        // Verificar que está en estado dragging
        assert!(actuator.is_dragging(entity));
    }

    #[test]
    fn test_no_drag_before_6_ticks() {
        // AC3.3.1: No debe iniciar drag antes de 6 ticks estables
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut actuator = MoveActuator::new();
        let mut signal = SignalByte::default();

        // Solo 5 ticks de click
        for _ in 0..5 {
            signal.push(true);
        }

        let commands = actuator.update(entity, signal, Vec2::new(110.0, 105.0), &store);

        // No debe generar comandos
        assert_eq!(commands.len(), 0);
        assert!(!actuator.is_dragging(entity));
    }

    #[test]
    fn test_no_drag_with_unstable_signal() {
        // AC3.3.1: No debe iniciar drag si la señal no es estable
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut actuator = MoveActuator::new();
        let mut signal = SignalByte::default();

        // Señal inestable: 1-0-1-0-1-0
        for _ in 0..3 {
            signal.push(true);
            signal.push(false);
        }

        let commands = actuator.update(entity, signal, Vec2::new(110.0, 105.0), &store);

        assert_eq!(commands.len(), 0);
        assert!(!actuator.is_dragging(entity));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 3.3.2: Hysteresis - requiere 6 ticks de 0 para soltar
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_hysteresis_prevents_accidental_release() {
        // AC3.3.2: Hysteresis previene liberación accidental por ruido
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut actuator = MoveActuator::new();
        let mut signal = SignalByte::default();

        // Iniciar drag con 6 ticks estables
        for _ in 0..6 {
            signal.push(true);
        }
        let _ = actuator.update(entity, signal, Vec2::new(110.0, 105.0), &store);
        assert!(actuator.is_dragging(entity));

        // Simular 1 tick de "noise" (paquete de red perdido)
        signal.push(false);
        signal.push(true);

        let commands = actuator.update(entity, signal, Vec2::new(115.0, 110.0), &store);

        // No debe soltar por 1 tick de ruido - debe seguir arrastrando
        assert!(actuator.is_dragging(entity));
        assert_eq!(commands.len(), 1); // Debe haber un comando Move
    }

    #[test]
    fn test_release_after_6_ticks_stable_low() {
        // AC3.3.2: Debe soltar después de 6 ticks estables en false
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut actuator = MoveActuator::new();
        let mut signal = SignalByte::default();

        // Iniciar drag
        for _ in 0..6 {
            signal.push(true);
        }
        let _ = actuator.update(entity, signal, Vec2::new(110.0, 105.0), &store);
        assert!(actuator.is_dragging(entity));

        // 6 ticks estables en false (señal de release)
        for _ in 0..6 {
            signal.push(false);
        }

        let commands = actuator.update(entity, signal, Vec2::new(110.0, 105.0), &store);

        // Ya no debe estar arrastrando
        assert!(!actuator.is_dragging(entity));
        // No debe generar comandos (drag terminado)
        assert_eq!(commands.len(), 0);
    }

    #[test]
    fn test_hysteresis_requires_exactly_6_ticks_to_release() {
        // AC3.3.2: Precisamente 6 ticks en false para soltar
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut actuator = MoveActuator::new();
        let mut signal = SignalByte::default();

        // Iniciar drag
        for _ in 0..6 {
            signal.push(true);
        }
        let _ = actuator.update(entity, signal, Vec2::new(110.0, 105.0), &store);

        // Solo 5 ticks en false (no suficiente para release)
        for _ in 0..5 {
            signal.push(false);
        }

        let _ = actuator.update(entity, signal, Vec2::new(110.0, 105.0), &store);

        // Debe seguir arrastrando
        assert!(actuator.is_dragging(entity));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 3.3.3: Genera Command::Move con delta acumulado
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_move_command_uses_delta_from_start() {
        // AC3.3.3: Delta debe ser calculado desde la posición inicial
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut actuator = MoveActuator::new();
        let mut signal = SignalByte::default();

        // Iniciar drag
        for _ in 0..6 {
            signal.push(true);
        }

        // Mouse movió a (120, 130)
        let commands = actuator.update(entity, signal, Vec2::new(120.0, 130.0), &store);

        match commands[0] {
            Command::Move { delta, .. } => {
                // Delta = (120-100, 130-100) = (20, 30)
                assert_eq!(delta.x, 20.0);
                assert_eq!(delta.y, 30.0);
            }
            _ => panic!("Expected Move command"),
        }
    }

    #[test]
    fn test_delta_updates_during_drag() {
        // AC3.3.3: Delta debe actualizarse mientras se arrastra
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut actuator = MoveActuator::new();
        let mut signal = SignalByte::default();

        // Iniciar drag
        for _ in 0..6 {
            signal.push(true);
        }
        let _ = actuator.update(entity, signal, Vec2::new(110.0, 105.0), &store);

        // Mantener señal estable
        signal.push(true);

        // Mouse movió a (125, 140)
        let commands = actuator.update(entity, signal, Vec2::new(125.0, 140.0), &store);

        match commands[0] {
            Command::Move { delta, .. } => {
                // Delta desde start (100, 100) hasta (125, 140)
                assert_eq!(delta.x, 25.0);
                assert_eq!(delta.y, 40.0);
            }
            _ => panic!("Expected Move command"),
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // EDGE CASES
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_multiple_entities_can_drag_simultaneously() {
        // Múltiples entidades pueden arrastrarse simultáneamente
        let mut store = EntityStore::new();
        let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let entity2 = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0));

        let mut actuator = MoveActuator::new();

        // Iniciar drag en entity1
        let mut signal1 = SignalByte::default();
        for _ in 0..6 {
            signal1.push(true);
        }
        let _ = actuator.update(entity1, signal1, Vec2::new(110.0, 105.0), &store);

        // Iniciar drag en entity2
        let mut signal2 = SignalByte::default();
        for _ in 0..6 {
            signal2.push(true);
        }
        let _ = actuator.update(entity2, signal2, Vec2::new(210.0, 205.0), &store);

        // Ambas deben estar arrastrando
        assert!(actuator.is_dragging(entity1));
        assert!(actuator.is_dragging(entity2));
    }

    #[test]
    fn test_no_move_commands_when_not_dragging() {
        // No debe generar comandos si no está arrastrando
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut actuator = MoveActuator::new();
        let mut signal = SignalByte::default();

        // Señal inestable
        signal.push(true);
        signal.push(false);
        signal.push(true);

        let commands = actuator.update(entity, signal, Vec2::new(150.0, 150.0), &store);

        assert_eq!(commands.len(), 0);
    }

    #[test]
    fn test_drag_state_persists_across_updates() {
        // El estado de drag debe persistir entre actualizaciones
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut actuator = MoveActuator::new();
        let mut signal = SignalByte::default();

        // Iniciar drag
        for _ in 0..6 {
            signal.push(true);
        }
        let _ = actuator.update(entity, signal, Vec2::new(110.0, 105.0), &store);

        // Actualizar manteniendo señal estable
        for _ in 0..3 {
            signal.push(true);
            let _ = actuator.update(entity, signal, Vec2::new(110.0, 105.0), &store);
        }

        // Debe seguir arrastrando después de múltiples actualizaciones
        assert!(actuator.is_dragging(entity));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // GREEN PHASE CHECKLIST
    // ═══════════════════════════════════════════════════════════════════════════════
    //
    // After implementing MoveActuator in src/actuators/move.rs:
    //
    // 1. Run: cargo test --package archflow-logic --test move_tests
    // 2. Verify all tests pass
    //
    // ═══════════════════════════════════════════════════════════════════════════════
}
