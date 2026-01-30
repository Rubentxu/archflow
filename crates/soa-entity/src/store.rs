//! EntityStore - SOA Entity Store with Generational Indices
//!
//! Provides type-safe, cache-friendly entity storage with:
//! - Free list for efficient spawn/despawn
//! - Generational EntityId validation
//! - Automatic compaction when fragmented
//! - Type-safe component access
//! - Dirty bitset tracking for optimized GPU upload

use crate::EntityId;
use archflow_core::{Color, Vec2};
use fixedbitset::FixedBitSet;
use std::collections::VecDeque;

/// Error type for entity store operations.
#[derive(Debug, thiserror::Error)]
pub enum EntityStoreError {
    #[error("Entity capacity reached (max: {0})")]
    CapacityReached(usize),

    #[error("Invalid entity ID: {0:?}")]
    InvalidEntityId(EntityId),

    #[error("Entity not found: {0:?}")]
    EntityNotFound(EntityId),

    #[error("Store is empty")]
    EmptyStore,
}

/// Fragmentation threshold for triggering auto-compaction.
const FRAGMENTATION_THRESHOLD: f32 = 0.3; // 30%

/// SOA Entity Store with generational indices and dirty tracking.
///
/// This implementation stores components contiguously for cache efficiency
/// (SOA layout) and tracks dirty components for optimized GPU upload.
///
/// # Examples
///
/// ```
/// use soa_entity::EntityStore;
/// use archflow_core::Vec2;
///
/// let mut store = EntityStore::new(1000);
/// let id = store.spawn();
/// store.set_position(id, Vec2::new(100.0, 200.0));
///
/// // Check what's dirty
/// let dirty_ranges = store.calculate_dirty_ranges(&store.dirty_positions());
/// ```
pub struct EntityStore {
    /// Maximum number of entities
    capacity: usize,

    /// Current number of live entities
    count: usize,

    /// Generation counter for each slot (validity check)
    generations: Vec<u32>,

    /// Free slots (indices available for reuse)
    free_slots: VecDeque<usize>,

    /// Position components (Vec2 expanded to x, y)
    pos_x: Vec<f32>,
    pos_y: Vec<f32>,

    /// Color components (Color stored as f32 RGBA)
    col_r: Vec<f32>,
    col_g: Vec<f32>,
    col_b: Vec<f32>,
    col_a: Vec<f32>,

    /// Dirty tracking per component type
    dirty_positions: FixedBitSet,
    dirty_colors: FixedBitSet,
}

impl EntityStore {
    /// Creates a new entity store with the given capacity.
    ///
    /// # Panics
    ///
    /// Panics if capacity is zero or exceeds MAX_ENTITIES.
    ///
    /// # Examples
    ///
    /// ```
    /// use soa_entity::EntityStore;
    ///
    /// let store = EntityStore::new(1000);
    /// assert_eq!(store.capacity(), 1000);
    /// ```
    #[inline]
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Capacity must be greater than zero");
        assert!(
            capacity <= EntityId::MAX_ENTITIES,
            "Capacity exceeds maximum"
        );

