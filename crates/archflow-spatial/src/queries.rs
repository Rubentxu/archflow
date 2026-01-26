//! # Spatial Queries
//!
//! Optimized spatial queries for selection, hit testing, and viewport operations.

use crate::rtree::RTreeIndex;
use crate::trait_spatial_index::{SpatialIndex, AABB};
use archflow_records::{Record, RecordId};
use std::sync::Arc;

/// Spatial query operations.
pub struct SpatialQueries<R: Record> {
    index: Arc<dyn SpatialIndex<R>>,
}

impl<R: Record> SpatialQueries<R> {
    pub fn new(index: Arc<dyn SpatialIndex<R>>) -> Self {
        Self { index }
    }

    pub fn selection_expanded(&self, viewport: AABB<[f32; 2]>, padding: f32) -> Vec<RecordId> {
        let expanded = viewport.padding(padding);
        self.index.rect_query(expanded)
    }

    pub fn selection_by_zoom(
        &self,
        viewport: AABB<[f32; 2]>,
        zoom: f32,
        min_pixel_size: f32,
    ) -> Vec<RecordId> {
        let padding = min_pixel_size / zoom.max(0.01);
        let expanded = viewport.padding(padding);
        self.index.rect_query(expanded)
    }

    pub fn hit_test(&self, point: [f32; 2], options: HitTestOptions) -> HitTestResult {
        let candidates = self.index.point_query(point);

        let hits: Vec<(RecordId, f32)> = candidates
            .into_iter()
            .filter_map(|id| {
                let bounds = self.index.get_bounds(&id)?;
                if !bounds.contains(point) {
                    return None;
                }
                let z = self.get_z_order(&id);
                Some((id, z))
            })
            .collect();

        let sorted_hits: Vec<(RecordId, f32)> = hits
            .into_iter()
            .filter(|(_, z)| options.include_hidden || *z >= 0.0)
            .take(options.max_results)
            .collect();

        let top_hit = sorted_hits.first().map(|(id, _)| id.clone());

        HitTestResult {
            hits: sorted_hits.into_iter().map(|(id, _)| id).collect(),
            top_hit,
        }
    }

    fn get_z_order(&self, _id: &RecordId) -> f32 {
        0.0
    }
}

/// Hit test options.
#[derive(Debug, Clone, Copy)]
pub struct HitTestOptions {
    pub include_hidden: bool,
    pub max_results: usize,
}

impl Default for HitTestOptions {
    fn default() -> Self {
        Self {
            include_hidden: false,
            max_results: 10,
        }
    }
}

/// Hit test result with z-ordering.
#[derive(Debug, Clone)]
pub struct HitTestResult {
    pub hits: Vec<RecordId>,
    pub top_hit: Option<RecordId>,
}

