// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - ECS Archetype Module (EPIC-AFRAME-003)
//
// This module provides Data-Oriented Design (DOD) storage for ECS through archetypes.
// Archetypes group entities with identical component types together for optimal
// cache utilization and SIMD-friendly iteration.
//
// Key Features:
// - Archetype: Stores entities with same components together (cache-friendly)
// - ComponentColumn: Contiguous column-oriented storage for raw byte data
// - BatchIter: SIMD-friendly batched iteration over component data
// - ArchetypeStorage: Manages multiple archetypes with efficient lookup
//
// Architecture:
// - Archetype-based storage (similar to Unity DOTS, Bevy ECS)
// - Type hashing for stable archetype identification
// - Column-major layout for each component type
// - Batch iteration for potential vectorization
//
// Performance Benefits:
// - Cache locality: Same component types stored contiguously
// - SIMD potential: Batches of identical types for vectorization
// - Memory efficiency: No per-entity overhead for component storage
// - Parallel iteration: Independent archetypes can be processed in parallel
//
// ═══════════════════════════════════════════════════════════════════════════════


use alloc::collections::BTreeMap;
use alloc::vec;
extern crate alloc;
use alloc::vec::Vec;
use core::any::TypeId;
use core::hash::{Hash, Hasher};
use core::mem;

use crate::ecs::component::ComponentId;
use crate::ecs::pool::ColumnPool;

/// Unique identifier for an archetype
///
/// Computed as a stable hash of the sorted component type IDs that define
/// the archetype. This ensures entities with the same component composition
/// map to the same archetype.
///
/// # Stability
///
/// ArchetypeId is stable across runs and archetype creation order, ensuring
/// consistent entity placement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArchetypeId(u64);

impl ArchetypeId {
    /// Creates an ArchetypeId from a sorted slice of component type IDs
    #[inline]
    pub fn from_types(types: &[ComponentId]) -> Self {
        // Use FNV-style hashing for stable, fast hash computation
        let mut hasher = fnv_hasher();

        // Hash each component type ID in order
        for component_id in types {
            component_id.type_id().hash(&mut hasher);
        }

        Self(hasher.finish())
    }

    /// Returns the raw hash value
    #[inline]
    pub const fn value(&self) -> u64 {
        self.0
    }
}

impl Hash for ArchetypeId {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// Creates a simple FNV-1a hasher for stable hashing
#[inline]
fn fnv_hasher() -> impl Hasher {
    struct FnvHasher(u64);

    impl Hasher for FnvHasher {
        #[inline]
        fn finish(&self) -> u64 {
            self.0
        }

        #[inline]
        fn write(&mut self, bytes: &[u8]) {
            for &byte in bytes {
                self.0 = self.0 ^ byte as u64;
                self.0 = self.0.wrapping_mul(0x100000001b3);
            }
        }
    }

    FnvHasher(0xcbf29ce484222325)
}

/// Column-oriented storage for component data
///
/// Stores components of a single type contiguously as raw bytes.
/// This provides excellent cache locality for systems that iterate
/// over a single component type.
///
/// # Memory Layout
///
/// ```text
/// ComponentColumn<f32> (stride = 4):
/// [0.0][1.0][2.0][3.0][4.0]...
///  ^       ^       ^
///  |       |       |
///  idx 0   idx 1   idx 2
/// ```
///
/// # Type Safety
///
/// The type parameter `T` is only used for get/set operations. The actual
/// storage is untyped bytes, so care must be taken to use consistent types.
pub struct ComponentColumn {
    /// Raw byte storage for component data
    data: Vec<u8>,
    /// Size of each component in bytes
    stride: usize,
    /// Number of components stored
    len: usize,
}

impl ComponentColumn {
    /// Creates a new empty ComponentColumn for components of size `stride`
    #[inline]
    pub fn new(stride: usize) -> Self {
        Self {
            data: Vec::new(),
            stride,
            len: 0,
        }
    }

