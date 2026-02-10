// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - ECS SparseSet Module
//
// This module provides a SparseSet implementation for efficient component storage
// in the Entity Component System (ECS).
//
// Key Features:
// - O(1) insert, remove, and lookup operations
// - Dense iteration over active components (cache-friendly)
// - Memory-efficient for sparse component distributions
// - Stable entity indices during iteration
//
// Architecture:
// - Sparse array: Maps entity indices to dense array positions
// - Dense array: Stores actual component data contiguously
// - Both arrays grow dynamically but never shrink (simple implementation)
//
// Reference:
// - Data-Oriented Design by Richard Fabian
// - Speck ECS SparseSet implementation
// ═══════════════════════════════════════════════════════════════════════════════

use crate::ecs::component::ComponentStorage;
use alloc::vec::Vec;
use core::mem;

/// Sentinel value indicating an entity has no component in this sparse set
const SPARSE_SENTINEL: usize = usize::MAX;

/// SparseSet storage for ECS components
///
/// Provides O(1) insertion, removal, and lookup with efficient iteration.
///
/// # Memory Layout
///
/// ```text
/// Entity Index:  0   1   2   3   4   5   6
/// Sparse Array: [X, - , - , Y, - , Z, -]   (- = SPARSE_SENTINEL)
///               |       |       |
///               v       v       v
/// Dense Array: [A, B, C]                  (Component data)
/// Dense Index: 0   1   2
///               ^   ^   ^
///               |   |   |
/// Entity IDs:   0   3   5
/// ```
///
/// - `sparse[entity_id]` stores the index in `dense` where the component lives
/// - `dense[dense_index]` stores the actual component data
/// - `dense[dense_index]` also maps back to `sparse` via reverse lookup
///
/// # Performance
///
/// - **Insert**: O(1) - append to dense, update sparse
/// - **Remove**: O(1) - swap with last element, update both arrays
/// - **Lookup**: O(1) - direct sparse array access
/// - **Iteration**: O(n) - iterate only over dense array (cache-friendly)
///
/// # Examples
///
/// ```
/// use archflow_logic::ecs::SparseSet;
///
/// let mut set: SparseSet<u32> = SparseSet::new();
///
/// // Insert components for entities
/// set.insert(0, 10);
/// set.insert(5, 50);
/// set.insert(10, 100);
///
/// // Lookup by entity ID
/// assert_eq!(set.get(0), Some(&10));
/// assert_eq!(set.get(5), Some(&50));
/// assert_eq!(set.get(3), None);  // No component for entity 3
///
/// // Iterate over all components (dense iteration)
/// let values: Vec<_> = set.iter().copied().collect();
/// assert_eq!(values.len(), 3);
/// ```
#[derive(Debug)]
pub struct SparseSet<T> {
    /// Maps entity_id -> dense_index
    /// Uses SPARSE_SENTINEL to indicate "no component"
    sparse: Vec<usize>,

    /// Dense array of actual component data
    dense: Vec<T>,

    /// Maps dense_index -> entity_id
    /// Used for O(1) removal and stable iteration
    entities: Vec<usize>,
}

impl<T> SparseSet<T> {
    /// Creates a new empty SparseSet
    #[inline]
    pub fn new() -> Self {
        Self {
            sparse: Vec::new(),
            dense: Vec::new(),
            entities: Vec::new(),
        }
    }

    /// Creates a SparseSet with pre-allocated capacity
    #[inline]
    pub fn with_capacity(entity_capacity: usize, component_capacity: usize) -> Self {
        Self {
            sparse: Vec::with_capacity(entity_capacity),
            dense: Vec::with_capacity(component_capacity),
            entities: Vec::with_capacity(component_capacity),
        }
    }

    /// Ensures the sparse array can handle the given entity index
    #[inline]
    fn ensure_sparse_capacity(&mut self, entity_index: usize) {
        if entity_index >= self.sparse.len() {
            let new_size = entity_index + 1;
            self.sparse.resize(new_size, SPARSE_SENTINEL);
        }
    }

