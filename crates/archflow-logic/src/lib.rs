// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Signal Processing Module
//
// This crate provides the Logic Bricks system for ArchFlow Engine:
// - SignalByte: 6-tick history in 1 byte
// - Sensors: MouseOver, MouseClick, Proximity, KeyShortcut
// - Actuators: Highlight, Select, Move, etc.
// - Logic Mapping: Connect sensors to actuators with controllers
//
// Architecture Reference:
// - LOGIC_BRICKS_FEASIBILITY_STUDY.md
// - LOGIC_BRICKS_EPICS.md
// ═══════════════════════════════════════════════════════════════════════════════

#![no_std]
#![warn(missing_docs)]
#![warn(clippy::all)]

extern crate alloc;

pub mod actuators; // Epic 3: Actuators
pub mod sensors; // Epic 2: Sensors
pub mod signals; // Epic 1: SignalByte Foundation
                 // pub mod mapping;      // Epic 4: Logic Mapping

pub use actuators::{HighlightActuator, MoveActuator, SelectActuator, SelectMode};
pub use sensors::mouse_over::MouseOverSensor;
pub use signals::SignalByte;

// ═══════════════════════════════════════════════════════════════════════════════
// EPIC 1: SignalByte Foundation
// Status: IN PROGRESS
// ═══════════════════════════════════════════════════════════════════════════════
//
// [x] User Story 1.1: SignalByte with 6-tick history
//     [x] RED: Tests written in tests/signal_byte_tests.rs
//     [ ] GREEN: Implementation in src/signals.rs
//     [ ] REFACTOR: Optimization and documentation
//
// [ ] User Story 1.2: Edge Detection
//     [ ] RED: Write tests first
//     [ ] GREEN: Implement is_rising_edge(), is_falling_edge()
//     [ ] REFACTOR: Optimize bit patterns
//
// [ ] User Story 1.3: Pattern Matching
//     [ ] RED: Write tests first
//     [ ] GREEN: Implement is_steady(), count_ones(), count_zeros()
//     [ ] REFACTOR: Add const fn support
//
// ═══════════════════════════════════════════════════════════════════════════════