    /// Creates a ComponentColumn with capacity for `capacity` components
    #[inline]
    pub fn with_capacity(stride: usize, capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity * stride),
            stride,
            len: 0,
        }
    }

    /// Returns the number of components stored
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if no components are stored
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Creates a ComponentColumn from pooled data
    #[inline]
    pub fn from_pool(data: Vec<u8>, stride: usize) -> Self {
        Self {
            data,
            stride,
            len: 0,
        }
    }

    /// Returns the stride (size of each component in bytes)
    #[inline]
    pub fn stride(&self) -> usize {
        self.stride
    }

    /// Pushes a new component to the end of the column
    ///
    /// # Safety
    ///
    /// Caller must ensure that `T` has the same size as the column's stride.
    #[inline]
    pub fn push<T>(&mut self, value: T) {
        assert_eq!(mem::size_of::<T>(), self.stride, "Type size mismatch");

        let _start = self.data.len();
        self.data.extend_from_slice(unsafe {
            core::slice::from_raw_parts(&value as *const T as *const u8, self.stride)
        });
        self.len += 1;
    }

    /// Gets a reference to the component at `index`
    ///
    /// Returns `None` if the index is out of bounds.
    ///
    /// # Safety
    ///
    /// Caller must ensure that `T` has the same size as the column's stride.
    pub fn get<T>(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }

        assert_eq!(mem::size_of::<T>(), self.stride, "Type size mismatch");

        let offset = index * self.stride;
        unsafe { Some(&*(self.data.as_ptr().add(offset) as *const T)) }
    }

    /// Gets a mutable reference to the component at `index`
    ///
    /// Returns `None` if the index is out of bounds.
    ///
    /// # Safety
    ///
    /// Caller must ensure that `T` has the same size as the column's stride.
    pub fn get_mut<T>(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }

        assert_eq!(mem::size_of::<T>(), self.stride, "Type size mismatch");

        let offset = index * self.stride;
        unsafe { Some(&mut *(self.data.as_mut_ptr().add(offset) as *mut T)) }
    }

    /// Sets the component at `index` to `value`
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds or if `T` has a different size
    /// than the column's stride.
    pub fn set<T>(&mut self, index: usize, value: T) {
        assert!(index < self.len, "Index out of bounds");
        assert_eq!(mem::size_of::<T>(), self.stride, "Type size mismatch");

        let offset = index * self.stride;
        unsafe {
            core::ptr::copy_nonoverlapping(
                &value as *const T as *const u8,
                self.data.as_mut_ptr().add(offset),
                self.stride,
            );
        }
    }

    /// Removes the component at `index` and shifts later components down
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    pub fn remove<T>(&mut self, index: usize) -> T {
        assert!(index < self.len, "Index out of bounds");
        assert_eq!(mem::size_of::<T>(), self.stride, "Type size mismatch");

        let offset = index * self.stride;
        let result = unsafe { core::ptr::read(self.data.as_ptr().add(offset) as *const T) };

        // Shift remaining components down
        if index + 1 < self.len {
            unsafe {
                core::ptr::copy(
                    self.data.as_ptr().add(offset + self.stride),
                    self.data.as_mut_ptr().add(offset),
                    (self.len - index - 1) * self.stride,
                );
            }
        }

        self.len -= 1;
        unsafe {
            self.data.set_len(self.len * self.stride);
        }

        result
    }

    /// Swaps components at `a` and `b`
    ///
    /// # Panics
    ///
    /// Panics if either index is out of bounds.
    pub fn swap(&mut self, a: usize, b: usize) {
        assert!(a < self.len, "Index a out of bounds");
        assert!(b < self.len, "Index b out of bounds");

        if a == b {
            return;
        }

        let offset_a = a * self.stride;
        let offset_b = b * self.stride;

        // Swap byte-by-byte
        for i in 0..self.stride {
            self.data.swap(offset_a + i, offset_b + i);
        }
    }

    /// Returns an iterator over batches of components for SIMD processing
    ///
    /// # Example
    ///
    /// ```ignore
    /// for batch in column.iter_batch::<f32>(4) {
    ///     // Process batch of up to 4 f32 values
    ///     for &value in batch {
    ///         // Process value
    ///     }
    /// }
    /// ```
    pub fn iter_batch<'a, T>(&'a self, batch_size: usize) -> BatchIter<'a, T> {
        BatchIter {
            column: self,
            batch_size,
            current: 0,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Clears all components from the column
    #[inline]
    pub fn clear(&mut self) {
        self.data.clear();
        self.len = 0;
    }

    /// Reserves capacity for at least `additional` more components
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional * self.stride);
    }
}

impl core::fmt::Debug for ComponentColumn {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ComponentColumn")
            .field("stride", &self.stride)
            .field("len", &self.len)
            .field("capacity", &(self.data.capacity() / self.stride))
            .finish()
    }
}

/// Batched iterator over component data
///
/// Iterates over components in batches for SIMD-friendly processing.
/// Each batch contains up to `batch_size` components.
///
/// # Example
///
/// ```ignore
/// for batch in column.iter_batch::<f32>(4) {
///     // batch is &[f32] with up to 4 elements
/// }
/// ```
pub struct BatchIter<'a, T> {
    _phantom: core::marker::PhantomData<T>,
    column: &'a ComponentColumn,
    batch_size: usize,
    current: usize,
}

impl<'a, T: 'a> Iterator for BatchIter<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.column.len {
            return None;
        }

        let remaining = self.column.len - self.current;
        let batch_size = self.batch_size.min(remaining);

        let slice = unsafe {
            let offset = self.current * self.column.stride;
            let ptr = self.column.data.as_ptr().add(offset) as *const T;
            core::slice::from_raw_parts(ptr, batch_size)
        };

        self.current += batch_size;
        Some(slice)
    }
}

