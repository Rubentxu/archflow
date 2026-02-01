// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Double Tap Sensor Implementation
//
// This sensor detects double tap/click gestures on entities.
// Uses per-entity timing tracking with 6-tick signal history.
//
// Performance Characteristics:
// - O(n) where n = number of entities (single linear scan)
// - Per-entity timing state (last tap time, tap count)
// - Cache-friendly (sequential access to EntityStore SoA)
//
// Memory Impact:
// - 1 byte per entity (SignalByte)
// - 8 bytes per entity (u64 last tap time)
// - 1 byte per entity (tap count)
// - ~1MB for 100,000 entities
//
// ═══════════════════════════════════════════════════════════════════════════════

use crate::signals::SignalByte;
use alloc::vec;
use alloc::vec::Vec;
use archflow_core::{EntityId, Vec2};
use archflow_engine::{EntityStore, MAX_ENTITIES};

/// Maximum time between taps for double tap detection (milliseconds)
pub const DOUBLE_TAP_MS: u64 = 300;

/// Maximum time to wait before resetting tap counter
pub const TAP_TIMEOUT_MS: u64 = 500;

/// Sensor that detects double tap/click gestures on entities
///
/// Tracks per-entity tap timing and maintains 6-tick signal history.
///
/// # Examples
///
/// ```
/// use archflow_logic::sensors::DoubleTapSensor;
/// use archflow_logic::sensors::mouse_click::PointerButtons;
/// use archflow_core::Vec2;
/// use archflow_engine::EntityStore;
///
/// let mut store = EntityStore::new();
/// let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
///
/// let mut sensor = DoubleTapSensor::new();
///
/// // Double tap on entity
/// let mouse_pos = Vec2::new(100.0, 100.0);
/// let buttons = PointerButtons(PointerButtons::PRIMARY);
///
/// // First tap at t=0
/// sensor.sample(mouse_pos, true, 0, buttons, &store);
/// // Second tap at t=200ms (within DOUBLE_TAP_MS)
/// sensor.sample(mouse_pos, true, 200, buttons, &store);
///
/// assert!(sensor.is_double_tap(entity));
/// ```
pub struct DoubleTapSensor {
    /// Signal history for each entity
    signals: Vec<SignalByte>,

    /// When each entity was last tapped
    last_tap_time: Vec<Option<u64>>,

    /// Tap count for each entity
    tap_count: Vec<u8>,
}

