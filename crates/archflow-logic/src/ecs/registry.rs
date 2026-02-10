// ═══════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - ECS ComponentRegistry Module
//
// This module provides the ComponentRegistry for dynamic component management
// in the Entity Component System (ECS).
// ═══════════════════════════════════════════════════════════════════════════════════════

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use core::any::{Any, TypeId};

use super::component::{Component, ComponentId, ComponentStorage};
use super::sparse_set::SparseSet;

/// Registry for managing component storage dynamically
///
/// Provides runtime registration and type-safe access to component storage.
/// Internally uses a BTreeMap to store component storage by TypeId.
pub struct ComponentRegistry {
    /// Map from TypeId to boxed component storage
    storages: BTreeMap<TypeId, Box<dyn AnyComponentStorage>>,
}

impl core::fmt::Debug for ComponentRegistry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ComponentRegistry")
            .field("storages", &self.storages.len())
            .finish()
    }
}

impl ComponentRegistry {
    /// Creates a new empty ComponentRegistry
    #[inline]
    pub fn new() -> Self {
        Self {
            storages: BTreeMap::new(),
        }
    }

    /// Registers a new component type with its default storage
    ///
    /// # Panics
    ///
    /// Panics if the component type is already registered.
    #[inline]
    pub fn register<T: Component>(&mut self) {
        let type_id = TypeId::of::<T>();

        if self.storages.contains_key(&type_id) {
            panic!(
                "Component {:?} is already registered",
                core::any::type_name::<T>()
            );
        }

        let storage = T::Storage::default();
        self.storages.insert(
            type_id,
            Box::new(AnyStorageWrapper::<T::Storage>::new(storage)),
        );
    }

    /// Registers a component type with a custom storage instance
    ///
    /// # Panics
    ///
    /// Panics if the component type is already registered.
    #[inline]
    pub fn register_with_storage<T: Component>(&mut self, storage: T::Storage) {
        let type_id = TypeId::of::<T>();

        if self.storages.contains_key(&type_id) {
            panic!(
                "Component {:?} is already registered",
                core::any::type_name::<T>()
            );
        }

        self.storages.insert(
            type_id,
            Box::new(AnyStorageWrapper::<T::Storage>::new(storage)),
        );
    }

    /// Gets the storage for a component type
    ///
    /// Returns `None` if the component type is not registered.
    #[inline]
    pub fn get_storage<T: Component>(&self) -> Option<&T::Storage> {
        let type_id = TypeId::of::<T>();
        self.storages
            .get(&type_id)
            .and_then(|any| any.as_any().downcast_ref::<T::Storage>())
    }

    /// Gets mutable storage for a component type
    ///
    /// Returns `None` if the component type is not registered.
    #[inline]
    pub fn get_storage_mut<T: Component>(&mut self) -> Option<&mut T::Storage> {
        let type_id = TypeId::of::<T>();
        self.storages
            .get_mut(&type_id)
            .and_then(|any| any.as_any_mut().downcast_mut::<T::Storage>())
    }

    /// Returns true if a component type is registered
    #[inline]
    pub fn is_registered<T: Component>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.storages.contains_key(&type_id)
    }

    /// Returns the number of registered component types
    #[inline]
    pub fn len(&self) -> usize {
        self.storages.len()
    }

    /// Returns true if no component types are registered
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.storages.is_empty()
    }

    /// Clears all component storage
    #[inline]
    pub fn clear(&mut self) {
        self.storages.clear();
    }
}

impl Default for ComponentRegistry {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Type-erased component storage trait
trait AnyComponentStorage {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Wrapper for type-erasing ComponentStorage implementations
struct AnyStorageWrapper<S: ComponentStorage> {
    storage: S,
}

impl<S: ComponentStorage> AnyStorageWrapper<S> {
    #[inline]
    fn new(storage: S) -> Self {
        Self { storage }
    }
}

impl<S: ComponentStorage + core::fmt::Debug> core::fmt::Debug for AnyStorageWrapper<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AnyStorageWrapper")
            .field("storage", &self.storage)
            .finish()
    }
}

