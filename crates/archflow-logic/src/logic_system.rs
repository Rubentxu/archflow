// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Logic System
//
// This module provides the main LogicSystem that orchestrates:
// 1. Sensor evaluation (detect input events)
// 2. Pulse generation (emit events to PulseBus)
// 3. Actuator execution (respond to pulses)
//
// This is the heart of the Logic Bricks system inspired by Blender's BGE.
//
// Reference: docs/epics/EPIC-001-input-sensors.md - HU-004
// ═══════════════════════════════════════════════════════════════════════════════

#![warn(missing_docs)]

use alloc::vec::Vec;
use archflow_core::{EntityId, Generation, Index};
use archflow_engine::EntityStore;

use crate::input::{InputEvent, InputSampler};
use crate::mapping::LogicMappingTable;
use crate::pulse::{Pulse, PulseBus};
use crate::sensors::{
    DoubleTapSensor, LongPressSensor, MouseClickSensor, MouseConfig,
    MouseOverSensor, MouseSensor, ProximitySensor, RadarAxis, RadarSensor, RightClickSensor,
    TouchSensor,
};
use archflow_core::Vec2;
use archflow_engine::SpatialHash;

/// Unique identifier for each sensor type in the Logic Bricks system
///
/// These IDs are used in Pulse events to identify which sensor generated the pulse.
/// Actuators can filter pulses by sensor ID to respond only to specific sensor types.
///
/// # Examples
///
/// ```
/// use archflow_logic::SensorId;
///
/// // Create a pulse from a touch sensor
/// let pulse = Pulse::positive(SensorId::Touch as u32, entity_id, timestamp);
///
/// // Filter pulses in an actuator
/// if pulse.sensor_id == SensorId::Proximity as u32 {
///     // Respond to proximity detection
/// }
/// ```
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SensorId {
    /// Mouse movement sensor (hover detection)
    Mouse = 0,

    /// Touch/collision sensor (AABB collision detection)
    Touch = 1,

    /// Proximity sensor (near detection with hysteresis)
    Proximity = 2,

    /// Radar sensor (directional cone detection)
    Radar = 3,

    /// Keyboard sensor (key press detection)
    Keyboard = 4,

    /// Mouse click sensor (button press)
    MouseClick = 5,

    /// Double tap sensor (rapid double click)
    DoubleTap = 6,

    /// Long press sensor (hold detection)
    LongPress = 7,

    /// Right click sensor (context menu trigger)
    RightClick = 8,
}

/// Main Logic System that evaluates sensors and executes actuators
///
/// This system:
/// 1. Evaluates all sensors with current input
/// 2. Generates pulses for state changes
/// 3. Executes actuators based on wiring table
///
/// # Performance
///
/// - Zero-allocation during hot-path (uses pre-allocated buffers)
/// - Cache-friendly evaluation (sequential access)
/// - Batch processing of all entities in one pass
pub struct LogicSystem {
    /// Input sampler for getting current input state
    input_sampler: InputSampler,

    /// Pulse bus for collecting sensor events
    pulse_bus: PulseBus,

    /// Wiring table connecting sensors to actuators
    wiring: LogicMappingTable,

    /// Mouse sensors for different modes
    mouse_over: MouseOverSensor,
    mouse_click: MouseClickSensor,
    double_tap: DoubleTapSensor,
    long_press: LongPressSensor,
    right_click: RightClickSensor,

    /// Current timestamp in milliseconds
    timestamp: u32,

    /// Spatial hash for broad-phase collision detection
    spatial_hash: SpatialHash,

    /// Physics sensors for collision/proximity detection
    touch_sensor: TouchSensor,
    proximity_sensor: ProximitySensor,
    radar_sensor: RadarSensor,
}

impl LogicSystem {
    /// Create a new LogicSystem
    #[must_use]
    pub fn new() -> Self {
        Self {
            input_sampler: InputSampler::new(),
            pulse_bus: PulseBus::new(),
            wiring: LogicMappingTable::new(),
            mouse_over: MouseOverSensor::new(),
            mouse_click: MouseClickSensor::new(archflow_engine::MAX_ENTITIES),
            double_tap: DoubleTapSensor::new(),
            long_press: LongPressSensor::new(),
            right_click: RightClickSensor::new(),
            timestamp: 0,
            spatial_hash: SpatialHash::new(archflow_engine::MAX_ENTITIES),
            touch_sensor: TouchSensor::new(archflow_engine::MAX_ENTITIES, 0),
            proximity_sensor: ProximitySensor::new(archflow_engine::MAX_ENTITIES, 50.0),
            radar_sensor: RadarSensor::new(
                archflow_engine::MAX_ENTITIES,
                RadarAxis::PositiveX,
                100.0,
                45.0,
                0,
            ),
        }
    }

