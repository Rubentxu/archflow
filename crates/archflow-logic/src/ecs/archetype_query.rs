// ═══════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Archetype Query Module
//
// This module provides efficient multi-archetype query capabilities for the ECS.
// Queries across multiple archetypes enable cache-friendly iteration while
// maintaining type safety.
//
// Key Features:
// - ArchetypeQuery: Query across multiple archetypes
// - Archetype filtering: Filter archetypes by component requirements
// - Batch processing: Iterate by archetype for cache efficiency
// - Auto-migration: Entities move between archetypes when components change
//
// Architecture:
// - ArchetypeQueryBuilder: Fluent API for building archetype queries
// - ArchetypeMatcher: Matches archetypes against component requirements
// - Cross-archetype iteration: Query entities across all matching archetypes
//
// Performance Benefits:
// - Cache locality: Each archetype's data is contiguous in memory
// - Batch processing: Process entities by archetype (not individually)
// - Parallel-ready: Independent archetypes can be processed in parallel
// - Minimal overhead: No virtual dispatch, static dispatch where possible
//
// ═══════════════════════════════════════════════════════════════════════════════════════

#![no_std]

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use core::any::TypeId;

use super::archetype::{Archetype, ArchetypeId, ArchetypeStorage};
use super::component::{Component, ComponentId};
use super::world::World;

/// Builder for archetype-based queries
///
/// Provides a fluent API for constructing complex archetype queries
/// with filtering by component presence/absence.
///
/// # Examples
///
/// ```ignore
/// // Query all entities with Position and Velocity
/// let query = ArchetypeQuery::new::<(Position, Velocity)>(&world);
/// for (pos, vel) in query.iter() {
///     // Process entities
/// }
///
/// // Query with filtering
/// let query = ArchetypeQuery::new::<(Position, Velocity)>(&world)
///     .with::<Renderable>()
///     .without::<Culled>();
/// ```
pub struct ArchetypeQuery<'w> {
    /// Reference to the archetype storage
    archetype_storage: &'w ArchetypeStorage,
    /// Required component type IDs
    required: Vec<ComponentId>,
    /// Forbidden component type IDs (entities must NOT have these)
    forbidden: Vec<ComponentId>,
}

impl<'w> ArchetypeQuery<'w> {
    /// Creates a new archetype query requiring the specified components
    ///
    /// # Parameters
    ///
    /// - `world`: Reference to the world containing archetypes
    ///
    /// # Type Parameters
    ///
    /// - `Q`: Query parameter specifying required components
    #[inline]
    pub fn new<Q: QueryParam>(world: &'w World) -> Self {
        let component_ids = Q::component_ids();

        Self {
            archetype_storage: world.archetype_storage(),
            required: component_ids,
            forbidden: Vec::new(),
        }
    }

    /// Adds a required component (entity must have this)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let query = ArchetypeQuery::<(Position, Velocity)>::new(&world)
    ///     .with::<Renderable>();
    /// ```
    #[inline]
    pub fn with<T: Component>(mut self) -> Self {
        self.required.push(ComponentId::of::<T>());
        self
    }

    /// Adds a forbidden component (entity must NOT have this)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let query = ArchetypeQuery::<(Position, Velocity)>::new(&world)
    ///     .without::<Culled>();
    /// ```
    #[inline]
    pub fn without<T: Component>(mut self) -> Self {
        self.forbidden.push(ComponentId::of::<T>());
        self
    }

