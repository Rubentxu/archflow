// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Batch Selection Actuator with Delta Mask
//
// High-performance batch selection using bitmask operations for memory efficiency.
// Memory: 12.5KB per 100k entities (vs ~3MB HashSet)
//
// Performance Characteristics:
// - O(1) toggle per entity
// - O(n/8) memory for selection state
// - XOR-based undo/redo (same operation for undo)
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::vec;
use alloc::vec::Vec;
use archflow_core::EntityId;
use archflow_engine::EntityStore;

/// Bitmask for efficient delta tracking during selection operations
///
/// Stores selection state as 1 bit per entity, enabling:
/// - 12.5KB memory for 100k entities (vs ~3MB HashSet)
/// - O(1) toggle operations
/// - XOR-based undo/redo (same operation for execute and undo)
///
/// # Examples
///
/// ```
/// use archflow_logic::actuators::batch_select::DeltaMask;
///
/// let mut mask = DeltaMask::new(100);
///
/// // Toggle entity 42
/// mask.toggle(42);
/// assert!(mask.is_set(42));
///
/// // Toggle again (undo)
/// mask.toggle(42);
/// assert!(!mask.is_set(42));
/// ```
#[derive(Clone, Debug)]
pub struct DeltaMask {
    /// Bits stored as bytes (8 bits per byte)
    bits: Vec<u8>,
    /// Total number of bits (entities) this mask can hold
    capacity: usize,
}

impl DeltaMask {
    /// Creates a new DeltaMask with the given capacity
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of entities (bits) to support
    ///
    /// # Memory
    ///
    /// Uses `(capacity + 7) / 8` bytes of storage
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_logic::actuators::batch_select::DeltaMask;
    ///
    /// let mask = DeltaMask::new(100_000);
    /// // Uses 12,500 bytes (12.5KB)
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let bytes = (capacity + 7) / 8;
        Self {
            bits: vec![0u8; bytes],
            capacity,
        }
    }

    /// Creates a DeltaMask from a list of entity indices
    ///
    /// # Arguments
    ///
    /// * `entities` - List of entity indices to set in the mask
    /// * `capacity` - Maximum capacity (usually MAX_ENTITIES)
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_logic::actuators::batch_select::DeltaMask;
    ///
    /// let mask = DeltaMask::from_entities(&[1, 2, 3, 42, 100], 1000);
    /// assert!(mask.is_set(1));
    /// assert!(mask.is_set(42));
    /// assert!(!mask.is_set(50));
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn from_entities(entities: &[usize], capacity: usize) -> Self {
        let mut mask = Self::new(capacity);
        for &idx in entities {
            if idx < capacity {
                mask.toggle(idx);
            }
        }
        mask
    }

    /// Toggles the bit at the given index
    ///
    /// # Arguments
    ///
    /// * `idx` - Entity index to toggle
    ///
    /// # Panics
    ///
    /// Panics if `idx >= capacity`
    #[inline(always)]
    pub fn toggle(&mut self, idx: usize) {
        assert!(idx < self.capacity, "Index out of bounds");
        let byte_idx = idx / 8;
        let bit_idx = idx % 8;
        self.bits[byte_idx] ^= 1 << bit_idx;
    }

    /// Checks if the bit at the given index is set
    ///
    /// # Arguments
    ///
    /// * `idx` - Entity index to check
    ///
    /// # Returns
    ///
    /// `true` if the bit is set (1), `false` otherwise (0)
    ///
    /// # Panics
    ///
    /// Panics if `idx >= capacity`
    #[inline(always)]
    #[must_use]
    pub fn is_set(&self, idx: usize) -> bool {
        assert!(idx < self.capacity, "Index out of bounds");
        let byte_idx = idx / 8;
        let bit_idx = idx % 8;
        (self.bits[byte_idx] >> bit_idx) & 1 == 1
    }

    /// Returns the number of bits set to 1
    ///
    /// # Returns
    ///
    /// Count of set bits in the mask
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_logic::actuators::batch_select::DeltaMask;
    ///
    /// let mut mask = DeltaMask::new(100);
    /// assert_eq!(mask.count_ones(), 0);
    ///
    /// mask.toggle(1);
    /// mask.toggle(42);
    /// mask.toggle(7);
    /// assert_eq!(mask.count_ones(), 3);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn count_ones(&self) -> usize {
        self.bits
            .iter()
            .fold(0, |acc, &byte| acc + byte.count_ones() as usize)
    }

    /// Returns the capacity of the mask
    ///
    /// # Returns
    ///
    /// Maximum number of bits this mask can hold
    #[inline(always)]
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the number of bytes used by this mask
    ///
    /// # Returns
    ///
    /// Size in bytes: `(capacity + 7) / 8`
    #[inline(always)]
    #[must_use]
    pub fn len_bytes(&self) -> usize {
        self.bits.len()
    }

    /// Checks if the mask is empty (no bits set)
    ///
    /// # Returns
    ///
    /// `true` if no bits are set
    #[inline(always)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.count_ones() == 0
    }

    /// Clears all bits in the mask
    ///
    /// Resets to zero state without deallocating.
    #[inline(always)]
    pub fn clear(&mut self) {
        for byte in &mut self.bits {
            *byte = 0;
        }
    }

    /// Creates a copy of this mask
    ///
    /// # Returns
    ///
    /// New DeltaMask with identical state
    #[inline(always)]
    #[must_use]
    pub fn clone(&self) -> Self {
        Self {
            bits: self.bits.clone(),
            capacity: self.capacity,
        }
    }
}

