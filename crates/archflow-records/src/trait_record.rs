//! Record Trait Definition
//!
//! This module defines the `Record` trait - the central interface that all
//! records in the system must implement.
//!
//! # Design Principles
//!
//! - **Send + Sync**: Records can be shared across threads
//! - **Debug**: For logging and debugging
//! - **Clone**: For creating modified copies (with_index)
//!
//! # Core Methods
//!
//! | Method | Purpose | Default |
//! |--------|---------|---------|
//! | `id()` | Unique identifier | N/A |
//! | `type_name()` | Type for serialization | N/A |
//! | `index()` | Z-order position | None |
//! | `with_index()` | Create with index | identity |
//! | `bounds()` | Spatial bounds | None |
//! | `merge()` | CRDT merge | clone self |
//! | `eq_ignoring_metadata()` | Content equality | compare all |
//! | `validate()` | Invariant check | Ok(()) |

use crate::error::RecordError;
use crate::fractional_index::FractionalIndex;
use crate::record_id::RecordId;

/// Represents the spatial bounds of a record.
///
/// Used for spatial queries and rendering optimization.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Bounds {
    /// Minimum x coordinate
    pub min_x: f64,
    /// Minimum y coordinate
    pub min_y: f64,
    /// Maximum x coordinate
    pub max_x: f64,
    /// Maximum y coordinate
    pub max_y: f64,
}

impl Bounds {
    /// Creates a new bounds rectangle.
    #[inline]
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    /// Returns the width of the bounds.
    #[inline]
    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    /// Returns the height of the bounds.
    #[inline]
    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    /// Returns the center point of the bounds.
    #[inline]
    pub fn center(&self) -> (f64, f64) {
        (
            (self.min_x + self.max_x) / 2.0,
            (self.min_y + self.max_y) / 2.0,
        )
    }

    /// Checks if a point is within the bounds.
    #[inline]
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }

    /// Checks if this bounds intersects with another.
    #[inline]
    pub fn intersects(&self, other: &Bounds) -> bool {
        !(self.max_x < other.min_x
            || self.min_x > other.max_x
            || self.max_y < other.min_y
            || self.min_y > other.max_y)
    }

    /// Unions this bounds with another, returning the combined bounds.
    #[inline]
    pub fn union(&self, other: &Bounds) -> Bounds {
        Bounds::new(
            self.min_x.min(other.min_x),
            self.min_y.min(other.min_y),
            self.max_x.max(other.max_x),
            self.max_y.max(other.max_y),
        )
    }

    /// Returns a new bounds with the given padding.
    #[inline]
    pub fn padding(&self, padding: f64) -> Bounds {
        Bounds::new(
            self.min_x - padding,
            self.min_y - padding,
            self.max_x + padding,
            self.max_y + padding,
        )
    }
}

impl std::fmt::Display for Bounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Bounds({:.2}, {:.2}, {:.2}, {:.2})",
            self.min_x, self.min_y, self.max_x, self.max_y
        )
    }
}

/// The central trait that all records must implement.
///
/// This trait defines the interface for records in the ArchFlow system.
/// Records are the fundamental unit of data, representing entities in the
/// document model.
///
/// # Implementing Record
///
/// ```
/// use archflow_records::{Record, RecordId, FractionalIndex};
///
/// #[derive(Debug, Clone, PartialEq, Eq)]
/// struct MyRecord {
///     id: RecordId,
///     name: String,
///     value: i32,
/// }
///
/// impl Record for MyRecord {
///     fn id(&self) -> &RecordId {
///         &self.id
///     }
///
///     fn type_name(&self) -> &'static str {
///         "MyRecord"
///     }
///
///     fn index(&self) -> Option<&FractionalIndex> {
///         None
///     }
///
///     fn with_index(mut self, _index: FractionalIndex) -> Self {
///         self
///     }
///
///     fn eq_ignoring_metadata(&self, other: &Self) -> bool {
///         self.id == other.id && self.name == other.name && self.value == other.value
///     }
///
///     fn validate(&self) -> Result<(), archflow_records::RecordError> {
///         if self.name.is_empty() {
///             Err(archflow_records::RecordError::ValidationError(
///                 "name cannot be empty".to_string(),
///             ))
///         } else {
///             Ok(())
///         }
///     }
/// }
/// ```
pub trait Record: Send + Sync + std::fmt::Debug + Clone + 'static {
    /// Returns the unique identifier of this record.
    fn id(&self) -> &RecordId;

    /// Returns the type name of this record for serialization.
    ///
    /// This should be a unique identifier for the record type,
    /// typically the struct name.
    fn type_name(&self) -> &'static str;

    /// Returns the fractional index for ordering.
    ///
    /// If the record has a z-order position, return it here.
    /// This is used for maintaining visual ordering.
    fn index(&self) -> Option<&FractionalIndex> {
        None
    }

    /// Creates a new record with the given index.
    ///
    /// This is used when reordering records. The default
    /// implementation ignores the index.
    fn with_index(self, _index: FractionalIndex) -> Self
    where
        Self: Sized,
    {
        self
    }

    /// Returns the spatial bounds of this record.
    ///
    /// Used for spatial queries and rendering optimization.
    /// Default implementation returns None (no spatial extent).
    fn bounds(&self) -> Option<Bounds> {
        None
    }

    /// Merges another record of the same type into this one.
    ///
    /// This is used for CRDT-style conflict resolution in
    /// collaborative editing. The default implementation keeps
    /// the current value.
    fn merge(&self, _other: &Self) -> Self
    where
        Self: Sized,
    {
        self.clone()
    }

    /// Compares two records ignoring metadata.
    ///
    /// This is used for detecting actual content changes,
    /// ignoring differences in z-order or other metadata.
    /// Default implementation compares all fields.
    fn eq_ignoring_metadata(&self, other: &Self) -> bool
    where
        Self: PartialEq,
    {
        self == other
    }

    /// Validates the record's invariants.
    ///
    /// Override this to add domain-specific validation.
    /// The default implementation always succeeds.
    fn validate(&self) -> Result<(), RecordError> {
        Ok(())
    }
}