    /// Inserts a component for the given entity
    ///
    /// # Panics
    ///
    /// Panics if the entity already has a component in this set.
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_logic::ecs::SparseSet;
    ///
    /// let mut set: SparseSet<u32> = SparseSet::new();
    /// set.insert(5, 42);
    /// assert_eq!(set.get(5), Some(&42));
    /// ```
    #[inline]
    pub fn insert(&mut self, entity_id: usize, component: T) {
        self.ensure_sparse_capacity(entity_id);

        // Check for duplicate insert
        debug_assert!(
            self.sparse[entity_id] == SPARSE_SENTINEL,
            "Entity {} already has a component in this SparseSet",
            entity_id
        );

        let dense_index = self.dense.len();

        self.sparse[entity_id] = dense_index;
        self.dense.push(component);
        self.entities.push(entity_id);
    }

    /// Removes a component from the given entity
    ///
    /// Returns `None` if the entity has no component.
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_logic::ecs::SparseSet;
    ///
    /// let mut set: SparseSet<u32> = SparseSet::new();
    /// set.insert(5, 42);
    /// assert_eq!(set.remove(5), Some(42));
    /// assert_eq!(set.remove(5), None);
    /// ```
    #[inline]
    pub fn remove(&mut self, entity_id: usize) -> Option<T> {
        // Check if entity has a component
        let dense_index = *self.sparse.get(entity_id)?;

        if dense_index == SPARSE_SENTINEL {
            return None;
        }

        // Calculate last index
        let last_dense_index = self.dense.len() - 1;

        // Read the component to remove FIRST (before any mutations)
        let removed_component = unsafe {
            // SAFETY: We just verified dense_index is valid
            ptr::read(self.dense.get_unchecked(dense_index))
        };

        // Swap with last element if not already last
        if dense_index != last_dense_index {
            let last_entity_id = self.entities[last_dense_index];

            unsafe {
                // Move last component to removed position
                let component = ptr::read(self.dense.get_unchecked(last_dense_index));
                *self.dense.get_unchecked_mut(dense_index) = component;

                // Update sparse array for the moved entity
                *self.sparse.get_unchecked_mut(last_entity_id) = dense_index;

                // Update entities array
                *self.entities.get_unchecked_mut(dense_index) = last_entity_id;
            }

            // Remove last element
            self.dense.pop();
            self.entities.pop();
        } else {
            // Already the last element, just pop
            self.dense.pop();
            self.entities.pop();
        }

        // Mark sparse array entry as empty
        self.sparse[entity_id] = SPARSE_SENTINEL;

        Some(removed_component)
    }

    /// ```
    /// use archflow_logic::ecs::SparseSet;
    ///
    /// let mut set: SparseSet<u32> = SparseSet::new();
    /// set.insert(5, 42);
    /// assert_eq!(set.get(5), Some(&42));
    /// assert_eq!(set.get(3), None);
    /// ```
    #[inline]
    pub fn get(&self, entity_id: usize) -> Option<&T> {
        let dense_index = *self.sparse.get(entity_id)?;

        if dense_index == SPARSE_SENTINEL {
            return None;
        }

        // SAFETY: We just verified the index is valid
        unsafe { Some(self.dense.get_unchecked(dense_index)) }
    }

    /// Gets a mutable reference to the component for the given entity
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_logic::ecs::SparseSet;
    ///
    /// let mut set: SparseSet<u32> = SparseSet::new();
    /// set.insert(5, 42);
    ///
    /// if let Some(val) = set.get_mut(5) {
    ///     *val = 100;
    /// }
    ///
    /// assert_eq!(set.get(5), Some(&100));
    /// ```
    #[inline]
    pub fn get_mut(&mut self, entity_id: usize) -> Option<&mut T> {
        let dense_index = *self.sparse.get(entity_id)?;

        if dense_index == SPARSE_SENTINEL {
            return None;
        }

        // SAFETY: We just verified the index is valid
        unsafe { Some(self.dense.get_unchecked_mut(dense_index)) }
    }

