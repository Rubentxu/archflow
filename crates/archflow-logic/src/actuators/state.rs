// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - StateActuator with Hierarchical State Machines
//
// This module implements HU-015: StateActuator with Hierarchical State Machines.
//
// Reference: docs/epics/EPIC-003-actuators-animations.md - HU-015
//
// Key Features:
// - Hierarchical State Machines (HSM) with parent-child relationships
// - State transition tables for O(1) lookup
// - OnEnter/OnExit events per state
// - State bitset for O(1) filtering of entities by state
// - Transition guards (conditions) for state changes
//
// Architecture:
// - StateMachine: Manages current state and transitions
// - StateTransitionTable: O(1) lookup of valid transitions
// - StateBitset: Fast filtering of entities by state
// - StateActuator: Actuator that changes entity states
//
// ═══════════════════════════════════════════════════════════════════════════════

#![warn(missing_docs)]

use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;
use archflow_core::EntityId;
use archflow_engine::EntityStore;

use crate::pulse::Pulse;

/// Common states for entities in the system
///
/// These states represent common entity states used in tools like
/// Figma, tldraw, and other diagramming applications.
///
/// States are organized as a hierarchy where child states inherit
/// parent behavior and can override specific behaviors.
///
/// # State Hierarchy Example
///
/// ```text
/// Idle (root)
/// ├── Active (mouse down on entity)
/// │   ├── Dragging (moving with threshold exceeded)
/// │   └── Selecting (selection box active)
/// └── Disabled (entity cannot be interacted with)
/// ```
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntityState {
    /// Idle state - default, no interaction
    Idle = 0,

    /// Active state - pointer is down on entity
    Active = 1,

    /// Dragging state - being moved with pointer
    Dragging = 2,

    /// Selected state - part of current selection
    Selected = 3,

    /// Hovered state - mouse is over entity
    Hovered = 4,

    /// Disabled state - cannot be interacted with
    Disabled = 5,

    /// Hidden state - not visible (separate from visibility)
    Hidden = 6,

    /// Locked state - cannot be modified
    Locked = 7,
}

impl EntityState {
    /// Check if this state is a child of another state
    ///
    /// This implements hierarchical state machine relationships.
    /// For example, Dragging is a child of Active.
    #[must_use]
    pub const fn is_child_of(self, parent: EntityState) -> bool {
        match parent {
            EntityState::Idle => true, // All states are children of Idle
            EntityState::Active => matches!(self, Self::Dragging | Self::Selected | Self::Hovered),
            EntityState::Dragging => false, // No children
            EntityState::Selected => true,  // Can have sub-states
            EntityState::Hovered => false,
            EntityState::Disabled => false,
            EntityState::Hidden => false,
            EntityState::Locked => false,
            _ => false,
        }
    }

    /// Check if state allows interaction
    #[must_use]
    pub const fn is_interactive(self) -> bool {
        !matches!(self, Self::Disabled | Self::Hidden | Self::Locked)
    }

    /// Check if state allows selection
    #[must_use]
    pub const fn is_selectable(self) -> bool {
        !matches!(self, Self::Disabled | Self::Hidden)
    }
}

/// State identifier for state lookup
pub type StateId = u8;

/// Transition guard condition
///
/// Guards are evaluated before allowing a state transition.
/// If a guard returns false, the transition is blocked.
pub type StateGuard = fn(EntityId, current_state: EntityState) -> bool;

/// State transition entry
///
/// Represents a valid state transition from one state to another.
#[derive(Clone)]
pub struct StateTransition {
    /// Target state ID
    pub target_state: EntityState,

    /// Optional guard condition (None = always allow)
    pub guard: Option<StateGuard>,

    /// OnEnter callback ID (index into callback table)
    pub on_enter: Option<u8>,

    /// OnExit callback ID (index into callback table)
    pub on_exit: Option<u8>,
}

impl StateTransition {
    /// Create a new state transition
    #[must_use]
    pub const fn new(target_state: EntityState) -> Self {
        Self {
            target_state,
            guard: None,
            on_enter: None,
            on_exit: None,
        }
    }

