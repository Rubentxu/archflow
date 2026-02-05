// ═══════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Selection Actuator
//
// Actuators for entity selection: Single selection, deselection, and selection state.
// Implements US-001 from TEMA 1.
//
// Architecture:
// - SelectActuator: Single entity selection with visual feedback
// - SelectionState: Track current selection for undo/redo
//
// Performance:
// - O(1) for single select operations
// ═══════════════════════════════════════════════════════════════════════════════════════

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::panic;

use archflow_core::{EntityId, Generation, Index, MAX_ENTITIES, Vec2};
use archflow_engine::{Command, DeltaMask, EntityStore};

/// Selection mode types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionMode {
    /// Replace current selection
    Replace,
    /// Add to current selection
    Add,
    /// Remove from current selection
    Remove,
    /// Toggle selection
    Toggle,
}

/// State for selection operations
#[derive(Clone, Debug, Default)]
pub struct SelectionState {
    /// Currently selected entity
    pub selected_entity: Option<EntityId>,
    /// Previously selected entity (for undo)
    pub previous_selection: Option<EntityId>,
    /// Is in multi-select mode
    pub multi_select: bool,
}

/// Result of selection operation
#[derive(Clone, Debug)]
pub struct SelectionResult {
    /// Commands to apply
    pub commands: Vec<Command>,
    /// Message for user feedback
    pub message: String,
    /// Entity that was selected (if any)
    pub selected: Option<EntityId>,
    /// Entity that was deselected (if any)
    pub deselected: Option<EntityId>,
}

/// Configuration for selection visual feedback
#[derive(Clone, Copy, Debug)]
pub struct SelectionConfig {
    /// Selection border color (ARGB)
    pub border_color: u32,
    /// Selection border width (pixels)
    pub border_width: f32,
    /// Selection fill color (ARGB, semi-transparent)
    pub fill_color: u32,
    /// Handle color when selected
    pub handle_color: u32,
    /// Handle size (pixels)
    pub handle_size: f32,
    /// Animation duration for selection (ms)
    pub animation_duration_ms: u16,
}

impl Default for SelectionConfig {
    fn default() -> Self {
        Self {
            border_color: 0xFF4488FF,
            border_width: 2.0,
            fill_color: 0x204488FF,
            handle_color: 0xFFFFFFFF,
            handle_size: 8.0,
            animation_duration_ms: 150,
        }
    }
}

/// Actuator for managing entity selection.
///
/// Handles single entity selection, deselection, and provides
/// visual feedback for the selected state.
///
/// # Example
///
/// ```
/// use archflow_logic::actuators::selection::{SelectActuator, SelectionMode};
///
/// let mut actuator = SelectActuator::new();
/// let store = /* ... */;
///
/// // Select an entity
/// let result = actuator.select(entity_id, SelectionMode::Replace, &store);
/// ```
pub struct SelectActuator {
    /// Current selection state
    state: SelectionState,
    /// Visual configuration
    config: SelectionConfig,
}

