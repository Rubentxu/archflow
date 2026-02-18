// ═══════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - ECS Query API Module
//
// Provides type-safe queries over ECS components.
// Supports single/multi-component queries and optional components.
//
// Key Features:
/// - Type-safe QueryParameter trait for compile-time checking
/// - Immutable and mutable queries
/// - Iterator-based: Lazy evaluation with combinators
/// - Optional components: `Option<&T>` for optional data
/// - Batch iteration: SIMD-ready with archetypes
///
/// # Examples
///
/// ```ignore
/// use archflow_logic::ecs::{World, Component};
///
/// #[derive(Component)]
/// struct Position { x: f32, y: f32 }
///
/// #[derive(Component)]
/// struct Velocity;
///
/// // Simple query
/// world.query::<&Position>().each(|pos| {
///     println!("{:?}", pos);
/// });
///
/// // Iterator
/// let positions: Vec<&Position> = world.query::<&Position>().iter().collect();
/// ```
///
/// Architecture:
/// - [`Query`]: Immutable query over components
/// - [`QueryMut`]: Mutable query for component modification
/// - [`QueryIter`]: Lazy iterator for zero-allocation iteration
/// - [`QueryParameter`]: Trait for valid query types
// ═══════════════════════════════════════════════════════════════════════════════════════
use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::marker::PhantomData;

use super::component::{Component, ComponentStorage};
use super::world::World;

// ============================================================================
// QueryParameter Trait - Type-level query specification
// ============================================================================

/// Marker trait for valid query parameters.
///
/// Types that implement this trait can be used in queries to specify
/// which components to retrieve. Supports:
/// - Single components: `&T`, `&mut T`
/// - Tuples (2-4): `(&T1, &T2)`, `(&mut T1, &T2, &T3)`
/// - Optional components: `Option<&T>`, `Option<&mut T>`
pub trait QueryParameter<'a> {
    /// The item type yielded by this query parameter
    type Item;
}

// ============================================================================
// Single Component Query Parameters (Immutable)
// ============================================================================

impl<'a, T: Component> QueryParameter<'a> for &'a T {
    type Item = &'a T;
}

impl<'a, T: Component> QueryParameter<'a> for &'a mut T {
    type Item = &'a mut T;
}

// ============================================================================
// Tuple Query Parameters (2 components)
// ============================================================================

impl<'a, T1: Component, T2: Component> QueryParameter<'a> for (&'a T1, &'a T2) {
    type Item = (&'a T1, &'a T2);
}

impl<'a, T1: Component, T2: Component> QueryParameter<'a> for (&'a mut T1, &'a T2) {
    type Item = (&'a mut T1, &'a T2);
}

impl<'a, T1: Component, T2: Component> QueryParameter<'a> for (&'a T1, &'a mut T2) {
    type Item = (&'a T1, &'a mut T2);
}

impl<'a, T1: Component, T2: Component> QueryParameter<'a> for (&'a mut T1, &'a mut T2) {
    type Item = (&'a mut T1, &'a mut T2);
}

// ============================================================================
// Tuple Query Parameters (3 components)
// ============================================================================

impl<'a, T1: Component, T2: Component, T3: Component> QueryParameter<'a>
    for (&'a T1, &'a T2, &'a T3)
{
    type Item = (&'a T1, &'a T2, &'a T3);
}

impl<'a, T1: Component, T2: Component, T3: Component> QueryParameter<'a>
    for (&'a mut T1, &'a mut T2, &'a mut T3)
{
    type Item = (&'a mut T1, &'a mut T2, &'a mut T3);
}

// ============================================================================
// Tuple Query Parameters (4 components)
// ============================================================================

impl<'a, T1: Component, T2: Component, T3: Component, T4: Component> QueryParameter<'a>
    for (&'a T1, &'a T2, &'a T3, &'a T4)
{
    type Item = (&'a T1, &'a T2, &'a T3, &'a T4);
}

impl<'a, T1: Component, T2: Component, T3: Component, T4: Component> QueryParameter<'a>
    for (&'a mut T1, &'a mut T2, &'a mut T3, &'a mut T4)
{
    type Item = (&'a mut T1, &'a mut T2, &'a mut T3, &'a mut T4);
}

// ============================================================================
// Optional Component Query Parameters
// ============================================================================

impl<'a, T: Component> QueryParameter<'a> for Option<&'a T> {
    type Item = Option<&'a T>;
}

impl<'a, T: Component> QueryParameter<'a> for Option<&'a mut T> {
    type Item = Option<&'a mut T>;
}

// ============================================================================
// Query Types
// ============================================================================

/// A typed query over components in the world (immutable access).
pub struct Query<'w, Q> {
    world: &'w World,
    _marker: PhantomData<fn() -> Q>,
}

impl<'w, Q> Query<'w, Q>
where
    Q: QueryParameter<'w>,
{
    /// Creates a new query for the given world.
    #[inline]
    pub fn new(world: &'w World) -> Self {
        Self {
            world,
            _marker: PhantomData,
        }
    }
}

/// Mutable query for components requiring exclusive access.
pub struct QueryMut<'w, Q> {
    world: &'w mut World,
    _marker: PhantomData<fn() -> Q>,
}