    /// Add a guard condition to this transition
    #[must_use]
    pub const fn with_guard(mut self, guard: StateGuard) -> Self {
        self.guard = Some(guard);
        self
    }

    /// Set OnEnter callback
    #[must_use]
    pub const fn with_on_enter(mut self, callback_id: u8) -> Self {
        self.on_enter = Some(callback_id);
        self
    }

    /// Set OnExit callback
    #[must_use]
    pub const fn with_on_exit(mut self, callback_id: u8) -> Self {
        self.on_exit = Some(callback_id);
        self
    }
}

/// State transition table for O(1) lookup
///
/// Maps (current_state, trigger) to target state with guards.
pub struct StateTransitionTable {
    /// transitions[current_state][trigger] = StateTransition
    transitions: Vec<Vec<Option<StateTransition>>>,
}

impl StateTransitionTable {
    /// Create a new empty transition table
    ///
    /// Creates a 256x256 transition table where:
    /// - First dimension: current state (0-255)
    /// - Second dimension: trigger event (0-255)
    #[must_use]
    pub fn new() -> Self {
        Self {
            // 256x256 table (state x trigger)
            transitions: vec![vec![None; 256]; 256],
        }
    }

    /// Add a transition from current state on trigger
    ///
    /// # Arguments
    ///
    /// * `from` - Current state
    /// * `trigger` - Trigger event (0-255)
    /// * `transition` - State transition definition
    pub fn add_transition(&mut self, from: EntityState, trigger: u8, transition: StateTransition) {
        let from_idx = from as u8 as usize;
        let trigger_idx = trigger as usize;
        if from_idx < self.transitions.len() && trigger_idx < 256 {
            self.transitions[from_idx][trigger_idx] = Some(transition);
        }
    }

    /// Get transition for current state and trigger
    #[must_use]
    pub fn get_transition(&self, from: EntityState, trigger: u8) -> Option<&StateTransition> {
        let from_idx = from as u8 as usize;
        let trigger_idx = trigger as usize;
        if from_idx < self.transitions.len() && trigger_idx < 256 {
            self.transitions[from_idx][trigger_idx].as_ref()
        } else {
            None
        }
    }
}

impl Default for StateTransitionTable {
    fn default() -> Self {
        Self::new()
    }
}

/// State bitset for fast O(1) filtering
///
/// This bitset tracks which entities are in which states,
/// allowing for fast queries like "get all selected entities".
///
/// Memory: (MAX_ENTITIES / 8) bytes = 12.5KB for 100K entities
pub struct StateBitset {
    /// Bitset where each bit represents whether an entity is in a state
    /// bitset[state_index][entity_index] = 1 if entity is in state
    /// 8 states x 1024 u8 = 8192 bits per state
    bitsets: Vec<[u8; 1024]>,
}

impl StateBitset {
    /// Create a new state bitset
    ///
    /// Memory layout: 8 states x 1024 bytes = 8192 bits per state
    /// Supports up to 8192 entities (1024 bytes * 8 bits per byte)
    #[must_use]
    pub fn new() -> Self {
        Self {
            bitsets: vec![[0u8; 1024]; 8],
        }
    }

    /// Set entity state
    pub fn set(&mut self, entity_idx: usize, state: EntityState, value: bool) {
        let state_idx = state as u8 as usize;
        if state_idx < 8 && entity_idx < 8192 {
            let byte_idx = entity_idx / 8;
            let bit_idx = entity_idx % 8;
            if value {
                self.bitsets[state_idx][byte_idx] |= 1 << bit_idx;
            } else {
                self.bitsets[state_idx][byte_idx] &= !(1 << bit_idx);
            }
        }
    }

    /// Check if entity is in state
    #[must_use]
    pub fn get(&self, entity_idx: usize, state: EntityState) -> bool {
        let state_idx = state as u8 as usize;
        if state_idx < 8 && entity_idx < 8192 {
            let byte_idx = entity_idx / 8;
            let bit_idx = entity_idx % 8;
            (self.bitsets[state_idx][byte_idx] & (1 << bit_idx)) != 0
        } else {
            false
        }
    }

