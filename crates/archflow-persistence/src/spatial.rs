// ═══════════════════════════════════════════════════════════════════════════════
// Spatial Hash Builder - Rebuild SpatialIndex from loaded document
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]

use archflow_core::{EntityId, Vec2};
use std::collections::BTreeMap;
use std::vec::Vec;

use crate::{PersistenceResult, SpatialIndexData, StoreSnapshot};

/// Builder for creating SpatialHash data from document entities
pub struct SpatialHashBuilder {
    /// Cell size for the spatial index
    cell_size: f32,
}

impl SpatialHashBuilder {
    /// Create a new builder with default cell size (64px)
    #[must_use]
    pub const fn new() -> Self {
        Self { cell_size: 64.0 }
    }

    /// Create a new builder with custom cell size
    #[must_use]
    pub const fn with_cell_size(cell_size: f32) -> Self {
        Self { cell_size }
    }

    /// Build spatial index data from store snapshot
    pub fn build(&self, store: &StoreSnapshot) -> PersistenceResult<SpatialIndexData> {
        // Find the bounds of all entities to determine grid size
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for entity in &store.entities {
            let transform = entity.transform;
            let x = transform[0];
            let y = transform[1];
            let w = transform[2];
            let h = transform[3];

            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x + w);
            max_y = max_y.max(y + h);
        }

        // If no entities, return empty index
        if store.entities.is_empty() {
            return Ok(SpatialIndexData {
                cell_size: self.cell_size,
                cell_count: 0,
                cells: Vec::new(),
            });
        }

        // Calculate grid dimensions
        let grid_width = ((max_x - min_x) / self.cell_size).ceil() as usize;
        let grid_height = ((max_y - min_y) / self.cell_size).ceil() as usize;
        let cell_count = grid_width * grid_height;

        // Initialize cells
        let mut cells = vec![Vec::new(); cell_count];

        // Insert entities into cells
        for entity in &store.entities {
            if !self.is_visible(entity.metadata) {
                continue;
            }

            let transform = entity.transform;
            let x = transform[0];
            let y = transform[1];
            let w = transform[2];
            let h = transform[3];

            // Get cells covered by this entity
            let min_cell_x = ((x - min_x) / self.cell_size).floor() as usize;
            let min_cell_y = ((y - min_y) / self.cell_size).floor() as usize;
            let max_cell_x = (((x + w) - min_x) / self.cell_size).floor() as usize;
            let max_cell_y = (((y + h) - min_y) / self.cell_size).floor() as usize;

            // Clamp to grid bounds
            let min_cell_x = min_cell_x.min(grid_width - 1);
            let min_cell_y = min_cell_y.min(grid_height - 1);
            let max_cell_x = max_cell_x.min(grid_width - 1);
            let max_cell_y = max_cell_y.min(grid_height - 1);

            // Add to all covered cells
            for cy in min_cell_y..=max_cell_y {
                for cx in min_cell_x..=max_cell_x {
                    let cell_idx = cy * grid_width + cx;
                    if cell_idx < cells.len() {
                        cells[cell_idx].push(entity.id);
                    }
                }
            }
        }

        Ok(SpatialIndexData {
            cell_size: self.cell_size,
            cell_count,
            cells,
        })
    }

    /// Build engine spatial hash (64px cells for rendering)
    pub fn build_engine_hash(&self, store: &StoreSnapshot) -> PersistenceResult<SpatialIndexData> {
        Self::with_cell_size(64.0).build(store)
    }

    /// Build logic spatial hash (40px cells for collision)
    pub fn build_logic_hash(&self, store: &StoreSnapshot) -> PersistenceResult<SpatialIndexData> {
        Self::with_cell_size(40.0).build(store)
    }

    /// Check if entity is visible based on metadata
    fn is_visible(&self, metadata: u32) -> bool {
        // Visibility is in bit 8: [shape:4 | layer:4 | visibility:1 | selected:1 | locked:1 | ...]
        (metadata & (1 << 8)) != 0
    }
}

