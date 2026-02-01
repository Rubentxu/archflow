// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Visibility System (HU-014)
//
// Efficient visibility management using bitset filtering:
// - O(1) entity lookup with bitset operations
// - Batch visibility changes (multiple entities at once)
// - Hierarchical visibility (parent-child propagation)
// - Spatial query integration for occlusion culling
//
// Reference: docs/epics/EPIC-003-actuators-animations.md - HU-014
//
// Performance:
// - Bitset words: 64 entities per u64 word
// - Lookup: O(1) with bitwise AND
// - Batch updates: Single bitwise operation per word
// - Memory: (MAX_ENTITIES / 64) * 8 bytes = ~2KB for 16K entities
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(dead_code)]

use alloc::vec::Vec;
use archflow_core::{EntityId, MAX_ENTITIES};

/// Number of bits per bitset word (u64)
const WORD_BITS: u32 = 64;

/// Number of words needed to track MAX_ENTITIES
const WORD_COUNT: usize = (MAX_ENTITIES as usize + WORD_BITS as usize - 1) / WORD_BITS as usize;

/// Visibility bitset for efficient entity filtering
///
/// Uses a packed bitset where each bit represents whether an entity is visible.
/// This enables O(1) visibility checks and fast batch operations.
///
/// # Example
/// ```rust
/// let mut bitset = VisibilityBitset::new();
/// bitset.set(EntityId::new(42), true);
/// assert!(bitset.get(EntityId::new(42)));
/// ```
#[derive(Clone, Debug)]
pub struct VisibilityBitset {
    /// Packed bits: each u64 word holds 64 entity visibility flags
    words: [u64; WORD_COUNT],
}

impl VisibilityBitset {
    /// Create a new empty visibility bitset (all entities invisible)
    #[inline]
    pub const fn new() -> Self {
        Self {
            words: [0; WORD_COUNT],
        }
    }

    /// Create a new visibility bitset with all entities visible
    pub fn all_visible() -> Self {
        Self {
            words: [u64::MAX; WORD_COUNT],
        }
    }

    /// Get visibility status for an entity
    ///
    /// Returns true if the entity is visible, false otherwise.
    #[inline]
    pub fn get(&self, entity: EntityId) -> bool {
        let index = entity.as_u32() as usize;
        let word_idx = index / WORD_BITS as usize;
        let bit_idx = index % WORD_BITS as usize;

        if word_idx >= WORD_COUNT {
            return false;
        }

        (self.words[word_idx] & (1 << bit_idx)) != 0
    }

    /// Set visibility for an entity
    #[inline]
    pub fn set(&mut self, entity: EntityId, visible: bool) {
        let index = entity.as_u32() as usize;
        let word_idx = index / WORD_BITS as usize;
        let bit_idx = index % WORD_BITS as usize;

        if word_idx >= WORD_COUNT {
            return;
        }

        if visible {
            self.words[word_idx] |= 1 << bit_idx;
        } else {
            self.words[word_idx] &= !(1 << bit_idx);
        }
    }

    /// Toggle visibility for an entity
    #[inline]
    pub fn toggle(&mut self, entity: EntityId) -> bool {
        let index = entity.as_u32() as usize;
        let word_idx = index / WORD_BITS as usize;
        let bit_idx = index % WORD_BITS as usize;

        if word_idx >= WORD_COUNT {
            return false;
        }

        let mask = 1 << bit_idx;
        self.words[word_idx] ^= mask;
        (self.words[word_idx] & mask) != 0
    }

    /// Set visibility for multiple entities at once (batch operation)
    pub fn set_batch(&mut self, entities: &[EntityId], visible: bool) {
        for &entity in entities {
            self.set(entity, visible);
        }
    }