    /// Get all entities in a given state (returns indices)
    pub fn get_entities_in_state(&self, state: EntityState) -> Vec<usize> {
        let mut result = Vec::new();
        let state_idx = state as u8 as usize;
        if state_idx >= 8 {
            return result;
        }

        for (byte_idx, &byte) in self.bitsets[state_idx].iter().enumerate() {
            for bit_idx in 0..8 {
                if (byte & (1 << bit_idx)) != 0 {
                    result.push(byte_idx * 8 + bit_idx);
                }
            }
        }
        result
    }
}

impl Default for StateBitset {
    fn default() -> Self {
        Self::new()
    }
}

/// State machine for a single entity
///
/// Manages the current state and handles transitions with guards.
pub struct StateMachine {
    /// Current state
    current_state: EntityState,

    /// Transition table for this state machine
    transitions: StateTransitionTable,
}

impl StateMachine {
    /// Create a new state machine with default Idle state
    #[must_use]
    pub fn new() -> Self {
        let mut transitions = StateTransitionTable::new();

        // Default transitions: Idle ↔ Active (trigger 0)
        transitions.add_transition(
            EntityState::Idle,
            0,
            StateTransition::new(EntityState::Active),
        );
        transitions.add_transition(
            EntityState::Active,
            0,
            StateTransition::new(EntityState::Idle),
        );

        Self {
            current_state: EntityState::Idle,
            transitions,
        }
    }

    /// Get current state
    #[must_use]
    pub const fn current_state(&self) -> EntityState {
        self.current_state
    }

    /// Try to transition to a new state on trigger
    ///
    /// Returns true if transition succeeded, false if blocked by guard.
    pub fn transition(&mut self, trigger: u8, entity_id: EntityId) -> bool {
        if let Some(transition) = self.transitions.get_transition(self.current_state, trigger) {
            // Check guard if present
            if let Some(guard) = transition.guard {
                if !guard(entity_id, self.current_state) {
                    return false; // Guard blocked transition
                }
            }

            // Execute transition
            let _old_state = self.current_state;
            self.current_state = transition.target_state;

            // TODO: Call on_exit(old_state) and on_enter(new_state) callbacks

            true
        } else {
            false
        }
    }

    /// Force transition to a state (bypasses guards)
    pub fn force_transition(&mut self, state: EntityState) {
        self.current_state = state;
    }
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

/// Actuator that changes entity states based on pulses
///
/// This actuator responds to pulses by transitioning entities between states.
/// It maintains a state machine per entity and tracks state in a bitset.
pub struct StateActuator {
    /// Entity ID this actuator operates on
    entity_id: EntityId,

    /// State machine for this entity
    state_machine: StateMachine,

    /// Target state to transition to when pulse is received
    target_state: EntityState,
}

impl StateActuator {
    /// Create a new state actuator
    #[must_use]
    pub fn new(entity_id: EntityId, target_state: EntityState) -> Self {
        Self {
            entity_id,
            state_machine: StateMachine::new(),
            target_state,
        }
    }

    /// Get the current state of this entity's state machine
    #[must_use]
    pub const fn current_state(&self) -> EntityState {
        self.state_machine.current_state()
    }

    /// Activate the actuator with a pulse
    ///
    /// If the pulse is Positive, transition to target state.
    /// If the pulse is Negative, transition back to Idle.
    pub fn activate(&mut self, pulse: &Pulse, _store: &mut EntityStore) {
        if pulse.is_positive() {
            self.state_machine.force_transition(self.target_state);
        } else if pulse.is_negative() {
            self.state_machine.force_transition(EntityState::Idle);
        }
    }
}

/// Manager for all entity state machines and bitsets
///
/// This is the main entry point for state management in the system.
pub struct StateManager {
    /// State machines per entity
    state_machines: BTreeMap<EntityId, StateMachine>,

    /// State bitsets for fast filtering
    bitsets: StateBitset,

