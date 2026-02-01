// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - MouseClick Sensor Implementation
//
// This sensor detects mouse button clicks on entities using:
// - AABB hit testing (is mouse over entity?)
// - Button state tracking (which buttons are pressed?)
// - SignalByte for 6-tick history (rising edge detection, double-click patterns)
//
// Performance Characteristics:
// - O(n) where n = number of entities (single linear scan)
// - Zero-allocation (pre-allocated SignalByte array per button)
// - Cache-friendly (sequential access to EntityStore SoA)
//
// Memory Impact:
// - 3 bytes per entity (primary + secondary + middle buttons)
// - 300KB for 100,000 entities
//
// ═══════════════════════════════════════════════════════════════════════════════

use crate::signals::SignalByte;
use alloc::vec;
use alloc::vec::Vec;
use archflow_core::Vec2;
use archflow_engine::EntityStore;

/// Bitmask representing mouse button states
///
/// Each bit represents a different mouse button:
/// - Bit 0: Primary (left click)
/// - Bit 1: Secondary (right click)
/// - Bit 2: Middle (wheel click)
/// - Bit 3: Back button
/// - Bit 4: Forward button
///
/// # Examples
///
/// ```
/// use archflow_logic::sensors::mouse_click::PointerButtons;
///
/// let primary = PointerButtons::from_u8(PointerButtons::PRIMARY);
/// assert!(primary.is_primary());
///
/// let both = PointerButtons::from_u8(PointerButtons::PRIMARY | PointerButtons::SECONDARY);
/// assert!(both.is_primary() && both.is_secondary());
/// ```
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct PointerButtons(pub u8);

impl PointerButtons {
    /// Primary mouse button (left click)
    pub const PRIMARY: u8 = 0b00000001;

    /// Secondary mouse button (right click)
    pub const SECONDARY: u8 = 0b00000010;

    /// Middle mouse button (wheel click)
    pub const MIDDLE: u8 = 0b00000100;

    /// Back mouse button
    pub const BACK: u8 = 0b00001000;

    /// Forward mouse button
    pub const FORWARD: u8 = 0b00010000;

    /// Creates a PointerButtons from a raw u8 value
    ///
    /// # Examples
    ///
    /// ```
    /// let buttons = PointerButtons::from_u8(0b00000011);
    /// assert!(buttons.is_primary());
    /// assert!(buttons.is_secondary());
    /// ```
    #[inline(always)]
    #[must_use]
    pub const fn from_u8(value: u8) -> Self {
        Self(value)
    }

    /// Returns true if primary button (left click) is pressed
    ///
    /// # Examples
    ///
    /// ```
    /// let buttons = PointerButtons::from_u8(PointerButtons::PRIMARY);
    /// assert!(buttons.is_primary());
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn is_primary(self) -> bool {
        self.0 & Self::PRIMARY != 0
    }

    /// Returns true if secondary button (right click) is pressed
    ///
    /// # Examples
    ///
    /// ```
    /// let buttons = PointerButtons::from_u8(PointerButtons::SECONDARY);
    /// assert!(buttons.is_secondary());
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn is_secondary(self) -> bool {
        self.0 & Self::SECONDARY != 0
    }

    /// Returns true if middle button (wheel click) is pressed
    #[inline(always)]
    #[must_use]
    pub fn is_middle(self) -> bool {
        self.0 & Self::MIDDLE != 0
    }

    /// Returns the raw u8 value
    #[inline(always)]
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self.0
    }
}

/// Sensor that detects mouse button clicks on entities
///
/// This sensor tracks click state for three mouse buttons independently:
/// - Index 0: Primary (left click)
/// - Index 1: Secondary (right click)
/// - Index 2: Middle (wheel click)
///
/// Each button has its own 6-tick history for edge detection and pattern matching.
///
/// # Examples
///
/// ```
/// use archflow_logic::sensors::mouse_click::{MouseClickSensor, PointerButtons};
/// use archflow_core::Vec2;
/// use archflow_engine::EntityStore;
///
/// let mut store = EntityStore::new();
/// let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
///
/// let mut sensor = MouseClickSensor::new(archflow_engine::MAX_ENTITIES);
///
/// // Mouse at entity position + primary button pressed
/// sensor.sample(
///     Vec2::new(100.0, 100.0),
///     PointerButtons::from_u8(PointerButtons::PRIMARY),
///     &store
/// );
///
/// assert!(sensor.on_click(entity));
/// ```
///
/// # Performance
///
/// - **Time**: O(n) single scan per `sample()` call
/// - **Space**: 3 bytes per entity (one SignalByte per button)
/// - **Allocations**: Zero (pre-allocated on construction)
pub struct MouseClickSensor {
    /// Signal history for each entity and button
    ///
    /// Layout: Vec<[primary: SignalByte, secondary: SignalByte, middle: SignalByte]>
    /// Each SignalByte stores 6 ticks of click state history
    signals: Vec<[SignalByte; 3]>,
}