/// Actuator for batch selection operations using DeltaMask
///
/// Provides efficient selection/deselection of multiple entities:
/// - Uses DeltaMask for memory-efficient state tracking (12.5KB/100k)
/// - Supports XOR toggle semantics for intuitive multi-select
/// - Implements undo/redo via same XOR operation
///
/// # Selection Modes
///
/// - **Single**: Clear previous selection, select new entities
/// - **Multi**: Toggle entities (add to or remove from selection)
/// - **Replace**: Same as Single (clear and select new)
///
/// # Memory Comparison
///
/// | Implementation | Memory/100k Entities |
/// |----------------|----------------------|
/// | HashSet<EntityId> | ~3,000 KB |
/// | Vec<EntityId> | ~800 KB |
/// | **DeltaMask** | **12.5 KB** |
///
/// # Examples
///
/// ```
/// use archflow_core::Vec2;
/// use archflow_engine::EntityStore;
/// use archflow_logic::actuators::batch_select::{BatchSelectActuator, SelectMode};
///
/// let mut store = EntityStore::new();
/// let e1 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(20.0, 20.0));
/// let e2 = store.spawn(Vec2::new(150.0, 50.0), Vec2::new(20.0, 20.0));
/// let e3 = store.spawn(Vec2::new(250.0, 50.0), Vec2::new(20.0, 20.0));
///
/// let mut actuator = BatchSelectActuator::new();
///
/// // Multi-select two entities
/// let entities = vec![e1, e2];
/// actuator.execute(&mut store, &entities, SelectMode::Multi);
///
/// assert!(actuator.is_selected(e1));
/// assert!(actuator.is_selected(e2));
/// assert!(!actuator.is_selected(e3));
///
/// // Undo
/// actuator.undo(&mut store);
/// assert!(!actuator.is_selected(e1));
/// assert!(!actuator.is_selected(e2));
/// ```
pub struct BatchSelectActuator {
    /// Current selection state as bitmask
    selection_mask: DeltaMask,

    /// Delta mask for the last operation (for undo)
    last_delta: Option<DeltaMask>,

    /// History of deltas for redo support
    redo_stack: Vec<DeltaMask>,
}