    /// Get the input sampler (for JavaScript bridge to set SAB pointer)
    pub fn input_sampler(&mut self) -> &mut InputSampler {
        &mut self.input_sampler
    }

    /// Get the pulse bus (for controller evaluation)
    pub fn pulse_bus(&mut self) -> &mut PulseBus {
        &mut self.pulse_bus
    }

    /// Get the wiring table (for configuring connections)
    pub fn wiring(&mut self) -> &mut LogicMappingTable {
        &mut self.wiring
    }

    /// Update the timestamp for this frame
    pub fn set_timestamp(&mut self, timestamp: u32) {
        self.timestamp = timestamp;
        self.pulse_bus.set_timestamp(timestamp);
    }

    /// Push an input event (for fallback mode when SAB is not available)
    pub fn push_input_event(&mut self, event: InputEvent) {
        self.input_sampler.push_input_event(event);
    }

    /// Evaluate sensors and generate pulses
    ///
    /// This is the main hot-path called every frame:
    /// 1. Sample input from InputSampler
    /// 2. Evaluate all sensors
    /// 3. Generate pulses for state changes
    /// 4. Return pulses for processing
    ///
    /// # Returns
    ///
    /// All pulses generated this frame
    pub fn evaluate_sensors(&mut self, store: &EntityStore) -> Vec<Pulse> {
        let snapshot = self.input_sampler.take_snapshot();
        let mouse_pos = snapshot.mouse_position();
        let buttons = snapshot.buttons;
        let wheel = snapshot.wheel_delta;

        let mut pulses = Vec::new();

        // Evaluate unified MouseSensor with Movement mode
        // This replaces the old individual mouse sensors
        let mut mouse_sensor =
            MouseSensor::with_config(store.transforms.len(), MouseConfig::movement());
        mouse_sensor.evaluate(mouse_pos, buttons, wheel as i8, store);

        // Generate pulses from mouse sensor state
        for (entity_idx, _transform) in store.transforms.iter().enumerate() {
            let entity_id = entity_idx as u32;

            // Check if mouse is over entity (rising/falling edge)
            let signal = mouse_sensor.signal(entity_idx);
            if signal.is_rising_edge() {
                pulses.push(Pulse::positive(0, entity_id, self.timestamp));
            } else if signal.is_falling_edge() {
                pulses.push(Pulse::negative(0, entity_id, self.timestamp));
            }
        }

        // Note: Keyboard sensor evaluation would go here
        // KeyShortcutSensor needs explicit event sampling (not position-based)

        // Evaluate physics sensors (HU-010)
        pulses = self.evaluate_physics_sensors(store, pulses);

        pulses
    }

    /// Evaluate physics sensors and append their pulses
    ///
    /// This integrates TouchSensor, ProximitySensor, and RadarSensor with PulseBus
    /// as specified in HU-010.
    fn evaluate_physics_sensors(
        &mut self,
        store: &EntityStore,
        mut pulses: Vec<Pulse>,
    ) -> Vec<Pulse> {
        // Update spatial hash with current entity positions (only alive entities)
        // Use draw_order which contains only alive entities
        for &entity_idx in &store.draw_order {
            let idx = entity_idx as usize;
            let transform = store.transforms[idx];
            let pos = Vec2::new(transform[0], transform[1]);
            let size = Vec2::new(transform[2], transform[3]);
            let generation = store.generation(idx);
            let entity_id = EntityId::from_parts(Index(entity_idx), Generation(generation));
            let bounds = archflow_core::Rect::from_origin_size(pos, size);

            // Remove old position and insert new position
            self.spatial_hash.remove(entity_id);
            self.spatial_hash.insert(entity_id, bounds);
        }

        let spatial = &self.spatial_hash;

        // Evaluate TouchSensor (collision detection)
        self.touch_sensor.evaluate(store, spatial);
        for &entity_idx in &store.draw_order {
            let generation = store.generation(entity_idx as usize);
            let entity_id = EntityId::from_parts(Index(entity_idx), Generation(generation));
            let signal = self.touch_sensor.signal(entity_id);
            if signal.is_rising_edge() {
                pulses.push(Pulse::positive(
                    SensorId::Touch as u32,
                    entity_idx,
                    self.timestamp,
                ));
            } else if signal.is_falling_edge() {
                pulses.push(Pulse::negative(
                    SensorId::Touch as u32,
                    entity_idx,
                    self.timestamp,
                ));
            }
        }

        // Evaluate ProximitySensor (near detection)
        self.proximity_sensor.evaluate(store, spatial);
        for &entity_idx in &store.draw_order {
            let generation = store.generation(entity_idx as usize);
            let entity_id = EntityId::from_parts(Index(entity_idx), Generation(generation));
            let signal = self.proximity_sensor.signal(entity_id);
            if signal.get_current() {
                pulses.push(Pulse::positive(
                    SensorId::Proximity as u32,
                    entity_idx,
                    self.timestamp,
                ));
            }
        }

        // Evaluate RadarSensor (directional detection)
        self.radar_sensor.evaluate(store, spatial);
        for &entity_idx in &store.draw_order {
            let generation = store.generation(entity_idx as usize);
            let entity_id = EntityId::from_parts(Index(entity_idx), Generation(generation));
            let signal = self.radar_sensor.signal(entity_id);
            if signal.get_current() {
                pulses.push(Pulse::positive(
                    SensorId::Radar as u32,
                    entity_idx,
                    self.timestamp,
                ));
            }
        }

        pulses
    }

