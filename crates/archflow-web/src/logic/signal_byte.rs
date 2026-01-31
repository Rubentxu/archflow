// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Web - SignalByte WASM Binding
//
// Epic 5.1: Expose SignalByte to JavaScript/TypeScript
//
// Provides a JavaScript-accessible wrapper around archflow_logic::signals::SignalByte.
// This allows web developers to use the binary signal processing capabilities
// of the Logic Bricks system directly in their applications.
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]

use archflow_logic::signals::SignalByte as CoreSignalByte;
use wasm_bindgen::prelude::*;

/// SignalByte WASM wrapper
///
/// A binary signal with 6-tick history for edge detection and pattern matching.
/// This is the JavaScript-accessible version of the core SignalByte type.
///
/// # JavaScript Example
/// ```javascript
/// const signal = new SignalByte();
/// signal.push(true);
/// signal.push(true);
/// signal.push(false);
/// console.log(signal.getHistory()); // 6 (0b00000110)
/// console.log(signal.isStableHigh(3)); // false
/// ```
#[wasm_bindgen]
pub struct SignalByteWasm {
    inner: CoreSignalByte,
}

#[wasm_bindgen]
impl SignalByteWasm {
    /// Creates a new SignalByte with all bits set to 0
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: CoreSignalByte::default(),
        }
    }

    /// Creates a SignalByte from a u8 value
    #[wasm_bindgen]
    pub fn from(value: u8) -> Self {
        Self {
            inner: CoreSignalByte::from(value),
        }
    }

    /// Pushes a new signal state, shifting the history left
    ///
    /// # Arguments
    /// * `active` - true if the signal is active, false otherwise
    ///
    /// # JavaScript Example
    /// ```javascript
    /// const signal = new SignalByte();
    /// signal.push(true);  // 0b00000001
    /// signal.push(true);  // 0b00000011
    /// signal.push(false); // 0b00000110
    /// ```
    #[wasm_bindgen]
    pub fn push(&mut self, active: bool) {
        self.inner.push(active);
    }

    /// Returns the current signal state (tick T0, least significant bit)
    ///
    /// # JavaScript Example
    /// ```javascript
    /// const signal = new SignalByte();
    /// signal.push(true);
    /// console.log(signal.getCurrent()); // true
    /// ```
    #[wasm_bindgen]
    pub fn get_current(&self) -> bool {
        self.inner.get_current()
    }

    /// Returns the 6-bit history of the signal
    ///
    /// # JavaScript Example
    /// ```javascript
    /// const signal = new SignalByte();
    /// signal.push(true);
    /// signal.push(true);
    /// signal.push(false);
    /// console.log(signal.getHistory()); // 6 (0b00000110)
    /// ```
    #[wasm_bindgen]
    pub fn get_history(&self) -> u8 {
        self.inner.get_history()
    }

    /// Returns the raw u8 value (for serialization)
    #[wasm_bindgen]
    pub fn as_u8(&self) -> u8 {
        self.inner.as_u8()
    }

    /// Detects rising edge: 0 in T-1, 1 in T
    ///
    /// Pattern: [xxxx01]
    ///
    /// # JavaScript Example
    /// ```javascript
    /// const signal = new SignalByte();
    /// signal.push(false);
    /// signal.push(true);
    /// console.log(signal.isRisingEdge()); // true
    /// ```
    #[wasm_bindgen]
    pub fn is_rising_edge(&self) -> bool {
        self.inner.is_rising_edge()
    }

    /// Detects falling edge: 1 in T-1, 0 in T
    ///
    /// Pattern: [xxxx10]
    ///
    /// # JavaScript Example
    /// ```javascript
    /// const signal = new SignalByte();
    /// signal.push(true);
    /// signal.push(false);
    /// console.log(signal.isFallingEdge()); // true
    /// ```
    #[wasm_bindgen]
    pub fn is_falling_edge(&self) -> bool {
        self.inner.is_falling_edge()
    }

    /// Returns true if there is any edge (rising or falling)
    #[wasm_bindgen]
    pub fn any_edge(&self) -> bool {
        self.inner.any_edge()
    }

    /// Checks if the signal has been steady (all 1s) for the last N ticks
    ///
    /// # Arguments
    /// * `ticks` - Number of ticks to check (1-6)
    ///
    /// # JavaScript Example
    /// ```javascript
    /// const signal = SignalByte.from(0b00111111);
    /// console.log(signal.isSteadyHigh(6)); // true
    /// console.log(signal.isSteadyHigh(3)); // true
    /// ```
    #[wasm_bindgen]
    pub fn is_steady_high(&self, ticks: u8) -> bool {
        self.inner.is_steady_high(ticks)
    }

    /// Checks if the signal has been steady (all 0s) for the last N ticks
    ///
    /// # JavaScript Example
    /// ```javascript
    /// const signal = new SignalByte();
    /// console.log(signal.isSteadyLow(3)); // true
    /// ```
    #[wasm_bindgen]
    pub fn is_steady_low(&self, ticks: u8) -> bool {
        self.inner.is_steady_low(ticks)
    }

    /// Alias for isSteadyHigh (for backward compatibility)
    #[wasm_bindgen]
    pub fn is_steady(&self, ticks: u8) -> bool {
        self.inner.is_steady(ticks)
    }

    /// Counts how many ticks are 1 in the history
    ///
    /// # JavaScript Example
    /// ```javascript
    /// const signal = SignalByte.from(0b00110111);
    /// console.log(signal.countOnes()); // 5
    /// ```
    #[wasm_bindgen]
    pub fn count_ones(&self) -> u8 {
        self.inner.count_ones()
    }

    /// Counts how many ticks are 0 in the history
    ///
    /// # JavaScript Example
    /// ```javascript
    /// const signal = SignalByte.from(0b00110111);
    /// console.log(signal.countZeros()); // 1
    /// ```
    #[wasm_bindgen]
    pub fn count_zeros(&self) -> u8 {
        self.inner.count_zeros()
    }

    /// Returns the size in bytes (always 1)
    ///
    /// # JavaScript Example
    /// ```javascript
    /// const signal = new SignalByte();
    /// console.log(signal.size()); // 1
    /// ```
    #[wasm_bindgen]
    pub fn size(&self) -> usize {
        core::mem::size_of::<CoreSignalByte>()
    }
}

