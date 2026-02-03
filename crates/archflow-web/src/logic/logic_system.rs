// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Web - LogicSystem WASM Binding
//
// Epic 5.6: Expose LogicSystem to JavaScript/TypeScript
//
// Provides a JavaScript-accessible wrapper for the LogicSystem
// that orchestrates sensor evaluation and actuator execution.
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]

use alloc::vec::Vec;
use archflow_engine::EntityStore;
use archflow_logic::{LogicSystem, Pulse};
use wasm_bindgen::prelude::*;

/// WASM wrapper for LogicSystem
///
/// This provides JavaScript access to the main Logic Bricks orchestration system.
///
/// # JavaScript Example
/// ```javascript
/// import { LogicSystem } from '@archflow/sdk';
///
/// const system = new LogicSystem();
/// system.update(timestamp);
/// ```
#[wasm_bindgen]
pub struct LogicSystemWasm {
    inner: LogicSystem,
}

#[wasm_bindgen]
impl LogicSystemWasm {
    /// Creates a new LogicSystem
    ///
    /// # JavaScript Example
    /// ```javascript
    /// const system = new LogicSystem();
    /// ```
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: LogicSystem::new(),
        }
    }

    /// Updates the logic system timestamp
    ///
    /// This should be called each frame before sensor evaluation.
    ///
    /// # Arguments
    /// * `timestamp_ms` - Current timestamp in milliseconds
    ///
    /// # JavaScript Example
    /// ```javascript
    /// system.update(performance.now());
    /// ```
    #[wasm_bindgen]
    pub fn update(&mut self, timestamp_ms: u64) {
        // Convert u64 to u32 for the LogicSystem (milliseconds to seconds-ish scale)
        self.inner.set_timestamp(timestamp_ms as u32);
    }

    /// Get the inner LogicSystem for internal use
    ///
    /// This is used by the engine to perform sensor evaluation.
    pub(crate) fn inner(&mut self) -> &mut LogicSystem {
        &mut self.inner
    }
}

impl Default for LogicSystemWasm {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PULSE WASM WRAPPER
// ═══════════════════════════════════════════════════════════════════════════════

/// WASM wrapper for Pulse events
///
/// Represents a sensor state change event flowing through the Logic Bricks system.
///
/// # JavaScript Example
/// ```javascript
/// const pulse = {
///   entityId: 123,
///   sensorId: 5,
///   isActive: true,
///   timestamp: 1000
/// };
/// ```
#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct PulseWasm {
    /// Entity ID that generated the pulse
    entity_id: u32,

    /// Sensor ID that generated the pulse
    sensor_id: u32,

    /// The pulse state (true = positive/active, false = negative/inactive)
    state: bool,

    /// Timestamp when the pulse was generated
    timestamp: u32,
}

#[wasm_bindgen]
impl PulseWasm {
    /// Creates a new PulseWasm
    ///
    /// # JavaScript Example
    /// ```javascript
    /// const pulse = new PulseWasm(123, 5, true, 1000);
    /// ```
    #[wasm_bindgen(constructor)]
    pub fn new(entity_id: u32, sensor_id: u32, state: bool, timestamp: u32) -> Self {
        Self {
            entity_id,
            sensor_id,
            state,
            timestamp,
        }
    }

    /// Get the entity ID
    #[wasm_bindgen]
    pub fn entity_id(&self) -> u32 {
        self.entity_id
    }

    /// Get the sensor ID
    #[wasm_bindgen]
    pub fn sensor_id(&self) -> u32 {
        self.sensor_id
    }

    /// Check if the pulse is active (positive edge)
    ///
    /// Returns true for positive pulses (sensor became TRUE)
    /// Returns false for negative pulses (sensor became FALSE)
    #[wasm_bindgen]
    pub fn is_active(&self) -> bool {
        self.state
    }

    /// Get the timestamp
    #[wasm_bindgen]
    pub fn timestamp(&self) -> u32 {
        self.timestamp
    }
}

impl From<Pulse> for PulseWasm {
    fn from(pulse: Pulse) -> Self {
        Self {
            entity_id: pulse.entity_id,
            sensor_id: pulse.sensor_id,
            state: pulse.state.is_positive(),
            timestamp: pulse.timestamp,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WASM TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logic_system_creation() {
        let _system = LogicSystemWasm::new();
    }

    #[test]
    fn test_logic_system_default() {
        let _system = LogicSystemWasm::default();
    }

    #[test]
    fn test_pulse_wasm_constructor() {
        let pulse = PulseWasm::new(123, 5, true, 1000);
        assert_eq!(pulse.entity_id(), 123);
        assert_eq!(pulse.sensor_id(), 5);
        assert!(pulse.is_active());
        assert_eq!(pulse.timestamp(), 1000);
    }

    #[test]
    fn test_pulse_wasm_from_positive() {
        let pulse = Pulse::positive(5, 123, 1000);
        let wasm = PulseWasm::from(pulse);
        assert_eq!(wasm.entity_id(), 123);
        assert_eq!(wasm.sensor_id(), 5);
        assert!(wasm.is_active());
        assert_eq!(wasm.timestamp(), 1000);
    }

    #[test]
    fn test_pulse_wasm_from_negative() {
        let pulse = Pulse::negative(3, 456, 2000);
        let wasm = PulseWasm::from(pulse);
        assert_eq!(wasm.entity_id(), 456);
        assert_eq!(wasm.sensor_id(), 3);
        assert!(!wasm.is_active());
        assert_eq!(wasm.timestamp(), 2000);
    }

    #[test]
    fn test_pulse_wasm_copy() {
        let pulse = PulseWasm {
            entity_id: 1,
            sensor_id: 2,
            state: true,
            timestamp: 100,
        };

        let copy = pulse;
        assert_eq!(copy.entity_id(), 1);
        assert_eq!(pulse.entity_id(), 1);
    }
}
