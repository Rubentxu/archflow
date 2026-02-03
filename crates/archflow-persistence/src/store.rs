// ═══════════════════════════════════════════════════════════════════════════════
// Entity Mapper - Convert between EntityStore and StoreSnapshot
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]

use archflow_core::EntityId;
use archflow_engine::EntityStore;
use std::vec::Vec;

use crate::{
    ArchitectureData, EntityData, PersistenceError, PersistenceResult, PropValue, StoreSnapshot,
    TextData,
};

/// Mapper for converting EntityStore to/from StoreSnapshot
pub struct EntityMapper;

impl EntityMapper {
    /// Convert EntityStore to StoreSnapshot
    ///
    /// This is a simplified version - in production this would properly
    /// extract all data from the SoA layout.
    pub fn from_store(store: &EntityStore) -> PersistenceResult<StoreSnapshot> {
        let entity_count = store.alive_count() as u32;
        let mut entities = Vec::new();

        // Iterate through alive entities
        // Note: This is a simplified implementation
        // In production, we'd properly iterate through the store
        for idx in 0..store.alive_count() {
            let entity_id = EntityId::new(idx as u32);

            entities.push(EntityData {
                id: entity_id,
                parent_id: store.parent_id.get(idx).copied().flatten(),
                transform: store
                    .transforms
                    .get(idx)
                    .copied()
                    .unwrap_or([0.0, 0.0, 100.0, 60.0]),
                world_transform: store
                    .world_transform
                    .get(idx)
                    .copied()
                    .unwrap_or([0.0, 0.0, 100.0, 60.0]),
                metadata: store.metadata.get(idx).copied().unwrap_or(0),
                color: store.colors.get(idx).copied().unwrap_or(0xFFCCDDEE),
                texture_index: store.texture_index.get(idx).copied().unwrap_or(0),
                color_tint: store
                    .color_tints
                    .get(idx)
                    .copied()
                    .unwrap_or([1.0, 1.0, 1.0, 1.0]),
                text: None,      // TODO: Extract text data
                arch_data: None, // TODO: Extract arch data
                props: std::collections::BTreeMap::new(),
            });
        }

        Ok(StoreSnapshot {
            version: 1,
            entity_count,
            entities,
        })
    }

    /// Apply StoreSnapshot to EntityStore
    ///
    /// This is a simplified version - in production this would properly
    /// populate all the SoA arrays.
    pub fn to_store(store: &mut EntityStore, snapshot: StoreSnapshot) -> PersistenceResult<()> {
        // Clear existing entities
        // Note: This is simplified - production would properly clear

        // Add entities from snapshot
        for entity_data in snapshot.entities {
            let idx = entity_data.id.index().0 as usize;

            if idx < store.transforms.len() {
                store.transforms[idx] = entity_data.transform;
                store.world_transform[idx] = entity_data.world_transform;
                store.metadata[idx] = entity_data.metadata;
                store.colors[idx] = entity_data.color;
                store.texture_index[idx] = entity_data.texture_index;
                store.color_tints[idx] = entity_data.color_tint;
                store.parent_id[idx] = entity_data.parent_id;

                // Mark dirty
                store.dirty_transform.insert(idx);
                store.dirty_render.insert(idx);
            }
        }

        store.set_alive_count(snapshot.entity_count as usize);
        store.dirty_z_order = true;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_store_snapshot() {
        let store = EntityStore::new();
        let snapshot = EntityMapper::from_store(&store).unwrap();

        assert_eq!(snapshot.entity_count, 0);
        assert!(snapshot.entities.is_empty());
    }
}
