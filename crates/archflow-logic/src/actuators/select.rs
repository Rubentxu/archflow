// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Select Actuator Implementation
//
// Epic 3.2: Select Actuator
// TDD Cycle: RED → GREEN → REFACTOR
//
// This actuator manages entity selection state by:
// - Tracking selected entities in a HashSet
// - Using EntityStore.set_selected() for visual feedback
// - Supporting Single/Multi/Replace selection modes
//
// Performance Characteristics:
// - O(1) selection/deselection (HashSet insert/remove)
// - O(n) clear operation (n = number of selected entities)
// - Zero-allocation during normal operation
//
// Memory Impact:
// - One HashSet entry per selected entity
// - Entries removed when entities are deselected
//
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::vec::Vec;

use archflow_core::EntityId;
use archflow_engine::EntityStore;

/// Selection mode for entity selection
///
/// # Examples
///
/// ```
/// use archflow_logic::actuators::SelectMode;
///
/// // Single mode: only one entity selected at a time
/// let mode = SelectMode::Single;
///
/// // Multi mode: multiple entities can be selected
/// let mode = SelectMode::Multi;
///
/// // Replace mode: clear previous selection and select new entity
/// let mode = SelectMode::Replace;
/// ```
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectMode {
    /// Select only this entity, deselect all others
    Single = 0,

    /// Add to selection (toggle if already selected)
    Multi = 1,

    /// Clear all and select only this entity
    Replace = 2,
}

/// Actuator that manages entity selection state
///
/// This actuator tracks which entities are currently selected and updates
/// the EntityStore metadata and visual feedback accordingly.
///
/// # Examples
///
/// ```
/// let mut store = EntityStore::new();
/// let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
///
/// let mut actuator = SelectActuator::new();
///
/// // Select entity in Single mode
/// actuator.update(&mut store, entity, true, SelectMode::Single);
/// assert!(actuator.is_selected(entity));
///
/// // Deselect entity
/// actuator.update(&mut store, entity, false, SelectMode::Single);
/// assert!(!actuator.is_selected(entity));
/// ```
pub struct SelectActuator {
    /// Set of currently selected entities
    selected: hashbrown::HashSet<EntityId>,
}

