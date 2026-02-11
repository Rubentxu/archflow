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
use archflow_logic::pulse::SensorState;
use archflow_logic::{EventData, LogicEvent, LogicSystem, Pulse};
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

    /// Check if there are pending events in the buffer
    ///
    /// # Returns
    /// true if there are events, false otherwise
    #[wasm_bindgen]
    pub fn has_events(&mut self) -> bool {
        self.inner.has_events()
    }

    /// Drain all pending events from the event buffer
    ///
    /// # Returns
    /// Array of event data objects (simplified for WASM)
    #[wasm_bindgen]
    pub fn drain_events(&mut self) -> Vec<JsLogicEventData> {
        let events = self.inner.event_buffer().drain();
        events
            .into_iter()
            .map(|event| {
                // Extract context data based on EventData variant
                let (data_1, data_2, data_3) = match event.data {
                    EventData::None => (0.0, 0.0, 0),
                    EventData::Proximity { distance } => (distance, 0.0, 0),
                    EventData::Drag { start_pos, .. } => (start_pos.0, start_pos.1, 0),
                    EventData::BoxSelection { count } => (0.0, 0.0, count),
                    EventData::Hover { entity_id } => (0.0, 0.0, entity_id.unwrap_or(0)),
                };

                JsLogicEventData {
                    event_type: event.event_type as u8,
                    entity_id: event.entity_id,
                    timestamp_us: event.timestamp_us,
                    data_1,
                    data_2,
                    data_3,
                }
            })
            .collect()
    }

    /// Get the number of pending events
    ///
    /// # Returns
    /// Number of events in the buffer
    #[wasm_bindgen]
    pub fn event_count(&mut self) -> usize {
        self.inner.event_buffer().len()
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // EPIC 8.x: BEHAVIOR BRIDGE - Simplified methods for JS integration
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Create a simple behavior
    ///
    /// # Arguments
    /// * `entity_id` - Entity ID for the behavior
    /// * `sensor_type` - Sensor type (0=Click, 1=Hover, 2=Drag, 3=Key)
    /// * `actuator_type` - Actuator type (0=Highlight, 1=Select, 2=Move, 3=Delete, 4=Emit)
    ///
    /// # Returns
    /// Behavior ID
    #[wasm_bindgen]
    pub fn create_behavior(
        &mut self,
        _entity_id: u32,
        _sensor_type: u8,
        _actuator_type: u8,
    ) -> usize {
        0 // Simplified: returns 0
    }

    /// Attach a behavior to an entity
    #[wasm_bindgen]
    pub fn attach_behavior(&mut self, _behavior_id: usize, _entity_id: u32) {
        // Simplified: no-op
    }

    /// Detach a behavior
    #[wasm_bindgen]
    pub fn detach_behavior(&mut self, _behavior_id: usize) {
        // Simplified: no-op
    }

    /// Get count of behaviors
    #[wasm_bindgen]
    pub fn behavior_count(&self) -> usize {
        0
    }

    /// Set behavior enabled/disabled
    #[wasm_bindgen]
    pub fn set_behavior_enabled(&mut self, _behavior_id: usize, _enabled: bool) {
        // Simplified: no-op
    }

    /// Check if behavior has events
    #[wasm_bindgen]
    pub fn behavior_has_events(&self, _behavior_id: usize) -> bool {
        false
    }

    /// Get behavior state as JSON
    #[wasm_bindgen]
    pub fn get_behavior_state(&self, _behavior_id: usize) -> js_sys::JsString {
        JsValue::from_str("{}").into()
    }
}

impl LogicSystemWasm {
    /// Get the inner LogicSystem for internal use
    ///
    /// This is used by the engine to perform sensor evaluation.
    pub(crate) fn inner(&mut self) -> &mut LogicSystem {
        &mut self.inner
    }

    /// Get the inner LogicSystem for internal use (mutable)
    ///
    /// This is used by the engine to perform sensor evaluation.
    pub(crate) fn inner_mut(&mut self) -> &mut LogicSystem {
        &mut self.inner
    }
}

impl Default for LogicSystemWasm {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// LOGIC EVENT DATA (WASM-friendly)
// ═══════════════════════════════════════════════════════════════════════════════

/// Simplified event data for WASM export
#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub struct JsLogicEventData {
    /// Event type identifier
    pub event_type: u8,

    /// Entity ID that triggered the event
    pub entity_id: u32,

    /// Timestamp in microseconds
    pub timestamp_us: u64,

    /// Additional data depending on event type:
    /// - ProximityAlert: f32 distance
    /// - DragStarted/DragEnded: f32 x, f32 y position
    /// - BoxSelectionCompleted: u32 count
    /// - HoverChanged: u32 entity_id (or 0 for none)
    pub data_1: f32,
    pub data_2: f32,
    pub data_3: u32,
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
///   state: 1, // Positive
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

    /// The pulse state (0=None, 1=Positive, 2=Negative)
    state: u8,

    /// Timestamp when the pulse occurred
    timestamp: u32,
}

#[wasm_bindgen]
impl PulseWasm {
    /// Create a new Pulse
    #[wasm_bindgen(constructor)]
    pub fn new(entity_id: u32, sensor_id: u32, state: u8, timestamp: u32) -> Self {
        Self {
            entity_id,
            sensor_id,
            state,
            timestamp,
        }
    }

    /// Get the entity ID
    #[wasm_bindgen(getter)]
    pub fn entity_id(&self) -> u32 {
        self.entity_id
    }

    /// Get the sensor ID
    #[wasm_bindgen(getter)]
    pub fn sensor_id(&self) -> u32 {
        self.sensor_id
    }

    /// Get the state (0=None, 1=Positive, 2=Negative)
    #[wasm_bindgen(getter)]
    pub fn state(&self) -> u8 {
        self.state
    }

    /// Get the timestamp
    #[wasm_bindgen(getter)]
    pub fn timestamp(&self) -> u32 {
        self.timestamp
    }
}

impl From<Pulse> for PulseWasm {
    fn from(pulse: Pulse) -> Self {
        Self {
            entity_id: pulse.entity_id,
            sensor_id: pulse.sensor_id,
            state: pulse.state as u8,
            timestamp: pulse.timestamp,
        }
    }
}

impl From<PulseWasm> for Pulse {
    fn from(pulse: PulseWasm) -> Self {
        Self {
            entity_id: pulse.entity_id,
            sensor_id: pulse.sensor_id,
            state: match pulse.state {
                0 => SensorState::None,
                1 => SensorState::Positive,
                2 => SensorState::Negative,
                _ => SensorState::None,
            },
            timestamp: pulse.timestamp,
        }
    }
}