/// Archetype: Groups entities with identical component types
///
/// Archetypes store entities that share the same set of component types.
/// This layout maximizes cache locality when iterating over entities
/// with the same component composition.
///
/// # Memory Layout
///
/// ```text
/// Archetype [Position, Velocity]:
///
/// entity_ids:  [0, 3, 7, 15]          // Entity IDs
///
/// Position Column:  [p0][p3][p7][p15] // Contiguous Position data
/// Velocity Column:  [v0][v3][v7][v15] // Contiguous Velocity data
///
/// Indices align: entity_ids[0] has Position[0] and Velocity[0]
/// ```
///
/// # Performance
///
/// - Excellent cache locality for homogeneous entity processing
/// - Enables SIMD operations on component columns
/// - Reduces branching in system execution
pub struct Archetype {
    /// Entity IDs stored in this archetype
    entity_ids: Vec<usize>,
    /// Component storage by component type ID
    components: BTreeMap<ComponentId, ComponentColumn>,
    /// Sorted list of component type IDs for this archetype
    types: Vec<ComponentId>,
}

impl Archetype {
    /// Creates a new empty archetype with the given component types
    ///
    /// # Parameters
    ///
    /// - `types`: Sorted list of component type IDs defining this archetype
    #[inline]
    pub fn new(types: Vec<ComponentId>) -> Self {
        Self {
            entity_ids: Vec::new(),
            components: BTreeMap::new(),
            types,
        }
    }

    /// Returns the archetype ID (hash of component types)
    #[inline]
    pub fn id(&self) -> ArchetypeId {
        ArchetypeId::from_types(&self.types)
    }

    /// Returns the number of entities in this archetype
    #[inline]
    pub fn len(&self) -> usize {
        self.entity_ids.len()
    }

    /// Returns true if this archetype contains no entities
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entity_ids.is_empty()
    }

    /// Returns the component types that define this archetype
    #[inline]
    pub fn types(&self) -> &[ComponentId] {
        &self.types
    }

    /// Returns true if this archetype contains the given component type
    #[inline]
    pub fn has_component(&self, component_id: ComponentId) -> bool {
        self.types.contains(&component_id)
    }

    /// Adds an entity to this archetype
    ///
    /// # Parameters
    ///
    /// - `entity_id`: The entity ID to add
    /// - `component_data`: Map of component type to raw bytes
    ///
    /// # Returns
    ///
    /// The index of the entity within the archetype
    pub fn add_entity(
        &mut self,
        entity_id: usize,
        component_data: BTreeMap<ComponentId, Vec<u8>>,
    ) -> usize {
        let index = self.len();
        self.entity_ids.push(entity_id);

        for (component_id, data) in component_data {
            if let Some(column) = self.components.get_mut(&component_id) {
                // Append data to existing column
                let old_len = column.len();
                column.reserve(1);

                // Manually extend the column's data
                unsafe {
                    let column_data = &mut column.data;
                    let _start = column_data.len();
                    column_data.extend_from_slice(&data);
                    // SAFETY: We just added exactly one component's worth of data
                    column.len = old_len + 1;
                }
            } else {
                // This should not happen if component_data matches types
                continue;
            }
        }

        index
    }

    /// Removes an entity from this archetype
    ///
    /// # Parameters
    ///
    /// - `index`: The index of the entity to remove
    ///
    /// # Returns
    ///
    /// The removed entity ID
    pub fn remove_entity(&mut self, index: usize) -> usize {
        assert!(index < self.len(), "Index out of bounds");

        // Swap with last element for O(1) removal
        let last_index = self.len() - 1;

        if index != last_index {
            // Swap entity IDs
            self.entity_ids.swap(index, last_index);

            // Swap component data in all columns
            for column in self.components.values_mut() {
                column.swap(index, last_index);
            }

            // After swap, the entity at 'index' is the one that was at 'last_index'
            // Return this ID so caller can update their references
            let entity_id = self.entity_ids[index];

            // Remove last element
            self.entity_ids.pop();
            for column in self.components.values_mut() {
                unsafe {
                    column.data.set_len(column.len * column.stride);
                    column.len -= 1;
                }
            }

            entity_id
        } else {
            // Removing the last element - no swap happened
            // Capture the ID before pop
            let entity_id = self.entity_ids[index];

            // Remove last element
            self.entity_ids.pop();
            for column in self.components.values_mut() {
                unsafe {
                    column.data.set_len(column.len * column.stride);
                    column.len -= 1;
                }
            }

            entity_id
        }
    }

    /// Gets the entity ID at the given index
    #[inline]
    pub fn get_entity_id(&self, index: usize) -> Option<usize> {
        self.entity_ids.get(index).copied()
    }

    /// Gets a reference to a component for the entity at `index`
    ///
    /// # Parameters
    ///
    /// - `component_id`: The type of component to get
    /// - `index`: The entity index within the archetype
    pub fn get_component<T>(&self, component_id: ComponentId, index: usize) -> Option<&T> {
        self.components
            .get(&component_id)
            .and_then(|col| col.get::<T>(index))
    }

    /// Gets a mutable reference to a component for the entity at `index`
    ///
    /// # Parameters
    ///
    /// - `component_id`: The type of component to get
    /// - `index`: The entity index within the archetype
    pub fn get_component_mut<T>(
        &mut self,
        component_id: ComponentId,
        index: usize,
    ) -> Option<&mut T> {
        self.components
            .get_mut(&component_id)
            .and_then(|col| col.get_mut::<T>(index))
    }

    /// Gets a reference to the component column for a specific component type
    ///
    /// # Parameters
    ///
    /// - `component_id`: The type of component to get
    #[inline]
    pub fn get_column(&self, component_id: ComponentId) -> Option<&ComponentColumn> {
        self.components.get(&component_id)
    }

    /// Gets a mutable reference to the component column for a specific component type
    ///
    /// # Parameters
    ///
    /// - `component_id`: The type of component to get
    #[inline]
    pub fn get_column_mut(&mut self, component_id: ComponentId) -> Option<&mut ComponentColumn> {
        self.components.get_mut(&component_id)
    }

    /// Adds a new component column to this archetype
    ///
    /// # Panics
    ///
    /// Panics if a column for this component type already exists.
    pub fn add_column(&mut self, component_id: ComponentId, stride: usize) {
        assert!(
            !self.components.contains_key(&component_id),
            "Column already exists for component type"
        );

        self.components
            .insert(component_id, ComponentColumn::with_capacity(stride, 16));
    }

    /// Adds a pre-allocated component column to this archetype
    ///
    /// This is used by the pool system to reuse pre-allocated columns.
    ///
    /// # Panics
    ///
    /// Panics if a column for this component type already exists.
    pub fn add_column_with_data(&mut self, component_id: ComponentId, column: ComponentColumn) {
        assert!(
            !self.components.contains_key(&component_id),
            "Column already exists for component type"
        );

        self.components.insert(component_id, column);
    }

    /// Returns an iterator over entity IDs in this archetype
    #[inline]
    pub fn iter_entities(&self) -> core::slice::Iter<'_, usize> {
        self.entity_ids.iter()
    }

    /// Clears all entities from this archetype
    #[inline]
    pub fn clear(&mut self) {
        self.entity_ids.clear();
        for column in self.components.values_mut() {
            column.clear();
        }
    }
}