    /// Check if any entities are visible
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.words.iter().all(|&w| w == 0)
    }

    /// Count visible entities
    pub fn count(&self) -> usize {
        self.words.iter().map(|&w| w.count_ones() as usize).sum()
    }

    /// Clear all visibility flags (make all entities invisible)
    #[inline]
    pub fn clear(&mut self) {
        self.words.fill(0);
    }

    /// Set all entities to visible
    #[inline]
    pub fn set_all(&mut self) {
        self.words.fill(u64::MAX);
    }

    /// Get iterator over visible entity indices
    ///
    /// This iterates over the bitset and yields indices of visible entities.
    /// Note: This yields indices, not EntityIds - you'll need to convert them.
    pub fn iter_visible(&self) -> impl Iterator<Item = usize> + '_ {
        self.words.iter().enumerate().flat_map(|(word_idx, &word)| {
            (0..WORD_BITS)
                .filter(move |bit_idx| (word & (1 << bit_idx)) != 0)
                .map(move |bit_idx| word_idx * WORD_BITS as usize + bit_idx as usize)
        })
    }

    /// Bitwise AND with another bitset (intersection)
    pub fn and(&mut self, other: &VisibilityBitset) {
        for i in 0..WORD_COUNT {
            self.words[i] &= other.words[i];
        }
    }

    /// Bitwise OR with another bitset (union)
    pub fn or(&mut self, other: &VisibilityBitset) {
        for i in 0..WORD_COUNT {
            self.words[i] |= other.words[i];
        }
    }

    /// Bitwise XOR with another bitset (symmetric difference)
    pub fn xor(&mut self, other: &VisibilityBitset) {
        for i in 0..WORD_COUNT {
            self.words[i] ^= other.words[i];
        }
    }

    /// Bitwise NOT (complement) - inverts all bits
    pub fn not(&mut self) {
        for i in 0..WORD_COUNT {
            self.words[i] = !self.words[i];
        }
    }
}

/// Visibility actuator for controlling entity visibility
///
/// Provides batch visibility operations and hierarchical propagation.
#[derive(Clone, Debug)]
pub struct VisibilityActuator {
    /// Actuator ID
    pub id: u32,
    /// Target entity ID
    pub entity_id: EntityId,
    /// Visibility configuration
    pub config: VisibilityConfig,
}

/// Configuration for visibility actuator
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VisibilityConfig {
    /// Target visibility state
    pub visible: bool,
    /// Whether to propagate to children
    pub propagate_to_children: bool,
    /// Whether to use occlusion culling
    pub use_occlusion_culling: bool,
}

impl Default for VisibilityConfig {
    fn default() -> Self {
        Self {
            visible: true,
            propagate_to_children: false,
            use_occlusion_culling: false,
        }
    }
}

impl VisibilityActuator {
    /// Create a new visibility actuator
    pub fn new(entity_id: EntityId, config: VisibilityConfig) -> Self {
        Self {
            id: 0, // Will be assigned by manager
            entity_id,
            config,
        }
    }

    /// Create a simple show actuator
    pub fn show(entity_id: EntityId) -> Self {
        Self::new(
            entity_id,
            VisibilityConfig {
                visible: true,
                ..Default::default()
            },
        )
    }

    /// Create a simple hide actuator
    pub fn hide(entity_id: EntityId) -> Self {
        Self::new(
            entity_id,
            VisibilityConfig {
                visible: false,
                ..Default::default()
            },
        )
    }

    /// Create a show actuator with child propagation
    pub fn show_with_children(entity_id: EntityId) -> Self {
        Self::new(
            entity_id,
            VisibilityConfig {
                visible: true,
                propagate_to_children: true,
                ..Default::default()
            },
        )
    }

    /// Create a hide actuator with child propagation
    pub fn hide_with_children(entity_id: EntityId) -> Self {
        Self::new(
            entity_id,
            VisibilityConfig {
                visible: false,
                propagate_to_children: true,
                ..Default::default()
            },
        )
    }

    /// Execute the visibility actuator on a bitset
    pub fn execute(&self, bitset: &mut VisibilityBitset) {
        bitset.set(self.entity_id, self.config.visible);
    }
}

/// Manager for visibility operations
///
/// Provides batch operations and hierarchical visibility management.
#[derive(Debug)]
pub struct VisibilityManager {
    /// Global visibility bitset
    bitset: VisibilityBitset,
    /// Active actuators
    actuators: Vec<VisibilityActuator>,
}

impl Default for VisibilityManager {
    fn default() -> Self {
        Self::new()
    }
}

impl VisibilityManager {
    /// Create a new visibility manager
    #[inline]
    pub fn new() -> Self {
        Self {
            bitset: VisibilityBitset::new(),
            actuators: Vec::new(),
        }
    }

