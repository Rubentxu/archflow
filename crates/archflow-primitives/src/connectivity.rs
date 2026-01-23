//! Ports & Connections - Sistema de conectividad entre nodos

use crate::{EntityId, Vec2};
use serde::{Deserialize, Serialize};

/// Tipo de dato que fluye por un puerto
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PortType {
    Any,
    Number,
    String,
    Boolean,
    Array,
    Object,
    Event,
    Color,
    Image,
    Pose,
    Geometry,
    Custom(String),
}

/// Dirección del flujo en el puerto
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortDirection {
    Input,
    Output,
    Bidirectional,
}

/// Puerto de conexión
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Port {
    pub id: EntityId,
    pub node_id: EntityId,
    pub name: String,
    pub port_type: PortType,
    pub direction: PortDirection,
    pub position: Vec2,
    pub radius: f32,
    pub connections: Vec<EntityId>,
    pub capacity: u32,
    pub enabled: bool,
}

impl Port {
    pub fn new(node_id: EntityId, name: &str, direction: PortDirection, position: Vec2) -> Self {
        Self {
            id: EntityId::new(),
            node_id,
            name: name.to_string(),
            port_type: PortType::Any,
            direction,
            position,
            radius: 8.0,
            connections: Vec::new(),
            capacity: 0,
            enabled: true,
        }
    }
    pub fn with_type(mut self, port_type: PortType) -> Self {
        self.port_type = port_type;
        self
    }
    pub fn with_capacity(mut self, capacity: u32) -> Self {
        self.capacity = capacity;
        self
    }
    pub fn is_connected(&self) -> bool {
        !self.connections.is_empty()
    }
    pub fn can_connect(&self) -> bool {
        self.enabled && (self.capacity == 0 || self.connections.len() < self.capacity as usize)
    }
    pub fn add_connection(&mut self, connection_id: EntityId) -> bool {
        if self.can_connect() {
            self.connections.push(connection_id);
            true
        } else {
            false
        }
    }
    pub fn remove_connection(&mut self, connection_id: &EntityId) {
        self.connections.retain(|id| id != connection_id);
    }
}

/// Tipo de conexión
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionType {
    Flow,
    Bidirectional,
    Event,
    Stream,
    Reference,
}

/// Estado de la conexión
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    Active,
    Inactive,
    Error,
    Disconnected,
}

/// Conexión entre dos puertos
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Connection {
    pub id: EntityId,
    pub source_port: EntityId,
    pub target_port: EntityId,
    pub connection_type: ConnectionType,
    pub state: ConnectionState,
    pub points: Vec<Vec2>,
    pub routing_type: RoutingType,
    pub stroke_color: String,
    pub stroke_width: f32,
    pub show_arrow: bool,
}

impl Connection {
    pub fn new(source_port: EntityId, target_port: EntityId) -> Self {
        Self {
            id: EntityId::new(),
            source_port,
            target_port,
            connection_type: ConnectionType::Flow,
            state: ConnectionState::Inactive,
            points: Vec::new(),
            routing_type: RoutingType::default(),
            stroke_color: "#666666".to_string(),
            stroke_width: 2.0,
            show_arrow: true,
        }
    }
    pub fn with_type(mut self, connection_type: ConnectionType) -> Self {
        self.connection_type = connection_type;
        self
    }
    pub fn is_valid(&self) -> bool {
        self.source_port != self.target_port
    }
    pub fn is_active(&self) -> bool {
        self.state == ConnectionState::Active
    }
}

/// Tipo de routing visual
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RoutingType {
    Straight,
    Orthogonal { corner_radius: f32 },
    Curved { curvature: f32 },
    Spline,
    Smart,
}

impl Default for RoutingType {
    fn default() -> Self {
        RoutingType::Orthogonal {
            corner_radius: 10.0,
        }
    }
}

impl RoutingType {
    pub fn orthogonal_path(start: Vec2, end: Vec2, corner_radius: f32) -> Vec<Vec2> {
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let horizontal = dx.abs() > dy.abs();
        let mid1 = if horizontal {
            Vec2::new(start.x + dx / 2.0, start.y)
        } else {
            Vec2::new(start.x, start.y + dy / 2.0)
        };
        let mid2 = if horizontal {
            Vec2::new(start.x + dx / 2.0, end.y)
        } else {
            Vec2::new(end.x, start.y + dy / 2.0)
        };
        vec![start, mid1, mid2, end]
    }
    pub fn curved_path(start: Vec2, end: Vec2, curvature: f32) -> Vec<Vec2> {
        let dx = end.x - start.x;
        let control_offset = dx.abs() * curvature;
        vec![
            start,
            Vec2::new(start.x + control_offset, start.y),
            Vec2::new(end.x - control_offset, end.y),
            end,
        ]
    }
}

