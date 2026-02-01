// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow SDK - Wiring System API
//
// This module provides the WiringBuilder for declarative configuration
// of connections between sensors and actuators.
//
// Reference: docs/epics/EPIC-SDK-PUBLIC-API.md - Section "API de Wiring"
// ═══════════════════════════════════════════════════════════════════════════════

use crate::sensors::SensorState;
use archflow_core::EntityId;
use std::vec::Vec;

/// Entity filter for connection targeting
///
/// Controls which entities a connection applies to.
#[derive(Clone, Debug)]
pub enum EntityFilter {
    /// Apply to all entities
    All,

    /// Apply only to entities with a specific tag
    Tag(String),

    /// Apply only to entities in a specific layer
    Layer(u8),

    /// Apply only to a specific entity
    Specific(EntityId),
}

/// A single connection between a sensor and an actuator
#[derive(Clone, Debug)]
pub struct Connection {
    /// ID of the sensor that emits pulses
    pub sensor_id: u32,

    /// ID of the actuator to trigger
    pub actuator_id: u32,

    /// Optional filter for which entities this applies to
    pub entity_filter: Option<EntityFilter>,

    /// Optional filter for which sensor states trigger
    pub state_filter: Option<SensorState>,
}

impl Connection {
    /// Create a new connection
    pub fn new(sensor_id: u32, actuator_id: u32) -> Self {
        Self {
            sensor_id,
            actuator_id,
            entity_filter: None,
            state_filter: None,
        }
    }

    /// Add an entity filter to this connection
    pub fn with_entity_filter(mut self, filter: EntityFilter) -> Self {
        self.entity_filter = Some(filter);
        self
    }

    /// Add a state filter to this connection
    pub fn with_state_filter(mut self, state: SensorState) -> Self {
        self.state_filter = Some(state);
        self
    }
}

/// Builder for creating wiring configurations
///
/// This provides a fluent, declarative API for connecting sensors
/// to actuators with optional filters.
///
/// # Example
///
/// ```rust
/// use archflow_sdk::wiring::WiringBuilder;
/// use archflow_sdk::sensors::SensorState;
///
/// let wiring = WiringBuilder::new()
///     .connect(0, 10)  // Connect sensor 0 to actuator 10
///     .on_entities_with_tag("button")
///     .on_positive()
///     .connect(1, 11)  // Connect sensor 1 to actuator 11
///     .on_entities_with_tag("slider")
///     .build();
/// ```
pub struct WiringBuilder {
    connections: Vec<Connection>,
}

impl Default for WiringBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl WiringBuilder {
    /// Create a new wiring builder
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
        }
    }

    /// Connect a sensor to an actuator
    ///
    /// This creates a connection that can be further configured
    /// with filters before building the final wiring table.
    pub fn connect(mut self, sensor_id: u32, actuator_id: u32) -> Self {
        self.connections
            .push(Connection::new(sensor_id, actuator_id));
        self
    }

    /// Filter to entities with a specific tag
    ///
    /// This applies to the LAST connection added.
    pub fn on_entities_with_tag(mut self, tag: &str) -> Self {
        if let Some(conn) = self.connections.last_mut() {
            conn.entity_filter = Some(EntityFilter::Tag(tag.to_string()));
        }
        self
    }

    /// Filter to entities in a specific layer
    ///
    /// This applies to the LAST connection added.
    pub fn on_entities_in_layer(mut self, layer: u8) -> Self {
        if let Some(conn) = self.connections.last_mut() {
            conn.entity_filter = Some(EntityFilter::Layer(layer));
        }
        self
    }

    /// Filter to a specific entity
    ///
    /// This applies to the LAST connection added.
    pub fn on_entity(mut self, entity_id: EntityId) -> Self {
        if let Some(conn) = self.connections.last_mut() {
            conn.entity_filter = Some(EntityFilter::Specific(entity_id));
        }
        self
    }

    /// Filter to Positive sensor state only
    ///
    /// This applies to the LAST connection added.
    pub fn on_positive(mut self) -> Self {
        if let Some(conn) = self.connections.last_mut() {
            conn.state_filter = Some(SensorState::Positive);
        }
        self
    }

    /// Filter to Negative sensor state only
    ///
    /// This applies to the LAST connection added.
    pub fn on_negative(mut self) -> Self {
        if let Some(conn) = self.connections.last_mut() {
            conn.state_filter = Some(SensorState::Negative);
        }
        self
    }

    /// Build the wiring table
    ///
    /// This finalizes the configuration and returns a WiringTable
    /// that can be used by the engine.
    pub fn build(self) -> WiringTable {
        WiringTable {
            connections: self.connections,
        }
    }
}

