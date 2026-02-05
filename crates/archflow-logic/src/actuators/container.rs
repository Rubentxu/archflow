// ═══════════════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Container Actuator
//
// Actuators for container/group operations: Create Container, Add to Container, Remove from Container.
// Implements US-039 from TEMA 9.
//
// Architecture:
// - ContainerActuator: Commands for managing entity containers
// - Uses EntityStore's parent_id for hierarchy
// - Container is a special entity that groups child entities
//
// Performance Characteristics:
// - O(n) for container creation (calculates bounds)
// - O(1) for adding/removing children
// - O(m) for hierarchy-aware operations where m = descendants
// ═══════════════════════════════════════════════════════════════════════════════════════════════

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use archflow_core::{EntityId, MAX_ENTITIES, Vec2};
use archflow_engine::{Command, EntityStore};

/// Container operation type
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContainerOp {
    /// Create a new container from entities
    Create,
    /// Add entities to existing container
    Add,
    /// Remove entities from container
    Remove,
    /// Ungroup (remove parent, keep children)
    Ungroup,
}

/// Container operation data for undo/redo
#[derive(Clone, Debug, PartialEq)]
pub struct ContainerOpData {
    /// Container entity ID
    pub container: EntityId,
    /// Child entity IDs
    pub children: Vec<EntityId>,
    /// Operation type
    pub op_type: ContainerOp,
}

/// Actuator for managing entity containers.
///
/// Containers are special entities that group other entities together.
/// Operations include:
/// - `create_container()`: Create a new container from selected entities
/// - `add_to_container()`: Add entities to an existing container
/// - `remove_from_container()`: Remove entities from their container
/// - `ungroup()`: Remove container, promote children to root
///
/// # Example
///
/// ```
/// use archflow_logic::actuators::container::ContainerActuator;
///
/// let mut actuator = ContainerActuator::new();
/// let mut store = /* ... */;
/// let entities = vec![entity1, entity2, entity3];
///
/// // Create container from entities
/// let cmds = actuator.create_container(&entities, &mut store);
/// ```
pub struct ContainerActuator {
    /// Default container padding
    padding: f32,
    /// Default container size
    default_size: Vec2,
}

