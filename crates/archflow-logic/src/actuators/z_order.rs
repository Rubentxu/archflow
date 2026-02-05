// ═══════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Z-Order Actuators
//
// Actuators for z-order manipulation: bring forward, send backward, bring to front, send to back.
// Implements US-023 from TEMA 5.
//
// Architecture:
// - ZOrderActuator: Commands for manipulating entity draw order
// - Uses EntityStore's draw_order Vec for O(1) position queries
// - Generates Command::ZOrder for undo/redo support
//
// Performance Characteristics:
// - O(1) for position queries
// - O(n) for reorder operations where n = entities after/before target
// ═══════════════════════════════════════════════════════════════════════════════════════

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use archflow_core::{EntityId, MAX_ENTITIES, Vec2};
use archflow_engine::{Command, EntityStore};

/// Z-order manipulation direction
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZOrderDirection {
    /// Move one step forward (up in z-stack)
    Forward,
    /// Move one step backward (down in z-stack)
    Backward,
    /// Move to top of z-stack (render on top)
    ToFront,
    /// Move to bottom of z-stack (render behind)
    ToBack,
}

/// Z-order operation data for undo/redo
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZOrderOp {
    /// Entity being moved
    entity: EntityId,
    /// Previous z-index
    previous_z: usize,
    /// New z-index
    new_z: usize,
    /// Direction of operation
    direction: ZOrderDirection,
}

/// Actuator for manipulating entity z-order (draw order).
///
/// Provides four operations:
/// - `forward()`: Move entity one position closer to front
/// - `backward()`: Move entity one position closer to back
/// - `to_front()`: Move entity to top of draw order
/// - `to_back()`: Move entity to bottom of draw order
///
/// # Performance
/// - Position queries: O(1)
/// - Reorder operations: O(n) where n = entities moved
///
/// # Example
///
/// ```
/// use archflow_logic::actuators::z_order::{ZOrderActuator, ZOrderDirection};
///
/// let mut actuator = ZOrderActuator::new();
/// let mut store = /* ... */;
/// let entity = /* ... */;
///
/// // Move entity to front
/// let cmds = actuator.to_front(entity, &mut store);
/// ```
pub struct ZOrderActuator {
    /// Batch mode for multiple entities
    batch_mode: bool,
}