impl BatchSelectActuator {
    /// Creates a new BatchSelectActuator
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_logic::actuators::batch_select::BatchSelectActuator;
    ///
    /// let actuator = BatchSelectActuator::new();
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            selection_mask: DeltaMask::new(archflow_engine::MAX_ENTITIES),
            last_delta: None,
            redo_stack: Vec::new(),
        }
    }

    /// Executes a batch selection operation
    ///
    /// # Arguments
    ///
    /// * `store` - EntityStore to update
    /// * `entities` - List of entities to select/deselect
    /// * `mode` - Selection mode (Single, Multi, Replace)
    ///
    /// # Returns
    ///
    /// Number of entities whose selection state changed
    ///
    /// # Panics
    ///
    /// Panics if entity index >= MAX_ENTITIES
    #[inline(never)]
    pub fn execute(
        &mut self,
        store: &mut EntityStore,
        entities: &[EntityId],
        mode: SelectMode,
    ) -> usize {
        // Clear redo stack on new operation
        self.redo_stack.clear();

        // Handle Single/Replace mode: clear previous selection first
        let mut changes = 0;
        if mode == SelectMode::Single || mode == SelectMode::Replace {
            // Build delta by clearing all currently selected entities
            let mut delta = DeltaMask::new(archflow_engine::MAX_ENTITIES);
            for idx in 0..self.selection_mask.capacity() {
                if self.selection_mask.is_set(idx) {
                    delta.toggle(idx);
                    store.set_selected(idx, false);
                    self.selection_mask.toggle(idx);
                    changes += 1;
                }
            }
            self.last_delta = Some(delta);
        }

        // Build delta mask for new selection
        let mut delta = if self.last_delta.is_some() && changes > 0 {
            self.last_delta.take().unwrap()
        } else {
            DeltaMask::new(archflow_engine::MAX_ENTITIES)
        };

        // Toggle or set entities based on mode
        for entity in entities {
            let idx = entity.index().0 as usize;

            if mode == SelectMode::Multi {
                // Toggle: create delta for change
                if !delta.is_set(idx) {
                    delta.toggle(idx);
                    // Apply toggle to selection state
                    let currently_selected = self.selection_mask.is_set(idx);
                    store.set_selected(idx, !currently_selected);
                    self.selection_mask.toggle(idx);
                    changes += 1;
                }
            } else {
                // Single/Replace: just set
                if !self.selection_mask.is_set(idx) {
                    delta.toggle(idx);
                    store.set_selected(idx, true);
                    self.selection_mask.toggle(idx);
                    changes += 1;
                }
            }
        }

        self.last_delta = Some(delta);
        changes
    }

    /// Undoes the last batch selection operation
    ///
    /// Uses XOR toggle semantics: applying the same delta again
    /// restores the previous state.
    ///
    /// # Arguments
    ///
    /// * `store` - EntityStore to update
    ///
    /// # Returns
    ///
    /// `true` if undo was successful, `false` if nothing to undo
    #[inline(never)]
    pub fn undo(&mut self, store: &mut EntityStore) -> bool {
        if let Some(ref delta) = self.last_delta {
            // Apply same delta (XOR) to restore previous state
            for idx in 0..delta.capacity() {
                if delta.is_set(idx) {
                    self.selection_mask.toggle(idx);
                    store.set_selected(idx, self.selection_mask.is_set(idx));
                }
            }

            // Push to redo stack
            self.redo_stack.push(delta.clone());
            self.last_delta = None;
            true
        } else {
            false
        }
    }

    /// Redoes the last undone operation
    ///
    /// # Arguments
    ///
    /// * `store` - EntityStore to update
    ///
    /// # Returns
    ///
    /// `true` if redo was successful, `false` if nothing to redo
    #[inline(never)]
    pub fn redo(&mut self, store: &mut EntityStore) -> bool {
        if let Some(delta) = self.redo_stack.pop() {
            // Apply same delta again
            for idx in 0..delta.capacity() {
                if delta.is_set(idx) {
                    self.selection_mask.toggle(idx);
                    store.set_selected(idx, self.selection_mask.is_set(idx));
                }
            }

            self.last_delta = Some(delta);
            true
        } else {
            false
        }
    }

    /// Checks if an entity is currently selected
    ///
    /// # Arguments
    ///
    /// * `entity` - Entity to check
    ///
    /// # Returns
    ///
    /// `true` if the entity is selected
    #[inline(always)]
    #[must_use]
    pub fn is_selected(&self, entity: EntityId) -> bool {
        let idx = entity.index().0 as usize;
        idx < self.selection_mask.capacity() && self.selection_mask.is_set(idx)
    }

    /// Returns the number of selected entities
    ///
    /// # Returns
    ///
    /// Count of bits set in the selection mask
    #[inline(always)]
    #[must_use]
    pub fn selection_count(&self) -> usize {
        self.selection_mask.count_ones()
    }

    /// Returns all currently selected entities
    ///
    /// # Returns
    ///
    /// Vector of selected EntityIds
    #[inline(always)]
    #[must_use]
    pub fn current_selection(&self) -> Vec<EntityId> {
        let mut result = Vec::new();
        for idx in 0..self.selection_mask.capacity() {
            if self.selection_mask.is_set(idx) {
                result.push(EntityId::new(idx as u32));
            }
        }
        result
    }

    /// Clears all selections
    ///
    /// # Arguments
    ///
    /// * `store` - EntityStore to update
    ///
    /// # Returns
    ///
    /// Number of entities that were deselected
    #[inline(never)]
    pub fn clear(&mut self, store: &mut EntityStore) -> usize {
        let count = self.selection_count();

        for idx in 0..self.selection_mask.capacity() {
            if self.selection_mask.is_set(idx) {
                store.set_selected(idx, false);
            }
        }

        self.selection_mask.clear();
        self.last_delta = None;
        self.redo_stack.clear();

        count
    }

    /// Checks if undo is available
    ///
    /// # Returns
    ///
    /// `true` if there is an operation to undo
    #[inline(always)]
    #[must_use]
    pub fn can_undo(&self) -> bool {
        self.last_delta.is_some()
    }

    /// Checks if redo is available
    ///
    /// # Returns
    ///
    /// `true` if there is an operation to redo
    #[inline(always)]
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

