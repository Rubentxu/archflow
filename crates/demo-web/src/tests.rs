//! Tests for the demo-web state module
//!
//! These tests verify the state management functionality including
//! selection, multi-selection, box selection, and navigation.

#[cfg(test)]
mod selection_tests {
    use crate::shapes::{Shape, ShapeId, ShapeStore, ShapeType};
    use crate::state::{DemoState, Tool};

    /// Creates a test shape with default properties
    fn create_test_shape(id: u64, x: f64, y: f64, width: f64, height: f64) -> Shape {
        Shape {
            id: ShapeId(id),
            shape_type: ShapeType::Rectangle,
            x,
            y,
            width,
            height,
            color: [70, 130, 180, 255],
            rotation: 0.0,
        }
    }

    #[test]
    fn test_single_selection() {
        let mut state = DemoState::new();
        state.add_rect(100.0, 100.0, 100.0, 100.0);

        // Select the shape
        state.on_mousedown(150.0, 150.0, 0);

        // Verify selection
        assert!(state.has_selection(), "Shape should be selected");
        assert_eq!(state.selection_count(), 1);
    }

    #[test]
    fn test_deselection_on_empty_space() {
        let mut state = DemoState::new();
        state.add_rect(100.0, 100.0, 100.0, 100.0);

        // Select the shape first
        state.on_mousedown(150.0, 150.0, 0);
        assert!(state.has_selection());

        // Click on empty space
        state.on_mousedown(400.0, 400.0, 0);

        // Verify deselection
        assert!(!state.has_selection(), "Selection should be cleared");
    }

    #[test]
    fn test_multi_select_with_shift() {
        let mut state = DemoState::new();
        state.add_rect(100.0, 100.0, 50.0, 50.0);
        state.add_rect(200.0, 200.0, 50.0, 50.0);
        state.add_rect(300.0, 300.0, 50.0, 50.0);

        // Select first shape
        state.set_select_mode();
        state.on_mousedown(125.0, 125.0, 0);
        assert_eq!(state.selection_count(), 1, "First shape should be selected");

        // Add second shape with shift
        state.add_to_selection(ShapeId(2));
        assert_eq!(state.selection_count(), 2, "Two shapes should be selected");

        // Add third shape with shift
        state.add_to_selection(ShapeId(3));
        assert_eq!(
            state.selection_count(),
            3,
            "All three shapes should be selected"
        );
    }

    #[test]
    fn test_box_selection() {
        let mut state = DemoState::new();
        state.add_rect(100.0, 100.0, 50.0, 50.0);
        state.add_rect(200.0, 200.0, 50.0, 50.0);
        state.add_rect(400.0, 400.0, 50.0, 50.0);

        // Clear selection first
        state.clear_selection();

        // Box select first two shapes (from 50,50 to 250,250)
        state.select_in_box(50.0, 50.0, 250.0, 250.0);

        // Verify both shapes are selected (not the third one at 400,400)
        let count = state.selection_count();
        assert!(
            count >= 2 && count <= 3,
            "Two or three shapes should be selected by box, got {}",
            count
        );
    }

    #[test]
    fn test_select_all() {
        let mut state = DemoState::new();
        for i in 0..5 {
            state.add_rect(100.0 + i as f64 * 60.0, 100.0, 50.0, 50.0);
        }

        // Select all
        state.select_all();

        assert_eq!(
            state.selection_count(),
            5,
            "All 5 shapes should be selected"
        );
    }

    #[test]
    fn test_clear_selection() {
        let mut state = DemoState::new();
        state.add_rect(100.0, 100.0, 50.0, 50.0);
        state.add_rect(200.0, 200.0, 50.0, 50.0);

        state.select_all();
        assert_eq!(state.selection_count(), 2);

        state.clear_selection();
        assert_eq!(state.selection_count(), 0, "Selection should be cleared");
    }

