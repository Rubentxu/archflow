//! # Viewport Manager
//!
//! Manages element visibility based on viewport with culling optimization.

use crate::rtree::RTreeIndex;
use crate::trait_spatial_index::{Frustum, SpatialIndex, AABB};
use archflow_records::{Record, RecordId, RecordStore};
use std::sync::Arc;

/// Viewport manager for spatial culling.
pub struct ViewportManager<R: Record + Sized> {
    tree: RTreeIndex<R>,
    last_viewport: Option<Arc<crate::trait_spatial_index::AABB<[f32; 2]>>>,
    visible_cache: Vec<RecordId>,
}

impl<R: Record + Sized> ViewportManager<R> {
    pub fn new() -> Self {
        Self {
            tree: RTreeIndex::new(16),
            last_viewport: None,
            visible_cache: Vec::new(),
        }
    }

    pub fn update_index(&mut self, record_store: &RecordStore<R>, changeset: &ChangeSet) {
        for index in changeset.updated.ones() {
            if let Some(id) = record_store.mapper.resolve_id(index) {
                if let Some(record) = record_store.get(&id) {
                    self.tree.remove(&id);
                }
            }
        }

        for index in changeset.updated.ones().chain(changeset.created.ones()) {
            if let Some(id) = record_store.mapper.resolve_id(index) {
                if let Some(record) = record_store.get(&id) {
                    if let Some(bounds) = record.bounds() {
                        self.tree.insert(id, bounds);
                    }
                }
            }
        }
    }

    pub fn get_visible_elements(
        &mut self,
        viewport: crate::trait_spatial_index::AABB<[f32; 2]>,
    ) -> &[RecordId] {
        if Some(viewport) == self.last_viewport {
            return &self.visible_cache;
        }

        self.visible_cache = self
            .tree
            .locate_in_envelope_intersecting(&viewport)
            .map(|obj| obj.id.clone())
            .collect();

        self.last_viewport = Some(viewport);
        &self.visible_cache
    }

    pub fn len(&self) -> usize {
        self.tree.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }
}

impl ViewportManager {
    pub fn new() -> Self {
        Self {
            tree: RTreeIndex::new(16),
            last_viewport: None,
            visible_cache: Vec::new(),
        }
    }

    pub fn update_index<R: Record>(
        &mut self,
        record_store: &RecordStore<R>,
        changeset: &ChangeSet,
    ) {
        for index in changeset.updated.ones() {
            if let Some(id) = record_store.mapper.resolve_id(index) {
                if let Some(record) = record_store.get(&id) {
                    self.tree.remove(&id);
                }
            }
        }

        for index in changeset.updated.ones().chain(changeset.created.ones()) {
            if let Some(id) = record_store.mapper.resolve_id(index) {
                if let Some(record) = record_store.get(&id) {
                    if let Some(bounds) = record.bounds() {
                        self.tree.insert(id, bounds);
                    }
                }
            }
        }
    }

    pub fn get_visible_elements(&mut self, viewport: AABB<[f32; 2]>) -> &[RecordId] {
        if Some(viewport) == self.last_viewport {
            return &self.visible_cache;
        }

        self.visible_cache = self
            .tree
            .locate_in_envelope_intersecting(&viewport)
            .map(|obj| obj.id.clone())
            .collect();

        self.last_viewport = Some(viewport);
        &self.visible_cache
    }

    pub fn len(&self) -> usize {
        self.tree.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }
}

/// Change set for incremental updates.
#[derive(Debug, Clone, Default)]
pub struct ChangeSet {
    pub created: u64,
    pub updated: u64,
    pub deleted: u64,
}

impl ChangeSet {
    pub fn is_empty(&self) -> bool {
        self.created == 0 && self.updated == 0 && self.deleted == 0
    }
}

