//! Integration test: Records with Geometry
//!
//! Tests that Record IDs work correctly with spatial data.

use archflow_core::geometry::Vec2;
use archflow_core::records::{FractionalIndex, Record, RecordId, Store};

/// A simple shape record for testing
#[derive(Debug, Clone, PartialEq)]
struct ShapeRecord {
    id: RecordId,
    index: FractionalIndex,
    type_name: String,
    position: Vec2,
    width: f32,
    height: f32,
}

impl Record for ShapeRecord {
    fn id(&self) -> &RecordId {
        &self.id
    }

    fn type_name(&self) -> &str {
        &self.type_name
    }

    fn index(&self) -> &FractionalIndex {
        &self.index
    }

    fn with_index(&self, index: FractionalIndex) -> Self {
        Self {
            id: self.id.clone(),
            index,
            type_name: self.type_name.clone(),
            position: self.position,
            width: self.width,
            height: self.height,
        }
    }
}

fn make_record_id(suffix: &str) -> RecordId {
    RecordId::new(format!("test-id-{}", suffix))
}

#[test]
fn test_store_with_vec2_records() {
    let mut store = Store::new();

    // Insert shape records with positions - must use unique IDs
    let shapes = vec![
        ShapeRecord {
            id: make_record_id("rect01"),
            index: FractionalIndex::new("a0".to_string()),
            type_name: "rect".to_string(),
            position: Vec2::new(100.0, 100.0),
            width: 50.0,
            height: 50.0,
        },
        ShapeRecord {
            id: make_record_id("rect02"),
            index: FractionalIndex::new("a1".to_string()),
            type_name: "rect".to_string(),
            position: Vec2::new(200.0, 200.0),
            width: 100.0,
            height: 80.0,
        },
    ];

    for shape in &shapes {
        store.put(shape.clone());
    }

    // Verify records are stored
    assert_eq!(store.len(), 2);

    // Retrieve and verify using RecordId
    let id = store.iter().next().unwrap().id();
    let retrieved = store.get(id).unwrap();
    assert_eq!(retrieved.type_name(), "rect");
    assert_eq!(retrieved.position, Vec2::new(100.0, 100.0));
}

#[test]
fn test_store_undo_redo_with_positions() {
    let mut store = Store::new();

    let id = make_record_id("shape1");

    // Add initial record
    store.put(ShapeRecord {
        id: id.clone(),
        index: FractionalIndex::new("a0".to_string()),
        type_name: "shape".to_string(),
        position: Vec2::ZERO,
        width: 100.0,
        height: 100.0,
    });

    // Update position
    store.put(ShapeRecord {
        id: id.clone(),
        index: FractionalIndex::new("a0".to_string()),
        type_name: "shape".to_string(),
        position: Vec2::new(50.0, 50.0),
        width: 100.0,
        height: 100.0,
    });

    // Undo should restore original position
    assert!(store.undo());
    let shape = store.get(&id).unwrap();
    assert_eq!(shape.position, Vec2::ZERO);

    // Redo should restore new position
    assert!(store.redo());
    let shape = store.get(&id).unwrap();
    assert_eq!(shape.position, Vec2::new(50.0, 50.0));
}

#[test]
fn test_store_with_fractional_indexing() {
    let mut store = Store::new();

    let first_id = make_record_id("first");
    store.put(ShapeRecord {
        id: first_id.clone(),
        index: FractionalIndex::new("a0".to_string()),
        type_name: "shape".to_string(),
        position: Vec2::ZERO,
        width: 10.0,
        height: 10.0,
    });

    // Insert between
    let between = FractionalIndex::between(None, Some(store.iter().next().unwrap().index()));
    store.put(ShapeRecord {
        id: make_record_id("middle"),
        index: between,
        type_name: "shape".to_string(),
        position: Vec2::new(5.0, 5.0),
        width: 10.0,
        height: 10.0,
    });

    // Verify ordering - should be 2 items
    assert_eq!(store.len(), 2);
}

#[test]
fn test_vec2_distance_calculations() {
    let pos1 = Vec2::new(0.0, 0.0);
    let pos2 = Vec2::new(3.0, 4.0);

    // Distance should be 5 (3-4-5 triangle)
    assert!((pos1.distance_to(pos2) - 5.0).abs() < 0.001);

    // Dot product should be 0 (perpendicular)
    let pos3 = Vec2::new(1.0, 0.0);
    let pos4 = Vec2::new(0.0, 1.0);
    assert!((pos3.dot(pos4)).abs() < 0.001);
}

#[test]
fn test_vec2_lerp_for_animation() {
    let start = Vec2::new(0.0, 0.0);
    let end = Vec2::new(100.0, 100.0);

    // 50% should be midpoint
    let mid = Vec2::lerp(start, end, 0.5);
    assert!((mid.x() - 50.0).abs() < 0.001);
    assert!((mid.y() - 50.0).abs() < 0.001);

    // 0% should be start
    let zero = Vec2::lerp(start, end, 0.0);
    assert_eq!(zero, start);

    // 100% should be end
    let one = Vec2::lerp(start, end, 1.0);
    assert_eq!(one, end);
}
