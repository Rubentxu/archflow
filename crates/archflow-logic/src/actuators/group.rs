// ═══════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - GroupActuator
//
// Actuator for grouping and ungrouping entities into hierarchical structures.
// Implements US-021 and US-022 from TEMA 5.
//
// Architecture:
// - GroupActuator: Creates parent-child relationships between entities
// - UngroupActuator: Breaks parent-child relationships
// - Uses EntityStore's parent_id and local_transform for hierarchy
//
// Performance Characteristics:
// - O(n) for grouping n entities
// - O(n) for ungrouping n entities
// - World transform recalculated on parent change
// ═════════════════════════════════════════════════════════════════════════════════════

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use archflow_core::{EntityId, Vec2};
use archflow_engine::{Command, EntityStore};

/// Configuration for grouping behavior.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroupConfig {
    /// Create group as locked
    pub lock_group: bool,
    /// Center group on bounding box center
    pub center_on_bbox: bool,
    /// Preserve relative positions when grouping
    pub preserve_positions: bool,
}

impl Default for GroupConfig {
    fn default() -> Self {
        Self {
            lock_group: false,
            center_on_bbox: true,
            preserve_positions: true,
        }
    }
}

/// Operation data for undo/redo.
#[derive(Clone, Debug, PartialEq)]
pub struct GroupOp {
    /// The parent entity created (None for ungroup)
    pub parent_id: Option<EntityId>,
    /// The child entities in the group
    pub child_ids: Vec<EntityId>,
    /// Previous parent of each child (None = was root)
    pub previous_parents: Vec<Option<EntityId>>,
    /// Previous local transforms
    pub previous_local_transforms: Vec<[f32; 4]>,
}

/// Result of a grouping operation.
#[derive(Clone, Debug, PartialEq)]
pub struct GroupResult {
    /// The created parent entity (if grouping)
    pub parent_id: Option<EntityId>,
    /// The entities that were grouped
    pub child_ids: Vec<EntityId>,
    /// Operation performed
    pub op_type: GroupOpType,
    /// Whether the operation succeeded
    pub success: bool,
}

/// Type of group operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupOpType {
    /// Created a new group
    Group,
    /// Added to existing group
    AddToGroup,
    /// Removed from group (ungroup)
    Ungroup,
}

/// Actuator for grouping entities.
///
/// Provides grouping functionality:
/// - Create groups from selected entities
/// - Add entities to existing groups
/// - Ungroup entities (restore to root)
/// - Configurable behavior for positions and locking
///
/// # Performance
/// - O(n) for grouping n entities
/// - O(n) for ungrouping n entities
///
/// # Example
///
/// ```
/// use archflow_logic::actuators::group::{GroupActuator, GroupConfig};
///
/// let mut actuator = GroupActuator::new();
/// let entities = vec![e1, e2, e3];
///
/// // Group entities
/// let result = actuator.group_entities(&entities, &mut store);
/// ```
pub struct GroupActuator {
    /// Configuration for grouping
    config: GroupConfig,
}