impl Default for SignalByteWasm {
    fn default() -> Self {
        Self::new()
    }
}

impl From<CoreSignalByte> for SignalByteWasm {
    fn from(inner: CoreSignalByte) -> Self {
        Self { inner }
    }
}

impl From<SignalByteWasm> for CoreSignalByte {
    fn from(wasm: SignalByteWasm) -> Self {
        wasm.inner
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WASM TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_byte_size() {
        let signal = SignalByteWasm::new();
        assert_eq!(signal.size(), 1);
    }

    #[test]
    fn test_push_shifts_left() {
        let mut signal = SignalByteWasm::new();

        signal.push(true);
        signal.push(true);
        signal.push(false);

        assert_eq!(signal.get_history(), 0b00000110);
    }

    #[test]
    fn test_get_current_returns_lsb() {
        let mut signal = SignalByteWasm::new();

        signal.push(true);
        assert!(signal.get_current());

        signal.push(false);
        assert!(!signal.get_current());
    }

    #[test]
    fn test_rising_edge_detection() {
        let mut signal = SignalByteWasm::new();

        signal.push(false);
        signal.push(true);

        assert!(signal.is_rising_edge());
        assert!(!signal.is_falling_edge());
    }

    #[test]
    fn test_falling_edge_detection() {
        let mut signal = SignalByteWasm::new();

        signal.push(true);
        signal.push(false);

        assert!(signal.is_falling_edge());
        assert!(!signal.is_rising_edge());
    }

    #[test]
    fn test_steady_high() {
        let signal = SignalByteWasm::from(0b00111111);
        assert!(signal.is_steady_high(6));
        assert!(signal.is_steady_high(3));
    }

    #[test]
    fn test_count_ones() {
        let signal = SignalByteWasm::from(0b00110111);
        assert_eq!(signal.count_ones(), 5);
    }

    #[test]
    fn test_count_zeros() {
        let signal = SignalByteWasm::from(0b00110111);
        assert_eq!(signal.count_zeros(), 1);
    }

    #[test]
    fn test_from_u8() {
        let signal = SignalByteWasm::from(0b00111111);
        assert_eq!(signal.get_history(), 0b00111111);
    }
}
