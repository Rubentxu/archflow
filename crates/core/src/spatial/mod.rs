// Copyright 2024 ArchFlow Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Spatial indexing module - R-Tree for O(log n) spatial queries.

use crate::geometry::Bounds;
use crate::records::RecordId;
use rstar::{Point, RTreeObject, AABB};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Simplified spatial object that can be indexed.
pub trait SpatialObject:
    Clone + Send + Sync + 'static + Debug + RTreeObject<Envelope = AABB<[f32; 2]>> + Point<Scalar = f32>
{
    /// Returns the bounding box of the object.
    fn bounds(&self) -> Bounds;

    /// Returns the ID of the object.
    fn id(&self) -> &RecordId;
}

/// R-Tree based spatial index for O(log n) spatial queries.
#[derive(Debug, Clone)]
pub struct SpatialIndex<T: SpatialObject> {
    tree: rstar::RTree<T>,
}

impl<T: SpatialObject> Default for SpatialIndex<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: SpatialObject> SpatialIndex<T> {
    /// Creates a new empty spatial index.
    pub fn new() -> Self {
        Self {
            tree: rstar::RTree::new(),
        }
    }

    /// Inserts an object into the index.
    pub fn insert(&mut self, object: T) {
        self.tree.insert(object);
    }

    /// Removes an object by ID.
    pub fn remove(&mut self, id: &RecordId) -> Option<T> {
        let obj_to_remove = self.tree.iter().find(|obj| obj.id() == id).cloned();
        if let Some(obj) = obj_to_remove {
            self.tree.remove(&obj)
        } else {
            None
        }
    }

    /// Finds all objects at a specific point.
    pub fn point_query(&self, x: f32, y: f32) -> Vec<&T> {
        let envelope = AABB::from_corners([x, y], [x, y]);
        self.tree.locate_in_envelope(&envelope).collect()
    }

    /// Finds all objects that contain a specific point.
    pub fn contains_point(&self, point: crate::geometry::Vec2) -> Vec<&T> {
        self.point_query(point.x(), point.y())
    }

    /// Finds all objects that intersect with a bounds.
    pub fn query_bounds(&self, bounds: Bounds) -> Vec<&T> {
        let envelope = AABB::from_corners(
            [bounds.x(), bounds.y()],
            [bounds.x() + bounds.width(), bounds.y() + bounds.height()],
        );
        self.tree.locate_in_envelope(&envelope).collect()
    }

    /// Frustum culling.
    pub fn frustum_query(&self, viewport: Bounds) -> Vec<&T> {
        self.query_bounds(viewport)
    }

    /// Returns the number of objects.
    pub fn len(&self) -> usize {
        self.tree.iter().count()
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> bool {
        self.tree.iter().next().is_none()
    }

    /// Clears all objects.
    pub fn clear(&mut self) {
        self.tree = rstar::RTree::new();
    }
}

// Integration tests are in integration_tests/
