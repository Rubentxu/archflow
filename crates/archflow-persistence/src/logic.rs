// ═══════════════════════════════════════════════════════════════════════════════
// Logic Wiring Serializer - Persist Sensor→Controller→Actuator connections
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]
#![allow(clippy::module_name_repetitions)]

use archflow_core::EntityId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::vec::Vec;

use crate::{PersistenceError, PersistenceResult};

// ═══════════════════════════════════════════════════════════════════════════════
// SERIALIZABLE LOGIC WIRING
// ═══════════════════════════════════════════════════════════════════════════════

/// Serializable representation of Logic Bricks wiring
///
/// This represents the Sensor→Controller→Actuator connections that can be
/// persisted and loaded from documents.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SerializableWiring {
    /// All sensor definitions
    pub sensors: Vec<SerializableSensor>,
    /// All controller definitions
    pub controllers: Vec<SerializableController>,
    /// All actuator definitions
    pub actuators: Vec<SerializableActuator>,
    /// Wiring connections between sensors, controllers, and actuators
    pub connections: Vec<SerializableConnection>,
}

/// Serializable sensor definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SerializableSensor {
    /// Sensor ID
    pub id: u32,
    /// Sensor type name
    #[serde(rename = "type")]
    pub type_: String,
    /// Sensor name
    pub name: String,
    /// Sensor configuration
    pub config: serde_json::Value,
}

/// Serializable controller definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SerializableController {
    /// Controller ID
    pub id: u32,
    /// Controller type name
    #[serde(rename = "type")]
    pub type_: String,
    /// Controller name
    pub name: String,
    /// Controller configuration
    pub config: serde_json::Value,
}

/// Serializable actuator definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SerializableActuator {
    /// Actuator ID
    pub id: u32,
    /// Actuator type name
    #[serde(rename = "type")]
    pub type_: String,
    /// Actuator name
    pub name: String,
    /// Actuator configuration
    pub config: serde_json::Value,
}

/// Serializable wiring connection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SerializableConnection {
    /// Target entity ID
    pub entity_id: String,
    /// Sensor ID
    pub sensor_id: u32,
    /// Controller ID
    pub controller_id: u32,
    /// Actuator ID
    pub actuator_id: u32,
    /// Connection priority (for ordering)
    pub priority: u32,
    /// Whether the connection is enabled
    pub enabled: bool,
}

impl SerializableWiring {
    /// Create an empty wiring table
    #[must_use]
    pub fn new() -> Self {
        Self {
            sensors: Vec::new(),
            controllers: Vec::new(),
            actuators: Vec::new(),
            connections: Vec::new(),
        }
    }

    /// Check if the wiring table is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.connections.is_empty()
    }

    /// Get the number of connections
    #[must_use]
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Add a sensor to the wiring table
    pub fn add_sensor(&mut self, sensor: SerializableSensor) {
        self.sensors.push(sensor);
    }

    /// Add a controller to the wiring table
    pub fn add_controller(&mut self, controller: SerializableController) {
        self.controllers.push(controller);
    }

    /// Add an actuator to the wiring table
    pub fn add_actuator(&mut self, actuator: SerializableActuator) {
        self.actuators.push(actuator);
    }

    /// Add a connection to the wiring table
    pub fn add_connection(&mut self, connection: SerializableConnection) {
        self.connections.push(connection);
    }

    /// Get all connections for a specific entity
    #[must_use]
    pub fn connections_for_entity(&self, entity_id: &str) -> Vec<&SerializableConnection> {
        self.connections
            .iter()
            .filter(|c| c.entity_id == entity_id)
            .collect()
    }

    /// Remove all connections for an entity
    pub fn remove_entity_connections(&mut self, entity_id: &str) {
        self.connections.retain(|c| c.entity_id != entity_id);
    }
}

