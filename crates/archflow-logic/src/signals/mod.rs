// ═══════════════════════════════════════════════════════════════════════════════════════
// Signals Module - Signal Processing Components
//
// This module provides signal types for the Logic Bricks system:
// - SignalByte: 6-tick history in 1 byte (bit-packed)
// - SignalState: Wrapper with BGE-style analysis methods
// - SignalChannelBuffer: FixedBitSet-based double-buffered channel management
//
// Reference: UPBGE source/gameengine/GameLogic/SCA_ISensor.cpp
// ═══════════════════════════════════════════════════════════════════════════════════════

use core::fmt;

// Re-export SignalByte and SignalState from inline definitions
// (Keeping the original signals.rs content inline for compatibility)

/// Signal byte with 6-tick history for edge detection and debouncing
///
/// # Layout
///
/// 
///
/// - T0: Current tick (most recent)
/// - T1-T5: Previous ticks (history)
/// - R: Reserved bits for future use
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct SignalByte(u8);

impl SignalByte {
    /// Creates a SignalByte from a raw u8 value
    #[inline(always)]
    #[must_use]
    pub const fn from(value: u8) -> Self {
        Self(value)
    }
    /// Creates a new SignalByte with all zeros
    #[inline(always)]
    #[must_use]
    pub const fn new() -> Self {
        Self(0)
    }
    /// Inserts a new state by shifting the history left
    #[inline(always)]
    pub fn push(&mut self, active: bool) {
        self.0 = (self.0 << 1) | (active as u8);
    }
    /// Returns the current state (tick T0, least significant bit)
    #[inline(always)]
    #[must_use]
    pub fn get_current(&self) -> bool {
        (self.0 & 1) != 0
    }
    /// Returns the previous state (tick T1)
    #[inline(always)]
    #[must_use]
    pub fn get_prev(&self) -> bool {
        (self.0 & 2) != 0
    }
    /// Returns true if the current signal state is positive (active high)
    #[inline(always)]
    #[must_use]
    pub fn is_positive(&self) -> bool {
        self.get_current()
    }
    /// Returns true if the current signal state is negative (active low)
    #[inline(always)]
    #[must_use]
    pub fn is_negative(&self) -> bool {
        !self.get_current()
    }
    /// Returns the 6-bit history (bits T5 through T0)
    #[inline(always)]
    #[must_use]
    pub fn get_history(&self) -> u8 {
        self.0 & 0b111111
    }
    /// Returns the raw u8 value (for serialization)
    #[inline(always)]
    #[must_use]
    pub const fn as_u8(&self) -> u8 {
        self.0
    }
    /// Detects rising edge: 0 in T-1, 1 in T
    #[inline(always)]
    #[must_use]
    pub fn is_rising_edge(&self) -> bool {
        (self.0 & 0b00000011) == 0b00000001
    }
    /// Detects falling edge: 1 in T-1, 0 in T
    #[inline(always)]
    #[must_use]
    pub fn is_falling_edge(&self) -> bool {
        (self.0 & 0b00000011) == 0b00000010
    }
    /// Returns true if there's any edge (rising or falling)
    #[inline(always)]
    #[must_use]
    pub fn any_edge(&self) -> bool {
        self.is_rising_edge() || self.is_falling_edge()
    }
    /// Checks if signal has been steadily high for the last N ticks
    #[inline(always)]
    #[must_use]
    pub fn is_steady_high(&self, ticks: u8) -> bool {
        let mask = ((1u8 << ticks).wrapping_sub(1)) & 0b111111;
        (self.0 & mask) == mask
    }
    /// Checks if signal has been steadily low for the last N ticks
    #[inline(always)]
    #[must_use]
    pub fn is_steady_low(&self, ticks: u8) -> bool {
        let mask = ((1u8 << ticks).wrapping_sub(1)) & 0b111111;
        (self.0 & mask) == 0
    }
    /// Alias for is_steady_high for compatibility
    #[inline(always)]
    #[must_use]
    pub fn is_steady(&self, ticks: u8) -> bool {
        self.is_steady_high(ticks)
    }
    /// Counts how many ticks in history are 1
    #[inline(always)]
    #[must_use]
    pub fn count_ones(&self) -> u8 {
        self.get_history().count_ones() as u8
    }
    /// Counts how many ticks in history are 0
    #[inline(always)]
    #[must_use]
    pub fn count_zeros(&self) -> u8 {
        6 - self.count_ones()
    }
    /// Detects double-click pattern: click-pause-pause-click-pause-pause
    #[inline(always)]
    #[must_use]
    pub fn is_double_click_pattern(&self) -> bool {
        (self.get_history() & 0b111111) == 0b00100100
    }
    /// Detects if signal has noise (frequent transitions)
    #[inline(always)]
    #[must_use]
    pub fn has_noise(&self) -> bool {
        let h = self.get_history();
        let transitions = (h ^ (h >> 1)).count_ones();
        transitions >= 3
    }
    /// Evaluates BGE-style output with pulse mode support
    #[inline(always)]
    #[must_use]
    pub fn evaluate_bge_output(&self, true_pulse: bool, false_pulse: bool) -> SensorOutput {
        let positive = self.get_current();
        let triggered = self.any_edge();
        let should_trigger = triggered || (true_pulse && positive) || (false_pulse && !positive);
        SensorOutput {
            state: positive,
            should_trigger,
        }
    }
    /// Evaluates with true pulse only (BGE tap mode)
    #[inline(always)]
    #[must_use]
    pub fn with_true_pulse(&self) -> SensorOutput {
        self.evaluate_bge_output(true, false)
    }
    /// Evaluates with false pulse only
    #[inline(always)]
    #[must_use]
    pub fn with_false_pulse(&self) -> SensorOutput {
        self.evaluate_bge_output(false, true)
    }
    /// Evaluates with both pulse modes
    #[inline(always)]
    #[must_use]
    pub fn with_both_pulses(&self) -> SensorOutput {
        self.evaluate_bge_output(true, true)
    }
}

