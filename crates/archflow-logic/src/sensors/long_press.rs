// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Long Press Sensor Implementation
//
// This sensor detects when entities are pressed/touched for an extended duration.
// Uses per-entity timing tracking with 6-tick signal history.
//
// Performance Characteristics:
// - O(n) where n = number of entities (single linear scan)
// - Per-entity timing state (press start time)
// - Cache-friendly (sequential access to EntityStore SoA)
//
// Memory Impact:
// - 1 byte per entity (SignalByte)
// - 8 bytes per entity (u64 timestamp) for timing
// - ~900KB for 100,000 entities
//
// ═══════════════════════════════════════════════════════════════════════════════

use crate::signals::SignalByte;
use alloc::vec;
use alloc::vec::Vec;
use archflow_core::{EntityId, Vec2};
use archflow_engine::{EntityStore, MAX_ENTITIES};

/// Default long press duration in milliseconds
pub const LONG_PRESS_MS: u64 = 500;

/// Maximum time to wait before resetting press state
pub const RELEASE_TIMEOUT_MS: u64 = 500;

/// Sensor that detects long press (press-and-hold) on entities
///
/// Tracks per-entity press duration and maintains 6-tick signal history.
///
/// # Examples
///
/// ```
/// use archflow_logic::sensors::LongPressSensor;
/// use archflow_core::Vec2;
/// use archflow_engine::EntityStore;
///
/// let mut store = EntityStore::new();
/// let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
///
/// let mut sensor = LongPressSensor::new();
///
/// // Simulate 600ms of pressing
/// let current_time = 600;
/// sensor.sample(Vec2::new(100.0, 100.0), true, current_time, &store);
///
/// assert!(sensor.is_long_press(entity));
/// ```
pub struct LongPressSensor {
    /// Signal history for each entity
    signals: Vec<SignalByte>,

    /// When each entity press started (None if not pressed)
    press_start_time: Vec<Option<u64>>,

    /// When each entity was released (for timeout tracking)
    release_time: Vec<Option<u64>>,
}

impl LongPressSensor {
    /// Creates a new LongPressSensor with capacity for MAX_ENTITIES
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            signals: vec![SignalByte::default(); MAX_ENTITIES],
            press_start_time: vec![None; MAX_ENTITIES],
            release_time: vec![None; MAX_ENTITIES],
        }
    }

    /// Samples press state for all entities
    ///
    /// # Arguments
    ///
    /// * `mouse_pos` - Current mouse/touch position
    /// * `is_pressed` - Whether primary button is pressed
    /// * `current_time` - Current timestamp in milliseconds
    /// * `store` - EntityStore with positions and sizes
    ///
    /// # Performance
    ///
    /// O(n) single scan, zero-allocation
    #[inline(never)]
    pub fn sample(
        &mut self,
        mouse_pos: Vec2,
        is_pressed: bool,
        current_time: u64,
        store: &EntityStore,
    ) {
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

            // Long press = is_over AND is_pressed AND duration exceeded
            let is_long_press = if is_over && is_pressed {
                if let Some(start_time) = self.press_start_time[i] {
                    let duration = current_time.saturating_sub(start_time);
                    duration >= LONG_PRESS_MS
                } else {
                    // Just started pressing
                    self.press_start_time[i] = Some(current_time);
                    self.release_time[i] = None;
                    false
                }
            } else {
                // Not pressing - check for timeout
                if self.press_start_time[i].is_some() && self.release_time[i].is_none() {
                    self.release_time[i] = Some(current_time);
                }

                if let Some(release) = self.release_time[i] {
                    let time_since_release = current_time.saturating_sub(release);
                    if time_since_release > RELEASE_TIMEOUT_MS {
                        self.press_start_time[i] = None;
                        self.release_time[i] = None;
                    }
                }
                false
            };

            self.signals[i].push(is_long_press);
        }
    }

    /// Returns true if entity is currently in long press state
    #[inline(always)]
    #[must_use]
    pub fn is_long_press(&self, entity: EntityId) -> bool {
        let idx = entity.index().0 as usize;
        if idx < self.signals.len() {
            self.signals[idx].get_current()
        } else {
            false
        }
    }

    /// Detects when long press just triggered (rising edge)
    #[inline(always)]
    #[must_use]
    pub fn on_long_press(&self, entity: EntityId) -> bool {
        let idx = entity.index().0 as usize;
        if idx < self.signals.len() {
            self.signals[idx].is_rising_edge()
        } else {
            false
        }
    }

    /// Detects when long press just ended (falling edge)
    #[inline(always)]
    #[must_use]
    pub fn on_long_press_end(&self, entity: EntityId) -> bool {
        let idx = entity.index().0 as usize;
        if idx < self.signals.len() {
            self.signals[idx].is_falling_edge()
        } else {
            false
        }
    }

    /// Get press duration for an entity
    #[inline(always)]
    #[must_use]
    pub fn press_duration(&self, entity: EntityId, current_time: u64) -> Option<u64> {
        let idx = entity.index().0 as usize;
        if idx < self.press_start_time.len() {
            self.press_start_time[idx].map(|start| current_time.saturating_sub(start))
        } else {
            None
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let sensor = LongPressSensor::new();
        assert_eq!(sensor.signals.len(), MAX_ENTITIES);
    }

    #[test]
    fn test_short_press_no_trigger() {
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = LongPressSensor::new();
        let mouse_pos = Vec2::new(100.0, 100.0);

        // Press for only 200ms
        sensor.sample(mouse_pos, true, 0, &store);
        sensor.sample(mouse_pos, true, 200, &store);

        assert!(!sensor.is_long_press(entity));
    }

    #[test]
    fn test_long_press_triggers() {
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = LongPressSensor::new();
        let mouse_pos = Vec2::new(100.0, 100.0);

        // Press for 600ms
        sensor.sample(mouse_pos, true, 0, &store);
        sensor.sample(mouse_pos, true, 400, &store);
        sensor.sample(mouse_pos, true, 600, &store);

        assert!(sensor.is_long_press(entity));
    }

    #[test]
    fn test_rising_edge() {
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = LongPressSensor::new();
        let mouse_pos = Vec2::new(100.0, 100.0);

        sensor.sample(mouse_pos, true, 0, &store);
        sensor.sample(mouse_pos, true, 400, &store);
        sensor.sample(mouse_pos, true, 600, &store);

        assert!(sensor.on_long_press(entity));
    }

    #[test]
    fn test_no_trigger_when_not_over() {
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = LongPressSensor::new();
        let mouse_pos = Vec2::new(200.0, 200.0); // Far from entity

        // Press for 600ms but not over entity
        sensor.sample(mouse_pos, true, 0, &store);
        sensor.sample(mouse_pos, true, 600, &store);

        assert!(!sensor.is_long_press(entity));
    }
}
