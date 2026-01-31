// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Actuators Module
//
// Epic 3: Actuadores Fundamentales
//
// Actuators transform sensor signals into Commands that modify EntityStore state.
// They follow the TDD approach: RED → GREEN → REFACTOR
//
// Architecture:
// - Sensors (input) → Actuators (transformation) → Commands (output)
// - Actuators maintain state for undo/restore operations
// - Each actuator generates zero or more commands per update
// ═══════════════════════════════════════════════════════════════════════════════

pub mod highlight;
pub mod mov;
pub mod select;

pub use highlight::HighlightActuator;
pub use mov::MoveActuator;
pub use select::{SelectActuator, SelectMode};