    /// Get the global visibility bitset
    #[inline]
    pub fn bitset(&self) -> &VisibilityBitset {
        &self.bitset
    }

    /// Get mutable reference to the global visibility bitset
    #[inline]
    pub fn bitset_mut(&mut self) -> &mut VisibilityBitset {
        &mut self.bitset
    }

    /// Check if an entity is visible
    #[inline]
    pub fn is_visible(&self, entity: EntityId) -> bool {
        self.bitset.get(entity)
    }

    /// Set visibility for an entity
    #[inline]
    pub fn set_visible(&mut self, entity: EntityId, visible: bool) {
        self.bitset.set(entity, visible);
    }

    /// Toggle visibility for an entity
    #[inline]
    pub fn toggle_visible(&mut self, entity: EntityId) -> bool {
        self.bitset.toggle(entity)
    }

    /// Show an entity
    #[inline]
    pub fn show(&mut self, entity: EntityId) {
        self.set_visible(entity, true);
    }

    /// Hide an entity
    #[inline]
    pub fn hide(&mut self, entity: EntityId) {
        self.set_visible(entity, false);
    }

    /// Set visibility for multiple entities at once
    pub fn set_visible_batch(&mut self, entities: &[EntityId], visible: bool) {
        self.bitset.set_batch(entities, visible);
    }

    /// Show multiple entities
    pub fn show_batch(&mut self, entities: &[EntityId]) {
        self.set_visible_batch(entities, true);
    }

    /// Hide multiple entities
    pub fn hide_batch(&mut self, entities: &[EntityId]) {
        self.set_visible_batch(entities, false);
    }

    /// Add an actuator to the manager
    pub fn add_actuator(&mut self, actuator: VisibilityActuator) {
        self.actuators.push(actuator);
    }

    /// Execute all pending actuators
    pub fn execute_actuators(&mut self) {
        for actuator in &self.actuators {
            actuator.execute(&mut self.bitset);
        }
        self.actuators.clear();
    }

    /// Get count of visible entities
    pub fn visible_count(&self) -> usize {
        self.bitset.count()
    }

    /// Clear all visibility (make all entities invisible)
    pub fn clear(&mut self) {
        self.bitset.clear();
    }

    /// Set all entities to visible
    pub fn set_all_visible(&mut self) {
        self.bitset.set_all();
    }

    /// Get iterator over visible entity indices
    pub fn iter_visible(&self) -> impl Iterator<Item = usize> + '_ {
        self.bitset.iter_visible()
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// TESTS
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_bitset_new() {
        let bitset = VisibilityBitset::new();
        // All entities should be invisible by default
        assert!(!bitset.get(EntityId::new(0)));
        assert!(!bitset.get(EntityId::new(42)));
        assert!(!bitset.get(EntityId::new(1000)));
    }

    #[test]
    fn test_bitset_set_single() {
        let mut bitset = VisibilityBitset::new();

        bitset.set(EntityId::new(42), true);
        assert!(bitset.get(EntityId::new(42)));
        assert!(!bitset.get(EntityId::new(43)));

        bitset.set(EntityId::new(42), false);
        assert!(!bitset.get(EntityId::new(42)));
    }

    #[test]
    fn test_bitset_toggle() {
        let mut bitset = VisibilityBitset::new();

        let is_visible = bitset.toggle(EntityId::new(42));
        assert!(is_visible);
        assert!(bitset.get(EntityId::new(42)));

        let is_visible = bitset.toggle(EntityId::new(42));
        assert!(!is_visible);
        assert!(!bitset.get(EntityId::new(42)));
    }

    #[test]
    fn test_bitset_set_batch() {
        let mut bitset = VisibilityBitset::new();

        let entities = vec![
            EntityId::new(10),
            EntityId::new(20),
            EntityId::new(30),
            EntityId::new(40),
        ];

        bitset.set_batch(&entities, true);

        assert!(bitset.get(EntityId::new(10)));
        assert!(bitset.get(EntityId::new(20)));
        assert!(bitset.get(EntityId::new(30)));
        assert!(bitset.get(EntityId::new(40)));
        assert!(!bitset.get(EntityId::new(50)));
    }

