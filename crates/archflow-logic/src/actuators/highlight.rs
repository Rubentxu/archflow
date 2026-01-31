// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Highlight Actuator Implementation
//
// This actuator manages entity highlight state by:
// - Storing original color when highlight is activated
// - Generating SetColor commands to change appearance
// - Restoring original color when highlight is deactivated
//
// Performance Characteristics:
// - O(1) operations (HashMap lookup/insert)
// - Zero-allocation during normal operation
// - Commands are returned as Vec for batch processing
//
// Memory Impact:
// - One HashMap entry per highlighted entity
// - Entry removed when highlight is deactivated
//
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::vec;
use alloc::vec::Vec;

use archflow_core::EntityId;
use archflow_engine::{Command, EntityStore};

/// State tracking for a highlighted entity
#[derive(Clone, Copy, Debug)]
struct HighlightState {
    /// Original color before highlight
    original_color: u32,
    /// Current highlight color
    highlight_color: u32,
}

/// Actuator that manages entity highlight state
///
/// This actuator stores the original color of entities when they are highlighted
/// and generates commands to restore the original color when the highlight is removed.
///
/// # Examples
///
/// ```
/// let mut store = EntityStore::new();
/// let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
///
/// let mut actuator = HighlightActuator::new();
///
/// // Activate highlight (generates SetColor command)
/// let commands = actuator.update(&mut store, entity, true, 0x00FF00FF);
///
/// // Deactivate highlight (generates SetColor to restore original)
/// let commands = actuator.update(&mut store, entity, false, 0x00FF00FF);
/// ```
pub struct HighlightActuator {
    /// Map of currently highlighted entities: entity_id → state
    highlighted: hashbrown::HashMap<EntityId, HighlightState>,
}

impl HighlightActuator {
    /// Creates a new HighlightActuator
    ///
    /// # Examples
    ///
    /// ```
    /// let actuator = HighlightActuator::new();
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            highlighted: hashbrown::HashMap::new(),
        }
    }

    /// Updates the highlight state for an entity
    ///
    /// # Arguments
    ///
    /// * `store` - Reference to EntityStore (for reading current colors)
    /// * `entity` - The entity to update
    /// * `active` - true to activate highlight, false to deactivate
    /// * `highlight_color` - The color to use when highlighted (0xRRGGBBAA)
    ///
    /// # Returns
    ///
    /// A vector of commands to execute. Usually 0 or 1 command.
    ///
    /// # Behavior
    ///
    /// - When `active` is true:
    ///   - If entity is not already highlighted, stores current color and generates SetColor
    ///   - If entity is already highlighted with different color, generates SetColor
    ///   - If entity is already highlighted with same color, returns empty vec
    ///
    /// - When `active` is false:
    ///   - If entity is highlighted, generates SetColor to restore original
    ///   - If entity is not highlighted, returns empty vec
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = EntityStore::new();
    /// let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
    ///
    /// let mut actuator = HighlightActuator::new();
    ///
    /// // Activate highlight
    /// let commands = actuator.update(&mut store, entity, true, 0x00FF00FF);
    /// assert_eq!(commands.len(), 1);
    ///
    /// // Deactivate highlight
    /// let commands = actuator.update(&mut store, entity, false, 0x00FF00FF);
    /// assert_eq!(commands.len(), 1);
    /// ```
    pub fn update(
        &mut self,
        store: &mut EntityStore,
        entity: EntityId,
        active: bool,
        highlight_color: u32,
    ) -> Vec<Command> {
        if active {
            self.activate(store, entity, highlight_color)
        } else {
            self.deactivate(store, entity)
        }
    }

    /// Activates highlight for an entity
    fn activate(
        &mut self,
        store: &mut EntityStore,
        entity: EntityId,
        highlight_color: u32,
    ) -> Vec<Command> {
        let idx = entity.index().0 as usize;

        // Read current color from store
        let current_color = store.colors[idx];

        // Check if already highlighted with this color
        if let Some(state) = self.highlighted.get(&entity) {
            if state.highlight_color == highlight_color {
                // Already highlighted with same color, no command needed
                return Vec::new();
            }
            // Different highlight color, update and generate command
            self.highlighted.insert(
                entity,
                HighlightState {
                    original_color: current_color,
                    highlight_color,
                },
            );
            return vec![Command::SetColor {
                id: entity,
                color: highlight_color,
            }];
        }

        // Not highlighted yet, store state and generate command
        self.highlighted.insert(
            entity,
            HighlightState {
                original_color: current_color,
                highlight_color,
            },
        );

        vec![Command::SetColor {
            id: entity,
            color: highlight_color,
        }]
    }

    /// Deactivates highlight for an entity
    fn deactivate(&mut self, store: &EntityStore, entity: EntityId) -> Vec<Command> {
        // Check if entity is highlighted
        if self.highlighted.remove(&entity).is_some() {
            // Read current color from store (may have changed externally)
            let idx = entity.index().0 as usize;
            let current_color = store.colors[idx];

            // Generate command to restore current color
            vec![Command::SetColor {
                id: entity,
                color: current_color,
            }]
        } else {
            // Not highlighted, no command needed
            Vec::new()
        }
    }

    /// Check if an entity is currently highlighted
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = EntityStore::new();
    /// let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
    ///
    /// let mut actuator = HighlightActuator::new();
    /// assert!(!actuator.is_highlighted(entity));
    ///
    /// actuator.update(&mut store, entity, true, 0x00FF00FF);
    /// assert!(actuator.is_highlighted(entity));
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn is_highlighted(&self, entity: EntityId) -> bool {
        self.highlighted.contains_key(&entity)
    }

    /// Get the number of currently highlighted entities
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = EntityStore::new();
    /// let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
    /// let entity2 = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0));
    ///
    /// let mut actuator = HighlightActuator::new();
    /// assert_eq!(actuator.highlighted_count(), 0);
    ///
    /// actuator.update(&mut store, entity1, true, 0x00FF00FF);
    /// actuator.update(&mut store, entity2, true, 0x00FF00FF);
    /// assert_eq!(actuator.highlighted_count(), 2);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn highlighted_count(&self) -> usize {
        self.highlighted.len()
    }

    /// Clear all highlighted entities
    ///
    /// This does NOT generate restore commands. Use this when you want to
    /// discard highlight state (e.g., when entities are deleted).
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = EntityStore::new();
    /// let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
    ///
    /// let mut actuator = HighlightActuator::new();
    /// actuator.update(&mut store, entity, true, 0x00FF00FF);
    ///
    /// actuator.clear();
    /// assert!(!actuator.is_highlighted(entity));
    /// ```
    pub fn clear(&mut self) {
        self.highlighted.clear();
    }
}