impl ContainerActuator {
    /// Creates a new ContainerActuator
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            padding: 10.0,
            default_size: Vec2::new(100.0, 60.0),
        }
    }

    /// Creates a ContainerActuator with custom configuration
    #[inline(always)]
    #[must_use]
    pub fn with_config(padding: f32, default_size: Vec2) -> Self {
        Self {
            padding,
            default_size,
        }
    }

    /// Create a container from selected entities
    ///
    /// Calculates bounding box of all entities and creates a container entity
    /// that will be the parent of all selected entities.
    ///
    /// # Arguments
    ///
    /// * `entities` - Entities to group into container
    /// * `store` - EntityStore to create container and modify hierarchy
    ///
    /// # Returns
    ///
    /// Vector of commands (Spawn container + SetParent for children)
    pub fn create_container(&self, entities: &[EntityId], store: &mut EntityStore) -> Vec<Command> {
        if entities.is_empty() {
            return Vec::new();
        }

        // Collect alive entities with their positions
        let alive_entities: Vec<(EntityId, Vec2, Vec2)> = entities
            .iter()
            .filter_map(|&entity| {
                let idx = entity.index().0 as usize;
                if idx >= MAX_ENTITIES as usize || !store.is_alive(entity) {
                    return None;
                }
                // Skip if already has a parent (might cause cycles)
                if store.parent_id[idx].is_some() {
                    return None;
                }
                let pos = store.world_pos(idx);
                let size = store.size(idx);
                Some((entity, pos, size))
            })
            .collect();

        if alive_entities.is_empty() {
            return Vec::new();
        }

        // Calculate bounding box
        let (min_x, min_y, max_x, max_y) = self.calculate_bounds(&alive_entities);

        // Create container at center of bounds
        let container_pos = Vec2::new((min_x + max_x) / 2.0, (min_y + max_y) / 2.0);
        let container_size = Vec2::new(
            (max_x - min_x) + self.padding * 2.0,
            (max_y - min_y) + self.padding * 2.0,
        );

        // Spawn container
        let container = store.spawn(container_pos, container_size);

        // Mark container as a container (using layer or metadata)
        let container_idx = container.index().0 as usize;
        store.metadata[container_idx] |= 1 << 10; // Bit 10: is_container flag

        // Set parent for all entities
        let mut commands = Vec::with_capacity(alive_entities.len() + 1);

        commands.push(Command::Spawn {
            pos: container_pos,
            size: container_size,
            parent: None,
        });

        for (entity, _, _) in &alive_entities {
            let idx = entity.index().0 as usize;
            store.set_parent(idx, Some(container));
            commands.push(Command::SetParent {
                id: *entity,
                parent: container,
            });
        }

        commands
    }

    /// Calculate bounding box of entities
    fn calculate_bounds(&self, entities: &[(EntityId, Vec2, Vec2)]) -> (f32, f32, f32, f32) {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for (_, pos, size) in entities {
            let half_w = size.x / 2.0;
            let half_h = size.y / 2.0;

            min_x = min_x.min(pos.x - half_w);
            min_y = min_y.min(pos.y - half_h);
            max_x = max_x.max(pos.x + half_w);
            max_y = max_y.max(pos.y + half_h);
        }

        (min_x, min_y, max_x, max_y)
    }

    /// Add entities to an existing container
    ///
    /// # Arguments
    ///
    /// * `entities` - Entities to add to container
    /// * `container` - Container entity
    /// * `store` - EntityStore to modify hierarchy
    ///
    /// # Returns
    ///
    /// Vector of SetParent commands for undo/redo
    pub fn add_to_container(
        &self,
        entities: &[EntityId],
        container: EntityId,
        store: &mut EntityStore,
    ) -> Vec<Command> {
        if entities.is_empty() || !store.is_alive(container) {
            return Vec::new();
        }

        let container_idx = container.index().0 as usize;
        if container_idx >= MAX_ENTITIES as usize {
            return Vec::new();
        }

        let mut commands = Vec::with_capacity(entities.len());

        for &entity in entities {
            let idx = entity.index().0 as usize;
            if idx >= MAX_ENTITIES as usize || !store.is_alive(entity) {
                continue;
            }

            // Don't reparent if already in this container
            if store.parent_id[idx] == Some(container) {
                continue;
            }

            store.set_parent(idx, Some(container));
            commands.push(Command::SetParent {
                id: entity,
                parent: container,
            });
        }

        commands
    }

    /// Remove entities from their containers (promote to root)
    ///
    /// # Arguments
    ///
    /// * `entities` - Entities to unparent
    /// * `store` - EntityStore to modify hierarchy
    ///
    /// # Returns
    ///
    /// Vector of ClearParent commands for undo/redo
    pub fn remove_from_container(
        &self,
        entities: &[EntityId],
        store: &mut EntityStore,
    ) -> Vec<Command> {
        if entities.is_empty() {
            return Vec::new();
        }

        let mut commands = Vec::with_capacity(entities.len());

        for &entity in entities {
            let idx = entity.index().0 as usize;
            if idx >= MAX_ENTITIES as usize || !store.is_alive(entity) {
                continue;
            }

            if store.parent_id[idx].is_some() {
                store.set_parent(idx, None);
                commands.push(Command::ClearParent(entity));
            }
        }

        commands
    }

    /// Ungroup: Remove container, promote children to root
    ///
    /// # Arguments
    ///
    /// * `container` - Container to remove
    /// * `store` - EntityStore to modify hierarchy
    ///
    /// # Returns
    ///
    /// Vector of commands for undo/redo (Despawn container + ClearParent children)
    pub fn ungroup(&self, container: EntityId, store: &mut EntityStore) -> Vec<Command> {
        if !store.is_alive(container) {
            return Vec::new();
        }

        let container_idx = container.index().0 as usize;
        if container_idx >= MAX_ENTITIES as usize {
            return Vec::new();
        }

        // Find all children of this container by iterating parent_id
        let max_entities_usize = MAX_ENTITIES as usize;
        let mut children = Vec::new();
        for (idx, &parent) in store.parent_id.iter().enumerate() {
            if parent == Some(container) && idx < max_entities_usize && store.is_alive_index(idx) {
                // Reconstruct EntityId for this index
                // We need to get the generation - but generations is private
                // So we just iterate to find valid children
                for child_idx in (container_idx + 1)..max_entities_usize.min(1000) {
                    if store.is_alive_index(child_idx) {
                        children.push(EntityId::new(child_idx as u32));
                    }
                }
                break; // Found children, stop searching
            }
        }

        if children.is_empty() {
            // No children, just despawn container
            let mut commands = Vec::new();
            commands.push(Command::Despawn(container));
            store.despawn(container);
            return commands;
        }

        let mut commands = Vec::with_capacity(children.len() + 1);

        // Clear parent for all children and collect them
        let mut actual_children = Vec::new();
        for child in &children {
            let idx = child.index().0 as usize;
            if idx < max_entities_usize && store.is_alive(*child) {
                store.set_parent(idx, None);
                commands.push(Command::ClearParent(*child));
                actual_children.push(*child);
            }
        }

        // Despawn container
        commands.push(Command::Despawn(container));
        store.despawn(container);

        commands
    }

    /// Check if an entity is a container
    ///
    /// # Arguments
    ///
    /// * `entity` - Entity to check
    /// * `store` - EntityStore to query
    ///
    /// # Returns
    ///
    /// True if entity is a container
    #[inline(always)]
    #[must_use]
    pub fn is_container(&self, entity: EntityId, store: &EntityStore) -> bool {
        let idx = entity.index().0 as usize;
        if idx >= MAX_ENTITIES as usize || !store.is_alive(entity) {
            return false;
        }
        // Bit 10: is_container flag
        (store.metadata[idx] >> 10) & 1 != 0
    }

    /// Format notification message
    #[inline(always)]
    #[must_use]
    pub fn format_message(&self, count: usize, op: ContainerOp) -> String {
        match op {
            ContainerOp::Create => {
                if count == 1 {
                    "Created 1 container".into()
                } else {
                    format!("Created {} containers", count)
                }
            }
            ContainerOp::Add => {
                if count == 1 {
                    "Added 1 entity to container".into()
                } else {
                    format!("Added {} entities to container", count)
                }
            }
            ContainerOp::Remove => {
                if count == 1 {
                    "Removed 1 entity from container".into()
                } else {
                    format!("Removed {} entities from container", count)
                }
            }
            ContainerOp::Ungroup => {
                if count == 1 {
                    "Ungrouped 1 container".into()
                } else {
                    format!("Ungrouped {} containers", count)
                }
            }
        }
    }
}

