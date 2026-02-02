// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Physics Pulse System (HU-010 Gap Fix)
//
// This module extends the basic Pulse system with physics-specific metadata
// for collision, proximity, and radar sensors.
//
// Reference: docs/epics/EPIC-002-physics-sensors.md - HU-010
//
// Gap Fixed:
// - Pulse now includes `other_entity` for collision metadata
// - Support for hit lists (multiple simultaneous collisions)
// - Physics-specific pulse types (Collision, Proximity, Radar)
// ═══════════════════════════════════════════════════════════════════════════════

#![no_std]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use crate::pulse::{Pulse, PulseBus, SensorState};

/// Physics-specific pulse metadata
///
/// This extends the basic Pulse with information about what triggered
/// the pulse in a physics context.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PhysicsMetadata {
    /// No additional metadata (simple sensor)
    None,

    /// Collision sensor: which entity we collided with
    Collision {
        /// The other entity involved in the collision (index as u32)
        other_entity: u32,
    },

    /// Proximity sensor: distance to nearest entity
    Proximity {
        /// Distance to the nearest entity
        distance: f32,
        /// Entity that triggered the proximity (index as u32)
        nearest_entity: u32,
    },

    /// Radar sensor: detected entity in cone
    Radar {
        /// Detected entity (index as u32)
        entity: u32,
        /// Angle from radar center (in radians)
        angle: f32,
        /// Distance to detected entity
        distance: f32,
    },
}

/// Physics pulse with metadata for collision/proximity/radar sensors
///
/// This extends the basic Pulse with physics-specific information,
/// allowing actuators to know exactly what triggered the pulse.
///
/// # Memory Layout
/// ```text
/// Total: 24 bytes (8 bytes metadata + 16 bytes Pulse)
/// - sensor_id: u32 (4 bytes)
/// - entity_id: u32 (4 bytes)
/// - state: SensorState (1 byte, padded to 4)
/// - timestamp: u32 (4 bytes)
/// - metadata: PhysicsMetadata (8 bytes for union)
/// ```
///
/// # Example
///
/// ```
/// let physics_pulse = PhysicsPulse::collision(
///     sensor_id,
///     entity_id,
///     other_entity_id,
///     timestamp
/// );
///
/// if physics_pulse.is_collision() {
///     let other = physics_pulse.other_entity();
///     // Handle collision with other entity
/// }
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PhysicsPulse {
    /// Base pulse information
    pulse: Pulse,

    /// Physics-specific metadata
    metadata: PhysicsMetadata,
}

impl PhysicsPulse {
    /// Create a physics pulse from a base pulse with collision metadata
    #[inline(always)]
    #[must_use]
    pub fn collision(sensor_id: u32, entity_id: u32, other_entity: u32, timestamp: u32) -> Self {
        Self {
            pulse: Pulse::positive(sensor_id, entity_id, timestamp),
            metadata: PhysicsMetadata::Collision { other_entity },
        }
    }

    /// Create a physics pulse from a base pulse with proximity metadata
    #[inline(always)]
    #[must_use]
    pub fn proximity(
        sensor_id: u32,
        entity_id: u32,
        distance: f32,
        nearest_entity: u32,
        timestamp: u32,
    ) -> Self {
        Self {
            pulse: Pulse::positive(sensor_id, entity_id, timestamp),
            metadata: PhysicsMetadata::Proximity {
                distance,
                nearest_entity,
            },
        }
    }

    /// Create a physics pulse from a base pulse with radar metadata
    #[inline(always)]
    #[must_use]
    pub fn radar(
        sensor_id: u32,
        entity_id: u32,
        detected_entity: u32,
        angle: f32,
        distance: f32,
        timestamp: u32,
    ) -> Self {
        Self {
            pulse: Pulse::positive(sensor_id, entity_id, timestamp),
            metadata: PhysicsMetadata::Radar {
                entity: detected_entity,
                angle,
                distance,
            },
        }
    }

    /// Create a physics pulse with no metadata (passthrough from simple pulse)
    #[inline(always)]
    #[must_use]
    pub fn from_pulse(pulse: Pulse) -> Self {
        Self {
            pulse,
            metadata: PhysicsMetadata::None,
        }
    }

    /// Get the base pulse
    #[inline(always)]
    #[must_use]
    pub const fn pulse(&self) -> Pulse {
        self.pulse
    }

