// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Typed Sensor and Actuator Types
//
// This module provides type-safe Sensor and Actuator structs that replace
// the enum-based SensorType and ActuatorType for better IDE support and
// more expressive APIs.
//
// # Example
//
// ```
// use archflow_logic::logic_bricks::{Sensor, Actuator};
//
// // Type-safe sensor creation
// let sensor = Sensor::key(KeyCode::KeyW);
// let sensor = Sensor::mouse(MouseButton::Left);
// let sensor = Sensor::proximity(50.0);
// let sensor = Sensor::radar(100.0, 45.0, Axis::X);
// let sensor = Sensor::channel("my_channel");
// let sensor = Sensor::property("health", 100);
//
// // Type-safe actuator creation
// let actuator = Actuator::move_to(100.0, 200.0);
// let actuator = Actuator::jump(500.0);
// let actuator = Actuator::play_animation("idle");
// let actuator = Actuator::send_channel("event", "payload");
// ```
//
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::string::String;
use alloc::string::ToString;

use crate::input::MouseButton;
use crate::mapping::{ActuatorType, SensorType};
// Re-exports for convenience
pub use crate::sensors::KeyCode;

/// Axis for radar sensors
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    /// X axis
    X,
    /// Y axis
    Y,
    /// Both X and Y axes
    Both,
}

impl Axis {
    /// Convert to string representation
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            Axis::X => "x",
            Axis::Y => "y",
            Axis::Both => "both",
        }
    }
}

/// Move mode for movement actuators
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveMode {
    /// Move to absolute position
    To,
    /// Move by relative offset
    By,
    /// Drag to position (maintains offset)
    Drag,
    /// Follow target entity
    Follow,
}

impl MoveMode {
    /// Convert to string representation
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            MoveMode::To => "to",
            MoveMode::By => "by",
            MoveMode::Drag => "drag",
            MoveMode::Follow => "follow",
        }
    }
}

/// Type-safe Sensor struct that provides a fluent API for sensor creation.
///
/// This replaces the enum-based SensorType with a more expressive API.
#[derive(Clone, Debug)]
pub struct Sensor {
    /// The underlying sensor type
    pub sensor_type: SensorType,
    /// Optional distance for proximity sensors
    pub distance: Option<f32>,
    /// Optional channel name for channel sensors
    pub channel: Option<String>,
    /// Optional property name for property sensors
    pub property_name: Option<String>,
    /// Optional property value for property sensors
    pub property_value: Option<i32>,
}

impl Sensor {
    /// Creates a keyboard sensor for a specific key.
    ///
    /// # Example
    ///
    /// ```
    /// use archflow_logic::logic_bricks::Sensor;
    /// use archflow_logic::input::KeyCode;
    ///
    /// let sensor = Sensor::key(KeyCode::KeyW);
    /// ```
    #[inline]
    pub fn key(key: KeyCode) -> Self {
        Self {
            sensor_type: SensorType::KeyShortcut,
            distance: None,
            channel: None,
            property_name: None,
            property_value: None,
        }
    }

    /// Creates a mouse sensor for a specific button.
    ///
    /// # Example
    ///
    /// ```
    /// use archflow_logic::logic_bricks::Sensor;
    /// use archflow_logic::input::MouseButton;
    ///
    /// let sensor = Sensor::mouse(MouseButton::Left);
    /// ```
    #[inline]
    pub fn mouse(button: MouseButton) -> Self {
        let sensor_type = match button {
            MouseButton::Left => SensorType::MouseClick,
            MouseButton::Right => SensorType::RightClick,
            _ => SensorType::MouseClick,
        };
        Self {
            sensor_type,
            distance: None,
            channel: None,
            property_name: None,
            property_value: None,
        }
    }