impl Default for SerializableWiring {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SERIALIZER / DESERIALIZER
// ═══════════════════════════════════════════════════════════════════════════════

/// Serializer for converting between internal LogicMappingTable and SerializableWiring
pub struct LogicWiringSerializer;

impl LogicWiringSerializer {
    /// Create an empty serializable wiring table
    #[must_use]
    pub fn new() -> SerializableWiring {
        SerializableWiring::new()
    }

    /// Serialize wiring table to JSON string
    pub fn to_json(wiring: &SerializableWiring) -> PersistenceResult<String> {
        serde_json::to_string(wiring).map_err(|e| PersistenceError::Serialization(e.to_string()))
    }

    /// Serialize wiring table to pretty JSON string
    pub fn to_json_pretty(wiring: &SerializableWiring) -> PersistenceResult<String> {
        serde_json::to_string_pretty(wiring)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))
    }

    /// Deserialize wiring table from JSON string
    pub fn from_json(json: &str) -> PersistenceResult<SerializableWiring> {
        serde_json::from_str(json).map_err(|e| PersistenceError::Deserialization(e.to_string()))
    }

    /// Create a highlight connection (MouseOver → Direct → Highlight)
    pub fn create_highlight_connection(entity_id: EntityId) -> SerializableConnection {
        SerializableConnection {
            entity_id: entity_id_format(entity_id),
            sensor_id: 0,     // MouseOver
            controller_id: 0, // Direct
            actuator_id: 0,   // Highlight
            priority: 0,
            enabled: true,
        }
    }

    /// Create a select connection (MouseClick → Direct → Select)
    pub fn create_select_connection(entity_id: EntityId) -> SerializableConnection {
        SerializableConnection {
            entity_id: entity_id_format(entity_id),
            sensor_id: 1,     // MouseClick
            controller_id: 0, // Direct
            actuator_id: 1,   // Select
            priority: 0,
            enabled: true,
        }
    }

    /// Create a drag connection (MouseDrag → Direct → Move)
    pub fn create_drag_connection(entity_id: EntityId) -> SerializableConnection {
        SerializableConnection {
            entity_id: entity_id_format(entity_id),
            sensor_id: 2,     // MouseDrag
            controller_id: 0, // Direct
            actuator_id: 2,   // Move
            priority: 0,
            enabled: true,
        }
    }
}

