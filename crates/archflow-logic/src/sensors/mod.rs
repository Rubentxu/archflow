// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Sensors Module
//
// Provides sensors for detecting user input events.
//
// All sensors use SignalByte for 6-tick history, enabling edge detection
// (rising/falling edges) and pattern matching (steady, debounce).
// ═══════════════════════════════════════════════════════════════════════════════

//! Sensors for detecting user input events
//!
//! This module contains various sensors for detecting mouse, keyboard, and touch events.
//!
//! # SignalByte Integration
//!
//! All sensors use `SignalByte` internally to maintain a 6-tick history, enabling:
//! - **Edge Detection**: `is_rising_edge()` for trigger actions, `is_falling_edge()` for release
//! - **Pattern Matching**: `is_steady_high()` for hysteresis, `is_steady_low()` for debounce
//! - **Signal Analysis**: `count_ones()` for pulse counting, `has_noise()` for filtering
//!
//! # Example
//!
//! ```rust
//! use archflow_logic::sensors::MouseOverSensor;
//!
//! let mut sensor = MouseOverSensor::new();
//! let is_hover = sensor.sample(hit_detected, InputEventType::Move);
//!
//! if sensor.is_rising_edge() {
//!     // Entity just became hovered - trigger highlight
//! }
//! ```

pub mod collision;
pub mod double_tap;
pub mod key_shortcut;
pub mod long_press;
pub mod mouse;
pub mod mouse_click;
pub mod mouse_over;
pub mod proximity;
pub mod radar;
pub mod right_click;
pub mod touch;

pub use collision::CollisionSensor;
pub use double_tap::{DoubleTapSensor, DOUBLE_TAP_MS, TAP_TIMEOUT_MS};
pub use key_shortcut::KeyShortcutSensor;
pub use long_press::{LongPressSensor, LONG_PRESS_MS};
pub use mouse::{MouseConfig, MouseMode, MouseSensor};
pub use mouse_click::{MouseClickSensor, PointerButtons};
pub use mouse_over::MouseOverSensor;
pub use proximity::ProximitySensor;
pub use radar::{RadarAxis, RadarSensor};
pub use right_click::RightClickSensor;
pub use touch::TouchSensor;