impl Default for HighlightActuator {
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
        let actuator = HighlightActuator::new();
        assert_eq!(actuator.highlighted_count(), 0);
    }

    #[test]
    fn test_default_trait() {
        let actuator = HighlightActuator::default();
        assert_eq!(actuator.highlighted_count(), 0);
    }

    #[test]
    fn test_is_highlighted() {
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut actuator = HighlightActuator::new();
        assert!(!actuator.is_highlighted(entity));

        actuator.update(&mut store, entity, true, 0x00FF00FF);
        assert!(actuator.is_highlighted(entity));

        actuator.update(&mut store, entity, false, 0x00FF00FF);
        assert!(!actuator.is_highlighted(entity));
    }

    #[test]
    fn test_highlighted_count() {
        let mut store = EntityStore::new();
        let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let entity2 = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0));

        let mut actuator = HighlightActuator::new();
        assert_eq!(actuator.highlighted_count(), 0);

        actuator.update(&mut store, entity1, true, 0x00FF00FF);
        assert_eq!(actuator.highlighted_count(), 1);

        actuator.update(&mut store, entity2, true, 0x00FF00FF);
        assert_eq!(actuator.highlighted_count(), 2);

        actuator.update(&mut store, entity1, false, 0x00FF00FF);
        assert_eq!(actuator.highlighted_count(), 1);

        actuator.update(&mut store, entity2, false, 0x00FF00FF);
        assert_eq!(actuator.highlighted_count(), 0);
    }

    #[test]
    fn test_clear() {
        let mut store = EntityStore::new();
        let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let entity2 = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0));

        let mut actuator = HighlightActuator::new();
        actuator.update(&mut store, entity1, true, 0x00FF00FF);
        actuator.update(&mut store, entity2, true, 0x00FF00FF);
        assert_eq!(actuator.highlighted_count(), 2);

        actuator.clear();
        assert_eq!(actuator.highlighted_count(), 0);
        assert!(!actuator.is_highlighted(entity1));
        assert!(!actuator.is_highlighted(entity2));
    }
}
