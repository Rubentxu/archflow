// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - MouseOver Sensor Implementation
//
// This sensor detects when the mouse cursor is over an entity using
// Axis-Aligned Bounding Box (AABB) hit testing with 6-tick history tracking.
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

use crate::signals::SignalByte;
use alloc::vec;
use alloc::vec::Vec;
use archflow_core::{EntityId, Vec2};
use archflow_engine::{EntityStore, MAX_ENTITIES};

/// Sensor that detects when the mouse is over an entity
///
/// # Examples
///
/// ```
/// use archflow_logic::MouseOverSensor;
/// use archflow_core::Vec2;
/// use archflow_engine::EntityStore;
///
/// let mut store = EntityStore::new();
/// let entity = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
///
/// let mut sensor = MouseOverSensor::new(archflow_engine::MAX_ENTITIES);
///
/// // Mouse at center of entity
/// sensor.sample(Vec2::new(100.0, 100.0), &store);
/// assert!(sensor.is_over(entity));
///
/// // Mouse outside entity
/// sensor.sample(Vec2::new(200.0, 200.0), &store);
/// assert!(!sensor.is_over(entity));
/// ```
///
/// # Performance
///
/// - **Time**: O(n) single scan per `sample()` call
/// - **Space**: 1 byte per entity
/// - **Allocations**: Zero (pre-allocated on construction)
pub struct MouseOverSensor {
    /// Signal history for each entity
    ///
    /// Each SignalByte stores 6 ticks of mouse-over state:
    /// - bit 0 (T0): current frame
    /// - bits 1-5 (T1-T5): previous 5 frames
    signals: Vec<SignalByte>,
}

impl MouseOverSensor {
    /// Creates a new MouseOverSensor with capacity for MAX_ENTITIES
    ///
    /// # Examples
    ///
    /// ```
    /// let sensor = MouseOverSensor::new();
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            signals: vec![SignalByte::default(); MAX_ENTITIES],
        }
    }

    /// Samples the mouse position against all entities
    ///
    /// This performs AABB hit testing for each entity and updates their
    /// 6-tick signal history. Call this once per frame.
    ///
    /// # Arguments
    ///
    /// * `mouse_pos` - Mouse position in world coordinates
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
    /// sensor.sample(Vec2::new(100.0, 100.0), &store);
    /// ```
    #[inline(never)] // Prevent inlining to keep binary size small
    pub fn sample(&mut self, mouse_pos: Vec2, store: &EntityStore) {
        // Process all entities in a single cache-friendly loop
        // This is O(n) but with very low constant factor due to:
        // - Sequential memory access (SoA layout)
        // - Simple comparisons (no branching complexity)
        // - No allocations (pure computation)

        for (i, transform) in store.transforms.iter().enumerate() {
            // Transform is [x, y, width, height]
            let center_x = transform[0];
            let center_y = transform[1];
            let width = transform[2];
            let height = transform[3];

            // AABB hit test (Axis-Aligned Bounding Box)
            // Formula: mouse_x >= center_x - width/2  AND  mouse_x <= center_x + width/2
            //          (same for y)

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

            // Update 6-tick history for this entity
            self.signals[i].push(is_over);
        }
    }

    /// Returns true if the mouse is currently over the entity
    ///
    /// This checks the current frame (tick T0) only.
    ///
    /// # Examples
    ///
    /// ```
    /// sensor.sample(mouse_pos, &store);
    /// if sensor.is_over(entity) {
    ///     // Mouse is over entity this frame
    /// }
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn is_over(&self, entity: EntityId) -> bool {
        let idx = entity.index().0 as usize;
        // Safety: EntityId should always be valid
        // In production, add bounds check if needed
        if idx < self.signals.len() {
            self.signals[idx].get_current()
        } else {
            false
        }
    }

    /// Detects the moment when mouse enters the entity (rising edge)
    ///
    /// Returns true only on the frame when the mouse transitions from
    /// outside (0) to inside (1).
    ///
    /// # Examples
    ///
    /// ```
    /// if sensor.on_hover_enter(entity) {
    ///     // Mouse just entered the entity
    ///     // Play sound, show highlight, etc.
    /// }
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn on_hover_enter(&self, entity: EntityId) -> bool {
        let idx = entity.index().0 as usize;
        if idx < self.signals.len() {
            self.signals[idx].is_rising_edge()
        } else {
            false
        }
    }

    /// Detects the moment when mouse leaves the entity (falling edge)
    ///
    /// Returns true only on the frame when the mouse transitions from
    /// inside (1) to outside (0).
    ///
    /// # Examples
    ///
    /// ```
    /// if sensor.on_hover_exit(entity) {
    ///     // Mouse just left the entity
    ///     // Hide tooltip, remove highlight, etc.
    /// }
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn on_hover_exit(&self, entity: EntityId) -> bool {
        let idx = entity.index().0 as usize;
        if idx < self.signals.len() {
            self.signals[idx].is_falling_edge()
        } else {
            false
        }
    }

    /// Returns true if mouse has been steadily over entity for N ticks
    ///
    /// This is useful for debouncing and detecting intentional interaction.
    /// For example, `is_stable_over(entity, 6)` means "mouse has been over
    /// the entity for 6 consecutive frames (100ms at 60 FPS)".
    ///
    /// # Arguments
    ///
    /// * `entity` - Entity to check
    /// * `ticks` - Number of consecutive ticks required (1-6)
    ///
    /// # Examples
    ///
    /// ```
    /// // Show tooltip after 100ms of steady hover (6 ticks @ 60fps)
    /// if sensor.is_stable_over(entity, 6) {
    ///     show_tooltip(entity);
    /// }
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn is_stable_over(&self, entity: EntityId, ticks: u8) -> bool {
        let idx = entity.index().0 as usize;
        if idx < self.signals.len() {
            self.signals[idx].is_steady(ticks)
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
    fn test_capacity() {
        let sensor = MouseOverSensor::new();
        assert_eq!(sensor.signals.len(), MAX_ENTITIES);
    }

    #[test]
    fn test_signals_initialized_to_zero() {
        let sensor = MouseOverSensor::new();
        for signal in &sensor.signals[..100] {
            assert_eq!(signal.as_u8(), 0);
        }
    }

    #[test]
    fn test_aabb_formula() {
        // Verify AABB hit test formula
        // center (100, 100), size (50, 50) → bounds: [75, 125] × [75, 125]

        let pos = Vec2::new(100.0, 100.0);
        let size = Vec2::new(50.0, 50.0);
        let half_w = size.x * 0.5; // 25.0
        let half_h = size.y * 0.5; // 25.0

        let min_x = pos.x - half_w; // 75.0
        let max_x = pos.x + half_w; // 125.0
        let min_y = pos.y - half_h; // 75.0
        let max_y = pos.y + half_h; // 125.0

        // Test corners
        assert!(100.0 >= min_x && 100.0 <= max_x && 100.0 >= min_y && 100.0 <= max_y);
        assert!(75.0 >= min_x && 75.0 <= max_x && 75.0 >= min_y && 75.0 <= max_y);
        assert!(125.0 >= min_x && 125.0 <= max_x && 125.0 >= min_y && 125.0 <= max_y);

        // Test outside
        assert!(!(74.0 >= min_x && 74.0 <= max_x && 74.0 >= min_y && 74.0 <= max_y));
    }
}