/// A complete wiring table
///
/// This contains all connections between sensors and actuators
/// and is used by the engine to route pulses.
#[derive(Clone, Debug)]
pub struct WiringTable {
    connections: Vec<Connection>,
}

impl WiringTable {
    /// Get all connections in the table
    pub fn connections(&self) -> &[Connection] {
        &self.connections
    }

    /// Find all actuators connected to a specific sensor
    pub fn find_actuators_for_sensor(&self, sensor_id: u32) -> Vec<u32> {
        self.connections
            .iter()
            .filter(|c| c.sensor_id == sensor_id)
            .map(|c| c.actuator_id)
            .collect()
    }

    /// Check if a connection's entity filter matches an entity
    pub fn entity_matches_filter(
        &self,
        entity_id: EntityId,
        connection: &Connection,
        store: &archflow_engine::EntityStore,
    ) -> bool {
        match &connection.entity_filter {
            None => true,
            Some(EntityFilter::All) => true,
            Some(EntityFilter::Specific(id)) => *id == entity_id,
            Some(EntityFilter::Tag(_tag)) => {
                // TODO: Implement tag checking when EntityStore has tags
                true
            }
            Some(EntityFilter::Layer(layer)) => {
                // TODO: Implement layer checking when available
                // store.layer() requires usize index, not EntityId
                store.layer(entity_id.index().0 as usize) == *layer
            }
        }
    }

    /// Check if a connection's state filter matches a pulse state
    pub fn state_matches_filter(connection: &Connection, state: SensorState) -> bool {
        match connection.state_filter {
            None => true,
            Some(filter_state) => state == filter_state,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_core::EntityId;

    #[test]
    fn test_wiring_builder_basic() {
        let wiring = WiringBuilder::new().connect(0, 10).connect(1, 11).build();

        assert_eq!(wiring.connections().len(), 2);
    }

    #[test]
    fn test_wiring_builder_with_filters() {
        let wiring = WiringBuilder::new()
            .connect(0, 10)
            .on_entities_with_tag("button")
            .on_positive()
            .build();

        let conn = &wiring.connections()[0];
        assert!(conn.entity_filter.is_some());
        assert!(conn.state_filter.is_some());
    }

    #[test]
    fn test_connection_new() {
        let conn = Connection::new(5, 15);
        assert_eq!(conn.sensor_id, 5);
        assert_eq!(conn.actuator_id, 15);
        assert!(conn.entity_filter.is_none());
        assert!(conn.state_filter.is_none());
    }

    #[test]
    fn test_connection_with_filters() {
        let conn = Connection::new(5, 15)
            .with_entity_filter(EntityFilter::All)
            .with_state_filter(SensorState::Positive);

        assert!(conn.entity_filter.is_some());
        assert!(conn.state_filter.is_some());
    }

    #[test]
    fn test_find_actuators_for_sensor() {
        let wiring = WiringBuilder::new()
            .connect(0, 10)
            .connect(0, 11)
            .connect(0, 12)
            .connect(1, 13)
            .build();

        let actuators = wiring.find_actuators_for_sensor(0);
        assert_eq!(actuators.len(), 3);
        assert!(actuators.contains(&10));
        assert!(actuators.contains(&11));
        assert!(actuators.contains(&12));
        assert!(!actuators.contains(&13));
    }

    #[test]
    fn test_state_matches_filter() {
        let conn = Connection::new(0, 1).with_state_filter(SensorState::Positive);

        assert!(WiringTable::state_matches_filter(
            &conn,
            SensorState::Positive
        ));
        assert!(!WiringTable::state_matches_filter(
            &conn,
            SensorState::Negative
        ));
        assert!(!WiringTable::state_matches_filter(&conn, SensorState::None));
    }

    #[test]
    fn test_state_matches_filter_no_filter() {
        let conn = Connection::new(0, 1);

        // With no filter, all states match
        assert!(WiringTable::state_matches_filter(
            &conn,
            SensorState::Positive
        ));
        assert!(WiringTable::state_matches_filter(
            &conn,
            SensorState::Negative
        ));
        assert!(WiringTable::state_matches_filter(&conn, SensorState::None));
    }
}