impl Default for BatchSelectActuator {
    fn default() -> Self {
        Self::new()
    }
}

/// Selection mode enum (shared with SelectActuator for compatibility)
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectMode {
    /// Select only this entity, deselect all others
    Single = 0,
    /// Add to selection (toggle if already selected)
    Multi = 1,
    /// Clear all and select only this entity
    Replace = 2,
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_core::Vec2;

    #[test]
    fn test_delta_mask_toggle() {
        let mut mask = DeltaMask::new(100);

        assert!(!mask.is_set(42));
        mask.toggle(42);
        assert!(mask.is_set(42));
        mask.toggle(42);
        assert!(!mask.is_set(42));
    }

    #[test]
    fn test_delta_mask_count_ones() {
        let mut mask = DeltaMask::new(100);

        assert_eq!(mask.count_ones(), 0);

        mask.toggle(1);
        mask.toggle(42);
        mask.toggle(7);

        assert_eq!(mask.count_ones(), 3);
    }

    #[test]
    fn test_delta_mask_from_entities() {
        let mask = DeltaMask::from_entities(&[1, 2, 3, 42], 100);

        assert!(mask.is_set(1));
        assert!(mask.is_set(2));
        assert!(mask.is_set(3));
        assert!(mask.is_set(42));
        assert!(!mask.is_set(50));
    }

    #[test]
    fn test_delta_mask_memory_usage() {
        // 100k entities = 100,000 bits = 12,500 bytes
        let mask = DeltaMask::new(100_000);
        assert_eq!(mask.len_bytes(), 12_500); // (100000 + 7) / 8 = 12500
    }

    #[test]
    fn test_batch_select_actuator_basic() {
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(20.0, 20.0));
        let e2 = store.spawn(Vec2::new(150.0, 50.0), Vec2::new(20.0, 20.0));
        let e3 = store.spawn(Vec2::new(250.0, 50.0), Vec2::new(20.0, 20.0));

        let mut actuator = BatchSelectActuator::new();

        // Multi-select two entities
        let entities = vec![e1, e2];
        actuator.execute(&mut store, &entities, SelectMode::Multi);

        assert!(actuator.is_selected(e1));
        assert!(actuator.is_selected(e2));
        assert!(!actuator.is_selected(e3));
        assert_eq!(actuator.selection_count(), 2);
    }

    #[test]
    fn test_batch_select_actuator_undo() {
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(20.0, 20.0));
        let e2 = store.spawn(Vec2::new(150.0, 50.0), Vec2::new(20.0, 20.0));

        let mut actuator = BatchSelectActuator::new();

        // Select entities
        actuator.execute(&mut store, &vec![e1, e2], SelectMode::Multi);
        assert!(actuator.is_selected(e1));
        assert!(actuator.can_undo());

        // Undo
        actuator.undo(&mut store);
        assert!(!actuator.is_selected(e1));
        assert!(!actuator.is_selected(e2));
        assert!(!actuator.can_undo());
    }

    #[test]
    fn test_batch_select_actuator_single_mode() {
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(20.0, 20.0));
        let e2 = store.spawn(Vec2::new(150.0, 50.0), Vec2::new(20.0, 20.0));
        let e3 = store.spawn(Vec2::new(250.0, 50.0), Vec2::new(20.0, 20.0));

        let mut actuator = BatchSelectActuator::new();

        // Select e1
        actuator.execute(&mut store, &vec![e1], SelectMode::Single);
        assert!(actuator.is_selected(e1));
        assert_eq!(actuator.selection_count(), 1);

        // Select e2 in Single mode (should clear e1)
        actuator.execute(&mut store, &vec![e2], SelectMode::Single);
        assert!(!actuator.is_selected(e1));
        assert!(actuator.is_selected(e2));
        assert_eq!(actuator.selection_count(), 1);
    }

    #[test]
    fn test_batch_select_actuator_clear() {
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(20.0, 20.0));
        let e2 = store.spawn(Vec2::new(150.0, 50.0), Vec2::new(20.0, 20.0));

        let mut actuator = BatchSelectActuator::new();

        actuator.execute(&mut store, &vec![e1, e2], SelectMode::Multi);
        assert_eq!(actuator.selection_count(), 2);

        let cleared = actuator.clear(&mut store);
        assert_eq!(cleared, 2);
        assert_eq!(actuator.selection_count(), 0);
        assert!(!actuator.is_selected(e1));
        assert!(!actuator.is_selected(e2));
    }
}
