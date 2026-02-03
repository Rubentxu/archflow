// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Interaction - Hit Testing with O(1) Spatial Queries
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 13
//
// Hit Testing System:
// - O(1) point queries using SpatialHash
// - Z-order aware (topmost entity wins)
// - Rectangle selection (marquee)
// - AABB-based refinement
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(dead_code)]

use alloc::vec::Vec;

use archflow_core::{EntityId, Rect, Vec2};
use archflow_engine::{EntityStore, SpatialHash};

/// Hit Tester providing O(1) spatial queries using SpatialHash
///
/// **How it achieves O(1):**
/// 1. SpatialHash provides O(1) cell lookup from world position
/// 2. Only entities in relevant cell(s) are candidates
/// 3. AABB refinement filters actual hits
/// 4. Z-order selection picks topmost entity
///
/// **Performance:**
/// - Point query: O(1) average case, O(k) worst case where k = entities in cell
/// - Rect query: O(c) where c = number of cells covering the rect
/// - Each query benefits from spatial hashing vs O(n) brute force
pub struct HitTester;

impl HitTester {
    /// Find the topmost entity at a world-space point
    ///
    /// This is the primary hit testing operation for mouse clicks and taps.
    /// Returns the entity with the highest Z-order at the given position.
    ///
    /// # Algorithm
    /// 1. Query SpatialHash for entities in the cell containing the point
    /// 2. For each candidate, check if point is within entity bounds
    /// 3. Return the candidate with highest Z-order (topmost)
    ///
    /// # Arguments
    /// * `cursor_world` - World-space position to test (e.g., mouse cursor)
    /// * `spatial` - SpatialHash index for fast candidate lookup
    /// * `store` - EntityStore for AABB bounds and Z-order
    ///
    /// # Returns
    /// * `Some(EntityId)` - Topmost entity at the position
    /// * `None` - No entity at this position
    pub fn find_at(
        cursor_world: Vec2,
        spatial: &SpatialHash,
        store: &EntityStore,
    ) -> Option<EntityId> {
        // O(1): Get candidates from SpatialHash cell(s)
        let candidates = spatial.query_point(cursor_world);

        let mut best_hit: Option<EntityId> = None;
        let mut max_z: i32 = -1;

        // O(k): Refine with AABB test and Z-order
        // k = number of entities in the queried cell(s)
        for id in candidates {
            let idx = id.index().0 as usize;

            // Skip if entity not alive
            if !store.is_alive(id) {
                continue;
            }

            // Skip if not visible
            if !store.is_visible(idx) {
                continue;
            }

            // Get entity bounds from world transform
            let world_transform = store.world_transform[idx];
            let pos = Vec2::new(world_transform[0], world_transform[1]);
            let size = Vec2::new(world_transform[2], world_transform[3]);

            let rect = Rect::from_center_size(pos, size);

            // AABB containment test
            if rect.contains(cursor_world) {
                // Check Z-order (layer determines draw order)
                let layer = store.layer(idx) as i32;
                if layer > max_z {
                    max_z = layer;
                    best_hit = Some(id);
                }
            }
        }

        best_hit
    }

    /// Find all entities within a rectangular region
    ///
    /// Used for marquee selection (drag-select) and viewport culling.
    /// Returns all entities whose bounds intersect with the selection rectangle.
    ///
    /// # Algorithm
    /// 1. Query SpatialHash for entities in cells covering the rect
    /// 2. For each candidate, check if entity bounds intersect selection
    /// 3. Return all intersecting entities (unordered)
    ///
    /// # Arguments
    /// * `selection_rect` - World-space rectangle to test
    /// * `spatial` - SpatialHash index for fast candidate lookup
    /// * `store` - EntityStore for AABB bounds
    ///
    /// # Returns
    /// Vector of all entities intersecting the rectangle
    pub fn find_in_rect(
        selection_rect: Rect,
        spatial: &SpatialHash,
        store: &EntityStore,
    ) -> Vec<EntityId> {
        // O(c): Get candidates from cells covering the rectangle
        // c = number of cells intersected by the rectangle
        let candidates = spatial.query_rect(selection_rect);

        let mut results = Vec::new();

        // O(k): Refine with AABB intersection test
        for id in candidates {
            let idx = id.index().0 as usize;

            // Skip if entity not alive
            if !store.is_alive(id) {
                continue;
            }

            // Skip if not visible
            if !store.is_visible(idx) {
                continue;
            }

            // Get entity bounds
            let world_transform = store.world_transform[idx];
            let pos = Vec2::new(world_transform[0], world_transform[1]);
            let size = Vec2::new(world_transform[2], world_transform[3]);

            let entity_rect = Rect::from_center_size(pos, size);

            // Check intersection with selection rectangle
            if selection_rect.intersects(&entity_rect) {
                results.push(id);
            }
        }

        results
    }

