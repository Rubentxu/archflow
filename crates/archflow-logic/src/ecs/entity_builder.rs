// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - EntityBuilder Module
//
// This module provides a fluent API for entity creation in the ECS.
// It follows the builder pattern to allow chained configuration of entities.
//
// # Example
//
// ```
// use archflow_logic::ecs::entity_builder::WorldSpawnExt;
//
// let entity = world.spawn()
//     .insert(Position { x: 10.0, y: 20.0 })
//     .insert(Velocity { dx: 1.0, dy: 2.0 })
//     .name("Player")
//     .build();
// ```
//
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(unused)]

use alloc::string::String;
use alloc::vec::Vec;

use crate::ecs::behavior_block::{BehaviorBlock, BehaviorBlockBuilder, BehaviorBlockComponent};
use crate::ecs::component::Component;
use crate::ecs::components::NamedComponent;
use crate::ecs::query::EntityId;
use crate::ecs::world::World;

/// Extension trait to add `spawn()` method to World.
///
/// This trait provides the fluent API for entity creation:
/// ```
/// use archflow_logic::ecs::entity_builder::WorldSpawnExt;
///
/// let entity = world.spawn()
///     .insert(Position { x: 0.0, y: 0.0 })
///     .build();
/// ```
pub trait WorldSpawnExt {
    /// Spawns a new entity with a fluent builder.
    fn spawn(&mut self) -> EntityBuilder;
}

impl WorldSpawnExt for World {
    /// Creates a new EntityBuilder for spawning an entity.
    #[inline]
    fn spawn(&mut self) -> EntityBuilder {
        EntityBuilder::new(self)
    }
}

/// Trait for types that can build entities fluently.
///
/// This trait defines the interface for building entities with a fluent API.
/// It is implemented by `World` to provide `world.spawn()`.
pub trait EntityBuildable<'w> {
    /// The output type of the build process (usually EntityId)
    type Output;

    /// Inserts a component into the entity being built.
    fn insert<C: Component>(self, component: C) -> Self;

    /// Sets the name of the entity for debugging purposes.
    fn name(self, name: impl Into<String>) -> Self;

    /// Finalizes the entity creation and returns the result.
    fn build(self) -> Self::Output;
}

/// A builder for creating entities with a fluent API.
///
/// This struct is returned by `World::spawn()` and allows
/// chaining component insertions and other configurations.
///
/// # Example
///
/// ```
/// use archflow_logic::ecs::entity_builder::WorldSpawnExt;
///
/// let entity = world.spawn()
///     .insert(Position { x: 10.0, y: 20.0 })
///     .insert(Velocity { dx: 1.0, dy: 2.0 })
///     .name("Player")
///     .build();
/// ```
pub struct EntityBuilder<'w> {
    /// Reference to the world where the entity will be created
    pub(crate) world: &'w mut World,
    /// The entity being built
    pub(crate) entity: EntityId,
    /// Optional name for the entity
    pub(crate) name: Option<String>,
    /// Behavior blocks attached to this entity
    pub(crate) behaviors: Vec<BehaviorBlock>,
}

impl<'w> EntityBuilder<'w> {
    /// Creates a new EntityBuilder.
    ///
    /// This is typically called via `world.spawn()`.
    #[inline]
    pub fn new(world: &'w mut World) -> Self {
        let entity = world.create_entity();
        Self {
            world,
            entity,
            name: None,
            behaviors: Vec::new(),
        }
    }

    /// Inserts a component into the entity being built.
    ///
    /// Components are added immediately to the entity.
    ///
    /// # Returns
    ///
    /// Returns `Self` to allow method chaining.
    ///
    /// # Example
    ///
    /// ```
    /// use archflow_logic::ecs::entity_builder::WorldSpawnExt;
    ///
    /// let entity = world.spawn()
    ///     .insert(Position { x: 10.0, y: 20.0 })
    ///     .insert(Velocity { dx: 1.0, dy: 2.0 })
    ///     .build();
    /// ```
    #[inline]
    pub fn insert<C: Component>(mut self, component: C) -> Self {
        let result = self.world.add_component(self.entity, component);
        if !result {
            // Component insertion failed, but we continue
            // The entity was still created
        }
        self
    }

    /// Sets the name of the entity for debugging purposes.
    ///
    /// If multiple `name()` calls are made, the last one wins.
    ///
    /// # Returns
    ///
    /// Returns `Self` to allow method chaining.
    ///
    /// # Example
    ///
    /// ```
    /// use archflow_logic::ecs::entity_builder::WorldSpawnExt;
    ///
    /// let entity = world.spawn()
    ///     .name("Player")
    ///     .build();
    /// ```
    #[inline]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Starts a new behavior block for this entity.
    ///
    /// This method begins a new behavior block that can contain sensors,
    /// controllers, and actuators. Use `.end()` on the BehaviorBlockBuilder
    /// to return to the EntityBuilder.
    ///
    /// # Returns
    ///
    /// Returns a `BehaviorBlockBuilder` to configure the behavior.
    ///
    /// # Example
    ///
    /// ```
    /// use archflow_logic::ecs::entity_builder::WorldSpawnExt;
    /// use archflow_logic::{SensorType, Controller, ActuatorType};
    ///
    /// let entity = world.spawn()
    ///     .behavior("move")
    ///         .sensor(SensorType::KeyShortcut)
    ///         .controller(Controller::Direct)
    ///         .actuator(ActuatorType::Move)
    ///     .build();
    /// ```
    #[inline]
    pub fn behavior(self, name: impl Into<String>) -> BehaviorBlockBuilder<'w> {
        BehaviorBlockBuilder::new(self, name)
    }

    /// Finalizes the entity creation.
    ///
    /// This method:
    /// 1. Returns the created entity
    /// 2. If a name was set, adds a NamedComponent to the entity
    ///
    /// # Returns
    ///
    /// The `EntityId` of the created entity.
    ///
    /// # Example
    ///
    /// ```
    /// use archflow_logic::ecs::entity_builder::WorldSpawnExt;
    ///
    /// let entity = world.spawn()
    ///     .insert(Position { x: 0.0, y: 0.0 })
    ///     .build();
    /// ```
    #[inline]
    pub fn build(mut self) -> EntityId {
        // If a name was set, add NamedComponent to the entity
        if let Some(name) = self.name.take() {
            let _ = self
                .world
                .add_component(self.entity, NamedComponent::new(name));
        }

        // If behaviors were defined, add BehaviorBlockComponent to the entity
        if !self.behaviors.is_empty() {
            let _ = self.world.add_component(
                self.entity,
                BehaviorBlockComponent::with_blocks(self.behaviors),
            );
        }

        self.entity
    }
}

impl<'w> EntityBuildable<'w> for EntityBuilder<'w> {
    type Output = EntityId;

    #[inline]
    fn insert<C: Component>(self, component: C) -> Self {
        EntityBuilder::insert(self, component)
    }

    #[inline]
    fn name(self, name: impl Into<String>) -> Self {
        EntityBuilder::name(self, name)
    }

    #[inline]
    fn build(self) -> Self::Output {
        EntityBuilder::build(self)
    }
}