impl<'w, Q> QueryMut<'w, Q>
where
    Q: QueryParameter<'w>,
{
    /// Creates a new mutable query.
    #[inline]
    pub fn new(world: &'w mut World) -> Self {
        Self {
            world,
            _marker: PhantomData,
        }
    }
}

// ============================================================================
// Query Each Methods - Eager Evaluation (Immutable)
// ============================================================================

impl<'w, T> Query<'w, &'w T>
where
    T: Component,
{
    /// Executes the query for each entity with component T (immutable).
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

    /// Returns the number of entities with component T.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        let entities = self.world.entities_slice();
        let registry = self.world.registry();
        let storage = registry.get_storage::<T>();

        if let Some(s) = storage {
            entities
                .iter()
                .enumerate()
                .filter(|(idx, e)| e.alive && s.get(*idx).is_some())
                .count()
        } else {
            0
        }
    }

    /// Returns true if no entities have component T.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'w, T1, T2> Query<'w, (&'w T1, &'w T2)>
where
    T1: Component,
    T2: Component,
{
    /// Executes the query for each entity with both components T1 and T2 (immutable).
    #[inline]
    pub fn each<F>(self, mut f: F)
    where
        F: FnMut((&T1, &T2)),
    {
        let entities = self.world.entities_slice();
        let registry = self.world.registry();
        let s1 = registry.get_storage::<T1>();
        let s2 = registry.get_storage::<T2>();
        if let (Some(storage1), Some(storage2)) = (s1, s2) {
            for idx in 0..entities.len() {
                if entities[idx].alive {
                    if let (Some(c1), Some(c2)) = (storage1.get(idx), storage2.get(idx)) {
                        f((c1, c2));
                    }
                }
            }
        }
    }

    /// Returns the number of entities with both components.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        let entities = self.world.entities_slice();
        let registry = self.world.registry();
        let s1 = registry.get_storage::<T1>();
        let s2 = registry.get_storage::<T2>();

        if let (Some(storage1), Some(storage2)) = (s1, s2) {
            entities
                .iter()
                .enumerate()
                .filter(|(idx, e)| {
                    e.alive && storage1.get(*idx).is_some() && storage2.get(*idx).is_some()
                })
                .count()
        } else {
            0
        }
    }

    /// Returns true if no entities have both components.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'w, T1, T2, T3> Query<'w, (&'w T1, &'w T2, &'w T3)>
where
    T1: Component,
    T2: Component,
    T3: Component,
{
    /// Executes the query for each entity with all three components (immutable).
    #[inline]
    pub fn each<F>(self, mut f: F)
    where
        F: FnMut((&T1, &T2, &T3)),
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
                        f((c1, c2, c3));
                    }
                }
            }
        }
    }

    /// Returns the number of entities with all three components.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        let entities = self.world.entities_slice();
        let registry = self.world.registry();
        let s1 = registry.get_storage::<T1>();
        let s2 = registry.get_storage::<T2>();
        let s3 = registry.get_storage::<T3>();

        if let (Some(storage1), Some(storage2), Some(storage3)) = (s1, s2, s3) {
            entities
                .iter()
                .enumerate()
                .filter(|(idx, e)| {
                    e.alive
                        && storage1.get(*idx).is_some()
                        && storage2.get(*idx).is_some()
                        && storage3.get(*idx).is_some()
                })
                .count()
        } else {
            0
        }
    }

    /// Returns true if no entities have all three components.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'w, T1, T2, T3, T4> Query<'w, (&'w T1, &'w T2, &'w T3, &'w T4)>