impl SelectActuator {
    /// Creates a new SelectActuator with default configuration
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: SelectionState::default(),
            config: SelectionConfig::default(),
        }
    }

    /// Creates a new SelectActuator with custom configuration
    #[inline(always)]
    #[must_use]
    pub fn with_config(config: SelectionConfig) -> Self {
        Self {
            state: SelectionState::default(),
            config,
        }
    }

    /// Get current selection state
    #[inline(always)]
    #[must_use]
    pub fn state(&self) -> &SelectionState {
        &self.state
    }

    /// Get currently selected entity
    #[inline(always)]
    #[must_use]
    pub fn selected_entity(&self) -> Option<EntityId> {
        self.state.selected_entity
    }

    /// Check if an entity is currently selected
    #[inline(always)]
    #[must_use]
    pub fn is_selected(&self, entity_id: EntityId) -> bool {
        self.state.selected_entity == Some(entity_id)
    }

    /// Check if in multi-select mode
    #[inline(always)]
    #[must_use]
    pub fn multi_select(&self) -> bool {
        self.state.multi_select
    }

    /// Select an entity with the given mode
    ///
    /// # Arguments
    ///
    /// * `entity_id` - Entity to select
    /// * `mode` - Selection mode (replace, add, remove, toggle)
    /// * `store` - EntityStore to query
    ///
    /// # Returns
    ///
    /// Selection result with commands and feedback
    pub fn select(
        &mut self,
        entity_id: EntityId,
        mode: SelectionMode,
        store: &EntityStore,
    ) -> SelectionResult {
        // Check if entity exists
        if !store.is_alive(entity_id) {
            return SelectionResult {
                commands: Vec::new(),
                message: "Cannot select: entity does not exist".into(),
                selected: None,
                deselected: None,
            };
        }

        // Store previous selection for undo
        let previous = self.state.selected_entity;

        // Clear previous selection visual
        let mut commands = Vec::new();
        if let Some(prev_entity) = previous {
            commands.push(Command::SetLayer {
                id: prev_entity,
                layer: 0,
            });
        }

        // Apply selection mode using DeltaMask
        match mode {
            SelectionMode::Replace => {
                self.state.selected_entity = Some(entity_id);
                self.state.multi_select = false;

                // Create delta mask for new selection
                let idx = entity_id.index().0;
                let mask = DeltaMask::from_indices(&[idx], MAX_ENTITIES as usize);

                commands.push(Command::Select(mask));
                commands.push(Command::SetLayer {
                    id: entity_id,
                    layer: 100, // Selection layer
                });

                SelectionResult {
                    commands,
                    message: format!("Selected entity {}", idx),
                    selected: Some(entity_id),
                    deselected: previous,
                }
            }
            SelectionMode::Add => {
                if self.state.selected_entity != Some(entity_id) {
                    self.state.selected_entity = Some(entity_id);
                    self.state.multi_select = true;

                    let idx = entity_id.index().0;
                    let mask = DeltaMask::from_indices(&[idx], MAX_ENTITIES as usize);

                    commands.push(Command::Select(mask));
                    commands.push(Command::SetLayer {
                        id: entity_id,
                        layer: 100,
                    });

                    SelectionResult {
                        commands,
                        message: format!("Added entity {} to selection", idx),
                        selected: Some(entity_id),
                        deselected: None,
                    }
                } else {
                    SelectionResult {
                        commands: Vec::new(),
                        message: String::new(),
                        selected: None,
                        deselected: None,
                    }
                }
            }
            SelectionMode::Remove => {
                if self.state.selected_entity == Some(entity_id) {
                    self.state.selected_entity = None;

                    let idx = entity_id.index().0;
                    let mask = DeltaMask::from_indices(&[idx], MAX_ENTITIES as usize);

                    commands.push(Command::Select(mask));
                    commands.push(Command::SetLayer {
                        id: entity_id,
                        layer: 0,
                    });

                    SelectionResult {
                        commands,
                        message: format!("Removed entity {} from selection", idx),
                        selected: None,
                        deselected: Some(entity_id),
                    }
                } else {
                    SelectionResult {
                        commands: Vec::new(),
                        message: String::new(),
                        selected: None,
                        deselected: None,
                    }
                }
            }
            SelectionMode::Toggle => {
                if self.state.selected_entity == Some(entity_id) {
                    // Deselect
                    self.state.selected_entity = None;

                    let idx = entity_id.index().0;
                    let mask = DeltaMask::from_indices(&[idx], MAX_ENTITIES as usize);

                    commands.push(Command::Select(mask));
                    commands.push(Command::SetLayer {
                        id: entity_id,
                        layer: 0,
                    });

                    SelectionResult {
                        commands,
                        message: format!("Deselected entity {}", idx),
                        selected: None,
                        deselected: Some(entity_id),
                    }
                } else {
                    // Select
                    self.state.selected_entity = Some(entity_id);
                    self.state.multi_select = true;

                    let idx = entity_id.index().0;
                    let mask = DeltaMask::from_indices(&[idx], MAX_ENTITIES as usize);

                    commands.push(Command::Select(mask));
                    commands.push(Command::SetLayer {
                        id: entity_id,
                        layer: 100,
                    });

                    SelectionResult {
                        commands,
                        message: format!("Selected entity {}", idx),
                        selected: Some(entity_id),
                        deselected: previous,
                    }
                }
            }
        }
    }

    /// Deselect the current entity
    ///
    /// # Arguments
    ///
    /// * `store` - EntityStore to query
    ///
    /// # Returns
    ///
    /// Selection result with commands
    pub fn deselect(&mut self, store: &EntityStore) -> SelectionResult {
        let previous = self.state.selected_entity;

        if let Some(entity_id) = previous {
            let mut commands = Vec::new();
            let idx = entity_id.index().0;
            let mask = DeltaMask::from_indices(&[idx], MAX_ENTITIES as usize);

            commands.push(Command::Select(mask));
            commands.push(Command::SetLayer {
                id: entity_id,
                layer: 0,
            });

            self.state.selected_entity = None;
            self.state.multi_select = false;

            SelectionResult {
                commands,
                message: "Deselected all".into(),
                selected: None,
                deselected: Some(entity_id),
            }
        } else {
            SelectionResult {
                commands: Vec::new(),
                message: String::new(),
                selected: None,
                deselected: None,
            }
        }
    }

    /// Deselect all entities
    ///
    /// # Arguments
    ///
    /// * `store` - EntityStore to query
    ///
    /// # Returns
    ///
    /// Selection result with commands
    #[must_use]
    pub fn deselect_all(&mut self, store: &EntityStore) -> SelectionResult {
        self.deselect(store)
    }

    /// Select nothing (clear selection without commands)
    #[inline(always)]
    pub fn clear(&mut self) {
        self.state = SelectionState::default();
    }

    /// Get the selection layer for an entity
    #[inline(always)]
    #[must_use]
    pub fn selection_layer(&self) -> u8 {
        100
    }

    /// Get configuration
    #[inline(always)]
    #[must_use]
    pub fn config(&self) -> &SelectionConfig {
        &self.config
    }
}

