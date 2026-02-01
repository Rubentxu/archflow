// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Actuators Module
//
// Actuators transform sensor signals into Commands that modify EntityStore state.
//
// Architecture:
// - Sensors (input) → Actuators (transformation) → Commands (output)
// - Actuators maintain state for undo/restore operations
// - Each actuator generates zero or more commands per update
// ═══════════════════════════════════════════════════════════════════════════════

pub mod highlight;
pub mod message;
pub mod move_;
pub mod property;
pub mod select;
pub mod state;

pub use highlight::HighlightActuator;
pub use message::{Message, MessageActuator, MessageBus, MessagePayload};
pub use move_::MoveActuator;
pub use property::{Property, PropertyActuator};
pub use select::{SelectActuator, SelectMode};
pub use state::{
    EntityState, StateActuator, StateBitset, StateId, StateMachine, StateManager, StateTransition,
    StateTransitionTable,
};