    /// Returns true if the entity has a component in this set
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_logic::ecs::SparseSet;
    ///
    /// let mut set: SparseSet<u32> = SparseSet::new();
    /// assert!(!set.contains(5));
    /// set.insert(5, 42);
    /// assert!(set.contains(5));
    /// ```
    #[inline]
    pub fn contains(&self, entity_id: usize) -> bool {
        self.sparse
            .get(entity_id)
            .map_or(false, |&idx| idx != SPARSE_SENTINEL)
    }

    /// Returns the number of components stored
    #[inline]
    pub fn len(&self) -> usize {
        self.dense.len()
    }

    /// Returns true if no components are stored
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.dense.is_empty()
    }

    /// Clears all components
    ///
    /// Note: This does NOT shrink the sparse array, only the dense arrays.
    #[inline]
    pub fn clear(&mut self) {
        self.sparse.clear();
        self.dense.clear();
        self.entities.clear();
    }

    /// Returns an iterator over all (entity_id, &component) pairs
    ///
    /// Iteration is dense and cache-friendly, making it ideal for
    /// game loops and systems processing many entities.
    ///
    /// # Examples
    ///
    /// ```
    /// use archflow_logic::ecs::SparseSet;
    ///
    /// let mut set: SparseSet<u32> = SparseSet::new();
    /// set.insert(0, 10);
    /// set.insert(5, 50);
    /// set.insert(10, 100);
    ///
    /// let pairs: Vec<_> = set.iter_entity().collect();
    /// assert_eq!(pairs.len(), 3);
    /// ```
    #[inline]
    pub fn iter_entity(&self) -> IterEntity<'_, T> {
        IterEntity {
            sparse_set: self,
            index: 0,
        }
    }

    /// Returns an iterator over all components
    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.dense.iter()
    }

    /// Returns a mutable iterator over all components
    #[inline]
    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, T> {
        self.dense.iter_mut()
    }

    /// Gets the entity ID at the given dense index
    ///
    /// # Panics
    ///
    /// Panics if `dense_index >= len()`
    #[inline]
    pub fn entity_at(&self, dense_index: usize) -> usize {
        self.entities[dense_index]
    }

    /// Returns the current sparse array capacity
    #[inline]
    pub fn sparse_capacity(&self) -> usize {
        self.sparse.len()
    }

    /// Returns the current dense array capacity
    #[inline]
    pub fn dense_capacity(&self) -> usize {
        self.dense.capacity()
    }
}

impl<T> Default for SparseSet<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Default + 'static> ComponentStorage for SparseSet<T> {
    type Item = T;

    #[inline]
    fn insert(&mut self, entity_index: usize, component: T) {
        self.insert(entity_index, component);
    }

    #[inline]
    fn remove(&mut self, entity_index: usize) -> Option<T> {
        self.remove(entity_index)
    }

    #[inline]
    fn get(&self, entity_index: usize) -> Option<&T> {
        self.get(entity_index)
    }

    #[inline]
    fn get_mut(&mut self, entity_index: usize) -> Option<&mut T> {
        self.get_mut(entity_index)
    }

    #[inline]
    fn contains(&self, entity_index: usize) -> bool {
        self.contains(entity_index)
    }

    #[inline]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    #[inline]
    fn clear(&mut self) {
        self.clear();
    }
}

/// Iterator over (entity_id, &component) pairs in a SparseSet
#[derive(Debug)]
pub struct IterEntity<'a, T> {
    sparse_set: &'a SparseSet<T>,
    index: usize,
}

