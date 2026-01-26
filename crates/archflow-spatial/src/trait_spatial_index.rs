//! # Spatial Index Trait
//!
//! Abstraction for spatial indexing systems.

use archflow_records::{Bounds, Record, RecordId};
use std::fmt;

/// Abstraction for spatial indexing.
pub trait SpatialIndex<R: Record>: Send + Sync {
    type Bounds: Bounds;

    type Iterator: Iterator<Item = (RecordId, Self::Bounds)>;

    fn insert(&mut self, id: RecordId, bounds: Self::Bounds);

    fn remove(&mut self, id: RecordId);

    fn update(&mut self, id: RecordId, new_bounds: Self::Bounds);

    fn point_query(&self, point: [f32; 2]) -> Vec<RecordId>;

    fn rect_query(&self, bounds: Self::Bounds) -> Vec<RecordId>;

    fn frustum_query(&self, frustum: &Frustum) -> Vec<RecordId>;

    fn nearest(&self, point: [f32; 2], limit: usize) -> Vec<(RecordId, f32)>;

    fn get_bounds(&self, id: RecordId) -> Option<Self::Bounds>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;
}

/// Viewport frustum for culling.
#[derive(Debug, Clone, Copy)]
pub struct Frustum {
    pub bounds: Bounds,
}

impl Frustum {
    pub const fn new(bounds: Bounds) -> Self {
        Self { bounds }
    }
}

#[cfg(test)]
mod trait_spatial_index_tests {
    use super::*;
    use archflow_records::{FractionalIndex, Record, RecordId};
    use std::str::FromStr;

    #[derive(Debug, Clone, PartialEq, Eq)]
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

        fn with_index(mut self, _index: FractionalIndex) -> Self {
            self
        }
    }

    #[test]
    fn test_bounds_center() {
        let bounds = Bounds::new(0.0, 0.0, 10.0, 10.0);
        let center = bounds.center();

        assert_eq!(center[0], 5.0);
        assert_eq!(center[1], 5.0);
    }

    #[test]
    fn test_bounds_area() {
        let bounds = Bounds::new(0.0, 0.0, 10.0, 10.0);
        let area = bounds.area();

        assert_eq!(area, 100.0);
    }

    #[test]
    fn test_bounds_grow() {
        let bounds = Bounds::new(0.0, 0.0, 10.0, 10.0);
        let grown = bounds.padding(5.0);

        assert_eq!(grown.min_x, -5.0);
        assert_eq!(grown.min_y, -5.0);
        assert_eq!(grown.max_x, 15.0);
        assert_eq!(grown.max_y, 15.0);
    }

    #[test]
    fn test_bounds_contains() {
        let bounds = Bounds::new(0.0, 0.0, 10.0, 10.0);

        assert!(bounds.contains(5.0, 5.0));
        assert!(!bounds.contains(20.0, 20.0));
        assert!(bounds.contains(0.0, 0.0));
        assert!(bounds.contains(10.0, 10.0));
    }

    #[test]
    fn test_bounds_intersects() {
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
}

/// Trait for spatial bounds.
pub trait SpatialBounds: Send + Sync + Clone + PartialEq {
    fn from_record(record: &impl HasBounds) -> Self;

    fn contains(&self, point: [f32; 2]) -> bool;

    fn intersects(&self, other: &Self) -> bool;

    fn center(&self) -> [f32; 2];

    fn area(&self) -> f32;

    fn grow(&self, amount: f32) -> Self;

    fn to_aabb(&self) -> AABB<[f32; 2]>;
}

/// Trait for records with bounds.
pub trait HasBounds {
    fn bounds(&self) -> Option<AABB<[f32; 2]>>;
}

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AABB<T: Copy + PartialEq> {
    pub min: T,
    pub max: T,
}

impl<T: Copy + PartialEq> AABB<T> {
    pub const fn from_corners(min: T, max: T) -> Self {
        Self { min, max }
    }
}

/// Viewport frustum for culling.
#[derive(Debug, Clone, Copy)]
pub struct Frustum {
    pub bounds: AABB<[f32; 2]>,
}

impl Frustum {
    pub const fn new(bounds: AABB<[f32; 2]>) -> Self {
        Self { bounds }
    }
}

impl fmt::Display for Frustum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Frustum({}, {} -> {}, {})",
            self.bounds.min[0], self.bounds.min[1], self.bounds.max[0], self.bounds.max[1]
        )
    }
}

#[cfg(test)]
mod trait_spatial_index_tests {
    use super::*;
    use archflow_records::{FractionalIndex, Record, RecordId};
    use std::str::FromStr;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct TestRecord {
        pub id: RecordId,
        pub bounds: Option<AABB<[f32; 2]>>,
    }

    impl Record for TestRecord {
        fn id(&self) -> &RecordId {
            &self.id
        }

        fn type_name(&self) -> &'static str {
            "TestRecord"
        }

        fn index(&self) -> Option<&FractionalIndex> {
            None
        }

        fn with_index(self, _index: FractionalIndex) -> Self {
            self
        }
    }

    impl HasBounds for TestRecord {
        fn bounds(&self) -> Option<AABB<[f32; 2]>> {
            self.bounds
        }
    }

    #[test]
    fn test_spatial_bounds_center() {
        let bounds = AABB::from_corners([0.0, 0.0], [10.0, 10.0]);
        let center = bounds.center();

        assert_eq!(center[0], 5.0);
        assert_eq!(center[1], 5.0);
    }

    #[test]
    fn test_spatial_bounds_area() {
        let bounds = AABB::from_corners([0.0, 0.0], [10.0, 10.0]);
        let area = bounds.area();

        assert_eq!(area, 100.0);
    }

    #[test]
    fn test_spatial_bounds_grow() {
        let bounds = AABB::from_corners([0.0, 0.0], [10.0, 10.0]);
        let grown = bounds.grow(5.0);

        assert_eq!(grown.min[0], -5.0);
        assert_eq!(grown.min[1], -5.0);
        assert_eq!(grown.max[0], 15.0);
        assert_eq!(grown.max[1], 15.0);
    }

    #[test]
    fn test_spatial_bounds_contains() {
        let bounds = AABB::from_corners([0.0, 0.0], [10.0, 10.0]);

        assert!(bounds.contains([5.0, 5.0]));
        assert!(!bounds.contains([20.0, 20.0]));
        assert!(bounds.contains([0.0, 0.0]));
        assert!(bounds.contains([10.0, 10.0]));
    }

    #[test]
    fn test_spatial_bounds_intersects() {
        let bounds1 = AABB::from_corners([0.0, 0.0], [10.0, 10.0]);
        let bounds2 = AABB::from_corners([5.0, 5.0], [15.0, 15.0]);

        assert!(bounds1.intersects(&bounds2));

        let bounds3 = AABB::from_corners([20.0, 20.0], [30.0, 30.0]);
        assert!(!bounds1.intersects(&bounds3));
    }

    #[test]
    fn test_aabb_from_corners() {
        let aabb = AABB::from_corners([0.0, 0.0], [10.0, 10.0]);

        assert_eq!(aabb.min, [0.0, 0.0]);
        assert_eq!(aabb.max, [10.0, 10.0]);
    }

    #[test]
    fn test_frustum_display() {
        let bounds = AABB::from_corners([0.0, 0.0], [100.0, 100.0]);
        let frustum = Frustum::new(bounds);

        let display = format!("{}", frustum);
        assert!(display.contains("Frustum"));
        assert!(display.contains("0"));
        assert!(display.contains("100"));
    }
}