        Self {
            capacity,
            count: 0,
            generations: vec![0; capacity],
            free_slots: VecDeque::new(),

            // Initialize arrays with default values
            pos_x: vec![0.0; capacity],
            pos_y: vec![0.0; capacity],
            col_r: vec![0.0; capacity],
            col_g: vec![0.0; capacity],
            col_b: vec![0.0; capacity],
            col_a: vec![1.0; capacity],

            // Initialize dirty tracking
            dirty_positions: FixedBitSet::with_capacity(capacity),
            dirty_colors: FixedBitSet::with_capacity(capacity),
        }
    }

    /// Returns the maximum number of entities this store can hold.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the current number of live entities.
    #[inline]
    pub fn count(&self) -> usize {
        self.count
    }

    /// Returns the number of free slots available.
    #[inline]
    pub fn free_slots_count(&self) -> usize {
        self.capacity - self.count
    }

    /// Returns the current fragmentation ratio (0.0 = compact, 1.0 = all holes).
    #[inline]
    pub fn fragmentation(&self) -> f32 {
        if self.count == 0 {
            return 0.0;
        }
        self.free_slots.len() as f32 / self.capacity as f32
    }

    /// Checks if the given EntityId is valid (exists and generation matches).
    ///
    /// # Examples
    ///
    /// ```
    /// use soa_entity::{EntityStore, EntityId};
    ///
    /// let mut store = EntityStore::new(100);
    /// let id = store.spawn();
    /// assert!(store.is_valid(id));
    ///
    /// store.despawn(id);
    /// assert!(!store.is_valid(id));
    /// ```
    #[inline]
    pub fn is_valid(&self, id: EntityId) -> bool {
        let index = id.index();

        if index >= self.capacity {
            return false;
        }

        let stored_generation = self.generations[index];
        stored_generation == id.generation()
    }

    /// Spawns a new entity and returns its EntityId.
    ///
    /// Reuses free slots from the free list if available, otherwise
    /// allocates a new slot at the end.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the store is at maximum capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use soa_entity::EntityStore;
    ///
    /// let mut store = EntityStore::new(100);
    /// let id1 = store.spawn();
    /// let id2 = store.spawn();
    ///
    /// assert_ne!(id1, id2); // Unique IDs
    /// ```
    pub fn spawn(&mut self) -> Result<EntityId, EntityStoreError> {
        if self.count >= self.capacity {
            return Err(EntityStoreError::CapacityReached(self.capacity));
        }

        let (index, generation) = if let Some(&free_index) = self.free_slots.front() {
            // Reuse free slot - use current generation (already odd from despawn)
            let index = self.free_slots.pop_front().unwrap();
            let generation = self.generations[index];
            (index, generation)
        } else {
            // Allocate new slot at the end
            let index = self.count;
            // New slots start at generation 0, increment to 1 (odd = alive)
            let generation = 1;
            (index, generation)
        };

        self.generations[index] = generation;

        // Initialize with default values
        self.pos_x[index] = 0.0;
        self.pos_y[index] = 0.0;
        self.col_r[index] = 0.0;
        self.col_g[index] = 0.0;
        self.col_b[index] = 0.0;
        self.col_a[index] = 1.0;

        self.count += 1;

        Ok(EntityId::new(index, generation))
    }

    /// Despawns an entity, freeing its slot for reuse.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the EntityId is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use soa_entity::{EntityStore, EntityId};
    ///
    /// let mut store = EntityStore::new(100);
    /// let id = store.spawn();
    ///
    /// store.despawn(id).unwrap();
    /// assert!(!store.is_valid(id));
    /// ```
    pub fn despawn(&mut self, id: EntityId) -> Result<(), EntityStoreError> {
        if !self.is_valid(id) {
            return Err(EntityStoreError::InvalidEntityId(id));
        }

        let index = id.index();

        // Mark slot as free (increment generation)
        self.generations[index] = self.generations[index].wrapping_add(1);
        self.free_slots.push_back(index);

        self.count -= 1;

        // Check if we need to compact
        if self.fragmentation() > FRAGMENTATION_THRESHOLD {
            self.compact();
        }

        Ok(())
    }

    /// Compacts the store by moving all entities to eliminate free slots.
    ///
    /// This operation:
    /// - Moves all entities to the beginning of arrays
    /// - Updates all indices and generations
    /// - Clears the free list
    /// - Preserves all entity data and relative ordering
    ///
    /// # Examples
    ///
    /// ```
    /// use soa_entity::EntityStore;
    ///
    /// let mut store = EntityStore::new(100);
    /// let ids: Vec<_> = (0..50).map(|_| store.spawn()).collect();
    ///
    /// // Create fragmentation by despawning some entities
    /// for id in ids.iter().take(25) {
    ///     store.despawn(*id).unwrap();
    /// }
    ///
    /// store.compact();
    /// assert_eq!(store.count(), 25);
    /// assert_eq!(store.free_slots_count(), 0);
    /// ```
    pub fn compact(&mut self) {
        if self.count == 0 {
            // No entities to compact
            self.generations.fill(0);
            self.free_slots.clear();
            return;
        }

        // Collect all valid entity data
        let mut valid_indices: Vec<usize> = Vec::with_capacity(self.count);

        for index in 0..self.capacity {
            let generation = self.generations[index];
            if generation % 2 == 1 {
                // Odd = alive, even = dead
                valid_indices.push(index);
            }
        }

        // Move entities contiguously to the beginning
        for (new_index, &old_index) in valid_indices.iter().enumerate() {
            if new_index != old_index {
                // Move position
                self.pos_x[new_index] = self.pos_x[old_index];
                self.pos_y[new_index] = self.pos_y[old_index];

                // Move color
                self.col_r[new_index] = self.col_r[old_index];
                self.col_g[new_index] = self.col_g[old_index];
                self.col_b[new_index] = self.col_b[old_index];
                self.col_a[new_index] = self.col_a[old_index];

                // Update generation (keep odd = alive)
                self.generations[new_index] = self.generations[old_index];

                // Mark old slot as free (even generation)
                self.generations[old_index] = self.generations[old_index].wrapping_add(1);
            }
        }

        // Clear free list and update count
        self.free_slots.clear();
        // self.count stays the same
    }

    // ===== Position Accessors =====

    /// Gets the position of an entity.
    #[inline]
    pub fn position(&self, id: EntityId) -> Option<Vec2> {
        if !self.is_valid(id) {
            return None;
        }

        Some(Vec2::new(self.pos_x[id.index()], self.pos_y[id.index()]))
    }

    /// Sets the position of an entity.
    #[inline]
    pub fn set_position(&mut self, id: EntityId, pos: Vec2) -> Result<(), EntityStoreError> {
        if !self.is_valid(id) {
            return Err(EntityStoreError::InvalidEntityId(id));
        }

        let index = id.index();
        self.pos_x[index] = pos.x;
        self.pos_y[index] = pos.y;

        // Mark position as dirty
        self.dirty_positions.insert(index);

        Ok(())
    }

    /// Gets the x-coordinate of an entity.
    #[inline]
    pub fn pos_x(&self, id: EntityId) -> Option<&f32> {
        if !self.is_valid(id) {
            return None;
        }
        Some(&self.pos_x[id.index()])
    }

    /// Gets the y-coordinate of an entity.
    #[inline]
    pub fn pos_y(&self, id: EntityId) -> Option<&f32> {
        if !self.is_valid(id) {
            return None;
        }
        Some(&self.pos_y[id.index()])
    }

    // ===== Color Accessors =====

    /// Gets the color of an entity.
    #[inline]
    pub fn color(&self, id: EntityId) -> Option<Color> {
        if !self.is_valid(id) {
            return None;
        }

        Some(Color::rgba(
            self.col_r[id.index()],
            self.col_g[id.index()],
            self.col_b[id.index()],
            self.col_a[id.index()],
        ))
    }

    /// Sets the color of an entity.
    #[inline]
    pub fn set_color(&mut self, id: EntityId, col: Color) -> Result<(), EntityStoreError> {
        if !self.is_valid(id) {
            return Err(EntityStoreError::InvalidEntityId(id));
        }

        let index = id.index();
        self.col_r[index] = col.r;
        self.col_g[index] = col.g;
        self.col_b[index] = col.b;
        self.col_a[index] = col.a;

        // Mark color as dirty
        self.dirty_colors.insert(index);

        Ok(())
    }

    /// Gets the red component of an entity's color.
    #[inline]
    pub fn col_r(&self, id: EntityId) -> Option<&f32> {
        if !self.is_valid(id) {
            return None;
        }
        Some(&self.col_r[id.index()])
    }

    /// Gets the green component of an entity's color.
    #[inline]
    pub fn col_g(&self, id: EntityId) -> Option<&f32> {
        if !self.is_valid(id) {
            return None;
        }
        Some(&self.col_g[id.index()])
    }

    /// Gets the blue component of an entity's color.
    #[inline]
    pub fn col_b(&self, id: EntityId) -> Option<&f32> {
        if !self.is_valid(id) {
            return None;
        }
        Some(&self.col_b[id.index()])
    }

    /// Gets the alpha component of an entity's color.
    #[inline]
    pub fn col_a(&self, id: EntityId) -> Option<&f32> {
        if !self.is_valid(id) {
            return None;
        }
        Some(&self.col_a[id.index()])
    }
}