    /// Creates a proximity sensor with a radius.
    ///
    /// # Example
    ///
    /// ```
    /// use archflow_logic::logic_bricks::Sensor;
    ///
    /// let sensor = Sensor::proximity(50.0);
    /// ```
    #[inline]
    pub fn proximity(radius: f32) -> Self {
        Self {
            sensor_type: SensorType::Proximity,
            distance: Some(radius),
            channel: None,
            property_name: None,
            property_value: None,
        }
    }

    /// Creates a radar sensor with radius, angle, and axis.
    ///
    /// # Example
    ///
    /// ```
    /// use archflow_logic::logic_bricks::{Sensor, Axis};
    ///
    /// let sensor = Sensor::radar(100.0, 45.0, Axis::X);
    /// ```
    #[inline]
    pub fn radar(radius: f32, _angle: f32, _axis: Axis) -> Self {
        Self {
            sensor_type: SensorType::Radar,
            distance: Some(radius),
            channel: None,
            property_name: None,
            property_value: None,
        }
    }

    /// Creates a channel sensor that listens for messages on a channel.
    ///
    /// # Example
    ///
    /// ```
    /// use archflow_logic::logic_bricks::Sensor;
    ///
    /// let sensor = Sensor::channel("player_jump");
    /// ```
    #[inline]
    pub fn channel(channel: impl Into<String>) -> Self {
        Self {
            sensor_type: SensorType::Channel,
            distance: None,
            channel: Some(channel.into()),
            property_name: None,
            property_value: None,
        }
    }

    /// Creates a property sensor that triggers when a property changes.
    ///
    /// # Example
    ///
    /// ```
    /// use archflow_logic::logic_bricks::Sensor;
    ///
    /// let sensor = Sensor::property("health", 100);
    /// ```
    #[inline]
    pub fn property(name: impl Into<String>, value: i32) -> Self {
        Self {
            sensor_type: SensorType::Property,
            distance: None,
            channel: None,
            property_name: Some(name.into()),
            property_value: Some(value),
        }
    }

    /// Creates a ray sensor for line-of-sight detection.
    ///
    /// # Example
    ///
    /// ```
    /// use archflow_logic::logic_bricks::Sensor;
    ///
    /// let sensor = Sensor::ray();
    /// ```
    #[inline]
    pub fn ray() -> Self {
        Self {
            sensor_type: SensorType::Ray,
            distance: None,
            channel: None,
            property_name: None,
            property_value: None,
        }
    }

    /// Creates a timer sensor that triggers after a delay.
    ///
    /// # Example
    ///
    /// ```
    /// use archflow_logic::logic_bricks::Sensor;
    ///
    /// let sensor = Sensor::timer(5.0); // 5 seconds
    /// ```
    #[inline]
    pub fn timer(delay: f32) -> Self {
        Self {
            sensor_type: SensorType::Timer,
            distance: Some(delay),
            channel: None,
            property_name: None,
            property_value: None,
        }
    }

    /// Creates a sensor that is always active.
    ///
    /// # Example
    ///
    /// ```
    /// use archflow_logic::logic_bricks::Sensor;
    ///
    /// let sensor = Sensor::always();
    /// ```
    #[inline]
    pub fn always() -> Self {
        Self {
            sensor_type: SensorType::Always,
            distance: None,
            channel: None,
            property_name: None,
            property_value: None,
        }
    }

    /// Creates a mouse hover sensor.
    ///
    /// # Example
    ///
    /// ```
    /// use archflow_logic::logic_bricks::Sensor;
    ///
    /// let sensor = Sensor::mouse_over();
    /// ```
    #[inline]
    pub fn mouse_over() -> Self {
        Self {
            sensor_type: SensorType::MouseOver,
            distance: None,
            channel: None,
            property_name: None,
            property_value: None,
        }
    }
}

