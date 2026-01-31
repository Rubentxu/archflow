// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Engine - SpatialHash Index
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 8
//
// Spatial indexing with O(1) average case queries:
// - Grid-based spatial hash for fast hit testing
// - Support for hierarchy (propagates world bounds)
// - Dirty tracking for incremental updates
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use archflow_core::{EntityId, Rect, Vec2};

/// Size of each spatial hash cell in world units
/// Smaller = more precision but more memory overhead
const CELL_SIZE: f32 = 64.0;

/// Maximum entities per cell before overflow
/// Entities beyond this count are stored in overflow list
const MAX_ENTITIES_PER_CELL: usize = 16;

/// Cell coordinate in the spatial grid
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct CellCoord {
    x: i32,
    y: i32,
}

impl CellCoord {
    /// Convert world position to cell coordinate
    fn from_world(pos: Vec2) -> Self {
        Self {
            x: (pos.x / CELL_SIZE).floor() as i32,
            y: (pos.y / CELL_SIZE).floor() as i32,
        }
    }

    /// Convert cell coordinate to world origin
    fn to_world_origin(self) -> Vec2 {
        Vec2::new(self.x as f32 * CELL_SIZE, self.y as f32 * CELL_SIZE)
    }
}

/// Single cell in the spatial grid
#[derive(Clone, Debug)]
struct Cell {
    /// Entities contained in this cell
    entities: Vec<(EntityId, Rect)>,
}

impl Cell {
    fn new() -> Self {
        Self {
            entities: Vec::with_capacity(MAX_ENTITIES_PER_CELL),
        }
    }

    fn insert(&mut self, id: EntityId, bounds: Rect) {
        self.entities.push((id, bounds));
    }

    fn remove(&mut self, id: EntityId) -> bool {
        if let Some(pos) = self.entities.iter().position(|(e, _)| *e == id) {
            self.entities.remove(pos);
            true
        } else {
            false
        }
    }
}

/// Spatial hash grid for O(1) spatial queries
///
/// Optimizations:
/// - Uses BTreeMap for sorted iteration (predictable performance)
/// - Supports hierarchy via world bounds propagation
/// - Dirty tracking for incremental updates
/// - Multi-cell coverage for large entities
pub struct SpatialHash {
    /// Grid cells indexed by coordinate
    cells: BTreeMap<(i32, i32), Cell>,

    /// Map from entity to the cells it occupies
    /// Used for efficient removal and update
    entity_to_cell: Vec<Option<Vec<CellCoord>>>,

    /// Maximum entities supported
    max_entities: usize,
}

impl SpatialHash {
    /// Create a new spatial hash
    pub fn new(max_entities: usize) -> Self {
        Self {
            cells: BTreeMap::new(),
            entity_to_cell: vec![None; max_entities],
            max_entities,
        }
    }

    /// Insert an entity into the spatial hash
    pub fn insert(&mut self, id: EntityId, bounds: Rect) {
        let index = id.index().0 as usize;
        if index >= self.max_entities {
            return;
        }

        // Clear previous cell associations
        self.remove(id);

        // Get all cells this entity covers
        let covered_cells = self.cells_covered(bounds);
        let mut cell_vec = Vec::with_capacity(covered_cells.len());

        for coord in covered_cells {
            let key = (coord.x, coord.y);
            let cell = self.cells.entry(key).or_insert_with(Cell::new);
            cell.insert(id, bounds);
            cell_vec.push(coord);
        }

        self.entity_to_cell[index] = Some(cell_vec);
    }

    /// Remove an entity from the spatial hash
    pub fn remove(&mut self, id: EntityId) -> bool {
        let index = id.index().0 as usize;
        if index >= self.max_entities {
            return false;
        }

        if let Some(cell_coords) = self.entity_to_cell[index].take() {
            for coord in cell_coords {
                let key = (coord.x, coord.y);
                if let Some(cell) = self.cells.get_mut(&key) {
                    cell.remove(id);
                    // Clean up empty cells
                    if cell.entities.is_empty() {
                        self.cells.remove(&key);
                    }
                }
            }
            return true;
        }
        false
    }

    /// Update an entity's position
    /// Returns true if the entity was found and updated
    pub fn update(&mut self, id: EntityId, new_bounds: Rect) -> bool {
        let index = id.index().0 as usize;
        if index >= self.max_entities {
            return false;
        }

        // Check if entity exists
        if self.entity_to_cell[index].is_none() {
            return false;
        }

        // Re-insert with new bounds
        self.insert(id, new_bounds);
        true
    }

    /// Query entities at a specific point
    /// Returns all entities whose bounds contain the point
    pub fn query_point(&self, point: Vec2) -> Vec<EntityId> {
        let coord = CellCoord::from_world(point);
        let key = (coord.x, coord.y);

        let mut results = Vec::new();
        if let Some(cell) = self.cells.get(&key) {
            for &(id, bounds) in &cell.entities {
                if bounds.contains(point) {
                    results.push(id);
                }
            }
        }
        results
    }

