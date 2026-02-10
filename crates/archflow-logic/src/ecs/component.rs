// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - ECS Component Module
//
// This module provides the core Component trait and base types for the Entity
// Component System (ECS) implementation.
//
// Key Features:
// - Component trait: Marker trait for component types
// - ComponentId: Unique identifier for component types
// - Type-safe component registration and storage
//
// Architecture:
// - Zero-cost abstraction: Component is a marker trait with no runtime overhead
// - Type-safe: Uses TypeId for compile-time component identification
// - No_std compatible: Uses alloc for collections
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::vec::Vec;
use core::any::TypeId;

/// Unique identifier for a component type
///
/// Internally uses `TypeId` but provides a type-safe wrapper for component
/// identification in the ECS registry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentId {
    /// Internal TypeId
    type_id: TypeId,
}

impl ComponentId {
    /// Creates a new ComponentId from a TypeId
    #[inline]
    pub const fn new(type_id: TypeId) -> Self {
        Self { type_id }
    }

    /// Creates a ComponentId for type T
    #[inline]
    pub fn of<T: 'static>() -> Self {
        Self::new(TypeId::of::<T>())
    }

    /// Returns the internal TypeId
    #[inline]
    pub const fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// Creates a ComponentId from a type name (for testing/debugging)
    #[inline]
    pub fn from_type_name<T: 'static>() -> Self {
        Self::of::<T>()
    }
}

/// Marker trait for types that can be used as ECS components
///
/// Any type that implements this trait can be registered with the ComponentRegistry
/// and stored in component storage.
///
/// # Requirements
///
/// Components must:
/// - Be `'static` (no borrowed data)
/// - Be `Send + Sync` (safe to share across threads)
/// - Implement `Sized` (stored inline in arrays/vectors)
///
/// # Examples
///
/// ```
/// use archflow_logic::ecs::Component;
///
/// #[derive(Clone, Debug)]
/// struct Position {
///     x: f32,
///     y: f32,
/// }
///
/// impl Component for Position {
///     type Storage = Vec<Position>;
/// }
/// ```
pub trait Component: 'static + Send + Sync + Sized {
    /// The storage type used for this component
    ///
    /// This allows custom storage strategies per component type.
    /// Most components will use `Vec<Self>` or `SparseSet<Self>`.
    type Storage: ComponentStorage<Item = Self>;
}

/// Trait for component storage backends
///
/// Defines the interface for storing and accessing component data.
/// Implementations can use different strategies (dense arrays, sparse sets, etc.).
pub trait ComponentStorage: 'static + Default {
    /// The type of component being stored
    type Item;

    /// Inserts a component at the given entity index
    fn insert(&mut self, entity_index: usize, component: Self::Item);

    /// Removes a component at the given entity index
    fn remove(&mut self, entity_index: usize) -> Option<Self::Item>;

    /// Gets a reference to the component at the given entity index
    fn get(&self, entity_index: usize) -> Option<&Self::Item>;

    /// Gets a mutable reference to the component at the given entity index
    fn get_mut(&mut self, entity_index: usize) -> Option<&mut Self::Item>;

    /// Returns true if a component exists for the given entity
    fn contains(&self, entity_index: usize) -> bool;

    /// Returns the number of stored components
    fn len(&self) -> usize;

    /// Returns true if no components are stored
    fn is_empty(&self) -> bool;

    /// Clears all components
    fn clear(&mut self);
}

/// Default storage implementation using a simple Vec
///
/// Stores components in a dense vector with optional entries.
/// Simple but memory-inefficient for sparse components.
#[derive(Debug)]
pub struct VecStorage<T: Component> {
    /// Internal storage, using Option to represent missing components
    data: Vec<Option<T>>,
}

impl<T: Component> VecStorage<T> {
    /// Creates a new empty VecStorage
    #[inline]
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Creates a VecStorage with the given capacity
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    /// Ensures the internal vector has capacity for the given entity index
    #[inline]
    fn ensure_capacity(&mut self, entity_index: usize) {
        if entity_index >= self.data.len() {
            while entity_index >= self.data.len() { self.data.push(None); }
        }
    }
}

impl<T: Component> Default for VecStorage<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Component> ComponentStorage for VecStorage<T> {
    type Item = T;

    #[inline]
    fn insert(&mut self, entity_index: usize, component: T) {
        self.ensure_capacity(entity_index);
        self.data[entity_index] = Some(component);
    }