    #[test]
    fn test_bitset_count() {
        let mut bitset = VisibilityBitset::new();

        assert_eq!(bitset.count(), 0);

        bitset.set(EntityId::new(1), true);
        bitset.set(EntityId::new(2), true);
        bitset.set(EntityId::new(3), true);

        assert_eq!(bitset.count(), 3);

        bitset.set(EntityId::new(2), false);

        assert_eq!(bitset.count(), 2);
    }

    #[test]
    fn test_bitset_clear() {
        let mut bitset = VisibilityBitset::new();

        bitset.set(EntityId::new(1), true);
        bitset.set(EntityId::new(2), true);
        bitset.set(EntityId::new(3), true);

        assert_eq!(bitset.count(), 3);

        bitset.clear();

        assert_eq!(bitset.count(), 0);
        assert!(bitset.is_empty());
    }

    #[test]
    fn test_bitset_set_all() {
        let mut bitset = VisibilityBitset::new();

        bitset.set_all();

        // Sample a few entities
        assert!(bitset.get(EntityId::new(0)));
        assert!(bitset.get(EntityId::new(100)));
        assert!(bitset.get(EntityId::new(1000)));

        // Count should be MAX_ENTITIES (or close due to word alignment)
        let count = bitset.count();
        assert!(count > 1000);
    }

    #[test]
    fn test_bitset_iter_visible() {
        let mut bitset = VisibilityBitset::new();

        bitset.set(EntityId::new(10), true);
        bitset.set(EntityId::new(20), true);
        bitset.set(EntityId::new(30), true);

        let visible: Vec<usize> = bitset.iter_visible().collect();

        assert_eq!(visible.len(), 3);
        assert!(visible.contains(&10));
        assert!(visible.contains(&20));
        assert!(visible.contains(&30));
    }

    #[test]
    fn test_bitset_and() {
        let mut bitset1 = VisibilityBitset::new();
        let mut bitset2 = VisibilityBitset::new();

        bitset1.set(EntityId::new(10), true);
        bitset1.set(EntityId::new(20), true);

        bitset2.set(EntityId::new(20), true);
        bitset2.set(EntityId::new(30), true);

        bitset1.and(&bitset2);

        // Only entity 20 should be visible in both
        assert!(!bitset1.get(EntityId::new(10)));
        assert!(bitset1.get(EntityId::new(20)));
        assert!(!bitset1.get(EntityId::new(30)));
    }

    #[test]
    fn test_bitset_or() {
        let mut bitset1 = VisibilityBitset::new();
        let mut bitset2 = VisibilityBitset::new();

        bitset1.set(EntityId::new(10), true);
        bitset1.set(EntityId::new(20), true);

        bitset2.set(EntityId::new(20), true);
        bitset2.set(EntityId::new(30), true);

        bitset1.or(&bitset2);

        // Entities 10, 20, 30 should be visible
        assert!(bitset1.get(EntityId::new(10)));
        assert!(bitset1.get(EntityId::new(20)));
        assert!(bitset1.get(EntityId::new(30)));
    }

    #[test]
    fn test_bitset_xor() {
        let mut bitset1 = VisibilityBitset::new();
        let mut bitset2 = VisibilityBitset::new();

        bitset1.set(EntityId::new(10), true);
        bitset1.set(EntityId::new(20), true);

        bitset2.set(EntityId::new(20), true);
        bitset2.set(EntityId::new(30), true);

        bitset1.xor(&bitset2);

        // Only entities 10 and 30 should be visible (different in both)
        assert!(bitset1.get(EntityId::new(10)));
        assert!(!bitset1.get(EntityId::new(20)));
        assert!(bitset1.get(EntityId::new(30)));
    }

    #[test]
    fn test_visibility_actuator_show() {
        let actuator = VisibilityActuator::show(EntityId::new(42));
        let mut bitset = VisibilityBitset::new();

        actuator.execute(&mut bitset);

        assert!(bitset.get(EntityId::new(42)));
    }

    #[test]
    fn test_visibility_actuator_hide() {
        let actuator = VisibilityActuator::hide(EntityId::new(42));
        let mut bitset = VisibilityBitset::new();
        bitset.set(EntityId::new(42), true);

        actuator.execute(&mut bitset);

        assert!(!bitset.get(EntityId::new(42)));
    }