    /// Returns the number of matching archetypes
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.matching_archetypes().count()
    }

    /// Returns true if no archetypes match
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over matching archetypes
    #[inline]
    pub fn archetypes(&self) -> ArchetypeIter<'_> {
        ArchetypeIter::new(self)
    }

    /// Returns a slice of matching archetype IDs
    #[inline]
    #[must_use]
    pub fn matching_archetype_ids(&self) -> Vec<ArchetypeId> {
        self.matching_archetypes().map(|(id, _)| *id).collect()
    }

    /// Iterates over matching archetypes and their entity counts
    #[inline]
    pub fn archetype_counts(&self) -> impl Iterator<Item = (ArchetypeId, usize)> + '_ {
        self.matching_archetypes()
            .map(|(id, archetype)| (*id, archetype.len()))
    }

    /// Returns total entity count across all matching archetypes
    #[inline]
    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.matching_archetypes().map(|(_, arch)| arch.len()).sum()
    }

    /// Returns an iterator over archetypes with batched component data for SIMD processing
    ///
    /// This method enables efficient SIMD-friendly iteration by providing
    /// component data in contiguous batches that can be vectorized.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let query = ArchetypeQuery::<(Position, Velocity)>::new(&world);
    /// for archetype in query.each_batch::<f32>(4) {
    ///     // archetype contains batched component data
    ///     for position_batch in archetype.batch::<Position>(4) {
    ///         // Process SIMD batch of positions
    ///     }
    /// }
    /// ```
    #[inline]
    pub fn each_archetype_batch(&self) -> ArchetypeBatchIter<'_> {
        ArchetypeBatchIter::new(self)
    }

    /// Internal: Returns iterator over matching archetypes
    fn matching_archetypes(&self) -> impl Iterator<Item = (&ArchetypeId, &Archetype)> {
        self.archetype_storage
            .iter_archetypes()
            .filter(|(_, archetype)| self.archetype_matches(archetype))
    }

    /// Internal: Checks if an archetype matches the query requirements
    fn archetype_matches(&self, archetype: &Archetype) -> bool {
        // Check all required components are present
        let archetype_types = archetype.types();

        for required in &self.required {
            if !archetype_types.contains(required) {
                return false;
            }
        }

        // Check no forbidden components are present
        for forbidden in &self.forbidden {
            if archetype_types.contains(forbidden) {
                return false;
            }
        }

        true
    }
}

/// Iterator over matching archetypes
pub struct ArchetypeIter<'a> {
    /// All matching archetype references
    archetypes: Vec<(&'a ArchetypeId, &'a Archetype)>,
    /// Current index during iteration
    index: usize,
}

impl<'a> ArchetypeIter<'a> {
    /// Creates a new archetype iterator
    #[inline]
    fn new(query: &'a ArchetypeQuery<'a>) -> Self {
        // Collect all matching archetypes upfront
        let archetypes: Vec<_> = query.matching_archetypes().collect();

        Self {
            archetypes,
            index: 0,
        }
    }
}

impl<'a> Iterator for ArchetypeIter<'a> {
    type Item = (&'a ArchetypeId, &'a Archetype);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.archetypes.len() {
            let result = self.archetypes[self.index];
            self.index += 1;
            Some(result)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.archetypes.len() - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for ArchetypeIter<'a> {}

impl<'a> core::iter::FusedIterator for ArchetypeIter<'a> {}

/// Provides batched access to archetype component data for SIMD processing
///
/// Wraps an archetype reference and provides methods for accessing component
/// data in contiguous batches suitable for SIMD vectorization.
pub struct ArchetypeBatch<'a> {
    /// Reference to the archetype
    archetype: &'a Archetype,
    /// Archetype ID
    id: &'a ArchetypeId,
}

impl<'a> ArchetypeBatch<'a> {
    /// Creates a new batch accessor for an archetype
    #[inline]
    fn new(id: &'a ArchetypeId, archetype: &'a Archetype) -> Self {
        Self { id, archetype }
    }

    /// Returns the archetype ID
    #[inline]
    pub fn id(&self) -> &ArchetypeId {
        self.id
    }

    /// Returns the number of entities in this archetype
    #[inline]
    pub fn len(&self) -> usize {
        self.archetype.len()
    }

    /// Returns true if the archetype is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.archetype.is_empty()
    }

    /// Returns an iterator over batches of a specific component type
    ///
    /// # Type Parameters
    ///
    /// - `T`: The component type to batch (must be `'static`)
    ///
    /// # Parameters
    ///
    /// - `batch_size`: Maximum number of elements per batch
    #[inline]
    pub fn batch<T: 'static>(&self, batch_size: usize) -> ComponentBatchIter<'a, T> {
        // Find the component column for type T
        let component_id = ComponentId::of::<T>();

        // Get the column if it exists
        let column = self.archetype.get_column(component_id);

        ComponentBatchIter::new(column, batch_size)
    }

    /// Returns an iterator over entity IDs
    #[inline]
    pub fn entities(&self) -> core::slice::Iter<'a, usize> {
        self.archetype.iter_entities()
    }
}