impl ZOrderActuator {
    /// Creates a new ZOrderActuator
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self { batch_mode: false }
    }

    /// Creates a ZOrderActuator with batch mode enabled
    #[inline(always)]
    #[must_use]
    pub fn with_batch_mode(enabled: bool) -> Self {
        Self {
            batch_mode: enabled,
        }
    }

    /// Get the current z-index of an entity
    ///
    /// # Arguments
    ///
    /// * `entity` - Entity to query
    /// * `store` - EntityStore to read from
    ///
    /// # Returns
    ///
    /// Some(z_index) if entity is alive, None otherwise
    #[inline(always)]
    #[must_use]
    pub fn z_index(&self, entity: EntityId, store: &EntityStore) -> Option<usize> {
        let idx = entity.index().0 as usize;
        if idx >= MAX_ENTITIES as usize || !store.is_alive(entity) {
            return None;
        }
        store.draw_order.iter().position(|&x| x as usize == idx)
    }

    /// Move entity one step forward in z-order
    ///
    /// # Arguments
    ///
    /// * `entity` - Entity to move
    /// * `store` - EntityStore to modify
    ///
    /// # Returns
    ///
    /// Vector of ZOrder commands for undo/redo
    pub fn forward(&self, entity: EntityId, store: &mut EntityStore) -> Vec<Command> {
        let (old_z, new_z) = match self.calculate_move(entity, store, ZOrderDirection::Forward) {
            Some(pair) => pair,
            None => return Vec::new(),
        };

        self.apply_move(entity, old_z, new_z, store)
    }

    /// Move entity one step backward in z-order
    ///
    /// # Arguments
    ///
    /// * `entity` - Entity to move
    /// * `store` - EntityStore to modify
    ///
    /// # Returns
    ///
    /// Vector of ZOrder commands for undo/redo
    pub fn backward(&self, entity: EntityId, store: &mut EntityStore) -> Vec<Command> {
        let (old_z, new_z) = match self.calculate_move(entity, store, ZOrderDirection::Backward) {
            Some(pair) => pair,
            None => return Vec::new(),
        };

        self.apply_move(entity, old_z, new_z, store)
    }

    /// Move entity to the front of z-order (top of draw stack)
    ///
    /// # Arguments
    ///
    /// * `entity` - Entity to move
    /// * `store` - EntityStore to modify
    ///
    /// # Returns
    ///
    /// Vector of ZOrder commands for undo/redo
    pub fn to_front(&self, entity: EntityId, store: &mut EntityStore) -> Vec<Command> {
        let (old_z, new_z) = match self.calculate_move(entity, store, ZOrderDirection::ToFront) {
            Some(pair) => pair,
            None => return Vec::new(),
        };

        self.apply_move(entity, old_z, new_z, store)
    }

    /// Move entity to the back of z-order (bottom of draw stack)
    ///
    /// # Arguments
    ///
    /// * `entity` - Entity to move
    /// * `store` - EntityStore to modify
    ///
    /// # Returns
    ///
    /// Vector of ZOrder commands for undo/redo
    pub fn to_back(&self, entity: EntityId, store: &mut EntityStore) -> Vec<Command> {
        let (old_z, new_z) = match self.calculate_move(entity, store, ZOrderDirection::ToBack) {
            Some(pair) => pair,
            None => return Vec::new(),
        };

        self.apply_move(entity, old_z, new_z, store)
    }

    /// Calculate new z-position for an entity
    fn calculate_move(
        &self,
        entity: EntityId,
        store: &EntityStore,
        direction: ZOrderDirection,
    ) -> Option<(usize, usize)> {
        let idx = entity.index().0 as usize;
        if idx >= MAX_ENTITIES as usize || !store.is_alive(entity) {
            return None;
        }

        let current_z = match store.draw_order.iter().position(|&x| x as usize == idx) {
            Some(pos) => pos,
            None => return None,
        };

        let draw_order_len = store.draw_order.len();
        if draw_order_len <= 1 {
            return None; // Nothing to reorder
        }

        let new_z = match direction {
            ZOrderDirection::Forward => {
                if current_z >= draw_order_len - 1 {
                    return None; // Already at front
                }
                current_z + 1
            }
            ZOrderDirection::Backward => {
                if current_z == 0 {
                    return None; // Already at back
                }
                current_z.saturating_sub(1)
            }
            ZOrderDirection::ToFront => draw_order_len - 1,
            ZOrderDirection::ToBack => 0,
        };

        Some((current_z, new_z))
    }

    /// Apply z-order move to store
    fn apply_move(
        &self,
        entity: EntityId,
        old_z: usize,
        new_z: usize,
        store: &mut EntityStore,
    ) -> Vec<Command> {
        let idx = entity.index().0 as usize;

        // Remove from current position
        let removed = store.draw_order.remove(old_z);

        // Insert at new position
        store.draw_order.insert(new_z, removed);

        // Mark z-order dirty
        store.dirty_z_order = true;

        // Create command for undo/redo
        vec![Command::ZOrder {
            entity,
            old_z_index: old_z,
            new_z_index: new_z,
        }]
    }

    /// Move multiple entities together in z-order
    ///
    /// # Arguments
    ///
    /// * `entities` - Entities to move (all moved to same relative position)
    /// * `store` - EntityStore to modify
    /// * `direction` - Direction to move all entities
    ///
    /// # Returns
    ///
    /// Vector of ZOrder commands for undo/redo
    pub fn move_batch(
        &self,
        entities: &[EntityId],
        store: &mut EntityStore,
        direction: ZOrderDirection,
    ) -> Vec<Command> {
        if entities.is_empty() {
            return Vec::new();
        }

        let mut commands = Vec::with_capacity(entities.len());

        // Process entities from back to front for forward moves
        // Process from front to back for backward moves
        let mut indices: Vec<usize> = entities
            .iter()
            .filter_map(|&e| self.z_index(e, store))
            .collect();

        // For Forward/ToFront: process from highest z to lowest (descending)
        // For Backward/ToBack: process from lowest z to highest (ascending)
        // This prevents overwriting positions when moving multiple entities
        match direction {
            ZOrderDirection::Forward | ZOrderDirection::ToFront => {
                indices.sort_unstable_by(|a, b| b.cmp(a))
            }
            ZOrderDirection::Backward | ZOrderDirection::ToBack => {
                indices.sort_unstable_by(|a, b| a.cmp(b))
            }
        }

        for &z in &indices {
            let entity_id = EntityId::new(store.draw_order[z] as u32);
            let cmds = match direction {
                ZOrderDirection::Forward => self.forward(entity_id, store),
                ZOrderDirection::Backward => self.backward(entity_id, store),
                ZOrderDirection::ToFront => self.to_front(entity_id, store),
                ZOrderDirection::ToBack => self.to_back(entity_id, store),
            };
            commands.extend(cmds);
        }

        commands
    }

    /// Format notification message
    #[inline(always)]
    #[must_use]
    pub fn format_message(&self, count: usize, direction: ZOrderDirection) -> String {
        let action = match direction {
            ZOrderDirection::Forward => "brought forward",
            ZOrderDirection::Backward => "sent backward",
            ZOrderDirection::ToFront => "brought to front",
            ZOrderDirection::ToBack => "sent to back",
        };

        if count == 1 {
            format!("1 entity {}", action)
        } else {
            format!("{} entities {}", count, action)
        }
    }
}