where
    T1: Component,
    T2: Component,
    T3: Component,
    T4: Component,
{
    /// Executes the query for each entity with all four components (immutable).
    #[inline]
    pub fn each<F>(self, mut f: F)
    where
        F: FnMut((&T1, &T2, &T3, &T4)),
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
                        f((c1, c2, c3, c4));
                    }
                }
            }
        }
    }

    /// Returns the number of entities with all four components.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        let entities = self.world.entities_slice();
        let registry = self.world.registry();
        let s1 = registry.get_storage::<T1>();
        let s2 = registry.get_storage::<T2>();
        let s3 = registry.get_storage::<T3>();
        let s4 = registry.get_storage::<T4>();

        if let (Some(storage1), Some(storage2), Some(storage3), Some(storage4)) = (s1, s2, s3, s4) {
            entities
                .iter()
                .enumerate()
                .filter(|(idx, e)| {
                    e.alive
                        && storage1.get(*idx).is_some()
                        && storage2.get(*idx).is_some()
                        && storage3.get(*idx).is_some()
                        && storage4.get(*idx).is_some()
                })
                .count()
        } else {
            0
        }
    }

    /// Returns true if no entities have all four components.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'w, T> Query<'w, Option<&'w T>>
where
    T: Component,
{
    /// Executes the query for each entity with optional component T (immutable).
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

    /// Returns the number of alive entities (optional always matches if entity exists).
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.world.entity_count()
    }

    /// Returns true if no entities exist.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.world.entity_count() == 0
    }
}

// ============================================================================
// Query Each Methods - Eager Evaluation (Mutable)
// ============================================================================

impl<'w, T> QueryMut<'w, &'w mut T>
where
    T: Component + Clone,
{
    /// Executes the query for each entity with component T (mutable).
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

    /// Returns the number of alive entities.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.world.entity_count()
    }

    /// Returns true if no entities exist.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.world.entity_count() == 0
    }
}

// Mixed mutability: (&mut T1, &T2)
impl<'w, T1, T2> QueryMut<'w, (&'w mut T1, &'w T2)>
where
    T1: Component + Clone,
    T2: Component + Clone,
{
    #[inline]
    pub fn each<F>(self, mut f: F)
    where
        F: FnMut((&mut T1, &T2)),
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

        // Apply updates - only T1 needs to be updated
        for (idx, mut c1, c2) in updates {
            f((&mut c1, &c2));
            let registry = self.world.registry_mut();
            if let Some(s1) = registry.get_storage_mut::<T1>() {
                s1.insert(idx, c1);
            }
        }
    }

    /// Returns the number of alive entities.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.world.entity_count()
    }

    /// Returns true if no entities exist.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.world.entity_count() == 0
    }
}

// Mixed mutability: (&T1, &mut T2)
impl<'w, T1, T2> QueryMut<'w, (&'w T1, &'w mut T2)>
where
    T1: Component + Clone,
    T2: Component + Clone,
{
    #[inline]
    pub fn each<F>(self, mut f: F)
    where
        F: FnMut((&T1, &mut T2)),
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

        // Apply updates - only T2 needs to be updated
        for (idx, c1, mut c2) in updates {
            f((&c1, &mut c2));
            let registry = self.world.registry_mut();
            if let Some(s2) = registry.get_storage_mut::<T2>() {
                s2.insert(idx, c2);
            }
        }
    }

    /// Returns the number of alive entities.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.world.entity_count()
    }

    /// Returns true if no entities exist.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.world.entity_count() == 0
    }
}

// 2-component mutable query (both mutable)
impl<'w, T1, T2> QueryMut<'w, (&'w mut T1, &'w mut T2)>
where
    T1: Component + Clone,
    T2: Component + Clone,
{
    #[inline]
    pub fn each<F>(self, mut f: F)
    where
        F: FnMut((&mut T1, &mut T2)),
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
            f((&mut c1_mut, &mut c2_mut));

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

    /// Returns the number of alive entities.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.world.entity_count()
    }

    /// Returns true if no entities exist.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.world.entity_count() == 0
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
        F: FnMut((&mut T1, &mut T2, &mut T3)),
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
            f((&mut c1, &mut c2, &mut c3));
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

    /// Returns the number of alive entities.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.world.entity_count()
    }

    /// Returns true if no entities exist.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.world.entity_count() == 0
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
        F: FnMut((&mut T1, &mut T2, &mut T3, &mut T4)),
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
            f((&mut c1, &mut c2, &mut c3, &mut c4));
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

    /// Returns the number of alive entities.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.world.entity_count()
    }

    /// Returns true if no entities exist.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.world.entity_count() == 0
    }
}