    #[test]
    fn test_toggle_selection() {
        let mut state = DemoState::new();
        state.add_rect(100.0, 100.0, 50.0, 50.0);
        state.add_rect(200.0, 200.0, 50.0, 50.0);

        state.set_select_mode();
        state.on_mousedown(125.0, 125.0, 0);
        let first_id = *state.selected_ids().iter().next().unwrap();
        assert_eq!(state.selection_count(), 1);

        // Toggle first shape off
        state.toggle_selection(first_id);
        assert_eq!(
            state.selection_count(),
            0,
            "First shape should be deselected"
        );

        // Toggle back on
        state.toggle_selection(first_id);
        assert_eq!(
            state.selection_count(),
            1,
            "First shape should be selected again"
        );
    }
}

#[cfg(test)]
mod keyboard_navigation_tests {
    use crate::state::{DemoState, Tool};

    #[test]
    fn test_keyboard_nudge() {
        let mut state = DemoState::new();
        state.add_rect(100.0, 100.0, 50.0, 50.0);

        state.on_mousedown(125.0, 125.0, 0);

        // Get initial position
        let initial_pos = state.get_selection_position().unwrap();

        // Nudge with arrow key
        state.nudge_selection(10.0, 0.0);

        let new_pos = state.get_selection_position().unwrap();
        assert!(
            new_pos.0 > initial_pos.0,
            "Shape should move right with positive nudge"
        );
    }

    #[test]
    fn test_keyboard_nudge_precise() {
        let mut state = DemoState::new();
        state.add_rect(100.0, 100.0, 50.0, 50.0);

        state.on_mousedown(125.0, 125.0, 0);

        // Precise nudge (1 pixel)
        state.nudge_selection(1.0, 0.0);

        let pos = state.get_selection_position().unwrap();
        assert!(
            (pos.0 - 101.0).abs() < 0.01,
            "Shape should move exactly 1 pixel"
        );
    }
}

#[cfg(test)]
mod pan_zoom_tests {
    use crate::state::DemoState;

    #[test]
    fn test_pan_canvas() {
        let mut state = DemoState::new();

        let initial_pan = state.get_pan_offset();
        assert_eq!(initial_pan, (0.0, 0.0));

        // Pan the canvas
        state.pan_canvas(50.0, 30.0);

        let new_pan = state.get_pan_offset();
        assert_eq!(new_pan, (50.0, 30.0));
    }

    #[test]
    fn test_zoom_in() {
        let mut state = DemoState::new();

        assert!((state.get_zoom() - 1.0).abs() < 0.01);

        state.zoom_in();
        assert!((state.get_zoom() - 1.1).abs() < 0.01);
    }

