//! # R-Tree Implementation
//!
//! R-Tree wrapper for spatial indexing using rstar crate.

use crate::trait_spatial_index::Frustum;
use archflow_records::{Bounds, RecordId};
use rstar::{PointDistance, RTree, RTreeObject, AABB};
use std::collections::HashMap;

/// Converts archflow_records::Bounds to rstar::AABB<[f64; 2]>
fn bounds_to_aabb(bounds: &Bounds) -> AABB<[f64; 2]> {
    AABB::from_corners([bounds.min_x, bounds.min_y], [bounds.max_x, bounds.max_y])
}

/// Wrapper for R-Tree indexing.
pub struct RTreeIndex {
    tree: RTree<SpatialObject>,
    id_to_bounds: HashMap<RecordId, Bounds>,
    capacity: usize,
}

/// Spatial object for R-Tree indexing.
#[derive(Debug, Clone, PartialEq)]
pub struct SpatialObject {
    pub id: RecordId,
    pub bounds: Bounds,
}

impl SpatialObject {
    pub fn new(id: RecordId, bounds: Bounds) -> Self {
        Self { id, bounds }
    }
}

impl RTreeObject for SpatialObject {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        bounds_to_aabb(&self.bounds)
    }
}

impl PointDistance for SpatialObject {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let center = self.bounds.center();
        let dx = center.0 - point[0];
        let dy = center.1 - point[1];
        dx * dx + dy * dy
    }
}

impl RTreeIndex {
    pub fn new(capacity: usize) -> Self {
        Self {
            tree: RTree::new(),
            id_to_bounds: HashMap::new(),
            capacity,
        }
    }

    pub fn insert(&mut self, id: RecordId, bounds: Bounds) {
        let id_clone = id.clone();
        self.tree
            .insert(SpatialObject::new(id_clone.clone(), bounds.clone()));
        self.id_to_bounds.insert(id_clone, bounds);
    }

    pub fn remove(&mut self, id: &RecordId) {
        // Find and collect the object to remove to avoid borrow issues
        let to_remove: Option<SpatialObject> = self.tree.iter().find(|obj| obj.id == *id).cloned();
        if let Some(obj) = to_remove {
            self.tree.remove(&obj);
            self.id_to_bounds.remove(id);
        }
    }

    pub fn update(&mut self, id: RecordId, new_bounds: Bounds) {
        self.remove(&id);
        self.insert(id, new_bounds);
    }

    pub fn point_query(&self, point: [f64; 2]) -> Vec<RecordId> {
        // Find objects whose envelope contains the query point
        self.tree
            .locate_all_at_point(&point)
            .filter_map(|obj| {
                let bounds = &obj.bounds;
                if bounds.contains(point[0], point[1]) {
                    Some(obj.id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn rect_query(&self, bounds: Bounds) -> Vec<RecordId> {
        let aabb = bounds_to_aabb(&bounds);
        self.tree
            .locate_in_envelope_intersecting(&aabb)
            .map(|obj| obj.id.clone())
            .collect()
    }

    pub fn frustum_query(&self, frustum: &Frustum) -> Vec<RecordId> {
        let aabb = bounds_to_aabb(&frustum.bounds);
        self.tree
            .locate_in_envelope_intersecting(&aabb)
            .map(|obj| obj.id.clone())
            .collect()
    }

    pub fn nearest(&self, point: [f64; 2], limit: usize) -> Vec<(RecordId, f64)> {
        match self.tree.nearest_neighbor(&point) {
            Some(obj) => {
                let result = vec![(obj.id.clone(), obj.distance_2(&point).sqrt())];
                result.into_iter().take(limit).collect()
            }
            None => vec![],
        }
    }

    pub fn get_bounds(&self, id: &RecordId) -> Option<Bounds> {
        self.id_to_bounds.get(id).cloned().or_else(|| {
            self.tree
                .iter()
                .find(|obj| obj.id == *id)
                .map(|obj| obj.bounds.clone())
        })
    }

    pub fn len(&self) -> usize {
        self.tree.size()
    }

    pub fn is_empty(&self) -> bool {
        self.tree.size() == 0
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

#[cfg(test)]
mod rtree_index_tests {
    use super::*;
    use archflow_records::{FractionalIndex, Record, RecordId};
    use std::str::FromStr;

    #[derive(Debug, Clone, PartialEq)]
    pub struct TestRecord {
        pub id: RecordId,
        pub index: Option<FractionalIndex>,
        pub name: String,
        pub value: i32,
    }

    impl Record for TestRecord {
        fn id(&self) -> &RecordId {
            &self.id
        }

        fn type_name(&self) -> &'static str {
            "TestRecord"
        }

        fn index(&self) -> Option<&FractionalIndex> {
            self.index.as_ref()
        }

        fn with_index(mut self, _index: FractionalIndex) -> Self {
            self
        }
    }

    #[test]
    fn test_bounds_to_aabb_conversion() {
        let bounds = Bounds::new(0.0, 0.0, 10.0, 10.0);
        let aabb = bounds_to_aabb(&bounds);

        assert_eq!(aabb.lower(), [0.0, 0.0]);
        assert_eq!(aabb.upper(), [10.0, 10.0]);
    }

    #[test]
    fn test_rtree_insert() {
        let mut index = RTreeIndex::new(16);

        let bounds = Bounds::new(0.0, 0.0, 10.0, 10.0);
        let id = RecordId::from_str("rtree_test_00000001").unwrap();

        index.insert(id, bounds);
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn test_rtree_point_query() {
        let mut index = RTreeIndex::new(16);

        let bounds = Bounds::new(0.0, 0.0, 10.0, 10.0);
        let id = RecordId::from_str("point_query_00000001").unwrap();
        index.insert(id, bounds);

        let results = index.point_query([5.0, 5.0]);
        assert_eq!(results.len(), 1);

        let results = index.point_query([20.0, 20.0]);
        assert!(results.is_empty());
    }

    #[test]
    fn test_rtree_rect_query() {
        let mut index = RTreeIndex::new(16);

        for i in 0..10 {
            let bounds = Bounds::new(i as f64 * 10.0, 0.0, i as f64 * 10.0 + 5.0, 5.0);
            let id = RecordId::from_str(&format!("rect_query_{:08}", i)).unwrap();
            index.insert(id, bounds);
        }

        let query = Bounds::new(20.0, 0.0, 40.0, 10.0);
        let results = index.rect_query(query);
        // Elements at x=20, 30, and 40 should intersect with query bounds (20, 0) to (40, 10)
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_rtree_remove() {
        let mut index = RTreeIndex::new(16);

        let bounds = Bounds::new(0.0, 0.0, 10.0, 10.0);
        let id = RecordId::from_str("remove_test_00000001").unwrap();
        index.insert(id.clone(), bounds);
        assert_eq!(index.len(), 1);

        index.remove(&id);
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_rtree_update() {
        let mut index = RTreeIndex::new(16);

        let id = RecordId::from_str("update_test_00000001").unwrap();
        let bounds1 = Bounds::new(0.0, 0.0, 5.0, 5.0);
        let bounds2 = Bounds::new(10.0, 10.0, 20.0, 20.0);

        index.insert(id.clone(), bounds1);
        index.update(id.clone(), bounds2);

        let results = index.point_query([15.0, 15.0]);
        assert_eq!(results.len(), 1);

        let old_results = index.point_query([2.5, 2.5]);
        assert!(old_results.is_empty());
    }
}