/// Colección de puertos
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortCollection {
    pub node_id: EntityId,
    pub ports: Vec<Port>,
}

impl PortCollection {
    pub fn new(node_id: EntityId) -> Self {
        Self {
            node_id,
            ports: Vec::new(),
        }
    }
    pub fn add_port(&mut self, port: Port) {
        self.ports.push(port);
    }
    pub fn remove_port(&mut self, port_id: EntityId) -> Option<Port> {
        self.ports
            .iter()
            .position(|p| p.id == port_id)
            .map(|i| self.ports.remove(i))
    }
    pub fn get_port(&self, port_id: EntityId) -> Option<&Port> {
        self.ports.iter().find(|p| p.id == port_id)
    }
    pub fn get_port_mut(&mut self, port_id: EntityId) -> Option<&mut Port> {
        self.ports.iter_mut().find(|p| p.id == port_id)
    }
}

/// Gestor de conexiones
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionManager {
    pub connections: Vec<Connection>,
    pub port_collections: Vec<PortCollection>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
            port_collections: Vec::new(),
        }
    }
    pub fn add_port(&mut self, node_id: EntityId, port: Port) {
        if let Some(collection) = self
            .port_collections
            .iter_mut()
            .find(|c| c.node_id == node_id)
        {
            collection.add_port(port);
        } else {
            let mut collection = PortCollection::new(node_id);
            collection.add_port(port);
            self.port_collections.push(collection);
        }
    }
    pub fn connect(&mut self, source_port: EntityId, target_port: EntityId) -> Option<Connection> {
        if source_port == target_port {
            return None;
        }
        let (source_coll_idx, target_coll_idx) =
            self.find_port_indices(source_port, target_port)?;

        // Check capacity first using immutable borrow
        let source_can = self
            .get_port(source_coll_idx, source_port)
            .map(|p| p.can_connect())
            .unwrap_or(false);
        let target_can = self
            .get_port(target_coll_idx, target_port)
            .map(|p| p.can_connect())
            .unwrap_or(false);

        if !source_can || !target_can {
            return None;
        }

        let connection_id = EntityId::new();

        // Add connection to both ports using separate borrows
        if let Some(source) = self.get_port_mut(source_coll_idx, source_port) {
            source.add_connection(connection_id);
        }
        if let Some(target) = self.get_port_mut(target_coll_idx, target_port) {
            target.add_connection(connection_id);
        }

        let connection = Connection {
            id: connection_id,
            source_port,
            target_port,
            connection_type: ConnectionType::Flow,
            state: ConnectionState::Inactive,
            points: Vec::new(),
            routing_type: RoutingType::default(),
            stroke_color: "#666666".to_string(),
            stroke_width: 2.0,
            show_arrow: true,
        };

        self.connections.push(connection.clone());
        Some(connection)
    }
    fn get_port(&self, coll_idx: usize, port_id: EntityId) -> Option<&Port> {
        self.port_collections[coll_idx]
            .ports
            .iter()
            .find(|p| p.id == port_id)
    }
    pub fn disconnect(&mut self, connection_id: EntityId) -> Option<Connection> {
        if let Some(pos) = self.connections.iter().position(|c| c.id == connection_id) {
            let connection = self.connections.remove(pos);
            for coll in &mut self.port_collections {
                if let Some(port) = coll.get_port_mut(connection.source_port) {
                    port.remove_connection(&connection_id);
                }
                if let Some(port) = coll.get_port_mut(connection.target_port) {
                    port.remove_connection(&connection_id);
                }
            }
            Some(connection)
        } else {
            None
        }
    }
    fn find_port_indices(&self, id1: EntityId, id2: EntityId) -> Option<(usize, usize)> {
        for (i, coll) in self.port_collections.iter().enumerate() {
            for port in &coll.ports {
                if port.id == id1 {
                    for (j, coll2) in self.port_collections.iter().enumerate() {
                        for port2 in &coll2.ports {
                            if port2.id == id2 {
                                return Some((i, j));
                            }
                        }
                    }
                }
            }
        }
        None
    }
    fn get_port_mut(&mut self, coll_idx: usize, port_id: EntityId) -> Option<&mut Port> {
        self.port_collections[coll_idx]
            .ports
            .iter_mut()
            .find(|p| p.id == port_id)
    }
    pub fn get_connections_for_port(&self, port_id: EntityId) -> Vec<&Connection> {
        self.connections
            .iter()
            .filter(|c| c.source_port == port_id || c.target_port == port_id)
            .collect()
    }
}