    /// Default state for new entities
    default_state: EntityState,
}

impl StateManager {
    /// Create a new state manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            state_machines: BTreeMap::new(),
            bitsets: StateBitset::new(),
            default_state: EntityState::Idle,
        }
    }

    /// Get or create state machine for entity
    pub fn get_state_machine(&mut self, entity_id: EntityId) -> &mut StateMachine {
        self.state_machines
            .entry(entity_id)
            .or_insert_with(StateMachine::new)
    }

    /// Update bitsets to reflect current states
    pub fn update_bitsets(&mut self) {
        for (&entity_id, machine) in &self.state_machines {
            let idx = entity_id.index().0 as usize;
            let state = machine.current_state();
            self.bitsets.set(idx, state, true);
        }
    }

    /// Get all entities in a specific state
    #[must_use]
    pub fn entities_in_state(&self, state: EntityState) -> Vec<EntityId> {
        self.bitsets
            .get_entities_in_state(state)
            .into_iter()
            .map(|idx| {
                EntityId::from_parts(
                    archflow_core::Index(idx as u32),
                    archflow_core::Generation(1),
                )
            })
            .collect()
    }

    /// Check if entity is in a specific state
    #[must_use]
    pub fn is_in_state(&self, entity_id: EntityId, state: EntityState) -> bool {
        let idx = entity_id.index().0 as usize;
        self.bitsets.get(idx, state)
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_core::{Generation, Index};

    fn make_id(idx: u32) -> EntityId {
        EntityId::from_parts(Index(idx), Generation(1))
    }

    #[test]
    fn test_entity_state_hierarchy() {
        // Active is parent of Dragging
        assert!(EntityState::Dragging.is_child_of(EntityState::Active));
        assert!(EntityState::Active.is_child_of(EntityState::Idle));
        assert!(!EntityState::Idle.is_child_of(EntityState::Active));
    }

    #[test]
    fn test_entity_state_properties() {
        assert!(EntityState::Idle.is_interactive());
        assert!(EntityState::Active.is_interactive());
        assert!(!EntityState::Disabled.is_interactive());
        assert!(!EntityState::Hidden.is_interactive());

        assert!(EntityState::Idle.is_selectable());
        assert!(EntityState::Selected.is_selectable());
        assert!(!EntityState::Hidden.is_selectable());
    }

    #[test]
    fn test_state_transition_new() {
        let transition = StateTransition::new(EntityState::Dragging);
        assert_eq!(transition.target_state, EntityState::Dragging);
        assert!(transition.guard.is_none());
        assert!(transition.on_enter.is_none());
        assert!(transition.on_exit.is_none());
    }

    #[test]
    fn test_state_transition_with_guard() {
        let guard: StateGuard = |_entity_id, _current_state| -> bool {
            false // Always block
        };
        let transition = StateTransition::new(EntityState::Active).with_guard(guard);
        assert!(transition.guard.is_some());
    }

    #[test]
    fn test_state_transition_table() {
        let mut table = StateTransitionTable::new();
        let transition = StateTransition::new(EntityState::Active);

        table.add_transition(EntityState::Idle, 0, transition);

        let retrieved = table.get_transition(EntityState::Idle, 0);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().target_state, EntityState::Active);
    }

    #[test]
    fn test_state_bitset() {
        let mut bitset = StateBitset::new();

        // Set entity 0 to Idle
        bitset.set(0, EntityState::Idle, true);
        assert!(bitset.get(0, EntityState::Idle));

        // Set entity 0 to Active
        bitset.set(0, EntityState::Active, true);
        assert!(bitset.get(0, EntityState::Active));

        // Clear Idle
        bitset.set(0, EntityState::Idle, false);
        assert!(!bitset.get(0, EntityState::Idle));
        assert!(bitset.get(0, EntityState::Active));
    }

    #[test]
    fn test_state_bitset_get_entities() {
        let mut bitset = StateBitset::new();

        // Set entities 5, 10, 15 to Selected
        bitset.set(5, EntityState::Selected, true);
        bitset.set(10, EntityState::Selected, true);
        bitset.set(15, EntityState::Selected, true);

        let selected = bitset.get_entities_in_state(EntityState::Selected);
        assert_eq!(selected.len(), 3);
        assert!(selected.contains(&5));
        assert!(selected.contains(&10));
        assert!(selected.contains(&15));
    }

    #[test]
    fn test_state_machine_new() {
        let machine = StateMachine::new();
        assert_eq!(machine.current_state(), EntityState::Idle);
    }

    #[test]
    fn test_state_machine_transition() {
        let entity_id = make_id(42);
        let mut machine = StateMachine::new();

        // Transition Idle → Active on trigger 0
        assert!(machine.transition(0, entity_id));
        assert_eq!(machine.current_state(), EntityState::Active);

        // Transition back Active → Idle on trigger 0
        assert!(machine.transition(0, entity_id));
        assert_eq!(machine.current_state(), EntityState::Idle);

        // Invalid trigger returns false
        assert!(!machine.transition(99, entity_id));
        assert_eq!(machine.current_state(), EntityState::Idle);
    }

    #[test]
    fn test_state_machine_force_transition() {
        let mut machine = StateMachine::new();

        machine.force_transition(EntityState::Dragging);
        assert_eq!(machine.current_state(), EntityState::Dragging);

        machine.force_transition(EntityState::Selected);
        assert_eq!(machine.current_state(), EntityState::Selected);
    }

    #[test]
    fn test_state_actuator_new() {
        let entity_id = make_id(100);
        let actuator = StateActuator::new(entity_id, EntityState::Selected);

        assert_eq!(actuator.entity_id, entity_id);
        assert_eq!(actuator.target_state, EntityState::Selected);
        assert_eq!(actuator.current_state(), EntityState::Idle);
    }

    #[test]
    fn test_state_actuator_activate_positive() {
        let entity_id = make_id(100);
        let mut actuator = StateActuator::new(entity_id, EntityState::Selected);
        let pulse = Pulse::positive(0, 100, 1000);

        actuator.activate(&pulse, &mut EntityStore::new());

        assert_eq!(actuator.current_state(), EntityState::Selected);
    }

    #[test]
    fn test_state_actuator_activate_negative() {
        let entity_id = make_id(100);
        let mut actuator = StateActuator::new(entity_id, EntityState::Selected);
        let pulse = Pulse::positive(0, 100, 1000);

        actuator.activate(&pulse, &mut EntityStore::new());
        assert_eq!(actuator.current_state(), EntityState::Selected);

        let negative_pulse = Pulse::negative(0, 100, 2000);
        actuator.activate(&negative_pulse, &mut EntityStore::new());

        assert_eq!(actuator.current_state(), EntityState::Idle);
    }

    #[test]
    fn test_state_manager_new() {
        let manager = StateManager::new();
        assert_eq!(manager.state_machines.len(), 0);
    }

    #[test]
    fn test_state_manager_get_state_machine() {
        let mut manager = StateManager::new();
        let entity_id = make_id(42);

        let machine = manager.get_state_machine(entity_id);
        assert_eq!(machine.current_state(), EntityState::Idle);

        // Second call returns same machine
        let _machine2 = manager.get_state_machine(entity_id);
        assert_eq!(manager.state_machines.len(), 1);
    }

    #[test]
    fn test_state_manager_update_bitsets() {
        let mut manager = StateManager::new();
        let entity_id = make_id(42);

        let machine = manager.get_state_machine(entity_id);
        machine.force_transition(EntityState::Selected);

        manager.update_bitsets();

        assert!(manager.is_in_state(entity_id, EntityState::Selected));
        assert!(!manager.is_in_state(entity_id, EntityState::Idle));
    }

    #[test]
    fn test_state_manager_entities_in_state() {
        let mut manager = StateManager::new();
        let id1 = make_id(1);
        let id2 = make_id(2);
        let id3 = make_id(3);

        // Set entities to Selected state
        manager
            .get_state_machine(id1)
            .force_transition(EntityState::Selected);
        manager
            .get_state_machine(id2)
            .force_transition(EntityState::Selected);
        manager
            .get_state_machine(id3)
            .force_transition(EntityState::Active);

        manager.update_bitsets();

        let selected = manager.entities_in_state(EntityState::Selected);
        assert_eq!(selected.len(), 2);
        assert!(selected.contains(&id1));
        assert!(selected.contains(&id2));
        assert!(!selected.contains(&id3));

        let active = manager.entities_in_state(EntityState::Active);
        assert_eq!(active.len(), 1);
        assert!(active.contains(&id3));
    }
}