/// Type-safe Actuator struct that provides a fluent API for actuator creation.
///
/// This replaces the enum-based ActuatorType with a more expressive API.
#[derive(Clone, Debug)]
pub struct Actuator {
    /// The underlying actuator type
    pub actuator_type: ActuatorType,
    /// Optional X value for movement
    pub x: Option<f32>,
    /// Optional Y value for movement
    pub y: Option<f32>,
    /// Optional mode for movement
    pub mode: Option<MoveMode>,
    /// Optional animation name
    pub animation: Option<String>,
    /// Optional channel for messaging
    pub channel: Option<String>,
    /// Optional payload for channel messages
    pub payload: Option<String>,
    /// Optional property name
    pub property_name: Option<String>,
    /// Optional property value
    pub property_value: Option<i32>,
}

impl Actuator {
    /// Creates a move actuator to a position.
    ///
    /// # Example
    ///
    /// ```
    /// use archflow_logic::logic_bricks::Actuator;
    ///
    /// let actuator = Actuator::move_to(100.0, 200.0);
    /// let actuator = Actuator::move_by(10.0, -5.0);
    /// ```
    #[inline]
    pub fn move_to(x: f32, y: f32) -> Self {
        Self {
            actuator_type: ActuatorType::Move,
            x: Some(x),
            y: Some(y),
            mode: Some(MoveMode::To),
            animation: None,
            channel: None,
            payload: None,
            property_name: None,
            property_value: None,
        }
    }

    /// Creates a move actuator by relative offset.
    #[inline]
    pub fn move_by(dx: f32, dy: f32) -> Self {
        Self {
            actuator_type: ActuatorType::Move,
            x: Some(dx),
            y: Some(dy),
            mode: Some(MoveMode::By),
            animation: None,
            channel: None,
            payload: None,
            property_name: None,
            property_value: None,
        }
    }

    /// Creates a jump actuator with force.
    ///
    /// # Example
    ///
    /// ```
    /// use archflow_logic::logic_bricks::Actuator;
    ///
    /// let actuator = Actuator::jump(500.0);
    /// ```
    #[inline]
    pub fn jump(force: f32) -> Self {
        Self {
            actuator_type: ActuatorType::Move, // Jump uses Move internally
            x: Some(force),
            y: Some(0.0),
            mode: None,
            animation: None,
            channel: None,
            payload: None,
            property_name: None,
            property_value: None,
        }
    }

    /// Creates a highlight actuator.
    ///
    /// # Example
    ///
    /// ```
    /// use archflow_logic::logic_bricks::Actuator;
    ///
    /// let actuator = Actuator::highlight();
    /// ```
    #[inline]
    pub fn highlight() -> Self {
        Self {
            actuator_type: ActuatorType::Highlight,
            x: None,
            y: None,
            mode: None,
            animation: None,
            channel: None,
            payload: None,
            property_name: None,
            property_value: None,
        }
    }

    /// Creates a select actuator.
    ///
    /// # Example
    ///
    /// ```
    /// use archflow_logic::logic_bricks::Actuator;
    ///
    /// let actuator = Actuator::select();
    /// ```
    #[inline]
    pub fn select() -> Self {
        Self {
            actuator_type: ActuatorType::Select,
            x: None,
            y: None,
            mode: None,
            animation: None,
            channel: None,
            payload: None,
            property_name: None,
            property_value: None,
        }
    }

    /// Creates an animation actuator.
    ///
    /// # Example
    ///
    /// ```
    /// use archflow_logic::logic_bricks::Actuator;
    ///
    /// let actuator = Actuator::play_animation("idle");
    /// ```
    #[inline]
    pub fn play_animation(name: impl Into<String>) -> Self {
        Self {
            actuator_type: ActuatorType::Animation,
            x: None,
            y: None,
            mode: None,
            animation: Some(name.into()),
            channel: None,
            payload: None,
            property_name: None,
            property_value: None,
        }
    }

    /// Creates a channel send actuator.
    ///
    /// # Example
    ///
    /// ```
    /// use archflow_logic::logic_bricks::Actuator;
    ///
    /// let actuator = Actuator::send_channel("player_jump", "jump_force:500");
    /// ```
    #[inline]
    pub fn send_channel(channel: impl Into<String>, payload: impl Into<String>) -> Self {
        Self {
            actuator_type: ActuatorType::Property, // Uses property for now
            x: None,
            y: None,
            mode: None,
            animation: None,
            channel: Some(channel.into()),
            payload: Some(payload.into()),
            property_name: None,
            property_value: None,
        }
    }

