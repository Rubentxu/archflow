//! Synchronization system for Records to ECS
//!
//! This module provides systems for synchronizing Record changes to ECS entities
//! using ChangeSet for O(C) performance optimization.

use archflow_records::{Record, RecordId, RecordStore};
use bevy_ecs::prelude::*;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::components::{Dirty, RecordRef, Transform};

/// System that synchronizes Record changes to ECS entities.
///
/// This system processes changes from RecordStore's ChangeSet and:
/// - Spawns new entities for created records
/// - Updates existing entities for updated records
/// - Despawns entities for deleted records
///
/// # Performance
///
/// The system achieves O(C) complexity where C is the number of changed records,
/// not O(N) where N is the total number of records.
///
/// # Examples
///
/// ```ignore
/// use archflow_ecs_hybrid::sync_records_to_ecs_system;
/// use bevy_ecs::prelude::*;
/// use archflow_records::RecordStore;
///
/// #[derive(Record)]
/// struct MyRecord {
///     // fields
/// }
///
/// fn main() {
///     let mut world = World::new();
///     world.insert_resource(RecordStore::<MyRecord>::new());
///
///     let mut schedule = Schedule::default();
///     schedule.add_systems(sync_records_to_ecs_system::<MyRecord>);
///
///     schedule.run(&mut world);
/// }
/// ```
pub fn sync_records_to_ecs_system<R: Record + Clone>(
    mut record_store: ResMut<RecordStore<R>>,
    mut query: Query<(Entity, &mut RecordRef, &mut Transform)>,
    mut commands: Commands,
) {
    let changeset = record_store.drain_changes();

    // O(1) early exit if no changes
    if changeset.is_empty() {
        return;
    }

    // Build entity lookup from query results
    let mut entity_map: HashMap<RecordId, Entity> = HashMap::new();
    for (entity, record_ref, _) in query.iter() {
        entity_map.insert(record_ref.record_id.clone(), entity);
    }

    // Process created records - spawn new entities
    for (id, record) in record_store.iter_created(&changeset) {
        commands.spawn((
            RecordRef::new(id.clone()),
            Transform::from_record(record),
            Dirty::created(),
        ));
    }

    // Process updated records - update existing entities
    for (id, record) in record_store.iter_updated(&changeset) {
        if let Some(&entity) = entity_map.get(id) {
            if let Ok(mut query_item) = query.get_mut(entity) {
                let (_, ref mut record_ref, mut transform) = query_item;
                *transform = Transform::from_record(record);
                record_ref.clear_dirty();
            }
        }
    }

    // Process deleted records - despawn entities
    for id in changeset.deleted_ids() {
        if let Some(&entity) = entity_map.get(id) {
            commands.entity(entity).despawn();
        }
    }
}

/// Extension trait for converting RecordId to ECS Entity.
///
/// This provides a convenient way to create Entity references
/// from RecordId for ECS operations.
pub trait RecordIdEntityExt {
    /// Converts RecordId to Entity.
    ///
    /// This assumes RecordId was created from a u64 Entity index.
    /// Returns Entity with same index.
    fn to_entity(&self) -> Entity;
}

impl RecordIdEntityExt for RecordId {
    fn to_entity(&self) -> Entity {
        Entity::from_bits(self.as_u64())
    }
}

/// Extension trait for extracting u64 from RecordId.
pub trait RecordIdU64Ext {
    fn as_u64(&self) -> u64;
}

