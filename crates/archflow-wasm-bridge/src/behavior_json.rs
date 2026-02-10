// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Web - Behavior JSON API
//
// Implements the declarative JSON API for behaviors as specified in the
// Developer Manual. This allows JavaScript developers to define behaviors
// using JSON configuration, following the A-Frame pattern.
//
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]

use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use serde::Deserialize;
use wasm_bindgen::prelude::*;

use crate::logic::{
    factories::{self, *},
    logic_system::LogicSystemWasm,
    sensor_type::SensorType,
};

// ═══════════════════════════════════════════════════════════════════════════════
// TYPE DEFINITIONS - JSON Deserialization
// ═══════════════════════════════════════════════════════════════════════════════

/// Behavior definition as specified in JSON
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BehaviorDefinition {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub components: Vec<ComponentDefinition>,
}

/// Individual component definition
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentDefinition {
    /// Component type (e.g., "sensor-mouse", "actuator-highlight")
    #[serde(rename = "type")]
    pub component_type: String,
    /// Component configuration
    pub config: serde_json::Value,
}

/// Sensor configuration variants
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SensorMouseConfig {
    pub mode: String,
    #[serde(default = "default_button")]
    pub button: u8,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SensorKeyboardConfig {
    pub keys: Vec<u32>,
    #[serde(default)]
    pub modifiers: u8,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SensorTimerConfig {
    pub duration_ms: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SensorPropertyConfig {
    pub property: String,
}

/// Actuator configuration variants
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActuatorHighlightConfig {
    #[serde(default = "default_highlight_color")]
    pub color: String,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActuatorSelectConfig {
    #[serde(default = "default_select_mode")]
    pub mode: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActuatorMoveConfig {
    #[serde(default = "default_move_mode")]
    pub mode: String,
    #[serde(default = "default_speed")]
    pub speed: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActuatorEventConfig {
    pub name: String,
    #[serde(default)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActuatorDeleteConfig {}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActuatorPropertyConfig {
    pub property: String,
    pub value: serde_json::Value,
}

/// Controller configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControllerDirectConfig {}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControllerDebounceConfig {
    #[serde(default = "default_debounce_ticks")]
    pub ticks: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControllerHysteresisConfig {
    #[serde(default = "default_hysteresis_threshold")]
    pub threshold: f32,
}

// ═══════════════════════════════════════════════════════════════════════════════
// DEFAULT VALUES
// ═══════════════════════════════════════════════════════════════════════════════

fn default_button() -> u8 { 0 }
fn default_highlight_color() -> String { "#ffff00".to_string() }
fn default_opacity() -> f32 { 0.5 }
fn default_select_mode() -> String { "single".to_string() }
fn default_move_mode() -> String { "follow-cursor".to_string() }
fn default_speed() -> f32 { 5.0 }
fn default_debounce_ticks() -> u32 { 6 }
fn default_hysteresis_threshold() -> f32 { 0.1 }

// ═══════════════════════════════════════════════════════════════════════════════
// ERROR TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug)]
pub enum BehaviorError {
    ParseError(String),
    InvalidComponentType(String),
    InvalidConfig(String),
    RegistrationError(String),
}

impl std::fmt::Display for BehaviorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BehaviorError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            BehaviorError::InvalidComponentType(t) => write!(f, "Invalid component type: {}", t),
            BehaviorError::InvalidConfig(msg) => write!(f, "Invalid config: {}", msg),
            BehaviorError::RegistrationError(msg) => write!(f, "Registration error: {}", msg),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BEHAVIOR REGISTRY
// ═══════════════════════════════════════════════════════════════════════════════

/// Registry for managing behavior definitions and their execution
pub struct BehaviorRegistry {
    logic_system: RefCell<Option<LogicSystemWasm>>,
}

impl BehaviorRegistry {
    pub fn new() -> Self {
        Self {
            logic_system: RefCell::new(None),
        }
    }

    pub fn set_logic_system(&self, system: LogicSystemWasm) {
        *self.logic_system.borrow_mut() = Some(system);
    }

    /// Register a single behavior from JSON
    pub fn register_behavior(&self, json: &str) -> Result<(), BehaviorError> {
        let definition: BehaviorDefinition = serde_json::from_str(json)
            .map_err(|e| BehaviorError::ParseError(e.to_string()))?;

        self.register_behavior_definition(&definition)
    }

    /// Register multiple behaviors from JSON array
    pub fn register_behaviors(&self, json_array: &str) -> Result<(), BehaviorError> {
        let definitions: Vec<BehaviorDefinition> = serde_json::from_str(json_array)
            .map_err(|e| BehaviorError::ParseError(e.to_string()))?;

        for definition in &definitions {
            self.register_behavior_definition(definition)?;
        }

        Ok(())
    }

    /// Register a behavior definition
    fn register_behavior_definition(&self, definition: &BehaviorDefinition) -> Result<(), BehaviorError> {
        let logic_system = self.logic_system.borrow()
            .as_ref()
            .ok_or_else(|| BehaviorError::RegistrationError("Logic system not initialized".to_string()))?;

        // Process components and create behavior chain
        let mut builder = crate::logic::BrickChainBuilder::new(logic_system);

        for component in &definition.components {
            self.process_component(&mut builder, component)?;
        }

        // Build the behavior chain
        builder.build();

        Ok(())
    }

    /// Process a single component definition
    fn process_component(
        &self,
        builder: &mut crate::logic::BrickChainBuilder,
        component: &ComponentDefinition,
    ) -> Result<(), BehaviorError> {
        match component.component_type.as_str() {
            // Sensors
            "sensor-mouse" => self.process_sensor_mouse(builder, &component.config),
            "sensor-keyboard" => self.process_sensor_keyboard(builder, &component.config),
            "sensor-timer" => self.process_sensor_timer(builder, &component.config),
            "sensor-property" => self.process_sensor_property(builder, &component.config),

            // Actuators
            "actuator-highlight" => self.process_actuator_highlight(builder, &component.config),
            "actuator-select" => self.process_actuator_select(builder, &component.config),
            "actuator-move" => self.process_actuator_move(builder, &component.config),
            "actuator-event" => self.process_actuator_event(builder, &component.config),
            "actuator-delete" => self.process_actuator_delete(builder, &component.config),
            "actuator-property" => self.process_actuator_property(builder, &component.config),

            // Controllers
            "controller-direct" => self.process_controller_direct(builder, &component.config),
            "controller-debounce" => self.process_controller_debounce(builder, &component.config),
            "controller-hysteresis" => self.process_controller_hysteresis(builder, &component.config),

            _ => Err(BehaviorError::InvalidComponentType(component.component_type.clone())),
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // SENSOR PROCESSORS
    // ═══════════════════════════════════════════════════════════════════════════════

    fn process_sensor_mouse(
        &self,
        builder: &mut crate::logic::BrickChainBuilder,
        config: &serde_json::Value,
    ) -> Result<(), BehaviorError> {
        let mouse_config: SensorMouseConfig = serde_json::from_value(config.clone())
            .map_err(|e| BehaviorError::InvalidConfig(format!("mouse sensor: {}", e)))?;

        match mouse_config.mode.as_str() {
            "hover" => {
                builder.sensor(sensor_mouse_hover());
            }
            "click" => {
                builder.sensor(sensor_mouse_click(mouse_config.button));
            }
            "drag" => {
                builder.sensor(sensor_mouse_drag(mouse_config.button));
            }
            "wheel" => {
                builder.sensor(sensor_mouse_wheel());
            }
            _ => {
                return Err(BehaviorError::InvalidConfig(format!("unknown mouse mode: {}", mouse_config.mode)));
            }
        }

        Ok(())
    }

    fn process_sensor_keyboard(
        &self,
        builder: &mut crate::logic::BrickChainBuilder,
        config: &serde_json::Value,
    ) -> Result<(), BehaviorError> {
        let key_config: SensorKeyboardConfig = serde_json::from_value(config.clone())
            .map_err(|e| BehaviorError::InvalidConfig(format!("keyboard sensor: {}", e)))?;

        for key in &key_config.keys {
            builder.sensor(sensor_keyboard_key(*key));
        }

        Ok(())
    }

    fn process_sensor_timer(
        &self,
        builder: &mut crate::logic::BrickChainBuilder,
        config: &serde_json::Value,
    ) -> Result<(), BehaviorError> {
        let timer_config: SensorTimerConfig = serde_json::from_value(config.clone())
            .map_err(|e| BehaviorError::InvalidConfig(format!("timer sensor: {}", e)))?;

        builder.sensor(sensor_timer_delay(timer_config.duration_ms));

        Ok(())
    }

    fn process_sensor_property(
        &self,
        builder: &mut crate::logic::BrickChainBuilder,
        config: &serde_json::Value,
    ) -> Result<(), BehaviorError> {
        let prop_config: SensorPropertyConfig = serde_json::from_value(config.clone())
            .map_err(|e| BehaviorError::InvalidConfig(format!("property sensor: {}", e)))?;

        builder.sensor(sensor_property_changed(&prop_config.property));

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ACTUATOR PROCESSORS
    // ═══════════════════════════════════════════════════════════════════════════════

    fn process_actuator_highlight(
        &self,
        builder: &mut crate::logic::BrickChainBuilder,
        config: &serde_json::Value,
    ) -> Result<(), BehaviorError> {
        let highlight_config: ActuatorHighlightConfig = serde_json::from_value(config.clone())
            .map_err(|e| BehaviorError::InvalidConfig(format!("highlight actuator: {}", e)))?;

        let color = parse_color(&highlight_config.color)?;
        builder.actuator(actuator_highlight(color, highlight_config.opacity));

        Ok(())
    }

    fn process_actuator_select(
        &self,
        builder: &mut crate::logic::BrickChainBuilder,
        config: &serde_json::Value,
    ) -> Result<(), BehaviorError> {
        let select_config: ActuatorSelectConfig = serde_json::from_value(config.clone())
            .map_err(|e| BehaviorError::InvalidConfig(format!("select actuator: {}", e)))?;

        match select_config.mode.as_str() {
            "single" => {
                builder.actuator(actuator_select_single());
            }
            "multi" => {
                builder.actuator(actuator_select_multi());
            }
            "toggle" => {
                builder.actuator(actuator_select_toggle());
            }
            "clear" => {
                builder.actuator(actuator_select_clear());
            }
            _ => {
                return Err(BehaviorError::InvalidConfig(format!("unknown select mode: {}", select_config.mode)));
            }
        }

        Ok(())
    }

    fn process_actuator_move(
        &self,
        builder: &mut crate::logic::BrickChainBuilder,
        config: &serde_json::Value,
    ) -> Result<(), BehaviorError> {
        let move_config: ActuatorMoveConfig = serde_json::from_value(config.clone())
            .map_err(|e| BehaviorError::InvalidConfig(format!("move actuator: {}", e)))?;

        let move_config_type = match move_config.mode.as_str() {
            "follow-cursor" => MoveConfigType::FollowCursor,
            "relative" => MoveConfigType::Relative,
            _ => {
                return Err(BehaviorError::InvalidConfig(format!("unknown move mode: {}", move_config.mode)));
            }
        };

        builder.actuator(actuator_move(move_config_type.to_config(move_config.speed)));

        Ok(())
    }

    fn process_actuator_event(
        &self,
        builder: &mut crate::logic::BrickChainBuilder,
        config: &serde_json::Value,
    ) -> Result<(), BehaviorError> {
        let event_config: ActuatorEventConfig = serde_json::from_value(config.clone())
            .map_err(|e| BehaviorError::InvalidConfig(format!("event actuator: {}", e)))?;

        builder.actuator(actuator_emit_event(&event_config.name, &event_config.data.to_string()));

        Ok(())
    }

    fn process_actuator_delete(
        &self,
        builder: &mut crate::logic::BrickChainBuilder,
        _config: &serde_json::Value,
    ) -> Result<(), BehaviorError> {
        builder.actuator(actuator_delete());

        Ok(())
    }

    fn process_actuator_property(
        &self,
        builder: &mut crate::logic::BrickChainBuilder,
        config: &serde_json::Value,
    ) -> Result<(), BehaviorError> {
        let prop_config: ActuatorPropertyConfig = serde_json::from_value(config.clone())
            .map_err(|e| BehaviorError::InvalidConfig(format!("property actuator: {}", e)))?;

        let value = match prop_config.value {
            serde_json::Value::String(s) => PropertyValue::String(s),
            serde_json::Value::Number(n) => {
                if let Some(f) = n.as_f64() {
                    PropertyValue::Float(f as f32)
                } else if let Some(i) = n.as_i64() {
                    PropertyValue::Int(i as i32)
                } else {
                    return Err(BehaviorError::InvalidConfig("invalid number value".to_string()));
                }
            }
            serde_json::Value::Bool(b) => PropertyValue::Bool(b),
            _ => {
                return Err(BehaviorError::InvalidConfig("unsupported property value type".to_string()));
            }
        };

        builder.actuator(factories::actuator_property(&prop_config.property, value));

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // CONTROLLER PROCESSORS
    // ═══════════════════════════════════════════════════════════════════════════════

    fn process_controller_direct(
        &self,
        builder: &mut crate::logic::BrickChainBuilder,
        _config: &serde_json::Value,
    ) -> Result<(), BehaviorError> {
        builder.controller(factory_direct());

        Ok(())
    }

    fn process_controller_debounce(
        &self,
        builder: &mut crate::logic::BrickChainBuilder,
        config: &serde_json::Value,
    ) -> Result<(), BehaviorError> {
        let debounce_config: ControllerDebounceConfig = serde_json::from_value(config.clone())
            .map_err(|e| BehaviorError::InvalidConfig(format!("debounce controller: {}", e)))?;

        builder.controller(factory_debounce(debounce_config.ticks));

        Ok(())
    }

    fn process_controller_hysteresis(
        &self,
        builder: &mut crate::logic::BrickChainBuilder,
        config: &serde_json::Value,
    ) -> Result<(), BehaviorError> {
        let hysteresis_config: ControllerHysteresisConfig = serde_json::from_value(config.clone())
            .map_err(|e| BehaviorError::InvalidConfig(format!("hysteresis controller: {}", e)))?;

        builder.controller(factory_hysteresis(hysteresis_config.threshold));

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPER FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════

/// Parse hex color string to u32 (ABGR format for WebGL)
fn parse_color(hex: &str) -> Result<u32, BehaviorError> {
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 {
        return Err(BehaviorError::InvalidConfig("invalid color format".to_string()));
    }

    let r = u8::from_str_radix(&hex[0..2], 16)
        .map_err(|_| BehaviorError::InvalidConfig("invalid color".to_string()))?;
    let g = u8::from_str_radix(&hex[2..4], 16)
        .map_err(|_| BehaviorError::InvalidConfig("invalid color".to_string()))?;
    let b = u8::from_str_radix(&hex[4..6], 16)
        .map_err(|_| BehaviorError::InvalidConfig("invalid color".to_string()))?;

    // Convert to ABGR format
    Ok(0xFF000000 | ((b as u32) << 16) | ((g as u32) << 8) | (r as u32))
}

/// Move configuration type enum
enum MoveConfigType {
    FollowCursor,
    Relative,
}

impl MoveConfigType {
    fn to_config(&self, speed: f32) -> MoveConfig {
        match self {
            MoveConfigType::FollowCursor => MoveConfig {
                axis: archflow_logic::MoveAxis::Both,
                snap: 0.0,
                speed,
            },
            MoveConfigType::Relative => MoveConfig {
                axis: archflow_logic::MoveAxis::Both,
                snap: 0.0,
                speed,
            },
        }
    }
}

impl Default for BehaviorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_color_valid() {
        assert_eq!(parse_color("#ff0000").unwrap(), 0xFF0000FF);
        assert_eq!(parse_color("#00ff00").unwrap(), 0xFF00FF00);
        assert_eq!(parse_color("#0000ff").unwrap(), 0xFFFF0000);
    }

    #[test]
    fn test_parse_color_invalid() {
        assert!(parse_color("#fff").is_err());
        assert!(parse_color("ffffff").is_err());
        assert!(parse_color("#gggggg").is_err());
    }

    #[test]
    fn test_behavior_definition_parse() {
        let json = r#"{
            "id": "hover-highlight",
            "name": "Hover Highlight",
            "description": "Highlights on hover",
            "components": [
                {
                    "type": "sensor-mouse",
                    "config": { "mode": "hover" }
                },
                {
                    "type": "actuator-highlight",
                    "config": { "color": "#ffff00", "opacity": 0.5 }
                }
            ]
        }"#;

        let definition: BehaviorDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(definition.id, "hover-highlight");
        assert_eq!(definition.name, "Hover Highlight");
        assert_eq!(definition.components.len(), 2);
    }
}
