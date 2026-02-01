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
pub mod move_;
pub mod property;
pub mod select;

pub use highlight::HighlightActuator;
pub use move_::MoveActuator;
pub use property::{Property, PropertyActuator};
pub use select::{SelectActuator, SelectMode};