impl RecordIdU64Ext for RecordId {
    fn as_u64(&self) -> u64 {
        let s = self.as_str();
        if let Some(hex_str) = s.strip_prefix("id_") {
            u64::from_str_radix(hex_str.trim_start_matches('0'), 10).unwrap_or_else(|_| {
                let mut hash = std::collections::hash_map::DefaultHasher::new();
                s.hash(&mut hash);
                hash.finish()
            })
        } else {
            let mut hash = std::collections::hash_map::DefaultHasher::new();
            s.hash(&mut hash);
            hash.finish()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_records::{Bounds, Record};
    use bevy_ecs::{schedule::Schedule, world::World};
    use std::str::FromStr;

    #[derive(Debug, Clone, PartialEq)]
    struct MockRecord {
        id: RecordId,
        bounds: Option<Bounds>,
    }

    impl Record for MockRecord {
        fn id(&self) -> &RecordId {
            &self.id
        }

        fn type_name(&self) -> &'static str {
            "MockRecord"
        }

        fn bounds(&self) -> Option<Bounds> {
            self.bounds.clone()
        }
    }

    fn create_test_record(id_str: &str) -> MockRecord {
        let id = RecordId::from_str(id_str).unwrap();
        let bounds = Bounds::new(100.0, 100.0, 200.0, 200.0);
        MockRecord {
            id,
            bounds: Some(bounds),
        }
    }

    #[test]
    fn test_empty_changeset_no_work() {
        let mut store: RecordStore<MockRecord> = RecordStore::new();
        let changeset = store.drain_changes();
        assert_eq!(changeset.change_count(), 0);
        assert!(changeset.is_empty());
    }

    #[test]
    fn test_created_record_in_changeset() {
        let mut store: RecordStore<MockRecord> = RecordStore::new();
        let _id = RecordId::from_str("spawn_test_001").unwrap();
        let record = create_test_record("spawn_test_001");
        store.put(record);

        let changeset = store.drain_changes();
        assert_eq!(changeset.created_count(), 1);
        assert_eq!(changeset.change_count(), 1);
        assert!(!changeset.is_empty());
    }

    #[test]
    fn test_updated_record_in_changeset() {
        let mut store: RecordStore<MockRecord> = RecordStore::new();
        let _id = RecordId::from_str("update_test_001").unwrap();
        let record = create_test_record("update_test_001");
        store.put(record);
        store.drain_changes();

        // Modify record
        let record = create_test_record("update_test_001");
        store.put(record);

        let changeset = store.drain_changes();
        assert_eq!(changeset.updated_count(), 1);
        assert_eq!(changeset.change_count(), 1);
        assert!(!changeset.is_empty());
    }

    #[test]
    fn test_deleted_record_in_changeset() {
        let mut store: RecordStore<MockRecord> = RecordStore::new();
        let id = RecordId::from_str("delete_test_001").unwrap();
        let record = create_test_record("delete_test_001");
        store.put(record);
        store.drain_changes();

        store.remove(&id);

        let changeset = store.drain_changes();
        assert_eq!(changeset.deleted_count(), 1);
        assert_eq!(changeset.change_count(), 1);
        assert!(!changeset.is_empty());
    }

    #[test]
    fn test_record_id_entity_ext() {
        let id = RecordId::from_u64(12345);
        let entity = id.to_entity();
        assert_eq!(entity.to_bits(), 12345);
    }

    #[test]
    fn test_record_id_u64_ext() {
        let id1 = RecordId::from_u64(12345);
        let u64_val = id1.as_u64();
        assert_eq!(u64_val, 12345);
    }

    #[test]
    fn test_sync_system_with_created_records() {
        let mut world = World::new();
        let store: RecordStore<MockRecord> = RecordStore::new();

        // Insert resource
        world.insert_resource(store);

        // Create records
        let record1 = create_test_record("sync_create_001");
        let record2 = create_test_record("sync_create_002");
        world.resource_mut::<RecordStore<MockRecord>>().put(record1);
        world.resource_mut::<RecordStore<MockRecord>>().put(record2);

        // Create schedule and add system
        let mut schedule = Schedule::default();
        schedule.add_systems(sync_records_to_ecs_system::<MockRecord>);

        // Run sync system
        schedule.run(&mut world);

        // Verify entities were spawned
        let entities = world
            .query::<(Entity, &RecordRef, &Transform)>()
            .iter(&world)
            .count();
        assert_eq!(entities, 2);
    }

    #[test]
    fn test_sync_system_with_updated_records() {
        let mut world = World::new();
        let store: RecordStore<MockRecord> = RecordStore::new();

        world.insert_resource(store);

        // Create and sync initial record
        let record1 = create_test_record("sync_update_001");
        world.resource_mut::<RecordStore<MockRecord>>().put(record1);

        let mut schedule = Schedule::default();
        schedule.add_systems(sync_records_to_ecs_system::<MockRecord>);
        schedule.run(&mut world);

        // Drain changes
        world
            .resource_mut::<RecordStore<MockRecord>>()
            .drain_changes();

        // Update record with new bounds
        let updated_id = RecordId::from_str("sync_update_001").unwrap();
        let updated_record = MockRecord {
            id: updated_id.clone(),
            bounds: Some(Bounds::new(300.0, 300.0, 400.0, 400.0)),
        };
        world
            .resource_mut::<RecordStore<MockRecord>>()
            .put(updated_record);

        // Run sync system
        schedule.run(&mut world);

        // Verify entity was updated
        let found_updated = world.query::<(&RecordRef, &Transform)>().iter(&world).any(
            |(record_ref, transform)| {
                record_ref.record_id.as_str() == "sync_update_001"
                    && transform.position.x == 350.0
                    && transform.position.y == 350.0
            },
        );
        assert!(found_updated);
    }

    #[test]
    fn test_sync_system_with_deleted_records() {
        let mut world = World::new();
        let store: RecordStore<MockRecord> = RecordStore::new();

        world.insert_resource(store);

        // Create and sync initial records
        let record1 = create_test_record("sync_delete_001");
        let record2 = create_test_record("sync_delete_002");
        world.resource_mut::<RecordStore<MockRecord>>().put(record1);
        world.resource_mut::<RecordStore<MockRecord>>().put(record2);

        let mut schedule = Schedule::default();
        schedule.add_systems(sync_records_to_ecs_system::<MockRecord>);
        schedule.run(&mut world);

        // Drain changes
        world
            .resource_mut::<RecordStore<MockRecord>>()
            .drain_changes();

        // Get entity ID before deletion
        let entity_to_delete = world
            .query::<(Entity, &RecordRef)>()
            .iter(&world)
            .find(|(_, record_ref)| record_ref.record_id.as_str() == "sync_delete_001")
            .map(|(entity, _)| entity);

        assert!(entity_to_delete.is_some());

        // Delete record
        let delete_id = RecordId::from_str("sync_delete_001").unwrap();
        world
            .resource_mut::<RecordStore<MockRecord>>()
            .remove(&delete_id);

        // Run sync system
        schedule.run(&mut world);

        // Verify entity was despawned
        let records: Vec<_> = world
            .query::<(&RecordRef,)>()
            .iter(&world)
            .map(|(record_ref,)| record_ref.record_id.as_str())
            .collect();
        assert!(!records.contains(&"sync_delete_001"));
        assert!(records.contains(&"sync_delete_002"));
    }
}
