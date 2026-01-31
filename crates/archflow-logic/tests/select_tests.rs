// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Select Actuator Tests
//
// Epic 3.2: Select Actuator
// TDD Approach: Red → Green → Refactor
//
// These tests verify the SelectActuator which:
// - Manages entity selection state (Single/Multi/Replace modes)
// - Uses EntityStore.set_selected() for visual feedback
// - Tracks currently selected entities
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
    use archflow_logic::actuators::{SelectActuator, SelectMode};

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 3.2.1: Select { entity, mode } donde mode = Single/Multi/Replace
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_select_single_mode() {
        // AC3.2.1: Single mode debe seleccionar una entidad y deseleccionar las anteriores
        let mut store = EntityStore::new();
        let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let entity2 = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0));

        let mut actuator = SelectActuator::new();

        // Seleccionar entity1
        actuator.update(&mut store, entity1, true, SelectMode::Single);

        assert!(store.is_selected(entity1.index().0 as usize));

        // Seleccionar entity2 en modo Single (debe deseleccionar entity1)
        actuator.update(&mut store, entity2, true, SelectMode::Single);

        assert!(!store.is_selected(entity1.index().0 as usize));
        assert!(store.is_selected(entity2.index().0 as usize));
    }

    #[test]
    fn test_select_multi_mode() {
        // AC3.2.1: Multi mode permite múltiples selecciones simultáneas
        let mut store = EntityStore::new();
        let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let entity2 = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0));

        let mut actuator = SelectActuator::new();

        // Seleccionar entity1
        actuator.update(&mut store, entity1, true, SelectMode::Multi);
        assert!(store.is_selected(entity1.index().0 as usize));

        // Seleccionar entity2 en modo Multi (ambos deben quedar seleccionados)
        actuator.update(&mut store, entity2, true, SelectMode::Multi);

        assert!(store.is_selected(entity1.index().0 as usize));
        assert!(store.is_selected(entity2.index().0 as usize));
    }

    #[test]
    fn test_select_replace_mode() {
        // AC3.2.1: Replace mode borra selección anterior y selecciona la nueva
        let mut store = EntityStore::new();
        let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let entity2 = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0));
        let entity3 = store.spawn(Vec2::new(300.0, 300.0), Vec2::new(50.0, 50.0));

        let mut actuator = SelectActuator::new();

        // Seleccionar entity1 y entity2 en modo Multi
        actuator.update(&mut store, entity1, true, SelectMode::Multi);
        actuator.update(&mut store, entity2, true, SelectMode::Multi);
        assert!(store.is_selected(entity1.index().0 as usize));
        assert!(store.is_selected(entity2.index().0 as usize));

        // Seleccionar entity3 en modo Replace (solo entity3 debe quedar)
        actuator.update(&mut store, entity3, true, SelectMode::Replace);

        assert!(!store.is_selected(entity1.index().0 as usize));
        assert!(!store.is_selected(entity2.index().0 as usize));
        assert!(store.is_selected(entity3.index().0 as usize));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 3.2.2: Usa EntityStore.set_selected()
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_select_uses_store_set_selected() {
        // AC3.2.2: Verificar que se usa set_selected() correctamente
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut actuator = SelectActuator::new();

        // Activar selección
        actuator.update(&mut store, entity, true, SelectMode::Single);

        // Verificar metadata de selección (bit 9) y color tint
        let idx = entity.index().0 as usize;
        assert!(store.is_selected(idx));

        // Color tint debe estar activo para feedback visual
        let tint = store.color_tints[idx];
        // [0.3, 0.5, 1.0, 0.3] es el tint de selección
        assert_eq!(tint[0], 0.3);
        assert_eq!(tint[1], 0.5);
        assert_eq!(tint[2], 1.0);
    }

    #[test]
    fn test_deselect_clears_visual_feedback() {
        // AC3.2.2: Desseleccionar debe limpiar el feedback visual
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut actuator = SelectActuator::new();

        // Seleccionar
        actuator.update(&mut store, entity, true, SelectMode::Single);
        assert!(store.is_selected(entity.index().0 as usize));

        // Deseleccionar
        actuator.update(&mut store, entity, false, SelectMode::Single);
        assert!(!store.is_selected(entity.index().0 as usize));

        // Color tint debe ser [1.0, 1.0, 1.0, 1.0] (normal)
        let tint = store.color_tints[entity.index().0 as usize];
        assert_eq!(tint, [1.0, 1.0, 1.0, 1.0]);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACCEPTANCE CRITERIA 3.2.3: Compatible con selection manager existente
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_get_selected_entities() {
        // AC3.2.3: Debe poder retornar lista de entidades seleccionadas
        let mut store = EntityStore::new();
        let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let entity2 = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0));
        let entity3 = store.spawn(Vec2::new(300.0, 300.0), Vec2::new(50.0, 50.0));

        let mut actuator = SelectActuator::new();

        // Seleccionar entity1 y entity2
        actuator.update(&mut store, entity1, true, SelectMode::Multi);
        actuator.update(&mut store, entity2, true, SelectMode::Multi);

        // Obtener lista de seleccionados
        let selected = actuator.selected_entities();
        assert_eq!(selected.len(), 2);
        assert!(selected.contains(&entity1));
        assert!(selected.contains(&entity2));
        assert!(!selected.contains(&entity3));
    }

    #[test]
    fn test_clear_all_selections() {
        // AC3.2.3: Debe poder limpiar todas las selecciones
        let mut store = EntityStore::new();
        let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let entity2 = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0));

        let mut actuator = SelectActuator::new();

        actuator.update(&mut store, entity1, true, SelectMode::Multi);
        actuator.update(&mut store, entity2, true, SelectMode::Multi);

        assert_eq!(actuator.selected_count(), 2);

        // Limpiar todas las selecciones
        actuator.clear_all(&mut store);

        assert_eq!(actuator.selected_count(), 0);
        assert!(!store.is_selected(entity1.index().0 as usize));
        assert!(!store.is_selected(entity2.index().0 as usize));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // EDGE CASES
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_deselect_in_multi_mode() {
        // Deseleccionar una entidad específica en modo Multi
        let mut store = EntityStore::new();
        let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let entity2 = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0));

        let mut actuator = SelectActuator::new();

        // Seleccionar ambas
        actuator.update(&mut store, entity1, true, SelectMode::Multi);
        actuator.update(&mut store, entity2, true, SelectMode::Multi);
        assert!(store.is_selected(entity1.index().0 as usize));
        assert!(store.is_selected(entity2.index().0 as usize));

        // Deseleccionar entity1 (activ=false)
        actuator.update(&mut store, entity1, false, SelectMode::Multi);

        assert!(!store.is_selected(entity1.index().0 as usize));
        assert!(store.is_selected(entity2.index().0 as usize));
    }

    #[test]
    fn test_toggle_selection_in_multi_mode() {
        // Toggle: seleccionar si no está seleccionada, deseleccionar si lo está
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut actuator = SelectActuator::new();

        // Primer toggle: activar (debe seleccionar)
        actuator.update(&mut store, entity, true, SelectMode::Multi);
        assert!(store.is_selected(entity.index().0 as usize));
        assert_eq!(actuator.selected_count(), 1);

        // Segundo toggle: activar mismo (no debe cambiar nada)
        actuator.update(&mut store, entity, true, SelectMode::Multi);
        assert!(store.is_selected(entity.index().0 as usize));
        assert_eq!(actuator.selected_count(), 1);

        // Tercer toggle: desactivar (debe deseleccionar)
        actuator.update(&mut store, entity, false, SelectMode::Multi);
        assert!(!store.is_selected(entity.index().0 as usize));
        assert_eq!(actuator.selected_count(), 0);
    }

    #[test]
    fn test_no_duplicate_selection() {
        // Seleccionar la misma entidad dos veces no debe duplicar
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut actuator = SelectActuator::new();

        actuator.update(&mut store, entity, true, SelectMode::Multi);
        actuator.update(&mut store, entity, true, SelectMode::Multi);

        assert_eq!(actuator.selected_count(), 1);
    }

    #[test]
    fn test_deselect_when_not_selected() {
        // Deseleccionar cuando no está seleccionado no debe hacer nada
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut actuator = SelectActuator::new();

        // Deseleccionar sin haber seleccionado primero
        actuator.update(&mut store, entity, false, SelectMode::Multi);

        assert_eq!(actuator.selected_count(), 0);
        assert!(!store.is_selected(entity.index().0 as usize));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // GREEN PHASE CHECKLIST
    // ═══════════════════════════════════════════════════════════════════════════════
    //
    // After implementing SelectActuator in src/actuators/select.rs:
    //
    // 1. Run: cargo test --package archflow-logic --test select_tests
    // 2. Verify all tests pass
    //
    // ═══════════════════════════════════════════════════════════════════════════════
}