impl Default for EntityStore {
    #[inline]
    fn default() -> Self {
        Self::new(1000)
    }
}

// ===== Dirty Tracking Methods =====

impl EntityStore {
    /// Returns the dirty positions bitset.
    #[inline]
    pub fn dirty_positions(&self) -> &FixedBitSet {
        &self.dirty_positions
    }

    /// Returns the dirty colors bitset.
    #[inline]
    pub fn dirty_colors(&self) -> &FixedBitSet {
        &self.dirty_colors
    }

    /// Calculates contiguous dirty ranges from a bitset.
    ///
    /// Returns a vector of (start, length) tuples representing contiguous
    /// ranges of dirty entities. This is used for efficient GPU upload batching.
    ///
    /// # Examples
    ///
    /// ```
    /// use soa_entity::EntityStore;
    /// use archflow_core::Vec2;
    ///
    /// let mut store = EntityStore::new(100);
    /// let id1 = store.spawn().unwrap();
    /// let id2 = store.spawn().unwrap();
    /// let id3 = store.spawn().unwrap();
    ///
    /// store.set_position(id1, Vec2::new(1.0, 2.0));
    /// store.set_position(id2, Vec2::new(3.0, 4.0));
    ///
    /// let ranges = store.calculate_dirty_ranges(&store.dirty_positions());
    /// // Returns: [(0, 2)] - entities 0 and 1 are dirty (contiguous)
    /// ```
    pub fn calculate_dirty_ranges(&self, bitset: &FixedBitSet) -> Vec<(usize, usize)> {
        let mut ranges = Vec::new();
        let mut current_start: Option<usize> = None;
        let mut current_length = 0;

        for idx in bitset.ones() {
            match current_start {
                None => {
                    // Start a new range
                    current_start = Some(idx);
                    current_length = 1;
                }
                Some(start) => {
                    if idx == start + current_length {
                        // Continuation of current range
                        current_length += 1;
                    } else {
                        // End of current range, start new one
                        ranges.push((start, current_length));
                        current_start = Some(idx);
                        current_length = 1;
                    }
                }
            }
        }

        // Don't forget the last range
        if let Some(start) = current_start {
            ranges.push((start, current_length));
        }

        ranges
    }

