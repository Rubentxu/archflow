// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Pulse System (Blender Game Engine Pattern)
//
// This module implements the pulse-based event system inspired by Blender's BGE.
// Sensors emit pulses (Positive/Negative/None) which flow through controllers
// to actuators. This is much more efficient than polling state.
//
// Architecture Reference:
// - KX_ISensor::Evaluate() - Original Blender implementation
// - SCA_ILogicController - Controller logic in Blender
//
// Key Insight: Sensors are PRODUCERS of pulses, not data stores.
// Controllers consume pulses and decide whether to forward them.
// Actuators respond to pulses by executing Commands.
//
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::vec::Vec;

/// Represents the output of a sensor evaluation
///
/// In Blender's BGE, sensors don't return bool - they return a state
/// that indicates whether they should trigger connected controllers.
///
/// - `None`: No change, don't trigger anything (most efficient!)
/// - `Positive`: Sensor condition became TRUE (or is TRUE with level triggering)
/// - `Negative`: Sensor condition became FALSE
///
/// This design means controllers only wake up when something relevant happens.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SensorState {
    /// No pulse - sensor state didn't change, or is inactive
    None = 0,

    /// Positive pulse - sensor condition is TRUE
    /// This should trigger connected actuators
    Positive = 1,

    /// Negative pulse - sensor condition changed from TRUE to FALSE
    /// This should STOP connected actuators
    Negative = 2,
}

impl SensorState {
    /// Returns true if this is a positive pulse
    #[must_use]
    pub const fn is_positive(self) -> bool {
        matches!(self, Self::Positive)
    }

    /// Returns true if this is a negative pulse
    #[must_use]
    pub const fn is_negative(self) -> bool {
        matches!(self, Self::Negative)
    }

    /// Returns true if this is any pulse (positive or negative)
    #[must_use]
    pub fn is_pulse(self) -> bool {
        self != Self::None
    }

    /// Converts a bool to a SensorState (for simple sensors)
    #[must_use]
    pub const fn from_bool(value: bool) -> Self {
        if value { Self::Positive } else { Self::None }
    }
}

/// A pulse event flowing through the logic system
///
/// This represents ONE pulse from ONE sensor to potentially multiple controllers.
/// In a collaborative system, pulses can be serialized and sent over the network.
///
/// # Memory Layout (optimized for WASM)
/// ```text
/// Total: 16 bytes
/// - sensor_id: u32 (4 bytes) - Which sensor emitted this
/// - entity_id: u32 (4 bytes) - Which entity this pulse affects
/// - state: SensorState (1 byte, but padded to 4 for alignment)
/// - timestamp: u32 (4 bytes) - When this pulse was emitted
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Pulse {
    /// Which sensor generated this pulse
    pub sensor_id: u32,

    /// Which entity this pulse is for
    pub entity_id: u32,

    /// The pulse state (Positive/Negative/None)
    pub state: SensorState,

    /// Timestamp for timing-sensitive logic
    pub timestamp: u32,
}

impl Pulse {
    /// Create a new positive pulse
    #[must_use]
    pub const fn positive(sensor_id: u32, entity_id: u32, timestamp: u32) -> Self {
        Self {
            sensor_id,
            entity_id,
            state: SensorState::Positive,
            timestamp,
        }
    }

    /// Create a new negative pulse
    #[must_use]
    pub const fn negative(sensor_id: u32, entity_id: u32, timestamp: u32) -> Self {
        Self {
            sensor_id,
            entity_id,
            state: SensorState::Negative,
            timestamp,
        }
    }

    /// Returns true if this is a positive pulse
    #[must_use]
    pub const fn is_positive(self) -> bool {
        self.state.is_positive()
    }

    /// Returns true if this is a negative pulse
    #[must_use]
    pub const fn is_negative(self) -> bool {
        self.state.is_negative()
    }

    /// Returns true if this is any pulse at all
    #[must_use]
    pub fn is_pulse(self) -> bool {
        self.state.is_pulse()
    }
}

