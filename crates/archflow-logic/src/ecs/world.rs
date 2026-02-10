// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - ECS World Module (Updated with Query Execution)
//
// This module provides the World container - the main entry point for the ECS.
// World manages entities, components, and provides the Query API for systems.
//
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use core::any::TypeId;
use core::ops::Range;

use super::component::{Component, ComponentId, ComponentStorage};
use super::query::{EntityId, Query, QueryMut, QueryParameter};
use super::registry::ComponentRegistry;
use super::system::{System, SystemScheduler};

/// Maximum number of entities supported
const MAX_ENTITIES: usize = 1_000_000;

/// Placeholder for future entity metadata
#[derive(Debug, Clone)]
pub(crate) struct EntityMeta {
    /// Generation for detecting stale references
    pub(crate) generation: u32,
    /// Whether this entity slot is alive
    pub(crate) alive: bool,
    /// Which components this entity has (for fast query filtering)
    pub(crate) components: BTreeSet<TypeId>,
}

impl Default for EntityMeta {
    fn default() -> Self {
        Self {
            generation: 0,
            alive: false,
            components: BTreeSet::new(),
        }
    }
}

/// The ECS World - main container for entities and components
///
/// World manages all entities and their components, providing a unified API
/// for entity lifecycle and component management.
///
/// # Examples
///
/// ```ignore
/// let mut world = World::new();
///
/// // Create entity
/// let entity = world.create_entity();
///
/// // Add components
/// world.add_component(entity, Position { x: 0.0, y: 0.0 });
///
/// // Query components
/// world.query::<(&mut Position, &Velocity)>().each(|(pos, vel)| {
///     pos.x += vel.dx * dt;
/// });
/// ```
pub struct World {
    /// Component registry holding all component storage
    registry: ComponentRegistry,
    /// Entity metadata (generation, alive status, component types)
    entities: Vec<EntityMeta>,
    /// Free list for entity reuse
    free_list: Vec<usize>,
    /// Next entity index to allocate
    next_index: usize,
    /// System scheduler
    scheduler: SystemScheduler,
}

impl World {
    /// Creates a new empty World
    #[inline]
    pub fn new() -> Self {
        Self {
            registry: ComponentRegistry::new(),
            entities: Vec::new(),
            free_list: Vec::new(),
            next_index: 0,
            scheduler: SystemScheduler::new(),
        }
    }

    /// Creates a new entity
    #[inline]
    pub fn create_entity(&mut self) -> EntityId {
        let index = if let Some(&index) = self.free_list.last() {
            self.free_list.pop().unwrap()
        } else {
            let index = self.next_index;
            self.next_index += 1;
            self.entities.push(EntityMeta::default());
            index
        };

        let meta = &mut self.entities[index];
        meta.generation = meta.generation.wrapping_add(1);
        meta.alive = true;
        meta.components.clear();

        EntityId::new(index, meta.generation)
    }

    /// Destroys an entity
    #[inline]
    pub fn destroy_entity(&mut self, entity: EntityId) -> bool {
        let index = entity.index();

        if index >= self.entities.len() {
            return false;
        }

        let meta = &mut self.entities[index];

        if meta.generation != entity.generation() {
            return false;
        }

        if !meta.alive {
            return false;
        }

        meta.alive = false;
        meta.components.clear();
        self.free_list.push(index);

        true
    }

    /// Registers a component type with the world
    #[inline]
    pub fn register_component<T: Component>(&mut self) {
        self.registry.register::<T>();
    }

    /// Adds a component to an entity
    #[inline]
    pub fn add_component<T: Component>(&mut self, entity: EntityId, component: T) -> bool {
        if !self.is_entity_alive(entity) {
            return false;
        }

        if !self.registry.is_registered::<T>() {
            self.registry.register::<T>();
        }

        let index = entity.as_usize();
        let type_id = TypeId::of::<T>();

        // Track component in entity metadata
        if let Some(meta) = self.entities.get_mut(index) {
            meta.components.insert(type_id);
        }

        if let Some(storage) = self.registry.get_storage_mut::<T>() {
            storage.insert(index, component);
            true
        } else {
            false
        }
    }

    /// Removes a component from an entity
    #[inline]
    pub fn remove_component<T: Component>(&mut self, entity: EntityId) -> Option<T> {
        if !self.is_entity_alive(entity) {
            return None;
        }

        let index = entity.as_usize();
        let type_id = TypeId::of::<T>();

        // Remove from entity metadata
        if let Some(meta) = self.entities.get_mut(index) {
            meta.components.remove(&type_id);
        }

        if let Some(storage) = self.registry.get_storage_mut::<T>() {
            storage.remove(index)
        } else {
            None
        }
    }

    /// Gets a reference to a component on an entity
    #[inline]
    pub fn get_component<T: Component>(&self, entity: EntityId) -> Option<&T> {
        if !self.is_entity_alive(entity) {
            return None;
        }

        self.registry.get_storage::<T>()?.get(entity.as_usize())
    }

