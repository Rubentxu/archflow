// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow SDK - Public Sensor API
//
// This module defines the public Sensor trait that SDK developers use
// to create custom sensors.
//
// Reference: docs/epics/EPIC-SDK-PUBLIC-API.md - Section "API de Sensores"
// ═════════════════════════════════════════════════════════════════════════════

/// Sensor state output
///
/// Represents the output of a sensor evaluation.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SensorState {
    /// No pulse - sensor state didn't change, or is inactive
    None = 0,

    /// Positive pulse - sensor condition is TRUE (trigger actuators)
    Positive = 1,

    /// Negative pulse - sensor condition is FALSE (stop actuators)
    Negative = 2,
}

impl SensorState {
    /// Returns true if this is a positive pulse
    #[must_use]
    pub const fn is_positive(&self) -> bool {
        matches!(self, Self::Positive)
    }

    /// Returns true if this is a negative pulse
    #[must_use]
    pub const fn is_negative(&self) -> bool {
        matches!(self, Self::Negative)
    }

    /// Returns true if this is any pulse (positive or negative)
    #[must_use]
    pub fn is_pulse(&self) -> bool {
        *self != Self::None
    }

    /// Converts a bool to a SensorState (for simple sensors)
    #[must_use]
    pub const fn from_bool(value: bool) -> Self {
        if value {
            Self::Positive
        } else {
            Self::None
        }
    }
}

/// Context provided during sensor evaluation
///
/// This struct contains all the information a sensor needs to evaluate
/// its condition. It's passed by reference to minimize copying.
#[derive(Clone, Copy)]
pub struct SensorContext<'a> {
    /// Reference to the EntityStore (read-only access)
    /// Sensors can query entity properties but cannot modify them
    pub store: &'a archflow_engine::EntityStore,

    /// Current input snapshot (mouse, keyboard, etc.)
    pub input: &'a InputSnapshot,

    /// Current frame timestamp
    pub timestamp: u32,
}

/// Snapshot of input state at a specific moment in time
///
/// This is a zero-copy representation of the current input state.
/// For WASM targets, this maps to a SharedArrayBuffer for <2ms latency.
#[derive(Clone, Copy, Debug)]
pub struct InputSnapshot {
    /// Mouse position in screen coordinates
    pub mouse_position: (f32, f32),

    /// Mouse button state (bitmask: bit 0 = left, bit 1 = right, bit 2 = middle)
    pub mouse_buttons: u8,

    /// Keyboard modifier state (bitmask: bit 0 = shift, bit 1 = ctrl, bit 2 = alt)
    pub modifiers: u8,

    /// Mouse wheel delta (positive = up, negative = down)
    pub wheel_delta: i8,

    /// Timestamp when this snapshot was taken
    pub timestamp: u32,
}

impl InputSnapshot {
    /// Check if a specific mouse button is pressed
    #[inline]
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        match button {
            MouseButton::Left => (self.mouse_buttons & 0b001) != 0,
            MouseButton::Right => (self.mouse_buttons & 0b010) != 0,
            MouseButton::Middle => (self.mouse_buttons & 0b100) != 0,
        }
    }

    /// Check if shift key is held
    #[inline]
    pub fn is_shift_pressed(&self) -> bool {
        (self.modifiers & 0b001) != 0
    }

    /// Check if ctrl key is held
    #[inline]
    pub fn is_ctrl_pressed(&self) -> bool {
        (self.modifiers & 0b010) != 0
    }

    /// Check if alt key is held
    #[inline]
    pub fn is_alt_pressed(&self) -> bool {
        (self.modifiers & 0b100) != 0
    }
}

/// Mouse button enumeration
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Configuration for a sensor
///
/// This struct contains metadata about a sensor that's useful
/// for debugging, UI display, and serialization.
#[derive(Clone, Debug)]
pub struct SensorConfig {
    /// Human-readable name for this sensor
    pub name: String,

    /// Sensor type identifier
    pub sensor_type: SensorType,

    /// Whether this sensor is enabled
    pub enabled: bool,
}

/// Types of sensors supported by the SDK
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SensorType {
    /// Mouse-based sensor (click, hover, etc.)
    Mouse,

    /// Keyboard-based sensor (key press, shortcut)
    Keyboard,

    /// Proximity-based sensor (distance detection)
    Proximity,

    /// Time-based sensor (timer, interval)
    Timer,

    /// Custom sensor type
    Custom(u8),
}

