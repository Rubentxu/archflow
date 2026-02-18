// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow SDK - Public API for Developers
//
// This crate provides the PUBLIC API that developers using the SDK will see.
// It follows EPIC-SDK-API specifications:
//
// - Sensor trait for custom sensors
// - Actuator trait for custom actuators
// - WiringBuilder for configuration
// - Snapper for snap-to-grid/entity/guide
//
// Key Principle: The public API IS the product.
// Developers don't see internal implementation, only this API.
//
// Architecture Reference: docs/epics/EPIC-SDK-PUBLIC-API.md
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(unused_imports)]

pub mod actuators;
pub mod sensors;
pub mod snap;
pub mod wiring;

// ═══════════════════════════════════════════════════════════════════════════════
// RE-EXPORTS - Public API
// ═══════════════════════════════════════════════════════════════════════════════

pub use actuators::{Actuator, ActuatorConfig, Pulse};
pub use sensors::{Sensor, SensorConfig, SensorContext, SensorState};
pub use snap::{SnapConfig, SnapGuide, Snapper};
pub use wiring::{Connection, WiringBuilder, WiringTable};

// ═══════════════════════════════════════════════════════════════════════════════
// PRELUDE - Common imports for convenience
// ═══════════════════════════════════════════════════════════════════════════════

pub mod prelude {
    pub use crate::actuators::*;
    pub use crate::sensors::*;
    pub use crate::snap::*;
    pub use crate::wiring::*;
}