impl From<u8> for SignalByte {
    #[inline(always)]
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<SignalByte> for u8 {
    #[inline(always)]
    fn from(signal: SignalByte) -> Self {
        signal.0
    }
}

impl fmt::Binary for SignalByte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Binary::fmt(&self.0, f)
    }
}

impl fmt::LowerHex for SignalByte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for SignalByte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

/// Signal State - Wrapper around SignalByte with BGE-style analysis methods
#[derive(Clone, Copy, Debug, Default)]
pub struct SignalState(SignalByte);

impl SignalState {
    /// Sample a new signal value (pushes to 6-tick history)
    #[inline(always)]
    pub fn sample(&mut self, active: bool) {
        self.0.push(active);
    }
    /// Get the current signal state (most recent sample)
    #[inline(always)]
    #[must_use]
    pub fn is_positive(&self) -> bool {
        self.0.get_current()
    }
    /// Get the previous signal state (second most recent sample)
    #[inline(always)]
    #[must_use]
    pub fn was_positive(&self) -> bool {
        self.0.get_prev()
    }
    /// Check for rising edge (false -> true transition)
    #[inline(always)]
    #[must_use]
    pub fn is_rising_edge(&self) -> bool {
        self.0.is_rising_edge()
    }
    /// Check for falling edge (true -> false transition)
    #[inline(always)]
    #[must_use]
    pub fn is_falling_edge(&self) -> bool {
        self.0.is_falling_edge()
    }
    /// Check for any edge (state change)
    #[inline(always)]
    #[must_use]
    pub fn any_edge(&self) -> bool {
        self.is_rising_edge() || self.is_falling_edge()
    }
    /// Check if signal has been stable (no edges) for at least N ticks
    #[inline(always)]
    #[must_use]
    pub fn is_stable(&self, stable_ticks: u8) -> bool {
        if stable_ticks == 0 {
            return true;
        }
        let mask = (1u8 << stable_ticks) - 1;
        let last_n = self.0.get_history() & mask;
        last_n == 0 || last_n == mask
    }
    /// Check if signal matches double-click pattern
    #[inline(always)]
    #[must_use]
    pub fn is_double_click(&self) -> bool {
        self.0.is_double_click_pattern()
    }
    /// Count the number of active samples in history
    #[inline(always)]
    #[must_use]
    pub fn active_count(&self) -> u8 {
        self.0.count_ones()
    }
    /// Check if signal is steady high (all recent samples are active)
    #[inline(always)]
    #[must_use]
    pub fn is_steady_high(&self, ticks: u8) -> bool {
        if ticks == 0 {
            return false;
        }
        let mask = (1u8 << ticks) - 1;
        (self.0.get_history() & mask) == mask
    }
    /// Check if signal is steady low (all recent samples are inactive)
    #[inline(always)]
    #[must_use]
    pub fn is_steady_low(&self, ticks: u8) -> bool {
        if ticks == 0 {
            return true;
        }
        let mask = (1u8 << ticks) - 1;
        (self.0.get_history() & mask) == 0
    }
    /// Get the underlying SignalByte for direct access
    #[inline(always)]
    #[must_use]
    pub fn signal(&self) -> SignalByte {
        self.0
    }
    /// Get the full 6-tick history
    #[inline(always)]
    #[must_use]
    pub fn history(&self) -> u8 {
        self.0.get_history()
    }
}

