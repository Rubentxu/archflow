// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Move Actuator Implementation
//
// Epic 3.3: Move Actuator with Hysteresis
// TDD Cycle: RED → GREEN → REFACTOR
//
// This actuator manages entity drag state with hysteresis:
// - Initiates drag only after 6 ticks of steady signal (prevents accidental drag)
// - Requires 6 ticks of 0 to release (prevents accidental release)
// - Generates Command::Move with delta from start position
//
// Performance Characteristics:
// - O(1) operations (HashMap lookup/insert)
// - Zero-allocation during normal operation
// - Commands returned as Vec for batch processing
//
// Memory Impact:
// - One HashMap entry per dragging entity
// - Entry removed when drag is released
//
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::vec;
use alloc::vec::Vec;

use archflow_core::{EntityId, Vec2};
use archflow_engine::{Command, EntityStore};

use crate::signals::SignalByte;

/// State tracking for a dragging entity
#[derive(Clone, Copy, Debug)]
struct DragState {
    /// Start position when drag began
    start_pos: Vec2,
}

/// Actuator that manages entity drag state with hysteresis
///
/// This actuator implements hysteresis to prevent accidental drag initiation
/// and release. The drag only starts after 6 ticks of steady signal and only
/// releases after 6 ticks of steady low signal.
///
/// # Examples
///
/// ```
/// let mut store = EntityStore::new();
/// let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
///
/// let mut actuator = MoveActuator::new();
/// let mut signal = SignalByte::default();
///
/// // Build up steady signal
/// for _ in 0..6 {
///     signal.push(true);
/// }
///
/// // Update with current mouse position
/// let commands = actuator.update(entity, signal, Vec2::new(120.0, 130.0), &store);
/// assert!(!commands.is_empty());
/// assert!(actuator.is_dragging(entity));
/// ```
pub struct MoveActuator {
    /// Map of currently dragging entities: entity_id → drag state
    dragging: hashbrown::HashMap<EntityId, DragState>,
}

impl MoveActuator {
    /// Creates a new MoveActuator
    ///
    /// # Examples
    ///
    /// ```
    /// let actuator = MoveActuator::new();
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            dragging: hashbrown::HashMap::new(),
        }
    }

    /// Updates the drag state for an entity
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to update
    /// * `signal` - SignalByte containing click state history
    /// * `mouse_pos` - Current mouse position (for calculating delta)
    /// * `store` - Reference to EntityStore (for reading entity positions)
    ///
    /// # Returns
    ///
    /// A vector of commands to execute. Usually 0 or 1 command.
    ///
    /// # Behavior
    ///
    /// - **Not dragging**: Check if signal is steady high for 6 ticks → start drag
    /// - **Dragging**:
    ///   - If signal is steady low for 6 ticks → end drag
    ///   - Otherwise → continue dragging, generate Move command
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = EntityStore::new();
    /// let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
    ///
    /// let mut actuator = MoveActuator::new();
    /// let mut signal = SignalByte::default();
    ///
    /// // Build steady signal
    /// for _ in 0..6 {
    ///     signal.push(true);
    /// }
    ///
    /// // Start drag
    /// let commands = actuator.update(entity, signal, Vec2::new(110.0, 105.0), &store);
    /// assert_eq!(commands.len(), 1);
    /// assert!(actuator.is_dragging(entity));
    /// ```
    pub fn update(
        &mut self,
        entity: EntityId,
        signal: SignalByte,
        mouse_pos: Vec2,
        store: &EntityStore,
    ) -> Vec<Command> {
        let is_dragging = self.dragging.contains_key(&entity);

        if is_dragging {
            self.update_dragging(entity, signal, mouse_pos)
        } else {
            self.try_start_drag(entity, signal, mouse_pos, store)
        }
    }

    /// Try to start dragging (only if signal is steady high for 6 ticks)
    fn try_start_drag(
        &mut self,
        entity: EntityId,
        signal: SignalByte,
        mouse_pos: Vec2,
        store: &EntityStore,
    ) -> Vec<Command> {
        // Check if signal is steady high for 6 ticks
        if !signal.is_steady_high(6) {
            return Vec::new();
        }

        // Get current entity position
        let idx = entity.index().0 as usize;
        let start_pos = Vec2::new(store.transforms[idx][0], store.transforms[idx][1]);

        // Start dragging
        self.dragging.insert(entity, DragState { start_pos });

        // Generate Move command with initial delta
        let delta = mouse_pos - start_pos;
        vec![Command::Move { id: entity, delta }]
    }

    /// Update while dragging
    fn update_dragging(
        &mut self,
        entity: EntityId,
        signal: SignalByte,
        mouse_pos: Vec2,
    ) -> Vec<Command> {
        // Check if signal is steady low for 6 ticks → end drag
        if signal.is_steady_low(6) {
            self.dragging.remove(&entity);
            return Vec::new();
        }

        // Continue dragging - generate Move command
        if let Some(state) = self.dragging.get(&entity) {
            let delta = mouse_pos - state.start_pos;
            return vec![Command::Move { id: entity, delta }];
        }

        Vec::new()
    }

    /// Check if an entity is currently being dragged
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = EntityStore::new();
    /// let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
    ///
    /// let mut actuator = MoveActuator::new();
    /// assert!(!actuator.is_dragging(entity));
    ///
    /// let mut signal = SignalByte::default();
    /// for _ in 0..6 {
    ///     signal.push(true);
    /// }
    ///
    /// actuator.update(entity, signal, Vec2::new(110.0, 105.0), &store);
    /// assert!(actuator.is_dragging(entity));
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn is_dragging(&self, entity: EntityId) -> bool {
        self.dragging.contains_key(&entity)
    }

    /// Get the number of currently dragging entities
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = EntityStore::new();
    /// let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
    /// let entity2 = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0));
    ///
    /// let mut actuator = MoveActuator::new();
    /// assert_eq!(actuator.dragging_count(), 0);
    ///
    /// // Start dragging entity1
    /// let mut signal = SignalByte::default();
    /// for _ in 0..6 {
    ///     signal.push(true);
    /// }
    /// actuator.update(entity1, signal, Vec2::new(110.0, 105.0), &store);
    /// assert_eq!(actuator.dragging_count(), 1);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn dragging_count(&self) -> usize {
        self.dragging.len()
    }

    /// Clear all dragging entities
    ///
    /// This does NOT generate any commands. Use this when you want to
    /// discard drag state (e.g., when entities are deleted).
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = EntityStore::new();
    /// let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
    ///
    /// let mut actuator = MoveActuator::new();
    /// let mut signal = SignalByte::default();
    /// for _ in 0..6 {
    ///     signal.push(true);
    /// }
    ///
    /// actuator.update(entity, signal, Vec2::new(110.0, 105.0), &store);
    ///
    /// actuator.clear();
    /// assert!(!actuator.is_dragging(entity));
    /// ```
    pub fn clear(&mut self) {
        self.dragging.clear();
    }
}