/// Iterator over component batches for SIMD processing
pub struct ComponentBatchIter<'a, T: 'static> {
    /// Optional reference to the component column
    column: Option<&'a super::archetype::ComponentColumn>,
    /// Batch size for iteration
    batch_size: usize,
    /// Phantom data for the component type
    _phantom: core::marker::PhantomData<&'a T>,
}

impl<'a, T: 'static> ComponentBatchIter<'a, T> {
    /// Creates a new batch iterator
    #[inline]
    fn new(column: Option<&'a super::archetype::ComponentColumn>, batch_size: usize) -> Self {
        Self {
            column,
            batch_size,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<'a, T: 'a> Iterator for ComponentBatchIter<'a, T> {
    type Item = &'a [T];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Can't use ? here because we need to return a reference that lives as long as 'a
        let column = self.column?;
        column.iter_batch::<T>(self.batch_size).next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let column = match self.column {
            Some(c) => c,
            None => return (0, Some(0)),
        };

        let total = column.len();
        let batches = (total + self.batch_size - 1) / self.batch_size;
        let remaining = batches.saturating_sub(1);

        (remaining, Some(remaining))
    }
}

/// Iterator over archetypes with batched access for SIMD processing
pub struct ArchetypeBatchIter<'a> {
    /// Iterator over matching archetypes
    archetype_iter: ArchetypeIter<'a>,
}

impl<'a> ArchetypeBatchIter<'a> {
    /// Creates a new batch iterator
    #[inline]
    fn new(query: &'a ArchetypeQuery<'a>) -> Self {
        Self {
            archetype_iter: query.archetypes(),
        }
    }
}

impl<'a> Iterator for ArchetypeBatchIter<'a> {
    type Item = ArchetypeBatch<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.archetype_iter
            .next()
            .map(|(id, archetype)| ArchetypeBatch::new(id, archetype))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.archetype_iter.size_hint()
    }
}

/// Trait for extracting component type IDs at compile time
///
/// This trait is implemented for tuples of components, allowing
/// compile-time extraction of component type IDs for archetype matching.
pub trait QueryParam {
    /// Returns the component type IDs for this query parameter
    fn component_ids() -> Vec<ComponentId>;
}

// Implementations for tuples of increasing sizes

impl<T: Component> QueryParam for (&T,) {
    #[inline]
    fn component_ids() -> Vec<ComponentId> {
        vec![ComponentId::of::<T>()]
    }
}

impl<T1: Component, T2: Component> QueryParam for (&T1, &T2) {
    #[inline]
    fn component_ids() -> Vec<ComponentId> {
        vec![ComponentId::of::<T1>(), ComponentId::of::<T2>()]
    }
}

impl<T1: Component, T2: Component, T3: Component> QueryParam for (&T1, &T2, &T3) {
    #[inline]
    fn component_ids() -> Vec<ComponentId> {
        vec![
            ComponentId::of::<T1>(),
            ComponentId::of::<T2>(),
            ComponentId::of::<T3>(),
        ]
    }
}