/// The Pulse Bus - collects pulses from all sensors
///
/// This is the central event queue for the Logic Bricks system.
/// Sensors write pulses to the bus, and controllers read from it.
///
/// # Performance
///
/// - Zero-allocation: pulses are written to pre-allocated Vec
/// - Cache-friendly: sequential memory access
/// - Network-ready: pulses can be serialized for multiplayer
///
/// # Example
///
/// ```
/// let mut bus = PulseBus::new();
///
/// // Sensor emits a pulse
/// bus.push(Pulse::positive(0, entity_id, timestamp));
///
/// // Later, controllers process all pulses
/// for pulse in bus.drain() {
///     controller.process_pulse(pulse);
/// }
/// ```
pub struct PulseBus {
    /// Pulse events for this frame (cleared each tick)
    pulses: Vec<Pulse>,

    /// Current timestamp in milliseconds
    timestamp: u32,
}

impl PulseBus {
    /// Create a new pulse bus with pre-allocated capacity
    #[must_use]
    pub fn new() -> Self {
        Self {
            pulses: Vec::with_capacity(256), // Pre-allocate for common case
            timestamp: 0,
        }
    }

    /// Set the current timestamp
    pub fn set_timestamp(&mut self, timestamp: u32) {
        self.timestamp = timestamp;
    }

    /// Get the current timestamp
    #[inline(always)]
    #[must_use]
    pub fn timestamp(&self) -> u32 {
        self.timestamp
    }

    /// Push a pulse onto the bus
    ///
    /// This is called by sensors during evaluation.
    pub fn push(&mut self, pulse: Pulse) {
        self.pulses.push(pulse);
    }

    /// Push a positive pulse (convenience method)
    pub fn push_positive(&mut self, sensor_id: u32, entity_id: u32) {
        self.pulses
            .push(Pulse::positive(sensor_id, entity_id, self.timestamp));
    }

    /// Push a negative pulse (convenience method)
    pub fn push_negative(&mut self, sensor_id: u32, entity_id: u32) {
        self.pulses
            .push(Pulse::negative(sensor_id, entity_id, self.timestamp));
    }

    /// Get all pulses and clear the buffer
    ///
    /// This should be called once per frame by the logic dispatcher.
    pub fn drain(&mut self) -> Vec<Pulse> {
        let pulses = core::mem::take(&mut self.pulses);
        pulses
    }

    /// Clear all pulses (called at end of frame)
    pub fn clear(&mut self) {
        self.pulses.clear();
    }

    /// Returns true if there are any pulses this frame
    #[must_use]
    pub fn has_pulses(&self) -> bool {
        !self.pulses.is_empty()
    }

    /// Returns the number of pulses
    #[must_use]
    pub fn len(&self) -> usize {
        self.pulses.len()
    }
}

impl Default for PulseBus {
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

    #[test]
    fn test_sensor_state_from_bool() {
        assert_eq!(SensorState::from_bool(true), SensorState::Positive);
        assert_eq!(SensorState::from_bool(false), SensorState::None);
    }

    #[test]
    fn test_pulse_creation() {
        let pulse = Pulse::positive(0, 100, 1000);
        assert_eq!(pulse.sensor_id, 0);
        assert_eq!(pulse.entity_id, 100);
        assert!(pulse.is_positive());
        assert!(!pulse.is_negative());
    }

    #[test]
    fn test_pulse_bus() {
        let mut bus = PulseBus::new();
        bus.set_timestamp(100);

        assert!(!bus.has_pulses());
        assert_eq!(bus.len(), 0);

        bus.push_positive(0, 100);

        assert!(bus.has_pulses());
        assert_eq!(bus.len(), 1);

        let pulses = bus.drain();
        assert_eq!(pulses.len(), 1);
        assert!(pulses[0].is_positive());

        assert!(!bus.has_pulses()); // Cleared after drain
    }

    #[test]
    fn test_multiple_pulses() {
        let mut bus = PulseBus::new();
        bus.set_timestamp(100);

        bus.push_positive(0, 100);
        bus.push_negative(1, 200);
        bus.push_positive(0, 300);

        assert_eq!(bus.len(), 3);

        let pulses = bus.drain();
        assert!(pulses[0].is_positive());
        assert!(pulses[1].is_negative());
        assert!(pulses[2].is_positive());
    }
}