/// Helper function to format EntityId as string (since EntityId doesn't implement Display)
fn entity_id_format(entity_id: EntityId) -> String {
    format!("{}:{}", entity_id.index().0, entity_id.generation().0)
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serializable_wiring_new() {
        let wiring = SerializableWiring::new();
        assert!(wiring.is_empty());
        assert_eq!(wiring.connection_count(), 0);
    }

    #[test]
    fn test_serializable_wiring_default() {
        let wiring = SerializableWiring::default();
        assert!(wiring.is_empty());
    }

    #[test]
    fn test_add_sensor() {
        let mut wiring = SerializableWiring::new();
        wiring.add_sensor(SerializableSensor {
            id: 0,
            type_: "MouseOver".into(),
            name: "Test Sensor".into(),
            config: serde_json::json!({}),
        });
        assert_eq!(wiring.sensors.len(), 1);
    }

    #[test]
    fn test_add_connection() {
        let mut wiring = SerializableWiring::new();
        wiring.add_connection(SerializableConnection {
            entity_id: "entity:1:0".into(),
            sensor_id: 0,
            controller_id: 0,
            actuator_id: 0,
            priority: 0,
            enabled: true,
        });
        assert_eq!(wiring.connection_count(), 1);
    }

    #[test]
    fn test_connections_for_entity() {
        let mut wiring = SerializableWiring::new();

        // Add connections for two different entities
        wiring.add_connection(SerializableConnection {
            entity_id: "entity:1:0".into(),
            sensor_id: 0,
            controller_id: 0,
            actuator_id: 0,
            priority: 0,
            enabled: true,
        });

        wiring.add_connection(SerializableConnection {
            entity_id: "entity:1:0".into(),
            sensor_id: 1,
            controller_id: 0,
            actuator_id: 1,
            priority: 0,
            enabled: true,
        });

        wiring.add_connection(SerializableConnection {
            entity_id: "entity:2:0".into(),
            sensor_id: 0,
            controller_id: 0,
            actuator_id: 0,
            priority: 0,
            enabled: true,
        });

        let connections = wiring.connections_for_entity("entity:1:0");
        assert_eq!(connections.len(), 2);
    }

    #[test]
    fn test_remove_entity_connections() {
        let mut wiring = SerializableWiring::new();

        wiring.add_connection(SerializableConnection {
            entity_id: "entity:1:0".into(),
            sensor_id: 0,
            controller_id: 0,
            actuator_id: 0,
            priority: 0,
            enabled: true,
        });

        wiring.add_connection(SerializableConnection {
            entity_id: "entity:2:0".into(),
            sensor_id: 0,
            controller_id: 0,
            actuator_id: 0,
            priority: 0,
            enabled: true,
        });

        wiring.remove_entity_connections("entity:1:0");
        assert_eq!(wiring.connection_count(), 1);
        assert_eq!(wiring.connections[0].entity_id, "entity:2:0");
    }

    #[test]
    fn test_serialize_to_json() {
        let wiring = SerializableWiring::new();
        let json = LogicWiringSerializer::to_json(&wiring).unwrap();

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["sensors"].is_array());
        assert!(parsed["controllers"].is_array());
        assert!(parsed["actuators"].is_array());
        assert!(parsed["connections"].is_array());
    }

    #[test]
    fn test_deserialize_from_json() {
        let json = r#"{"sensors":[],"controllers":[],"actuators":[],"connections":[]}"#;
        let wiring = LogicWiringSerializer::from_json(json).unwrap();

        assert!(wiring.is_empty());
    }

    #[test]
    fn test_serialize_round_trip() {
        let mut wiring = SerializableWiring::new();

        wiring.add_sensor(SerializableSensor {
            id: 0,
            type_: "MouseOver".into(),
            name: "Test".into(),
            config: serde_json::json!({"threshold": 0.5}),
        });

        wiring.add_connection(SerializableConnection {
            entity_id: "entity:1:0".into(),
            sensor_id: 0,
            controller_id: 0,
            actuator_id: 0,
            priority: 0,
            enabled: true,
        });

        let json = LogicWiringSerializer::to_json(&wiring).unwrap();
        let wiring2 = LogicWiringSerializer::from_json(&json).unwrap();

        assert_eq!(wiring2.sensors.len(), 1);
        assert_eq!(wiring2.sensors[0].type_, "MouseOver");
        assert_eq!(wiring2.connections.len(), 1);
    }

    #[test]
    fn test_create_highlight_connection() {
        let entity = EntityId::new(1);
        let conn = LogicWiringSerializer::create_highlight_connection(entity);

        assert_eq!(conn.entity_id, "1:0"); // entity_id_format returns "index:generation"
        assert_eq!(conn.sensor_id, 0);
        assert_eq!(conn.controller_id, 0);
        assert_eq!(conn.actuator_id, 0);
        assert!(conn.enabled);
    }

    #[test]
    fn test_create_select_connection() {
        let entity = EntityId::new(1);
        let conn = LogicWiringSerializer::create_select_connection(entity);

        assert_eq!(conn.entity_id, "1:0"); // entity_id_format returns "index:generation"
        assert_eq!(conn.sensor_id, 1);
        assert_eq!(conn.actuator_id, 1);
    }

    #[test]
    fn test_create_drag_connection() {
        let entity = EntityId::new(1);
        let conn = LogicWiringSerializer::create_drag_connection(entity);

        assert_eq!(conn.entity_id, "1:0"); // entity_id_format returns "index:generation"
        assert_eq!(conn.sensor_id, 2);
        assert_eq!(conn.actuator_id, 2);
    }

    #[test]
    fn test_invalid_json() {
        let json = "not valid json";
        let result = LogicWiringSerializer::from_json(json);
        assert!(result.is_err());
    }
}
