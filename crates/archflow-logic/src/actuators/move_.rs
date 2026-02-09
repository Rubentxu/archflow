// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Move Actuator Implementation
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

/// Axis constraint for drag operations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragAxis {
    /// No constraint (free movement)
    Both = 0,
    /// X-axis only
    X = 1,
    /// Y-axis only
    Y = 2,
}

/// State tracking for a dragging entity
#[derive(Clone, Copy, Debug)]
struct DragState {
    /// Original entity position when drag started (for calculating total delta)
    start_pos: Vec2,
    /// Last mouse position for tracking
    last_mouse_pos: Vec2,
    /// Axis constraint for this drag
    axis: DragAxis,
    /// Grid snap value (0 to disable)
    snap: f32,
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
    /// Per-entity drag configuration: entity_id → (axis, snap)
    config: hashbrown::HashMap<EntityId, (DragAxis, f32)>,
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
            config: hashbrown::HashMap::new(),
        }
    }

    /// Set the drag axis constraint for an entity
    ///
    /// # Arguments
    ///
    /// * `entity` - Entity to configure
    /// * `axis` - Axis constraint (X, Y, or Both)
    ///
    /// # Examples
    ///
    /// ```
    /// let mut actuator = MoveActuator::new();
    /// actuator.set_axis(entity, DragAxis::X);
    /// ```
    #[inline(always)]
    pub fn set_axis(&mut self, entity: EntityId, axis: DragAxis) {
        if let Some(config) = self.config.get_mut(&entity) {
            config.0 = axis;
        } else {
            self.config.insert(entity, (axis, 0.0));
        }
    }

    /// Set the grid snap value for an entity
    ///
    /// # Arguments
    ///
    /// * `entity` - Entity to configure
    /// * `snap` - Grid snap value (0 to disable)
    ///
    /// # Examples
    ///
    /// ```
    /// let mut actuator = MoveActuator::new();
    /// actuator.set_snap(entity, 10.0);
    /// ```
    #[inline(always)]
    pub fn set_snap(&mut self, entity: EntityId, snap: f32) {
        if let Some(config) = self.config.get_mut(&entity) {
            config.1 = snap;
        } else {
            self.config.insert(entity, (DragAxis::Both, snap));
        }
    }

    /// Clear the drag configuration for an entity
    ///
    /// Resets axis to Both and snap to 0.
    ///
    /// # Arguments
    ///
    /// * `entity` - Entity to clear configuration for
    ///
    /// # Examples
    ///
    /// ```
    /// let mut actuator = MoveActuator::new();
    /// actuator.set_axis(entity, DragAxis::X);
    /// actuator.set_snap(entity, 10.0);
    /// actuator.clear_config(entity);
    /// ```
    #[inline(always)]
    pub fn clear_config(&mut self, entity: EntityId) {
        self.config.remove(&entity);
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

        // Get entity position for calculating delta from start
        let idx = entity.index().0 as usize;
        let entity_pos = if idx < store.transforms.len() {
            Vec2::new(store.transforms[idx][0], store.transforms[idx][1])
        } else {
            Vec2::ZERO
        };

        // Calculate delta from entity start position to current mouse position
        let delta = mouse_pos - entity_pos;

        // Get default axis and snap from config
        let config = self
            .config
            .get(&entity)
            .copied()
            .unwrap_or((DragAxis::Both, 0.0));

        // Start dragging
        self.dragging.insert(
            entity,
            DragState {
                start_pos: entity_pos,
                last_mouse_pos: mouse_pos,
                axis: config.0,
                snap: config.1,
            },
        );

        // Generate Move command with delta from entity position to current mouse position
        vec![Command::Move {
            id: entity,
            delta: Self::apply_axis_constraint(delta, config.0),
        }]
    }

    /// Apply axis constraint to delta vector
    fn apply_axis_constraint(delta: Vec2, axis: DragAxis) -> Vec2 {
        match axis {
            DragAxis::X => Vec2::new(delta.x, 0.0),
            DragAxis::Y => Vec2::new(0.0, delta.y),
            DragAxis::Both => delta,
        }
    }

    /// Apply grid snapping to delta
    fn apply_snap(delta: Vec2, snap: f32) -> Vec2 {
        if snap <= 0.0 {
            return delta;
        }
        Vec2::new(
            (delta.x / snap).round() * snap,
            (delta.y / snap).round() * snap,
        )
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

        // Continue dragging - generate Move command with delta from start position
        if let Some(state) = self.dragging.get_mut(&entity) {
            let raw_delta = mouse_pos - state.start_pos;
            state.last_mouse_pos = mouse_pos;

            let constrained_delta = Self::apply_axis_constraint(raw_delta, state.axis);
            let snapped_delta = Self::apply_snap(constrained_delta, state.snap);

            if snapped_delta.x == 0.0 && snapped_delta.y == 0.0 {
                return Vec::new();
            }

            return vec![Command::Move {
                id: entity,
                delta: snapped_delta,
            }];
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

    /// Get all entities that are currently in a dragging state
    pub fn dragging_entities(&self) -> Vec<EntityId> {
        self.dragging.keys().copied().collect()
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

    #[test]
    fn test_relative_movement() {
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let mut actuator = MoveActuator::new();
        let mut signal = SignalByte::default();
        for _ in 0..6 {
            signal.push(true);
        }

        // Frame 1: Drag starts - should generate Move with delta from entity position
        let cmds = actuator.update(entity, signal, Vec2::new(110.0, 110.0), &store);
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            Command::Move { id, delta } => {
                assert_eq!(*id, entity);
                // Delta from entity position (100, 100) to mouse (110, 110)
                assert_eq!(delta.x, 10.0);
                assert_eq!(delta.y, 10.0);
            }
            _ => panic!("Expected Move command, got {:?}", cmds[0]),
        }

        // Frame 2: Mouse moves 5 more units X
        let cmds = actuator.update(entity, signal, Vec2::new(115.0, 110.0), &store);
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            Command::Move { id, delta } => {
                assert_eq!(*id, entity);
                // Delta from entity position (100, 100) to mouse (115, 110)
                assert_eq!(delta.x, 15.0);
                assert_eq!(delta.y, 10.0);
            }
            _ => panic!("Expected Move command, got {:?}", cmds[0]),
        }
    }
}
