// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Sensor Type
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
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
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

    /// Touch sensor - detects AABB collision between entities
    Touch = 4,

    /// Radar sensor - detects entities in a directional cone
    Radar = 5,

    /// DoubleTap sensor - detects rapid double-click pattern
    DoubleTap = 6,

    /// LongPress sensor - detects mouse button held down
    LongPress = 7,

    /// RightClick sensor - detects right mouse button click
    RightClick = 8,
    /// Always sensor - constantly active every frame
    Always = 9,
    /// Property sensor - detects changes in entity properties
    Property = 10,
    /// Ray sensor - line of sight detection
    Ray = 11,
    /// Timer sensor - activates after a delay
    Timer = 12,
    /// Channel sensor - listens for messages on a channel
    Channel = 13,
}

impl SensorType {
    /// Returns the index value for this sensor type
    #[inline(always)]
    #[must_use]
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// Returns the SensorType from a u8 index
    #[inline(always)]
    #[must_use]
    pub fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::MouseOver),
            1 => Some(Self::MouseClick),
            2 => Some(Self::Proximity),
            3 => Some(Self::KeyShortcut),
            4 => Some(Self::Touch),
            5 => Some(Self::Radar),
            6 => Some(Self::DoubleTap),
            7 => Some(Self::LongPress),
            8 => Some(Self::RightClick),
            9 => Some(Self::Always),
            10 => Some(Self::Property),
            11 => Some(Self::Ray),
            12 => Some(Self::Timer),
            13 => Some(Self::Channel),
            _ => None,
        }
    }
}
