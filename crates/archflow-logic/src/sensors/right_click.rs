// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Right Click Sensor Implementation
//
// This sensor detects right mouse button clicks on entities using
// AABB hit testing combined with right button state, with 6-tick history tracking.
//
// Performance Characteristics:
// - O(n) where n = number of entities (single linear scan)
// - Zero-allocation (pre-allocated SignalByte array)
// - Cache-friendly (sequential access to EntityStore SoA)
//
// Memory Impact:
// - 1 byte per entity (SignalByte)
// - 100KB for 100,000 entities
//
// ═══════════════════════════════════════════════════════════════════════════════

use crate::sensors::mouse_click::PointerButtons;
use crate::signals::SignalByte;
use alloc::vec;
use alloc::vec::Vec;
use archflow_core::{EntityId, Vec2};
use archflow_engine::{EntityStore, MAX_ENTITIES};

/// Sensor that detects right mouse button clicks on entities
///
/// Combines AABB hit testing with right button state to detect right-clicks.
/// Maintains 6-tick signal history per entity for edge detection.
///
/// # Examples
///
/// ```
/// use archflow_logic::sensors::RightClickSensor;
/// use archflow_logic::sensors::mouse_click::PointerButtons;
/// use archflow_core::Vec2;
/// use archflow_engine::EntityStore;
///
/// let mut store = EntityStore::new();
/// let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
///
/// let mut sensor = RightClickSensor::new();
///
/// // Right click on entity
/// let mouse_pos = Vec2::new(100.0, 100.0);
/// let buttons = PointerButtons::from_u8(PointerButtons::SECONDARY);
/// sensor.sample(mouse_pos, buttons, &store);
///
/// assert!(sensor.is_clicked(entity));
/// assert!(sensor.on_right_click(entity));  // Rising edge
/// ```
///
/// # Performance
///
/// - **Time**: O(n) single scan per `sample()` call
/// - **Space**: 1 byte per entity
/// - **Allocations**: Zero (pre-allocated on construction)
pub struct RightClickSensor {
    /// Signal history for each entity
    ///
    /// Each SignalByte stores 6 ticks of right-click state:
    /// - bit 0 (T0): current frame
    /// - bits 1-5 (T1-T5): previous 5 frames
    signals: Vec<SignalByte>,
}

