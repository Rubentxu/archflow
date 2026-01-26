//! # Spatial Index Trait
//!
//! Abstraction for spatial indexing systems using rstar.

use archflow_records::{Bounds, Record, RecordId};
use std::fmt;

/// Abstraction for spatial indexing.
pub trait SpatialIndex<R: Record>: Send + Sync {
    fn insert(&mut self, id: RecordId, bounds: Bounds);

    fn remove(&mut self, id: RecordId);

    fn update(&mut self, id: RecordId, new_bounds: Bounds);

    fn point_query(&self, point: [f64; 2]) -> Vec<RecordId>;

    fn rect_query(&self, bounds: Bounds) -> Vec<RecordId>;

    fn frustum_query(&self, frustum: &Frustum) -> Vec<RecordId>;

    fn nearest(&self, point: [f64; 2], limit: usize) -> Vec<(RecordId, f64)>;

    fn get_bounds(&self, id: RecordId) -> Option<Bounds>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;
}

/// Viewport frustum for culling.
#[derive(Debug, Clone)]
pub struct Frustum {
    pub bounds: Bounds,
}

impl Frustum {
    pub const fn new(bounds: Bounds) -> Self {
        Self { bounds }
    }
}

impl fmt::Display for Frustum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Frustum({:.2}, {:.2} -> {:.2}, {:.2})",
            self.bounds.min_x, self.bounds.min_y, self.bounds.max_x, self.bounds.max_y
        )
    }
}

#[cfg(test)]
mod trait_spatial_index_tests {
    use super::*;
    use archflow_records::{FractionalIndex, Record, RecordId};
    use std::str::FromStr;

    #[derive(Debug, Clone, PartialEq)]
    pub struct TestRecord {
        pub id: RecordId,
        pub bounds: Option<Bounds>,
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

        fn bounds(&self) -> Option<Bounds> {
            self.bounds.clone()
        }

        fn with_index(self, _index: FractionalIndex) -> Self {
            self
        }
    }

    #[test]
    fn test_bounds_center_tuple() {
        let bounds = Bounds::new(0.0, 0.0, 10.0, 10.0);
        let center = bounds.center();

        assert_eq!(center, (5.0, 5.0));
    }

    #[test]
    fn test_bounds_area() {
        let bounds = Bounds::new(0.0, 0.0, 10.0, 10.0);
        let area = bounds.width() * bounds.height();

        assert_eq!(area, 100.0);
    }

    #[test]
    fn test_bounds_padding() {
        let bounds = Bounds::new(0.0, 0.0, 10.0, 10.0);
        let padded = bounds.padding(5.0);

        assert_eq!(padded.min_x, -5.0);
        assert_eq!(padded.min_y, -5.0);
        assert_eq!(padded.max_x, 15.0);
        assert_eq!(padded.max_y, 15.0);
    }

    #[test]
    fn test_bounds_contains_tuple() {
        let bounds = Bounds::new(0.0, 0.0, 10.0, 10.0);

        assert!(bounds.contains(5.0, 5.0));
        assert!(!bounds.contains(20.0, 20.0));
        assert!(bounds.contains(0.0, 0.0));
        assert!(bounds.contains(10.0, 10.0));
    }

    #[test]
    fn test_bounds_intersects_tuple() {
        let bounds1 = Bounds::new(0.0, 0.0, 10.0, 10.0);
        let bounds2 = Bounds::new(5.0, 5.0, 15.0, 15.0);

        assert!(bounds1.intersects(&bounds2));

        let bounds3 = Bounds::new(20.0, 20.0, 30.0, 30.0);
        assert!(!bounds1.intersects(&bounds3));
    }

    #[test]
    fn test_frustum_display() {
        let bounds = Bounds::new(0.0, 0.0, 100.0, 100.0);
        let frustum = Frustum::new(bounds);

        let display = format!("{}", frustum);
        assert!(display.contains("Frustum"));
        assert!(display.contains("0"));
        assert!(display.contains("100"));
    }

    #[test]
    fn test_record_bounds() {
        let id = RecordId::from_str("bounds_test_00000001").unwrap();
        let record = TestRecord {
            id: id.clone(),
            bounds: Some(Bounds::new(0.0, 0.0, 10.0, 10.0)),
            index: None,
            name: String::from("test"),
            value: 42,
        };

        let bounds = record.bounds().unwrap();
        assert_eq!(bounds.min_x, 0.0);
        assert_eq!(bounds.max_x, 10.0);
    }
}