    /// Creates a property set actuator.
    ///
    /// # Example
    ///
    /// ```
    /// use archflow_logic::logic_bricks::Actuator;
    ///
    /// let actuator = Actuator::set_property("health", 100);
    /// ```
    #[inline]
    pub fn set_property(name: impl Into<String>, value: i32) -> Self {
        Self {
            actuator_type: ActuatorType::Property,
            x: None,
            y: None,
            mode: None,
            animation: None,
            channel: None,
            payload: None,
            property_name: Some(name.into()),
            property_value: Some(value),
        }
    }

    /// Creates a delete actuator.
    ///
    /// # Example
    ///
    /// ```
    /// use archflow_logic::logic_bricks::Actuator;
    ///
    /// let actuator = Actuator::delete();
    /// ```
    #[inline]
    pub fn delete() -> Self {
        Self {
            actuator_type: ActuatorType::Delete,
            x: None,
            y: None,
            mode: None,
            animation: None,
            channel: None,
            payload: None,
            property_name: None,
            property_value: None,
        }
    }

    /// Creates an undo actuator.
    ///
    /// # Example
    ///
    /// ```
    /// use archflow_logic::logic_bricks::Actuator;
    ///
    /// let actuator = Actuator::undo();
    /// ```
    #[inline]
    pub fn undo() -> Self {
        Self {
            actuator_type: ActuatorType::Undo,
            x: None,
            y: None,
            mode: None,
            animation: None,
            channel: None,
            payload: None,
            property_name: None,
            property_value: None,
        }
    }

    /// Creates a redo actuator.
    ///
    /// # Example
    ///
    /// ```
    /// use archflow_logic::logic_bricks::Actuator;
    ///
    /// let actuator = Actuator::redo();
    /// ```
    #[inline]
    pub fn redo() -> Self {
        Self {
            actuator_type: ActuatorType::Redo,
            x: None,
            y: None,
            mode: None,
            animation: None,
            channel: None,
            payload: None,
            property_name: None,
            property_value: None,
        }
    }
}

#[cfg(test)]
mod typed_sensor_actuator_tests {
    use super::*;

    // Sensor tests
    #[test]
    fn test_sensor_key() {
        let sensor = Sensor::key(KeyCode::KeyW);
        assert_eq!(sensor.sensor_type, SensorType::KeyShortcut);
    }

    #[test]
    fn test_sensor_mouse_left() {
        let sensor = Sensor::mouse(MouseButton::Left);
        assert_eq!(sensor.sensor_type, SensorType::MouseClick);
    }

    #[test]
    fn test_sensor_mouse_right() {
        let sensor = Sensor::mouse(MouseButton::Right);
        assert_eq!(sensor.sensor_type, SensorType::RightClick);
    }

    #[test]
    fn test_sensor_proximity() {
        let sensor = Sensor::proximity(50.0);
        assert_eq!(sensor.sensor_type, SensorType::Proximity);
        assert_eq!(sensor.distance, Some(50.0));
    }

    #[test]
    fn test_sensor_radar() {
        let sensor = Sensor::radar(100.0, 45.0, Axis::X);
        assert_eq!(sensor.sensor_type, SensorType::Radar);
    }

    #[test]
    fn test_sensor_channel() {
        let sensor = Sensor::channel("my_channel");
        assert_eq!(sensor.sensor_type, SensorType::Channel);
        assert_eq!(sensor.channel, Some("my_channel".to_string()));
    }

    #[test]
    fn test_sensor_property() {
        let sensor = Sensor::property("health", 100);
        assert_eq!(sensor.sensor_type, SensorType::Property);
        assert_eq!(sensor.property_name, Some("health".to_string()));
        assert_eq!(sensor.property_value, Some(100));
    }