    /// Find all entities within a rectangular region, respecting containment
    ///
    /// Similar to `find_in_rect`, but only returns entities that are
    /// **fully contained** within the selection rectangle.
    ///
    /// Useful for operations that require complete containment, like
    /// grouping selected items.
    ///
    /// # Arguments
    /// * `selection_rect` - World-space rectangle to test
    /// * `spatial` - SpatialHash index for fast candidate lookup
    /// * `store` - EntityStore for AABB bounds
    ///
    /// # Returns
    /// Vector of entities fully contained within the rectangle
    pub fn find_contained_in_rect(
        selection_rect: Rect,
        spatial: &SpatialHash,
        store: &EntityStore,
    ) -> Vec<EntityId> {
        let candidates = spatial.query_rect(selection_rect);

        let mut results = Vec::new();

        for id in candidates {
            let idx = id.index().0 as usize;

            if !store.is_alive(id) || !store.is_visible(idx) {
                continue;
            }

            let world_transform = store.world_transform[idx];
            let pos = Vec2::new(world_transform[0], world_transform[1]);
            let size = Vec2::new(world_transform[2], world_transform[3]);

            let entity_rect = Rect::from_center_size(pos, size);

            // Check if entity is fully contained within selection
            if selection_rect.contains_rect(&entity_rect) {
                results.push(id);
            }
        }

        results
    }

    /// Find entities near a point within a given radius
    ///
    /// Useful for "fuzzy" selection when the user clicks near but not
    /// exactly on an entity.
    ///
    /// # Arguments
    /// * `center` - World-space center point
    /// * `radius` - Search radius in world units
    /// * `spatial` - SpatialHash index for fast candidate lookup
    /// * `store` - EntityStore for AABB bounds
    ///
    /// # Returns
    /// Vector of entities within the radius, sorted by distance (nearest first)
    pub fn find_near(
        center: Vec2,
        radius: f32,
        spatial: &SpatialHash,
        store: &EntityStore,
    ) -> Vec<EntityId> {
        // Create search rectangle
        let search_rect = Rect::from_center_size(center, Vec2::splat(radius * 2.0));

        let candidates = spatial.query_rect(search_rect);

        let mut results: Vec<(EntityId, f32)> = Vec::new();

        for id in candidates {
            let idx = id.index().0 as usize;

            if !store.is_alive(id) || !store.is_visible(idx) {
                continue;
            }

            let world_transform = store.world_transform[idx];
            let pos = Vec2::new(world_transform[0], world_transform[1]);
            let size = Vec2::new(world_transform[2], world_transform[3]);

            let entity_rect = Rect::from_center_size(pos, size);

            // Find closest point on entity rect to center
            let closest = entity_rect.closest_point(center);
            let distance = center.distance(closest);

            if distance <= radius {
                results.push((id, distance));
            }
        }

        // Sort by distance (nearest first)
        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        // Extract entity IDs
        results.into_iter().map(|(id, _)| id).collect()
    }

