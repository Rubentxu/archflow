// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Sensor Type
//
// Epic 4: Logic Mapping Table
//
// This module defines the SensorType enum for type-safe sensor identification
// in LogicMapping connections.
//
// ═══════════════════════════════════════════════════════════════════════════════

/// Types of sensors that can be connected to actuators
///
/// This enum provides type-safe identification of sensors when creating
/// connections in the LogicMappingTable.
///
/// # Examples
///
/// ```
/// use archflow_logic::SensorType;
///
/// let sensor = SensorType::MouseOver;
/// ```
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SensorType {
    /// MouseOver sensor - detects when mouse is over an entity
    MouseOver = 0,

    /// MouseClick sensor - detects mouse button clicks
    MouseClick = 1,

    /// Proximity sensor - detects entities within a radius
    Proximity = 2,

    /// KeyShortcut sensor - detects keyboard shortcuts
    KeyShortcut = 3,
}

impl SensorType {
    /// Returns the index value for this sensor type
    #[inline(always)]
    #[must_use]
    pub const fn index(self) -> u8 {
        self as u8
    }
}