impl Default for ZOrderActuator {
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

    // ═══════════════════════════════════════════════════════════════════════════════════
    // ZOrderActuator Tests
    // ═══════════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_z_index_single_entity() {
        let actuator = ZOrderActuator::new();
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let z = actuator.z_index(entity, &store);
        assert_eq!(z, Some(0));
    }

    #[test]
    fn test_z_index_multiple_entities() {
        let actuator = ZOrderActuator::new();
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let e2 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(20.0, 20.0));
        let e3 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(30.0, 30.0));

        assert_eq!(actuator.z_index(e1, &store), Some(0));
        assert_eq!(actuator.z_index(e2, &store), Some(1));
        assert_eq!(actuator.z_index(e3, &store), Some(2));
    }

    #[test]
    fn test_forward() {
        let actuator = ZOrderActuator::new();
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let e2 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(20.0, 20.0));
        let e3 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(30.0, 30.0));

        // e1 is at z=0, move forward
        let cmds = actuator.forward(e1, &mut store);

        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            Command::ZOrder {
                entity,
                old_z_index,
                new_z_index,
                ..
            } => {
                assert_eq!(entity, &e1);
                assert_eq!(*old_z_index, 0);
                assert_eq!(*new_z_index, 1);
            }
            _ => panic!("Expected ZOrder command"),
        }

        // Verify new order: e2, e1, e3
        assert_eq!(actuator.z_index(e1, &store), Some(1));
        assert_eq!(actuator.z_index(e2, &store), Some(0));
        assert_eq!(actuator.z_index(e3, &store), Some(2));
    }

    #[test]
    fn test_backward() {
        let actuator = ZOrderActuator::new();
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let e2 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(20.0, 20.0));
        let e3 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(30.0, 30.0));

        // e3 is at z=2, move backward
        let cmds = actuator.backward(e3, &mut store);

        assert_eq!(cmds.len(), 1);

        // Verify new order: e1, e3, e2
        assert_eq!(actuator.z_index(e1, &store), Some(0));
        assert_eq!(actuator.z_index(e3, &store), Some(1));
        assert_eq!(actuator.z_index(e2, &store), Some(2));
    }

    #[test]
    fn test_to_front() {
        let actuator = ZOrderActuator::new();
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let e2 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(20.0, 20.0));
        let e3 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(30.0, 30.0));

        // Move e1 to front
        let cmds = actuator.to_front(e1, &mut store);

        assert_eq!(cmds.len(), 1);

        // Verify new order: e2, e3, e1
        assert_eq!(actuator.z_index(e2, &store), Some(0));
        assert_eq!(actuator.z_index(e3, &store), Some(1));
        assert_eq!(actuator.z_index(e1, &store), Some(2));
    }

    #[test]
    fn test_to_back() {
        let actuator = ZOrderActuator::new();
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let e2 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(20.0, 20.0));
        let e3 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(30.0, 30.0));

        // Move e3 to back
        let cmds = actuator.to_back(e3, &mut store);

        assert_eq!(cmds.len(), 1);

        // Verify new order: e3, e1, e2
        assert_eq!(actuator.z_index(e3, &store), Some(0));
        assert_eq!(actuator.z_index(e1, &store), Some(1));
        assert_eq!(actuator.z_index(e2, &store), Some(2));
    }

    #[test]
    fn test_forward_already_front() {
        let actuator = ZOrderActuator::new();
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let e2 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(20.0, 20.0));

        // e2 is already at front (z=1)
        let cmds = actuator.forward(e2, &mut store);

        assert!(cmds.is_empty());
        // Order unchanged
        assert_eq!(actuator.z_index(e1, &store), Some(0));
        assert_eq!(actuator.z_index(e2, &store), Some(1));
    }

    #[test]
    fn test_backward_already_back() {
        let actuator = ZOrderActuator::new();
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let e2 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(20.0, 20.0));

        // e1 is already at back (z=0)
        let cmds = actuator.backward(e1, &mut store);

        assert!(cmds.is_empty());
        // Order unchanged
        assert_eq!(actuator.z_index(e1, &store), Some(0));
        assert_eq!(actuator.z_index(e2, &store), Some(1));
    }

    #[test]
    fn test_forward_single_entity() {
        let actuator = ZOrderActuator::new();
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));

        // Only one entity - can't move forward
        let cmds = actuator.forward(e1, &mut store);

        assert!(cmds.is_empty());
    }

    #[test]
    fn test_batch_forward() {
        let actuator = ZOrderActuator::new();
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let e2 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(20.0, 20.0));
        let e3 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(30.0, 30.0));

        // Move e1 and e2 forward together
        let cmds = actuator.move_batch(&[e1, e2], &mut store, ZOrderDirection::Forward);

        // e1 moves from 0->1, e2 moves from 1->2
        assert_eq!(cmds.len(), 2);

        // Verify order: e3, e1, e2
        assert_eq!(actuator.z_index(e3, &store), Some(0));
        assert_eq!(actuator.z_index(e1, &store), Some(1));
        assert_eq!(actuator.z_index(e2, &store), Some(2));
    }

    #[test]
    fn test_format_message() {
        let actuator = ZOrderActuator::new();

        assert_eq!(
            actuator.format_message(1, ZOrderDirection::Forward),
            "1 entity brought forward"
        );
        assert_eq!(
            actuator.format_message(3, ZOrderDirection::Forward),
            "3 entities brought forward"
        );
        assert_eq!(
            actuator.format_message(1, ZOrderDirection::ToBack),
            "1 entity sent to back"
        );
    }

    #[test]
    fn test_invalid_entity() {
        let actuator = ZOrderActuator::new();
        let store = EntityStore::new();
        let invalid_id =
            EntityId::from_parts(archflow_core::Index(99999), archflow_core::Generation(1));

        assert_eq!(actuator.z_index(invalid_id, &store), None);
    }

    #[test]
    fn test_empty_store() {
        let actuator = ZOrderActuator::new();
        let mut store = EntityStore::new();

        let e1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        store.despawn(e1);

        assert_eq!(actuator.z_index(e1, &store), None);
    }
}