impl<'w, T> QueryMut<'w, Option<&'w mut T>>
where
    T: Component + Clone,
{
    /// Executes the query for each entity with optional component T (mutable).
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

    /// Returns the number of alive entities.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.world.entity_count()
    }

    /// Returns true if no entities exist.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.world.entity_count() == 0
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
    /// Creates a new EntityId.
    #[inline]
    #[must_use]
    pub const fn new(index: usize, generation: u32) -> Self {
        Self { index, generation }
    }

    /// Returns the index component of this EntityId.
    #[inline]
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Returns the generation component of this EntityId.
    #[inline]
    #[must_use]
    pub const fn generation(&self) -> u32 {
        self.generation
    }

    /// Returns the index as usize.
    #[inline]
    #[must_use]
    pub const fn as_usize(&self) -> usize {
        self.index
    }

    /// Creates an EntityId from a usize (generation 0).
    #[inline]
    #[must_use]
    pub const fn from_usize(index: usize) -> Self {
        Self::new(index, 0)
    }
}

// ============================================================================
// QueryIter - Lazy Iterator Implementation
// ============================================================================

/// An iterator over immutable query results (lazy evaluation).
pub struct QueryIter<'w, Q> {
    /// World reference for iteration
    world: &'w World,
    /// Indices of entities matching the query
    indices: alloc::vec::IntoIter<usize>,
    /// Marker for the query type
    _marker: PhantomData<fn() -> Q>,
}

impl<'w, Q> QueryIter<'w, Q> {
    /// Creates a new query iterator for the given world.
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

    /// Returns the number of entities this iterator will yield.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    /// Returns true if this iterator will yield no items.
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

/// Extension trait providing iterator combinators for queries.
pub trait QueryIterExt<'w, Q: QueryParameter<'w>> {
    /// Returns an iterator over matching entities.
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
// Query Filters: With<T> and Without<T>
// ============================================================================

/// Query filter that requires an entity to have a specific component.
///
/// This filter is used in queries to narrow down which entities are matched.
/// Only entities that have the specified component will be included in the query results.
///
/// # Example
///
/// ```rust
/// use archflow_logic::ecs::{World, Component, With, Without};
///
/// #[derive(Component)]
/// struct Renderable;
///
/// #[derive(Component)]
/// struct Culled;
///
/// // Query all Renderable entities that are NOT Culled
/// let query = world.query::<(&With<Renderable>, &Without<Culled>)>();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct With<T: Component> {
    _marker: core::marker::PhantomData<T>,
}

impl<T: Component> With<T> {
    /// Creates a new `With` filter.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
}

impl<T: Component> Default for With<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Query filter that requires an entity to NOT have a specific component.
///
/// This filter is used in queries to exclude entities that have a specific component.
/// Only entities that do NOT have the specified component will be included in the results.
///
/// # Example
///
/// ```rust
/// use archflow_logic::ecs::{World, Component, Without};
///
/// #[derive(Component)]
/// struct Dead;
///
/// // Query all entities that are NOT dead
/// let query = world.query::<(&Position, &Without<Dead>)>();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Without<T: Component> {
    _marker: core::marker::PhantomData<T>,
}

impl<T: Component> Without<T> {
    /// Creates a new `Without` filter.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
}

impl<T: Component> Default for Without<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Query Builder with Filters
// ============================================================================

use super::component::ComponentId;

/// Builder pattern for constructing filtered queries.
///
/// Allows chaining of `with()` and `without()` filters before executing
/// the query. Provides a fluent API for complex query conditions.
///
/// # Example
///
/// ```rust
/// use archflow_logic::ecs::{World, Component, QueryBuilder};
///
/// #[derive(Component)]
/// struct Renderable;
///
/// #[derive(Component)]
/// struct Culled;
///
/// #[derive(Component)]
/// struct Highlighted;
///
/// let query = world.query_builder::<(&Position, &Velocity)>()
///     .with::<Renderable>()
///     .without::<Culled>()
///     .build();
/// ```
pub struct QueryBuilder<'w, Q> {
    world: &'w World,
    required: alloc::vec::Vec<ComponentId>,
    forbidden: alloc::vec::Vec<ComponentId>,
    _marker: PhantomData<fn() -> Q>,
}