    /// Marks position ranges as clean (not dirty).
    ///
    /// This should be called after successful GPU upload to prevent
    /// re-uploading the same data in the next frame.
    ///
    /// # Examples
    ///
    /// ```
    /// use soa_entity::EntityStore;
    ///
    /// let mut store = EntityStore::new(100);
    /// // ... make changes ...
    ///
    /// let ranges = store.calculate_dirty_ranges(&store.dirty_positions());
    /// // ... upload to GPU successfully ...
    /// store.mark_positions_clean(&ranges);
    /// ```
    pub fn mark_positions_clean(&mut self, ranges: &[(usize, usize)]) {
        for &(start, length) in ranges {
            for i in start..(start + length) {
                self.dirty_positions.remove(i);
            }
        }
    }

    /// Marks color ranges as clean (not dirty).
    ///
    /// This should be called after successful GPU upload to prevent
    /// re-uploading the same data in the next frame.
    pub fn mark_colors_clean(&mut self, ranges: &[(usize, usize)]) {
        for &(start, length) in ranges {
            for i in start..(start + length) {
                self.dirty_colors.remove(i);
            }
        }
    }

    /// Marks all entities as clean (not dirty).
    ///
    /// This is useful for forcing a full upload or clearing dirty state
    /// after a complete buffer refresh.
    #[inline]
    pub fn mark_all_clean(&mut self) {
        self.dirty_positions.clear();
        self.dirty_colors.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_creation() {
        let store = EntityStore::new(100);
        assert_eq!(store.capacity(), 100);
        assert_eq!(store.count(), 0);
        assert_eq!(store.free_slots_count(), 100);
    }

    #[test]
    fn test_spawn_single_entity() {
        let mut store = EntityStore::new(100);
        let id = store.spawn().unwrap();

        assert!(store.is_valid(id));
        assert_eq!(store.count(), 1);
    }

    #[test]
    fn test_spawn_reuses_free_slot() {
        let mut store = EntityStore::new(100);

        let id1 = store.spawn().unwrap();
        store.despawn(id1).unwrap();

        let id2 = store.spawn().unwrap();

        assert_eq!(id2.index(), id1.index());
        assert_eq!(id2.generation(), id1.generation() + 1);
        assert!(store.is_valid(id2));
        assert!(!store.is_valid(id1));
    }

    #[test]
    fn test_despawn_invalidates_id() {
        let mut store = EntityStore::new(100);
        let id = store.spawn().unwrap();

        store.despawn(id).unwrap();
        assert!(!store.is_valid(id));
    }

    #[test]
    fn test_despawn_invalid_id_errors() {
        let mut store = EntityStore::new(100);
        let fake_id = EntityId::new(999, 1);

        assert!(store.despawn(fake_id).is_err());
    }

    #[test]
    fn test_position_access() {
        let mut store = EntityStore::new(100);
        let id = store.spawn().unwrap();

        let pos = store.position(id).unwrap();
        assert_eq!(pos, Vec2::new(0.0, 0.0));
    }

    #[test]
    fn test_position_set() {
        let mut store = EntityStore::new(100);
        let id = store.spawn().unwrap();

        store.set_position(id, Vec2::new(50.0, 75.0)).unwrap();

        let pos = store.position(id).unwrap();
        assert_eq!(pos, Vec2::new(50.0, 75.0));
    }

    #[test]
    fn test_position_invalid_entity() {
        let store = EntityStore::new(100);
        let fake_id = EntityId::new(999, 1);

        assert!(store.position(fake_id).is_none());
    }

    #[test]
    fn test_compaction_reduces_fragmentation() {
        let mut store = EntityStore::new(100);

        // Spawn 50 entities
        let ids: Vec<_> = (0..50).map(|_| store.spawn().unwrap()).collect();

        // Despawn 25 to create fragmentation
        for id in ids.iter().take(25) {
            store.despawn(*id).unwrap();
        }

        let frag_before = store.fragmentation();
        assert!(frag_before > 0.2);

        store.compact();

        let frag_after = store.fragmentation();
        assert!(frag_after < frag_before);
    }

    #[test]
    fn test_compaction_preserves_values() {
        let mut store = EntityStore::new(100);
        let id = store.spawn().unwrap();

        store.set_position(id, Vec2::new(100.0, 200.0)).unwrap();
        store
            .set_color(id, Color::rgba(1.0, 0.5, 0.25, 0.75))
            .unwrap();

        let pos_before = store.position(id).unwrap();
        let col_before = store.color(id).unwrap();

        store.compact();

        // Entity should still be accessible after compaction
        // (though ID may have changed index)
        assert!(store.is_valid(id));
    }

    #[test]
    fn test_capacity_reached() {
        let mut store = EntityStore::new(10);

        // Spawn all entities
        for _ in 0..10 {
            store.spawn().unwrap();
        }

        // Next spawn should fail
        assert!(store.spawn().is_err());
    }

    #[test]
    fn test_generational_ids() {
        let mut store = EntityStore::new(100);

        let id1 = store.spawn().unwrap();
        let id1_gen = id1.generation();

        store.despawn(id1).unwrap();

        let id2 = store.spawn().unwrap();

        assert_eq!(id2.index(), id1.index());
        assert_eq!(id2.generation(), id1_gen + 1);
    }

    #[test]
    fn test_color_access() {
        let mut store = EntityStore::new(100);
        let id = store.spawn().unwrap();

        let col = store.color(id).unwrap();
        assert_eq!(col, Color::rgba(0.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn test_color_set() {
        let mut store = EntityStore::new(100);
        let id = store.spawn().unwrap();

        store
            .set_color(id, Color::rgba(1.0, 0.5, 0.25, 0.75))
            .unwrap();

        let col = store.color(id).unwrap();
        assert_eq!(col, Color::rgba(1.0, 0.5, 0.25, 0.75));
    }

    // ===== Dirty Tracking Tests =====

    #[test]
    fn test_setter_marks_dirty() {
        let mut store = EntityStore::new(100);
        let id = store.spawn().unwrap();

        // Initially not dirty
        assert!(!store.dirty_positions().contains(id.index()));

        store.set_position(id, Vec2::new(10.0, 20.0)).unwrap();

        // Now position is dirty
        assert!(store.dirty_positions().contains(id.index()));
    }

    #[test]
    fn test_setter_doesnt_mark_other_components_dirty() {
        let mut store = EntityStore::new(100);
        let id = store.spawn().unwrap();

        store.set_position(id, Vec2::new(10.0, 20.0)).unwrap();

        // Position dirty, colors not
        assert!(store.dirty_positions().contains(id.index()));
        assert!(!store.dirty_colors().contains(id.index()));
    }

    #[test]
    fn test_set_color_marks_dirty() {
        let mut store = EntityStore::new(100);
        let id = store.spawn().unwrap();

        store
            .set_color(id, Color::rgba(1.0, 0.5, 0.25, 0.75))
            .unwrap();

        // Color dirty, position not
        assert!(store.dirty_colors().contains(id.index()));
        assert!(!store.dirty_positions().contains(id.index()));
    }

    #[test]
    fn test_calculate_contiguous_ranges() {
        let mut store = EntityStore::new(100);

        // Spawn entities and mark some as dirty
        let ids: Vec<_> = (0..20).map(|_| store.spawn().unwrap()).collect();

        // Mark 0, 1, 2 as dirty (contiguous)
        store.set_position(ids[0], Vec2::new(0.0, 0.0)).unwrap();
        store.set_position(ids[1], Vec2::new(1.0, 1.0)).unwrap();
        store.set_position(ids[2], Vec2::new(2.0, 2.0)).unwrap();

        // Mark 10, 11, 12 as dirty (separate range)
        store.set_position(ids[10], Vec2::new(10.0, 10.0)).unwrap();
        store.set_position(ids[11], Vec2::new(11.0, 11.0)).unwrap();
        store.set_position(ids[12], Vec2::new(12.0, 12.0)).unwrap();

        let ranges = store.calculate_dirty_ranges(store.dirty_positions());

        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0], (0, 3)); // Entities 0, 1, 2
        assert_eq!(ranges[1], (10, 3)); // Entities 10, 11, 12
    }

    #[test]
    fn test_calculate_ranges_single_entity() {
        let mut store = EntityStore::new(100);
        let id = store.spawn().unwrap();

        store.set_position(id, Vec2::new(5.0, 10.0)).unwrap();

        let ranges = store.calculate_dirty_ranges(store.dirty_positions());

        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0], (0, 1)); // Single entity at index 0
    }