impl GroupActuator {
    /// Creates a new GroupActuator with default config.
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: GroupConfig::default(),
        }
    }

    /// Creates a GroupActuator with custom config.
    #[inline(always)]
    #[must_use]
    pub fn with_config(config: GroupConfig) -> Self {
        Self { config }
    }

    /// Returns the current configuration.
    #[inline(always)]
    #[must_use]
    pub fn config(&self) -> GroupConfig {
        self.config
    }

    /// Updates the configuration.
    #[inline(always)]
    pub fn set_config(&mut self, config: GroupConfig) {
        self.config = config;
    }

    /// Groups multiple entities into a single parent.
    ///
    /// # Arguments
    ///
    /// * `entities` - Entities to group
    /// * `store` - Entity store
    ///
    /// # Returns
    ///
    /// Group result with parent and children
    pub fn group_entities(&self, entities: &[EntityId], store: &mut EntityStore) -> GroupResult {
        if entities.is_empty() {
            return GroupResult {
                parent_id: None,
                child_ids: Vec::new(),
                op_type: GroupOpType::Group,
                success: false,
            };
        }

        // Calculate bounding box center
        let (min_x, max_x, min_y, max_y) = self.calculate_bbox(entities, store);
        let center_x = (min_x + max_x) / 2.0;
        let center_y = (min_y + max_y) / 2.0;
        let width = max_x - min_x;
        let height = max_y - min_y;

        // Create parent entity
        let parent_id = store.spawn(Vec2::new(center_x, center_y), Vec2::new(width, height));

        // Set metadata to mark as group container
        let parent_idx = parent_id.index().0 as usize;
        store.metadata[parent_idx] |= 0x10; // Set group/container flag

        // Store previous state for undo
        let mut previous_parents: Vec<Option<EntityId>> = Vec::new();
        let mut previous_local_transforms: Vec<[f32; 4]> = Vec::new();

        // Reparent all entities
        for &entity_id in entities {
            let idx = entity_id.index().0 as usize;
            if idx >= store.transforms.len() {
                continue;
            }

            // Save previous state
            previous_parents.push(store.parent_id[idx]);
            previous_local_transforms.push(store.local_transform[idx]);

            // Set new parent
            store.parent_id[idx] = Some(parent_id);

            // Calculate and store local transform
            let world_transform = store.transforms[idx];
            let local_x = world_transform[0] - center_x;
            let local_y = world_transform[1] - center_y;
            store.local_transform[idx] = [local_x, local_y, world_transform[2], world_transform[3]];

            // Mark for hierarchy update
            store.dirty_hierarchy.insert(idx);
        }

        GroupResult {
            parent_id: Some(parent_id),
            child_ids: entities.to_vec(),
            op_type: GroupOpType::Group,
            success: true,
        }
    }

    /// Adds entities to an existing group.
    ///
    /// # Arguments
    ///
    /// * `entities` - Entities to add
    /// * `parent_id` - Existing group parent
    /// * `store` - Entity store
    ///
    /// # Returns
    ///
    /// Group result
    pub fn add_to_group(
        &self,
        entities: &[EntityId],
        parent_id: EntityId,
        store: &mut EntityStore,
    ) -> GroupResult {
        let parent_idx = parent_id.index().0 as usize;
        if parent_idx >= store.transforms.len() {
            return GroupResult {
                parent_id: Some(parent_id),
                child_ids: entities.to_vec(),
                op_type: GroupOpType::AddToGroup,
                success: false,
            };
        }

        let parent_pos = Vec2::new(
            store.transforms[parent_idx][0],
            store.transforms[parent_idx][1],
        );

        let mut previous_parents: Vec<Option<EntityId>> = Vec::new();
        let mut previous_local_transforms: Vec<[f32; 4]> = Vec::new();

        for &entity_id in entities {
            let idx = entity_id.index().0 as usize;
            if idx >= store.transforms.len() {
                continue;
            }

            // Save previous state
            previous_parents.push(store.parent_id[idx]);
            previous_local_transforms.push(store.local_transform[idx]);

            // Set new parent
            store.parent_id[idx] = Some(parent_id);

            // Calculate local transform
            let world_transform = store.transforms[idx];
            let local_x = world_transform[0] - parent_pos.x;
            let local_y = world_transform[1] - parent_pos.y;
            store.local_transform[idx] = [local_x, local_y, world_transform[2], world_transform[3]];

            store.dirty_hierarchy.insert(idx);
        }

        GroupResult {
            parent_id: Some(parent_id),
            child_ids: entities.to_vec(),
            op_type: GroupOpType::AddToGroup,
            success: true,
        }
    }

    /// Removes entities from their parent (ungroup).
    ///
    /// # Arguments
    ///
    /// * `entities` - Entities to ungroup
    /// * `store` - Entity store
    ///
    /// # Returns
    ///
    /// Group result with ungrouped entities
    pub fn ungroup_entities(&self, entities: &[EntityId], store: &mut EntityStore) -> GroupResult {
        let mut ungrouped: Vec<EntityId> = Vec::new();

        for &entity_id in entities {
            let idx = entity_id.index().0 as usize;
            if idx >= store.transforms.len() {
                continue;
            }

            if store.parent_id[idx].is_some() {
                // Calculate world position from local + parent
                let local = store.local_transform[idx];
                if let Some(parent_id) = store.parent_id[idx] {
                    let parent_idx = parent_id.index().0 as usize;
                    if parent_idx < store.transforms.len() {
                        store.transforms[idx][0] = store.transforms[parent_idx][0] + local[0];
                        store.transforms[idx][1] = store.transforms[parent_idx][1] + local[1];
                    }
                }
                store.transforms[idx][2] = local[2];
                store.transforms[idx][3] = local[3];

                // Remove parent
                store.parent_id[idx] = None;
                store.dirty_hierarchy.insert(idx);
                ungrouped.push(entity_id);
            }
        }

        GroupResult {
            parent_id: None,
            child_ids: ungrouped.clone(),
            op_type: GroupOpType::Ungroup,
            success: !ungrouped.is_empty(),
        }
    }

    /// Ungroups all children of a parent entity.
    ///
    /// # Arguments
    ///
    /// * `parent_id` - Parent entity to ungroup
    /// * `store` - Entity store
    ///
    /// # Returns
    ///
    /// Group result
    pub fn ungroup_all(&self, parent_id: EntityId, store: &mut EntityStore) -> GroupResult {
        let parent_idx = parent_id.index().0 as usize;
        if parent_idx >= store.transforms.len() {
            return GroupResult {
                parent_id: Some(parent_id),
                child_ids: Vec::new(),
                op_type: GroupOpType::Ungroup,
                success: false,
            };
        }

        // Find all children of this parent
        let mut children: Vec<EntityId> = Vec::new();
        for idx in 0..store.parent_id.len() {
            if store.parent_id[idx] == Some(parent_id) {
                children.push(EntityId::from_parts(
                    archflow_core::Index(idx as u32),
                    archflow_core::Generation(store.generation(idx)),
                ));
            }
        }

        // Ungroup all children
        self.ungroup_entities(&children, store)
    }

    /// Calculates bounding box for entities.
    #[inline(always)]
    fn calculate_bbox(&self, entities: &[EntityId], store: &EntityStore) -> (f32, f32, f32, f32) {
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        for &entity_id in entities {
            let idx = entity_id.index().0 as usize;
            if idx >= store.transforms.len() {
                continue;
            }

            let t = store.transforms[idx];
            let x = t[0];
            let y = t[1];
            let w = t[2];
            let h = t[3];

            min_x = min_x.min(x);
            max_x = max_x.max(x + w);
            min_y = min_y.min(y);
            max_y = max_y.max(y + h);
        }

        (min_x, max_x, min_y, max_y)
    }

    /// Checks if an entity is a group container.
    #[inline(always)]
    pub fn is_group(&self, entity_id: EntityId, store: &EntityStore) -> bool {
        let idx = entity_id.index().0 as usize;
        if idx >= store.metadata.len() {
            return false;
        }
        (store.metadata[idx] & 0x10) != 0
    }

    /// Gets all children of a group.
    #[inline(always)]
    pub fn get_children(&self, parent_id: EntityId, store: &EntityStore) -> Vec<EntityId> {
        let parent_idx = parent_id.index().0 as usize;
        if parent_idx >= store.parent_id.len() {
            return Vec::new();
        }

        let mut children: Vec<EntityId> = Vec::new();
        for idx in 0..store.parent_id.len() {
            if store.parent_id[idx] == Some(parent_id) {
                children.push(EntityId::from_parts(
                    archflow_core::Index(idx as u32),
                    archflow_core::Generation(store.generation(idx)),
                ));
            }
        }
        children
    }
}