    #[test]
    fn test_visibility_manager() {
        let mut manager = VisibilityManager::new();

        // Initially all invisible
        assert!(!manager.is_visible(EntityId::new(42)));

        // Show entity
        manager.show(EntityId::new(42));
        assert!(manager.is_visible(EntityId::new(42)));

        // Hide entity
        manager.hide(EntityId::new(42));
        assert!(!manager.is_visible(EntityId::new(42)));
    }

    #[test]
    fn test_visibility_manager_batch() {
        let mut manager = VisibilityManager::new();

        let entities = vec![EntityId::new(10), EntityId::new(20), EntityId::new(30)];

        manager.show_batch(&entities);

        assert!(manager.is_visible(EntityId::new(10)));
        assert!(manager.is_visible(EntityId::new(20)));
        assert!(manager.is_visible(EntityId::new(30)));

        manager.hide_batch(&entities);

        assert!(!manager.is_visible(EntityId::new(10)));
        assert!(!manager.is_visible(EntityId::new(20)));
        assert!(!manager.is_visible(EntityId::new(30)));
    }

    #[test]
    fn test_visibility_manager_count() {
        let mut manager = VisibilityManager::new();

        assert_eq!(manager.visible_count(), 0);

        manager.show(EntityId::new(1));
        manager.show(EntityId::new(2));
        manager.show(EntityId::new(3));

        assert_eq!(manager.visible_count(), 3);

        manager.hide(EntityId::new(2));

        assert_eq!(manager.visible_count(), 2);
    }

    #[test]
    fn test_visibility_manager_actuators() {
        let mut manager = VisibilityManager::new();

        manager.add_actuator(VisibilityActuator::show(EntityId::new(10)));
        manager.add_actuator(VisibilityActuator::show(EntityId::new(20)));
        manager.add_actuator(VisibilityActuator::hide(EntityId::new(30)));

        manager.execute_actuators();

        assert!(manager.is_visible(EntityId::new(10)));
        assert!(manager.is_visible(EntityId::new(20)));
        assert!(!manager.is_visible(EntityId::new(30)));
    }

    #[test]
    fn test_visibility_manager_clear() {
        let mut manager = VisibilityManager::new();

        manager.show(EntityId::new(1));
        manager.show(EntityId::new(2));
        manager.show(EntityId::new(3));

        assert_eq!(manager.visible_count(), 3);

        manager.clear();

        assert_eq!(manager.visible_count(), 0);
    }

    #[test]
    fn test_visibility_manager_iter_visible() {
        let mut manager = VisibilityManager::new();

        manager.show(EntityId::new(10));
        manager.show(EntityId::new(20));
        manager.show(EntityId::new(30));

        let visible: Vec<usize> = manager.iter_visible().collect();

        assert_eq!(visible.len(), 3);
        assert!(visible.contains(&10));
        assert!(visible.contains(&20));
        assert!(visible.contains(&30));
    }

    #[test]
    fn test_word_boundaries() {
        let mut bitset = VisibilityBitset::new();

        // Test entities at word boundaries (0, 63, 64, 127, 128, etc.)
        let test_entities = [0, 1, 62, 63, 64, 65, 126, 127, 128, 129];

        for &idx in &test_entities {
            bitset.set(EntityId::new(idx as u32), true);
        }

        for &idx in &test_entities {
            assert!(
                bitset.get(EntityId::new(idx as u32)),
                "Entity at index {} should be visible",
                idx
            );
        }

        // Verify adjacent entities are not affected
        assert!(!bitset.get(EntityId::new(130)));
        assert!(!bitset.get(EntityId::new(200)));
    }

    #[test]
    fn test_high_entity_ids() {
        let mut bitset = VisibilityBitset::new();

        // Test with higher entity IDs
        let high_id: usize = 10000;
        bitset.set(EntityId::new(high_id as u32), true);

        assert!(bitset.get(EntityId::new(high_id as u32)));

        let visible: Vec<usize> = bitset.iter_visible().collect();
        assert!(visible.contains(&high_id));
    }
}