impl RightClickSensor {
    /// Creates a new RightClickSensor with capacity for MAX_ENTITIES
    ///
    /// # Examples
    ///
    /// ```
    /// let sensor = RightClickSensor::new();
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            signals: vec![SignalByte::default(); MAX_ENTITIES],
        }
    }

    /// Samples the mouse position and button state against all entities
    ///
    /// This performs AABB hit testing for each entity combined with right button
    /// state, then updates their 6-tick signal history. Call this once per frame.
    ///
    /// # Arguments
    ///
    /// * `mouse_pos` - Mouse position in world coordinates
    /// * `buttons` - Current mouse button state
    /// * `store` - EntityStore with positions and sizes
    ///
    /// # Complexity
    ///
    /// O(n) where n = `store.len()` (number of entities)
    ///
    /// # Performance
    ///
    /// - Zero-allocation
    /// - Cache-friendly (linear scan of SoA arrays)
    /// - Branch-predictor friendly (simple boolean checks)
    ///
    /// # Examples
    ///
    /// ```
    /// let buttons = PointerButtons::from_u8(PointerButtons::SECONDARY);
    /// sensor.sample(Vec2::new(100.0, 100.0), buttons, &store);
    /// ```
    #[inline(never)]
    pub fn sample(&mut self, mouse_pos: Vec2, buttons: PointerButtons, store: &EntityStore) {
        // Extract right button state once
        let is_right_button_pressed = buttons.0 & PointerButtons::SECONDARY != 0;

        // Process all entities in a single cache-friendly loop
        for (i, transform) in store.transforms.iter().enumerate() {
            // Transform is [x, y, width, height]
            let center_x = transform[0];
            let center_y = transform[1];
            let width = transform[2];
            let height = transform[3];

            // AABB hit test
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

            // Right click = mouse is over entity AND right button is pressed
            let is_right_click = is_over && is_right_button_pressed;

            // Update 6-tick history for this entity
            self.signals[i].push(is_right_click);
        }
    }

    /// Returns true if right button is currently pressed on the entity
    ///
    /// This checks the current frame (tick T0) only.
    ///
    /// # Examples
    ///
    /// ```
    /// sensor.sample(mouse_pos, buttons, &store);
    /// if sensor.is_clicked(entity) {
    ///     // Right button is down on entity this frame
    /// }
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn is_clicked(&self, entity: EntityId) -> bool {
        let idx = entity.index().0 as usize;
        if idx < self.signals.len() {
            self.signals[idx].get_current()
        } else {
            false
        }
    }

    /// Detects the moment when right button is pressed on the entity (rising edge)
    ///
    /// Returns true only on the frame when the right button transitions from
    /// not pressed (0) to pressed (1) while over the entity.
    ///
    /// # Examples
    ///
    /// ```
    /// if sensor.on_right_click(entity) {
    ///     // Right button just pressed on entity
    ///     // Show context menu, trigger action, etc.
    /// }
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn on_right_click(&self, entity: EntityId) -> bool {
        let idx = entity.index().0 as usize;
        if idx < self.signals.len() {
            self.signals[idx].is_rising_edge()
        } else {
            false
        }
    }

    /// Detects the moment when right button is released on the entity (falling edge)
    ///
    /// Returns true only on the frame when the right button transitions from
    /// pressed (1) to not pressed (0) while over the entity.
    ///
    /// # Examples
    ///
    /// ```
    /// if sensor.on_right_release(entity) {
    ///     // Right button just released on entity
    /// }
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn on_right_release(&self, entity: EntityId) -> bool {
        let idx = entity.index().0 as usize;
        if idx < self.signals.len() {
            self.signals[idx].is_falling_edge()
        } else {
            false
        }
    }

    /// Returns true if right button has been steadily held on entity for N ticks
    ///
    /// This is useful for detecting deliberate right-click-and-hold gestures.
    ///
    /// # Arguments
    ///
    /// * `entity` - Entity to check
    /// * `ticks` - Number of consecutive ticks required (1-6)
    ///
    /// # Examples
    ///
    /// ```
    /// // Show context menu after 100ms of steady right-click (6 ticks @ 60fps)
    /// if sensor.is_stable_right_click(entity, 6) {
    ///     show_context_menu(entity);
    /// }
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn is_stable_right_click(&self, entity: EntityId, ticks: u8) -> bool {
        let idx = entity.index().0 as usize;
        if idx < self.signals.len() {
            self.signals[idx].is_steady(ticks)
        } else {
            false
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_engine::EntityStore;

    #[test]
    fn test_new() {
        let sensor = RightClickSensor::new();
        assert_eq!(sensor.signals.len(), MAX_ENTITIES);
    }

    #[test]
    fn test_right_click_detected() {
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = RightClickSensor::new();

        // Right click on entity
        let mouse_pos = Vec2::new(100.0, 100.0);
        let buttons = PointerButtons(PointerButtons::SECONDARY);
        sensor.sample(mouse_pos, buttons, &store);

        assert!(sensor.is_clicked(entity));
    }

    #[test]
    fn test_no_click_when_not_over() {
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = RightClickSensor::new();

        // Right click NOT on entity (mouse far away)
        let mouse_pos = Vec2::new(200.0, 200.0);
        let buttons = PointerButtons(PointerButtons::SECONDARY);
        sensor.sample(mouse_pos, buttons, &store);

        assert!(!sensor.is_clicked(entity));
    }

    #[test]
    fn test_no_click_when_left_button() {
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = RightClickSensor::new();

        // Left click on entity
        let mouse_pos = Vec2::new(100.0, 100.0);
        let buttons = PointerButtons(PointerButtons::PRIMARY);
        sensor.sample(mouse_pos, buttons, &store);

        assert!(!sensor.is_clicked(entity));
    }

    #[test]
    fn test_rising_edge() {
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = RightClickSensor::new();
        let mouse_pos = Vec2::new(100.0, 100.0);

        // First sample: no click
        sensor.sample(mouse_pos, PointerButtons(0), &store);
        assert!(!sensor.on_right_click(entity));

        // Second sample: right click starts
        sensor.sample(mouse_pos, PointerButtons(PointerButtons::SECONDARY), &store);
        assert!(sensor.on_right_click(entity));

        // Third sample: still clicking
        sensor.sample(mouse_pos, PointerButtons(PointerButtons::SECONDARY), &store);
        assert!(!sensor.on_right_click(entity)); // No longer rising edge
    }

    #[test]
    fn test_falling_edge() {
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = RightClickSensor::new();
        let mouse_pos = Vec2::new(100.0, 100.0);

        // Start right click
        sensor.sample(mouse_pos, PointerButtons(PointerButtons::SECONDARY), &store);
        assert!(!sensor.on_right_release(entity));

        // Continue clicking
        sensor.sample(mouse_pos, PointerButtons(PointerButtons::SECONDARY), &store);
        sensor.sample(mouse_pos, PointerButtons(PointerButtons::SECONDARY), &store);

        // Release
        sensor.sample(mouse_pos, PointerButtons(0), &store);
        assert!(sensor.on_right_release(entity));
    }

    #[test]
    fn test_stable_right_click() {
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        let mut sensor = RightClickSensor::new();
        let mouse_pos = Vec2::new(100.0, 100.0);
        let buttons = PointerButtons(PointerButtons::SECONDARY);

        // Start clicking
        sensor.sample(mouse_pos, buttons, &store);
        assert!(!sensor.is_stable_right_click(entity, 6));

        // Continue for 6 ticks
        for _ in 0..5 {
            sensor.sample(mouse_pos, buttons, &store);
        }

        assert!(sensor.is_stable_right_click(entity, 6));
    }
}
