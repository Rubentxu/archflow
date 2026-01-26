//! # ArchFlow Records Foundation
//!
//! Type-safe IDs, fractional indexing, and delta management system for ArchFlow V2.
//! This crate provides the foundational building blocks for record-based data management.
//!
//! ## Features
//!
//! - **Type-Safe IDs**: `RecordId` with validation and UUID support
//! - **Fractional Indexing**: Conflict-free ordering like tldraw/Figma
//! - **Delta Management**: O(1) memory undo/redo system
//! - **Record Store**: Efficient change tracking with `FixedBitSet`
//! - **Spatial Indexing**: Optional integration with rstar
//!
//! ## Quick Start
//!
//! ```rust
//! use archflow_records::{RecordId, Record, RecordStore};
//!
//! #[derive(Debug, Clone)]
//! struct MyRecord {
//!     id: RecordId,
//!     name: String,
//! }
//!
//! impl Record for MyRecord {
//!     fn id(&self) -> &RecordId { &self.id }
//!     fn type_name(&self) -> &'static str { "MyRecord" }
//!     fn index(&self) -> Option<&FractionalIndex> { None }
//!     fn with_index(self, _index: FractionalIndex) -> Self { self }
//!     fn eq_ignoring_metadata(&self, other: &Self) -> bool {
//!         self.id == other.id && self.name == other.name
//!     }
//!     fn validate(&self) -> Result<(), RecordError> { Ok(()) }
//! }
//!
//! fn main() {
//!     let mut store = RecordStore::new();
//!     let id = RecordId::from_str("record_123").unwrap();
//!     let record = MyRecord { id, name: "test".into() };
//!     store.put(record);
//! }
//! ```

pub mod error;
pub use error::RecordError;

mod record_id;
pub use record_id::RecordId;

mod fractional_index;
pub use fractional_index::FractionalIndex;

mod delta;
pub use delta::{DeltaManager, RecordChange};

mod store;
pub use store::{ChangeSet, RecordStore};

mod trait_record;
pub use trait_record::{Bounds, Record};
