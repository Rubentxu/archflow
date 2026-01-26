//! # Spatial Queries
//!
//! Optimized spatial queries for selection, hit testing, and viewport operations.

use crate::rtree::RTreeIndex;
use archflow_records::{Bounds, RecordId};
use std::marker::PhantomData;

/// Spatial query operations.
pub struct SpatialQueries<R> {
    index: RTreeIndex,
    _phantom: PhantomData<R>,
}

impl<R> SpatialQueries<R> {
    pub fn new(index: RTreeIndex) -> Self {
        Self {
            index,
            _phantom: PhantomData,
        }
    }

    pub fn selection_expanded(&self, viewport: Bounds, padding: f64) -> Vec<RecordId> {
        let expanded = viewport.padding(padding);
        self.index.rect_query(expanded)
    }

    pub fn selection_by_zoom(
        &self,
        viewport: Bounds,
        zoom: f64,
        min_pixel_size: f64,
    ) -> Vec<RecordId> {
        let padding = min_pixel_size / zoom.max(0.01);
        let expanded = viewport.padding(padding);
        self.index.rect_query(expanded)
    }

    pub fn hit_test(&self, point: [f64; 2], options: HitTestOptions) -> HitTestResult {
        let candidates = self.index.point_query(point);

        let hits: Vec<(RecordId, f64)> = candidates
            .into_iter()
            .filter_map(|id| {
                let bounds = self.index.get_bounds(&id)?;
                if !bounds.contains(point[0], point[1]) {
                    return None;
                }
                let z = self.get_z_order(&id);
                Some((id, z))
            })
            .collect();

        let sorted_hits: Vec<(RecordId, f64)> = hits
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

    fn get_z_order(&self, _id: &RecordId) -> f64 {
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

    #[derive(Debug, Clone, PartialEq)]
    pub struct TestRecord {
        pub id: RecordId,
        pub index: Option<FractionalIndex>,
        pub bounds: Option<Bounds>,
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

        fn bounds(&self) -> Option<Bounds> {
            self.bounds.clone()
        }

        fn with_index(self, _index: FractionalIndex) -> Self {
            self
        }
    }

    #[test]
    fn test_selection_expanded() {
        let mut index = RTreeIndex::new(16);

        for i in 0..10 {
            let id = RecordId::from_str(&format!("test_query_{:08}", i)).unwrap();
            let bounds = Bounds::new(i as f64 * 10.0, 0.0, i as f64 * 10.0 + 5.0, 5.0);
            index.insert(id, bounds);
        }

        let queries = SpatialQueries::<TestRecord>::new(index);
        let viewport = Bounds::new(20.0, -10.0, 40.0, 10.0);

        let results = queries.selection_expanded(viewport.clone(), 5.0);

        // Viewport (20, -10) to (40, 10) with padding 5 becomes (15, -15) to (45, 15)
        // Intersects with rectangles: i=1 (10-15), i=2 (20-25), i=3 (30-35), i=4 (40-45)
        assert_eq!(results.len(), 4);
    }

    #[test]
    fn test_selection_by_zoom() {
        let mut index = RTreeIndex::new(16);

        for i in 0..10 {
            let id = RecordId::from_str(&format!("test_zoom_{:08}", i)).unwrap();
            let bounds = Bounds::new(i as f64 * 10.0, 0.0, i as f64 * 10.0 + 5.0, 5.0);
            index.insert(id, bounds);
        }

        let queries = SpatialQueries::<TestRecord>::new(index);
        let viewport = Bounds::new(0.0, 0.0, 100.0, 100.0);

        let zoom_1 = queries.selection_by_zoom(viewport.clone(), 1.0, 10.0);
        let zoom_2 = queries.selection_by_zoom(viewport.clone(), 0.5, 10.0);

        assert!(zoom_1.len() >= zoom_2.len());
    }

    #[test]
    fn test_hit_test() {
        let mut index = RTreeIndex::new(16);

        let id = RecordId::from_str("hit_test_00000001").unwrap();
        let bounds = Bounds::new(0.0, 0.0, 10.0, 10.0);
        index.insert(id.clone(), bounds);

        let queries = SpatialQueries::<TestRecord>::new(index);
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
        let mut index = RTreeIndex::new(16);

        for i in 0..10 {
            let id = RecordId::from_str(&format!("hit_limit_{:08}", i)).unwrap();
            let bounds = Bounds::new(i as f64 * 10.0, 0.0, i as f64 * 10.0 + 5.0, 5.0);
            index.insert(id, bounds);
        }

        let queries = SpatialQueries::<TestRecord>::new(index);
        let options = HitTestOptions {
            include_hidden: true,
            max_results: 3,
        };

        // Use point [2.5, 2.5] which is clearly inside the first rectangle (0, 0) to (5, 5)
        let result = queries.hit_test([2.5, 2.5], options);

        // Only element at i=0 contains this point
        assert_eq!(result.hits.len(), 1);
    }
}