impl Default for MoveActuator {
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
        let actuator = MoveActuator::new();
        assert_eq!(actuator.dragging_count(), 0);
    }

    #[test]
    fn test_default_trait() {
        let actuator = MoveActuator::default();
        assert_eq!(actuator.dragging_count(), 0);
    }

    #[test]
    fn test_is_dragging() {
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut actuator = MoveActuator::new();
        assert!(!actuator.is_dragging(entity));

        let mut signal = SignalByte::default();
        for _ in 0..6 {
            signal.push(true);
        }

        actuator.update(entity, signal, Vec2::new(110.0, 105.0), &store);
        assert!(actuator.is_dragging(entity));
    }

    #[test]
    fn test_dragging_count() {
        let mut store = EntityStore::new();
        let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let entity2 = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0));

        let mut actuator = MoveActuator::new();
        assert_eq!(actuator.dragging_count(), 0);

        let mut signal1 = SignalByte::default();
        for _ in 0..6 {
            signal1.push(true);
        }
        actuator.update(entity1, signal1, Vec2::new(110.0, 105.0), &store);
        assert_eq!(actuator.dragging_count(), 1);

        let mut signal2 = SignalByte::default();
        for _ in 0..6 {
            signal2.push(true);
        }
        actuator.update(entity2, signal2, Vec2::new(210.0, 205.0), &store);
        assert_eq!(actuator.dragging_count(), 2);
    }

    #[test]
    fn test_clear() {
        let mut store = EntityStore::new();
        let entity1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let entity2 = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(50.0, 50.0));

        let mut actuator = MoveActuator::new();

        let mut signal1 = SignalByte::default();
        for _ in 0..6 {
            signal1.push(true);
        }
        actuator.update(entity1, signal1, Vec2::new(110.0, 105.0), &store);

        let mut signal2 = SignalByte::default();
        for _ in 0..6 {
            signal2.push(true);
        }
        actuator.update(entity2, signal2, Vec2::new(210.0, 205.0), &store);

        assert_eq!(actuator.dragging_count(), 2);

        actuator.clear();
        assert_eq!(actuator.dragging_count(), 0);
        assert!(!actuator.is_dragging(entity1));
        assert!(!actuator.is_dragging(entity2));
    }
}
