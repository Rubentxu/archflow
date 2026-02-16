// ═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - BGE Logic System Integration
//
// This module provides the BgeLogicSystem that orchestrates the hybrid BGE architecture:
// 1. Sensor Evaluation: Detects events based on world state and input
// 2. Controller Evaluation: Combines sensor outputs using boolean logic
// 3. Actuator Execution: Performs actions based on controller outputs
//
// ═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════

#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use crate::ecs::hybrid::{
    ActuatorComponent, ClickType, ControllerComponent, PropertyComparator, SensorComponent,
};
use crate::ecs::physics_components::{
    Acceleration, AnimationState, HighlightState, PhysicsMaterial, SelectionState, Transform,
    Velocity,
};
use crate::ecs::query::EntityId;
use crate::ecs::system::System;
use crate::ecs::world::World;
use crate::input::InputSampler;

/// Result of evaluating a sensor
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SensorEvaluation {
    /// Sensor is active (triggered)
    Active = 0,
    /// Sensor is inactive
    Inactive = 1,
    /// Sensor is in undefined state
    None = 2,
}

/// Configuration for BgeLogicSystem
#[derive(Clone, Debug, PartialEq)]
#[repr(C)]
pub struct BgeLogicConfig {
    /// Whether to execute actuators only when controller state changes (edge-triggered)
    pub edge_triggered: bool,
    /// Whether to evaluate sensors every tick or only when triggered
    pub evaluate_always: bool,
    /// Maximum entities to process per batch
    pub batch_size: usize,
    /// Whether to emit debug events for sensor evaluations
    pub debug_events: bool,
    /// Default selection group (for exclusive selection)
    pub default_selection_group: u32,
}

impl Default for BgeLogicConfig {
    fn default() -> Self {
        Self {
            edge_triggered: true,
            evaluate_always: true,
            batch_size: 256,
            debug_events: false,
            default_selection_group: 0,
        }
    }
}

/// Statistics from BgeLogicSystem execution
#[derive(Clone, Debug, Default)]
#[repr(C)]
pub struct BgeLogicStats {
    /// Total entities evaluated
    pub entities_evaluated: usize,
    /// Total sensors evaluated
    pub sensors_evaluated: usize,
    /// Total controllers evaluated
    pub controllers_evaluated: usize,
    /// Total actuators executed
    pub actuators_executed: usize,
    /// Total state changes detected
    pub state_changes: usize,
}

/// State tracking for toggle controllers
#[derive(Clone, Debug, Default)]
#[repr(C)]
struct ToggleState {
    pub is_on: bool,
    pub was_active: bool,
}

/// State tracking for pulse controllers
#[derive(Clone, Debug, Default)]
#[repr(C)]
struct PulseState {
    pub remaining_ticks: u32,
    pub was_active: bool,
}

/// State tracking for delay controllers
#[derive(Clone, Debug, Default)]
#[repr(C)]
struct DelayState {
    pub remaining_ticks: u32,
    pub was_triggered: bool,
}

/// State tracking for one-shot controllers
#[derive(Clone, Debug, Default)]
#[repr(C)]
struct OneShotState {
    pub has_fired: bool,
}

/// State tracking for timer sensors.
#[derive(Clone, Debug, Default)]
#[repr(C)]
struct TimerState {
    /// Remaining ticks until activation.
    pub remaining_ticks: u32,
    /// Whether the timer has already fired.
    pub has_fired: bool,
    /// Whether the timer was active in the previous tick.
    pub was_active: bool,
}

/// BgeLogicSystem: Evaluates sensors, controllers, and executes actuators
///
/// This system implements the BGE logic bricks paradigm using the ECS architecture:
/// - Sensors detect input/world state changes
/// - Controllers combine sensor outputs with boolean logic
/// - Actuators perform actions based on controller outputs
pub struct BgeLogicSystem {
    /// System configuration
    config: BgeLogicConfig,

    /// Statistics from last execution
    stats: BgeLogicStats,

    /// Input sampler for sensor evaluation
    input_sampler: InputSampler,

    /// Previous sensor states (for edge detection)
    previous_states: alloc::collections::BTreeMap<EntityId, SensorEvaluation>,

