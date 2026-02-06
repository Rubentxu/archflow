// ═══════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Signal Processing Module
//
// This module provides signal types for the Logic Bricks system:
// - SignalByte: 6-tick history in 1 byte (bit-packed)
// - SignalState: Wrapper with BGE-style analysis methods
//
// Reference: UPBGE source/gameengine/GameLogic/SCA_ISensor.cpp
// ═══════════════════════════════════════════════════════════════════════════════════════

use core::fmt;

/// Signal byte with 6-tick history for edge detection and debouncing
///
/// # Layout
///
/// ```text
/// Bit:  7   6   5   4   3   2   1   0
///     [ R | R | T5| T4| T3| T2| T1| T0]
/// ```
///
/// - `T0`: Current tick (most recent)
/// - `T1-T5`: Previous ticks (history)
/// - `R`: Reserved bits for future use
///
/// # Examples
///
/// ```
/// use archflow_logic::SignalByte;
///
/// let mut signal = SignalByte::default();
/// signal.push(true);   // 0b00000001
/// signal.push(true);   // 0b00000011
/// signal.push(false);  // 0b00000110
///
/// assert_eq!(signal.get_history(), 0b00000110);
/// assert!(!signal.get_current()); // T0 is 0
/// ```
///
/// # Performance
///
/// - Size: Exactly 1 byte (`#[repr(transparent)]`)
/// - Copy: Zero-copy semantic (implements `Copy`)
/// - Inline: All methods are `#[inline(always)]` for zero-cost abstraction
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

    /// Alias for `is_steady_high` for compatibility
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

// ═══════════════════════════════════════════════════════════════════════════════════════
// SignalState - BGE-style wrapper around SignalByte
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Signal State - Wrapper around SignalByte with BGE-style analysis methods
///
/// This provides the state analysis that BGE's SCA_ISensor performs on its
/// internal signal history. Wraps SignalByte (6-tick history) with methods
/// for edge detection and state queries.
///
/// # BGE Architecture Reference
///
/// In BGE, each sensor has an internal signal (6-bit history):
/// ```cpp
/// // SCA_ISensor.cpp
/// void SCA_ISensor::Activate(SCA_LogicManager* manager) {
///   bool trigger = Evaluate();
///   bool old_state = m_state;
///   m_state = trigger != m_invert;
///   // Edge detection and pulse generation happens here
/// }
/// ```
///
/// # Example
///
/// ```rust
/// use archflow_logic::signals::SignalState;
///
/// let mut state = SignalState::default();
/// state.sample(true);   // T0: 1
/// state.sample(false);  // T1: 0
/// state.sample(false);  // T2: 0
/// state.sample(true);   // T3: 1
/// state.sample(true);   // T4: 1
/// state.sample(true);   // T5: 1
///
/// assert!(state.is_rising_edge());   // false → true transition detected
/// assert!(state.is_positive());       // Current state is true
/// ```
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

    /// Check for rising edge (false → true transition)
    #[inline(always)]
    #[must_use]
    pub fn is_rising_edge(&self) -> bool {
        self.0.is_rising_edge()
    }

    /// Check for falling edge (true → false transition)
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
    fn test_get_prev() {
        let mut signal = SignalByte::default();
        signal.push(true);
        assert!(!signal.get_prev()); // Default was 0
        signal.push(false);
        assert!(signal.get_prev()); // Previous was true
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
    fn test_signal_state_double_click() {
        let mut state = SignalState::default();
        state.sample(true); // Click
        state.sample(false); // Release
        state.sample(false); // Pause
        state.sample(true); // Click
        state.sample(false); // Release
        state.sample(false); // Release
        assert!(state.is_double_click());
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// SensorOutput - BGE-style Pulse Output (HU-002)
// ═══════════════════════════════════════════════════════════════════════════════════════

/// Output from a sensor evaluation in BGE-style.
///
/// Separates the current state from whether the controller should react.
/// This is crucial for implementing BGE's True/False pulse modes.
///
/// # BGE Pulse Modes
///
/// In BGE, sensors have `[tap]` and `[inv]` buttons:
/// - **Tap** (True Pulse): Emit pulse while signal is positive
/// - **False Pulse**: Emit pulse when signal goes negative
/// - **Invert**: Invert the output signal
///
/// # Example
///
/// ```
/// use archflow_logic::signals::{SensorOutput, SignalByte};
///
/// let mut signal = SignalByte::default();
/// signal.push(false);  // T1 = 0
/// signal.push(true);   // T0 = 1 (rising edge)
///
/// // With true_pulse enabled, should trigger on rising edge
/// let output = signal.evaluate_bge_output(true, false);
/// assert!(output.should_trigger);
/// assert!(output.state);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SensorOutput {
    /// The current state (positive = true, negative = false)
    pub state: bool,

    /// Whether the controller should react this frame
    /// True when: rising edge, falling edge, or pulse mode active
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

impl SignalByte {
    /// Evaluates BGE-style output with pulse mode support
    ///
    /// This implements the SCA_ISensor pulse generation logic from BGE:
    /// - Emits trigger on rising edge (0 -> 1)
    /// - Emits trigger on falling edge (1 -> 0) if false_pulse enabled
    /// - Emits trigger continuously if true_pulse enabled and state is positive
    ///
    /// # Arguments
    ///
    /// * `true_pulse` - Emit pulse while signal is positive (BGE tap mode)
    /// * `false_pulse` - Emit pulse on falling edge (BGE false pulse)
    ///
    /// # Returns
    ///
    /// `SensorOutput` with current state and trigger decision
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

#[cfg(test)]
mod sensor_output_tests {
    use super::*;

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
    fn test_falling_edge_triggers() {
        let mut signal = SignalByte::default();
        signal.push(true);
        signal.push(false);

        let output = signal.evaluate_bge_output(false, false);
        assert!(output.should_trigger);
        assert!(!output.state);
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

    #[test]
    fn test_false_pulse_on_falling() {
        let mut signal = SignalByte::default();
        signal.push(true);
        signal.push(false);

        let output = signal.evaluate_bge_output(false, true);
        assert!(output.should_trigger);
        assert!(!output.state);
    }

    #[test]
    fn test_no_pulse_no_trigger_without_edge() {
        let mut signal = SignalByte::default();
        for _ in 0..6 {
            signal.push(true);
        }

        let output = signal.evaluate_bge_output(false, false);
        assert!(!output.should_trigger);
        assert!(output.state);
    }

    #[test]
    fn test_combined_pulse_modes() {
        let mut signal = SignalByte::default();
        signal.push(true);
        signal.push(false);

        let output = signal.evaluate_bge_output(true, true);
        assert!(output.should_trigger);
    }

    #[test]
    fn test_with_true_pulse() {
        let mut signal = SignalByte::default();
        for _ in 0..6 {
            signal.push(true);
        }

        let output = signal.with_true_pulse();
        assert!(output.should_trigger);
        assert!(output.state);
    }

    #[test]
    fn test_sensor_output_construction() {
        let output = SensorOutput::new(true, true);
        assert!(output.is_active());
        assert!(output.triggered());
    }

    #[test]
    fn test_sensor_output_inactive() {
        let output = SensorOutput::new(false, false);
        assert!(!output.is_active());
        assert!(!output.triggered());
    }
}