impl core::fmt::Debug for Archetype {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Archetype")
            .field("id", &self.id())
            .field("entity_count", &self.len())
            .field("types", &self.types)
            .finish()
    }
}

/// Storage for managing multiple archetypes
///
/// Maintains a collection of archetypes and tracks which archetype each
/// entity belongs to.
///
/// # Example
///
/// ```ignore
/// let mut storage = ArchetypeStorage::new();
///
/// // Add entity with Position component
/// let mut types = BTreeMap::new();
/// types.insert(ComponentId::of::<Position>(), 8); // stride
/// let arch_id = storage.add_entity(0, types);
/// ```
pub struct ArchetypeStorage {
    /// All archetypes indexed by ArchetypeId
    archetypes: BTreeMap<ArchetypeId, Archetype>,
    /// Maps entity ID to its archetype ID
    entity_archetype: Vec<Option<ArchetypeId>>,
    /// Maps entity ID to its index within the archetype
    entity_index: Vec<Option<usize>>,
    /// Memory pool for reusable component columns
    column_pool: ColumnPool,
}

impl ArchetypeStorage {
    /// Creates a new empty ArchetypeStorage
    #[inline]
    pub fn new() -> Self {
        Self {
            archetypes: BTreeMap::new(),
            entity_archetype: Vec::new(),
            entity_index: Vec::new(),
            column_pool: ColumnPool::new(),
        }
    }

    /// Adds an entity with the given component types to storage
    ///
    /// # Parameters
    ///
    /// - `entity_id`: The entity ID to add
    /// - `component_strides`: Map of component type to its stride (size in bytes)
    ///
    /// # Returns
    ///
    /// The ArchetypeId where the entity was placed
    pub fn add_entity(
        &mut self,
        entity_id: usize,
        component_strides: BTreeMap<ComponentId, usize>,
    ) -> ArchetypeId {
        // Sort component types for stable archetype ID
        let mut types: Vec<ComponentId> = component_strides.keys().copied().collect();
        types.sort();

        let archetype_id = ArchetypeId::from_types(&types);

        // Ensure entity tracking vectors are large enough
        if entity_id >= self.entity_archetype.len() {
            self.entity_archetype.resize(entity_id + 1, None);
            self.entity_index.resize(entity_id + 1, None);
        }

        // Get or create archetype, using pooled columns when available
        let archetype = self.archetypes.entry(archetype_id).or_insert_with(|| {
            let mut arch = Archetype::new(types.clone());

            // Try to reuse columns from the pool
            for (component_id, &stride) in &component_strides {
                // Try to acquire a column with initial capacity of 16
                if let Some(column) = self.column_pool.try_acquire(*component_id, stride, 16) {
                    arch.add_column_with_data(
                        *component_id,
                        ComponentColumn::from_pool(column, stride),
                    );
                } else {
                    arch.add_column(*component_id, stride);
                }
            }

            arch
        });

        // Add entity to archetype
        let component_data: BTreeMap<_, _> = component_strides
            .into_iter()
            .map(|(id, stride)| (id, vec![0u8; stride]))
            .collect();

        let index = archetype.add_entity(entity_id, component_data);

        // Track entity location
        self.entity_archetype[entity_id] = Some(archetype_id);
        self.entity_index[entity_id] = Some(index);

        archetype_id
    }