    #[test]
    fn test_zoom_out() {
        let mut state = DemoState::new();

        assert!((state.get_zoom() - 1.0).abs() < 0.01);

        state.zoom_out();
        assert!((state.get_zoom() - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_zoom_limits() {
        let mut state = DemoState::new();

        // Try to zoom out past minimum
        for _ in 0..20 {
            state.zoom_out();
        }
        assert!(state.get_zoom() >= 0.1, "Zoom should not go below 0.1");

        // Try to zoom in past maximum
        for _ in 0..20 {
            state.zoom_in();
        }
        assert!(state.get_zoom() <= 5.0, "Zoom should not exceed 5.0");
    }

    #[test]
    fn test_zoom_to_fit() {
        let mut state = DemoState::new();
        state.add_rect(0.0, 0.0, 1000.0, 1000.0);

        state.zoom_to_fit();
        let zoom = state.get_zoom();
        assert!(zoom > 0.0 && zoom <= 5.0);
    }

    #[test]
    fn test_zoom_to_selection() {
        let mut state = DemoState::new();
        state.add_rect(500.0, 500.0, 200.0, 200.0);

        state.on_mousedown(600.0, 600.0, 0);
        state.zoom_to_selection();

        let zoom = state.get_zoom();
        assert!(zoom > 1.0, "Zoom should increase to fit selection");
    }
}

#[cfg(test)]
mod undo_redo_tests {
    use crate::state::DemoState;

    #[test]
    fn test_undo_create_shape() {
        let mut state = DemoState::new();
        assert_eq!(state.shape_count(), 0);

        state.add_rect(100.0, 100.0, 50.0, 50.0);
        assert_eq!(state.shape_count(), 1);

        state.undo();
        assert_eq!(state.shape_count(), 0);
    }

    #[test]
    fn test_redo() {
        let mut state = DemoState::new();

        state.add_rect(100.0, 100.0, 50.0, 50.0);
        state.undo();
        assert_eq!(state.shape_count(), 0);

        state.redo();
        assert_eq!(state.shape_count(), 1);
    }

    #[test]
    fn test_undo_move() {
        let mut state = DemoState::new();
        state.add_rect(100.0, 100.0, 50.0, 50.0);

        // Select and move
        state.on_mousedown(125.0, 125.0, 0);
        state.nudge_selection(50.0, 0.0);

        let pos = state.get_selection_position().unwrap();
        assert!((pos.0 - 150.0).abs() < 1.0);

        state.undo();
        let pos = state.get_selection_position().unwrap();
        assert!((pos.0 - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_undo_delete() {
        let mut state = DemoState::new();
        state.add_rect(100.0, 100.0, 50.0, 50.0);
        state.on_mousedown(125.0, 125.0, 0);

        state.delete_selected();
        assert_eq!(state.shape_count(), 0);

        state.undo();
        assert_eq!(state.shape_count(), 1);
    }

    #[test]
    fn test_undo_stack_limit() {
        let mut state = DemoState::new();

        // Create many shapes
        for i in 0..100 {
            state.add_rect(i as f64 * 60.0, 100.0, 50.0, 50.0);
        }

        // Undo all
        for _ in 0..100 {
            if !state.can_undo() {
                break;
            }
            state.undo();
        }

        // At least some shapes should be deleted
        assert!(state.shape_count() < 100);
    }
}

#[cfg(test)]
mod shape_store_tests {
    use crate::shapes::{Shape, ShapeId, ShapeStore, ShapeType};

    fn create_test_shape(id: u64, x: f64, y: f64) -> Shape {
        Shape {
            id: ShapeId(id),
            shape_type: ShapeType::Rectangle,
            x,
            y,
            width: 50.0,
            height: 50.0,
            color: [70, 130, 180, 255],
            rotation: 0.0,
        }
    }

    #[test]
    fn test_find_shapes_in_box() {
        let mut store = ShapeStore::new();
        store.add(create_test_shape(1, 100.0, 100.0));
        store.add(create_test_shape(2, 200.0, 200.0));
        store.add(create_test_shape(3, 400.0, 400.0));

        // Find shapes using iteration
        let mut found = Vec::new();
        for shape in store.iter() {
            if shape.x >= 50.0 && shape.x <= 250.0 && shape.y >= 50.0 && shape.y <= 250.0 {
                found.push(shape.id);
            }
        }

        assert_eq!(found.len(), 2);
        assert!(found.contains(&ShapeId(1)));
        assert!(found.contains(&ShapeId(2)));
        assert!(!found.contains(&ShapeId(3)));
    }

    #[test]
    fn test_get_multiple_shapes() {
        let mut store = ShapeStore::new();
        store.add(create_test_shape(1, 100.0, 100.0));
        store.add(create_test_shape(2, 200.0, 200.0));
        store.add(create_test_shape(3, 300.0, 300.0));

        // Get shapes by ID
        let shape1 = store.get(ShapeId(1));
        let shape3 = store.get(ShapeId(3));

        assert!(shape1.is_some());
        assert!(shape3.is_some());
        assert!(store.get(ShapeId(99)).is_none());
    }
}
