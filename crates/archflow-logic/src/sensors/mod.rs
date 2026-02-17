// ═══════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Sensors Module
//
// Provides sensors for detecting user input events.
//
// All sensors use SignalByte for 6-tick history, enabling edge detection
// (rising/falling edges) and pattern matching (steady, debounce).
//
// # BGE Architecture Reference
//
// In BGE, each sensor inherits from SCA_ISensor which provides:
// - Signal history (6 ticks)
// - Pulse generation (frequency, tap, level parameters)
// - Edge detection (rising/falling edges)
//
// Reference: UPBGE source/gameengine/GameLogic/SCA_ISensor.cpp
// ═══════════════════════════════════════════════════════════════════════════════════════

//! Sensors for detecting user input events
//!
//! This module contains various sensors for detecting mouse, keyboard, and touch events.
//!
//! # Unified Mouse Sensor
//!
//! The `MouseSensor` is a unified sensor that handles ALL mouse interactions:
//! - Mouse movement (hover) → MouseMode::Movement
//! - Left click → MouseMode::LeftButton
//! - Right click → MouseMode::RightButton
//! - Middle click → MouseMode::MiddleButton
//! - Wheel scroll → MouseMode::WheelUp / WheelDown
//!
//! This follows BGE's pattern of having ONE mouse sensor class with configurable modes.
//! The old separate sensors (MouseClickSensor, RightClickSensor, etc.) are now deprecated.
//!
//! # BGE-Style Configuration
//!
//! Each sensor supports BGE-style pulse parameters:
//! - `skipped_ticks`: Delay between pulse emissions (0 = every tick)
//! - `tap`: Single pulse on true→false transition
//! - `invert`: Invert the output signal
//!
//! # Example
//!
//! ```rust
//! use archflow_logic::sensors::{MouseSensor, MouseConfig};
//!
//! // Create a mouse-over sensor (default)
//! let mut sensor = MouseSensor::with_config(
//!     store.capacity(),
//!     MouseConfig::movement()
//! );
//!
//! // Create a click sensor with tap mode (for double-click via SignalByte)
//! let mut click_sensor = MouseSensor::with_config(
//!     store.capacity(),
//!     MouseConfig::left_button().tap(true)
//! );
//!
//! // Create a right-click sensor
//! let mut right_click = MouseSensor::with_config(
//!     store.capacity(),
//!     MouseConfig::right_button()
//! );
//! ```

pub mod box_select;
pub mod collision;
pub mod key_shortcut;
pub mod mouse;
pub mod near;
pub mod proximity;
pub mod radar;
pub mod touch;

// Re-exports
pub use key_shortcut::KeyCode;

pub use box_select::{BoxSelectSensor, BoxSelection};
pub use collision::CollisionSensor;
pub use key_shortcut::KeyShortcutSensor;
pub use mouse::{MouseConfig, MouseMode, MouseSensor};
pub use near::NearSensor;
pub use proximity::ProximitySensor;
pub use radar::{RadarAxis, RadarSensor};
pub use touch::TouchSensor;

// Re-export SignalState for convenience (used by multiple sensors)
pub use crate::signals::SignalState;
