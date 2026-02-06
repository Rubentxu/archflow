// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Logic Driver (HU-002: Adaptive Signal Sampling)
//
// This module implements the LogicDriver for adaptive signal sampling,
// extending the 6-bit SignalByte history to cover human-scale time windows.
//
// Problem: At 60 FPS, 6 bits only cover 100ms (6 frames × 16.67ms).
// Human gestures like double-click require 200-500ms windows.
//
// Solution: LogicDriver with configurable frequency and sticky accumulator.
// With F=4, 6 bits cover 400ms - perfect for human gestures.
//
// Reference: docs/epics/ultimas_epics/sugerencias.md
// ═══════════════════════════════════════════════════════════════════════════════

use crate::signals::SignalByte;

/// Logic Driver for adaptive signal sampling
///
/// Manages when signals should be sampled based on configurable frequency.
/// This extends the effective time window of SignalByte without losing information.
///
/// # Timing Windows
///
/// | Frequency | Window (6 bits) | Use Case |
/// |-----------|-----------------|----------|
/// | 1 (60Hz) | 100ms | Physics, fast collisions |
/// | 4 (15Hz) | **400ms** | **Double-click, human gestures** |
/// | 10 (6Hz) | 1.0s | AI states, cooldowns |
///
/// # Example
///
/// ```
/// use archflow_logic::logic_driver::LogicDriver;
///
/// let mut driver = LogicDriver::new(4); // Sample every 4 frames
/// let mut signal = SignalByte::default();
///
/// // Simulate rapid clicks that span multiple sampling periods
/// for _ in 0..24 {
///     driver.update(&mut signal, true);  // Button pressed
///     driver.update(&mut signal, false); // Button released
/// }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct LogicDriver {
    /// Current frame counter (0 to frequency - 1)
    frame_counter: u8,

    /// Sampling frequency: sample every N frames
    /// - F=1: Sample every frame (100ms window)
    /// - F=4: Sample every 4 frames (400ms window)
    /// - F=10: Sample every 10 frames (1s window)
    frequency: u8,

    /// Sticky accumulator captures input state between samples
    /// Prevents losing fast clicks that occur between sample points
    sticky_accumulator: bool,
}

impl LogicDriver {
    /// Creates a new LogicDriver with the specified frequency
    ///
    /// # Arguments
    ///
    /// * `frequency` - Sample every N frames (1-10 recommended)
    ///
    /// # Panics
    ///
    /// Panics if frequency is 0.
    #[inline(always)]
    #[must_use]
    pub fn new(frequency: u8) -> Self {
        assert!(frequency > 0, "Frequency must be >= 1");
        Self {
            frame_counter: 0,
            frequency,
            sticky_accumulator: false,
        }
    }

    /// Creates a LogicDriver optimized for human gestures
    ///
    /// Uses F=4, giving a 400ms window - ideal for double-clicks.
    #[inline(always)]
    #[must_use]
    pub fn human_gestures() -> Self {
        Self::new(4)
    }

    /// Creates a LogicDriver optimized for fast physics
    ///
    /// Uses F=1, giving a 100ms window for precise timing.
    #[inline(always)]
    #[must_use]
    pub fn physics() -> Self {
        Self::new(1)
    }

    /// Updates the signal with the current raw input
    ///
    /// The raw input is accumulated in `sticky_accumulator` until the
    /// sampling period elapses, then pushed to the signal.
    ///
    /// # Arguments
    ///
    /// * `signal` - The SignalByte to update
    /// * `raw_input` - The current raw input state (e.g., button pressed)
    ///
    /// # Note
    ///
    /// If the raw input is true at ANY point between samples,
    /// the accumulated value will be true when sampled.
    #[inline(always)]
    pub fn update(&mut self, signal: &mut SignalByte, raw_input: bool) {
        if raw_input {
            self.sticky_accumulator = true;
        }

        self.frame_counter += 1;

        if self.frame_counter >= self.frequency {
            // Time to sample - push accumulated state
            signal.push(self.sticky_accumulator);

            // Reset for next cycle
            self.frame_counter = 0;
            self.sticky_accumulator = false;
        }
    }

    /// Updates multiple signals with the same raw input
    ///
    /// More efficient than calling `update()` individually when
    /// the same input affects multiple signals.
    #[inline(always)]
    pub fn update_multiple(&mut self, signals: &mut [SignalByte], raw_input: bool) {
        if raw_input {
            self.sticky_accumulator = true;
        }

        self.frame_counter += 1;

        if self.frame_counter >= self.frequency {
            // Sample all signals
            for signal in signals {
                signal.push(self.sticky_accumulator);
            }

            // Reset for next cycle
            self.frame_counter = 0;
            self.sticky_accumulator = false;
        }
    }

    /// Forces an immediate sample (bypasses frequency timer)
    ///
    /// Useful for triggering immediate responses or synchronization.
    #[inline(always)]
    pub fn force_sample(&mut self, signal: &mut SignalByte, raw_input: bool) {
        signal.push(raw_input || self.sticky_accumulator);
        self.frame_counter = 0;
        self.sticky_accumulator = false;
    }