impl MouseClickSensor {
    /// Creates a new MouseClickSensor with capacity for `capacity` entities
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of entities to track (typically `EntityStore::capacity()`)
    ///
    /// # Examples
    ///
    /// ```
    /// let store = EntityStore::new();
    /// let sensor = MouseClickSensor::new(archflow_engine::MAX_ENTITIES);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            // [primary, secondary, middle] for each entity
            signals: vec![
                [
                    SignalByte::default(),
                    SignalByte::default(),
                    SignalByte::default(),
                ];
                capacity
            ],
        }
    }

    /// Samples the mouse position and button state against all entities
    ///
    /// This performs AABB hit testing for each entity and updates their
    /// 6-tick signal history for each button. Call this once per frame.
    ///
    /// # Arguments
    ///
    /// * `mouse_pos` - Mouse position in world coordinates
    /// * `buttons` - Current state of all mouse buttons
    /// * `store` - EntityStore with transforms
    ///
    /// # Complexity
    ///
    /// O(n) where n = `store.transforms.len()` (number of entities)
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
    /// sensor.sample(
    ///     Vec2::new(100.0, 100.0),
    ///     PointerButtons::from_u8(PointerButtons::PRIMARY),
    ///     &store
    /// );
    /// ```
    #[inline(never)] // Prevent inlining to keep binary size small
    pub fn sample(&mut self, mouse_pos: Vec2, buttons: PointerButtons, store: &EntityStore) {
        // Process all entities in a single cache-friendly loop
        // O(n) but with very low constant factor due to sequential memory access

        let is_primary = buttons.is_primary();
        let is_secondary = buttons.is_secondary();
        let is_middle = buttons.is_middle();

        for (i, transform) in store.transforms.iter().enumerate() {
            // Transform is [x, y, width, height]
            let center_x = transform[0];
            let center_y = transform[1];
            let width = transform[2];
            let height = transform[3];

            // AABB hit test (Axis-Aligned Bounding Box)
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

            // Update 6-tick history for each button channel
            // Signal is only 1 if BOTH conditions are met:
            // 1. Mouse is over the entity (AABB hit test)
            // 2. The specific button is pressed
            self.signals[i][0].push(is_over && is_primary);
            self.signals[i][1].push(is_over && is_secondary);
            self.signals[i][2].push(is_over && is_middle);
        }
    }

    /// Detects click with primary button (left click) on the entity
    ///
    /// Returns true only on the frame when the primary button transitions from
    /// not pressed to pressed while the mouse is over the entity (rising edge).
    ///
    /// # Examples
    ///
    /// ```
    /// sensor.sample(mouse_pos, buttons, &store);
    /// if sensor.on_click(entity) {
    ///     // Primary button just clicked on entity
    ///     // Handle selection, drag start, etc.
    /// }
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn on_click(&self, entity: archflow_core::EntityId) -> bool {
        let idx = entity.index().0 as usize;
        if idx < self.signals.len() {
            self.signals[idx][0].is_rising_edge()
        } else {
            false
        }
    }

    /// Detects click with secondary button (right click) on the entity
    ///
    /// Returns true only on the frame when the secondary button transitions from
    /// not pressed to pressed while the mouse is over the entity (rising edge).
    ///
    /// # Examples
    ///
    /// ```
    /// if sensor.on_right_click(entity) {
    ///     // Secondary button just clicked on entity
    ///     // Show context menu, etc.
    /// }
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn on_right_click(&self, entity: archflow_core::EntityId) -> bool {
        let idx = entity.index().0 as usize;
        if idx < self.signals.len() {
            self.signals[idx][1].is_rising_edge()
        } else {
            false
        }
    }

    /// Detects double-click pattern with primary button
    ///
    /// Returns true when the 6-tick history matches the double-click pattern:
    /// `click - pause - click` = `00100101`
    ///
    /// Pattern breakdown:
    /// - T0 (bit 0): Current frame - released
    /// - T1 (bit 1): Released
    /// - T2 (bit 2): Released
    /// - T3 (bit 3): Clicked (1)
    /// - T4 (bit 4): Released
    /// - T5 (bit 5): Clicked (1)
    ///
    /// # Examples
    ///
    /// ```
    /// if sensor.on_double_click(entity) {
    ///     // Double-click detected
    ///     // Open file, edit text, etc.
    /// }
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn on_double_click(&self, entity: archflow_core::EntityId) -> bool {
        let idx = entity.index().0 as usize;
        if idx < self.signals.len() {
            self.signals[idx][0].is_double_click_pattern()
        } else {
            false
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS (inline for verification during development)
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pointer_buttons_constants() {
        assert_eq!(PointerButtons::PRIMARY, 0b00000001);
        assert_eq!(PointerButtons::SECONDARY, 0b00000010);
        assert_eq!(PointerButtons::MIDDLE, 0b00000100);
        assert_eq!(PointerButtons::BACK, 0b00001000);
        assert_eq!(PointerButtons::FORWARD, 0b00010000);
    }

    #[test]
    fn test_pointer_buttons_is_methods() {
        let primary = PointerButtons::from_u8(PointerButtons::PRIMARY);
        assert!(primary.is_primary());
        assert!(!primary.is_secondary());
        assert!(!primary.is_middle());

        let secondary = PointerButtons::from_u8(PointerButtons::SECONDARY);
        assert!(!secondary.is_primary());
        assert!(secondary.is_secondary());
        assert!(!secondary.is_middle());

        let middle = PointerButtons::from_u8(PointerButtons::MIDDLE);
        assert!(!middle.is_primary());
        assert!(!middle.is_secondary());
        assert!(middle.is_middle());

        let both = PointerButtons::from_u8(PointerButtons::PRIMARY | PointerButtons::SECONDARY);
        assert!(both.is_primary());
        assert!(both.is_secondary());
        assert!(!both.is_middle());
    }

    #[test]
    fn test_mouse_click_sensor_capacity() {
        let sensor = MouseClickSensor::new(1000);
        assert_eq!(sensor.signals.len(), 1000);
    }

    #[test]
    fn test_signals_initialized_to_zero() {
        let sensor = MouseClickSensor::new(100);
        for entity_signals in &sensor.signals {
            assert_eq!(entity_signals[0].as_u8(), 0);
            assert_eq!(entity_signals[1].as_u8(), 0);
            assert_eq!(entity_signals[2].as_u8(), 0);
        }
    }
}