    /// Check if a point hits a specific entity
    ///
    /// Direct hit test for a known entity, bypassing the spatial index.
    /// Useful when you already know which entity you're testing against.
    ///
    /// # Arguments
    /// * `point` - World-space point to test
    /// * `entity` - Specific entity to test against
    /// * `store` - EntityStore for AABB bounds
    ///
    /// # Returns
    /// `true` if the point is within the entity's bounds
    pub fn hits_entity(point: Vec2, entity: EntityId, store: &EntityStore) -> bool {
        let idx = entity.index().0 as usize;

        if !store.is_alive(entity) || !store.is_visible(idx) {
            return false;
        }

        let world_transform = store.world_transform[idx];
        let pos = Vec2::new(world_transform[0], world_transform[1]);
        let size = Vec2::new(world_transform[2], world_transform[3]);

        let rect = Rect::from_center_size(pos, size);
        rect.contains(point)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_core::{MAX_ENTITIES, Rect};
    use archflow_engine::EntityStore;

    #[test]
    fn test_hits_entity_visible() {
        let mut store = EntityStore::new();

        let id = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        // Point inside entity
        assert!(HitTester::hits_entity(Vec2::new(100.0, 100.0), id, &store));

        // Point outside entity
        assert!(!HitTester::hits_entity(Vec2::new(200.0, 200.0), id, &store));
    }

    #[test]
    fn test_hits_entity_invisible() {
        let mut store = EntityStore::new();

        let id = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let idx = id.index().0 as usize;

        // Hide the entity
        store.set_visible(idx, false);

        // Should not hit even if point is inside bounds
        assert!(!HitTester::hits_entity(Vec2::new(100.0, 100.0), id, &store));
    }

    #[test]
    fn test_find_at_empty_store() {
        let store = EntityStore::new();
        let spatial = SpatialHash::new(MAX_ENTITIES as usize);

        let result = HitTester::find_at(Vec2::new(100.0, 100.0), &spatial, &store);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_at_single_entity() {
        let mut store = EntityStore::new();
        let mut spatial = SpatialHash::new(MAX_ENTITIES as usize);

        let id = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let idx = id.index().0 as usize;
        let bounds = Rect::from_center_size(store.pos(idx), store.size(idx));
        spatial.insert(id, bounds);

        // Hit center of entity
        let result = HitTester::find_at(Vec2::new(100.0, 100.0), &spatial, &store);
        assert_eq!(result, Some(id));

        // Miss entity
        let result = HitTester::find_at(Vec2::new(200.0, 200.0), &spatial, &store);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_at_z_order() {
        let mut store = EntityStore::new();
        let mut spatial = SpatialHash::new(MAX_ENTITIES as usize);

        let id1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let id2 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        // Set id2 to higher layer (should be "on top")
        let idx2 = id2.index().0 as usize;
        store.set_layer(idx2, 5);

        // Insert both entities in spatial hash
        let idx1 = id1.index().0 as usize;
        spatial.insert(
            id1,
            Rect::from_center_size(store.pos(idx1), store.size(idx1)),
        );
        spatial.insert(
            id2,
            Rect::from_center_size(store.pos(idx2), store.size(idx2)),
        );

        // Should hit id2 (higher Z-order)
        let result = HitTester::find_at(Vec2::new(100.0, 100.0), &spatial, &store);
        assert_eq!(result, Some(id2));
    }

    #[test]
    fn test_find_in_rect_empty() {
        let store = EntityStore::new();
        let spatial = SpatialHash::new(MAX_ENTITIES as usize);

        let rect = Rect::from_center_size(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));
        let results = HitTester::find_in_rect(rect, &spatial, &store);

        assert!(results.is_empty());
    }

    #[test]
    fn test_find_in_rect_multiple() {
        let mut store = EntityStore::new();
        let mut spatial = SpatialHash::new(MAX_ENTITIES as usize);

        let id1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(20.0, 20.0));
        let id2 = store.spawn(Vec2::new(120.0, 100.0), Vec2::new(20.0, 20.0));
        let id3 = store.spawn(Vec2::new(200.0, 200.0), Vec2::new(20.0, 20.0)); // Far away

        // Insert entities into spatial hash
        let idx1 = id1.index().0 as usize;
        let idx2 = id2.index().0 as usize;
        let idx3 = id3.index().0 as usize;
        spatial.insert(
            id1,
            Rect::from_center_size(store.pos(idx1), store.size(idx1)),
        );
        spatial.insert(
            id2,
            Rect::from_center_size(store.pos(idx2), store.size(idx2)),
        );
        spatial.insert(
            id3,
            Rect::from_center_size(store.pos(idx3), store.size(idx3)),
        );

        // Select area covering first two entities
        let rect = Rect::new(80.0, 80.0, 150.0, 130.0);
        let results = HitTester::find_in_rect(rect, &spatial, &store);

        assert_eq!(results.len(), 2);
        assert!(results.contains(&id1));
        assert!(results.contains(&id2));
        assert!(!results.contains(&id3));
    }

    #[test]
    fn test_find_contained_in_rect() {
        let mut store = EntityStore::new();
        let mut spatial = SpatialHash::new(MAX_ENTITIES as usize);

        // Entity fully inside selection
        let id1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(20.0, 20.0));

        // Entity partially overlapping selection
        let id2 = store.spawn(Vec2::new(115.0, 100.0), Vec2::new(20.0, 20.0));

        // Insert into spatial hash
        let idx1 = id1.index().0 as usize;
        let idx2 = id2.index().0 as usize;
        spatial.insert(
            id1,
            Rect::from_center_size(store.pos(idx1), store.size(idx1)),
        );
        spatial.insert(
            id2,
            Rect::from_center_size(store.pos(idx2), store.size(idx2)),
        );

        // Selection from 90-110 x 90-110
        let rect = Rect::new(90.0, 90.0, 110.0, 110.0);

        // find_in_rect includes partial overlaps
        let partial_results = HitTester::find_in_rect(rect, &spatial, &store);
        assert_eq!(partial_results.len(), 2);

        // find_contained_in_rect only includes fully contained
        let contained_results = HitTester::find_contained_in_rect(rect, &spatial, &store);
        assert_eq!(contained_results.len(), 1);
        assert_eq!(contained_results[0], id1);
    }