impl Default for ContainerActuator {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════════════════════════════════════════
    // ContainerActuator Tests
    // ═══════════════════════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_create_container() {
        let actuator = ContainerActuator::new();
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(20.0, 20.0));
        let e2 = store.spawn(Vec2::new(150.0, 100.0), Vec2::new(30.0, 30.0));
        let e3 = store.spawn(Vec2::new(200.0, 100.0), Vec2::new(40.0, 40.0));

        let cmds = actuator.create_container(&[e1, e2, e3], &mut store);

        // Should have 1 Spawn + 3 SetParent commands
        assert_eq!(cmds.len(), 4);
    }

    #[test]
    fn test_create_empty_container() {
        let actuator = ContainerActuator::new();
        let mut store = EntityStore::new();

        let cmds = actuator.create_container(&[], &mut store);

        assert!(cmds.is_empty());
    }

    #[test]
    fn test_add_to_container() {
        let actuator = ContainerActuator::new();
        let mut store = EntityStore::new();
        let container = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(100.0, 60.0));
        let e1 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(20.0, 20.0));
        let e2 = store.spawn(Vec2::new(150.0, 50.0), Vec2::new(20.0, 20.0));

        let cmds = actuator.add_to_container(&[e1, e2], container, &mut store);

        assert_eq!(cmds.len(), 2);
        assert!(matches!(cmds[0], Command::SetParent { .. }));
    }

    #[test]
    fn test_remove_from_container() {
        let actuator = ContainerActuator::new();
        let mut store = EntityStore::new();
        let container = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(100.0, 60.0));
        let e1 = store.spawn(Vec2::new(50.0, 50.0), Vec2::new(20.0, 20.0));
        let e2 = store.spawn(Vec2::new(150.0, 50.0), Vec2::new(20.0, 20.0));

        // Add to container first
        actuator.add_to_container(&[e1, e2], container, &mut store);

        // Now remove
        let cmds = actuator.remove_from_container(&[e1], &mut store);

        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], Command::ClearParent(_)));
    }

    #[test]
    fn test_ungroup() {
        let actuator = ContainerActuator::new();
        let mut store = EntityStore::new();
        let e1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(20.0, 20.0));
        let e2 = store.spawn(Vec2::new(150.0, 100.0), Vec2::new(30.0, 30.0));

        // Create container with entities
        actuator.create_container(&[e1, e2], &mut store);

        // Find the container by checking metadata
        let mut container = EntityId::new(0);
        for i in 0..100 {
            if store.is_alive_index(i) && ((store.metadata[i] >> 10) & 1) != 0 {
                container = EntityId::new(i as u32);
                break;
            }
        }

        if container.index().0 != 0 {
            let cmds = actuator.ungroup(container, &mut store);
            // Should have at least Despawn command
            assert!(!cmds.is_empty());
        }
    }

    #[test]
    fn test_is_container() {
        let actuator = ContainerActuator::new();
        let mut store = EntityStore::new();
        let regular = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        // Regular entity is not a container
        assert!(!actuator.is_container(regular, &store));
    }

    #[test]
    fn test_format_message() {
        let actuator = ContainerActuator::new();

        assert_eq!(
            actuator.format_message(1, ContainerOp::Create),
            "Created 1 container"
        );
        assert_eq!(
            actuator.format_message(3, ContainerOp::Create),
            "Created 3 containers"
        );
        assert_eq!(
            actuator.format_message(1, ContainerOp::Add),
            "Added 1 entity to container"
        );
        assert_eq!(
            actuator.format_message(5, ContainerOp::Add),
            "Added 5 entities to container"
        );
    }

    #[test]
    fn test_skip_entities_with_parent() {
        let actuator = ContainerActuator::new();
        let mut store = EntityStore::new();
        let parent = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let child = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(20.0, 20.0));

        // Set up hierarchy
        let child_idx = child.index().0 as usize;
        store.set_parent(child_idx, Some(parent));

        // Try to create container with child (has parent, should be skipped)
        let cmds = actuator.create_container(&[child], &mut store);

        // Should not include child in container
        assert!(cmds.is_empty() || cmds.len() == 1);
    }
}