impl Default for SelectActuator {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════════════════════════
    // SelectActuator Tests
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_select_actuator_initial_state() {
        let actuator = SelectActuator::new();
        assert!(actuator.selected_entity().is_none());
        assert!(!actuator.multi_select());
    }

    #[test]
    fn test_select_single_entity() {
        let mut actuator = SelectActuator::new();
        let mut store = EntityStore::new();

        let entity = store.spawn(Vec2::ZERO, Vec2::new(100.0, 50.0));
        let result = actuator.select(entity, SelectionMode::Replace, &store);

        assert!(result.selected.is_some());
        assert_eq!(result.selected, Some(entity));
        assert!(!result.commands.is_empty());
    }

    #[test]
    fn test_deselect_entity() {
        let mut actuator = SelectActuator::new();
        let mut store = EntityStore::new();

        let entity = store.spawn(Vec2::ZERO, Vec2::new(100.0, 50.0));
        actuator.select(entity, SelectionMode::Replace, &store);

        let result = actuator.deselect(&store);

        assert!(result.deselected.is_some());
        assert!(actuator.selected_entity().is_none());
    }

    #[test]
    fn test_select_different_entity_replaces() {
        let mut actuator = SelectActuator::new();
        let mut store = EntityStore::new();

        let entity1 = store.spawn(Vec2::ZERO, Vec2::new(100.0, 50.0));
        let entity2 = store.spawn(Vec2::new(200.0, 0.0), Vec2::new(100.0, 50.0));

        actuator.select(entity1, SelectionMode::Replace, &store);
        assert_eq!(actuator.selected_entity(), Some(entity1));

        let result = actuator.select(entity2, SelectionMode::Replace, &store);

        assert_eq!(result.selected, Some(entity2));
        assert_eq!(result.deselected, Some(entity1));
    }

    #[test]
    fn test_toggle_selection() {
        let mut actuator = SelectActuator::new();
        let mut store = EntityStore::new();

        let entity = store.spawn(Vec2::ZERO, Vec2::new(100.0, 50.0));

        // Toggle on
        actuator.select(entity, SelectionMode::Toggle, &store);
        assert!(actuator.is_selected(entity));

        // Toggle off
        actuator.select(entity, SelectionMode::Toggle, &store);
        assert!(!actuator.is_selected(entity));
    }