    /// Returns true if a sample will be taken this frame
    #[inline(always)]
    #[must_use]
    pub const fn is_sample_frame(&self) -> bool {
        self.frame_counter == 0
    }

    /// Returns the current frame counter value
    #[inline(always)]
    #[must_use]
    pub const fn frame_counter(&self) -> u8 {
        self.frame_counter
    }

    /// Returns the current accumulated value
    #[inline(always)]
    #[must_use]
    pub const fn accumulated(&self) -> bool {
        self.sticky_accumulator
    }

    /// Resets the driver to initial state
    #[inline(always)]
    pub fn reset(&mut self) {
        self.frame_counter = 0;
        self.sticky_accumulator = false;
    }
}

impl Default for LogicDriver {
    /// Creates a LogicDriver with default frequency (4 frames = 400ms window)
    fn default() -> Self {
        Self::human_gestures()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let driver = LogicDriver::new(4);
        assert_eq!(driver.frame_counter, 0);
        assert_eq!(driver.frequency, 4);
        assert!(!driver.accumulated());
    }

    #[test]
    #[should_panic]
    fn test_new_zero_frequency_panics() {
        LogicDriver::new(0);
    }

    #[test]
    fn test_human_gestures_default() {
        let driver = LogicDriver::human_gestures();
        assert_eq!(driver.frequency, 4);
    }

    #[test]
    fn test_physics_default() {
        let driver = LogicDriver::physics();
        assert_eq!(driver.frequency, 1);
    }

    #[test]
    fn test_sticky_accumulator_captures_fast_click() {
        let mut driver = LogicDriver::new(4); // Sample every 4 frames
        let mut signal = SignalByte::default();

        // Frame 0-2: Click happens between samples (accumulated = true)
        driver.update(&mut signal, false);
        driver.update(&mut signal, true); // Click! accumulated = true
        driver.update(&mut signal, false);

        // Frame 3: Update triggers sample (accumulated was true)
        driver.update(&mut signal, false);

        // Signal should have captured the click (sticky = true)
        assert!(signal.get_current());
    }

    #[test]
    fn test_sample_every_n_frames() {
        let mut driver = LogicDriver::new(3);
        let mut signal = SignalByte::default();

        // Frame 0: After init, next update (frame 1) will be sample
        // So frame_counter=0 means "ready to sample on next update"
        driver.update(&mut signal, true);
        assert!(!signal.get_current()); // First update (frame 1) not sampled yet

        driver.update(&mut signal, true);
        assert!(!signal.get_current()); // Second update (frame 2) not sampled yet

        // Third update triggers sample
        driver.update(&mut signal, true);
        assert!(signal.get_current()); // Sampled!
    }

    #[test]
    fn test_force_sample() {
        let mut driver = LogicDriver::new(10); // Very slow
        let mut signal = SignalByte::default();

        // Force immediate sample
        driver.force_sample(&mut signal, true);
        assert!(signal.get_current());

        // Driver should be reset
        assert_eq!(driver.frame_counter(), 0);
        assert!(!driver.accumulated());
    }

    #[test]
    fn test_update_multiple() {
        let mut driver = LogicDriver::new(2);
        let mut signals = [
            SignalByte::default(),
            SignalByte::default(),
            SignalByte::default(),
        ];

        // Update all at once
        driver.update_multiple(&mut signals, true);

        // First update doesn't sample
        assert!(!signals[0].get_current());

        // Second update samples
        driver.update_multiple(&mut signals, false);
        assert!(signals[0].get_current());
        assert!(signals[1].get_current());
        assert!(signals[2].get_current());
    }

    #[test]
    fn test_reset() {
        let mut driver = LogicDriver::new(4);
        let mut signal = SignalByte::default();

        // Advance to middle of cycle
        for _ in 0..3 {
            driver.update(&mut signal, true);
        }

        driver.reset();
        assert_eq!(driver.frame_counter(), 0);
        assert!(!driver.accumulated());
    }

    #[test]
    fn test_double_click_pattern() {
        // Simulate a double-click with F=1 (physics mode) to get 6 samples
        // Using F=1 means every update is a sample
        let mut driver = LogicDriver::new(1);
        let mut signal = SignalByte::default();

        // Pattern: true, false, false, true, false, false
        // First click: 1,0,0 (click down, release, pause)
        driver.update(&mut signal, true); // T0
        driver.update(&mut signal, false); // T1
        driver.update(&mut signal, false); // T2

        // Second click: 1,0,0 (click down, release, pause)
        driver.update(&mut signal, true); // T3
        driver.update(&mut signal, false); // T4
        driver.update(&mut signal, false); // T5

        // Check if pattern matches double-click (100100 = 0b00100100)
        assert!(signal.is_double_click_pattern());
    }

    #[test]
    fn test_consecutive_samples() {
        let mut driver = LogicDriver::new(1); // Sample every frame
        let mut signal = SignalByte::default();

        // Rapid updates - should track exactly
        for i in 0..10 {
            driver.update(&mut signal, i % 2 == 0);
        }

        // With F=1, should have full 6-tick history
        assert_ne!(signal.get_history(), 0);
    }
}