impl SelectActuator {
    /// Creates a new SelectActuator
    ///
    /// # Examples
    ///
    /// ```
    /// let actuator = SelectActuator::new();
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            selected: hashbrown::HashSet::new(),
        }
    }

    /// Updates the selection state for an entity
    ///
    /// # Arguments
    ///
    /// * `store` - Reference to EntityStore (for updating metadata/visual feedback)
    /// * `entity` - The entity to update
    /// * `active` - true to select, false to deselect
    /// * `mode` - Selection mode (Single/Multi/Replace)
    ///
    /// # Behavior
    ///
    /// - `SelectMode::Single`:
    ///   - If `active`: Deselect all others, select this entity
    ///   - If `!active`: Deselect this entity only
    ///
    /// - `SelectMode::Multi`:
    ///   - If `active`: Add to selection (if not already selected)
    ///   - If `!active`: Remove from selection
    ///
    /// - `SelectMode::Replace`:
    ///   - If `active`: Clear all, select this entity
    ///   - If `!active`: Remove from selection
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = EntityStore::new();
    /// let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
    /// let entity2 = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0));
    ///
    /// let mut actuator = SelectActuator::new();
    ///
    /// // Single mode: only entity2 is selected
    /// actuator.update(&mut store, entity1, true, SelectMode::Single);
    /// actuator.update(&mut store, entity2, true, SelectMode::Single);
    /// assert!(!actuator.is_selected(entity1));
    /// assert!(actuator.is_selected(entity2));
    ///
    /// // Multi mode: both selected
    /// actuator.update(&mut store, entity1, true, SelectMode::Multi);
    /// assert!(actuator.is_selected(entity1));
    /// assert!(actuator.is_selected(entity2));
    /// ```
    pub fn update(
        &mut self,
        store: &mut EntityStore,
        entity: EntityId,
        active: bool,
        mode: SelectMode,
    ) {
        if active {
            self.select(store, entity, mode);
        } else {
            self.deselect(store, entity);
        }
    }

    /// Select an entity according to the specified mode
    fn select(&mut self, store: &mut EntityStore, entity: EntityId, mode: SelectMode) {
        match mode {
            SelectMode::Single => {
                // Deselect all currently selected entities
                self.clear_all_internal(store);

                // Select this entity
                self.select_one(store, entity);
            }

            SelectMode::Multi => {
                // Add to selection (no-op if already selected)
                self.select_one(store, entity);
            }

            SelectMode::Replace => {
                // Clear all and select this entity
                self.clear_all_internal(store);
                self.select_one(store, entity);
            }
        }
    }

    /// Select a single entity (internal helper)
    fn select_one(&mut self, store: &mut EntityStore, entity: EntityId) {
        let idx = entity.index().0 as usize;

        // Check if already selected
        if self.selected.contains(&entity) {
            return;
        }

        // Add to selected set
        self.selected.insert(entity);

        // Update EntityStore metadata and visual feedback
        store.set_selected(idx, true);
    }

    /// Deselect an entity
    fn deselect(&mut self, store: &mut EntityStore, entity: EntityId) {
        // Remove from selected set
        if self.selected.remove(&entity) {
            // Update EntityStore metadata and visual feedback
            let idx = entity.index().0 as usize;
            store.set_selected(idx, false);
        }
    }

    /// Clear all selections (internal helper, doesn't clear the set)
    fn clear_all_internal(&mut self, store: &mut EntityStore) {
        for &entity in &self.selected {
            let idx = entity.index().0 as usize;
            store.set_selected(idx, false);
        }
        self.selected.clear();
    }

    /// Clear all selections and update EntityStore
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = EntityStore::new();
    /// let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
    /// let entity2 = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0));
    ///
    /// let mut actuator = SelectActuator::new();
    /// actuator.update(&mut store, entity1, true, SelectMode::Multi);
    /// actuator.update(&mut store, entity2, true, SelectMode::Multi);
    ///
    /// actuator.clear_all(&mut store);
    /// assert_eq!(actuator.selected_count(), 0);
    /// ```
    pub fn clear_all(&mut self, store: &mut EntityStore) {
        self.clear_all_internal(store);
    }

    /// Check if an entity is currently selected
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = EntityStore::new();
    /// let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
    ///
    /// let mut actuator = SelectActuator::new();
    /// assert!(!actuator.is_selected(entity));
    ///
    /// actuator.update(&mut store, entity, true, SelectMode::Single);
    /// assert!(actuator.is_selected(entity));
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn is_selected(&self, entity: EntityId) -> bool {
        self.selected.contains(&entity)
    }

    /// Get the number of currently selected entities
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = EntityStore::new();
    /// let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
    /// let entity2 = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0));
    ///
    /// let mut actuator = SelectActuator::new();
    /// assert_eq!(actuator.selected_count(), 0);
    ///
    /// actuator.update(&mut store, entity1, true, SelectMode::Multi);
    /// actuator.update(&mut store, entity2, true, SelectMode::Multi);
    /// assert_eq!(actuator.selected_count(), 2);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }

    /// Get a list of all currently selected entities
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = EntityStore::new();
    /// let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
    /// let entity2 = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0));
    ///
    /// let mut actuator = SelectActuator::new();
    /// actuator.update(&mut store, entity1, true, SelectMode::Multi);
    /// actuator.update(&mut store, entity2, true, SelectMode::Multi);
    ///
    /// let selected = actuator.selected_entities();
    /// assert_eq!(selected.len(), 2);
    /// ```
    #[must_use]
    pub fn selected_entities(&self) -> Vec<EntityId> {
        self.selected.iter().copied().collect()
    }
}

impl Default for SelectActuator {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS (inline for verification during development)
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_core::Vec2;

    #[test]
    fn test_actuator_initialization() {
        let actuator = SelectActuator::new();
        assert_eq!(actuator.selected_count(), 0);
    }

    #[test]
    fn test_default_trait() {
        let actuator = SelectActuator::default();
        assert_eq!(actuator.selected_count(), 0);
    }

    #[test]
    fn test_is_selected() {
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut actuator = SelectActuator::new();
        assert!(!actuator.is_selected(entity));

        actuator.update(&mut store, entity, true, SelectMode::Single);
        assert!(actuator.is_selected(entity));

        actuator.update(&mut store, entity, false, SelectMode::Single);
        assert!(!actuator.is_selected(entity));
    }

    #[test]
    fn test_selected_entities() {
        let mut store = EntityStore::new();
        let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let entity2 = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0));

        let mut actuator = SelectActuator::new();
        actuator.update(&mut store, entity1, true, SelectMode::Multi);
        actuator.update(&mut store, entity2, true, SelectMode::Multi);

        let selected = actuator.selected_entities();
        assert_eq!(selected.len(), 2);
        assert!(selected.contains(&entity1));
        assert!(selected.contains(&entity2));
    }

    #[test]
    fn test_clear_all() {
        let mut store = EntityStore::new();
        let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let entity2 = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0));

        let mut actuator = SelectActuator::new();
        actuator.update(&mut store, entity1, true, SelectMode::Multi);
        actuator.update(&mut store, entity2, true, SelectMode::Multi);
        assert_eq!(actuator.selected_count(), 2);

        actuator.clear_all(&mut store);
        assert_eq!(actuator.selected_count(), 0);
        assert!(!store.is_selected(entity1.index().0 as usize));
        assert!(!store.is_selected(entity2.index().0 as usize));
    }
}
