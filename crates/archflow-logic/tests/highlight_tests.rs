// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Highlight Actuator Tests
//
// Epic 3.1: Highlight Actuator
// TDD Approach: Red → Green → Refactor
//
// These tests verify the HighlightActuator which:
// - Changes entity color when activated
// - Restores original color when deactivated
// - Uses Command::SetColor for state changes
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
    use archflow_logic::actuators::HighlightActuator;

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 3.1.1: Highlight { entity, color } → Command::SetColor
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_highlight_generates_set_color_command() {
        // AC3.1.1: Debe generar Command::SetColor cuando se activa
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        store.set_color(entity.index().0 as usize, 0xFF0000FF); // Rojo inicial

        let mut actuator = HighlightActuator::new();
        let commands = actuator.update(&mut store, entity, true, 0x00FF00FF);

        assert_eq!(commands.len(), 1);
        match commands[0] {
            Command::SetColor { id, color } => {
                assert_eq!(id, entity);
                assert_eq!(color, 0x00FF00FF);
            }
            _ => panic!("Expected SetColor command, got {:?}", commands[0]),
        }
    }

    #[test]
    fn test_highlight_no_command_when_already_active() {
        // AC3.1.1: No debe generar comandos si ya está activivo con mismo color
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        store.set_color(entity.index().0 as usize, 0xFF0000FF);

        let mut actuator = HighlightActuator::new();

        // Primera activación
        let _ = actuator.update(&mut store, entity, true, 0x00FF00FF);

        // Segunda activación con mismo color (no debe generar nuevo comando)
        let commands = actuator.update(&mut store, entity, true, 0x00FF00FF);
        assert_eq!(commands.len(), 0);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 3.1.2: Soporta restore de color original
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_restore_original_color() {
        // AC3.1.2: Debe restaurar color original cuando se desactiva
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let original_color = 0xFF0000FF;
        store.set_color(entity.index().0 as usize, original_color);

        let mut actuator = HighlightActuator::new();

        // Activar highlight
        let _ = actuator.update(&mut store, entity, true, 0x00FF00FF);

        // Desactivar (restore)
        let commands = actuator.update(&mut store, entity, false, 0x00FF00FF);

        assert_eq!(commands.len(), 1);
        match commands[0] {
            Command::SetColor { color, .. } => {
                assert_eq!(color, original_color);
            }
            _ => panic!("Expected SetColor command"),
        }
    }

    #[test]
    fn test_restore_multiple_entities() {
        // AC3.1.2: Debe manejar múltiples entidades con diferentes colores
        let mut store = EntityStore::new();
        let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let entity2 = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0));

        store.set_color(entity1.index().0 as usize, 0xFF0000FF); // Rojo
        store.set_color(entity2.index().0 as usize, 0x00FF00FF); // Verde

        let mut actuator = HighlightActuator::new();

        // Activar ambas
        let _ = actuator.update(&mut store, entity1, true, 0x0000FFFF); // Azul
        let _ = actuator.update(&mut store, entity2, true, 0x0000FFFF); // Azul

        // Desactivar entity1
        let commands = actuator.update(&mut store, entity1, false, 0x0000FFFF);
        match commands[0] {
            Command::SetColor { color, .. } => {
                assert_eq!(color, 0xFF0000FF); // Restore rojo
            }
            _ => panic!("Expected SetColor command"),
        }

        // Desactivar entity2
        let commands = actuator.update(&mut store, entity2, false, 0x0000FFFF);
        match commands[0] {
            Command::SetColor { color, .. } => {
                assert_eq!(color, 0x00FF00FF); // Restore verde
            }
            _ => panic!("Expected SetColor command"),
        }
    }

    #[test]
    fn test_no_restore_if_never_activated() {
        // AC3.1.2: No debe restaurar si nunca se activó
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut actuator = HighlightActuator::new();

        // Desactivar sin activar primero
        let commands = actuator.update(&mut store, entity, false, 0x00FF00FF);

        assert_eq!(commands.len(), 0);
    }

    #[test]
    fn test_color_change_while_active() {
        // AC3.1.2: Si el color cambia externamente, debe capturar el nuevo original
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        store.set_color(entity.index().0 as usize, 0xFF0000FF); // Rojo

        let mut actuator = HighlightActuator::new();

        // Activar con verde
        let _ = actuator.update(&mut store, entity, true, 0x00FF00FF);

        // Cambiar color externamente a amarillo
        store.set_color(entity.index().0 as usize, 0xFFFF00FF);

        // Desactivar - debe restaurar amarillo (el color actual)
        let commands = actuator.update(&mut store, entity, false, 0x00FF00FF);
        match commands[0] {
            Command::SetColor { color, .. } => {
                assert_eq!(color, 0xFFFF00FF);
            }
            _ => panic!("Expected SetColor command"),
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 3.1.3: Fade animation opcional (v2 - NO implementar ahora)
    // ═══════════════════════════════════════════════════════════════════════════════

    // Fade animation se marca como deuda técnica para futura implementación

    // ═══════════════════════════════════════════════════════════════════════════════
    // EDGE CASES
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_different_highlight_color_replaces_original() {
        // Cambiar color de highlight mientras está activivo
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        store.set_color(entity.index().0 as usize, 0xFF0000FF);

        let mut actuator = HighlightActuator::new();

        // Activar con azul
        let _ = actuator.update(&mut store, entity, true, 0x0000FFFF);

        // Cambiar a verde mientras está activivo
        let commands = actuator.update(&mut store, entity, true, 0x00FF00FF);
        assert_eq!(commands.len(), 1);
        match commands[0] {
            Command::SetColor { color, .. } => {
                assert_eq!(color, 0x00FF00FF);
            }
            _ => panic!("Expected SetColor command"),
        }

        // Desactivar debe restaurar el rojo original
        let commands = actuator.update(&mut store, entity, false, 0x00FF00FF);
        match commands[0] {
            Command::SetColor { color, .. } => {
                assert_eq!(color, 0xFF0000FF);
            }
            _ => panic!("Expected SetColor command"),
        }
    }

    #[test]
    fn test_reactivate_after_deactivate() {
        // Activar → desactivar → activar nuevamente
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        store.set_color(entity.index().0 as usize, 0xFF0000FF);

        let mut actuator = HighlightActuator::new();

        // Primer ciclo
        let _ = actuator.update(&mut store, entity, true, 0x00FF00FF);
        let commands = actuator.update(&mut store, entity, false, 0x00FF00FF);
        assert_eq!(commands.len(), 1);

        // Segundo ciclo (debe capturar nuevo color original)
        store.set_color(entity.index().0 as usize, 0x0000FFFF); // Azul
        let commands = actuator.update(&mut store, entity, true, 0x00FF00FF);
        assert_eq!(commands.len(), 1);

        // Desactivar debe restaurar azul
        let commands = actuator.update(&mut store, entity, false, 0x00FF00FF);
        match commands[0] {
            Command::SetColor { color, .. } => {
                assert_eq!(color, 0x0000FFFF);
            }
            _ => panic!("Expected SetColor command"),
        }
    }

    #[test]
    fn test_no_command_on_repeated_deactivate() {
        // Desactivar múltiples veces no debe generar comandos duplicados
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        store.set_color(entity.index().0 as usize, 0xFF0000FF);

        let mut actuator = HighlightActuator::new();

        // Activar
        let _ = actuator.update(&mut store, entity, true, 0x00FF00FF);

        // Primera desactivación
        let commands = actuator.update(&mut store, entity, false, 0x00FF00FF);
        assert_eq!(commands.len(), 1);

        // Segunda desactivación (no debe generar comando)
        let commands = actuator.update(&mut store, entity, false, 0x00FF00FF);
        assert_eq!(commands.len(), 0);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // GREEN PHASE CHECKLIST
    // ═══════════════════════════════════════════════════════════════════════════════
    //
    // After implementing HighlightActuator in src/actuators/highlight.rs:
    //
    // 1. Run: cargo test --package archflow-logic --test highlight_tests
    // 2. Verify all tests pass
    //
    // ═══════════════════════════════════════════════════════════════════════════════
}