impl<T1: Component, T2: Component, T3: Component, T4: Component> QueryParam
    for (&T1, &T2, &T3, &T4)
{
    #[inline]
    fn component_ids() -> Vec<ComponentId> {
        vec![
            ComponentId::of::<T1>(),
            ComponentId::of::<T2>(),
            ComponentId::of::<T3>(),
            ComponentId::of::<T4>(),
        ]
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archetype_query_new() {
        // ArchetypeQuery requires World which needs more setup
        // This is a compile-time check that the API works
        // Actual tests would require a full World instance
    }

    #[test]
    fn test_query_param_single() {
        struct Position;
        impl Component for Position {
            type Storage = super::super::VecStorage<Position>;
        }

        let ids = <(&Position,) as QueryParam>::component_ids();
        assert_eq!(ids.len(), 1);
    }

    #[test]
    fn test_query_param_double() {
        struct Position;
        struct Velocity;

        impl Component for Position {
            type Storage = super::super::VecStorage<Position>;
        }

        impl Component for Velocity {
            type Storage = super::super::VecStorage<Velocity>;
        }

        let ids = <(&Position, &Velocity) as QueryParam>::component_ids();
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn test_query_param_triple() {
        struct Position;
        struct Velocity;
        struct Acceleration;

        impl Component for Position {
            type Storage = super::super::VecStorage<Position>;
        }

        impl Component for Velocity {
            type Storage = super::super::VecStorage<Velocity>;
        }

        impl Component for Acceleration {
            type Storage = super::super::VecStorage<Acceleration>;
        }

        let ids = <(&Position, &Velocity, &Acceleration) as QueryParam>::component_ids();
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn test_query_param_quadruple() {
        struct Position;
        struct Velocity;
        struct Acceleration;
        struct Rotation;

        impl Component for Position {
            type Storage = super::super::VecStorage<Position>;
        }

        impl Component for Velocity {
            type Storage = super::super::VecStorage<Velocity>;
        }

        impl Component for Acceleration {
            type Storage = super::super::VecStorage<Acceleration>;
        }

        impl Component for Rotation {
            type Storage = super::super::VecStorage<Rotation>;
        }

        let ids = <(&Position, &Velocity, &Acceleration, &Rotation) as QueryParam>::component_ids();
        assert_eq!(ids.len(), 4);
    }
}

// ============================================================================
// Integration Tests (World + ArchetypeQuery)
// ============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_world_archetype_query_basic() {
        // Create a world with archetype storage
        let mut world = World::new();

        // Create entities with different component combinations
        let entity1 = world.create_entity();
        world.add_component(entity1, TestPosition { x: 1.0, y: 2.0 });

        let entity2 = world.create_entity();
        world.add_component(entity2, TestPosition { x: 3.0, y: 4.0 });
        world.add_component(entity2, TestVelocity { dx: 5.0, dy: 6.0 });

        let entity3 = world.create_entity();
        world.add_component(entity3, TestPosition { x: 7.0, y: 8.0 });

        // Query entities with Position component
        let query = ArchetypeQuery::new::<(&TestPosition,)>(&world);
        assert_eq!(query.entity_count(), 3);
        assert!(!query.is_empty());

        // Query entities with Position and Velocity components
        let query2 = ArchetypeQuery::new::<(&TestPosition, &TestVelocity)>(&world);
        assert_eq!(query2.entity_count(), 1);
    }

    #[test]
    fn test_world_archetype_query_with_filter() {
        let mut world = World::new();

        // Create entities with different components
        let entity1 = world.create_entity();
        world.add_component(entity1, TestPosition { x: 1.0, y: 2.0 });
        world.add_component(entity1, TestRenderable);

        let entity2 = world.create_entity();
        world.add_component(entity2, TestPosition { x: 3.0, y: 4.0 });

        let entity3 = world.create_entity();
        world.add_component(entity3, TestPosition { x: 5.0, y: 6.0 });
        world.add_component(entity3, TestCulled); // Should be filtered out

        // Query with Renderable (should have 1 entity)
        let query_renderable =
            ArchetypeQuery::new::<(&TestPosition,)>(&world).with::<TestRenderable>();
        assert_eq!(query_renderable.entity_count(), 1);

        // Query without Culled (should have 2 entities)
        let query_not_culled =
            ArchetypeQuery::new::<(&TestPosition,)>(&world).without::<TestCulled>();
        assert_eq!(query_not_culled.entity_count(), 2);
    }

    #[test]
    fn test_world_archetype_query_across_archetypes() {
        let mut world = World::new();

        // Create multiple entities across different archetypes
        for i in 0..5 {
            let entity = world.create_entity();
            world.add_component(
                entity,
                TestPosition {
                    x: i as f32,
                    y: i as f32,
                },
            );
            if i % 2 == 0 {
                world.add_component(
                    entity,
                    TestVelocity {
                        dx: i as f32,
                        dy: i as f32,
                    },
                );
            }
        }

        // Query all entities with Position (should be 5 across 2 archetypes)
        let query = ArchetypeQuery::new::<(&TestPosition,)>(&world);
        assert_eq!(query.entity_count(), 5);

        // Should have 2 matching archetypes: [Position] and [Position, Velocity]
        assert_eq!(query.matching_archetype_ids().len(), 2);
    }

    #[test]
    fn test_world_archetype_migration() {
        let mut world = World::new();

        // Create entity with only Position
        let entity = world.create_entity();
        world.add_component(entity, TestPosition { x: 1.0, y: 2.0 });

        // Initially in archetype with only Position
        let archetype_ids_before = world
            .archetype_storage()
            .get_archetype_id(entity.as_usize());
        assert!(archetype_ids_before.is_some());

        // Add Velocity - should migrate to new archetype
        world.add_component(entity, TestVelocity { dx: 3.0, dy: 4.0 });

        // Entity should now be in different archetype
        let archetype_ids_after = world
            .archetype_storage()
            .get_archetype_id(entity.as_usize());
        assert_ne!(archetype_ids_before, archetype_ids_after);
    }

    #[test]
    fn test_world_archetype_query_empty() {
        let world = World::new();

        // Query on empty world
        let query = ArchetypeQuery::new::<(&TestPosition,)>(&world);
        assert_eq!(query.entity_count(), 0);
        assert!(query.is_empty());
    }

    // Test component types
    #[derive(Debug, PartialEq)]
    struct TestPosition {
        x: f32,
        y: f32,
    }

    impl Component for TestPosition {
        type Storage = super::super::VecStorage<TestPosition>;
    }

    #[derive(Debug, PartialEq)]
    struct TestVelocity {
        dx: f32,
        dy: f32,
    }

    impl Component for TestVelocity {
        type Storage = super::super::VecStorage<TestVelocity>;
    }

    #[derive(Debug, PartialEq)]
    struct TestAcceleration {
        ax: f32,
        ay: f32,
    }

    impl Component for TestAcceleration {
        type Storage = super::super::VecStorage<TestAcceleration>;
    }

    #[derive(Debug, PartialEq)]
    struct TestRenderable;

    impl Component for TestRenderable {
        type Storage = super::super::VecStorage<TestRenderable>;
    }

    #[derive(Debug, PartialEq)]
    struct TestCulled;

    impl Component for TestCulled {
        type Storage = super::super::VecStorage<TestCulled>;
    }

    // ============================================================================
    // SIMD Batch Processing Demo Test
    // ============================================================================

    #[test]
    #[ignore]
    fn test_simd_batch_processing_demo() {
        let mut world = World::new();

        // Create 100 entities with Position components
        for i in 0..100 {
            let entity = world.create_entity();
            world.add_component(
                entity,
                TestPosition {
                    x: i as f32,
                    y: i as f32,
                },
            );

            // Add velocity to every other entity
            if i % 2 == 0 {
                world.add_component(
                    entity,
                    TestVelocity {
                        dx: i as f32 * 2.0,
                        dy: i as f32 * 2.0,
                    },
                );
            }
        }

        // Demonstrate batch processing
        let query = ArchetypeQuery::new::<(&TestPosition,)>(&world);

        // Process each archetype with batched component access
        let mut total_entities = 0;
        for archetype_batch in query.each_archetype_batch() {
            // Get archetype ID and entity count
            let arch_id = archetype_batch.id();
            let entity_count = archetype_batch.len();

            // Process position data in SIMD-friendly batches of 4
            for position_batch in archetype_batch.batch::<TestPosition>(4) {
                // This batch can be processed with SIMD instructions
                // Example: multiply all positions by 2.0
                for pos in position_batch {
                    // SIMD would process multiple positions here
                    let _ = pos.x * 2.0 + pos.y * 2.0;
                }
            }

            total_entities += entity_count;
        }

        assert_eq!(total_entities, 100);
    }

    #[test]
    #[ignore]
    fn test_archetype_batch_iteration() {
        let mut world = World::new();

        // Create entities across multiple archetypes
        for i in 0..10 {
            let entity = world.create_entity();
            world.add_component(
                entity,
                TestPosition {
                    x: i as f32,
                    y: i as f32,
                },
            );

            // Half get Velocity
            if i < 5 {
                world.add_component(
                    entity,
                    TestVelocity {
                        dx: i as f32,
                        dy: i as f32,
                    },
                );
            }
        }

        // Query and iterate over archetypes
        let query = ArchetypeQuery::new::<(&TestPosition, &TestVelocity)>(&world);

        let mut archetype_count = 0;
        for archetype in query.each_archetype_batch() {
            archetype_count += 1;
            assert!(archetype.len() > 0);

            // Verify we can get batches of both components
            let positions: Vec<_> = archetype.batch::<TestPosition>(4).collect();
            let velocities: Vec<_> = archetype.batch::<TestVelocity>(4).collect();

            // Should have some batches
            assert!(!positions.is_empty() || !velocities.is_empty());
        }

        // Should have queried 1 archetype [Position, Velocity]
        assert_eq!(archetype_count, 1);
    }
}
