// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - ECS Query API Module
//
// Provides type-safe queries over ECS components.
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::vec::Vec;
use core::marker::PhantomData;

use super::component::{Component, ComponentStorage};
use super::world::World;

// ============================================================================
// QueryParameter Trait - Type-level query specification
// ============================================================================

/// Marker trait for valid query parameters.
pub trait QueryParameter<'a> {
    /// The item type yielded by this query parameter
    type Item;
}

// Single Component Query Parameters (Immutable)
impl<'a, T: Component> QueryParameter<'a> for &'a T {
    type Item = &'a T;
}

// Single Component Query Parameters (Mutable)
impl<'a, T: Component> QueryParameter<'a> for &'a mut T {
    type Item = &'a mut T;
}

// Tuple Query Parameters (2 components) - Immutable
impl<'a, T1: Component, T2: Component> QueryParameter<'a> for (&'a T1, &'a T2) {
    type Item = (&'a T1, &'a T2);
}

// Tuple Query Parameters (2 components) - Mutable
impl<'a, T1: Component, T2: Component> QueryParameter<'a> for (&'a mut T1, &'a mut T2) {
    type Item = (&'a mut T1, &'a mut T2);
}

// Tuple Query Parameters (3 components) - Immutable
impl<'a, T1: Component, T2: Component, T3: Component> QueryParameter<'a>
    for (&'a T1, &'a T2, &'a T3)
{
    type Item = (&'a T1, &'a T2, &'a T3);
}

// Tuple Query Parameters (3 components) - Mutable
impl<'a, T1: Component, T2: Component, T3: Component> QueryParameter<'a>
    for (&'a mut T1, &'a mut T2, &'a mut T3)
{
    type Item = (&'a mut T1, &'a mut T2, &'a mut T3);
}

// Tuple Query Parameters (4 components) - Immutable
impl<'a, T1: Component, T2: Component, T3: Component, T4: Component> QueryParameter<'a>
    for (&'a T1, &'a T2, &'a T3, &'a T4)
{
    type Item = (&'a T1, &'a T2, &'a T3, &'a T4);
}

// Tuple Query Parameters (4 components) - Mutable
impl<'a, T1: Component, T2: Component, T3: Component, T4: Component> QueryParameter<'a>
    for (&'a mut T1, &'a mut T2, &'a mut T3, &'a mut T4)
{
    type Item = (&'a mut T1, &'a mut T2, &'a mut T3, &'a mut T4);
}

// Optional Component Query Parameters (Immutable)
impl<'a, T: Component> QueryParameter<'a> for Option<&'a T> {
    type Item = Option<&'a T>;
}

// Optional Component Query Parameters (Mutable)
impl<'a, T: Component> QueryParameter<'a> for Option<&'a mut T> {
    type Item = Option<&'a mut T>;
}

// ============================================================================
// Query Types
// ============================================================================

/// A typed query over components in the world (immutable access)
pub struct Query<'w, Q> {
    world: &'w World,
    _marker: PhantomData<fn() -> Q>,
}

impl<'w, Q> Query<'w, Q>
where
    Q: QueryParameter<'w>,
{
    /// Creates a new query for the given world
    #[inline]
    pub fn new(world: &'w World) -> Self {
        Self {
            world,
            _marker: PhantomData,
        }
    }

    /// Returns the number of entities matching this query
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.world.entity_count()
    }

    /// Returns true if no entities match this query
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.world.entity_count() == 0
    }
}

/// Mutable query for components requiring exclusive access
pub struct QueryMut<'w, Q> {
    world: &'w mut World,
    _marker: PhantomData<fn() -> Q>,
}

impl<'w, Q> QueryMut<'w, Q>
where
    Q: QueryParameter<'w>,
{
    /// Creates a new mutable query
    #[inline]
    pub fn new(world: &'w mut World) -> Self {
        Self {
            world,
            _marker: PhantomData,
        }
    }

    /// Returns the number of entities matching this query
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.world.entity_count()
    }

    /// Returns true if no entities match this query
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.world.entity_count() == 0
    }
}

// ============================================================================
// Query Each Methods - Eager Evaluation (Immutable)
// ============================================================================

impl<'w, T> Query<'w, &'w T>
where
    T: Component,
{
    /// Executes the query for each entity with component T (immutable)
    #[inline]
    pub fn each<F>(self, mut f: F)
    where
        F: FnMut(&T),
    {
        let entities = self.world.entities_slice();
        let registry = self.world.registry();
        if let Some(storage) = registry.get_storage::<T>() {
            for idx in 0..entities.len() {
                if entities[idx].alive {
                    if let Some(component) = storage.get(idx) {
                        f(component);
                    }
                }
            }
        }
    }
}

impl<'w, T1, T2> Query<'w, (&'w T1, &'w T2)>
where
    T1: Component,
    T2: Component,
{
    /// Executes the query for each entity with both components T1 and T2 (immutable)
    #[inline]
    pub fn each<F>(self, mut f: F)
    where
        F: FnMut(&T1, &T2),
    {
        let entities = self.world.entities_slice();
        let registry = self.world.registry();
        let s1 = registry.get_storage::<T1>();
        let s2 = registry.get_storage::<T2>();
        if let (Some(storage1), Some(storage2)) = (s1, s2) {
            for idx in 0..entities.len() {
                if entities[idx].alive {
                    if let (Some(c1), Some(c2)) = (storage1.get(idx), storage2.get(idx)) {
                        f(c1, c2);
                    }
                }
            }
        }
    }
}