/// Output from a sensor evaluation in BGE-style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SensorOutput {
    /// The current state (positive = true, negative = false)
    pub state: bool,
    /// Whether the controller should react this frame
    pub should_trigger: bool,
}

impl SensorOutput {
    /// Creates a new SensorOutput
    #[inline(always)]
    #[must_use]
    pub const fn new(state: bool, should_trigger: bool) -> Self {
        Self {
            state,
            should_trigger,
        }
    }
    /// Returns true if the sensor is active and should trigger
    #[inline(always)]
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.state && self.should_trigger
    }
    /// Returns true if the sensor just triggered (edge or pulse)
    #[inline(always)]
    #[must_use]
    pub const fn triggered(&self) -> bool {
        self.should_trigger
    }
}

// Channel buffer module
pub mod channel_buffer;

pub use channel_buffer::SignalChannelBuffer;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_byte_size() {
        assert_eq!(core::mem::size_of::<SignalByte>(), 1);
    }

    #[test]
    fn test_push_shifts_left() {
        let mut signal = SignalByte::default();
        signal.push(true);
        signal.push(true);
        signal.push(false);
        assert_eq!(signal.get_history(), 0b00000110);
    }

    #[test]
    fn test_rising_edge() {
        let mut signal = SignalByte::default();
        signal.push(false);
        signal.push(true);
        assert!(signal.is_rising_edge());
    }

    #[test]
    fn test_falling_edge() {
        let mut signal = SignalByte::default();
        signal.push(true);
        signal.push(false);
        assert!(signal.is_falling_edge());
    }

    #[test]
    fn test_signal_state_sample() {
        let mut state = SignalState::default();
        state.sample(true);
        assert!(state.is_positive());
        assert!(!state.was_positive());
    }

    #[test]
    fn test_signal_state_rising_edge() {
        let mut state = SignalState::default();
        state.sample(false);
        state.sample(true);
        assert!(state.is_rising_edge());
    }

    #[test]
    fn test_rising_edge_triggers() {
        let mut signal = SignalByte::default();
        signal.push(false);
        signal.push(true);
        let output = signal.evaluate_bge_output(false, false);
        assert!(output.should_trigger);
        assert!(output.state);
    }

    #[test]
    fn test_true_pulse_continuous() {
        let mut signal = SignalByte::default();
        for _ in 0..6 {
            signal.push(true);
        }
        let output = signal.evaluate_bge_output(true, false);
        assert!(output.should_trigger);
        assert!(output.state);
    }
}
