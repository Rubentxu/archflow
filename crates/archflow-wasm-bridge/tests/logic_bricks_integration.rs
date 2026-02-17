// ═══════════════════════════════════════════════════════════════════════════════
// Logic Bricks Integration Test
//
// Tests the complete Logic Bricks pipeline integration with ArchFlowEngine
// ═══════════════════════════════════════════════════════════════════════════════

#![cfg(test)]

use archflow_core::Vec2;
use archflow_wasm_bridge::engine::ArchFlowEngine;
use archflow_wasm_bridge::logic_bricks_setup::LogicBricksSystem;

#[test]
fn test_logic_bricks_system_creation() {
    let system = LogicBricksSystem::new();

    // Verify the system is created with default state
    assert_eq!(system.selection_count(), 0);
    assert!(!system.has_events());
}

#[test]
fn test_engine_with_logic_bricks() {
    let mut engine = ArchFlowEngine::new(800.0, 600.0);

    // Verify engine has logic_bricks
    assert_eq!(engine.selection_count(), 0);

    // Create a test entity
    let entity_id = engine
        .store
        .spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

    // Test selection helper methods
    assert!(engine.is_selection_empty());
    assert_eq!(engine.selection_count(), 0);

    // Select the entity
    engine.select_entity(entity_id);

    // Verify selection
    assert!(!engine.is_selection_empty());
    assert_eq!(engine.selection_count(), 1);
    assert!(engine.is_entity_selected(entity_id));

    // Get selected entities
    let selected = engine.get_selected_entities();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0], entity_id);

    // Clear selection
    engine.clear_selection();
    assert!(engine.is_selection_empty());
    assert_eq!(engine.selection_count(), 0);
}

#[test]
fn test_multi_selection() {
    let mut engine = ArchFlowEngine::new(800.0, 600.0);

    // Create multiple entities
    let entity1 = engine
        .store
        .spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
    let entity2 = engine
        .store
        .spawn(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0));
    let entity3 = engine
        .store
        .spawn(Vec2::new(300.0, 300.0), Vec2::new(50.0, 50.0));

    // Toggle selection on multiple entities
    engine.toggle_entity_selection(entity1);
    assert_eq!(engine.selection_count(), 1);

    engine.toggle_entity_selection(entity2);
    assert_eq!(engine.selection_count(), 2);

    engine.toggle_entity_selection(entity3);
    assert_eq!(engine.selection_count(), 3);

    // Verify all are selected
    assert!(engine.is_entity_selected(entity1));
    assert!(engine.is_entity_selected(entity2));
    assert!(engine.is_entity_selected(entity3));

    // Toggle off one entity
    engine.toggle_entity_selection(entity2);
    assert_eq!(engine.selection_count(), 2);
    assert!(!engine.is_entity_selected(entity2));

    // Verify others still selected
    assert!(engine.is_entity_selected(entity1));
    assert!(engine.is_entity_selected(entity3));
}

#[cfg(target_arch = "wasm32")]
#[test]
fn test_tick_with_logic_bricks() {
    let mut engine = ArchFlowEngine::new(800.0, 600.0);

    // Create test entity
    let entity_id = engine
        .store
        .spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

    // Select entity
    engine.select_entity(entity_id);

    // Run tick (should execute Logic Bricks pipeline)
    engine.tick(16.0); // 16ms timestamp

    // Verify selection persists after tick
    assert!(engine.is_entity_selected(entity_id));
    assert_eq!(engine.selection_count(), 1);

    // Run multiple ticks
    for i in 1..10 {
        engine.tick((i * 16) as f64);
    }

    // Selection should still be valid
    assert!(engine.is_entity_selected(entity_id));
}

#[test]
fn test_event_polling() {
    let mut engine = ArchFlowEngine::new(800.0, 600.0);

    // Initially no events
    assert!(!engine.logic_bricks.has_events());

    let event_count = engine.logic_bricks.poll_events();
    assert_eq!(event_count, 0);

    // After polling, still no events
    assert!(!engine.logic_bricks.has_events());
}

#[test]
fn test_performance_batch_select() {
    let mut engine = ArchFlowEngine::new(800.0, 600.0);

    // Create 100 entities (reduced from 1000 for faster test)
    let mut entities = Vec::new();
    for i in 0..100 {
        let entity_id = engine.store.spawn(
            Vec2::new(i as f32 * 10.0, i as f32 * 10.0),
            Vec2::new(50.0, 50.0),
        );
        entities.push(entity_id);
    }

    // Select all entities (should be fast with BatchSelectActuator)
    for &entity_id in &entities {
        engine.toggle_entity_selection(entity_id);
    }

    // Verify all selected
    assert_eq!(engine.selection_count(), 100);

    // Clear all (should also be fast)
    engine.clear_selection();
    assert_eq!(engine.selection_count(), 0);
}