impl<S: ComponentStorage + 'static> AnyComponentStorage for AnyStorageWrapper<S> {
    #[inline]
    fn as_any(&self) -> &dyn Any {
        &self.storage
    }

    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        &mut self.storage
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::component::VecStorage;
    use crate::ecs::sparse_set::SparseSet;

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

    #[derive(Clone, Debug, PartialEq, Default)]
    struct Health {
        current: u32,
        max: u32,
    }

    impl Component for Health {
        type Storage = SparseSet<Health>;
    }

    #[test]
    fn test_register_component() {
        let mut registry = ComponentRegistry::new();

        assert!(!registry.is_registered::<Position>());
        assert_eq!(registry.len(), 0);

        registry.register::<Position>();

        assert!(registry.is_registered::<Position>());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    #[should_panic(expected = "already registered")]
    fn test_register_duplicate_panics() {
        let mut registry = ComponentRegistry::new();

        registry.register::<Position>();
        registry.register::<Position>(); // Should panic
    }

    #[test]
    fn test_component_storage_retrieval() {
        let mut registry = ComponentRegistry::new();

        registry.register::<Position>();
        registry.register::<Velocity>();

        // Get immutable storage
        let positions = registry.get_storage::<Position>();
        assert!(positions.is_some());

        // Get mutable storage
        let positions = registry.get_storage_mut::<Position>();
        assert!(positions.is_some());

        if let Some(storage) = positions {
            storage.insert(0, Position { x: 1.0, y: 2.0 });
            storage.insert(5, Position { x: 3.0, y: 4.0 });
        }

        // Verify data was stored
        let positions = registry.get_storage::<Position>();
        assert_eq!(
            positions.and_then(|p| p.get(0)),
            Some(&Position { x: 1.0, y: 2.0 })
        );
        assert_eq!(
            positions.and_then(|p| p.get(5)),
            Some(&Position { x: 3.0, y: 4.0 })
        );
    }

    #[test]
    fn test_get_unregistered_storage() {
        let mut registry = ComponentRegistry::new();

        assert!(registry.get_storage::<Position>().is_none());
        assert!(registry.get_storage_mut::<Position>().is_none());
    }

    #[test]
    fn test_multiple_components() {
        let mut registry = ComponentRegistry::new();

        registry.register::<Position>();
        registry.register::<Velocity>();
        registry.register::<Health>();

        assert_eq!(registry.len(), 3);
        assert!(registry.is_registered::<Position>());
        assert!(registry.is_registered::<Velocity>());
        assert!(registry.is_registered::<Health>());
    }

    #[test]
    fn test_registry_clear() {
        let mut registry = ComponentRegistry::new();

        registry.register::<Position>();
        registry.register::<Velocity>();

        assert_eq!(registry.len(), 2);

        registry.clear();

        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
        assert!(!registry.is_registered::<Position>());
    }

    #[test]
    fn test_sparse_set_storage() {
        let mut registry = ComponentRegistry::new();

        registry.register::<Health>();

        let health = registry.get_storage_mut::<Health>();
        assert!(health.is_some());

        if let Some(storage) = health {
            storage.insert(
                0,
                Health {
                    current: 100,
                    max: 100,
                },
            );
            storage.insert(
                10,
                Health {
                    current: 50,
                    max: 100,
                },
            );
        }

        // Verify data
        let health = registry.get_storage::<Health>();
        assert_eq!(
            health.and_then(|h| h.get(0)),
            Some(&Health {
                current: 100,
                max: 100
            })
        );
        assert_eq!(
            health.and_then(|h| h.get(10)),
            Some(&Health {
                current: 50,
                max: 100
            })
        );
        assert_eq!(health.and_then(|h| h.get(5)), None);
    }

    #[test]
    fn test_register_with_custom_storage() {
        let mut registry = ComponentRegistry::new();

        // Create a SparseSet with custom capacity
        let sparse_set = SparseSet::<Health>::with_capacity(100, 50);
        registry.register_with_storage::<Health>(sparse_set);

        assert!(registry.is_registered::<Health>());

        let health = registry.get_storage_mut::<Health>();
        assert!(health.is_some());
    }

    #[test]
    fn test_component_isolation() {
        let mut registry = ComponentRegistry::new();

        registry.register::<Position>();
        registry.register::<Velocity>();

        // Add Position data
        {
            let positions = registry.get_storage_mut::<Position>().unwrap();
            positions.insert(0, Position { x: 1.0, y: 2.0 });
        }

        // Add Velocity data
        {
            let velocities = registry.get_storage_mut::<Velocity>().unwrap();
            velocities.insert(0, Velocity { dx: 0.5, dy: 0.5 });
        }

        // Verify data is isolated
        let positions = registry.get_storage::<Position>().unwrap();
        let velocities = registry.get_storage::<Velocity>().unwrap();

        assert_eq!(positions.get(0), Some(&Position { x: 1.0, y: 2.0 }));
        assert_eq!(velocities.get(0), Some(&Velocity { dx: 0.5, dy: 0.5 }));
    }

    #[test]
    fn test_registry_default() {
        let registry = ComponentRegistry::default();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_multiple_entities_same_component() {
        let mut registry = ComponentRegistry::new();

        registry.register::<Position>();

        {
            let positions = registry.get_storage_mut::<Position>().unwrap();
            for i in 0..10 {
                positions.insert(
                    i,
                    Position {
                        x: i as f32,
                        y: i as f32 * 2.0,
                    },
                );
            }
        }

        let positions = registry.get_storage::<Position>().unwrap();

        for i in 0..10 {
            assert_eq!(
                positions.get(i),
                Some(&Position {
                    x: i as f32,
                    y: i as f32 * 2.0
                })
            );
        }
    }

    #[test]
    fn test_component_modification() {
        let mut registry = ComponentRegistry::new();

        registry.register::<Position>();

        // Insert initial value
        {
            let positions = registry.get_storage_mut::<Position>().unwrap();
            positions.insert(0, Position { x: 1.0, y: 2.0 });
        }

        // Modify value
        {
            let positions = registry.get_storage_mut::<Position>().unwrap();
            if let Some(pos) = positions.get_mut(0) {
                pos.x = 10.0;
                pos.y = 20.0;
            }
        }

        // Verify modification
        let positions = registry.get_storage::<Position>().unwrap();
        assert_eq!(positions.get(0), Some(&Position { x: 10.0, y: 20.0 }));
    }
}