    /// Get the sensor ID
    #[inline(always)]
    #[must_use]
    pub const fn sensor_id(&self) -> u32 {
        self.pulse.sensor_id
    }

    /// Get the entity ID (raw u32 index)
    #[inline(always)]
    #[must_use]
    pub const fn entity_id(&self) -> u32 {
        self.pulse.entity_id
    }

    /// Get the sensor state
    #[inline(always)]
    #[must_use]
    pub const fn state(&self) -> SensorState {
        self.pulse.state
    }

    /// Get the timestamp
    #[inline(always)]
    #[must_use]
    pub const fn timestamp(&self) -> u32 {
        self.pulse.timestamp
    }

    /// Check if this is a positive pulse
    #[inline(always)]
    #[must_use]
    pub const fn is_positive(&self) -> bool {
        self.pulse.is_positive()
    }

    /// Check if this is a negative pulse
    #[inline(always)]
    #[must_use]
    pub const fn is_negative(&self) -> bool {
        self.pulse.is_negative()
    }

    /// Check if this pulse has collision metadata
    #[inline(always)]
    #[must_use]
    pub fn is_collision(&self) -> bool {
        matches!(self.metadata, PhysicsMetadata::Collision { .. })
    }

    /// Get the other entity in a collision (only valid if is_collision())
    #[inline(always)]
    #[must_use]
    pub fn other_entity(&self) -> u32 {
        match self.metadata {
            PhysicsMetadata::Collision { other_entity } => other_entity,
            _ => u32::MAX,
        }
    }

    /// Check if this pulse has proximity metadata
    #[inline(always)]
    #[must_use]
    pub fn is_proximity(&self) -> bool {
        matches!(self.metadata, PhysicsMetadata::Proximity { .. })
    }

    /// Get the distance to nearest entity (only valid if is_proximity())
    #[inline(always)]
    #[must_use]
    pub fn distance(&self) -> f32 {
        match self.metadata {
            PhysicsMetadata::Proximity { distance, .. } => distance,
            PhysicsMetadata::Radar { distance, .. } => distance,
            _ => f32::INFINITY,
        }
    }

    /// Check if this pulse has radar metadata
    #[inline(always)]
    #[must_use]
    pub fn is_radar(&self) -> bool {
        matches!(self.metadata, PhysicsMetadata::Radar { .. })
    }

    /// Get the detected entity for radar (only valid if is_radar())
    #[inline(always)]
    #[must_use]
    pub fn detected_entity(&self) -> u32 {
        match self.metadata {
            PhysicsMetadata::Radar { entity, .. } => entity,
            PhysicsMetadata::Collision { other_entity } => other_entity,
            PhysicsMetadata::Proximity { nearest_entity, .. } => nearest_entity,
            PhysicsMetadata::None => u32::MAX,
        }
    }

    /// Get the angle for radar detection (only valid if is_radar())
    #[inline(always)]
    #[must_use]
    pub fn angle(&self) -> f32 {
        match self.metadata {
            PhysicsMetadata::Radar { angle, .. } => angle,
            _ => 0.0,
        }
    }
}

/// Extended PulseBus with physics-specific functionality
///
/// This extends the basic PulseBus with methods for physics sensors
/// to push pulses with metadata.
pub struct PhysicsPulseBus {
    /// Base pulse bus
    base: PulseBus,

    /// Physics pulses with metadata (separate buffer for rich pulses)
    physics_pulses: Vec<PhysicsPulse>,
}