impl<'a, T> Iterator for IterEntity<'a, T> {
    type Item = (usize, &'a T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.sparse_set.dense.len() {
            let dense_index = self.index;
            self.index += 1;

            let entity_id = self.sparse_set.entities[dense_index];
            let component = unsafe {
                // SAFETY: We've verified dense_index is within bounds
                self.sparse_set.dense.get_unchecked(dense_index)
            };

            return Some((entity_id, component));
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.sparse_set.dense.len().saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

impl<'a, T> ExactSizeIterator for IterEntity<'a, T> {}

// Need to import ptr for the remove implementation
use core::ptr;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sparse_set_insert() {
        let mut set: SparseSet<u32> = SparseSet::new();

        set.insert(0, 10);
        set.insert(5, 50);
        set.insert(10, 100);

        assert_eq!(set.get(0), Some(&10));
        assert_eq!(set.get(5), Some(&50));
        assert_eq!(set.get(10), Some(&100));
        assert_eq!(set.get(3), None);
    }

    #[test]
    fn test_sparse_set_remove() {
        let mut set: SparseSet<u32> = SparseSet::new();

        set.insert(0, 10);
        set.insert(5, 50);
        set.insert(10, 100);

        assert_eq!(set.remove(5), Some(50));
        assert_eq!(set.get(5), None);
        assert_eq!(set.len(), 2);

        // Remove non-existent entity
        assert_eq!(set.remove(99), None);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_sparse_set_remove_last() {
        let mut set: SparseSet<u32> = SparseSet::new();

        set.insert(0, 10);
        set.insert(5, 50);

        // Remove the last inserted element
        assert_eq!(set.remove(5), Some(50));
        assert_eq!(set.get(5), None);
        assert_eq!(set.get(0), Some(&10));
    }

    #[test]
    fn test_sparse_set_contains() {
        let mut set: SparseSet<u32> = SparseSet::new();

        assert!(!set.contains(0));
        assert!(!set.contains(5));

        set.insert(0, 10);
        assert!(set.contains(0));
        assert!(!set.contains(5));

        set.insert(5, 50);
        assert!(set.contains(5));

        set.remove(0);
        assert!(!set.contains(0));
        assert!(set.contains(5));
    }

    #[test]
    fn test_sparse_set_len() {
        let mut set: SparseSet<u32> = SparseSet::new();

        assert_eq!(set.len(), 0);
        assert!(set.is_empty());

        set.insert(0, 10);
        assert_eq!(set.len(), 1);
        assert!(!set.is_empty());

        set.insert(5, 50);
        set.insert(10, 100);
        assert_eq!(set.len(), 3);

        set.remove(5);
        assert_eq!(set.len(), 2);

        set.clear();
        assert_eq!(set.len(), 0);
        assert!(set.is_empty());
    }

    #[test]
    fn test_sparse_set_iteration() {
        let mut set: SparseSet<u32> = SparseSet::new();

        set.insert(0, 10);
        set.insert(5, 50);
        set.insert(10, 100);

        let values: Vec<_> = set.iter().copied().collect();
        assert_eq!(values.len(), 3);
        assert!(values.contains(&10));
        assert!(values.contains(&50));
        assert!(values.contains(&100));
    }

    #[test]
    fn test_sparse_set_iter_entity() {
        let mut set: SparseSet<u32> = SparseSet::new();

        set.insert(0, 10);
        set.insert(5, 50);
        set.insert(10, 100);

        let pairs: Vec<_> = set.iter_entity().collect();
        assert_eq!(pairs.len(), 3);

        // Check that we have the right entity-component pairs
        let pairs_map: alloc::collections::BTreeMap<_, _> = pairs.into_iter().collect();
        assert_eq!(pairs_map.get(&0), Some(&&10));
        assert_eq!(pairs_map.get(&5), Some(&&50));
        assert_eq!(pairs_map.get(&10), Some(&&100));
    }

    #[test]
    fn test_sparse_set_get_mut() {
        let mut set: SparseSet<u32> = SparseSet::new();

        set.insert(5, 50);

        if let Some(val) = set.get_mut(5) {
            *val = 100;
        }

        assert_eq!(set.get(5), Some(&100));
    }

    #[test]
    fn test_sparse_set_iter_mut() {
        let mut set: SparseSet<u32> = SparseSet::new();

        set.insert(0, 10);
        set.insert(5, 50);
        set.insert(10, 100);

        for val in set.iter_mut() {
            *val *= 2;
        }

        assert_eq!(set.get(0), Some(&20));
        assert_eq!(set.get(5), Some(&100));
        assert_eq!(set.get(10), Some(&200));
    }

    #[test]
    fn test_sparse_set_memory_efficiency() {
        let mut set: SparseSet<u32> = SparseSet::new();

        // Insert components for sparse entities (0, 100, 200, 300)
        set.insert(0, 0);
        set.insert(100, 100);
        set.insert(200, 200);
        set.insert(300, 300);

        // Sparse array should grow to accommodate entity 300
        assert!(set.sparse_capacity() >= 301);

        // But dense array only has 4 elements
        assert_eq!(set.len(), 4);
        assert_eq!(set.dense_capacity(), 4);

        // Verify sparse array sentinel values for missing entities
        assert!(set.contains(0));
        assert!(!set.contains(1));
        assert!(!set.contains(99));
        assert!(set.contains(100));
    }

    #[test]
    fn test_sparse_set_with_capacity() {
        let set: SparseSet<u32> = SparseSet::with_capacity(100, 10);

        assert!(set.is_empty());
        assert_eq!(set.sparse_capacity(), 0);
    }

    #[test]
    fn test_sparse_set_clear() {
        let mut set: SparseSet<u32> = SparseSet::new();

        set.insert(0, 10);
        set.insert(5, 50);

        set.clear();

        assert!(set.is_empty());
        assert!(!set.contains(0));
        assert!(!set.contains(5));
    }

    #[test]
    fn test_sparse_set_entity_at() {
        let mut set: SparseSet<u32> = SparseSet::new();

        set.insert(5, 50);
        set.insert(10, 100);
        set.insert(0, 10);

        // The entities are stored in insertion order
        assert_eq!(set.entity_at(0), 5);
        assert_eq!(set.entity_at(1), 10);
        assert_eq!(set.entity_at(2), 0);
    }

    #[test]
    fn test_sparse_set_remove_middle_element() {
        let mut set: SparseSet<u32> = SparseSet::new();

        set.insert(0, 10);
        set.insert(5, 50);
        set.insert(10, 100);

        // Remove middle element (5)
        let removed = set.remove(5);
        assert_eq!(removed, Some(50));

        // After removal, the last element should be moved to position 1
        // Check that the set is still consistent
        assert_eq!(set.len(), 2);
        assert!(set.contains(0));
        assert!(!set.contains(5));
        assert!(set.contains(10));

        // Verify we can still access remaining elements
        assert_eq!(set.get(0), Some(&10));
        assert_eq!(set.get(10), Some(&100));
    }

    #[test]
    fn test_sparse_set_complex_type() {
        #[derive(Debug, PartialEq)]
        struct Position {
            x: f32,
            y: f32,
        }

        let mut set: SparseSet<Position> = SparseSet::new();

        set.insert(0, Position { x: 1.0, y: 2.0 });
        set.insert(5, Position { x: 3.0, y: 4.0 });

        assert_eq!(set.get(0), Some(&Position { x: 1.0, y: 2.0 }));
        assert_eq!(set.get(5), Some(&Position { x: 3.0, y: 4.0 }));
    }

    #[test]
    fn test_sparse_set_capacity_growth() {
        let mut set: SparseSet<u32> = SparseSet::new();

        // Insert out of order to test sparse array growth
        set.insert(100, 100);
        assert!(set.sparse_capacity() >= 101);

        set.insert(0, 0);
        assert!(set.sparse_capacity() >= 101);

        set.insert(500, 500);
        assert!(set.sparse_capacity() >= 501);

        // Dense array should only have 3 elements
        assert_eq!(set.len(), 3);
    }
}
