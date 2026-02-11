// ═══════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - SIMD Integration Module (HU-SIMD-005)
//
// Provides collision detection and spatial hashing utilities optimized for WASM.
// Supports 2D/3D AABB collision detection with spatial partitioning.
//
// Key Features:
// - Aabb2D/Aabb3D: Axis-aligned bounding boxes for collision detection
// - CollisionSimdDetector: Optimized collision detection with SIMD hints
// - SpatialHash: Grid-based spatial partitioning for O(1) lookups
//
// ═══════════════════════════════════════════════════════════════════════════════════════

#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

/// SIMD batch size for f32 operations (4 floats = 128 bits)
pub const SIMD_F32_BATCH: usize = 4;

/// SIMD batch size for u8 operations (16 bytes = 128 bits)
pub const SIMD_U8_BATCH: usize = 16;

// ═══════════════════════════════════════════════════════════════════════════════════════
// COLLISION DETECTION TYPES
// ═════════════════════════════════════════════════════════════════════════════════════

/// Axis-Aligned Bounding Box for 2D collision detection.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Aabb2D {
    /// Minimum x coordinate
    pub min_x: f32,
    /// Minimum y coordinate
    pub min_y: f32,
    /// Maximum x coordinate
    pub max_x: f32,
    /// Maximum y coordinate
    pub max_y: f32,
}

impl Aabb2D {
    /// Creates a new AABB from center and half-extents.
    #[inline]
    #[must_use]
    pub fn from_center_extents(cx: f32, cy: f32, half_w: f32, half_h: f32) -> Self {
        Self {
            min_x: cx - half_w,
            min_y: cy - half_h,
            max_x: cx + half_w,
            max_y: cy + half_h,
        }
    }

    /// Creates an empty AABB (no overlap).
    #[inline]
    #[must_use]
    pub fn empty() -> Self {
        Self {
            min_x: f32::MAX,
            min_y: f32::MAX,
            max_x: f32::MIN,
            max_y: f32::MIN,
        }
    }

    /// Checks if this AABB intersects another.
    #[inline]
    #[must_use]
    pub fn intersects(&self, other: &Self) -> bool {
        self.min_x <= other.max_x
            && self.max_x >= other.min_x
            && self.min_y <= other.max_y
            && self.max_y >= other.min_y
    }

    /// Returns the intersection area of two AABBs.
    #[inline]
    #[must_use]
    pub fn intersection_area(&self, other: &Self) -> f32 {
        let inter_min_x = self.min_x.max(other.min_x);
        let inter_min_y = self.min_y.max(other.min_y);
        let inter_max_x = self.max_x.min(other.max_x);
        let inter_max_y = self.max_y.min(other.max_y);

        let width = (inter_max_x - inter_min_x).max(0.0);
        let height = (inter_max_y - inter_min_y).max(0.0);

        width * height
    }
}

/// Axis-Aligned Bounding Box for 3D collision detection.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Aabb3D {
    /// Minimum x coordinate
    pub min_x: f32,
    /// Minimum y coordinate
    pub min_y: f32,
    /// Minimum z coordinate
    pub min_z: f32,
    /// Maximum x coordinate
    pub max_x: f32,
    /// Maximum y coordinate
    pub max_y: f32,
    /// Maximum z coordinate
    pub max_z: f32,
}

impl Aabb3D {
    /// Creates a new 3D AABB from center and half-extents.
    #[inline]
    #[must_use]
    pub fn from_center_extents(
        cx: f32,
        cy: f32,
        cz: f32,
        half_x: f32,
        half_y: f32,
        half_z: f32,
    ) -> Self {
        Self {
            min_x: cx - half_x,
            min_y: cy - half_y,
            min_z: cz - half_z,
            max_x: cx + half_x,
            max_y: cy + half_y,
            max_z: cz + half_z,
        }
    }