#[cfg(test)]
mod spatial_queries_tests {
    use super::*;
    use archflow_records::{FractionalIndex, Record, RecordId};
    use std::str::FromStr;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct TestRecord {
        pub id: RecordId,
        pub bounds: Option<TestBounds>,
        pub index: Option<FractionalIndex>,
        pub name: String,
        pub value: i32,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct TestBounds {
        pub min: [f32; 2],
        pub max: [f32; 2],
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
    fn test_selection_expanded() {
        let mut index = RTreeIndex::<TestRecord>::new(16);

        for i in 0..10 {
            let id = RecordId::from_str(&format!("test_{:02}", i)).unwrap();
            let bounds = TestBounds {
                min: [i as f32 * 10.0, 0.0],
                max: [i as f32 * 10.0 + 5.0, 5.0],
            };
            index.insert(id, bounds);
        }

        let queries = SpatialQueries::new(Arc::new(index));
        let viewport = AABB::from_corners([20.0, -10.0], [40.0, 10.0]);

        let results = queries.selection_expanded(viewport, 5.0);

        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_selection_by_zoom() {
        let mut index = RTreeIndex::<TestRecord>::new(16);

        for i in 0..10 {
            let id = RecordId::from_str(&format!("test_{:02}", i)).unwrap();
            let bounds = TestBounds {
                min: [i as f32 * 10.0, 0.0],
                max: [i as f32 * 10.0 + 5.0, 5.0],
            };
            index.insert(id, bounds);
        }

        let queries = SpatialQueries::new(Arc::new(index));
        let viewport = AABB::from_corners([0.0, 0.0], [100.0, 100.0]);

        let zoom_1 = queries.selection_by_zoom(viewport, 1.0, 10.0);
        let zoom_2 = queries.selection_by_zoom(viewport, 0.5, 10.0);

        assert!(zoom_1.len() >= zoom_2.len());
    }

    #[test]
    fn test_hit_test() {
        let mut index = RTreeIndex::<TestRecord>::new(16);

        let id = RecordId::from_str("hit_test_001").unwrap();
        let bounds = TestBounds {
            min: [0.0, 0.0],
            max: [10.0, 10.0],
        };
        index.insert(id, bounds);

        let queries = SpatialQueries::new(Arc::new(index));
        let options = HitTestOptions {
            include_hidden: true,
            max_results: 5,
        };

        let result = queries.hit_test([5.0, 5.0], options);

        assert_eq!(result.hits.len(), 1);
        assert_eq!(result.top_hit, Some(id));
    }

    #[test]
    fn test_hit_test_with_limit() {
        let mut index = RTreeIndex::<TestRecord>::new(16);

        for i in 0..10 {
            let id = RecordId::from_str(&format!("hit_test_{:02}", i)).unwrap();
            let bounds = TestBounds {
                min: [i as f32 * 10.0, 0.0],
                max: [i as f32 * 10.0 + 5.0, 5.0],
            };
            index.insert(id, bounds);
        }

        let queries = SpatialQueries::new(Arc::new(index));
        let options = HitTestOptions {
            include_hidden: true,
            max_results: 3,
        };

        let result = queries.hit_test([5.0, 5.0], options);

        assert_eq!(result.hits.len(), 3);
    }
}

impl<R: Record> SpatialQueries<R> {
    pub fn new(index: Arc<dyn SpatialIndex<R>>) -> Self {
        Self { index }
    }

    pub fn selection_expanded(&self, viewport: AABB<[f32; 2]>, padding: f32) -> Vec<RecordId> {
        let expanded = viewport.grow(padding);
        self.index.rect_query(expanded)
    }

    pub fn selection_by_zoom(
        &self,
        viewport: AABB<[f32; 2]>,
        zoom: f32,
        min_pixel_size: f32,
    ) -> Vec<RecordId> {
        let padding = (min_pixel_size / zoom.max(0.01));
        let expanded = viewport.grow(padding);
        self.index.rect_query(expanded)
    }

    pub fn hit_test(&self, point: [f32; 2], options: HitTestOptions) -> HitTestResult {
        let candidates = self.index.point_query(point);

        let hits: Vec<(RecordId, f32)> = candidates
            .into_iter()
            .filter_map(|id| {
                let bounds = self.index.get_bounds(&id)?;
                if !bounds.contains(point) {
                    return None;
                }
                let z = self.get_z_order(&id);
                Some((id, z))
            })
            .collect();

        let sorted_hits: Vec<(RecordId, f32)> = hits
            .into_iter()
            .filter(|(_, z)| options.include_hidden || *z >= 0.0)
            .take(options.max_results)
            .collect();

        let top_hit = sorted_hits.first().map(|(id, _)| id.clone());

        HitTestResult {
            hits: sorted_hits.into_iter().map(|(id, _)| id).collect(),
            top_hit,
        }
    }

    fn get_z_order(&self, _id: &RecordId) -> f32 {
        0.0
    }
}

/// Hit test options.
#[derive(Debug, Clone, Copy)]
pub struct HitTestOptions {
    pub include_hidden: bool,
    pub max_results: usize,
}

impl Default for HitTestOptions {
    fn default() -> Self {
        Self {
            include_hidden: false,
            max_results: 10,
        }
    }
}

/// Hit test result with z-ordering.
#[derive(Debug, Clone)]
pub struct HitTestResult {
    pub hits: Vec<RecordId>,
    pub top_hit: Option<RecordId>,
}

#[cfg(test)]
mod spatial_queries_tests {
    use super::*;
    use archflow_records::{FractionalIndex, Record, RecordId};
    use std::str::FromStr;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct TestRecord {
        pub id: RecordId,
        pub bounds: Option<TestBounds>,
        pub index: Option<FractionalIndex>,
        pub name: String,
        pub value: i32,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct TestBounds {
        pub min: [f32; 2],
        pub max: [f32; 2],
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
    fn test_selection_expanded() {
        let mut index = RTreeIndex::<TestRecord>::new(16);

        for i in 0..10 {
            let id = RecordId::from_str(&format!("test_{:02}", i)).unwrap();
            let bounds = TestBounds {
                min: [i as f32 * 10.0, 0.0],
                max: [i as f32 * 10.0 + 5.0, 5.0],
            };
            index.insert(id, bounds);
        }

        let queries = SpatialQueries::new(Arc::new(index));
        let viewport = AABB::from_corners([20.0, -10.0], [40.0, 10.0]);

        let results = queries.selection_expanded(viewport, 5.0);

        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_selection_by_zoom() {
        let mut index = RTreeIndex::<TestRecord>::new(16);

        for i in 0..10 {
            let id = RecordId::from_str(&format!("test_{:02}", i)).unwrap();
            let bounds = TestBounds {
                min: [i as f32 * 10.0, 0.0],
                max: [i as f32 * 10.0 + 5.0, 5.0],
            };
            index.insert(id, bounds);
        }

        let queries = SpatialQueries::new(Arc::new(index));
        let viewport = AABB::from_corners([0.0, 0.0], [100.0, 100.0]);

        let zoom_1 = queries.selection_by_zoom(viewport, 1.0, 10.0);
        let zoom_2 = queries.selection_by_zoom(viewport, 0.5, 10.0);

        assert!(zoom_1.len() >= zoom_2.len());
    }

    #[test]
    fn test_hit_test() {
        let mut index = RTreeIndex::<TestRecord>::new(16);

        let id = RecordId::from_str("hit_test_001").unwrap();
        let bounds = TestBounds {
            min: [0.0, 0.0],
            max: [10.0, 10.0],
        };
        index.insert(id, bounds);

        let queries = SpatialQueries::new(Arc::new(index));
        let options = HitTestOptions {
            include_hidden: true,
            max_results: 5,
        };

        let result = queries.hit_test([5.0, 5.0], options);

        assert_eq!(result.hits.len(), 1);
        assert_eq!(result.top_hit, Some(id));
    }

    #[test]
    fn test_hit_test_with_limit() {
        let mut index = RTreeIndex::<TestRecord>::new(16);

        for i in 0..10 {
            let id = RecordId::from_str(&format!("hit_test_{:02}", i)).unwrap();
            let bounds = TestBounds {
                min: [i as f32 * 10.0, 0.0],
                max: [i as f32 * 10.0 + 5.0, 5.0],
            };
            index.insert(id, bounds);
        }

        let queries = SpatialQueries::new(Arc::new(index));
        let options = HitTestOptions {
            include_hidden: true,
            max_results: 3,
        };

        let result = queries.hit_test([5.0, 5.0], options);

        assert_eq!(result.hits.len(), 3);
    }
}