impl PhysicsPulseBus {
    /// Create a new physics pulse bus
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            base: PulseBus::new(),
            physics_pulses: Vec::with_capacity(256),
        }
    }

    /// Set the current timestamp (propagates to base bus)
    #[inline(always)]
    pub fn set_timestamp(&mut self, timestamp: u32) {
        self.base.set_timestamp(timestamp);
    }

    /// Push a base pulse (no physics metadata)
    #[inline(always)]
    pub fn push(&mut self, pulse: Pulse) {
        self.base.push(pulse);
    }

    /// Push a physics pulse with collision metadata
    #[inline(always)]
    pub fn push_collision(&mut self, sensor_id: u32, entity_id: u32, other_entity: u32) {
        self.physics_pulses.push(PhysicsPulse::collision(
            sensor_id,
            entity_id,
            other_entity,
            self.base.timestamp(),
        ));
    }

    /// Push a physics pulse with proximity metadata
    #[inline(always)]
    pub fn push_proximity(
        &mut self,
        sensor_id: u32,
        entity_id: u32,
        distance: f32,
        nearest_entity: u32,
    ) {
        self.physics_pulses.push(PhysicsPulse::proximity(
            sensor_id,
            entity_id,
            distance,
            nearest_entity,
            self.base.timestamp(),
        ));
    }

    /// Push a physics pulse with radar metadata
    #[inline(always)]
    pub fn push_radar(
        &mut self,
        sensor_id: u32,
        entity_id: u32,
        detected_entity: u32,
        angle: f32,
        distance: f32,
    ) {
        self.physics_pulses.push(PhysicsPulse::radar(
            sensor_id,
            entity_id,
            detected_entity,
            angle,
            distance,
            self.base.timestamp(),
        ));
    }

    /// Drain all base pulses (no metadata)
    #[inline(always)]
    pub fn drain_pulses(&mut self) -> Vec<Pulse> {
        self.base.drain()
    }

    /// Drain all physics pulses (with metadata)
    #[inline(always)]
    pub fn drain_physics(&mut self) -> Vec<PhysicsPulse> {
        core::mem::take(&mut self.physics_pulses)
    }

    /// Drain all pulses (base + physics)
    #[inline(always)]
    pub fn drain_all(&mut self) -> (Vec<Pulse>, Vec<PhysicsPulse>) {
        (self.drain_pulses(), self.drain_physics())
    }

    /// Clear all pulses
    #[inline(always)]
    pub fn clear(&mut self) {
        self.base.clear();
        self.physics_pulses.clear();
    }

    /// Returns true if there are any pulses
    #[inline(always)]
    #[must_use]
    pub fn has_pulses(&self) -> bool {
        self.base.has_pulses() || !self.physics_pulses.is_empty()
    }

    /// Returns the total number of pulses
    #[inline(always)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.base.len() + self.physics_pulses.len()
    }
}

impl Default for PhysicsPulseBus {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═════════════════════════════════════════════════════════════════════════════==

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_pulse_creation() {
        let pulse = PhysicsPulse::collision(1, 10, 20, 1000);
        assert_eq!(pulse.sensor_id(), 1);
        assert_eq!(pulse.entity_id(), 10);
        assert!(pulse.is_positive());
        assert!(pulse.is_collision());
        assert_eq!(pulse.other_entity(), 20);
    }

    #[test]
    fn test_proximity_pulse() {
        let pulse = PhysicsPulse::proximity(2, 10, 25.0, 20, 1000);
        assert!(pulse.is_proximity());
        assert!((pulse.distance() - 25.0).abs() < 0.001);
    }

    #[test]
    fn test_radar_pulse() {
        let pulse = PhysicsPulse::radar(3, 10, 20, 0.5, 50.0, 1000);
        assert!(pulse.is_radar());
        assert!((pulse.angle() - 0.5).abs() < 0.001);
        assert!((pulse.distance() - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_from_pulse() {
        let base = Pulse::positive(5, 10, 1000);
        let physics = PhysicsPulse::from_pulse(base);
        assert_eq!(physics.sensor_id(), 5);
        assert!(physics.is_positive());
    }

    #[test]
    fn test_physics_pulse_bus() {
        let mut bus = PhysicsPulseBus::new();
        bus.set_timestamp(1000);

        bus.push_collision(1, 10, 20);
        bus.push_proximity(2, 10, 25.0, 30);
        bus.push(Pulse::positive(3, 40, 1000));

        assert!(bus.has_pulses());
        assert_eq!(bus.len(), 3);

        let (pulses, physics) = bus.drain_all();
        assert_eq!(pulses.len(), 1);
        assert_eq!(physics.len(), 2);

        assert!(!bus.has_pulses());
    }

    #[test]
    fn test_physics_pulse_negative_state() {
        let pulse = PhysicsPulse::collision(1, 10, 20, 1000);
        // Create negative version manually
        let negative = PhysicsPulse {
            pulse: Pulse::negative(1, 10, 1000),
            metadata: PhysicsMetadata::Collision { other_entity: 20 },
        };
        assert!(negative.is_negative());
    }

    #[test]
    fn test_invalid_entity_returns_max() {
        let pulse = PhysicsPulse::from_pulse(Pulse::positive(1, 10, 1000));
        assert_eq!(pulse.other_entity(), u32::MAX);
        assert_eq!(pulse.detected_entity(), u32::MAX);
    }
}