    #[test]
    fn test_add_to_selection() {
        let mut actuator = SelectActuator::new();
        let mut store = EntityStore::new();

        let entity1 = store.spawn(Vec2::ZERO, Vec2::new(100.0, 50.0));
        let entity2 = store.spawn(Vec2::new(200.0, 0.0), Vec2::new(100.0, 50.0));

        actuator.select(entity1, SelectionMode::Replace, &store);
        assert!(actuator.is_selected(entity1));
        assert!(!actuator.multi_select());

        actuator.select(entity2, SelectionMode::Add, &store);
        assert!(actuator.multi_select());
    }

    #[test]
    fn test_remove_from_selection() {
        let mut actuator = SelectActuator::new();
        let mut store = EntityStore::new();

        let entity = store.spawn(Vec2::ZERO, Vec2::new(100.0, 50.0));

        // Select the entity
        actuator.select(entity, SelectionMode::Replace, &store);
        assert!(actuator.is_selected(entity));

        // Remove it using toggle
        let result = actuator.select(entity, SelectionMode::Toggle, &store);

        assert!(result.deselected.is_some());
        assert!(!actuator.is_selected(entity));
    }

    #[test]
    fn test_deselect_all() {
        let mut actuator = SelectActuator::new();
        let mut store = EntityStore::new();

        let entity = store.spawn(Vec2::ZERO, Vec2::new(100.0, 50.0));
        actuator.select(entity, SelectionMode::Replace, &store);

        let result = actuator.deselect_all(&store);

        assert!(result.deselected.is_some());
        assert!(actuator.selected_entity().is_none());
    }

    #[test]
    fn test_clear_selection() {
        let mut actuator = SelectActuator::new();
        let mut store = EntityStore::new();

        let entity = store.spawn(Vec2::ZERO, Vec2::new(100.0, 50.0));
        actuator.select(entity, SelectionMode::Replace, &store);

        actuator.clear();

        assert!(actuator.selected_entity().is_none());
    }

    #[test]
    fn test_select_nonexistent_entity() {
        let mut actuator = SelectActuator::new();
        let store = EntityStore::new();
        // Create an entity ID with index beyond MAX_ENTITIES-1
        // EntityId index is 24 bits, so we can use a value that exceeds MAX_ENTITIES
        let max_index = MAX_ENTITIES as u32;
        let nonexistent = EntityId::from_parts(Index(max_index), Generation(0));

        // This should be rejected because index >= MAX_ENTITIES
        let result = actuator.select(nonexistent, SelectionMode::Replace, &store);

        assert!(result.commands.is_empty());
    }

    #[test]
    fn test_selection_layer() {
        let actuator = SelectActuator::new();
        assert_eq!(actuator.selection_layer(), 100);
    }

    #[test]
    fn test_multi_select_flag() {
        let mut actuator = SelectActuator::new();
        let mut store = EntityStore::new();

        let entity1 = store.spawn(Vec2::ZERO, Vec2::new(100.0, 50.0));
        let entity2 = store.spawn(Vec2::new(200.0, 0.0), Vec2::new(100.0, 50.0));

        // Single select
        actuator.select(entity1, SelectionMode::Replace, &store);
        assert!(!actuator.multi_select());

        // Add to selection
        actuator.select(entity2, SelectionMode::Add, &store);
        assert!(actuator.multi_select());
    }

    #[test]
    fn test_selection_config() {
        let config = SelectionConfig {
            border_color: 0xFF0000FF,
            border_width: 3.0,
            fill_color: 0x200000FF,
            handle_color: 0xFF000000,
            handle_size: 10.0,
            animation_duration_ms: 200,
        };
        let actuator = SelectActuator::with_config(config);

        assert_eq!(actuator.config().border_color, 0xFF0000FF);
        assert_eq!(actuator.config().border_width, 3.0);
    }
}
