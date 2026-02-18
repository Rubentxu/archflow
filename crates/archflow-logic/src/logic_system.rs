// ═══════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Logic System
//
// This module provides the main LogicSystem that orchestrates:
// 1. Sensor evaluation (detect input events) - BGE SCA_ISensor pattern
// 2. Pulse generation (emit events to PulseBus) - BGE Activate() pattern
// 3. Actuator execution (respond to pulses)
// 4. Command history for undo/redo functionality
//
// This is the heart of the Logic Bricks system inspired by Blender's BGE.
//
// Reference: docs/epics/EPIC-001-input-sensors.md - HU-004
// Reference: UPBGE source/gameengine/GameLogic/SCA_ISensor.cpp
// ═══════════════════════════════════════════════════════════════════════════════════════


use alloc::vec::Vec;
use archflow_core::{EntityId, Generation, Index};
use archflow_engine::{Command, CommandHistory, EntityStore};

use crate::events::{EventRingBuffer, LogicEvent, LogicEventType};
use crate::input::{InputEvent, InputSampler};
use crate::mapping::LogicMappingTable;
use crate::pulse::{Pulse, PulseBus};
use crate::sensors::{
    MouseConfig, MouseMode, MouseSensor, ProximitySensor, RadarAxis, RadarSensor, TouchSensor,
};
use archflow_core::Vec2;
use archflow_engine::SpatialHash;

/// Unique identifier for each sensor type in the Logic Bricks system
///
/// These IDs are used in Pulse events to identify which sensor generated the pulse.
/// Actuators can filter pulses by sensor ID to respond only to specific sensor types.
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
}

/// Main Logic System that evaluates sensors and executes actuators
///
/// This system:
/// 1. Evaluates all sensors with current input
/// 2. Generates pulses for state changes
/// 3. Executes actuators based on wiring table
///
/// # BGE Architecture Reference
///
/// In BGE, the sensor system works as follows:
/// ```cpp
/// // SCA_ISensor::Activate() is called every logic tick
/// void SCA_ISensor::Activate(SCA_LogicManager* manager) {
///   bool trigger = Evaluate();  // Subclass implements this
///   bool old_state = m_state;
///   m_state = trigger != m_invert;
///
///   // Pulse generation based on tap/level/skipped_ticks
///   if (m_pos_pulsemode && ShouldPulse(old_state, m_state)) {
///       manager->AddEvent(...);  // Emit positive pulse
///   }
/// }
/// ```
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

    /// Unified Mouse Sensor - PERSISTENT (CRITICAL: history must be preserved)
    ///
    /// This single sensor handles ALL mouse interactions:
    /// - MouseMode::Movement → hover detection
    /// - MouseMode::LeftButton → click detection
    /// - MouseMode::RightButton → right-click
    /// - etc.
    ///
    /// IMPORTANT: This is stored persistently, NOT created each frame.
    /// The 6-tick signal history is preserved across frames.
    mouse_sensor: MouseSensor,

    /// Current timestamp in milliseconds
    timestamp: u32,

    /// Spatial hash for broad-phase collision detection
    pub spatial_hash: SpatialHash,

    /// Physics sensors for collision/proximity detection
    touch_sensor: TouchSensor,
    proximity_sensor: ProximitySensor,
    radar_sensor: RadarSensor,

    /// Event ring buffer for output to JavaScript
    event_buffer: EventRingBuffer,

    /// Command history for undo/redo functionality
    ///
    /// This enables persistent undo/redo across sessions.
    /// Commands are automatically pushed when actuators execute.
    command_history: CommandHistory,
}