    /// Gets a mutable reference to a component on an entity
    #[inline]
    pub fn get_component_mut<T: Component>(&mut self, entity: EntityId) -> Option<&mut T> {
        if !self.is_entity_alive(entity) {
            return None;
        }

        self.registry
            .get_storage_mut::<T>()?
            .get_mut(entity.as_usize())
    }

    /// Checks if an entity is alive (not destroyed)
    #[inline]
    pub fn is_entity_alive(&self, entity: EntityId) -> bool {
        let index = entity.index();

        if index >= self.entities.len() {
            return false;
        }

        let meta = &self.entities[index];
        meta.alive && meta.generation == entity.generation()
    }

    /// Checks if an entity has a specific component
    #[inline]
    pub fn has_component<T: Component>(&self, entity: EntityId) -> bool {
        if !self.is_entity_alive(entity) {
            return false;
        }

        let type_id = TypeId::of::<T>();
        if let Some(meta) = self.entities.get(entity.as_usize()) {
            meta.components.contains(&type_id)
        } else {
            false
        }
    }

    /// Returns entities that have all specified component types
    ///
    /// This is a simplified implementation for VecStorage components.
    /// For production, this would need to handle SparseSet and Archetype storage.
    pub fn entities_with_components<T: Component>(&self) -> Vec<EntityId> {
        let type_id = TypeId::of::<T>();
        let mut entities = Vec::new();

        for (index, meta) in self.entities.iter().enumerate() {
            if meta.alive && meta.components.contains(&type_id) {
                entities.push(EntityId::new(index, meta.generation));
            }
        }

        entities
    }

    /// Creates a query over components (immutable)
    #[inline]
    pub fn query<'w, Q: QueryParameter<'w>>(&'w self) -> Query<'w, Q> {
        Query::new(self)
    }

    /// Creates a mutable query over components
    ///
    /// Use this when you need to modify components during iteration.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// world.query_mut::<&mut Position>().each(|pos| {
    ///     pos.x += 1.0;
    /// });
    /// ```
    #[inline]
    pub fn query_mut<'w, Q: QueryParameter<'w>>(&'w mut self) -> QueryMut<'w, Q> {
        QueryMut::new(self)
    }

    /// Adds a system to the world's scheduler
    #[inline]
    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.scheduler.add_system(system);
    }

    /// Adds a system by type (convenience method)
    #[inline]
    pub fn add_system_type<S: System + 'static>(&mut self, system: S) {
        self.scheduler.add_system_type(system);
    }

    /// Runs all systems in the scheduler
    #[inline]
    pub fn run_systems(&mut self, delta_time: f32) {
        // Execute startup systems first
        let mut startup_systems = core::mem::take(&mut self.scheduler.startup_systems);
        for system in &mut startup_systems {
            system.run(self, 0.0);
        }

        // Collect all systems by priority
        let mut priorities: Vec<i32> = self.scheduler.systems.keys().copied().collect();
        priorities.sort();

        // For each priority, collect systems to run
        for &priority in priorities.iter().rev() {
            // Take the system list to avoid borrow conflicts
            let system_list = self.scheduler.systems.remove(&priority);
            if let Some(mut systems) = system_list {
                for system in &mut systems {
                    system.run(self, delta_time);
                }
                // Re-insert the empty list if we want to preserve scheduler state
                if !systems.is_empty() {
                    self.scheduler.systems.insert(priority, systems);
                }
            }
        }
    }

    /// Returns the number of alive entities
    #[inline]
    pub fn entity_count(&self) -> usize {
        self.entities.iter().filter(|m| m.alive).count()
    }

    /// Returns the total number of registered component types
    #[inline]
    pub fn component_type_count(&self) -> usize {
        self.registry.len()
    }

    /// Clears all entities and components
    #[inline]
    pub fn clear(&mut self) {
        self.entities.clear();
        self.free_list.clear();
        self.next_index = 0;
        self.registry.clear();
        self.scheduler.clear();
    }

    // ==========================================================================
    // Internal methods for Query API
    // ==========================================================================

    /// Get reference to registry (for Query)
    #[inline]
    pub(crate) fn registry(&self) -> &ComponentRegistry {
        &self.registry
    }

    /// Get mutable reference to registry (for Query)
    #[inline]
    pub(crate) fn registry_mut(&mut self) -> &mut ComponentRegistry {
        &mut self.registry
    }

    /// Get entities slice (for Query iteration)
    #[inline]
    pub(crate) fn entities_slice(&self) -> &[EntityMeta] {
        &self.entities
    }

    /// Get entity metadata (for Query)
    #[inline]
    pub(crate) fn get_entity_meta(&self, index: usize) -> Option<&EntityMeta> {
        self.entities.get(index)
    }
}

impl Default for World {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Debug for World {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("World")
            .field("entity_count", &self.entity_count())
            .field("component_types", &self.component_type_count())
            .field("scheduler", &self.scheduler)
            .finish()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::component::VecStorage;

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
    fn test_world_new() {
        let world = World::new();
        assert_eq!(world.entity_count(), 0);
        assert_eq!(world.component_type_count(), 0);
    }