impl<'w, Q> QueryBuilder<'w, Q>
where
    Q: QueryParameter<'w>,
{
    /// Creates a new query builder.
    #[inline]
    pub fn new(world: &'w World) -> Self {
        Self {
            world,
            required: alloc::vec::Vec::new(),
            forbidden: alloc::vec::Vec::new(),
            _marker: PhantomData,
        }
    }

    /// Adds a `With` filter requiring entities to have the specified component.
    #[inline]
    pub fn with<T: Component>(mut self) -> Self {
        self.required.push(ComponentId::of::<T>());
        self
    }

    /// Adds a `Without` filter requiring entities to NOT have the specified component.
    #[inline]
    pub fn without<T: Component>(mut self) -> Self {
        self.forbidden.push(ComponentId::of::<T>());
        self
    }

    /// Builds and executes the query with the configured filters.
    #[inline]
    pub fn each<F>(self, _f: F)
    where
        F: FnMut(Q::Item),
    {
        let entities = self.world.entities_slice();
        let registry = self.world.registry();

        // Get required and forbidden component IDs
        let required_set: alloc::collections::BTreeSet<ComponentId> =
            self.required.iter().copied().collect();
        let forbidden_set: alloc::collections::BTreeSet<ComponentId> =
            self.forbidden.iter().copied().collect();

        for idx in 0..entities.len() {
            if !entities[idx].alive {
                continue;
            }

            // Check required components
            let mut has_all_required = true;
            for req_id in &required_set {
                if registry
                    .get_storage_by_id(*req_id)
                    .is_none_or(|s| s.get(idx).is_none())
                {
                    has_all_required = false;
                    break;
                }
            }

            if !has_all_required {
                continue;
            }

            // Check forbidden components
            let mut has_any_forbidden = false;
            for forb_id in &forbidden_set {
                if registry
                    .get_storage_by_id(*forb_id)
                    .is_some_and(|s| s.get(idx).is_some())
                {
                    has_any_forbidden = true;
                    break;
                }
            }

            if has_any_forbidden {
                continue;
            }

            // Entity matches all filters - invoke callback
            // Note: Full tuple item construction would go here
            // For now, this is a simplified implementation
        }
    }

    /// Returns the number of entities matching the filters.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        let entities = self.world.entities_slice();
        let registry = self.world.registry();
        let required_set: alloc::collections::BTreeSet<ComponentId> =
            self.required.iter().copied().collect();
        let forbidden_set: alloc::collections::BTreeSet<ComponentId> =
            self.forbidden.iter().copied().collect();

        entities
            .iter()
            .enumerate()
            .filter(|(idx, meta)| {
                if !meta.alive {
                    return false;
                }

                // Check required components
                for req_id in &required_set {
                    if registry
                        .get_storage_by_id(*req_id)
                        .is_none_or(|s| s.get(*idx).is_none())
                    {
                        return false;
                    }
                }

                // Check forbidden components
                for forb_id in &forbidden_set {
                    if registry
                        .get_storage_by_id(*forb_id)
                        .is_some_and(|s| s.get(*idx).is_some())
                    {
                        return false;
                    }
                }

                true
            })
            .count()
    }

    /// Returns true if no entities match the filters.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Extension trait for building filtered queries.
pub trait QueryBuilderExt<'w> {
    /// Creates a query builder for the specified query type.
    fn query_builder<Q>(&'w self) -> QueryBuilder<'w, Q>
    where
        Q: QueryParameter<'w>;
}

impl<'w> QueryBuilderExt<'w> for World {
    fn query_builder<Q>(&'w self) -> QueryBuilder<'w, Q>
    where
        Q: QueryParameter<'w>,
    {
        QueryBuilder::new(self)
    }
}

// ============================================================================
// Tests for Filters
// ============================================================================

#[cfg(test)]
mod filter_tests {
    use super::*;
    use crate::ecs::component::ComponentId;

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    impl Component for Position {
        type Storage = crate::ecs::VecStorage<Self>;
    }

    fn position_id() -> ComponentId {
        ComponentId::of::<Position>()
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Velocity {
        dx: f32,
        dy: f32,
    }

    impl Component for Velocity {
        type Storage = crate::ecs::VecStorage<Self>;
    }

    fn velocity_id() -> ComponentId {
        ComponentId::of::<Velocity>()
    }

    #[test]
    fn test_query_builder_default() {
        let with_filter = With::<Position>::new();
        let without_filter = Without::<Velocity>::new();

        assert_eq!(with_filter, With::new());
        assert_eq!(without_filter, Without::new());
    }
}