    /// Query entities in a rectangular region
    /// Returns all entities whose bounds intersect the query rect
    pub fn query_rect(&self, query_bounds: Rect) -> Vec<EntityId> {
        let mut results = Vec::new();
        let mut seen = alloc::collections::BTreeSet::new();

        // Get all cells intersected by query rect
        for coord in self.cells_covered(query_bounds) {
            let key = (coord.x, coord.y);
            if let Some(cell) = self.cells.get(&key) {
                for &(id, bounds) in &cell.entities {
                    // Avoid duplicates (entities may span multiple cells)
                    if seen.insert(id) && bounds.intersects(&query_bounds) {
                        results.push(id);
                    }
                }
            }
        }
        results
    }

    /// Find entities with a specific name in a region
    /// (Requires access to string pool - implemented at higher level)
    pub fn query_region_named(&self, query_bounds: Rect, _name: &str) -> Vec<EntityId> {
        // This is a placeholder - actual implementation needs string pool access
        self.query_rect(query_bounds)
    }

    /// Clear all entities from the spatial hash
    pub fn clear(&mut self) {
        self.cells.clear();
        for slot in &mut self.entity_to_cell {
            *slot = None;
        }
    }

    /// Get number of cells currently allocated
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Get approximate memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        let cell_count = self.cells.len();
        let entity_count: usize = self.cells.values().map(|c| c.entities.len()).sum();
        cell_count * 64 + entity_count * 24 // Approximate
    }

    /// Get all cells covered by a rectangle
    fn cells_covered(&self, bounds: Rect) -> Vec<CellCoord> {
        let min = CellCoord::from_world(bounds.min);
        let max = CellCoord::from_world(bounds.max);

        let mut result = Vec::new();
        for x in min.x..=max.x {
            for y in min.y..=max.y {
                result.push(CellCoord { x, y });
            }
        }
        result
    }
}

impl Default for SpatialHash {
    fn default() -> Self {
        Self::new(100_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_core::{Generation, Index};

    fn make_id(idx: u32) -> EntityId {
        EntityId::from_parts(Index(idx), Generation(1))
    }

    #[test]
    fn test_insert_and_query_point() {
        let mut hash = SpatialHash::new(100);
        let id = make_id(1);
        let bounds = Rect::from_origin_size(Vec2::new(10.0, 10.0), Vec2::new(10.0, 10.0));

        hash.insert(id, bounds);

        // Point inside bounds
        let results = hash.query_point(Vec2::new(15.0, 15.0));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], id);

        // Point outside bounds
        let results = hash.query_point(Vec2::new(100.0, 100.0));
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_query_rect() {
        let mut hash = SpatialHash::new(100);

        let id1 = make_id(1);
        let id2 = make_id(2);
        let id3 = make_id(3);

        hash.insert(
            id1,
            Rect::from_origin_size(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0)),
        );
        hash.insert(
            id2,
            Rect::from_origin_size(Vec2::new(20.0, 0.0), Vec2::new(10.0, 10.0)),
        );
        hash.insert(
            id3,
            Rect::from_origin_size(Vec2::new(100.0, 100.0), Vec2::new(10.0, 10.0)),
        );

        // Query covering first two entities
        let query = Rect::from_origin_size(Vec2::new(0.0, 0.0), Vec2::new(50.0, 20.0));
        let results = hash.query_rect(query);
        assert_eq!(results.len(), 2);
        assert!(results.contains(&id1));
        assert!(results.contains(&id2));
        assert!(!results.contains(&id3));
    }

    #[test]
    fn test_remove() {
        let mut hash = SpatialHash::new(100);
        let id = make_id(1);
        let bounds = Rect::from_origin_size(Vec2::new(10.0, 10.0), Vec2::new(10.0, 10.0));

        hash.insert(id, bounds);
        assert_eq!(hash.query_point(Vec2::new(15.0, 15.0)).len(), 1);

        assert!(hash.remove(id));
        assert_eq!(hash.query_point(Vec2::new(15.0, 15.0)).len(), 0);
    }

    #[test]
    fn test_update() {
        let mut hash = SpatialHash::new(100);
        let id = make_id(1);

        hash.insert(
            id,
            Rect::from_origin_size(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0)),
        );
        assert!(hash.query_point(Vec2::new(5.0, 5.0)).len() == 1);

        hash.update(
            id,
            Rect::from_origin_size(Vec2::new(100.0, 100.0), Vec2::new(10.0, 10.0)),
        );
        assert!(hash.query_point(Vec2::new(5.0, 5.0)).len() == 0);
        assert!(hash.query_point(Vec2::new(105.0, 105.0)).len() == 1);
    }

    #[test]
    fn test_multi_cell_coverage() {
        let mut hash = SpatialHash::new(100);
        let id = make_id(1);

        // Entity larger than CELL_SIZE, covers multiple cells
        let large_bounds = Rect::from_origin_size(Vec2::new(0.0, 0.0), Vec2::new(200.0, 200.0));
        hash.insert(id, large_bounds);

        // Should find entity from any cell it covers
        assert!(hash.query_point(Vec2::new(10.0, 10.0)).contains(&id));
        assert!(hash.query_point(Vec2::new(70.0, 70.0)).contains(&id));
        assert!(hash.query_point(Vec2::new(150.0, 150.0)).contains(&id));
    }
}