    /// Checks if this AABB intersects another.
    #[inline]
    #[must_use]
    pub fn intersects(&self, other: &Self) -> bool {
        self.min_x <= other.max_x
            && self.max_x >= other.min_x
            && self.min_y <= other.max_y
            && self.max_y >= other.min_y
            && self.min_z <= other.max_z
            && self.max_z >= other.min_z
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// COLLISION DETECTOR
// ═════════════════════════════════════════════════════════════════════════════════════

/// Collision detector for 2D AABBs.
///
/// Provides efficient collision detection with batch processing support.
#[derive(Debug, Default)]
pub struct CollisionSimdDetector {
    /// Number of collisions detected
    collisions: usize,
}

impl CollisionSimdDetector {
    /// Creates a new collision detector.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self { collisions: 0 }
    }

    /// Resets the detector state.
    #[inline]
    pub fn reset(&mut self) {
        self.collisions = 0;
    }

    /// Detects collisions between two sets of AABBs.
    ///
    /// # Parameters
    ///
    /// - `aabbs1`: First set of AABBs
    /// - `aabbs2`: Second set of AABBs (must have same length as `aabbs1`)
    ///
    /// # Returns
    ///
    /// Vector of boolean results indicating collision for each pair
    ///
    /// # Panics
    ///
    /// Panics if the arrays have different lengths.
    #[inline]
    pub fn detect_collisions(&mut self, aabbs1: &[Aabb2D], aabbs2: &[Aabb2D]) -> Vec<bool> {
        assert_eq!(
            aabbs1.len(),
            aabbs2.len(),
            "AABB arrays must have equal length"
        );

        let len = aabbs1.len();
        let mut results = Vec::with_capacity(len);
        self.collisions = 0;

        for (a, b) in aabbs1.iter().zip(aabbs2.iter()) {
            let collision = a.intersects(b);
            results.push(collision);
            if collision {
                self.collisions += 1;
            }
        }

        results
    }

    /// Returns the number of collisions detected.
    #[inline]
    #[must_use]
    pub fn collision_count(&self) -> usize {
        self.collisions
    }

    /// Returns the collision ratio.
    #[inline]
    #[must_use]
    pub fn collision_ratio(&self, total: usize) -> f32 {
        if total > 0 {
            self.collisions as f32 / total as f32
        } else {
            0.0
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// SPATIAL HASH
// ═════════════════════════════════════════════════════════════════════════════════════

/// Spatial hash grid for efficient spatial queries.
///
/// Divides space into a grid of cells, enabling O(1) lookup
/// for entities within a spatial region.
#[derive(Debug, Default)]
pub struct SpatialHash {
    /// Cell size for the grid
    cell_size: f32,
    /// Map from cell coordinates to entity IDs
    cells: BTreeMap<(i32, i32, i32), Vec<usize>>,
}

impl SpatialHash {
    /// Creates a new spatial hash with the specified cell size.
    ///
    /// # Parameters
    ///
    /// - `cell_size`: Size of each grid cell (should match entity size)
    ///
    /// # Panics
    ///
    /// Panics if `cell_size` is not positive.
    #[inline]
    #[must_use]
    pub fn new(cell_size: f32) -> Self {
        assert!(cell_size > 0.0, "Cell size must be positive");
        Self {
            cell_size,
            cells: BTreeMap::new(),
        }
    }

    /// Inserts an entity at the specified position.
    #[inline]
    pub fn insert(&mut self, entity_id: usize, position: [f32; 3]) {
        let cell = self.position_to_cell(position);
        self.cells
            .entry(cell)
            .or_insert_with(Vec::new)
            .push(entity_id);
    }

    /// Removes an entity from the spatial hash.
    #[inline]
    pub fn remove(&mut self, entity_id: usize, position: [f32; 3]) {
        let cell = self.position_to_cell(position);
        if let Some(entities) = self.cells.get_mut(&cell) {
            entities.retain(|&id| id != entity_id);
        }
    }

    /// Updates an entity's position in the hash.
    #[inline]
    pub fn update(&mut self, entity_id: usize, old_pos: [f32; 3], new_pos: [f32; 3]) {
        let old_cell = self.position_to_cell(old_pos);
        let new_cell = self.position_to_cell(new_pos);

        if old_cell == new_cell {
            return;
        }

        // Remove from old cell
        if let Some(entities) = self.cells.get_mut(&old_cell) {
            entities.retain(|&id| id != entity_id);
        }

        // Add to new cell
        self.insert(entity_id, new_pos);
    }

    /// Returns entities within a radius of a position.
    #[inline]
    #[must_use]
    pub fn query_radius(&self, position: [f32; 3], radius: f32) -> Vec<usize> {
        let cell_radius = (radius / self.cell_size).ceil() as i32;
        let center_cell = self.position_to_cell(position);

        let mut results = Vec::new();

        for dz in -cell_radius..=cell_radius {
            for dy in -cell_radius..=cell_radius {
                for dx in -cell_radius..=cell_radius {
                    let cell = (center_cell.0 + dx, center_cell.1 + dy, center_cell.2 + dz);

                    if let Some(entities) = self.cells.get(&cell) {
                        results.extend(entities.iter().copied());
                    }
                }
            }
        }

        results
    }

    /// Returns entities in the same cell as a position.
    #[inline]
    #[must_use]
    pub fn query_cell(&self, position: [f32; 3]) -> Vec<usize> {
        let cell = self.position_to_cell(position);
        self.cells
            .get(&cell)
            .map(|v| v.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Converts a 3D position to grid cell coordinates.
    #[inline]
    #[must_use]
    fn position_to_cell(&self, position: [f32; 3]) -> (i32, i32, i32) {
        (
            (position[0] / self.cell_size).floor() as i32,
            (position[1] / self.cell_size).floor() as i32,
            (position[2] / self.cell_size).floor() as i32,
        )
    }

    /// Clears all entities from the hash.
    #[inline]
    pub fn clear(&mut self) {
        self.cells.clear();
    }

    /// Returns the number of cells with entities.
    #[inline]
    #[must_use]
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Returns the total number of entities in the hash.
    #[inline]
    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.cells.values().map(|v| v.len()).sum()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════════
// TESTS
// ═════════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════════════════════════
    // AABB TESTS
    // ═════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_aabb_intersects() {
        let aabb1 = Aabb2D::from_center_extents(0.0, 0.0, 1.0, 1.0);
        let aabb2 = Aabb2D::from_center_extents(0.5, 0.5, 1.0, 1.0);

        assert!(aabb1.intersects(&aabb2));
        assert!(aabb2.intersects(&aabb1));
    }

    #[test]
    fn test_aabb_no_intersect() {
        let aabb1 = Aabb2D::from_center_extents(0.0, 0.0, 1.0, 1.0);
        let aabb2 = Aabb2D::from_center_extents(5.0, 5.0, 1.0, 1.0);

        assert!(!aabb1.intersects(&aabb2));
    }

    #[test]
    fn test_aabb_touching() {
        let aabb1 = Aabb2D::from_center_extents(0.0, 0.0, 1.0, 1.0);
        let aabb2 = Aabb2D::from_center_extents(2.0, 0.0, 1.0, 1.0);

        assert!(aabb1.intersects(&aabb2));
    }

    #[test]
    fn test_aabb_intersection_area() {
        // Two AABBs that partially overlap
        // aabb1: min=(-1,-1), max=(1,1) → size 2x2
        // aabb2: min=(-0.5,-0.5), max=(0.5,0.5) → size 1x1
        // Intersection: min=(-0.5,-0.5), max=(0.5,0.5) → size 1x1 = area 1.0
        let aabb1 = Aabb2D::from_center_extents(0.0, 0.0, 1.0, 1.0);
        let aabb2 = Aabb2D::from_center_extents(0.0, 0.0, 0.5, 0.5);

        let area = aabb1.intersection_area(&aabb2);
        assert!((area - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_aabb_3d_intersects() {
        let aabb1 = Aabb3D::from_center_extents(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let aabb2 = Aabb3D::from_center_extents(0.5, 0.5, 0.5, 1.0, 1.0, 1.0);

        assert!(aabb1.intersects(&aabb2));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // COLLISION DETECTOR TESTS
    // ═════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_collision_detector_basic() {
        let mut detector = CollisionSimdDetector::new();

        let aabbs1 = vec![
            Aabb2D::from_center_extents(0.0, 0.0, 1.0, 1.0),
            Aabb2D::from_center_extents(5.0, 5.0, 1.0, 1.0),
        ];
        let aabbs2 = vec![
            Aabb2D::from_center_extents(0.5, 0.5, 1.0, 1.0), // Collision
            Aabb2D::from_center_extents(10.0, 10.0, 1.0, 1.0), // No collision
        ];

        let results = detector.detect_collisions(&aabbs1, &aabbs2);

        assert_eq!(results.len(), 2);
        assert!(results[0]);
        assert!(!results[1]);
        assert_eq!(detector.collision_count(), 1);
    }

    #[test]
    fn test_collision_detector_empty() {
        let mut detector = CollisionSimdDetector::new();
        let aabbs1: Vec<Aabb2D> = vec![];
        let aabbs2: Vec<Aabb2D> = vec![];

        let results = detector.detect_collisions(&aabbs1, &aabbs2);

        assert!(results.is_empty());
        assert_eq!(detector.collision_count(), 0);
    }

    #[test]
    fn test_collision_detector_reset() {
        let mut detector = CollisionSimdDetector::new();

        let aabbs = vec![Aabb2D::from_center_extents(0.0, 0.0, 1.0, 1.0)];
        let same = vec![Aabb2D::from_center_extents(0.0, 0.0, 1.0, 1.0)];

        detector.detect_collisions(&aabbs, &same);
        assert_eq!(detector.collision_count(), 1);

        detector.reset();
        assert_eq!(detector.collision_count(), 0);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // SPATIAL HASH TESTS
    // ═════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_spatial_hash_insert_query() {
        let mut hash = SpatialHash::new(1.0);

        hash.insert(0, [0.0, 0.0, 0.0]);
        hash.insert(1, [0.5, 0.5, 0.5]);
        hash.insert(2, [5.0, 5.0, 5.0]);

        let nearby = hash.query_radius([0.0, 0.0, 0.0], 1.0);
        assert!(nearby.contains(&0));
        assert!(nearby.contains(&1));
        assert!(!nearby.contains(&2));
    }

    #[test]
    fn test_spatial_hash_remove() {
        let mut hash = SpatialHash::new(1.0);

        hash.insert(0, [0.0, 0.0, 0.0]);
        hash.insert(1, [0.0, 0.0, 0.0]);

        hash.remove(0, [0.0, 0.0, 0.0]);

        let cell = hash.query_cell([0.0, 0.0, 0.0]);
        assert_eq!(cell.len(), 1);
        assert!(cell.contains(&1));
    }

    #[test]
    fn test_spatial_hash_update() {
        let mut hash = SpatialHash::new(1.0);

        hash.insert(0, [0.0, 0.0, 0.0]);
        hash.update(0, [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]);

        let old_cell = hash.query_cell([0.0, 0.0, 0.0]);
        let new_cell = hash.query_cell([10.0, 10.0, 10.0]);

        assert!(old_cell.is_empty());
        assert!(new_cell.contains(&0));
    }

    #[test]
    fn test_spatial_hash_clear() {
        let mut hash = SpatialHash::new(1.0);

        hash.insert(0, [0.0, 0.0, 0.0]);
        hash.insert(1, [5.0, 5.0, 5.0]);

        hash.clear();

        assert_eq!(hash.cell_count(), 0);
        assert_eq!(hash.entity_count(), 0);
    }

    #[test]
    fn test_spatial_hash_counts() {
        let mut hash = SpatialHash::new(1.0);

        for i in 0..10 {
            hash.insert(i, [i as f32, 0.0, 0.0]);
        }

        assert_eq!(hash.cell_count(), 10);
        assert_eq!(hash.entity_count(), 10);
    }

    #[test]
    fn test_spatial_hash_same_cell() {
        let mut hash = SpatialHash::new(2.0);

        // These should be in the same cell
        hash.insert(0, [0.0, 0.0, 0.0]);
        hash.insert(1, [0.5, 0.5, 0.5]);
        hash.insert(2, [1.0, 0.0, 0.0]);

        let cell = hash.query_cell([0.5, 0.5, 0.5]);
        assert_eq!(cell.len(), 3);
        assert!(cell.contains(&0));
        assert!(cell.contains(&1));
        assert!(cell.contains(&2));
    }
}