    #[test]
    fn test_world_create_entity() {
        let mut world = World::new();
        let entity = world.create_entity();

        assert_eq!(entity.index(), 0);
        assert_eq!(entity.generation(), 1);
        assert!(world.is_entity_alive(entity));
        assert_eq!(world.entity_count(), 1);
    }

    #[test]
    fn test_world_create_multiple_entities() {
        let mut world = World::new();

        let e1 = world.create_entity();
        let e2 = world.create_entity();
        let e3 = world.create_entity();

        assert_eq!(e1.index(), 0);
        assert_eq!(e2.index(), 1);
        assert_eq!(e3.index(), 2);
        assert_eq!(world.entity_count(), 3);
    }

    #[test]
    fn test_world_destroy_entity() {
        let mut world = World::new();
        let entity = world.create_entity();

        assert_eq!(world.entity_count(), 1);

        let destroyed = world.destroy_entity(entity);
        assert!(destroyed);
        assert_eq!(world.entity_count(), 0);
        assert!(!world.is_entity_alive(entity));
    }

    #[test]
    fn test_world_stale_entity_reference() {
        let mut world = World::new();
        let entity = world.create_entity();
        let original_generation = entity.generation();

        world.destroy_entity(entity);

        let new_entity = world.create_entity();
        assert_eq!(new_entity.index(), entity.index());
        assert_ne!(new_entity.generation(), original_generation);

        assert!(!world.is_entity_alive(entity));
    }

    #[test]
    fn test_world_add_component() {
        let mut world = World::new();
        let entity = world.create_entity();

        let added = world.add_component(entity, Position { x: 10.0, y: 20.0 });

        assert!(added);
        assert!(world.has_component::<Position>(entity));

        let pos = world.get_component::<Position>(entity);
        assert_eq!(pos, Some(&Position { x: 10.0, y: 20.0 }));
    }

    #[test]
    fn test_world_add_component_auto_registers() {
        let mut world = World::new();
        let entity = world.create_entity();

        assert_eq!(world.component_type_count(), 0);

        world.add_component(entity, Position { x: 0.0, y: 0.0 });

        assert_eq!(world.component_type_count(), 1);
    }

    #[test]
    fn test_world_add_component_to_dead_entity() {
        let mut world = World::new();
        let entity = world.create_entity();
        world.destroy_entity(entity);

        let added = world.add_component(entity, Position { x: 10.0, y: 20.0 });

        assert!(!added);
    }

    #[test]
    fn test_world_has_component() {
        let mut world = World::new();
        let entity = world.create_entity();

        assert!(!world.has_component::<Position>(entity));

        world.add_component(entity, Position { x: 0.0, y: 0.0 });

        assert!(world.has_component::<Position>(entity));
        assert!(!world.has_component::<Velocity>(entity));
    }

    #[test]
    fn test_world_remove_component() {
        let mut world = World::new();
        let entity = world.create_entity();

        world.add_component(entity, Position { x: 10.0, y: 20.0 });

        let removed = world.remove_component::<Position>(entity);
        assert_eq!(removed, Some(Position { x: 10.0, y: 20.0 }));

        let pos = world.get_component::<Position>(entity);
        assert_eq!(pos, None);
        assert!(!world.has_component::<Position>(entity));
    }

    #[test]
    fn test_world_multiple_components_per_entity() {
        let mut world = World::new();
        let entity = world.create_entity();

        world.add_component(entity, Position { x: 0.0, y: 0.0 });
        world.add_component(entity, Velocity { dx: 1.0, dy: 1.0 });
        world.add_component(
            entity,
            Health {
                current: 100,
                max: 100,
            },
        );

        assert!(world.has_component::<Position>(entity));
        assert!(world.has_component::<Velocity>(entity));
        assert!(world.has_component::<Health>(entity));
    }

    #[test]
    fn test_world_entities_with_components() {
        let mut world = World::new();

        let e1 = world.create_entity();
        let e2 = world.create_entity();
        let e3 = world.create_entity();

        world.add_component(e1, Position { x: 0.0, y: 0.0 });
        world.add_component(e2, Position { x: 1.0, y: 1.0 });
        // e3 has no Position

        let entities = world.entities_with_components::<Position>();
        assert_eq!(entities.len(), 2);
        assert!(entities.contains(&e1));
        assert!(entities.contains(&e2));
        assert!(!entities.contains(&e3));
    }

    #[test]
    fn test_world_clear() {
        let mut world = World::new();

        let e1 = world.create_entity();
        let e2 = world.create_entity();

        world.add_component(e1, Position { x: 0.0, y: 0.0 });
        world.add_component(e2, Position { x: 1.0, y: 1.0 });

        assert_eq!(world.entity_count(), 2);

        world.clear();

        assert_eq!(world.entity_count(), 0);
        assert_eq!(world.component_type_count(), 0);
    }
}
