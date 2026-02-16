// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Link Type
//
// This module defines the LinkType enum for different connection behaviors
// between sensors and actuators, and the ChannelScope for message-based
// communication in the LogicMapping system.
//
// ═══════════════════════════════════════════════════════════════════════════════

/// Types of links/connections between sensors and actuators
///
/// LinkTypes define how the output of a sensor or controller is transmitted
/// to an actuator. Different link types provide different behaviors:
///
/// | Type | Description | Use Case |
/// |------|-------------|----------|
/// | `Direct` | Pass-through signal, no modification | Simple triggers |
/// | `Pulse` | Single activation pulse, auto-resets | One-shot actions |
/// | `Toggle` | Toggles state on each activation | On/off switches |
/// | `Accumulate` | Accumulates signal strength over time | Progress bars, counters |
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LinkType {
    /// Direct pass-through: signal flows without modification
    Direct = 0,

    /// Pulse: single activation pulse that auto-resets
    Pulse = 1,

    /// Toggle: alternates between active/inactive on each signal
    Toggle = 2,

    /// Accumulate: accumulates signal value over time
    Accumulate = 3,
}

impl LinkType {
    /// Returns the index value for this link type
    #[inline(always)]
    #[must_use]
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// Returns the LinkType from a u8 index
    #[inline(always)]
    #[must_use]
    pub fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::Direct),
            1 => Some(Self::Pulse),
            2 => Some(Self::Toggle),
            3 => Some(Self::Accumulate),
            _ => None,
        }
    }
}

/// Channel scope for message-based communication
///
/// ChannelScope defines a unique identifier for a communication channel
/// within the game world, consisting of a level ID, room ID, and channel ID.
///
/// This enables sensors and actuators to communicate across different
/// parts of the game world through a publish-subscribe mechanism.
///
/// # Examples
///
/// ```
/// use archflow_logic::mapping::ChannelScope;
///
/// let scope = ChannelScope::new(1, 0, 42);
/// let global_id = scope.to_global_id();
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChannelScope {
    /// The level identifier (0-65535)
    pub level_id: u16,
    /// The room identifier within the level (0-255)
    pub room_id: u8,
    /// The channel identifier within the room (0-65535)
    pub channel_id: u16,
}

impl ChannelScope {
    /// Creates a new ChannelScope with the given level, room, and channel IDs
    ///
    /// # Arguments
    ///
    /// * `level_id` - The level identifier (0-65535)
    /// * `room_id` - The room identifier within the level (0-255)
    /// * `channel_id` - The channel identifier within the room (0-65535)
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_logic::mapping::ChannelScope;
    ///
    /// let scope = ChannelScope::new(1, 0, 42);
    /// ```
    #[must_use]
    pub const fn new(level_id: u16, room_id: u8, channel_id: u16) -> Self {
        Self {
            level_id,
            room_id,
            channel_id,
        }
    }

    /// Converts the channel scope to a global unique identifier
    ///
    /// The global ID is a 32-bit integer that uniquely identifies this
    /// channel scope across all levels and rooms. The format is:
    /// - Bits 0-15: channel_id
    /// - Bits 16-23: room_id
    /// - Bits 24-31: level_id (upper 8 bits of 16-bit level_id)
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_logic::mapping::ChannelScope;
    ///
    /// let scope = ChannelScope::new(1, 0, 42);
    /// let global_id = scope.to_global_id();
    /// assert_eq!(global_id, 0x0100002A);
    /// ```
    #[must_use]
    pub const fn to_global_id(self) -> u32 {
        let level_part = (self.level_id as u32) << 24;
        let room_part = (self.room_id as u32) << 16;
        let channel_part = self.channel_id as u32;
        level_part | room_part | channel_part
    }

    /// Creates a ChannelScope from a global unique identifier
    ///
    /// # Arguments
    ///
    /// * `global_id` - A 32-bit global identifier
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_logic::mapping::ChannelScope;
    ///
    /// let scope = ChannelScope::from_global_id(0x0100002A);
    /// assert_eq!(scope.level_id, 1);
    /// assert_eq!(scope.room_id, 0);
    /// assert_eq!(scope.channel_id, 42);
    /// ```
    #[must_use]
    pub const fn from_global_id(global_id: u32) -> Self {
        let level_id = (global_id >> 24) as u16;
        let room_id = ((global_id >> 16) & 0xFF) as u8;
        let channel_id = (global_id & 0xFFFF) as u16;
        Self {
            level_id,
            room_id,
            channel_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_type_index() {
        assert_eq!(LinkType::Direct.index(), 0);
        assert_eq!(LinkType::Pulse.index(), 1);
        assert_eq!(LinkType::Toggle.index(), 2);
        assert_eq!(LinkType::Accumulate.index(), 3);
    }

    #[test]
    fn test_link_type_from_index() {
        assert_eq!(LinkType::from_index(0), Some(LinkType::Direct));
        assert_eq!(LinkType::from_index(1), Some(LinkType::Pulse));
        assert_eq!(LinkType::from_index(2), Some(LinkType::Toggle));
        assert_eq!(LinkType::from_index(3), Some(LinkType::Accumulate));
        assert_eq!(LinkType::from_index(4), None);
    }

    #[test]
    fn test_channel_scope_to_global_id() {
        let scope = ChannelScope::new(1, 0, 42);
        assert_eq!(scope.to_global_id(), 0x0100002A);
    }

    #[test]
    fn test_channel_scope_from_global_id() {
        let scope = ChannelScope::from_global_id(0x0100002A);
        assert_eq!(scope.level_id, 1);
        assert_eq!(scope.room_id, 0);
        assert_eq!(scope.channel_id, 42);
    }

    #[test]
    fn test_channel_scope_roundtrip() {
        let original = ChannelScope::new(255, 127, 65535);
        let global_id = original.to_global_id();
        let restored = ChannelScope::from_global_id(global_id);
        assert_eq!(original, restored);
    }
}