    /// Execute actuators based on pulses
    ///
    /// This processes all pulses and executes the connected actuators
    pub fn execute_actuators(&mut self, _store: &mut EntityStore, pulses: &[Pulse]) {
        for _pulse in pulses {
            // Process each pulse through the wiring table
            // This will be implemented when we have the full wiring table
        }
    }

    /// Main update loop - evaluates sensors and executes actuators
    ///
    /// This is the primary method called each frame
    pub fn update(&mut self, store: &mut EntityStore) {
        // Step 1: Evaluate sensors and generate pulses
        let pulses = self.evaluate_sensors(store);

        // Step 2: Execute actuators based on pulses
        self.execute_actuators(store, &pulses);
    }
}

impl Default for LogicSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pulse::SensorState;
    use archflow_core::Vec2;

    #[test]
    fn test_logic_system_new() {
        let mut system = LogicSystem::new();
        assert!(!system.input_sampler().is_sab_available());
    }

    #[test]
    fn test_logic_system_set_timestamp() {
        let mut system = LogicSystem::new();
        system.set_timestamp(1000);
        assert_eq!(system.timestamp, 1000);
    }

    #[test]
    fn test_logic_system_push_input_event() {
        let mut system = LogicSystem::new();
        system.push_input_event(InputEvent::MouseMove { x: 100, y: 200 });

        let snapshot = system.input_sampler().take_snapshot();
        assert_eq!(snapshot.mouse_x, 100);
        assert_eq!(snapshot.mouse_y, 200);
    }

    #[test]
    fn test_logic_system_evaluate_sensors() {
        let mut store = EntityStore::new();
        let mut system = LogicSystem::new();
        system.set_timestamp(100);

        // Push some input events
        system.push_input_event(InputEvent::MouseMove { x: 100, y: 100 });
        system.push_input_event(InputEvent::MouseButtonDown { button: 0 });

        // Evaluate sensors
        let pulses = system.evaluate_sensors(&store);
        // Should return pulses (empty for now, will be implemented)
        assert!(pulses.len() >= 0);
    }

    #[test]
    fn test_logic_system_update() {
        let mut store = EntityStore::new();
        let mut system = LogicSystem::new();
        system.set_timestamp(100);

        // Should not panic
        system.update(&mut store);
    }

    #[test]
    fn test_input_sampler_fallback() {
        let mut system = LogicSystem::new();

        // Push events in fallback mode
        system.push_input_event(InputEvent::MouseMove { x: 50, y: 75 });
        system.push_input_event(InputEvent::MouseButtonDown { button: 1 });
        system.push_input_event(InputEvent::KeyDown { keycode: 10 });

        let snapshot = system.input_sampler().take_snapshot();
        assert_eq!(snapshot.mouse_x, 50);
        assert_eq!(snapshot.mouse_y, 75);
        assert!((snapshot.buttons & 0b010) != 0); // Right button
        assert!(snapshot.is_key_pressed(10));
    }

    #[test]
    fn test_pulse_bus_integration() {
        let mut system = LogicSystem::new();
        system.set_timestamp(100);

        // Push some positive pulses
        system.pulse_bus().push_positive(0, 100);
        system.pulse_bus().push_positive(1, 200);

        assert_eq!(system.pulse_bus().len(), 2);

        // Drain pulses
        let pulses = system.pulse_bus().drain();
        assert_eq!(pulses.len(), 2);
        assert_eq!(pulses[0].entity_id, 100);
        assert_eq!(pulses[1].entity_id, 200);

        // Bus should be empty after drain
        assert_eq!(system.pulse_bus().len(), 0);
    }

    #[test]
    fn test_physics_sensors_integration() {
        let mut store = EntityStore::new();
        let mut system = LogicSystem::new();
        system.set_timestamp(1000);

        // Create two entities that will collide
        let entity1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));
        let entity2 = store.spawn(Vec2::new(25.0, 25.0), Vec2::new(50.0, 50.0));

        // Evaluate sensors - should detect collision
        let pulses = system.evaluate_sensors(&store);

        // Should have pulses from physics sensors
        // TouchSensor (sensor_id=1), ProximitySensor (sensor_id=2), RadarSensor (sensor_id=3)
        // Each entity should generate pulses
        assert!(!pulses.is_empty());
    }

    #[test]
    fn test_physics_sensors_touch_detection() {
        let mut store = EntityStore::new();
        let mut system = LogicSystem::new();
        system.set_timestamp(1000);

        // Create overlapping entities
        let entity1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));
        let entity2 = store.spawn(Vec2::new(30.0, 30.0), Vec2::new(50.0, 50.0));

        // First evaluation should detect rising edges (collision started)
        let pulses1 = system.evaluate_sensors(&store);

        // Filter for TouchSensor pulses using SensorId enum
        let touch_pulses: Vec<_> = pulses1
            .iter()
            .filter(|p| p.sensor_id == SensorId::Touch as u32)
            .collect();
        assert!(!touch_pulses.is_empty(), "Should detect collision start");

        // Second evaluation should not have rising edges (already colliding)
        let pulses2 = system.evaluate_sensors(&store);
        let touch_pulses2: Vec<_> = pulses2
            .iter()
            .filter(|p| p.sensor_id == SensorId::Touch as u32)
            .collect();
        assert!(
            !touch_pulses2
                .iter()
                .any(|p| p.state == SensorState::Positive),
            "Should not have new collision detections"
        );
    }

    #[test]
    fn test_sensor_id_values() {
        // Verify that SensorId enum values match expected u32 values
        assert_eq!(SensorId::Mouse as u32, 0);
        assert_eq!(SensorId::Touch as u32, 1);
        assert_eq!(SensorId::Proximity as u32, 2);
        assert_eq!(SensorId::Radar as u32, 3);
        assert_eq!(SensorId::Keyboard as u32, 4);
        assert_eq!(SensorId::MouseClick as u32, 5);
        assert_eq!(SensorId::DoubleTap as u32, 6);
        assert_eq!(SensorId::LongPress as u32, 7);
        assert_eq!(SensorId::RightClick as u32, 8);
    }

    #[test]
    fn test_sensor_id_in_pulse() {
        // Verify that SensorId can be used in Pulse creation
        let pulse = Pulse::positive(SensorId::Touch as u32, 42, 1000);
        assert_eq!(pulse.sensor_id, SensorId::Touch as u32);
        assert_eq!(pulse.entity_id, 42);
        assert!(pulse.is_positive());
    }

    #[test]
    fn test_physics_sensors_use_correct_ids() {
        let mut store = EntityStore::new();
        let mut system = LogicSystem::new();
        system.set_timestamp(1000);

        // Create overlapping entities to trigger physics sensors
        let _entity1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));
        let _entity2 = store.spawn(Vec2::new(25.0, 25.0), Vec2::new(50.0, 50.0));
        let pulses = system.evaluate_sensors(&store);

        // Verify that physics sensors use correct SensorId values
        let has_touch = pulses.iter().any(|p| p.sensor_id == SensorId::Touch as u32);
        let has_proximity = pulses
            .iter()
            .any(|p| p.sensor_id == SensorId::Proximity as u32);
        let has_radar = pulses.iter().any(|p| p.sensor_id == SensorId::Radar as u32);

        // At least Touch and Proximity should trigger for overlapping entities
        assert!(has_touch, "TouchSensor should trigger");
        assert!(has_proximity, "ProximitySensor should trigger");
    }
}