#[cfg(test)]
mod record_trait_tests {
    use super::*;
    use crate::{FractionalIndex, Record, RecordError, RecordId};
    use std::str::FromStr;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestRecord {
        id: RecordId,
        index: Option<FractionalIndex>,
        name: String,
        value: i32,
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
        fn with_index(mut self, index: FractionalIndex) -> Self {
            self.index = Some(index);
            self
        }
        fn bounds(&self) -> Option<Bounds> {
            Some(Bounds::new(0.0, 0.0, 100.0, 100.0))
        }
        fn eq_ignoring_metadata(&self, other: &Self) -> bool {
            self.id == other.id && self.name == other.name && self.value == other.value
        }
        fn validate(&self) -> Result<(), RecordError> {
            if self.name.is_empty() {
                Err(RecordError::ValidationError("Name cannot be empty".into()))
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn test_record_id_retrieval() {
        let id = RecordId::from_str("trait_test_00001").unwrap();
        let record = TestRecord {
            id: id.clone(),
            index: None,
            name: "test".into(),
            value: 42,
        };
        assert_eq!(record.id(), &id);
    }

    #[test]
    fn test_record_with_index() {
        let id = RecordId::from_str("index_test_00001").unwrap();
        let index = FractionalIndex::first();
        let record = TestRecord {
            id: id.clone(),
            index: None,
            name: "test".into(),
            value: 42,
        }
        .with_index(index.clone());

        assert_eq!(record.index(), Some(&index));
    }

    #[test]
    fn test_record_validation() {
        let id = RecordId::from_str("validation_test_01").unwrap();
        let valid = TestRecord {
            id: id.clone(),
            index: None,
            name: "valid".into(),
            value: 0,
        };
        let invalid = TestRecord {
            id,
            index: None,
            name: "".into(),
            value: 0,
        };

        assert!(valid.validate().is_ok());
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_eq_ignoring_metadata() {
        let id = RecordId::from_str("metadata_test_001").unwrap();

        let r1 = TestRecord {
            id: id.clone(),
            index: None,
            name: "same".into(),
            value: 1,
        };
        let r2 = TestRecord {
            id: id.clone(),
            index: Some(FractionalIndex::first()),
            name: "same".into(),
            value: 1,
        };
        let r3 = TestRecord {
            id,
            index: None,
            name: "different".into(),
            value: 1,
        };

        assert!(r1.eq_ignoring_metadata(&r2));
        assert!(!r1.eq_ignoring_metadata(&r3));
    }

    #[test]
    fn test_bounds() {
        let id = RecordId::from_str("bounds_test_001").unwrap();
        let record = TestRecord {
            id,
            index: None,
            name: "test".into(),
            value: 42,
        };

        let bounds = record.bounds().unwrap();
        assert_eq!(bounds.min_x, 0.0);
        assert_eq!(bounds.max_x, 100.0);
    }

    #[test]
    fn test_type_name() {
        let id = RecordId::from_str("typename_test_001").unwrap();
        let record = TestRecord {
            id,
            index: None,
            name: "test".into(),
            value: 42,
        };

        assert_eq!(record.type_name(), "TestRecord");
    }

    #[test]
    fn test_default_implementations() {
        // Test record with minimal implementation
        #[derive(Debug, Clone, PartialEq, Eq)]
        struct MinimalRecord {
            id: RecordId,
        }

        impl Record for MinimalRecord {
            fn id(&self) -> &RecordId {
                &self.id
            }
            fn type_name(&self) -> &'static str {
                "MinimalRecord"
            }
        }

        let id = RecordId::from_str("minimal_test_001").unwrap();
        let record = MinimalRecord { id: id.clone() };

        // Default implementations should work
        assert!(record.index().is_none());
        assert!(record.bounds().is_none());
        assert!(record.validate().is_ok());
        assert!(record.eq_ignoring_metadata(&record));
    }

    #[test]
    fn test_bounds_operations() {
        let b1 = Bounds::new(0.0, 0.0, 10.0, 10.0);
        let b2 = Bounds::new(5.0, 5.0, 15.0, 15.0);

        // Contains
        assert!(b1.contains(5.0, 5.0));
        assert!(!b1.contains(20.0, 20.0));

        // Intersects
        assert!(b1.intersects(&b2));
        assert!(!b1.intersects(&Bounds::new(20.0, 20.0, 30.0, 30.0)));

        // Union
        let union = b1.union(&b2);
        assert_eq!(union.min_x, 0.0);
        assert_eq!(union.max_x, 15.0);

        // Dimensions
        assert_eq!(b1.width(), 10.0);
        assert_eq!(b1.height(), 10.0);

        // Center
        assert_eq!(b1.center(), (5.0, 5.0));

        // Padding
        let padded = b1.padding(5.0);
        assert_eq!(padded.min_x, -5.0);
        assert_eq!(padded.max_x, 15.0);
    }

    #[test]
    fn test_bounds_display() {
        let bounds = Bounds::new(1.5, 2.5, 10.0, 20.0);
        let s = format!("{}", bounds);
        assert!(s.contains("Bounds"));
        assert!(s.contains("1.50"));
    }
}