#[cfg(test)]
mod viewport_manager_tests {
    use super::*;
    use archflow_records::{FractionalIndex, Record, RecordId, RecordStore};
    use std::str::FromStr;
    use std::time::Duration;
    use std::time::Instant;

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
    fn test_viewport_culling() {
        let mut manager = ViewportManager::new();

        for i in 0..100 {
            let id = RecordId::from_str(&format!("viewport_{:06}", i)).unwrap();
            let bounds = TestBounds {
                min: [i as f32 * 10.0, 0.0],
                max: [i as f32 * 10.0 + 5.0, 5.0],
            };
            manager.tree.insert(id, bounds);
        }

        let viewport = AABB::from_corners([20.0, -10.0], [40.0, 10.0]);
        let visible = manager.get_visible_elements(viewport);

        let expected_ids: Vec<_> = (2..4)
            .map(|i| RecordId::from_str(&format!("viewport_{:06}", i)).unwrap())
            .collect();

        assert_eq!(visible.len(), 2);
        assert!(visible.iter().all(|id| expected_ids.contains(id)));
    }

    #[test]
    fn test_viewport_cache() {
        let mut manager = ViewportManager::new();

        let viewport = AABB::from_corners([0.0, 0.0], [100.0, 100.0]);
        let _ = manager.get_visible_elements(viewport);

        let start = Instant::now();
        for _ in 0..100 {
            let _ = manager.get_visible_elements(viewport);
        }
        let elapsed = start.elapsed();

        assert!(elapsed < Duration::from_millis(10));
    }
}

impl ViewportManager {
    pub fn new() -> Self {
        Self {
            tree: RTreeIndex::new(16),
            last_viewport: None,
            visible_cache: Vec::new(),
        }
    }

    pub fn update_index<R: Record>(
        &mut self,
        record_store: &RecordStore<R>,
        changeset: &ChangeSet,
    ) {
        for index in changeset.updated.ones() {
            if let Some(id) = record_store.mapper.resolve_id(index) {
                if let Some(record) = record_store.get(&id) {
                    self.tree.remove(&id);
                }
            }
        }

        for index in changeset.updated.ones().chain(changeset.created.ones()) {
            if let Some(id) = record_store.mapper.resolve_id(index) {
                if let Some(record) = record_store.get(&id) {
                    if let Some(bounds) = record.bounds() {
                        self.tree.insert(id, bounds);
                    }
                }
            }
        }
    }

    pub fn get_visible_elements(&mut self, viewport: AABB<[f32; 2]>) -> &[RecordId] {
        if Some(viewport) == self.last_viewport {
            return &self.visible_cache;
        }

        self.visible_cache = self
            .tree
            .locate_in_envelope_intersecting(&viewport)
            .map(|obj| obj.id.clone())
            .collect();

        self.last_viewport = Some(viewport);
        &self.visible_cache
    }

    pub fn len(&self) -> usize {
        self.tree.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }
}

/// Change set for incremental updates.
#[derive(Debug, Clone, Default)]
pub struct ChangeSet {
    pub created: u64,
    pub updated: u64,
    pub deleted: u64,
}

impl ChangeSet {
    pub fn is_empty(&self) -> bool {
        self.created == 0 && self.updated == 0 && self.deleted == 0
    }
}

#[cfg(test)]
mod viewport_manager_tests {
    use super::*;
    use archflow_records::{FractionalIndex, Record, RecordId, RecordStore};
    use std::str::FromStr;
    use std::time::Duration;
    use std::time::Instant;

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
    fn test_viewport_culling() {
        let mut manager = ViewportManager::new();

        for i in 0..100 {
            let id = RecordId::from_str(&format!("viewport_{:06}", i)).unwrap();
            let bounds = TestBounds {
                min: [i as f32 * 10.0, 0.0],
                max: [i as f32 * 10.0 + 5.0, 5.0],
            };
            manager.tree.insert(id, bounds);
        }

        let viewport = AABB::from_corners([20.0, -10.0], [40.0, 10.0]);
        let visible = manager.get_visible_elements(viewport);

        let expected_ids: Vec<_> = (2..4)
            .map(|i| RecordId::from_str(&format!("viewport_{:06}", i)).unwrap())
            .collect();

        assert_eq!(visible.len(), 2);
        assert!(visible.iter().all(|id| expected_ids.contains(id)));
    }

    #[test]
    fn test_viewport_cache() {
        let mut manager = ViewportManager::new();

        let viewport = AABB::from_corners([0.0, 0.0], [100.0, 100.0]);
        let _ = manager.get_visible_elements(viewport);

        let start = Instant::now();
        for _ in 0..100 {
            let _ = manager.get_visible_elements(viewport);
        }
        let elapsed = start.elapsed();

        assert!(elapsed < Duration::from_millis(10));
    }
}