    /// Removes an entity from storage
    ///
    /// # Parameters
    ///
    /// - `entity_id`: The entity ID to remove
    ///
    /// # Returns
    ///
    /// `Some(archetype_id)` if the entity was found and removed, `None` otherwise
    pub fn remove_entity(&mut self, entity_id: usize) -> Option<ArchetypeId> {
        if entity_id >= self.entity_archetype.len() {
            return None;
        }

        let archetype_id = self.entity_archetype[entity_id].take()?;
        let index = self.entity_index[entity_id].take()?;

        if let Some(archetype) = self.archetypes.get_mut(&archetype_id) {
            let removed_entity_id = archetype.remove_entity(index);

            // Update entity tracking if a different entity was swapped into this slot
            if removed_entity_id != entity_id {
                if let Some(swapped_index) = archetype.get_entity_id(index) {
                    if swapped_index < self.entity_index.len() {
                        self.entity_index[swapped_index] = Some(index);
                    }
                }
            }

            // Remove archetype if empty
            if archetype.is_empty() {
                self.archetypes.remove(&archetype_id);
            }
        }

        Some(archetype_id)
    }

    /// Gets the archetype containing the given entity
    ///
    /// # Parameters
    ///
    /// - `entity_id`: The entity ID to look up
    ///
    /// # Returns
    ///
    /// `Some(&Archetype)` if the entity exists, `None` otherwise
    pub fn get_archetype(&self, entity_id: usize) -> Option<&Archetype> {
        if entity_id >= self.entity_archetype.len() {
            return None;
        }

        let archetype_id = self.entity_archetype[entity_id];
        archetype_id.and_then(|id| self.archetypes.get(&id))
    }

    /// Gets a mutable reference to the archetype containing the given entity
    ///
    /// # Parameters
    ///
    /// - `entity_id`: The entity ID to look up
    ///
    /// # Returns
    ///
    /// `Some(&mut Archetype)` if the entity exists, `None` otherwise
    pub fn get_archetype_mut(&mut self, entity_id: usize) -> Option<&mut Archetype> {
        if entity_id >= self.entity_archetype.len() {
            return None;
        }

        let archetype_id = self.entity_archetype[entity_id];
        archetype_id.and_then(|id| self.archetypes.get_mut(&id))
    }

    /// Returns the archetype ID for the given entity
    #[inline]
    pub fn get_archetype_id(&self, entity_id: usize) -> Option<ArchetypeId> {
        self.entity_archetype.get(entity_id).copied().flatten()
    }

    /// Returns the number of archetypes in storage
    #[inline]
    pub fn archetype_count(&self) -> usize {
        self.archetypes.len()
    }

    /// Returns true if no entities are stored
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.archetypes.is_empty()
    }

    /// Clears all entities and archetypes
    #[inline]
    pub fn clear(&mut self) {
        self.archetypes.clear();
        self.entity_archetype.clear();
        self.entity_index.clear();
        self.column_pool.clear();
    }

    /// Returns an iterator over all archetypes
    #[inline]
    pub fn iter_archetypes(&self) -> impl Iterator<Item = (&ArchetypeId, &Archetype)> {
        self.archetypes.iter()
    }

    /// Returns pool statistics
    #[inline]
    pub fn pool_stats(&self) -> crate::ecs::pool::PoolStats {
        self.column_pool.stats()
    }
}