    /// Toggle states for Toggle controllers
    toggle_states: alloc::collections::BTreeMap<EntityId, ToggleState>,

    /// Pulse states for Pulse controllers
    pulse_states: alloc::collections::BTreeMap<EntityId, PulseState>,

    /// Delay states for Delay controllers
    delay_states: alloc::collections::BTreeMap<EntityId, DelayState>,

    /// One-shot states
    one_shot_states: alloc::collections::BTreeMap<EntityId, OneShotState>,

    /// Timer states for Timer sensors
    timer_states: alloc::collections::BTreeMap<EntityId, TimerState>,

    /// Currently selected entities (for exclusive selection)
    selected_entities: alloc::collections::BTreeMap<u32, Vec<EntityId>>,
}

impl BgeLogicSystem {
    /// Creates a new BgeLogicSystem with default configuration
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: BgeLogicConfig::default(),
            stats: BgeLogicStats::default(),
            input_sampler: InputSampler::new(),
            previous_states: alloc::collections::BTreeMap::new(),
            toggle_states: alloc::collections::BTreeMap::new(),
            pulse_states: alloc::collections::BTreeMap::new(),
            delay_states: alloc::collections::BTreeMap::new(),
            one_shot_states: alloc::collections::BTreeMap::new(),
            timer_states: alloc::collections::BTreeMap::new(),
            selected_entities: alloc::collections::BTreeMap::new(),
        }
    }

    /// Creates a BgeLogicSystem with custom configuration
    #[inline]
    #[must_use]
    pub fn with_config(config: BgeLogicConfig) -> Self {
        Self {
            config,
            stats: BgeLogicStats::default(),
            input_sampler: InputSampler::new(),
            previous_states: alloc::collections::BTreeMap::new(),
            toggle_states: alloc::collections::BTreeMap::new(),
            pulse_states: alloc::collections::BTreeMap::new(),
            delay_states: alloc::collections::BTreeMap::new(),
            one_shot_states: alloc::collections::BTreeMap::new(),
            timer_states: alloc::collections::BTreeMap::new(),
            selected_entities: alloc::collections::BTreeMap::new(),
        }
    }

    /// Returns a reference to the current configuration
    #[inline]
    #[must_use]
    pub const fn config(&self) -> &BgeLogicConfig {
        &self.config
    }

    /// Returns a mutable reference to the configuration
    #[inline]
    pub fn config_mut(&mut self) -> &mut BgeLogicConfig {
        &mut self.config
    }

    /// Returns statistics from the last execution
    #[inline]
    #[must_use]
    pub const fn stats(&self) -> &BgeLogicStats {
        &self.stats
    }

    /// Evaluates a mouse hover sensor
    #[inline]
    fn evaluate_mouse_hover(&self, _entity_id: EntityId, _distance: f32) -> SensorEvaluation {
        let snapshot = self.input_sampler.take_snapshot();
        let _mouse_x = snapshot.mouse_x as f32;
        let _mouse_y = snapshot.mouse_y as f32;

        // Placeholder: always returns inactive
        SensorEvaluation::Inactive
    }

    /// Evaluates a mouse click sensor
    #[inline]
    fn evaluate_mouse_click(
        &self,
        _entity_id: EntityId,
        button: u8,
        click_type: ClickType,
    ) -> SensorEvaluation {
        let snapshot = self.input_sampler.take_snapshot();
        let button_mask = 1 << button;

        let is_pressed = (snapshot.buttons & button_mask) != 0;

        match click_type {
            ClickType::Single => {
                if is_pressed {
                    SensorEvaluation::Active
                } else {
                    SensorEvaluation::Inactive
                }
            }
            ClickType::Double => {
                if is_pressed {
                    SensorEvaluation::Active
                } else {
                    SensorEvaluation::Inactive
                }
            }
            ClickType::Long => {
                if is_pressed {
                    SensorEvaluation::Active
                } else {
                    SensorEvaluation::Inactive
                }
            }
        }
    }

    /// Evaluates a proximity sensor
    #[inline]
    fn evaluate_proximity(&self, _entity_id: EntityId, _radius: f32) -> SensorEvaluation {
        SensorEvaluation::None
    }

    /// Evaluates a key shortcut sensor
    #[inline]
    fn evaluate_key_shortcut(&self, key: u32, _modifiers: u32) -> SensorEvaluation {
        let snapshot = self.input_sampler.take_snapshot();
        let byte_idx = (key / 8) as usize;
        let bit_mask = 1 << (key % 8);

        if byte_idx < 32 && (snapshot.keys[byte_idx] & bit_mask) != 0 {
            SensorEvaluation::Active
        } else {
            SensorEvaluation::Inactive
        }
    }

    /// Evaluates a sensor component and returns its state
    #[inline]
    fn evaluate_sensor(&mut self, entity_id: EntityId, sensor: &SensorComponent) -> SensorEvaluation {
        match sensor {
            SensorComponent::MouseHover { distance } => {
                self.evaluate_mouse_hover(entity_id, *distance)
            }
            SensorComponent::MouseClick { button, click_type } => {
                self.evaluate_mouse_click(entity_id, *button, *click_type)
            }
            SensorComponent::Proximity { radius, .. } => {
                self.evaluate_proximity(entity_id, *radius)
            }
            SensorComponent::KeyShortcut { key, modifiers: _ } => {
                self.evaluate_key_shortcut(*key, 0)
            }
            SensorComponent::DoubleTap { .. } => SensorEvaluation::None,
            SensorComponent::LongPress { .. } => SensorEvaluation::None,
            SensorComponent::RightClick => {
                self.evaluate_mouse_click(entity_id, 1, ClickType::Single)
            }
            SensorComponent::Always => SensorEvaluation::Active,
            SensorComponent::Property {
                property_name,
                comparator,
                target_value,
            } => self.evaluate_property(property_name, *comparator, *target_value),
            SensorComponent::Ray {
                origin,
                direction,
                max_distance,
            } => self.evaluate_ray(*origin, *direction, *max_distance),
            SensorComponent::Timer { duration_ticks } => {
                self.evaluate_timer(entity_id, *duration_ticks)
            }
            SensorComponent::Channel { channel_id } => self.evaluate_channel(*channel_id),
        }
    }

    /// Evaluates a property sensor.
    ///
    /// Compares an entity property against a target value using the specified comparator.
    /// Returns `SensorEvaluation::Active` if the condition is met, `SensorEvaluation::Inactive` otherwise.
    #[inline]
    fn evaluate_property(
        &self,
        _property_name: &str,
        _comparator: PropertyComparator,
        _target_value: f32,
    ) -> SensorEvaluation {
        // TODO: Integrate with actual property system
        // This would need access to entity properties from the ECS
        SensorEvaluation::None
    }

    /// Evaluates a ray sensor.
    ///
    /// Checks if a ray from origin in the given direction intersects with the entity
    /// within the specified maximum distance.
    #[inline]
    fn evaluate_ray(
        &self,
        _origin: [f32; 3],
        _direction: [f32; 3],
        _max_distance: f32,
    ) -> SensorEvaluation {
        // TODO: Integrate with actual raycasting system
        // This would need access to the physics/collision system
        SensorEvaluation::None
    }

    /// Evaluates a timer sensor.
    ///
    /// Tracks elapsed ticks for each entity and triggers when the duration is reached.
    /// The timer resets after triggering unless configured otherwise.
    #[inline]
    fn evaluate_timer(&mut self, entity_id: EntityId, duration_ticks: u32) -> SensorEvaluation {
        let state = self.timer_states.entry(entity_id).or_default();

        if state.remaining_ticks == 0 && !state.was_active {
            // Timer not started yet, initialize it
            state.remaining_ticks = duration_ticks;
            state.was_active = true;
            state.has_fired = false;
        }

        if state.remaining_ticks > 0 {
            state.remaining_ticks -= 1;
            if state.remaining_ticks == 0 && !state.has_fired {
                state.has_fired = true;
                return SensorEvaluation::Active;
            }
        }

        // Reset timer if it has fired and was inactive
        if state.remaining_ticks == 0 && state.was_active && state.has_fired {
            state.was_active = false;
        }

        SensorEvaluation::Inactive
    }

    /// Evaluates a channel sensor.
    ///
    /// Listens for messages on a specific channel and triggers when a message is received.
    #[inline]
    fn evaluate_channel(&self, _channel_id: u32) -> SensorEvaluation {
        // TODO: Integrate with actual message/channel system
        // This would need access to the message passing system
        SensorEvaluation::None
    }

    /// Evaluates a controller component based on sensor state
    #[inline]
    fn evaluate_controller(
        &mut self,
        entity_id: EntityId,
        controller: &ControllerComponent,
        sensor_state: SensorEvaluation,
    ) -> bool {
        match controller {
            ControllerComponent::Direct => sensor_state == SensorEvaluation::Active,

            ControllerComponent::And { conditions } => conditions
                .iter()
                .all(|c| self.evaluate_controller(entity_id, c, sensor_state)),

            ControllerComponent::Or { conditions } => conditions
                .iter()
                .any(|c| self.evaluate_controller(entity_id, c, sensor_state)),

            ControllerComponent::Not { condition } => {
                !self.evaluate_controller(entity_id, condition, sensor_state)
            }

            ControllerComponent::Pulse {
                tick_count,
                controller,
            } => {
                let state = self.pulse_states.entry(entity_id).or_default();

                if sensor_state == SensorEvaluation::Active && !state.was_active {
                    state.remaining_ticks = *tick_count;
                    state.was_active = true;
                } else if sensor_state != SensorEvaluation::Active {
                    state.was_active = false;
                }

                if state.remaining_ticks > 0 {
                    state.remaining_ticks -= 1;
                    true
                } else {
                    false
                }
            }

            ControllerComponent::Toggle { .. } => {
                let state = self.toggle_states.entry(entity_id).or_default();

                if sensor_state == SensorEvaluation::Active && !state.was_active {
                    state.is_on = !state.is_on;
                    state.was_active = true;
                    state.is_on
                } else {
                    if sensor_state != SensorEvaluation::Active {
                        state.was_active = false;
                    }
                    state.is_on
                }
            }

            ControllerComponent::Delay {
                delay_ticks,
                controller,
            } => {
                let state = self.delay_states.entry(entity_id).or_default();

                if sensor_state == SensorEvaluation::Active && !state.was_triggered {
                    state.remaining_ticks = *delay_ticks;
                    state.was_triggered = true;
                } else if sensor_state != SensorEvaluation::Active {
                    state.was_triggered = false;
                }

                if state.remaining_ticks > 0 && state.was_triggered {
                    state.remaining_ticks -= 1;
                    if state.remaining_ticks == 0 {
                        self.evaluate_controller(entity_id, controller, SensorEvaluation::Active)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }

            ControllerComponent::OneShot { controller } => {
                let state = self.one_shot_states.entry(entity_id).or_default();

                if sensor_state == SensorEvaluation::Active && !state.has_fired {
                    state.has_fired = true;
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Executes an actuator based on controller output
    #[inline]
    fn execute_actuator(
        &mut self,
        actuator: &ActuatorComponent,
        controller_output: bool,
        world: &mut World,
        entity_id: EntityId,
    ) {
        if !controller_output {
            return;
        }

        match actuator {
            ActuatorComponent::Highlight { color, pulse } => {
                if !world.has_component::<HighlightState>(entity_id) {
                    world.add_component(
                        entity_id,
                        HighlightState::active(color[0], color[1], color[2]),
                    );
                } else if let Some(highlight) = world.get_component_mut::<HighlightState>(entity_id)
                {
                    highlight.color_r = color[0];
                    highlight.color_g = color[1];
                    highlight.color_b = color[2];
                    highlight.is_highlighted = true;
                    highlight.intensity = 1.0;
                    if *pulse {
                        highlight.pulse_phase = 0.0;
                    }
                }
            }
            ActuatorComponent::Select { exclusive } => {
                let group = self.config.default_selection_group;

                if *exclusive {
                    if let Some(group_entities) = self.selected_entities.get(&group) {
                        for &e in group_entities {
                            if world.has_component::<SelectionState>(e) {
                                if let Some(sel) = world.get_component_mut::<SelectionState>(e) {
                                    sel.deselect();
                                }
                            }
                        }
                    }
                    self.selected_entities.insert(group, vec![entity_id]);
                } else {
                    if let Some(group_entities) = self.selected_entities.get_mut(&group) {
                        if !group_entities.contains(&entity_id) {
                            group_entities.push(entity_id);
                        }
                    } else {
                        self.selected_entities.insert(group, vec![entity_id]);
                    }
                }

                let selection_order = self
                    .selected_entities
                    .get(&group)
                    .map(|v| v.len() as u32)
                    .unwrap_or(1);

                if !world.has_component::<SelectionState>(entity_id) {
                    let mut state = SelectionState::new();
                    state.select(selection_order);
                    world.add_component(entity_id, state);
                } else if let Some(sel) = world.get_component_mut::<SelectionState>(entity_id) {
                    sel.select(selection_order);
                }
            }
            ActuatorComponent::Move {
                velocity,
                local_space: _,
            } => {
                if !world.has_component::<Velocity>(entity_id) {
                    world.add_component(entity_id, Velocity::new(velocity[0], velocity[1]));
                } else if let Some(vel) = world.get_component_mut::<Velocity>(entity_id) {
                    vel.dx = velocity[0];
                    vel.dy = velocity[1];
                }
            }
            ActuatorComponent::Rotate {
                rotation: (axis, angle),
                local_space: _,
            } => {
                let _ = *axis; // Consume the axis, we only use angle in 2D

                if !world.has_component::<Transform>(entity_id) {
                    world.add_component(entity_id, Transform::from_position(0.0, 0.0));
                }

                if let Some(transform) = world.get_component_mut::<Transform>(entity_id) {
                    transform.set_rotation(*angle);
                }
            }
            ActuatorComponent::Scale { scale } => {
                if !world.has_component::<Transform>(entity_id) {
                    world.add_component(
                        entity_id,
                        Transform::from_position_scale(0.0, 0.0, scale[0], scale[1]),
                    );
                } else if let Some(transform) = world.get_component_mut::<Transform>(entity_id) {
                    transform.set_scale(scale[0], scale[1]);
                }
            }
            ActuatorComponent::Sound { sound_id, volume } => {
                if self.config.debug_events {
                    let _msg = alloc::format!("Sound: {} at volume {:.2}", sound_id, volume);
                }
            }
            ActuatorComponent::Animation {
                animation_id,
                loop_animation,
            } => {
                let clip_id = animation_id.parse::<u32>().unwrap_or(0);

                if !world.has_component::<AnimationState>(entity_id) {
                    let mut state = AnimationState::new();
                    state.play(clip_id, 1.0, *loop_animation);
                    world.add_component(entity_id, state);
                } else if let Some(anim) = world.get_component_mut::<AnimationState>(entity_id) {
                    anim.play(clip_id, 1.0, *loop_animation);
                }
            }
            ActuatorComponent::Custom {
                action_type,
                params,
            } => {
                if self.config.debug_events {
                    let _msg = alloc::format!(
                        "Custom action: {} with {} params",
                        action_type,
                        params.len()
                    );
                }
            }
        }
    }

    /// Clears state for entities that no longer exist
    #[inline]
    fn cleanup_states(&mut self, active_entities: &[EntityId]) {
        self.previous_states
            .retain(|entity, _| active_entities.contains(entity));
        self.toggle_states
            .retain(|entity, _| active_entities.contains(entity));
        self.pulse_states
            .retain(|entity, _| active_entities.contains(entity));
        self.delay_states
            .retain(|entity, _| active_entities.contains(entity));
        self.one_shot_states
            .retain(|entity, _| active_entities.contains(entity));
    }
}

impl System for BgeLogicSystem {
    /// Returns the system name
    fn name(&self) -> &'static str {
        "BgeLogicSystem"
    }

    /// Returns the system priority (lower = runs first)
    fn priority(&self) -> i32 {
        30
    }

    /// Runs the BGE logic system
    fn run(&mut self, world: &mut World, _delta_time: f32) {
        // Reset stats
        self.stats = BgeLogicStats::default();

        // Collect entity IDs with sensors
        let sensor_entities: Vec<EntityId> = world
            .entities()
            .filter(|e| world.has_all::<SensorComponent>(*e))
            .collect();

        // Collect actuator execution data
        let mut actuator_executions: Vec<(EntityId, ActuatorComponent)> = Vec::new();

        // Evaluate sensors and controllers
        for entity in &sensor_entities {
            if let Some(sensor) = world.get_component::<SensorComponent>(*entity) {
                self.stats.entities_evaluated += 1;
                self.stats.sensors_evaluated += 1;

                // Evaluate sensor
                let sensor_state = self.evaluate_sensor(*entity, sensor);

                // Store previous state for edge detection
                let previous_state = self.previous_states.insert(*entity, sensor_state);

                // Detect state changes
                let state_changed = match previous_state {
                    Some(prev) => prev != sensor_state,
                    None => true,
                };

                if state_changed {
                    self.stats.state_changes += 1;
                }

                // Evaluate controller if present
                let controller_output =
                    if let Some(controller) = world.get_component::<ControllerComponent>(*entity) {
                        self.stats.controllers_evaluated += 1;
                        let output = self.evaluate_controller(*entity, controller, sensor_state);

                        if self.config.edge_triggered && !state_changed {
                            false
                        } else {
                            output
                        }
                    } else {
                        sensor_state == SensorEvaluation::Active
                    };

                // Collect actuator for execution
                if controller_output || !self.config.edge_triggered {
                    if let Some(actuator) = world.get_component::<ActuatorComponent>(*entity) {
                        actuator_executions.push((*entity, actuator.clone()));
                    }
                }
            }
        }

        // Execute actuators
        for (entity, actuator) in actuator_executions {
            self.execute_actuator(&actuator, true, world, entity);
            self.stats.actuators_executed += 1;
        }

        // Clean up states for removed entities
        self.cleanup_states(&sensor_entities);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::world::World;

    #[test]
    fn test_bge_logic_system_creation() {
        let system = BgeLogicSystem::new();
        assert_eq!(system.name(), "BgeLogicSystem");
        assert_eq!(system.priority(), 30);
    }

    #[test]
    fn test_bge_logic_config_default() {
        let config = BgeLogicConfig::default();
        assert!(config.edge_triggered);
        assert!(config.evaluate_always);
        assert_eq!(config.batch_size, 256);
        assert!(!config.debug_events);
    }

    #[test]
    fn test_bge_logic_stats_default() {
        let stats = BgeLogicStats::default();
        assert_eq!(stats.entities_evaluated, 0);
        assert_eq!(stats.sensors_evaluated, 0);
        assert_eq!(stats.controllers_evaluated, 0);
        assert_eq!(stats.actuators_executed, 0);
    }

    #[test]
    fn test_sensor_evaluation_inactive() {
        let system = BgeLogicSystem::new();
        let entity_id = EntityId::from_usize(0);

        // With no input, sensors should be inactive
        let state = system.evaluate_mouse_click(entity_id, 0, ClickType::Single);
        assert_eq!(state, SensorEvaluation::Inactive);
    }

    #[test]
    fn test_controller_direct() {
        let mut system = BgeLogicSystem::new();

        let active = system.evaluate_controller(
            EntityId::from_usize(0),
            &ControllerComponent::Direct,
            SensorEvaluation::Active,
        );
        assert!(active);

        let inactive = system.evaluate_controller(
            EntityId::from_usize(0),
            &ControllerComponent::Direct,
            SensorEvaluation::Inactive,
        );
        assert!(!inactive);
    }

    #[test]
    fn test_controller_not() {
        let mut system = BgeLogicSystem::new();
        let entity_id = EntityId::from_usize(1);

        let controller = ControllerComponent::Not {
            condition: Box::new(ControllerComponent::Direct),
        };

        let result = system.evaluate_controller(entity_id, &controller, SensorEvaluation::Active);
        assert!(!result);

        let result = system.evaluate_controller(entity_id, &controller, SensorEvaluation::Inactive);
        assert!(result);
    }

    #[test]
    fn test_controller_and() {
        let mut system = BgeLogicSystem::new();
        let entity_id = EntityId::from_usize(2);

        let controller = ControllerComponent::And {
            conditions: vec![ControllerComponent::Direct, ControllerComponent::Direct],
        };

        let result = system.evaluate_controller(entity_id, &controller, SensorEvaluation::Active);
        assert!(result);

        let result = system.evaluate_controller(entity_id, &controller, SensorEvaluation::Inactive);
        assert!(!result);
    }

    #[test]
    fn test_controller_or() {
        let mut system = BgeLogicSystem::new();
        let entity_id = EntityId::from_usize(3);

        let controller = ControllerComponent::Or {
            conditions: vec![
                ControllerComponent::Direct,
                ControllerComponent::Not {
                    condition: Box::new(ControllerComponent::Direct),
                },
            ],
        };

        let result = system.evaluate_controller(entity_id, &controller, SensorEvaluation::Active);
        assert!(result);

        let result = system.evaluate_controller(entity_id, &controller, SensorEvaluation::Inactive);
        assert!(result);
    }

    #[test]
    fn test_controller_toggle() {
        let mut system = BgeLogicSystem::new();
        let entity_id = EntityId::from_usize(4);

        let controller = ControllerComponent::Toggle {
            controller: Box::new(ControllerComponent::Direct),
        };

        // First activation: should toggle to ON
        let result1 = system.evaluate_controller(entity_id, &controller, SensorEvaluation::Active);
        assert!(result1, "First activation should return true");

        // Second call with Active: state is already active, returns current state (true)
        // Toggle waits for Inactive before allowing another change
        let result2 = system.evaluate_controller(entity_id, &controller, SensorEvaluation::Active);
        assert!(result2, "Still active, returns current state");

        // Deactivation: doesn't change is_on, just resets was_active flag
        let result3 =
            system.evaluate_controller(entity_id, &controller, SensorEvaluation::Inactive);
        assert!(result3, "Still ON, just resetting was_active");

        // Next activation: should toggle to OFF
        let result4 = system.evaluate_controller(entity_id, &controller, SensorEvaluation::Active);
        assert!(!result4, "Toggled to OFF");
    }

    #[test]
    fn test_key_shortcut_evaluation() {
        let system = BgeLogicSystem::new();

        let state = system.evaluate_key_shortcut(65, 0);
        assert_eq!(state, SensorEvaluation::Inactive);
    }

    #[test]
    fn test_run_with_empty_world() {
        let mut world = World::new();
        let mut system = BgeLogicSystem::new();

        system.run(&mut world, 1.0 / 60.0);

        assert_eq!(system.stats().entities_evaluated, 0);
        assert_eq!(system.stats().sensors_evaluated, 0);
    }

    #[test]
    fn test_highlight_actuator() {
        let mut world = World::new();
        let entity = world.create_entity();

        let mut system = BgeLogicSystem::new();

        world.add_component(
            entity,
            ActuatorComponent::Highlight {
                color: [1.0, 0.0, 0.0],
                pulse: false,
            },
        );

        system.execute_actuator(
            &ActuatorComponent::Highlight {
                color: [1.0, 0.0, 0.0],
                pulse: false,
            },
            true,
            &mut world,
            entity,
        );

        assert!(world.has_component::<HighlightState>(entity));
    }

    #[test]
    fn test_move_actuator() {
        let mut world = World::new();
        let entity = world.create_entity();

        let mut system = BgeLogicSystem::new();

        system.execute_actuator(
            &ActuatorComponent::Move {
                velocity: [10.0, 20.0, 0.0], // 3D velocity
                local_space: false,
            },
            true,
            &mut world,
            entity,
        );

        assert!(world.has_component::<Velocity>(entity));
    }

    #[test]
    fn test_select_actuator() {
        let mut world = World::new();
        let entity = world.create_entity();

        let mut system = BgeLogicSystem::new();

        system.execute_actuator(
            &ActuatorComponent::Select { exclusive: true },
            true,
            &mut world,
            entity,
        );

        assert!(world.has_component::<SelectionState>(entity));
    }

    #[test]
    fn test_animation_actuator() {
        let mut world = World::new();
        let entity = world.create_entity();

        let mut system = BgeLogicSystem::new();

        system.execute_actuator(
            &ActuatorComponent::Animation {
                animation_id: "1".to_string(),
                loop_animation: true,
            },
            true,
            &mut world,
            entity,
        );

        assert!(world.has_component::<AnimationState>(entity));
    }
}