impl Default for GroupActuator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_engine::EntityStore;

    fn create_test_store() -> EntityStore {
        EntityStore::new()
    }

    #[test]
    fn test_group_entities() {
        let mut store = create_test_store();
        let e1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));
        let e2 = store.spawn(Vec2::new(100.0, 0.0), Vec2::new(50.0, 50.0));
        let e3 = store.spawn(Vec2::new(50.0, 100.0), Vec2::new(50.0, 50.0));

        let actuator = GroupActuator::new();
        let entities = vec![e1, e2, e3];

        let result = actuator.group_entities(&entities, &mut store);

        assert!(result.success);
        assert!(result.parent_id.is_some());
        assert_eq!(result.child_ids.len(), 3);

        // Check children have parent set
        for &child in &result.child_ids {
            let idx = child.index().0 as usize;
            assert!(store.parent_id[idx].is_some());
            assert_eq!(store.parent_id[idx], result.parent_id);
        }
    }

    #[test]
    fn test_ungroup_entities() {
        let mut store = create_test_store();
        let e1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));
        let e2 = store.spawn(Vec2::new(100.0, 0.0), Vec2::new(50.0, 50.0));

        let actuator = GroupActuator::new();
        let entities = vec![e1, e2];

        // Group first
        actuator.group_entities(&entities, &mut store);

        // Then ungroup
        let result = actuator.ungroup_entities(&entities, &mut store);

        assert!(result.success);
        assert_eq!(result.child_ids.len(), 2);

        // Check children have no parent
        for &child in &result.child_ids {
            let idx = child.index().0 as usize;
            assert!(store.parent_id[idx].is_none());
        }
    }

    #[test]
    fn test_empty_group() {
        let mut store = create_test_store();
        let actuator = GroupActuator::new();

        let result = actuator.group_entities(&[], &mut store);

        assert!(!result.success);
        assert!(result.parent_id.is_none());
    }

    #[test]
    fn test_is_group() {
        let mut store = create_test_store();
        let e1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));
        let e2 = store.spawn(Vec2::new(100.0, 0.0), Vec2::new(50.0, 50.0));

        let actuator = GroupActuator::new();
        let result = actuator.group_entities(&[e1, e2], &mut store);

        if let Some(parent_id) = result.parent_id {
            assert!(actuator.is_group(parent_id, &store));
        }
    }

    #[test]
    fn test_get_children() {
        let mut store = create_test_store();
        let e1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));
        let e2 = store.spawn(Vec2::new(100.0, 0.0), Vec2::new(50.0, 50.0));

        let actuator = GroupActuator::new();
        let result = actuator.group_entities(&[e1, e2], &mut store);

        if let Some(parent_id) = result.parent_id {
            let children = actuator.get_children(parent_id, &store);
            assert_eq!(children.len(), 2);
        }
    }
}