impl DoubleTapSensor {
    /// Creates a new DoubleTapSensor with capacity for MAX_ENTITIES
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            signals: vec![SignalByte::default(); MAX_ENTITIES],
            last_tap_time: vec![None; MAX_ENTITIES],
            tap_count: vec![0; MAX_ENTITIES],
        }
    }

    /// Samples tap state for all entities
    ///
    /// # Arguments
    ///
    /// * `mouse_pos` - Current mouse position
    /// * `is_tapped` - Whether primary button is pressed this frame
    /// * `current_time` - Current timestamp in milliseconds
    /// * `buttons` - Current button state
    /// * `store` - EntityStore with positions and sizes
    ///
    /// # Performance
    ///
    /// O(n) single scan, zero-allocation
    #[inline(never)]
    pub fn sample(
        &mut self,
        mouse_pos: Vec2,
        is_tapped: bool,
        current_time: u64,
        buttons: crate::sensors::mouse_click::PointerButtons,
        store: &EntityStore,
    ) {
        // Only process primary button clicks
        let is_primary_click =
            is_tapped && (buttons.0 & crate::sensors::mouse_click::PointerButtons::PRIMARY != 0);

        for (i, transform) in store.transforms.iter().enumerate() {
            // AABB hit test
            let center_x = transform[0];
            let center_y = transform[1];
            let width = transform[2];
            let height = transform[3];

            let half_w = width * 0.5;
            let half_h = height * 0.5;

            let min_x = center_x - half_w;
            let max_x = center_x + half_w;
            let min_y = center_y - half_h;
            let max_y = center_y + half_h;

            let is_over = mouse_pos.x >= min_x
                && mouse_pos.x <= max_x
                && mouse_pos.y >= min_y
                && mouse_pos.y <= max_y;

            let is_double_tap = if is_over && is_primary_click {
                if let Some(last_tap) = self.last_tap_time[i] {
                    let time_since_last_tap = current_time.saturating_sub(last_tap);

                    if time_since_last_tap <= DOUBLE_TAP_MS {
                        // Quick succession - increment
                        self.tap_count[i] += 1;
                        self.tap_count[i] >= 2 // Double tap detected
                    } else {
                        // Too slow - reset to 1
                        self.tap_count[i] = 1;
                        self.last_tap_time[i] = Some(current_time);
                        false
                    }
                } else {
                    // First tap
                    self.tap_count[i] = 1;
                    self.last_tap_time[i] = Some(current_time);
                    false
                }
            } else {
                // Not tapping - check timeout
                if is_over && !is_primary_click && self.last_tap_time[i].is_some() {
                    let time_since_tap =
                        current_time.saturating_sub(self.last_tap_time[i].unwrap());
                    if time_since_tap > TAP_TIMEOUT_MS {
                        self.tap_count[i] = 0;
                        self.last_tap_time[i] = None;
                    }
                }
                false
            };

            self.signals[i].push(is_double_tap);
        }
    }

    /// Returns true if entity currently has double tap state
    #[inline(always)]
    #[must_use]
    pub fn is_double_tap(&self, entity: EntityId) -> bool {
        let idx = entity.index().0 as usize;
        if idx < self.signals.len() {
            self.signals[idx].get_current()
        } else {
            false
        }
    }

    /// Detects when double tap just occurred (rising edge)
    #[inline(always)]
    #[must_use]
    pub fn on_double_tap(&self, entity: EntityId) -> bool {
        let idx = entity.index().0 as usize;
        if idx < self.signals.len() {
            self.signals[idx].is_rising_edge()
        } else {
            false
        }
    }

    /// Get tap count for an entity
    #[inline(always)]
    #[must_use]
    pub fn tap_count(&self, entity: EntityId) -> u8 {
        let idx = entity.index().0 as usize;
        if idx < self.tap_count.len() {
            self.tap_count[idx]
        } else {
            0
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensors::mouse_click::PointerButtons;

    #[test]
    fn test_new() {
        let sensor = DoubleTapSensor::new();
        assert_eq!(sensor.signals.len(), MAX_ENTITIES);
    }

    #[test]
    fn test_double_tap_detected() {
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = DoubleTapSensor::new();
        let mouse_pos = Vec2::new(100.0, 100.0);
        let buttons = PointerButtons(PointerButtons::PRIMARY);

        // First tap
        sensor.sample(mouse_pos, true, 0, buttons, &store);
        assert!(!sensor.is_double_tap(entity));

        // Second tap within DOUBLE_TAP_MS
        sensor.sample(mouse_pos, true, 200, buttons, &store);
        assert!(sensor.is_double_tap(entity));
    }

    #[test]
    fn test_double_tap_too_slow() {
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = DoubleTapSensor::new();
        let mouse_pos = Vec2::new(100.0, 100.0);
        let buttons = PointerButtons(PointerButtons::PRIMARY);

        // First tap
        sensor.sample(mouse_pos, true, 0, buttons, &store);

        // Second tap too slow (400ms > DOUBLE_TAP_MS)
        sensor.sample(mouse_pos, true, 400, buttons, &store);
        assert!(!sensor.is_double_tap(entity));
    }

    #[test]
    fn test_rising_edge() {
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = DoubleTapSensor::new();
        let mouse_pos = Vec2::new(100.0, 100.0);
        let buttons = PointerButtons(PointerButtons::PRIMARY);

        sensor.sample(mouse_pos, true, 0, buttons, &store);
        sensor.sample(mouse_pos, true, 200, buttons, &store);

        assert!(sensor.on_double_tap(entity));
    }
}