    #[inline]
    fn remove(&mut self, entity_index: usize) -> Option<T> {
        if entity_index < self.data.len() {
            self.data[entity_index].take()
        } else {
            None
        }
    }

    #[inline]
    fn get(&self, entity_index: usize) -> Option<&T> {
        self.data.get(entity_index).and_then(|opt| opt.as_ref())
    }

    #[inline]
    fn get_mut(&mut self, entity_index: usize) -> Option<&mut T> {
        self.data.get_mut(entity_index).and_then(|opt| opt.as_mut())
    }

    #[inline]
    fn contains(&self, entity_index: usize) -> bool {
        self.get(entity_index).is_some()
    }

    #[inline]
    fn len(&self) -> usize {
        self.data.iter().filter(|opt| opt.is_some()).count()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    fn clear(&mut self) {
        self.data.clear();
    }
}

impl<T: Component> VecStorage<T> {
    /// Returns an iterator over all components (including None values)
    #[inline]
    pub fn iter(&self) -> core::slice::Iter<Option<T>> {
        self.data.iter()
    }

    /// Returns a mutable iterator over all components (including None values)
    #[inline]
    pub fn iter_mut(&mut self) -> core::slice::IterMut<Option<T>> {
        self.data.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct TestComponent {
        value: u32,
    }

    impl Component for TestComponent {
        type Storage = VecStorage<TestComponent>;
    }

    #[test]
    fn test_component_id_of() {
        let id = ComponentId::of::<TestComponent>();
        assert_eq!(id.type_id(), TypeId::of::<TestComponent>());
    }

    #[test]
    fn test_component_id_equality() {
        let id1 = ComponentId::of::<TestComponent>();
        let id2 = ComponentId::of::<TestComponent>();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_component_id_inequality() {
        #[derive(Clone, Debug, PartialEq)]
        struct OtherComponent;

        impl Component for OtherComponent {
            type Storage = VecStorage<OtherComponent>;
        }

        let id1 = ComponentId::of::<TestComponent>();
        let id2 = ComponentId::of::<OtherComponent>();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_vec_storage_insert() {
        let mut storage = VecStorage::<TestComponent>::new();
        storage.insert(0, TestComponent { value: 42 });
        storage.insert(5, TestComponent { value: 100 });

        assert_eq!(storage.get(0).unwrap().value, 42);
        assert_eq!(storage.get(5).unwrap().value, 100);
    }

    #[test]
    fn test_vec_storage_remove() {
        let mut storage = VecStorage::<TestComponent>::new();
        storage.insert(0, TestComponent { value: 42 });

        let removed = storage.remove(0);
        assert_eq!(removed.unwrap().value, 42);
        assert!(!storage.contains(0));
    }

    #[test]
    fn test_vec_storage_get_mut() {
        let mut storage = VecStorage::<TestComponent>::new();
        storage.insert(0, TestComponent { value: 42 });

        if let Some(comp) = storage.get_mut(0) {
            comp.value = 100;
        }

        assert_eq!(storage.get(0).unwrap().value, 100);
    }

    #[test]
    fn test_vec_storage_len() {
        let mut storage = VecStorage::<TestComponent>::new();
        assert_eq!(storage.len(), 0);
        assert!(storage.is_empty());

        storage.insert(0, TestComponent { value: 1 });
        storage.insert(5, TestComponent { value: 2 });

        assert_eq!(storage.len(), 2);
        assert!(!storage.is_empty());
    }

    #[test]
    fn test_vec_storage_clear() {
        let mut storage = VecStorage::<TestComponent>::new();
        storage.insert(0, TestComponent { value: 1 });
        storage.insert(1, TestComponent { value: 2 });

        storage.clear();

        assert_eq!(storage.len(), 0);
        assert!(storage.is_empty());
    }

    #[test]
    fn test_vec_storage_contains() {
        let mut storage = VecStorage::<TestComponent>::new();

        assert!(!storage.contains(0));

        storage.insert(0, TestComponent { value: 42 });
        assert!(storage.contains(0));
        assert!(!storage.contains(1));
    }

    #[test]
    fn test_vec_storage_out_of_bounds() {
        let mut storage = VecStorage::<TestComponent>::new();

        assert_eq!(storage.get(100), None);
        assert_eq!(storage.remove(100), None);
        assert!(!storage.contains(100));
    }

    #[test]
    fn test_vec_storage_with_capacity() {
        let storage = VecStorage::<TestComponent>::with_capacity(10);
        // Internal vec should have capacity 10, but length 0
        assert!(storage.is_empty());
    }
}