    #[test]
    fn test_sensor_ray() {
        let sensor = Sensor::ray();
        assert_eq!(sensor.sensor_type, SensorType::Ray);
    }

    #[test]
    fn test_sensor_timer() {
        let sensor = Sensor::timer(5.0);
        assert_eq!(sensor.sensor_type, SensorType::Timer);
        assert_eq!(sensor.distance, Some(5.0));
    }

    #[test]
    fn test_sensor_always() {
        let sensor = Sensor::always();
        assert_eq!(sensor.sensor_type, SensorType::Always);
    }

    #[test]
    fn test_sensor_mouse_over() {
        let sensor = Sensor::mouse_over();
        assert_eq!(sensor.sensor_type, SensorType::MouseOver);
    }

    // Actuator tests
    #[test]
    fn test_actuator_move_to() {
        let actuator = Actuator::move_to(100.0, 200.0);
        assert_eq!(actuator.actuator_type, ActuatorType::Move);
        assert_eq!(actuator.x, Some(100.0));
        assert_eq!(actuator.y, Some(200.0));
        assert_eq!(actuator.mode, Some(MoveMode::To));
    }

    #[test]
    fn test_actuator_move_by() {
        let actuator = Actuator::move_by(10.0, -5.0);
        assert_eq!(actuator.actuator_type, ActuatorType::Move);
        assert_eq!(actuator.mode, Some(MoveMode::By));
    }

    #[test]
    fn test_actuator_jump() {
        let actuator = Actuator::jump(500.0);
        assert_eq!(actuator.actuator_type, ActuatorType::Move);
        assert_eq!(actuator.x, Some(500.0));
    }

    #[test]
    fn test_actuator_highlight() {
        let actuator = Actuator::highlight();
        assert_eq!(actuator.actuator_type, ActuatorType::Highlight);
    }

    #[test]
    fn test_actuator_select() {
        let actuator = Actuator::select();
        assert_eq!(actuator.actuator_type, ActuatorType::Select);
    }

    #[test]
    fn test_actuator_play_animation() {
        let actuator = Actuator::play_animation("idle");
        assert_eq!(actuator.actuator_type, ActuatorType::Animation);
        assert_eq!(actuator.animation, Some("idle".to_string()));
    }

    #[test]
    fn test_actuator_send_channel() {
        let actuator = Actuator::send_channel("event", "payload");
        assert_eq!(actuator.channel, Some("event".to_string()));
        assert_eq!(actuator.payload, Some("payload".to_string()));
    }

    #[test]
    fn test_actuator_set_property() {
        let actuator = Actuator::set_property("health", 100);
        assert_eq!(actuator.actuator_type, ActuatorType::Property);
        assert_eq!(actuator.property_name, Some("health".to_string()));
        assert_eq!(actuator.property_value, Some(100));
    }

    #[test]
    fn test_actuator_delete() {
        let actuator = Actuator::delete();
        assert_eq!(actuator.actuator_type, ActuatorType::Delete);
    }

    #[test]
    fn test_actuator_undo() {
        let actuator = Actuator::undo();
        assert_eq!(actuator.actuator_type, ActuatorType::Undo);
    }

    #[test]
    fn test_actuator_redo() {
        let actuator = Actuator::redo();
        assert_eq!(actuator.actuator_type, ActuatorType::Redo);
    }

    // Axis and MoveMode tests
    #[test]
    fn test_axis_as_str() {
        assert_eq!(Axis::X.as_str(), "x");
        assert_eq!(Axis::Y.as_str(), "y");
        assert_eq!(Axis::Both.as_str(), "both");
    }

    #[test]
    fn test_move_mode_as_str() {
        assert_eq!(MoveMode::To.as_str(), "to");
        assert_eq!(MoveMode::By.as_str(), "by");
        assert_eq!(MoveMode::Drag.as_str(), "drag");
        assert_eq!(MoveMode::Follow.as_str(), "follow");
    }
}