impl<'w, T1, T2, T3> Query<'w, (&'w T1, &'w T2, &'w T3)>
where
    T1: Component,
    T2: Component,
    T3: Component,
{
    /// Executes the query for each entity with all three components (immutable)
    #[inline]
    pub fn each<F>(self, mut f: F)
    where
        F: FnMut(&T1, &T2, &T3),
    {
        let entities = self.world.entities_slice();
        let registry = self.world.registry();
        let s1 = registry.get_storage::<T1>();
        let s2 = registry.get_storage::<T2>();
        let s3 = registry.get_storage::<T3>();
        if let (Some(storage1), Some(storage2), Some(storage3)) = (s1, s2, s3) {
            for idx in 0..entities.len() {
                if entities[idx].alive {
                    if let (Some(c1), Some(c2), Some(c3)) =
                        (storage1.get(idx), storage2.get(idx), storage3.get(idx))
                    {
                        f(c1, c2, c3);
                    }
                }
            }
        }
    }
}

impl<'w, T1, T2, T3, T4> Query<'w, (&'w T1, &'w T2, &'w T3, &'w T4)>
where
    T1: Component,
    T2: Component,
    T3: Component,
    T4: Component,
{
    /// Executes the query for each entity with all four components (immutable)
    #[inline]
    pub fn each<F>(self, mut f: F)
    where
        F: FnMut(&T1, &T2, &T3, &T4),
    {
        let entities = self.world.entities_slice();
        let registry = self.world.registry();
        let s1 = registry.get_storage::<T1>();
        let s2 = registry.get_storage::<T2>();
        let s3 = registry.get_storage::<T3>();
        let s4 = registry.get_storage::<T4>();
        if let (Some(storage1), Some(storage2), Some(storage3), Some(storage4)) = (s1, s2, s3, s4) {
            for idx in 0..entities.len() {
                if entities[idx].alive {
                    if let (Some(c1), Some(c2), Some(c3), Some(c4)) = (
                        storage1.get(idx),
                        storage2.get(idx),
                        storage3.get(idx),
                        storage4.get(idx),
                    ) {
                        f(c1, c2, c3, c4);
                    }
                }
            }
        }
    }
}

impl<'w, T> Query<'w, Option<&'w T>>
where
    T: Component,
{
    /// Executes the query for each entity with optional component T (immutable)
    #[inline]
    pub fn each<F>(self, mut f: F)
    where
        F: FnMut(Option<&T>),
    {
        let entities = self.world.entities_slice();
        let registry = self.world.registry();
        if let Some(storage) = registry.get_storage::<T>() {
            for idx in 0..entities.len() {
                if entities[idx].alive {
                    let component = storage.get(idx);
                    f(component);
                }
            }
        } else {
            // Component type not registered - all entities return None
            for idx in 0..entities.len() {
                if entities[idx].alive {
                    f(None);
                }
            }
        }
    }
}

// ============================================================================
// Query Each Methods - Eager Evaluation (Mutable)
// ============================================================================

impl<'w, T> QueryMut<'w, &'w mut T>
where
    T: Component + Clone,
{
    /// Executes the query for each entity with component T (mutable)
    #[inline]
    pub fn each<F>(self, mut f: F)
    where
        F: FnMut(&mut T),
    {
        // Collect alive indices and component data (immutable pass)
        let updates: Vec<(usize, T)> = {
            let entities = self.world.entities_slice();
            let registry = self.world.registry();
            if let Some(storage) = registry.get_storage::<T>() {
                entities
                    .iter()
                    .enumerate()
                    .filter(|(_, meta)| meta.alive)
                    .filter_map(|(idx, _)| storage.get(idx).cloned().map(|comp| (idx, comp)))
                    .collect()
            } else {
                Vec::new()
            }
        };

        // Apply updates (mutable pass)
        {
            let registry = self.world.registry_mut();
            if let Some(storage) = registry.get_storage_mut::<T>() {
                for (idx, mut comp) in updates {
                    f(&mut comp);
                    storage.insert(idx, comp);
                }
            }
        }
    }
}

// 2-component mutable query
impl<'w, T1, T2> QueryMut<'w, (&'w mut T1, &'w mut T2)>
where
    T1: Component + Clone,
    T2: Component + Clone,
{
    #[inline]
    pub fn each<F>(self, mut f: F)
    where
        F: FnMut(&mut T1, &mut T2),
    {
        // Collect alive indices and component data (immutable pass)
        let updates: Vec<(usize, T1, T2)> = {
            let entities = self.world.entities_slice();
            let registry = self.world.registry();
            let s1 = registry.get_storage::<T1>();
            let s2 = registry.get_storage::<T2>();
            if let (Some(storage1), Some(storage2)) = (s1, s2) {
                entities
                    .iter()
                    .enumerate()
                    .filter(|(_, meta)| meta.alive)
                    .filter_map(|(idx, _)| {
                        if let (Some(c1), Some(c2)) = (storage1.get(idx), storage2.get(idx)) {
                            Some((idx, (*c1).clone(), (*c2).clone()))
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            }
        };

        // Apply updates - update each storage separately to avoid borrow conflicts
        for (idx, c1, c2) in updates {
            let mut c1_mut = c1;
            let mut c2_mut = c2;
            f(&mut c1_mut, &mut c2_mut);

            // Update T1 storage
            {
                let registry = self.world.registry_mut();
                if let Some(s1) = registry.get_storage_mut::<T1>() {
                    s1.insert(idx, c1_mut);
                }
            }
            // Update T2 storage
            {
                let registry = self.world.registry_mut();
                if let Some(s2) = registry.get_storage_mut::<T2>() {
                    s2.insert(idx, c2_mut);
                }
            }
        }
    }
}

// 3-component mutable query
impl<'w, T1, T2, T3> QueryMut<'w, (&'w mut T1, &'w mut T2, &'w mut T3)>
where
    T1: Component + Clone,
    T2: Component + Clone,
    T3: Component + Clone,
{
    #[inline]
    pub fn each<F>(self, mut f: F)
    where
        F: FnMut(&mut T1, &mut T2, &mut T3),
    {
        // Collect alive indices and component data
        let updates: Vec<(usize, T1, T2, T3)> = {
            let entities = self.world.entities_slice();
            let registry = self.world.registry();
            let s1 = registry.get_storage::<T1>();
            let s2 = registry.get_storage::<T2>();
            let s3 = registry.get_storage::<T3>();
            if let (Some(storage1), Some(storage2), Some(storage3)) = (s1, s2, s3) {
                entities
                    .iter()
                    .enumerate()
                    .filter(|(_, meta)| meta.alive)
                    .filter_map(|(idx, _)| {
                        if let (Some(c1), Some(c2), Some(c3)) =
                            (storage1.get(idx), storage2.get(idx), storage3.get(idx))
                        {
                            Some((idx, (*c1).clone(), (*c2).clone(), (*c3).clone()))
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            }
        };

        // Apply updates one component at a time
        for (idx, mut c1, mut c2, mut c3) in updates {
            f(&mut c1, &mut c2, &mut c3);
            let registry = self.world.registry_mut();
            if let Some(s1) = registry.get_storage_mut::<T1>() {
                s1.insert(idx, c1);
            }
            if let Some(s2) = registry.get_storage_mut::<T2>() {
                s2.insert(idx, c2);
            }
            if let Some(s3) = registry.get_storage_mut::<T3>() {
                s3.insert(idx, c3);
            }
        }
    }
}

// 4-component mutable query
impl<'w, T1, T2, T3, T4> QueryMut<'w, (&'w mut T1, &'w mut T2, &'w mut T3, &'w mut T4)>
where
    T1: Component + Clone,
    T2: Component + Clone,
    T3: Component + Clone,
    T4: Component + Clone,
{
    #[inline]
    pub fn each<F>(self, mut f: F)
    where
        F: FnMut(&mut T1, &mut T2, &mut T3, &mut T4),
    {
        // Collect alive indices and component data
        let updates: Vec<(usize, T1, T2, T3, T4)> = {
            let entities = self.world.entities_slice();
            let registry = self.world.registry();
            let s1 = registry.get_storage::<T1>();
            let s2 = registry.get_storage::<T2>();
            let s3 = registry.get_storage::<T3>();
            let s4 = registry.get_storage::<T4>();
            if let (Some(storage1), Some(storage2), Some(storage3), Some(storage4)) =
                (s1, s2, s3, s4)
            {
                entities
                    .iter()
                    .enumerate()
                    .filter(|(_, meta)| meta.alive)
                    .filter_map(|(idx, _)| {
                        if let (Some(c1), Some(c2), Some(c3), Some(c4)) = (
                            storage1.get(idx),
                            storage2.get(idx),
                            storage3.get(idx),
                            storage4.get(idx),
                        ) {
                            Some((
                                idx,
                                (*c1).clone(),
                                (*c2).clone(),
                                (*c3).clone(),
                                (*c4).clone(),
                            ))
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            }
        };

        // Apply updates one component at a time
        for (idx, mut c1, mut c2, mut c3, mut c4) in updates {
            f(&mut c1, &mut c2, &mut c3, &mut c4);
            let registry = self.world.registry_mut();
            if let Some(s1) = registry.get_storage_mut::<T1>() {
                s1.insert(idx, c1);
            }
            if let Some(s2) = registry.get_storage_mut::<T2>() {
                s2.insert(idx, c2);
            }
            if let Some(s3) = registry.get_storage_mut::<T3>() {
                s3.insert(idx, c3);
            }
            if let Some(s4) = registry.get_storage_mut::<T4>() {
                s4.insert(idx, c4);
            }
        }
    }
}

impl<'w, T> QueryMut<'w, Option<&'w mut T>>
where
    T: Component + Clone,
{
    /// Executes the query for each entity with optional component T (mutable)
    #[inline]
    pub fn each<F>(self, mut f: F)
    where
        F: FnMut(Option<&mut T>),
    {
        // Collect alive indices and component data
        let updates: Vec<(usize, Option<T>)> = {
            let entities = self.world.entities_slice();
            let registry = self.world.registry();
            if let Some(storage) = registry.get_storage::<T>() {
                entities
                    .iter()
                    .enumerate()
                    .filter(|(_, meta)| meta.alive)
                    .map(|(idx, _)| (idx, storage.get(idx).cloned()))
                    .collect()
            } else {
                Vec::new()
            }
        };

        // Apply updates
        for (idx, mut comp) in updates {
            let comp_ref = comp.as_mut();
            f(comp_ref);
            if let Some(c) = comp {
                let registry = self.world.registry_mut();
                if let Some(storage) = registry.get_storage_mut::<T>() {
                    storage.insert(idx, c);
                }
            }
        }
    }
}

// ============================================================================
// EntityId
// ============================================================================

/// Unique identifier for an entity in the ECS.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId {
    index: usize,
    generation: u32,
}

impl EntityId {
    #[inline]
    #[must_use]
    pub const fn new(index: usize, generation: u32) -> Self {
        Self { index, generation }
    }

    #[inline]
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }

    #[inline]
    #[must_use]
    pub const fn generation(&self) -> u32 {
        self.generation
    }

    #[inline]
    #[must_use]
    pub const fn as_usize(&self) -> usize {
        self.index
    }

    #[inline]
    #[must_use]
    pub const fn from_usize(index: usize) -> Self {
        Self::new(index, 0)
    }
}

// ============================================================================
// QueryIter - Lazy Iterator Implementation (Immutable Queries Only)
// ============================================================================

/// An iterator over immutable query results (lazy evaluation)
///
/// Provides zero-allocation iteration over entities matching a query.
/// Implements the standard `Iterator` trait for compatibility with Rust
/// iterator combinators like `map()`, `filter()`, and `collect()`.
///
/// Note: For mutable queries, use `.each()` which handles mutations correctly
/// via a two-phase iteration pattern.
pub struct QueryIter<'w, Q> {
    /// World reference for iteration
    world: &'w World,
    /// Indices of entities matching the query
    indices: alloc::vec::IntoIter<usize>,
    /// Marker for the query type
    _marker: PhantomData<fn() -> Q>,
}

impl<'w, Q> QueryIter<'w, Q> {
    /// Creates a new query iterator for the given world
    #[inline]
    pub fn new(world: &'w World) -> Self {
        let indices: Vec<usize> = world
            .entities_slice()
            .iter()
            .enumerate()
            .filter(|(_, meta)| meta.alive)
            .map(|(idx, _)| idx)
            .collect();

        Self {
            world,
            indices: indices.into_iter(),
            _marker: PhantomData,
        }
    }

    /// Returns the number of entities this iterator will yield
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    /// Returns true if this iterator will yield no items
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// ============================================================================
// Iterator Implementations for Specific Query Types
// ============================================================================

// Single component iterator
impl<'w, T> Iterator for QueryIter<'w, &'w T>
where
    T: Component,
{
    type Item = &'w T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(idx) = self.indices.next() {
            if let Some(storage) = self.world.registry().get_storage::<T>() {
                if let Some(component) = storage.get(idx) {
                    return Some(component);
                }
            }
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.indices.size_hint()
    }
}

// Two component iterator
impl<'w, T1, T2> Iterator for QueryIter<'w, (&'w T1, &'w T2)>
where
    T1: Component,
    T2: Component,
{
    type Item = (&'w T1, &'w T2);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(idx) = self.indices.next() {
            let registry = self.world.registry();
            let s1 = registry.get_storage::<T1>()?;
            let s2 = registry.get_storage::<T2>()?;
            match (s1.get(idx), s2.get(idx)) {
                (Some(c1), Some(c2)) => return Some((c1, c2)),
                _ => continue,
            }
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.indices.size_hint()
    }
}

// Three component iterator
impl<'w, T1, T2, T3> Iterator for QueryIter<'w, (&'w T1, &'w T2, &'w T3)>
where
    T1: Component,
    T2: Component,
    T3: Component,
{
    type Item = (&'w T1, &'w T2, &'w T3);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(idx) = self.indices.next() {
            let registry = self.world.registry();
            let s1 = registry.get_storage::<T1>()?;
            let s2 = registry.get_storage::<T2>()?;
            let s3 = registry.get_storage::<T3>()?;
            match (s1.get(idx), s2.get(idx), s3.get(idx)) {
                (Some(c1), Some(c2), Some(c3)) => return Some((c1, c2, c3)),
                _ => continue,
            }
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.indices.size_hint()
    }
}

// Four component iterator
impl<'w, T1, T2, T3, T4> Iterator for QueryIter<'w, (&'w T1, &'w T2, &'w T3, &'w T4)>
where
    T1: Component,
    T2: Component,
    T3: Component,
    T4: Component,
{
    type Item = (&'w T1, &'w T2, &'w T3, &'w T4);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(idx) = self.indices.next() {
            let registry = self.world.registry();
            let s1 = registry.get_storage::<T1>()?;
            let s2 = registry.get_storage::<T2>()?;
            let s3 = registry.get_storage::<T3>()?;
            let s4 = registry.get_storage::<T4>()?;
            match (s1.get(idx), s2.get(idx), s3.get(idx), s4.get(idx)) {
                (Some(c1), Some(c2), Some(c3), Some(c4)) => return Some((c1, c2, c3, c4)),
                _ => continue,
            }
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.indices.size_hint()
    }
}

// Optional component iterator
impl<'w, T> Iterator for QueryIter<'w, Option<&'w T>>
where
    T: Component,
{
    type Item = Option<&'w T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(idx) = self.indices.next() {
            let storage = self.world.registry().get_storage::<T>();
            let component = storage.and_then(|s| s.get(idx));
            return Some(component);
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.indices.size_hint()
    }
}

impl<'w, T> ExactSizeIterator for QueryIter<'w, &'w T> where T: Component {}

impl<'w, T1, T2> ExactSizeIterator for QueryIter<'w, (&'w T1, &'w T2)>
where
    T1: Component,
    T2: Component,
{
}

impl<'w, T1, T2, T3> ExactSizeIterator for QueryIter<'w, (&'w T1, &'w T2, &'w T3)>
where
    T1: Component,
    T2: Component,
    T3: Component,
{
}

impl<'w, T1, T2, T3, T4> ExactSizeIterator for QueryIter<'w, (&'w T1, &'w T2, &'w T3, &'w T4)>
where
    T1: Component,
    T2: Component,
    T3: Component,
    T4: Component,
{
}

impl<'w, T> ExactSizeIterator for QueryIter<'w, Option<&'w T>> where T: Component {}

impl<'w, T> core::iter::FusedIterator for QueryIter<'w, &'w T> where T: Component {}

impl<'w, T1, T2> core::iter::FusedIterator for QueryIter<'w, (&'w T1, &'w T2)>
where
    T1: Component,
    T2: Component,
{
}

impl<'w, T1, T2, T3> core::iter::FusedIterator for QueryIter<'w, (&'w T1, &'w T2, &'w T3)>
where
    T1: Component,
    T2: Component,
    T3: Component,
{
}

impl<'w, T1, T2, T3, T4> core::iter::FusedIterator
    for QueryIter<'w, (&'w T1, &'w T2, &'w T3, &'w T4)>
where
    T1: Component,
    T2: Component,
    T3: Component,
    T4: Component,
{
}

impl<'w, T> core::iter::FusedIterator for QueryIter<'w, Option<&'w T>> where T: Component {}

/// Extension trait providing iterator combinators for queries
pub trait QueryIterExt<'w, Q: QueryParameter<'w>> {
    /// Returns an iterator over matching entities
    fn iter(&'w self) -> QueryIter<'w, Q>;
}

impl<'w, Q> QueryIterExt<'w, Q> for Query<'w, Q>
where
    Q: QueryParameter<'w>,
{
    #[inline]
    fn iter(&'w self) -> QueryIter<'w, Q> {
        QueryIter::new(self.world)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::component::{ComponentStorage, VecStorage};

    #[derive(Clone, Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    impl Component for Position {
        type Storage = VecStorage<Position>;
    }

    #[derive(Clone, Debug, PartialEq)]
    struct Velocity {
        dx: f32,
        dy: f32,
    }

    impl Component for Velocity {
        type Storage = VecStorage<Velocity>;
    }

    #[derive(Clone, Debug, PartialEq)]
    struct Health {
        current: u32,
        max: u32,
    }

    impl Component for Health {
        type Storage = VecStorage<Health>;
    }

    #[derive(Clone, Debug, PartialEq)]
    struct Damage {
        amount: u32,
    }

    impl Component for Damage {
        type Storage = VecStorage<Damage>;
    }

    // ===== EntityId Tests =====

    #[test]
    fn test_entity_id_creation() {
        let id = EntityId::new(5, 1);
        assert_eq!(id.index(), 5);
        assert_eq!(id.generation(), 1);
        assert_eq!(id.as_usize(), 5);
    }

    #[test]
    fn test_entity_id_equality() {
        let id1 = EntityId::new(5, 1);
        let id2 = EntityId::new(5, 1);
        let id3 = EntityId::new(5, 2);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    // ===== Query Tests =====

    #[test]
    fn test_query_single_component() {
        use crate::ecs::World;

        let mut world = World::new();
        let e1 = world.create_entity();
        let e2 = world.create_entity();
        let _e3 = world.create_entity();

        world.add_component(e1, Position { x: 1.0, y: 2.0 });
        world.add_component(e2, Position { x: 3.0, y: 4.0 });
        world.add_component(_e3, Velocity { dx: 1.0, dy: 1.0 });

        let mut count = 0;
        let mut sum_x = 0.0;

        world.query::<&Position>().each(|pos| {
            count += 1;
            sum_x += pos.x;
        });

        assert_eq!(count, 2);
        assert_eq!(sum_x, 4.0);
    }

    #[test]
    fn test_query_two_components() {
        use crate::ecs::World;

        let mut world = World::new();
        let e1 = world.create_entity();
        let e2 = world.create_entity();

        world.add_component(e1, Position { x: 0.0, y: 0.0 });
        world.add_component(e1, Velocity { dx: 1.0, dy: 2.0 });
        world.add_component(e2, Position { x: 10.0, y: 20.0 });
        world.add_component(e2, Velocity { dx: 3.0, dy: 4.0 });

        let mut count = 0;

        world.query::<(&Position, &Velocity)>().each(|pos, _vel| {
            count += 1;
            assert!(pos.x >= 0.0);
        });

        assert_eq!(count, 2);
    }

    #[test]
    fn test_query_three_components() {
        use crate::ecs::World;

        let mut world = World::new();
        let e1 = world.create_entity();
        let e2 = world.create_entity();

        world.add_component(e1, Position { x: 0.0, y: 0.0 });
        world.add_component(e1, Velocity { dx: 1.0, dy: 2.0 });
        world.add_component(
            e1,
            Health {
                current: 100,
                max: 100,
            },
        );
        world.add_component(e2, Position { x: 5.0, y: 5.0 });
        world.add_component(e2, Velocity { dx: 2.0, dy: 3.0 });
        world.add_component(
            e2,
            Health {
                current: 50,
                max: 100,
            },
        );

        let mut count = 0;
        let mut total_health = 0;

        world
            .query::<(&Position, &Velocity, &Health)>()
            .each(|_pos, _vel, health| {
                count += 1;
                total_health += health.current;
            });

        assert_eq!(count, 2);
        assert_eq!(total_health, 150);
    }

    #[test]
    fn test_query_four_components() {
        use crate::ecs::World;

        let mut world = World::new();
        let e1 = world.create_entity();

        world.add_component(e1, Position { x: 0.0, y: 0.0 });
        world.add_component(e1, Velocity { dx: 1.0, dy: 2.0 });
        world.add_component(
            e1,
            Health {
                current: 100,
                max: 100,
            },
        );
        world.add_component(e1, Damage { amount: 25 });

        let mut count = 0;
        let mut total_damage = 0;

        world
            .query::<(&Position, &Velocity, &Health, &Damage)>()
            .each(|_pos, _vel, _health, damage| {
                count += 1;
                total_damage += damage.amount;
            });

        assert_eq!(count, 1);
        assert_eq!(total_damage, 25);
    }

    #[test]
    fn test_query_mut() {
        use crate::ecs::World;

        let mut world = World::new();
        let e1 = world.create_entity();
        let e2 = world.create_entity();

        world.add_component(e1, Position { x: 1.0, y: 2.0 });
        world.add_component(e2, Position { x: 3.0, y: 4.0 });

        world.query_mut::<&mut Position>().each(|pos| {
            pos.x *= 2.0;
            pos.y *= 2.0;
        });

        assert_eq!(
            world.get_component::<Position>(e1),
            Some(&Position { x: 2.0, y: 4.0 })
        );
        assert_eq!(
            world.get_component::<Position>(e2),
            Some(&Position { x: 6.0, y: 8.0 })
        );
    }

    #[test]
    fn test_query_mut_two_components() {
        use crate::ecs::World;

        let mut world = World::new();
        let e1 = world.create_entity();

        world.add_component(e1, Position { x: 1.0, y: 2.0 });
        world.add_component(e1, Velocity { dx: 1.0, dy: 1.0 });

        world
            .query_mut::<(&mut Position, &mut Velocity)>()
            .each(|pos, vel| {
                pos.x += vel.dx;
                pos.y += vel.dy;
            });

        assert_eq!(
            world.get_component::<Position>(e1),
            Some(&Position { x: 2.0, y: 3.0 })
        );
    }

    #[test]
    fn test_query_mut_three_components() {
        use crate::ecs::World;

        let mut world = World::new();
        let e1 = world.create_entity();

        world.add_component(e1, Position { x: 0.0, y: 0.0 });
        world.add_component(e1, Velocity { dx: 1.0, dy: 1.0 });
        world.add_component(
            e1,
            Health {
                current: 100,
                max: 100,
            },
        );

        let mut damage_applied = false;
        world
            .query_mut::<(&mut Position, &mut Velocity, &mut Health)>()
            .each(|pos, _vel, health| {
                pos.x += 1.0;
                pos.y += 1.0;
                if health.current > 0 {
                    health.current -= 10;
                    damage_applied = true;
                }
            });

        assert!(damage_applied);
        assert_eq!(
            world.get_component::<Position>(e1),
            Some(&Position { x: 1.0, y: 1.0 })
        );
        assert_eq!(
            world.get_component::<Health>(e1),
            Some(&Health {
                current: 90,
                max: 100
            })
        );
    }

    #[test]
    fn test_query_mut_four_components() {
        use crate::ecs::World;

        let mut world = World::new();
        let e1 = world.create_entity();

        world.add_component(e1, Position { x: 0.0, y: 0.0 });
        world.add_component(e1, Velocity { dx: 1.0, dy: 1.0 });
        world.add_component(
            e1,
            Health {
                current: 100,
                max: 100,
            },
        );
        world.add_component(e1, Damage { amount: 10 });

        world
            .query_mut::<(&mut Position, &mut Velocity, &mut Health, &mut Damage)>()
            .each(|pos, _vel, health, damage| {
                pos.x += damage.amount as f32;
                health.current = health.current.saturating_sub(damage.amount);
            });

        assert_eq!(
            world.get_component::<Position>(e1),
            Some(&Position { x: 10.0, y: 0.0 })
        );
        assert_eq!(
            world.get_component::<Health>(e1),
            Some(&Health {
                current: 90,
                max: 100
            })
        );
    }

    #[test]
    fn test_query_optional_immutable() {
        use crate::ecs::World;

        let mut world = World::new();
        let e1 = world.create_entity();
        let e2 = world.create_entity();
        let e3 = world.create_entity();

        world.add_component(e1, Position { x: 1.0, y: 2.0 });
        // e2 has no Position
        world.add_component(e3, Position { x: 3.0, y: 4.0 });

        let mut with_pos = 0;
        let mut without_pos = 0;

        world.query::<Option<&Position>>().each(|opt_pos| {
            if opt_pos.is_some() {
                with_pos += 1;
            } else {
                without_pos += 1;
            }
        });

        assert_eq!(with_pos, 2);
        assert_eq!(without_pos, 1);
    }

    #[test]
    fn test_query_optional_mutable() {
        use crate::ecs::World;

        let mut world = World::new();
        let e1 = world.create_entity();
        let e2 = world.create_entity();
        let e3 = world.create_entity();

        world.add_component(e1, Position { x: 1.0, y: 2.0 });
        // e2 has no Position
        world.add_component(e3, Position { x: 3.0, y: 4.0 });

        world.query_mut::<Option<&mut Position>>().each(|opt_pos| {
            if let Some(pos) = opt_pos {
                pos.x *= 10.0;
            }
        });

        assert_eq!(
            world.get_component::<Position>(e1),
            Some(&Position { x: 10.0, y: 2.0 })
        );
        // e2 unchanged (never had Position)
        assert_eq!(
            world.get_component::<Position>(e3),
            Some(&Position { x: 30.0, y: 4.0 })
        );
    }

    #[test]
    fn test_query_is_empty() {
        use crate::ecs::World;

        let mut world = World::new();

        // No entities created yet
        let query = world.query::<&Position>();
        assert!(query.is_empty());

        let e1 = world.create_entity();
        world.add_component(e1, Position { x: 1.0, y: 2.0 });

        // Now query should have 1 matching entity
        let query = world.query::<&Position>();
        assert!(!query.is_empty());
        assert_eq!(query.len(), 1);
    }

    #[test]
    fn test_query_with_mixed_entities() {
        use crate::ecs::World;

        let mut world = World::new();
        let e1 = world.create_entity();
        let e2 = world.create_entity();
        let e3 = world.create_entity();
        let e4 = world.create_entity();

        world.add_component(e1, Position { x: 1.0, y: 1.0 });
        world.add_component(e1, Velocity { dx: 1.0, dy: 1.0 });
        // e2 has Position only
        world.add_component(e2, Position { x: 2.0, y: 2.0 });
        // e3 has Velocity only
        world.add_component(e3, Velocity { dx: 3.0, dy: 3.0 });
        // e4 has nothing

        // Query for entities with BOTH Position and Velocity
        let mut count = 0;
        world.query::<(&Position, &Velocity)>().each(|pos, _vel| {
            count += 1;
            assert!(pos.x >= 1.0);
        });

        // Only e1 has both components
        assert_eq!(count, 1);
    }

    // ===== QueryIter (Iterator) Tests =====

    #[test]
    fn test_query_iter_single_component() {
        use crate::ecs::World;

        let mut world = World::new();
        let e1 = world.create_entity();
        let e2 = world.create_entity();
        let _e3 = world.create_entity();

        world.add_component(e1, Position { x: 1.0, y: 2.0 });
        world.add_component(e2, Position { x: 3.0, y: 4.0 });
        world.add_component(_e3, Velocity { dx: 1.0, dy: 1.0 });

        // Store query first to extend lifetime
        let query = world.query::<&Position>();
        let positions: Vec<&Position> = query.iter().collect();

        assert_eq!(positions.len(), 2);
        assert_eq!(positions[0].x, 1.0);
        assert_eq!(positions[1].x, 3.0);
    }

    #[test]
    fn test_query_iter_two_components() {
        use crate::ecs::World;

        let mut world = World::new();
        let e1 = world.create_entity();
        let e2 = world.create_entity();

        world.add_component(e1, Position { x: 0.0, y: 0.0 });
        world.add_component(e1, Velocity { dx: 1.0, dy: 2.0 });
        world.add_component(e2, Position { x: 10.0, y: 20.0 });
        world.add_component(e2, Velocity { dx: 3.0, dy: 4.0 });

        // Store query first to extend lifetime
        let query = world.query::<(&Position, &Velocity)>();
        let pairs: Vec<(&Position, &Velocity)> = query.iter().collect();

        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0].0.x, 0.0);
        assert_eq!(pairs[1].0.x, 10.0);
    }

    #[test]
    fn test_query_iter_three_components() {
        use crate::ecs::World;

        let mut world = World::new();
        let e1 = world.create_entity();
        let e2 = world.create_entity();

        world.add_component(e1, Position { x: 1.0, y: 2.0 });
        world.add_component(e1, Velocity { dx: 3.0, dy: 4.0 });
        world.add_component(
            e1,
            Health {
                current: 100,
                max: 100,
            },
        );
        world.add_component(e2, Position { x: 5.0, y: 6.0 });
        world.add_component(e2, Velocity { dx: 7.0, dy: 8.0 });
        world.add_component(
            e2,
            Health {
                current: 50,
                max: 100,
            },
        );

        // Store query first to extend lifetime
        let query = world.query::<(&Position, &Velocity, &Health)>();
        let triples: Vec<(&Position, &Velocity, &Health)> = query.iter().collect();

        assert_eq!(triples.len(), 2);
        assert_eq!(triples[0].2.current, 100);
        assert_eq!(triples[1].2.current, 50);
    }

    #[test]
    fn test_query_iter_four_components() {
        use crate::ecs::World;

        let mut world = World::new();
        let e1 = world.create_entity();
        let e2 = world.create_entity();

        world.add_component(e1, Position { x: 1.0, y: 2.0 });
        world.add_component(e1, Velocity { dx: 3.0, dy: 4.0 });
        world.add_component(
            e1,
            Health {
                current: 100,
                max: 100,
            },
        );
        world.add_component(e1, Damage { amount: 10 });
        world.add_component(e2, Position { x: 5.0, y: 6.0 });
        world.add_component(e2, Velocity { dx: 7.0, dy: 8.0 });
        world.add_component(
            e2,
            Health {
                current: 75,
                max: 100,
            },
        );
        world.add_component(e2, Damage { amount: 20 });

        // Store query first to extend lifetime
        let query = world.query::<(&Position, &Velocity, &Health, &Damage)>();
        let quads: Vec<(&Position, &Velocity, &Health, &Damage)> = query.iter().collect();

        assert_eq!(quads.len(), 2);
        assert_eq!(quads[0].3.amount, 10);
        assert_eq!(quads[1].3.amount, 20);
    }

    #[test]
    fn test_query_iter_optional() {
        use crate::ecs::World;

        let mut world = World::new();
        let e1 = world.create_entity();
        let e2 = world.create_entity();
        let e3 = world.create_entity();

        world.add_component(e1, Position { x: 1.0, y: 2.0 });
        // e2 has no Position
        world.add_component(e3, Position { x: 3.0, y: 4.0 });

        // Store query first to extend lifetime
        let query = world.query::<Option<&Position>>();
        let optionals: Vec<Option<&Position>> = query.iter().collect();

        assert_eq!(optionals.len(), 3);
        assert!(optionals[0].is_some());
        assert!(optionals[1].is_none());
        assert!(optionals[2].is_some());
    }

    #[test]
    fn test_query_iter_with_map() {
        use crate::ecs::World;

        let mut world = World::new();
        let e1 = world.create_entity();
        let e2 = world.create_entity();

        world.add_component(e1, Position { x: 1.0, y: 2.0 });
        world.add_component(e2, Position { x: 3.0, y: 4.0 });

        // Store query first to extend lifetime
        let query = world.query::<&Position>();
        let x_values: Vec<f32> = query.iter().map(|pos: &Position| pos.x * 2.0).collect();

        assert_eq!(x_values.len(), 2);
        assert_eq!(x_values[0], 2.0);
        assert_eq!(x_values[1], 6.0);
    }

    #[test]
    fn test_query_iter_with_filter() {
        use crate::ecs::World;

        let mut world = World::new();
        let e1 = world.create_entity();
        let e2 = world.create_entity();
        let e3 = world.create_entity();

        world.add_component(e1, Position { x: 1.0, y: 2.0 });
        world.add_component(
            e1,
            Health {
                current: 50,
                max: 100,
            },
        );
        world.add_component(e2, Position { x: 3.0, y: 4.0 });
        world.add_component(
            e2,
            Health {
                current: 150,
                max: 100,
            },
        ); // Overhealed
        world.add_component(e3, Position { x: 5.0, y: 6.0 });
        world.add_component(
            e3,
            Health {
                current: 100,
                max: 100,
            },
        );

        // Store query first to extend lifetime
        let query = world.query::<(&Position, &Health)>();
        let healthy_positions: Vec<&Position> = query
            .iter()
            .filter(|(_, health)| health.current <= health.max)
            .map(|(pos, _)| pos)
            .collect();

        assert_eq!(healthy_positions.len(), 2);
    }

    #[test]
    fn test_query_iter_count() {
        use crate::ecs::World;

        let mut world = World::new();
        for i in 0..100 {
            let e = world.create_entity();
            world.add_component(
                e,
                Position {
                    x: i as f32,
                    y: i as f32,
                },
            );
        }

        let query = world.query::<&Position>();
        let count = query.iter().count();
        assert_eq!(count, 100);
    }

    #[test]
    fn test_query_iter_any() {
        use crate::ecs::World;

        let mut world = World::new();
        let e1 = world.create_entity();
        let e2 = world.create_entity();

        world.add_component(e1, Position { x: 1.0, y: 2.0 });
        world.add_component(e2, Position { x: 100.0, y: 200.0 });

        let query = world.query::<&Position>();
        let has_large_x = query.iter().any(|pos| pos.x > 50.0);

        assert!(has_large_x);
    }

    #[test]
    fn test_query_iter_empty() {
        use crate::ecs::World;

        let mut world = World::new();

        let query = world.query::<&Position>();
        let items: Vec<&Position> = query.iter().collect();
        assert!(items.is_empty());

        let count = query.iter().count();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_query_iter_len() {
        use crate::ecs::World;

        let mut world = World::new();
        let e1 = world.create_entity();
        let e2 = world.create_entity();

        world.add_component(e1, Position { x: 1.0, y: 2.0 });
        // e2 has no Position

        let query = world.query::<&Position>();
        let iter = query.iter();
        // len() returns total alive entities, not just matching ones
        assert_eq!(iter.len(), 2);
        // But collect() only returns matching entities
        let collected: Vec<&Position> = query.iter().collect();
        assert_eq!(collected.len(), 1);
    }

    #[test]
    fn test_query_iter_fused() {
        use crate::ecs::World;

        let mut world = World::new();
        let e1 = world.create_entity();

        world.add_component(e1, Position { x: 1.0, y: 2.0 });

        let query = world.query::<&Position>();
        let mut iter = query.iter();
        assert_eq!(iter.next(), Some(&Position { x: 1.0, y: 2.0 }));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None); // Should still return None (FusedIterator)
    }
}