    #[test]
    fn test_find_near() {
        let mut store = EntityStore::new();
        let mut spatial = SpatialHash::new(MAX_ENTITIES as usize);

        let id1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(20.0, 20.0));
        let id2 = store.spawn(Vec2::new(110.0, 100.0), Vec2::new(20.0, 20.0));
        let id3 = store.spawn(Vec2::new(150.0, 150.0), Vec2::new(20.0, 20.0)); // Far away

        // Insert into spatial hash
        let idx1 = id1.index().0 as usize;
        let idx2 = id2.index().0 as usize;
        let idx3 = id3.index().0 as usize;
        spatial.insert(
            id1,
            Rect::from_center_size(store.pos(idx1), store.size(idx1)),
        );
        spatial.insert(
            id2,
            Rect::from_center_size(store.pos(idx2), store.size(idx2)),
        );
        spatial.insert(
            id3,
            Rect::from_center_size(store.pos(idx3), store.size(idx3)),
        );

        // Find entities within 20 units of (100, 100)
        let results = HitTester::find_near(Vec2::new(100.0, 100.0), 20.0, &spatial, &store);

        assert_eq!(results.len(), 2);
        assert!(results.contains(&id1));
        assert!(results.contains(&id2));
        assert!(!results.contains(&id3));
    }

    #[test]
    fn test_find_near_sorting() {
        let mut store = EntityStore::new();
        let mut spatial = SpatialHash::new(MAX_ENTITIES as usize);

        let id1 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(20.0, 20.0));
        let id2 = store.spawn(Vec2::new(110.0, 110.0), Vec2::new(20.0, 20.0));

        // Insert into spatial hash
        let idx1 = id1.index().0 as usize;
        let idx2 = id2.index().0 as usize;
        spatial.insert(
            id1,
            Rect::from_center_size(store.pos(idx1), store.size(idx1)),
        );
        spatial.insert(
            id2,
            Rect::from_center_size(store.pos(idx2), store.size(idx2)),
        );

        // Find near (100, 100), id1 should be first (closer)
        let results = HitTester::find_near(Vec2::new(100.0, 100.0), 30.0, &spatial, &store);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0], id1); // Nearest
        assert_eq!(results[1], id2);
    }
}
