//! # Viewport Manager
//!
//! Manages element visibility based on viewport with culling optimization.

use crate::rtree::RTreeIndex;
use crate::trait_spatial_index::Frustum;
use archflow_records::{Bounds, RecordId};
use std::marker::PhantomData;
use std::sync::Arc;

/// Viewport manager for spatial culling.
pub struct ViewportManager<R> {
    tree: RTreeIndex,
    last_viewport: Option<Arc<Bounds>>,
    visible_cache: Vec<RecordId>,
    _phantom: PhantomData<R>,
}

impl<R> ViewportManager<R> {
    pub fn new() -> Self {
        Self {
            tree: RTreeIndex::new(16),
            last_viewport: None,
            visible_cache: Vec::new(),
            _phantom: PhantomData,
        }
    }

    /// F.8: Update index with incremental changes from ChangeSet
    pub fn update_index(&mut self, changeset: &ChangeSet) {
        // Remove deleted elements
        for id in &changeset.deleted {
            self.tree.remove(id);
        }

        // Remove updated elements (will be re-inserted)
        for id in &changeset.updated {
            self.tree.remove(id);
        }

        // Insert new elements
        for id in &changeset.created {
            // Note: In a real implementation, we would fetch bounds from a RecordStore
            // For now, this is a placeholder that shows the structure
        }

        // Insert updated elements
        for id in &changeset.updated {
            // Note: In a real implementation, we would fetch bounds from a RecordStore
            // For now, this is a placeholder that shows the structure
        }

        // Clear viewport cache on index changes
        self.last_viewport = None;
        self.visible_cache.clear();
    }

    pub fn get_visible_elements(&mut self, viewport: &Bounds) -> &[RecordId] {
        let frustum = Frustum::new(viewport.clone());

        if let Some(last) = &self.last_viewport {
            if last.as_ref() == viewport {
                return &self.visible_cache;
            }
        }

        self.visible_cache = self.tree.frustum_query(&frustum);
        self.last_viewport = Some(Arc::new(viewport.clone()));
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
    pub created: Vec<RecordId>,
    pub updated: Vec<RecordId>,
    pub deleted: Vec<RecordId>,
}

impl ChangeSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.created.is_empty() && self.updated.is_empty() && self.deleted.is_empty()
    }
}

#[cfg(test)]
mod viewport_manager_tests {
    use super::*;
    use archflow_records::{FractionalIndex, Record, RecordId};
    use std::str::FromStr;
    use std::time::{Duration, Instant};

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

        fn with_index(mut self, _index: FractionalIndex) -> Self {
            self
        }

        fn bounds(&self) -> Option<Bounds> {
            self.bounds.clone()
        }
    }

    #[test]
    fn test_viewport_culling() {
        let mut manager = ViewportManager::<TestRecord>::new();

        for i in 0..100 {
            let id = RecordId::from_str(&format!("viewport_{:08}", i)).unwrap();
            let bounds = Bounds::new(i as f64 * 10.0, 0.0, i as f64 * 10.0 + 5.0, 5.0);
            manager.tree.insert(id, bounds);
        }

        let viewport = Bounds::new(20.0, -10.0, 40.0, 10.0);
        let visible = manager.get_visible_elements(&viewport);

        // Elements at x=20, 30, and 40 should intersect with viewport (20, -10) to (40, 10)
        // Due to the intersects() method including boundaries, all 3 elements are included
        assert_eq!(visible.len(), 3);
    }

    #[test]
    fn test_viewport_cache() {
        let mut manager = ViewportManager::<TestRecord>::new();
        let viewport = Bounds::new(0.0, 0.0, 100.0, 100.0);

        let _ = manager.get_visible_elements(&viewport);

        let viewport2 = viewport.clone();
        let start = Instant::now();
        for _ in 0..100 {
            let _ = manager.get_visible_elements(&viewport2);
        }
        let elapsed = start.elapsed();

        assert!(elapsed < Duration::from_millis(10));
    }

    #[test]
    fn test_changeset() {
        let mut changeset = ChangeSet::new();

        changeset
            .created
            .push(RecordId::from_str("created_00000001").unwrap());
        changeset
            .updated
            .push(RecordId::from_str("updated_00000001").unwrap());
        changeset
            .deleted
            .push(RecordId::from_str("deleted_00000001").unwrap());

        assert!(!changeset.is_empty());
        assert_eq!(changeset.created.len(), 1);
        assert_eq!(changeset.updated.len(), 1);
        assert_eq!(changeset.deleted.len(), 1);
    }
}
