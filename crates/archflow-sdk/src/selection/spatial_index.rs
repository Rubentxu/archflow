//! Hybrid Spatial Index - Grid-based spatial indexing
//!
//! This module provides a grid-based spatial indexing system for efficient
//! spatial queries.

use archflow_core::{EntityId, Rect, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// An entity with a bounding box for spatial indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialEntity {
    pub id: EntityId,
    pub bounds: Rect,
}

impl SpatialEntity {
    pub fn new(id: EntityId, bounds: Rect) -> Self {
        Self { id, bounds }
    }
}

/// Grid cell for grid-based indexing
#[derive(Debug, Clone, Default)]
struct GridCell {
    entities: HashSet<EntityId>,
}

/// Grid-based spatial index
#[derive(Debug, Clone)]
pub struct GridIndex {
    cell_size: f32,
    cells: HashMap<(i32, i32), GridCell>,
    entities: HashMap<EntityId, Rect>,
}

impl GridIndex {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
            entities: HashMap::new(),
        }
    }

    fn cell_coords(&self, point: Vec2) -> (i32, i32) {
        let x = (point.x / self.cell_size).floor() as i32;
        let y = (point.y / self.cell_size).floor() as i32;
        (x, y)
    }

    fn overlapping_cells(&self, rect: Rect) -> Vec<(i32, i32)> {
        let min_cell = self.cell_coords(rect.min);
        let max_cell = self.cell_coords(rect.max);

        let mut cells = Vec::new();
        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                cells.push((x, y));
            }
        }
        cells
    }

    pub fn insert(&mut self, id: EntityId, bounds: Rect) {
        self.entities.insert(id, bounds);
        for cell in self.overlapping_cells(bounds) {
            self.cells
                .entry(cell)
                .or_insert_with(GridCell::default)
                .entities
                .insert(id);
        }
    }

    pub fn remove(&mut self, id: &EntityId) -> Option<Rect> {
        if let Some(bounds) = self.entities.remove(id) {
            for cell in self.overlapping_cells(bounds) {
                if let Some(grid_cell) = self.cells.get_mut(&cell) {
                    grid_cell.entities.remove(id);
                }
            }
            Some(bounds)
        } else {
            None
        }
    }

    pub fn update(&mut self, id: EntityId, new_bounds: Rect) {
        self.remove(&id);
        self.insert(id, new_bounds);
    }

    pub fn query(&self, rect: Rect) -> Vec<EntityId> {
        let mut result = HashSet::new();
        for cell in self.overlapping_cells(rect) {
            if let Some(grid_cell) = self.cells.get(&cell) {
                result.extend(&grid_cell.entities);
            }
        }
        result
            .into_iter()
            .filter(|id| {
                if let Some(bounds) = self.entities.get(id) {
                    // Simple intersection check
                    !(rect.max.x < bounds.min.x
                        || rect.min.x > bounds.max.x
                        || rect.max.y < bounds.min.y
                        || rect.min.y > bounds.max.y)
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn all_ids(&self) -> Vec<EntityId> {
        self.entities.keys().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn clear(&mut self) {
        self.entities.clear();
        self.cells.clear();
    }
}

/// Hybrid spatial index (currently grid-only for simplicity)
#[derive(Debug, Clone)]
pub struct HybridSpatialIndex {
    grid: GridIndex,
}

impl HybridSpatialIndex {
    pub fn new(cell_size: f32) -> Self {
        Self {
            grid: GridIndex::new(cell_size),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(100.0)
    }

    pub fn bulk_load(&mut self, entities: &[(EntityId, Rect)]) {
        self.clear();
        for (id, bounds) in entities {
            self.insert(*id, *bounds);
        }
    }

    pub fn insert(&mut self, id: EntityId, bounds: Rect) {
        self.grid.insert(id, bounds);
    }

    pub fn remove(&mut self, id: &EntityId) -> Option<Rect> {
        self.grid.remove(id)
    }

    pub fn update(&mut self, id: EntityId, new_bounds: Rect) {
        self.grid.update(id, new_bounds);
    }

    pub fn query(&self, rect: Rect) -> Vec<EntityId> {
        self.grid.query(rect)
    }

    pub fn query_containing(&self, rect: Rect) -> Vec<EntityId> {
        self.query(rect)
            .into_iter()
            .filter(|id| {
                if let Some(bounds) = self.grid.entities.get(id) {
                    bounds.min.x <= rect.min.x
                        && bounds.min.y <= rect.min.y
                        && bounds.max.x >= rect.max.x
                        && bounds.max.y >= rect.max.y
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn all_ids(&self) -> Vec<EntityId> {
        self.grid.all_ids()
    }

    pub fn len(&self) -> usize {
        self.grid.len()
    }

    pub fn is_empty(&self) -> bool {
        self.grid.is_empty()
    }

    pub fn clear(&mut self) {
        self.grid.clear();
    }

    pub fn mode(&self) -> &'static str {
        "Grid"
    }
}

impl Default for HybridSpatialIndex {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Selection set with optimized bulk operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SelectionSet {
    inner: HashSet<EntityId>,
}

impl SelectionSet {
    pub fn new() -> Self {
        Self {
            inner: HashSet::new(),
        }
    }

    pub fn with_entities(entities: &[EntityId]) -> Self {
        Self {
            inner: entities.iter().cloned().collect(),
        }
    }

    pub fn add_all(&mut self, entities: &[EntityId]) {
        self.inner.extend(entities.iter().cloned());
    }

    pub fn remove_all(&mut self, entities: &[EntityId]) {
        for entity in entities {
            self.inner.remove(entity);
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn contains(&self, id: &EntityId) -> bool {
        self.inner.contains(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &EntityId> {
        self.inner.iter()
    }

    pub fn to_vec(&self) -> Vec<EntityId> {
        self.inner.iter().cloned().collect()
    }

    pub fn toggle(&mut self, id: &EntityId) -> bool {
        if self.inner.contains(id) {
            self.inner.remove(id);
            false
        } else {
            self.inner.insert(*id);
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entities(n: usize) -> Vec<(EntityId, Rect)> {
        (0..n)
            .map(|_| {
                let id = EntityId::new();
                let x = rand::random::<f32>() * 500.0;
                let y = rand::random::<f32>() * 500.0;
                let rect = Rect::from_min_max(Vec2::new(x, y), Vec2::new(x + 40.0, y + 40.0));
                (id, rect)
            })
            .collect()
    }

    #[test]
    fn test_query_empty_index() {
        let index = HybridSpatialIndex::with_defaults();
        let results = index.query(Rect::from_min_max(Vec2::ZERO, Vec2::new(100.0, 100.0)));
        assert!(results.is_empty());
    }

    #[test]
    fn test_insert_and_query() {
        let mut index = HybridSpatialIndex::with_defaults();

        let id1 = EntityId::new();
        let id2 = EntityId::new();

        index.insert(id1, Rect::from_min_max(Vec2::ZERO, Vec2::new(50.0, 50.0)));
        index.insert(
            id2,
            Rect::from_min_max(Vec2::new(100.0, 100.0), Vec2::new(150.0, 150.0)),
        );

        let results = index.query(Rect::from_min_max(Vec2::ZERO, Vec2::new(60.0, 60.0)));
        assert_eq!(results.len(), 1);
        assert!(results.contains(&id1));

        let results = index.query(Rect::from_min_max(Vec2::ZERO, Vec2::new(200.0, 200.0)));
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_delete_removes_from_index() {
        let mut index = HybridSpatialIndex::with_defaults();
        let entities = create_test_entities(5);
        let (id, _) = entities[0].clone();

        index.bulk_load(&entities);
        let results = index.query(entities[0].1);
        assert!(results.contains(&id));

        index.remove(&id);
        let results = index.query(entities[0].1);
        assert!(!results.contains(&id));
    }

    #[test]
    fn test_update_modifies_bounds() {
        let mut index = HybridSpatialIndex::with_defaults();

        let id = EntityId::new();
        let original_bounds = Rect::from_min_max(Vec2::ZERO, Vec2::new(50.0, 50.0));
        let new_bounds = Rect::from_min_max(Vec2::new(100.0, 100.0), Vec2::new(200.0, 200.0));

        index.insert(id, original_bounds);
        let results = index.query(original_bounds);
        assert!(results.contains(&id));

        index.update(id, new_bounds);
        let results = index.query(original_bounds);
        assert!(!results.contains(&id));
        let results = index.query(new_bounds);
        assert!(results.contains(&id));
    }

    #[test]
    fn test_selection_set_add_all() {
        let mut set = SelectionSet::new();
        let ids: Vec<EntityId> = (0..5).map(|_| EntityId::new()).collect();
        set.add_all(&ids);
        assert_eq!(set.len(), 5);
    }

    #[test]
    fn test_selection_set_remove_all() {
        let ids: Vec<EntityId> = (0..5).map(|_| EntityId::new()).collect();
        let mut set = SelectionSet::with_entities(&ids);
        let to_remove: Vec<EntityId> = ids[..2].to_vec();
        set.remove_all(&to_remove);
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn test_selection_set_clear() {
        let mut set =
            SelectionSet::with_entities(&(0..1000).map(|_| EntityId::new()).collect::<Vec<_>>());
        assert_eq!(set.len(), 1000);
        set.clear();
        assert!(set.is_empty());
    }

    #[test]
    fn test_selection_set_toggle() {
        let mut set = SelectionSet::new();
        let id = EntityId::new();
        assert!(!set.contains(&id));
        set.toggle(&id);
        assert!(set.contains(&id));
        set.toggle(&id);
        assert!(!set.contains(&id));
    }
}
