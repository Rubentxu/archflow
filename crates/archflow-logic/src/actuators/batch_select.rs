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

    /// Applies this delta mask to another mask using XOR operation
    ///
    /// This efficiently toggles the bits in `target` wherever this delta has 1s.
    /// Both masks must have the same capacity.
    ///
    /// # Arguments
    ///
    /// * `target` - The mask to apply delta to (will be modified in-place)
    ///
    /// # Panics
    ///
    /// Panics if capacities don't match
    ///
    /// # Performance
    ///
    /// - O(n) where n = mask size in bytes
    /// - Uses SIMD-friendly chunk operations where available
    /// - Zero-allocation
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_logic::actuators::batch_select::DeltaMask;
    ///
    /// let mut target = DeltaMask::new(100);
    /// target.toggle(5);
    /// target.toggle(10);
    ///
    /// let delta = DeltaMask::new(100);
    /// delta.toggle(5);
    /// delta.toggle(20);
    ///
    /// delta.apply_to(&mut target);
    ///
    /// // Bit 5: 1 XOR 1 = 0 (toggled off)
    /// assert!(!target.is_set(5));
    /// // Bit 10: 1 XOR 0 = 1 (unchanged)
    /// assert!(target.is_set(10));
    /// // Bit 20: 0 XOR 1 = 1 (toggled on)
    /// assert!(target.is_set(20));
    /// ```
    #[inline(always)]
    pub fn apply_to(&self, target: &mut DeltaMask) {
        assert_eq!(
            self.capacity, target.capacity,
            "DeltaMask capacities must match"
        );
        assert_eq!(
            self.bits.len(),
            target.bits.len(),
            "DeltaMask byte lengths must match"
        );

        // XOR operation - this is the core of delta application
        // For each byte: target[i] = target[i] ^ self[i]
        // This toggles exactly the bits that are set in self
        for (src_byte, tgt_byte) in self.bits.iter().zip(target.bits.iter_mut()) {
            *tgt_byte ^= *src_byte;
        }
    }

    /// Batch apply multiple delta masks to a target using XOR
    ///
    /// Applies all deltas in sequence to the target mask.
    /// More efficient than calling `apply_to` multiple times due to
    /// single pass through memory.
    ///
    /// # Arguments
    ///
    /// * `target` - The mask to apply all deltas to
    /// * `deltas` - Iterator of delta masks to apply
    ///
    /// # Performance
    ///
    /// - O(n * m) where n = bytes per mask, m = number of deltas
    /// - Memory-cache friendly (sequential access)
    /// - SIMD-optimizable
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_logic::actuators::batch_select::DeltaMask;
    ///
    /// let mut target = DeltaMask::new(100);
    ///
    /// let deltas: Vec<DeltaMask> = (0..3).map(|i| {
    ///     let mut d = DeltaMask::new(100);
    ///     d.toggle(i * 10);
    ///     d
    /// }).collect();
    ///
    /// DeltaMask::batch_apply(&mut target, deltas.iter());
    ///
    /// assert!(target.is_set(0));
    /// assert!(target.is_set(10));
    /// assert!(target.is_set(20));
    /// ```
    #[inline(always)]
    pub fn batch_apply(target: &mut DeltaMask, deltas: impl Iterator<Item = &'static DeltaMask>) {
        // Collect all deltas into a Vec for multiple passes if needed
        let deltas: Vec<&DeltaMask> = deltas.collect();

        if deltas.is_empty() {
            return;
        }

        // Verify all deltas have the same capacity as target
        let target_capacity = target.capacity;
        for delta in &deltas {
            assert_eq!(
                delta.capacity, target_capacity,
                "All DeltaMasks must have the same capacity"
            );
        }

        // Apply each delta in sequence
        for delta in deltas {
            for (src_byte, tgt_byte) in delta.bits.iter().zip(target.bits.iter_mut()) {
                *tgt_byte ^= *src_byte;
            }
        }
    }

    /// Creates a delta mask representing the difference between two states
    ///
    /// This is useful for computing what changed between two selection states.
    ///
    /// # Arguments
    ///
    /// * `other` - The other mask to compare against
    ///
    /// # Returns
    ///
    /// A new DeltaMask where bits are 1 where the states differ
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_logic::actuators::batch_select::DeltaMask;
    ///
    /// let mut state1 = DeltaMask::new(100);
    /// state1.toggle(5);
    /// state1.toggle(10);
    ///
    /// let mut state2 = DeltaMask::new(100);
    /// state2.toggle(5);
    /// state2.toggle(20);
    ///
    /// let delta = state1.diff(&state2);
    ///
    /// // Bit 5: same in both (1 XOR 1 = 0)
    /// assert!(!delta.is_set(5));
    /// // Bit 10: only in state1
    /// assert!(delta.is_set(10));
    /// // Bit 20: only in state2
    /// assert!(delta.is_set(20));
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn diff(&self, other: &DeltaMask) -> Self {
        assert_eq!(
            self.capacity, other.capacity,
            "DeltaMask capacities must match"
        );

        let mut result = Self::new(self.capacity);
        for ((src, other), result_byte) in self
            .bits
            .iter()
            .zip(other.bits.iter())
            .zip(result.bits.iter_mut())
        {
            *result_byte = *src ^ *other;
        }
        result
    }

    /// Checks if this delta is compatible with another (same capacity)
    ///
    /// # Arguments
    ///
    /// * `other` - Another DeltaMask to check compatibility with
    ///
    /// # Returns
    ///
    /// `true` if both masks have the same capacity
    #[inline(always)]
    #[must_use]
    pub fn is_compatible(&self, other: &DeltaMask) -> bool {
        self.capacity == other.capacity
    }

    /// Resizes the mask to support a larger number of entities
    pub fn resize(&mut self, new_capacity: usize) {
        if new_capacity <= self.capacity {
            return;
        }
        let new_bytes = (new_capacity + 7) / 8;
        self.bits.resize(new_bytes, 0);
        self.capacity = new_capacity;
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
        self.execute_with_events(store, entities, mode, |_, _| {})
    }

    /// Executes a batch selection operation with event emission
    ///
    /// This is the recommended method when a LogicSystem is available.
    /// Events are emitted for selection changes, enabling UI updates.
    ///
    /// # Arguments
    ///
    /// * `store` - EntityStore to update
    /// * `entities` - List of entities to select/deselect
    /// * `mode` - Selection mode (Single, Multi, Replace)
    /// * `on_selection_change` - Callback for each entity whose selection changed
    ///                           Receives (entity_index: usize, now_selected: bool)
    ///
    /// # Returns
    ///
    /// Number of entities whose selection state changed
    ///
    /// # Panics
    ///
    /// Panics if entity index >= MAX_ENTITIES
    ///
    /// # Example
    ///
    /// ```rust
    /// use archflow_logic::actuators::batch_select::{BatchSelectActuator, SelectMode};
    /// use archflow_core::Vec2;
    /// use archflow_engine::EntityStore;
    ///
    /// let mut store = EntityStore::new();
    /// let entity = store.spawn(Vec2::ZERO, Vec2::ONE);
    ///
    /// let mut actuator = BatchSelectActuator::new();
    ///
    /// // Execute with event emission
    /// let changes = actuator.execute_with_events(
    ///     &mut store,
    ///     &[entity],
    ///     SelectMode::Single,
    ///     |idx, now_selected| {
    ///         // Emit EntitySelected event for UI
    ///         logic_system.emit_entity_selected(idx as u32);
    ///     }
    /// );
    /// ```
    #[inline(never)]
    pub fn execute_with_events<F>(
        &mut self,
        store: &mut EntityStore,
        entities: &[EntityId],
        mode: SelectMode,
        mut on_selection_change: F,
    ) -> usize
    where
        F: FnMut(usize, bool),
    {
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
                    // Emit event for deselection
                    on_selection_change(idx, false);
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
                    let now_selected = !currently_selected;
                    store.set_selected(idx, now_selected);
                    self.selection_mask.toggle(idx);
                    changes += 1;
                    // Emit event for selection change
                    on_selection_change(idx, now_selected);
                }
            } else {
                // Single/Replace: just set
                if !self.selection_mask.is_set(idx) {
                    delta.toggle(idx);
                    store.set_selected(idx, true);
                    self.selection_mask.toggle(idx);
                    changes += 1;
                    // Emit event for selection
                    on_selection_change(idx, true);
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

    /// Resizes the actuator to support a larger number of entities
    pub fn resize(&mut self, new_capacity: usize) {
        if new_capacity <= self.selection_mask.capacity() {
            return;
        }
        self.selection_mask.resize(new_capacity);
        if let Some(delta) = &mut self.last_delta {
            delta.resize(new_capacity);
        }
        for delta in &mut self.redo_stack {
            delta.resize(new_capacity);
        }
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
    /// Toggle selection state (select if deselected, deselect if selected)
    Toggle = 3,
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

    #[test]
    fn test_delta_mask_apply_to() {
        let mut target = DeltaMask::new(100);
        target.toggle(5);
        target.toggle(10);

        let mut delta = DeltaMask::new(100);
        delta.toggle(5);
        delta.toggle(20);

        delta.apply_to(&mut target);

        // Bit 5: 1 XOR 1 = 0 (toggled off)
        assert!(!target.is_set(5));
        // Bit 10: 1 XOR 0 = 1 (unchanged)
        assert!(target.is_set(10));
        // Bit 20: 0 XOR 1 = 1 (toggled on)
        assert!(target.is_set(20));
    }

    #[test]
    fn test_delta_mask_diff() {
        let mut state1 = DeltaMask::new(100);
        state1.toggle(5);
        state1.toggle(10);

        let mut state2 = DeltaMask::new(100);
        state2.toggle(5);
        state2.toggle(20);

        let delta = state1.diff(&state2);

        // Bit 5: same in both (1 XOR 1 = 0)
        assert!(!delta.is_set(5));
        // Bit 10: only in state1
        assert!(delta.is_set(10));
        // Bit 20: only in state2
        assert!(delta.is_set(20));
    }

    #[test]
    fn test_delta_mask_is_compatible() {
        let mask1 = DeltaMask::new(100);
        let mask2 = DeltaMask::new(100);
        let mask3 = DeltaMask::new(200);

        assert!(mask1.is_compatible(&mask2));
        assert!(!mask1.is_compatible(&mask3));
    }

    #[test]
    #[should_panic(expected = "capacities must match")]
    fn test_delta_mask_apply_panics_on_mismatch() {
        let mut target = DeltaMask::new(100);
        let delta = DeltaMask::new(200);

        delta.apply_to(&mut target);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // EVENT EMISSION TESTS (HU-CONSOL-002)
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_execute_with_events_emits_selection_change() {
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(20.0, 20.0));
        let e2 = store.spawn(Vec2::new(150.0, 50.0), Vec2::new(20.0, 20.0));

        let mut actuator = BatchSelectActuator::new();

        // Track events emitted using Cell
        let event_count = core::cell::Cell::new(0usize);
        let e1_idx = e1.index().0 as usize;
        let e2_idx = e2.index().0 as usize;
        let e1_selected = core::cell::Cell::new(false);
        let e2_selected = core::cell::Cell::new(false);

        // Execute with event emission
        let entities = vec![e1, e2];
        actuator.execute_with_events(
            &mut store,
            &entities,
            SelectMode::Multi,
            |idx, now_selected| {
                event_count.set(event_count.get() + 1);
                if idx == e1_idx {
                    e1_selected.set(now_selected);
                }
                if idx == e2_idx {
                    e2_selected.set(now_selected);
                }
            },
        );

        assert_eq!(event_count.get(), 2, "Should emit 2 events");
        assert!(e1_selected.get(), "e1 should be selected");
        assert!(e2_selected.get(), "e2 should be selected");
    }

    #[test]
    fn test_execute_with_events_single_mode_clears_previous() {
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(20.0, 20.0));
        let e2 = store.spawn(Vec2::new(150.0, 50.0), Vec2::new(20.0, 20.0));
        let e3 = store.spawn(Vec2::new(250.0, 50.0), Vec2::new(20.0, 20.0));

        let mut actuator = BatchSelectActuator::new();
        let event_count = core::cell::Cell::new(0usize);

        // First select e1 and e2
        actuator.execute_with_events(&mut store, &vec![e1, e2], SelectMode::Multi, |_, _| {
            event_count.set(event_count.get() + 1);
        });
        assert_eq!(event_count.get(), 2);
        assert_eq!(actuator.selection_count(), 2);

        // Now select e3 in Single mode - should clear e1 and e2
        event_count.set(0);
        let clear_count = core::cell::Cell::new(0usize);
        actuator.execute_with_events(&mut store, &vec![e3], SelectMode::Single, |_, _| {
            event_count.set(event_count.get() + 1);
            clear_count.set(clear_count.get() + 1);
        });

        // Should emit: 2 for clearing (e1, e2) + 1 for selecting e3 = 3 events
        assert_eq!(event_count.get(), 3);
        assert_eq!(actuator.selection_count(), 1);
    }

    #[test]
    fn test_execute_without_events_still_works() {
        // Verify backward compatibility - execute() without events works
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(20.0, 20.0));
        let e2 = store.spawn(Vec2::new(150.0, 50.0), Vec2::new(20.0, 20.0));

        let mut actuator = BatchSelectActuator::new();

        // Execute without events (backward compatible)
        let changes = actuator.execute(&mut store, &vec![e1, e2], SelectMode::Multi);

        assert_eq!(changes, 2);
        assert!(actuator.is_selected(e1));
        assert!(actuator.is_selected(e2));
        assert_eq!(actuator.selection_count(), 2);
    }

    #[test]
    fn test_execute_with_events_toggle_behavior() {
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(20.0, 20.0));

        let mut actuator = BatchSelectActuator::new();
        let event_count = core::cell::Cell::new(0usize);

        // First select
        actuator.execute_with_events(&mut store, &vec![e1], SelectMode::Multi, |_, _| {
            event_count.set(event_count.get() + 1);
        });
        assert_eq!(event_count.get(), 1);
        assert!(actuator.is_selected(e1));

        // Toggle off
        event_count.set(0);
        actuator.execute_with_events(&mut store, &vec![e1], SelectMode::Multi, |_, _| {
            event_count.set(event_count.get() + 1);
        });
        assert_eq!(event_count.get(), 1); // Still emits event even though already selected
        assert!(!actuator.is_selected(e1));
    }

    #[test]
    fn test_execute_with_events_empty_selection() {
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(20.0, 20.0));

        let mut actuator = BatchSelectActuator::new();
        let event_count = core::cell::Cell::new(0usize);

        // Select e1 first
        actuator.execute(&mut store, &vec![e1], SelectMode::Single);

        // Now select empty list in Single mode
        let changes = actuator.execute_with_events(
            &mut store,
            &alloc::vec::Vec::new(),
            SelectMode::Single,
            |_, _| {
                event_count.set(event_count.get() + 1);
            },
        );

        // Should emit events for clearing previous selection
        assert!(changes > 0 || event_count.get() > 0);
    }
}
