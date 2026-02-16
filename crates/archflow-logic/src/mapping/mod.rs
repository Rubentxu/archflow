// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Logic Mapping Module
//
// This module provides the connection layer between sensors and actuators:
// - LogicMappingTable: Stores and evaluates sensor→actuator connections
// - Controller: Boolean logic (AND, OR, NOT) for combining sensor signals
// - Type-safe connections using enums
// ═══════════════════════════════════════════════════════════════════════════════

pub mod controller;
pub mod link_type;
pub mod mapping_table;
pub mod sensor_type;

pub use controller::Controller;
pub use link_type::{ChannelScope, LinkType};
pub use mapping_table::{ActuatorType, LogicMappingTable};
pub use sensor_type::SensorType;
