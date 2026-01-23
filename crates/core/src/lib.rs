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

//! ArchFlow Core
//!
//! Core domain layer containing:
//! - Records: Type-safe IDs, fractional indexing, delta-based history
//! - Geometry: Vec2 and bounds wrappers
//! - Spatial: R-Tree indexing for O(log n) queries
//!
//! # Example
//!
//! ```rust
//! use archflow_core::records::{RecordId, Store, Record, FractionalIndex};
//! use archflow_core::geometry::Vec2;
//!
//! #[derive(Debug, Clone)]
//! struct MyShape {
//!     id: RecordId,
//!     position: Vec2,
//!     index: FractionalIndex,
//! }
//!
//! impl Record for MyShape {
//!     fn id(&self) -> &RecordId { &self.id }
//!     fn type_name(&self) -> &str { "my_shape" }
//!     fn index(&self) -> &FractionalIndex { &self.index }
//!     fn with_index(&self, index: FractionalIndex) -> Self {
//!         Self { id: self.id.clone(), position: self.position, index }
//!     }
//! }
//!
//! let mut store = Store::new();
//! let id = RecordId::new("shape123456".to_string());
//! let shape = MyShape {
//!     id,
//!     position: Vec2::new(100.0, 200.0),
//!     index: FractionalIndex::between(None, None),
//! };
//!
//! store.put(shape);
//! assert!(store.get(&RecordId::new("shape123456".to_string())).is_some());
//! ```

pub mod records;
pub mod geometry;
pub mod spatial;

pub use records::{RecordId, FractionalIndex, Store, Record, RecordChange};
pub use geometry::{Vec2, Bounds};
pub use spatial::{SpatialIndex, SpatialObject};
