//! # R-Tree Implementation
//!
//! R-Tree wrapper for spatial indexing using rstar crate.

use crate::trait_spatial_index::{SpatialBounds, SpatialIndex};
use archflow_records::{Record, RecordId};
use rstar::{RStarInsertionStrategy, RTree};
use std::collections::HashMap;

/// Wrapper for R-Tree indexing.
pub struct RTreeIndex<R: Record> {
    tree: RTree<RTuple<R>, RStarInsertionStrategy>,
    id_to_bounds: HashMap<RecordId, crate::trait_spatial_index::AABB<[f32; 2]>>,
    capacity: usize,
}

/// Tuple wrapper for R-Tree objects.
#[derive(Debug, Clone)]
pub struct RTuple<R: Record> {
    pub id: RecordId,
    pub bounds: crate::trait_spatial_index::AABB<[f32; 2]>,
}

impl<R: Record> RTuple<R> {
    pub fn new(id: RecordId, bounds: crate::trait_spatial_index::AABB<[f32; 2]>) -> Self {
        Self { id, bounds }
    }
}

impl<R: Record> rstar::RTreeObject for RTuple<R> {
    type Envelope = crate::trait_spatial_index::AABB<[f32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.bounds
    }
}

/// Tuple wrapper for R-Tree objects.
#[derive(Debug, Clone)]
pub struct RTuple<R: Record> {
    pub id: RecordId,
    pub bounds: crate::trait_spatial_index::AABB<[f32; 2]>,
}

impl<R: Record> RTuple<R> {
    pub fn new(id: RecordId, bounds: crate::trait_spatial_index::AABB<[f32; 2]>) -> Self {
        Self { id, bounds }
    }
}

impl<R: Record> rstar::RTreeObject for RTuple<R> {
    type Envelope = crate::trait_spatial_index::AABB<[f32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.bounds
    }
}

/// Tuple wrapper for R-Tree objects.
#[derive(Debug, Clone)]
pub struct RTuple<R: Record> {
    pub id: RecordId,
    pub bounds: AABB<[f32; 2]>,
}

impl<R: Record> RTuple<R> {
    pub fn new(id: RecordId, bounds: AABB<[f32; 2]>) -> Self {
        Self { id, bounds }
    }
}

impl<R: Record> RTreeObject for RTuple<R> {
    type Envelope = AABB<[f32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.bounds
    }
}

impl<R: Record> RTreeIndex<R> {
    pub fn new(capacity: usize) -> Self {
        Self {
            tree: RTree::new_with_params(RStarInsertionStrategy::Ingestion, capacity),
            id_to_bounds: HashMap::new(),
            capacity,
        }
    }

    pub fn insert(&mut self, id: RecordId, bounds: R::Bounds) {
        let aabb = bounds.to_aabb();
        self.tree.insert(RTuple::new(id, aabb));
        self.id_to_bounds.insert(id, bounds);
    }

    pub fn remove(&mut self, id: &RecordId) {
        self.tree.remove(&|obj| obj.id == *id);
        self.id_to_bounds.remove(id);
    }

    pub fn update(&mut self, id: RecordId, new_bounds: R::Bounds) {
        self.remove(id);
        self.insert(id.clone(), new_bounds);
    }

    pub fn point_query(&self, point: [f32; 2]) -> Vec<RecordId> {
        let query_point = [point[0], point[1]];
        self.tree
            .locate_all_at_point(&query_point)
            .map(|obj| obj.id.clone())
            .collect()
    }

    pub fn rect_query(&self, bounds: R::Bounds) -> Self::Iterator {
        let aabb = bounds.to_aabb();
        let results = self
            .tree
            .locate_in_envelope_intersecting(&aabb)
            .map(|obj| (obj.id.clone(), obj.bounds.clone()));

        Box::new(results.into_iter())
    }

    pub fn frustum_query(&self, frustum: &Frustum) -> Vec<RecordId> {
        self.tree
            .locate_in_envelope_intersecting(&frustum.bounds)
            .map(|obj| obj.id.clone())
            .collect()
    }

    pub fn nearest(&self, point: [f32; 2], limit: usize) -> Vec<(RecordId, f32)> {
        let query_point = [point[0], point[1]];
        self.tree
            .nearest_neighbor(&query_point)
            .map(|obj| (obj.id.clone(), obj.bounds.center()))
            .into_iter()
            .take(limit)
            .collect()
    }

    pub fn get_bounds(&self, id: &RecordId) -> Option<R::Bounds> {
        self.id_to_bounds.get(id).cloned()
    }

    pub fn len(&self) -> usize {
        self.tree.size()
    }

    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }
}

/// Iterator for R-Tree query results.
pub struct RTreeIterator<R: Record> {
    inner: Box<dyn Iterator<Item = (RecordId, R::Bounds)> + Send>,
}

impl<R: Record> RTreeIterator<R> {
    pub fn new(items: Vec<(RecordId, R::Bounds)>) -> Self {
        Self {
            inner: Box::new(items.into_iter()),
        }
    }
}