/// Trait that all custom sensors must implement
///
/// This is the PRIMARY way SDK developers extend the input system.
/// The sensor evaluates its condition and returns a state that indicates
/// whether it should trigger connected actuators.
///
/// # Example
///
/// This example shows how to implement a simple proximity sensor that
/// triggers when the mouse is near an entity:
///
/// ```rust
/// use archflow_sdk::sensors::{Sensor, SensorContext, SensorState, SensorConfig};
/// use archflow_core::Vec2;
/// use archflow_core::EntityId;
///
/// struct ProximitySensor {
///     entity_id: EntityId,
///     threshold: f32,
///     config: SensorConfig,
/// }
///
/// impl Sensor for ProximitySensor {
///     fn evaluate(&mut self, ctx: &SensorContext) -> SensorState {
///         let idx = self.entity_id.index().0 as usize;
///         let entity_pos: Vec2 = ctx.store.pos(idx);
///         // In a real implementation, you would get mouse_position from ctx.input
///         let mouse_pos = Vec2::new(100.0, 100.0);
///
///         let delta = entity_pos - mouse_pos;
///         let distance = delta.length();
///
///         if distance < self.threshold {
///             SensorState::Positive
///         } else {
///             SensorState::Negative
///         }
///     }
///
///     fn config(&self) -> &SensorConfig {
///         &self.config
///     }
/// }
/// ```
pub trait Sensor {
    /// Evaluate the sensor and return its current state
    ///
    /// This method is called every frame (or whenever input changes).
    /// It should efficiently determine if the sensor's condition is met.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The sensor context containing store, input, and timestamp
    ///
    /// # Returns
    ///
    /// * `SensorState::Positive` - Sensor condition is TRUE (trigger actuators)
    /// * `SensorState::Negative` - Sensor condition is FALSE (stop actuators)
    /// * `SensorState::None` - No change, don't trigger anything
    ///
    /// # Performance Notes
    ///
    /// - This is a HOT PATH - called every frame for every active sensor
    /// - Avoid allocations (use stack or pre-allocated buffers)
    /// - Use `#[inline]` for simple checks
    /// - Consider using `#[inline(always)]` for trivial implementations
    fn evaluate(&mut self, ctx: &SensorContext) -> SensorState;

    /// Return the sensor's configuration
    ///
    /// This provides metadata about the sensor for debugging and UI.
    fn config(&self) -> &SensorConfig;

    /// Optional: Reset sensor state
    ///
    /// Called when the sensor should reset its internal state.
    /// Default implementation does nothing.
    #[allow(unused_variables)]
    fn reset(&mut self) {
        // Default: no-op
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensor_state_from_bool() {
        assert_eq!(SensorState::from_bool(true), SensorState::Positive);
        assert_eq!(SensorState::from_bool(false), SensorState::None);
    }

    #[test]
    fn test_sensor_state_is_positive() {
        assert!(SensorState::Positive.is_positive());
        assert!(!SensorState::Negative.is_positive());
        assert!(!SensorState::None.is_positive());
    }

    #[test]
    fn test_sensor_state_is_pulse() {
        assert!(SensorState::Positive.is_pulse());
        assert!(SensorState::Negative.is_pulse());
        assert!(!SensorState::None.is_pulse());
    }

    #[test]
    fn test_input_snapshot_mouse_buttons() {
        let snapshot = InputSnapshot {
            mouse_position: (100.0, 200.0),
            mouse_buttons: 0b101, // left and middle pressed
            modifiers: 0,
            wheel_delta: 0,
            timestamp: 0,
        };

        assert!(snapshot.is_mouse_button_pressed(MouseButton::Left));
        assert!(!snapshot.is_mouse_button_pressed(MouseButton::Right));
        assert!(snapshot.is_mouse_button_pressed(MouseButton::Middle));
    }

    #[test]
    fn test_input_snapshot_modifiers() {
        let snapshot = InputSnapshot {
            mouse_position: (0.0, 0.0),
            mouse_buttons: 0,
            modifiers: 0b011, // shift and ctrl pressed
            wheel_delta: 0,
            timestamp: 0,
        };

        assert!(snapshot.is_shift_pressed());
        assert!(snapshot.is_ctrl_pressed());
        assert!(!snapshot.is_alt_pressed());
    }
}