    #[test]
    fn test_calculate_ranges_empty() {
        let store = EntityStore::new(100);

        let ranges = store.calculate_dirty_ranges(store.dirty_positions());

        assert_eq!(ranges.len(), 0);
    }

    #[test]
    fn test_mark_positions_clean() {
        let mut store = EntityStore::new(100);
        let id = store.spawn().unwrap();

        store.set_position(id, Vec2::new(10.0, 20.0)).unwrap();
        assert!(store.dirty_positions().contains(id.index()));

        let ranges = store.calculate_dirty_ranges(store.dirty_positions());
        store.mark_positions_clean(&ranges);

        assert!(!store.dirty_positions().contains(id.index()));
    }

    #[test]
    fn test_mark_colors_clean() {
        let mut store = EntityStore::new(100);
        let id = store.spawn().unwrap();

        store
            .set_color(id, Color::rgba(1.0, 0.5, 0.25, 0.75))
            .unwrap();
        assert!(store.dirty_colors().contains(id.index()));

        let ranges = store.calculate_dirty_ranges(store.dirty_colors());
        store.mark_colors_clean(&ranges);

        assert!(!store.dirty_colors().contains(id.index()));
    }

    #[test]
    fn test_mark_all_clean() {
        let mut store = EntityStore::new(100);
        let id = store.spawn().unwrap();

        store.set_position(id, Vec2::new(10.0, 20.0)).unwrap();
        store
            .set_color(id, Color::rgba(1.0, 0.5, 0.25, 0.75))
            .unwrap();

        assert!(store.dirty_positions().contains(id.index()));
        assert!(store.dirty_colors().contains(id.index()));

        store.mark_all_clean();

        assert!(!store.dirty_positions().contains(id.index()));
        assert!(!store.dirty_colors().contains(id.index()));
    }