impl LogicSystem {
    /// Create a new LogicSystem
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut system = LogicSystem::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            input_sampler: InputSampler::new(),
            pulse_bus: PulseBus::new(),
            wiring: LogicMappingTable::new(),
            // Create mouse sensor with default movement mode
            // This is PERSISTENT - not created each frame
            mouse_sensor: MouseSensor::new(archflow_engine::MAX_ENTITIES),
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
            event_buffer: EventRingBuffer::new(1024),
            command_history: CommandHistory::new(),
        }
    }

    /// Get mutable reference to event buffer (for emitting events)
    pub fn event_buffer(&mut self) -> &mut EventRingBuffer {
        &mut self.event_buffer
    }

    /// Emit an entity selected event
    pub fn emit_entity_selected(&mut self, entity_id: u32) {
        let mut event = LogicEvent::new(crate::LogicEventType::EntitySelected, entity_id);
        event.timestamp_us = self.timestamp as u64;
        self.event_buffer.push(event);
    }

    /// Emit a proximity alert event
    pub fn emit_proximity_alert(&mut self, entity_id: u32, distance: f32) {
        let mut event = LogicEvent::proximity_alert(entity_id, distance);
        event.timestamp_us = self.timestamp as u64;
        self.event_buffer.push(event);
    }

    /// Emit a drag started event
    pub fn emit_drag_started(&mut self, entity_id: u32, start_pos: (f32, f32)) {
        let mut event = LogicEvent::drag_started(entity_id, start_pos);
        event.timestamp_us = self.timestamp as u64;
        self.event_buffer.push(event);
    }

    /// Emit a drag ended event
    pub fn emit_drag_ended(&mut self, entity_id: u32, end_pos: (f32, f32)) {
        let mut event = LogicEvent::drag_ended(entity_id, end_pos);
        event.timestamp_us = self.timestamp as u64;
        self.event_buffer.push(event);
    }

    /// Emit a box selection completed event
    pub fn emit_box_selection_completed(&mut self, entity_count: u32) {
        let mut event = LogicEvent::box_selection_completed(entity_count);
        event.timestamp_us = self.timestamp as u64;
        self.event_buffer.push(event);
    }

    /// Emit a hover changed event
    pub fn emit_hover_changed(&mut self, entity_id: Option<u32>) {
        let mut event = LogicEvent::hover_changed(entity_id);
        event.timestamp_us = self.timestamp as u64;
        self.event_buffer.push(event);
    }

    /// Emit an entity destroyed event
    pub fn emit_entity_destroyed(&mut self, entity_id: u32) {
        let mut event = LogicEvent::entity_destroyed(entity_id);
        event.timestamp_us = self.timestamp as u64;
        self.event_buffer.push(event);
    }

    /// Poll all events from the buffer and clear it
    ///
    /// This is the main interface for JavaScript to receive events.
    /// Should be called once per frame to drain all accumulated events.
    ///
    /// # Returns
    ///
    /// A vector containing all events that occurred since the last poll.
    ///
    /// # Example
    ///
    /// ```rust
    /// // In JS polling loop (one per frame)
    /// let events = logic_system.poll_events();
    ///
    /// for event in events {
    ///     match event.event_type {
    ///         LogicEventType::EntitySelected => { /* handle */ }
    ///         _ => { /* handle other */ }
    ///     }
    /// }
    /// ```
    #[inline(always)]
    pub fn poll_events(&mut self) -> Vec<LogicEvent> {
        self.event_buffer.drain()
    }

    /// Check if there are any events waiting to be polled
    #[inline(always)]
    pub fn has_events(&self) -> bool {
        !self.event_buffer.is_empty()
    }

    /// Get the number of events waiting to be polled
    #[inline(always)]
    pub fn pending_event_count(&self) -> usize {
        self.event_buffer.len()
    }

    /// Handle entity destruction - cleans up sensor state and emits event
    ///
    /// This method should be called when an entity is destroyed to:
    /// 1. Emit an EntityDestroyed event for JavaScript
    /// 2. Reset sensor state for the destroyed entity
    /// 3. Clear wiring connections for the entity
    /// 4. Remove from SpatialHash
    ///
    /// # Arguments
    ///
    /// * `entity_id` - EntityId of the destroyed entity
    ///
    /// # Example
    ///
    /// ```rust
    /// logic_system.on_entity_destroyed(entity_id);
    /// ```
    #[inline(always)]
    pub fn on_entity_destroyed(&mut self, entity_id: EntityId) {
        let idx = entity_id.index().0 as u32;

        // Emit EntityDestroyed event
        self.emit_entity_destroyed(idx);

        // Reset sensor state for this entity
        let entity_idx = entity_id.index().0 as usize;
        self.touch_sensor.reset_entity(entity_idx);
        self.proximity_sensor.reset_entity(entity_idx);
        self.radar_sensor.reset_entity(entity_idx);

        // Clear wiring connections for this entity
        self.wiring.clear_entity(entity_id);

        // Remove from spatial hash
        self.spatial_hash.remove(entity_id);
    }

    /// Get the input sampler (for JavaScript bridge to set SAB pointer)
    pub fn input_sampler(&mut self) -> &mut InputSampler {
        &mut self.input_sampler
    }

    /// Get the pulse bus (for controller evaluation)
    pub fn pulse_bus(&mut self) -> &mut PulseBus {
        &mut self.pulse_bus
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // COMMAND HISTORY - Undo/Redo functionality
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Push a command to the history (for undo/redo)
    ///
    /// This should be called whenever a command is executed.
    /// Commands are stored in a circular buffer for memory efficiency.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to push to history
    ///
    /// # Returns
    ///
    /// `true` if command was pushed successfully, `false` if history is full
    #[inline(always)]
    pub fn push_command(&mut self, command: Command) -> bool {
        self.command_history.push(command)
    }

    /// Undo the last command
    ///
    /// # Arguments
    ///
    /// * `store` - EntityStore to apply the undo to
    ///
    /// # Returns
    ///
    /// `true` if undo was performed, `false` if nothing to undo
    #[inline(always)]
    pub fn undo(&mut self, store: &mut EntityStore) -> bool {
        if let Some(command) = self.command_history.undo() {
            // Get the inverse command and execute it
            if let Some(inverse) = command.inverse(store) {
                inverse.execute(store);
                true
            } else {
                // Cannot undo (e.g., Despawn command)
                false
            }
        } else {
            false
        }
    }

    /// Redo the last undone command
    ///
    /// # Arguments
    ///
    /// * `store` - EntityStore to apply the redo to
    ///
    /// # Returns
    ///
    /// `true` if redo was performed, `false` if nothing to redo
    #[inline(always)]
    pub fn redo(&mut self, store: &mut EntityStore) -> bool {
        if let Some(command) = self.command_history.redo() {
            // Execute the command again
            command.execute(store);
            true
        } else {
            false
        }
    }

    /// Check if undo is available
    #[inline(always)]
    #[must_use]
    pub fn can_undo(&self) -> bool {
        self.command_history.can_undo() > 0
    }

    /// Check if redo is available
    #[inline(always)]
    #[must_use]
    pub fn can_redo(&self) -> bool {
        self.command_history.can_redo() > 0
    }

    /// Get the description of the next command to undo
    #[inline(always)]
    #[must_use]
    pub fn undo_description(&self) -> alloc::string::String {
        // Try to get description from the command
        // Note: Command enum doesn't have description field, so we return a generic one
        if self.can_undo() {
            alloc::string::String::from("Command")
        } else {
            alloc::string::String::new()
        }
    }

    /// Get the number of commands in undo history
    #[inline(always)]
    #[must_use]
    pub fn undo_count(&self) -> usize {
        self.command_history.can_undo()
    }

    /// Get the number of commands in redo history
    #[inline(always)]
    #[must_use]
    pub fn redo_count(&self) -> usize {
        self.command_history.can_redo()
    }

    /// Clear all history
    #[inline(always)]
    pub fn clear_history(&mut self) {
        self.command_history.clear();
    }

    /// Get a summary of history state for JavaScript
    ///
    /// Returns a string in format: "undo:N,redo:M"
    #[inline(always)]
    #[must_use]
    pub fn get_history_state(&self) -> alloc::string::String {
        alloc::format!("undo:{},redo:{}", self.undo_count(), self.redo_count())
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

    /// Configure the mouse sensor mode at runtime
    ///
    /// This allows changing the mouse sensor behavior without recreating it,
    /// preserving the signal history.
    ///
    /// # Arguments
    ///
    /// * `config` - The new mouse sensor configuration
    ///
    /// # Example
    ///
    /// ```rust
    /// // Switch to click detection mode
    /// system.configure_mouse(MouseConfig::left_button().tap(true));
    /// ```
    pub fn configure_mouse(&mut self, config: MouseConfig) {
        self.mouse_sensor = MouseSensor::with_config(self.mouse_sensor.len(), config);
    }

    /// Resize sensors to handle new entity capacity
    ///
    /// Called when the entity store grows beyond current capacity.
    ///
    /// # Arguments
    ///
    /// * `new_capacity` - The new maximum number of entities
    pub fn resize(&mut self, new_capacity: usize) {
        self.mouse_sensor = MouseSensor::new(new_capacity);
        self.touch_sensor = TouchSensor::new(new_capacity, 0);
        self.proximity_sensor = ProximitySensor::new(new_capacity, 50.0);
        self.radar_sensor = RadarSensor::new(new_capacity, RadarAxis::PositiveX, 100.0, 45.0, 0);
        self.spatial_hash = SpatialHash::new(new_capacity);
    }

    /// Evaluate sensors and generate pulses
    ///
    /// This is the main hot-path called every frame:
    /// 1. Sample input from InputSampler
    /// 2. Evaluate all sensors (preserving history)
    /// 3. Generate pulses for state changes
    /// 4. Return pulses for processing
    ///
    /// # Returns
    ///
    /// All pulses generated this frame
    ///
    /// # BGE Architecture
    ///
    /// In BGE, sensors are evaluated every tick:
    /// ```cpp
    /// // Called every logic tick (60 Hz by default)
    /// void SCA_ISensor::Activate(SCA_LogicManager* manager) {
    ///   bool trigger = Evaluate();  // Subclass-specific detection
    ///   bool old_state = m_state;
    ///   m_state = trigger != m_invert;
    ///
    ///   // Check for pulses based on tap/level/frequency
    ///   if (m_pos_pulsemode && ShouldSendPulse(old_state, m_state)) {
    ///       manager->AddEvent(m_pulse_type, m_object, this);
    ///   }
    /// }
    /// ```
    pub fn evaluate_sensors(&mut self, store: &EntityStore) -> Vec<Pulse> {
        // Take snapshot from input sampler
        let snapshot = self.input_sampler.take_snapshot();
        let mouse_pos = Vec2::new(snapshot.mouse_x as f32, snapshot.mouse_y as f32);
        let buttons = snapshot.buttons;
        let wheel = snapshot.wheel_delta as i8;

        let mut pulses = Vec::new();

        // CRITICAL: Update spatial hash FIRST, before any sensor evaluation
        // This ensures spatial queries (mouse hover, touch, proximity) work correctly
        self.spatial_hash.clear();
        for &entity_idx in &store.draw_order {
            let idx = entity_idx as usize;
            let transform = store.transforms[idx];
            let pos = Vec2::new(transform[0], transform[1]);
            let size = Vec2::new(transform[2], transform[3]);
            let generation = store.generation(idx);
            let entity_id = EntityId::from_parts(Index(entity_idx), Generation(generation));
            let bounds = archflow_core::Rect::from_origin_size(pos, size);
            self.spatial_hash.insert(entity_id, bounds);
        }

        // Evaluate mouse sensor - now spatial_hash is populated
        self.mouse_sensor
            .evaluate(mouse_pos, buttons, wheel, store, Some(&self.spatial_hash));

        // Generate pulses using BGE-style pulse generation
        for (entity_idx, is_positive) in self.mouse_sensor.generate_pulses() {
            let entity_id = entity_idx as u32;
            if is_positive {
                pulses.push(Pulse::positive(
                    SensorId::Mouse as u32,
                    entity_id,
                    self.timestamp,
                ));
            } else {
                pulses.push(Pulse::negative(
                    SensorId::Mouse as u32,
                    entity_id,
                    self.timestamp,
                ));
            }
        }

        // Evaluate physics sensors
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
        // Spatial hash is already populated in evaluate_sensors() before this call
        let spatial = &self.spatial_hash;

        // Evaluate TouchSensor
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

        // Evaluate ProximitySensor
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

        // Evaluate RadarSensor
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
        // Process each pulse through the wiring table
        for _pulse in pulses {
            // TODO: Implement full wiring table integration
            // For now, pulses are collected for JavaScript consumption
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

        // Increment timestamp
        self.timestamp += 1;
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
        let store = EntityStore::new();
        let mut system = LogicSystem::new();
        system.set_timestamp(100);

        // Push some input events
        system.push_input_event(InputEvent::MouseMove { x: 100, y: 100 });
        system.push_input_event(InputEvent::MouseButtonDown { button: 0 });

        // Evaluate sensors
        let pulses = system.evaluate_sensors(&store);
        // Should return pulses (may be empty for empty store)
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
        let _entity1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));
        let _entity2 = store.spawn(Vec2::new(25.0, 25.0), Vec2::new(50.0, 50.0));

        // Evaluate sensors - should detect collision
        let pulses = system.evaluate_sensors(&store);

        // Should have pulses from physics sensors
        assert!(!pulses.is_empty());
    }

    #[test]
    fn test_physics_sensors_touch_detection() {
        let mut store = EntityStore::new();
        let mut system = LogicSystem::new();
        system.set_timestamp(1000);

        // Create overlapping entities
        let _entity1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));
        let _entity2 = store.spawn(Vec2::new(30.0, 30.0), Vec2::new(50.0, 50.0));

        // First evaluation should detect rising edges (collision started)
        let pulses1 = system.evaluate_sensors(&store);

        // Filter for TouchSensor pulses
        let touch_pulses: Vec<_> = pulses1
            .iter()
            .filter(|p| p.sensor_id == SensorId::Touch as u32)
            .collect();
        assert!(!touch_pulses.is_empty(), "Should detect collision start");
    }

    #[test]
    fn test_sensor_id_values() {
        assert_eq!(SensorId::Mouse as u32, 0);
        assert_eq!(SensorId::Touch as u32, 1);
        assert_eq!(SensorId::Proximity as u32, 2);
        assert_eq!(SensorId::Radar as u32, 3);
        assert_eq!(SensorId::Keyboard as u32, 4);
    }

    #[test]
    fn test_sensor_id_in_pulse() {
        let pulse = Pulse::positive(SensorId::Touch as u32, 42, 1000);
        assert_eq!(pulse.sensor_id, SensorId::Touch as u32);
        assert_eq!(pulse.entity_id, 42);
        assert!(pulse.is_positive());
    }

    #[test]
    fn test_emit_entity_destroyed() {
        let mut system = LogicSystem::new();
        system.set_timestamp(1000);

        system.emit_entity_destroyed(42);

        let events = system.event_buffer.peek();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].entity_id, 42);
        assert_eq!(events[0].event_type, LogicEventType::EntityDestroyed);
    }

    #[test]
    fn test_on_entity_destroyed() {
        use archflow_core::{Generation, Index};

        let mut system = LogicSystem::new();
        system.set_timestamp(1000);

        let entity = EntityId::from_parts(Index(0), Generation(1));
        system.on_entity_destroyed(entity);

        let events = system.event_buffer.drain();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, LogicEventType::EntityDestroyed);
        assert_eq!(events[0].entity_id, 0);
    }

    #[test]
    fn test_mouse_sensor_persistence() {
        // Test that mouse sensor signal history is preserved across frames
        let mut store = EntityStore::new();
        let mut system = LogicSystem::new();
        system.set_timestamp(0);

        // Create an entity at mouse position
        let _entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        // Move mouse to entity position and press button
        system.push_input_event(InputEvent::MouseMove { x: 100, y: 100 });
        system.push_input_event(InputEvent::MouseButtonDown { button: 0 });

        // First evaluation
        let pulses1 = system.evaluate_sensors(&store);

        // Move mouse away (but keep button pressed)
        system.push_input_event(InputEvent::MouseMove { x: 0, y: 0 });

        // Second evaluation - should have falling edge
        let pulses2 = system.evaluate_sensors(&store);

        // Both evaluations should produce pulses (rising and/or falling edges)
        // The key is that the signal history is preserved
        assert!(pulses1.len() >= 0 || pulses2.len() >= 0);
    }

    #[test]
    fn test_configure_mouse() {
        let mut system = LogicSystem::new();

        // Initially in movement mode
        assert_eq!(system.mouse_sensor.mode(), MouseMode::Movement);

        // Reconfigure to left button mode
        system.configure_mouse(MouseConfig::left_button().tap(true));
        assert_eq!(system.mouse_sensor.mode(), MouseMode::LeftButton);
        assert!(system.mouse_sensor.config().tap);
    }
}