impl Default for SpatialHashBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EntityData, PropValue};

    #[test]
    fn test_builder_new() {
        let builder = SpatialHashBuilder::new();
        assert_eq!(builder.cell_size, 64.0);
    }

    #[test]
    fn test_builder_with_cell_size() {
        let builder = SpatialHashBuilder::with_cell_size(40.0);
        assert_eq!(builder.cell_size, 40.0);
    }

    #[test]
    fn test_build_empty_store() {
        let builder = SpatialHashBuilder::new();
        let store = StoreSnapshot {
            version: 1,
            entity_count: 0,
            entities: Vec::new(),
        };

        let result = builder.build(&store).unwrap();
        assert_eq!(result.cell_count, 0);
        assert!(result.cells.is_empty());
    }

    #[test]
    fn test_build_single_entity() {
        let builder = SpatialHashBuilder::new();
        let mut store = StoreSnapshot {
            version: 1,
            entity_count: 1,
            entities: Vec::new(),
        };

        store.entities.push(EntityData {
            id: EntityId::new(1),
            parent_id: None,
            transform: [100.0, 200.0, 50.0, 30.0],
            world_transform: [100.0, 200.0, 50.0, 30.0],
            metadata: 0x0101, // Visible
            color: 0xFF000000,
            texture_index: 0,
            color_tint: [1.0, 1.0, 1.0, 1.0],
            text: None,
            arch_data: None,
            props: BTreeMap::new(),
        });

        let result = builder.build(&store).unwrap();
        assert_eq!(result.cell_count, 1);
        assert_eq!(result.cells[0].len(), 1);
        assert_eq!(result.cells[0][0], EntityId::new(1));
    }

    #[test]
    fn test_build_multiple_entities() {
        let builder = SpatialHashBuilder::new();
        let mut store = StoreSnapshot {
            version: 1,
            entity_count: 3,
            entities: Vec::new(),
        };

        // Three entities at different positions
        for i in 0..3 {
            store.entities.push(EntityData {
                id: EntityId::new(i),
                parent_id: None,
                transform: [i as f32 * 100.0, 0.0, 50.0, 30.0],
                world_transform: [i as f32 * 100.0, 0.0, 50.0, 30.0],
                metadata: 0x0101,
                color: 0xFF000000,
                texture_index: 0,
                color_tint: [1.0, 1.0, 1.0, 1.0],
                text: None,
                arch_data: None,
                props: BTreeMap::new(),
            });
        }

        let result = builder.build(&store).unwrap();
        // Entities can span multiple cells (50px wide, cells are 64px)
        // Entity at 0 covers cell 0
        // Entity at 100 covers cells 1 and 2 (100+50=150 crosses cell boundary at 128)
        // Entity at 200 covers cell 3
        let total_entities: usize = result.cells.iter().map(|c| c.len()).sum();
        assert_eq!(total_entities, 4); // 4 cell entries due to multi-cell coverage
    }

    #[test]
    fn test_invisible_entities_not_indexed() {
        let builder = SpatialHashBuilder::new();
        let mut store = StoreSnapshot {
            version: 1,
            entity_count: 2,
            entities: Vec::new(),
        };

        // Visible entity
        store.entities.push(EntityData {
            id: EntityId::new(1),
            parent_id: None,
            transform: [100.0, 200.0, 50.0, 30.0],
            world_transform: [100.0, 200.0, 50.0, 30.0],
            metadata: 0x0101, // Visible (bit 8 set)
            color: 0xFF000000,
            texture_index: 0,
            color_tint: [1.0, 1.0, 1.0, 1.0],
            text: None,
            arch_data: None,
            props: BTreeMap::new(),
        });

        // Invisible entity
        store.entities.push(EntityData {
            id: EntityId::new(2),
            parent_id: None,
            transform: [150.0, 200.0, 50.0, 30.0],
            world_transform: [150.0, 200.0, 50.0, 30.0],
            metadata: 0x0001, // Not visible (bit 8 not set)
            color: 0xFF000000,
            texture_index: 0,
            color_tint: [1.0, 1.0, 1.0, 1.0],
            text: None,
            arch_data: None,
            props: BTreeMap::new(),
        });

        let result = builder.build(&store).unwrap();
        let total_entities: usize = result.cells.iter().map(|c| c.len()).sum();
        // Only the visible entity should be indexed
        assert_eq!(total_entities, 1);
    }

    #[test]
    fn test_engine_hash_cell_size() {
        let builder = SpatialHashBuilder::new();
        let result = builder
            .build_engine_hash(&StoreSnapshot {
                version: 1,
                entity_count: 0,
                entities: Vec::new(),
            })
            .unwrap();
        assert_eq!(result.cell_size, 64.0);
    }

    #[test]
    fn test_logic_hash_cell_size() {
        let builder = SpatialHashBuilder::new();
        let result = builder
            .build_logic_hash(&StoreSnapshot {
                version: 1,
                entity_count: 0,
                entities: Vec::new(),
            })
            .unwrap();
        assert_eq!(result.cell_size, 40.0);
    }
}