impl<R: Record> Iterator for RTreeIterator<R> {
    type Item = (RecordId, R::Bounds);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<R: Record> SpatialIndex<R> for RTreeIndex<R> {
    type Bounds = R::Bounds;
    type Iterator = RTreeIterator<R>;

    fn insert(&mut self, id: RecordId, bounds: Self::Bounds) {
        self.insert(id, bounds);
    }

    fn remove(&mut self, id: RecordId) {
        self.remove(id);
    }

    fn update(&mut self, id: RecordId, new_bounds: Self::Bounds) {
        self.update(id, new_bounds);
    }

    fn point_query(&self, point: [f32; 2]) -> Vec<RecordId> {
        self.point_query(point)
    }

    fn rect_query(&self, bounds: Self::Bounds) -> Vec<RecordId> {
        self.rect_query(bounds).map(|(id, _)| id).collect()
    }

    fn frustum_query(&self, frustum: &Frustum) -> Vec<RecordId> {
        self.frustum_query(frustum)
    }

    fn nearest(&self, point: [f32; 2], limit: usize) -> Vec<(RecordId, f32)> {
        self.nearest(point, limit)
    }

    fn get_bounds(&self, id: RecordId) -> Option<Self::Bounds> {
        self.get_bounds(&id)
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

#[cfg(test)]
mod rtree_index_tests {
    use super::*;
    use archflow_records::{FractionalIndex, Record, RecordId};
    use std::str::FromStr;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct TestBounds {
        pub min: [f32; 2],
        pub max: [f32; 2],
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
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

    impl crate::trait_spatial_index::SpatialBounds for TestBounds {
        fn from_record(_record: &impl crate::trait_spatial_index::HasBounds) -> Self {
            Self {
                min: [0.0, 0.0],
                max: [1.0, 1.0],
            }
        }

        fn contains(&self, point: [f32; 2]) -> bool {
            point[0] >= self.min[0]
                && point[0] <= self.max[0]
                && point[1] >= self.min[1]
                && point[1] <= self.max[1]
        }

        fn intersects(&self, other: &Self) -> bool {
            !(self.max[0] < other.min[0]
                || self.min[0] > other.max[0]
                || self.max[1] < other.min[1]
                || self.min[1] > other.max[1])
        }

        fn center(&self) -> [f32; 2] {
            [
                (self.min[0] + self.max[0]) / 2.0,
                (self.min[1] + self.max[1]) / 2.0,
            ]
        }

        fn area(&self) -> f32 {
            (self.max[0] - self.min[0]) * (self.max[1] - self.min[1])
        }

        fn grow(&self, amount: f32) -> Self {
            Self {
                min: [self.min[0] - amount, self.min[1] - amount],
                max: [self.max[0] + amount, self.max[1] + amount],
            }
        }

        fn to_aabb(&self) -> AABB<[f32; 2]> {
            crate::trait_spatial_index::AABB::from_corners(self.min, self.max)
        }
    }

    #[test]
    fn test_rtree_insert() {
        let mut index = RTreeIndex::<TestRecord>::new(16);

        let bounds = TestBounds {
            min: [0.0, 0.0],
            max: [10.0, 10.0],
        };
        let id = RecordId::from_str("rtree_test_001").unwrap();

        index.insert(id, bounds);
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn test_rtree_point_query() {
        let mut index = RTreeIndex::<TestRecord>::new(16);

        let bounds = TestBounds {
            min: [0.0, 0.0],
            max: [10.0, 10.0],
        };
        let id = RecordId::from_str("point_query_0001").unwrap();
        index.insert(id, bounds);

        let results = index.point_query([5.0, 5.0]);
        assert_eq!(results.len(), 1);

        let results = index.point_query([20.0, 20.0]);
        assert!(results.is_empty());
    }

    #[test]
    fn test_rtree_rect_query() {
        let mut index = RTreeIndex::<TestRecord>::new(16);

        for i in 0..10 {
            let bounds = TestBounds {
                min: [i as f32 * 10.0, 0.0],
                max: [i as f32 * 10.0 + 5.0, 5.0],
            };
            let id = RecordId::from_str(&format!("rect_query_{:04}", i)).unwrap();
            index.insert(id, bounds);
        }

        let query = TestBounds {
            min: [20.0, 0.0],
            max: [40.0, 10.0],
        };
        let results = index.rect_query(&query);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_rtree_remove() {
        let mut index = RTreeIndex::<TestRecord>::new(16);

        let bounds = TestBounds {
            min: [0.0, 0.0],
            max: [10.0, 10.0],
        };
        let id = RecordId::from_str("remove_test_001").unwrap();
        index.insert(id, bounds);
        assert_eq!(index.len(), 1);

        index.remove(&id);
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_rtree_update() {
        let mut index = RTreeIndex::<TestRecord>::new(16);

        let id = RecordId::from_str("update_test_001").unwrap();
        let bounds1 = TestBounds {
            min: [0.0, 0.0],
            max: [5.0, 5.0],
        };
        let bounds2 = TestBounds {
            min: [10.0, 10.0],
            max: [20.0, 20.0],
        };

        index.insert(id.clone(), bounds1);
        index.update(id.clone(), bounds2);

        let results = index.point_query([15.0, 15.0]);
        assert_eq!(results.len(), 1);

        let old_results = index.point_query([2.5, 2.5]);
        assert!(old_results.is_empty());
    }
}