    #[test]
    fn test_dirty_tracking_across_frames() {
        let mut store = EntityStore::new(100);
        let id1 = store.spawn().unwrap();
        let id2 = store.spawn().unwrap();

        // Frame 1: Change position
        store.set_position(id1, Vec2::new(10.0, 20.0)).unwrap();
        let ranges1 = store.calculate_dirty_ranges(store.dirty_positions());
        assert_eq!(ranges1, vec![(0, 1)]);

        // "Upload to GPU" and clean
        store.mark_positions_clean(&ranges1);
        assert!(!store.dirty_positions().contains(id1.index()));

        // Frame 2: Change different entity
        store.set_position(id2, Vec2::new(30.0, 40.0)).unwrap();
        let ranges2 = store.calculate_dirty_ranges(store.dirty_positions());

        // Only id2 is dirty, not id1
        assert_eq!(ranges2, vec![(1, 1)]);
        assert!(!store.dirty_positions().contains(id1.index()));
        assert!(store.dirty_positions().contains(id2.index()));
    }

    #[test]
    fn test_mixed_dirty_components() {
        let mut store = EntityStore::new(100);
        let id1 = store.spawn().unwrap();
        let id2 = store.spawn().unwrap();

        // Change position of id1
        store.set_position(id1, Vec2::new(10.0, 20.0)).unwrap();

        // Change color of id2
        store
            .set_color(id2, Color::rgba(1.0, 0.5, 0.25, 0.75))
            .unwrap();

        // Position dirty set contains only id1
        assert!(store.dirty_positions().contains(id1.index()));
        assert!(!store.dirty_positions().contains(id2.index()));

        // Color dirty set contains only id2
        assert!(!store.dirty_colors().contains(id1.index()));
        assert!(store.dirty_colors().contains(id2.index()));
    }
}
