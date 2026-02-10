// ═══════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - ECS Query API Module (Simplificada)
//
// Versión simplificada que permite queries type-safe sobre componentes.
// ═══════════════════════════════════════════════════════════════════════════════════════

use alloc::vec::Vec;
use core::marker::PhantomData;

use super::component::{Component, ComponentStorage};
use super::world::World;

// ============================================================================
// QueryParameter Trait - Type-level query specification
// ============================================================================

/// Marker trait for valid query parameters
pub trait QueryParameter<'a> {
    type Item;
}

/// Immutable reference: &T
impl<'a, T: Component> QueryParameter<'a> for &'a T {
    type Item = &'a T;
}

/// Mutable reference: &mut T
impl<'a, T: Component> QueryParameter<'a> for &'a mut T {
    type Item = &'a mut T;
}

/// Tuple of two immutable references: (&T1, &T2)
impl<'a, T1: Component, T2: Component> QueryParameter<'a> for (&'a T1, &'a T2) {
    type Item = (&'a T1, &'a T2);
}

/// Tuple of three immutable references: (&T1, &T2, &T3)
impl<'a, T1: Component, T2: Component, T3: Component> QueryParameter<'a>
    for (&'a T1, &'a T2, &'a T3)
{
    type Item = (&'a T1, &'a T2, &'a T3);
}

// ============================================================================
// Query Types
// ============================================================================

/// A typed query over components in the world (immutable)
pub struct Query<'w, Q: QueryParameter<'w>> {
    world: &'w World,
    entities: Vec<usize>,
    _marker: PhantomData<Q>,
}

impl<'w, Q: QueryParameter<'w>> Query<'w, Q> {
    /// Creates a new query for the given world
    #[inline]
    pub fn new(world: &'w World) -> Self {
        let entities = world
            .entities_slice()
            .iter()
            .enumerate()
            .filter_map(|(index, meta)| if meta.alive { Some(index) } else { None })
            .collect();
        Self {
            world,
            entities,
            _marker: PhantomData,
        }
    }
}

/// Mutable query for components requiring exclusive access
pub struct QueryMut<'w, Q: QueryParameter<'w>> {
    world: &'w mut World,
    entities: Vec<usize>,
    _marker: PhantomData<Q>,
}

impl<'w, Q: QueryParameter<'w>> QueryMut<'w, Q> {
    /// Creates a new mutable query
    #[inline]
    pub fn new(world: &'w mut World) -> Self {
        let entities = world
            .entities_slice()
            .iter()
            .enumerate()
            .filter_map(|(index, meta)| if meta.alive { Some(index) } else { None })
            .collect();
        Self {
            world,
            entities,
            _marker: PhantomData,
        }
    }
}

// ============================================================================
// Query Each - Single Component (Immutable)
// ============================================================================

impl<'w, T: Component> Query<'w, &'w T> {
    /// Executes the query for each entity with component T (immutable)
    #[inline]
    pub fn each<F>(self, mut f: F)
    where
        F: FnMut(&'w T),
    {
        let registry = self.world.registry();
        if let Some(storage) = registry.get_storage::<T>() {
            for &index in &self.entities {
                if let Some(component) = storage.get(index) {
                    f(component);
                }
            }
        }
    }
}

// ============================================================================
// Query Each - Single Component (Mutable)
// ============================================================================

impl<'w, T: Component> QueryMut<'w, &'w mut T> {
    /// Executes the query for each entity with component T (mutable)
    #[inline]
    pub fn each<F>(mut self, mut f: F)
    where
        F: FnMut(&mut T),
    {
        let registry = self.world.registry_mut();
        if let Some(storage) = registry.get_storage_mut::<T>() {
            for &index in &self.entities {
                if let Some(component) = storage.get_mut(index) {
                    f(component);
                }
            }
        }
    }
}

// ============================================================================
// Query Each - Two Components (Immutable)
// ============================================================================

impl<'w, T1: Component, T2: Component> Query<'w, (&'w T1, &'w T2)> {
    /// Executes the query for each entity with both components T1 and T2
    #[inline]
    pub fn each<F>(self, mut f: F)
    where
        F: FnMut((&'w T1, &'w T2)),
    {
        let registry = self.world.registry();
        let s1 = registry.get_storage::<T1>();
        let s2 = registry.get_storage::<T2>();
        if let (Some(storage1), Some(storage2)) = (s1, s2) {
            for &index in &self.entities {
                if let (Some(c1), Some(c2)) = (storage1.get(index), storage2.get(index)) {
                    f((c1, c2));
                }
            }
        }
    }
}

// ============================================================================
// Query Each - Three Components (Immutable)
// ============================================================================

impl<'w, T1: Component, T2: Component, T3: Component> Query<'w, (&'w T1, &'w T2, &'w T3)> {
    /// Executes the query for each entity with all three components
    #[inline]
    pub fn each<F>(self, mut f: F)
    where
        F: FnMut((&'w T1, &'w T2, &'w T3)),
    {
        let registry = self.world.registry();
        let s1 = registry.get_storage::<T1>();
        let s2 = registry.get_storage::<T2>();
        let s3 = registry.get_storage::<T3>();
        if let (Some(storage1), Some(storage2), Some(storage3)) = (s1, s2, s3) {
            for &index in &self.entities {
                if let (Some(c1), Some(c2), Some(c3)) = (
                    storage1.get(index),
                    storage2.get(index),
                    storage3.get(index),
                ) {
                    f((c1, c2, c3));
                }
            }
        }
    }
}

// ============================================================================
// EntityId
// ============================================================================

/// Unique identifier for an entity
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId {
    index: usize,
    generation: u32,
}

impl EntityId {
    /// Creates a new EntityId
    #[inline]
    pub const fn new(index: usize, generation: u32) -> Self {
        Self { index, generation }
    }

    /// Returns the entity index
    #[inline]
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Returns the generation
    #[inline]
    pub const fn generation(&self) -> u32 {
        self.generation
    }

    /// Returns the index as usize
    #[inline]
    pub const fn as_usize(&self) -> usize {
        self.index
    }

    /// Creates an EntityId from a usize
    #[inline]
    pub const fn from_usize(index: usize) -> Self {
        Self::new(index, 0)
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

    #[test]
    fn test_query_single_component() {
        use crate::ecs::World;

        let mut world = World::new();

        let e1 = world.create_entity();
        let e2 = world.create_entity();
        let e3 = world.create_entity();

        world.add_component(e1, Position { x: 1.0, y: 2.0 });
        world.add_component(e2, Position { x: 3.0, y: 4.0 });
        world.add_component(e3, Velocity { dx: 1.0, dy: 1.0 });

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
    fn test_query_tuple_components() {
        use crate::ecs::World;

        let mut world = World::new();

        let e1 = world.create_entity();
        let e2 = world.create_entity();

        world.add_component(e1, Position { x: 0.0, y: 0.0 });
        world.add_component(e1, Velocity { dx: 1.0, dy: 2.0 });
        world.add_component(e2, Position { x: 10.0, y: 20.0 });
        world.add_component(e2, Velocity { dx: 3.0, dy: 4.0 });

        let mut count = 0;

        world.query::<(&Position, &Velocity)>().each(|(pos, vel)| {
            count += 1;
            assert!(pos.x >= 0.0);
        });

        assert_eq!(count, 2);
    }

    #[test]
    fn test_query_mut_single_component() {
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
    fn test_query_triple_components() {
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
            .each(|(pos, vel, health)| {
                count += 1;
                total_health += health.current;
                assert!(pos.x >= 0.0);
            });

        assert_eq!(count, 2);
        assert_eq!(total_health, 150);
    }
}