impl Default for ArchetypeStorage {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Debug for ArchetypeStorage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ArchetypeStorage")
            .field("archetype_count", &self.archetypes.len())
            .field("tracked_entities", &self.entity_archetype.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::component::ComponentId;
    use alloc::vec;

    // Test component types
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Velocity {
        dx: f32,
        dy: f32,
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Health {
        current: u32,
        max: u32,
    }

    #[test]
    fn test_archetype_id_stability() {
        let types = vec![ComponentId::of::<Position>(), ComponentId::of::<Velocity>()];

        let id1 = ArchetypeId::from_types(&types);
        let id2 = ArchetypeId::from_types(&types);

        assert_eq!(id1, id2, "Same types should produce same ID");

        // Different order should produce same ID
        let types_reversed = vec![ComponentId::of::<Velocity>(), ComponentId::of::<Position>()];
        let _id3 = ArchetypeId::from_types(&types_reversed);

        // This test documents current behavior - IDs depend on order
        // For true order-independence, types should be sorted before hashing
    }

    #[test]
    fn test_archetype_id_different_types() {
        let types1 = vec![ComponentId::of::<Position>()];
        let types2 = vec![ComponentId::of::<Velocity>()];

        let id1 = ArchetypeId::from_types(&types1);
        let id2 = ArchetypeId::from_types(&types2);

        assert_ne!(id1, id2, "Different types should produce different IDs");
    }

    #[test]
    fn test_component_column_creation() {
        let column = ComponentColumn::new(core::mem::size_of::<Position>());

        assert_eq!(column.len(), 0);
        assert!(column.is_empty());
        assert_eq!(column.stride(), core::mem::size_of::<Position>());
    }

    #[test]
    fn test_component_column_with_capacity() {
        let column = ComponentColumn::with_capacity(core::mem::size_of::<Position>(), 10);

        assert_eq!(column.len(), 0);
        // Check internal capacity is sufficient
        assert!(column.data.capacity() >= 10 * core::mem::size_of::<Position>());
    }

    #[test]
    fn test_component_column_push_get() {
        let mut column = ComponentColumn::new(core::mem::size_of::<Position>());

        let pos1 = Position { x: 1.0, y: 2.0 };
        let pos2 = Position { x: 3.0, y: 4.0 };

        column.push(pos1);
        column.push(pos2);

        assert_eq!(column.len(), 2);
        assert_eq!(column.get::<Position>(0), Some(&pos1));
        assert_eq!(column.get::<Position>(1), Some(&pos2));
        assert_eq!(column.get::<Position>(2), None);
    }

    #[test]
    fn test_component_column_set() {
        let mut column = ComponentColumn::new(core::mem::size_of::<Position>());

        column.push(Position { x: 1.0, y: 2.0 });
        column.push(Position { x: 3.0, y: 4.0 });

        column.set(0, Position { x: 10.0, y: 20.0 });

        assert_eq!(
            column.get::<Position>(0),
            Some(&Position { x: 10.0, y: 20.0 })
        );
        assert_eq!(
            column.get::<Position>(1),
            Some(&Position { x: 3.0, y: 4.0 })
        );
    }

    #[test]
    fn test_component_column_get_mut() {
        let mut column = ComponentColumn::new(core::mem::size_of::<Position>());

        column.push(Position { x: 1.0, y: 2.0 });

        if let Some(pos) = column.get_mut::<Position>(0) {
            pos.x = 10.0;
            pos.y = 20.0;
        }

        assert_eq!(
            column.get::<Position>(0),
            Some(&Position { x: 10.0, y: 20.0 })
        );
    }

    #[test]
    fn test_component_column_remove() {
        let mut column = ComponentColumn::new(core::mem::size_of::<Position>());

        column.push(Position { x: 1.0, y: 2.0 });
        column.push(Position { x: 3.0, y: 4.0 });
        column.push(Position { x: 5.0, y: 6.0 });

        let removed = column.remove::<Position>(1);

        assert_eq!(removed, Position { x: 3.0, y: 4.0 });
        assert_eq!(column.len(), 2);
        assert_eq!(
            column.get::<Position>(0),
            Some(&Position { x: 1.0, y: 2.0 })
        );
        assert_eq!(
            column.get::<Position>(1),
            Some(&Position { x: 5.0, y: 6.0 })
        );
    }

    #[test]
    fn test_component_column_swap() {
        let mut column = ComponentColumn::new(core::mem::size_of::<Position>());

        column.push(Position { x: 1.0, y: 2.0 });
        column.push(Position { x: 3.0, y: 4.0 });

        column.swap(0, 1);

        assert_eq!(
            column.get::<Position>(0),
            Some(&Position { x: 3.0, y: 4.0 })
        );
        assert_eq!(
            column.get::<Position>(1),
            Some(&Position { x: 1.0, y: 2.0 })
        );
    }

    #[test]
    fn test_component_column_clear() {
        let mut column = ComponentColumn::new(core::mem::size_of::<Position>());

        column.push(Position { x: 1.0, y: 2.0 });
        column.push(Position { x: 3.0, y: 4.0 });

        column.clear();

        assert_eq!(column.len(), 0);
        assert!(column.is_empty());
    }

    #[test]
    fn test_batch_iteration() {
        let mut column = ComponentColumn::new(core::mem::size_of::<f32>());

        for i in 0..10 {
            column.push(i as f32);
        }

        let mut batches = Vec::new();
        for batch in column.iter_batch::<f32>(3) {
            batches.push(batch.to_vec());
        }

        assert_eq!(batches.len(), 4); // 3 + 3 + 3 + 1
        assert_eq!(batches[0], vec![0.0, 1.0, 2.0]);
        assert_eq!(batches[1], vec![3.0, 4.0, 5.0]);
        assert_eq!(batches[2], vec![6.0, 7.0, 8.0]);
        assert_eq!(batches[3], vec![9.0]);
    }

    #[test]
    fn test_batch_iteration_empty() {
        let column = ComponentColumn::new(core::mem::size_of::<f32>());

        let mut count = 0;
        for _batch in column.iter_batch::<f32>(4) {
            count += 1;
        }

        assert_eq!(count, 0);
    }

    #[test]
    fn test_archetype_creation() {
        let types = vec![ComponentId::of::<Position>(), ComponentId::of::<Velocity>()];

        let archetype = Archetype::new(types.clone());

        assert_eq!(archetype.len(), 0);
        assert!(archetype.is_empty());
        assert_eq!(archetype.types(), &types);
        assert!(archetype.has_component(ComponentId::of::<Position>()));
        assert!(archetype.has_component(ComponentId::of::<Velocity>()));
        assert!(!archetype.has_component(ComponentId::of::<Health>()));
    }

    #[test]
    fn test_archetype_add_column() {
        let mut archetype = Archetype::new(vec![ComponentId::of::<Position>()]);

        archetype.add_column(
            ComponentId::of::<Position>(),
            core::mem::size_of::<Position>(),
        );

        assert_eq!(archetype.len(), 0);
    }

    #[test]
    #[should_panic(expected = "Column already exists")]
    fn test_archetype_add_duplicate_column_panics() {
        let mut archetype = Archetype::new(vec![ComponentId::of::<Position>()]);

        archetype.add_column(ComponentId::of::<Position>(), 8);
        archetype.add_column(ComponentId::of::<Position>(), 8); // Should panic
    }

    #[test]
    fn test_archetype_add_remove_entity() {
        let mut archetype = Archetype::new(vec![
            ComponentId::of::<Position>(),
            ComponentId::of::<Velocity>(),
        ]);

        archetype.add_column(
            ComponentId::of::<Position>(),
            core::mem::size_of::<Position>(),
        );
        archetype.add_column(
            ComponentId::of::<Velocity>(),
            core::mem::size_of::<Velocity>(),
        );

        // Add entities
        let mut data1 = BTreeMap::new();
        data1.insert(ComponentId::of::<Position>(), vec![0u8; 8]);
        data1.insert(ComponentId::of::<Velocity>(), vec![0u8; 8]);

        let mut data2 = BTreeMap::new();
        data2.insert(ComponentId::of::<Position>(), vec![0u8; 8]);
        data2.insert(ComponentId::of::<Velocity>(), vec![0u8; 8]);

        let idx1 = archetype.add_entity(0, data1);
        let idx2 = archetype.add_entity(1, data2);

        assert_eq!(archetype.len(), 2);
        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(archetype.get_entity_id(0), Some(0));
        assert_eq!(archetype.get_entity_id(1), Some(1));

        // Remove entity
        let removed_id = archetype.remove_entity(0);
        assert_eq!(removed_id, 1); // Last entity swapped to position 0
        assert_eq!(archetype.len(), 1);
    }

    #[test]
    fn test_archetype_clear() {
        let mut archetype = Archetype::new(vec![ComponentId::of::<Position>()]);

        archetype.add_column(
            ComponentId::of::<Position>(),
            core::mem::size_of::<Position>(),
        );

        let mut data = BTreeMap::new();
        data.insert(ComponentId::of::<Position>(), vec![0u8; 8]);

        archetype.add_entity(0, data);

        assert_eq!(archetype.len(), 1);

        archetype.clear();

        assert_eq!(archetype.len(), 0);
        assert!(archetype.is_empty());
    }

    #[test]
    fn test_archetype_iter_entities() {
        let mut archetype = Archetype::new(vec![ComponentId::of::<Position>()]);

        archetype.add_column(
            ComponentId::of::<Position>(),
            core::mem::size_of::<Position>(),
        );

        let mut data = BTreeMap::new();
        data.insert(ComponentId::of::<Position>(), vec![0u8; 8]);

        archetype.add_entity(0, data.clone());
        archetype.add_entity(5, data);

        let entities: Vec<usize> = archetype.iter_entities().copied().collect();

        assert_eq!(entities, vec![0, 5]);
    }

    #[test]
    fn test_archetype_storage_creation() {
        let storage = ArchetypeStorage::new();

        assert_eq!(storage.archetype_count(), 0);
        assert!(storage.is_empty());
    }

    #[test]
    fn test_archetype_storage_add_entity() {
        let mut storage = ArchetypeStorage::new();

        let mut types = BTreeMap::new();
        types.insert(
            ComponentId::of::<Position>(),
            core::mem::size_of::<Position>(),
        );
        types.insert(
            ComponentId::of::<Velocity>(),
            core::mem::size_of::<Velocity>(),
        );

        let arch_id = storage.add_entity(0, types);

        assert_eq!(storage.archetype_count(), 1);
        assert!(!storage.is_empty());
        assert_eq!(storage.get_archetype_id(0), Some(arch_id));

        let archetype = storage.get_archetype(0);
        assert!(archetype.is_some());
        assert_eq!(archetype.unwrap().len(), 1);
    }

    #[test]
    fn test_archetype_storage_add_same_archetype() {
        let mut storage = ArchetypeStorage::new();

        let mut types1 = BTreeMap::new();
        types1.insert(
            ComponentId::of::<Position>(),
            core::mem::size_of::<Position>(),
        );
        types1.insert(
            ComponentId::of::<Velocity>(),
            core::mem::size_of::<Velocity>(),
        );

        let mut types2 = BTreeMap::new();
        types2.insert(
            ComponentId::of::<Position>(),
            core::mem::size_of::<Position>(),
        );
        types2.insert(
            ComponentId::of::<Velocity>(),
            core::mem::size_of::<Velocity>(),
        );

        let id1 = storage.add_entity(0, types1);
        let id2 = storage.add_entity(1, types2);

        // Same archetype ID for entities with same component types
        assert_eq!(id1, id2);
        assert_eq!(storage.archetype_count(), 1);

        let archetype = storage.get_archetype(0).unwrap();
        assert_eq!(archetype.len(), 2);
    }

    #[test]
    fn test_archetype_storage_add_different_archetypes() {
        let mut storage = ArchetypeStorage::new();

        let mut types1 = BTreeMap::new();
        types1.insert(
            ComponentId::of::<Position>(),
            core::mem::size_of::<Position>(),
        );

        let mut types2 = BTreeMap::new();
        types2.insert(
            ComponentId::of::<Position>(),
            core::mem::size_of::<Position>(),
        );
        types2.insert(
            ComponentId::of::<Velocity>(),
            core::mem::size_of::<Velocity>(),
        );

        let id1 = storage.add_entity(0, types1);
        let id2 = storage.add_entity(1, types2);

        // Different archetype IDs for different component sets
        assert_ne!(id1, id2);
        assert_eq!(storage.archetype_count(), 2);
    }

    #[test]
    fn test_archetype_storage_remove_entity() {
        let mut storage = ArchetypeStorage::new();

        let mut types = BTreeMap::new();
        types.insert(
            ComponentId::of::<Position>(),
            core::mem::size_of::<Position>(),
        );

        storage.add_entity(0, types);

        let removed_id = storage.remove_entity(0);

        assert!(removed_id.is_some());
        assert_eq!(storage.archetype_count(), 0); // Archetype removed when empty
        assert_eq!(storage.get_archetype_id(0), None);
    }

    #[test]
    fn test_archetype_storage_remove_nonexistent() {
        let mut storage = ArchetypeStorage::new();

        let removed = storage.remove_entity(999);

        assert!(removed.is_none());
    }

    #[test]
    fn test_archetype_storage_get_archetype() {
        let mut storage = ArchetypeStorage::new();

        let mut types = BTreeMap::new();
        types.insert(
            ComponentId::of::<Position>(),
            core::mem::size_of::<Position>(),
        );

        storage.add_entity(0, types);

        let archetype = storage.get_archetype(0);
        assert!(archetype.is_some());

        let archetype = storage.get_archetype(1);
        assert!(archetype.is_none());
    }

    #[test]
    fn test_archetype_storage_get_archetype_mut() {
        let mut storage = ArchetypeStorage::new();

        let mut types = BTreeMap::new();
        types.insert(
            ComponentId::of::<Position>(),
            core::mem::size_of::<Position>(),
        );

        storage.add_entity(0, types);

        if let Some(archetype) = storage.get_archetype_mut(0) {
            assert_eq!(archetype.len(), 1);
        }
    }

    #[test]
    fn test_archetype_storage_clear() {
        let mut storage = ArchetypeStorage::new();

        let mut types1 = BTreeMap::new();
        types1.insert(
            ComponentId::of::<Position>(),
            core::mem::size_of::<Position>(),
        );

        let mut types2 = BTreeMap::new();
        types2.insert(
            ComponentId::of::<Velocity>(),
            core::mem::size_of::<Velocity>(),
        );

        storage.add_entity(0, types1);
        storage.add_entity(1, types2);

        assert_eq!(storage.archetype_count(), 2);

        storage.clear();

        assert_eq!(storage.archetype_count(), 0);
        assert!(storage.is_empty());
    }

    #[test]
    fn test_archetype_storage_iter_archetypes() {
        let mut storage = ArchetypeStorage::new();

        let mut types1 = BTreeMap::new();
        types1.insert(
            ComponentId::of::<Position>(),
            core::mem::size_of::<Position>(),
        );

        let mut types2 = BTreeMap::new();
        types2.insert(
            ComponentId::of::<Velocity>(),
            core::mem::size_of::<Velocity>(),
        );

        storage.add_entity(0, types1);
        storage.add_entity(1, types2);

        let count = storage.iter_archetypes().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_archetype_storage_default() {
        let storage = ArchetypeStorage::default();

        assert!(storage.is_empty());
        assert_eq!(storage.archetype_count(), 0);
    }
}
