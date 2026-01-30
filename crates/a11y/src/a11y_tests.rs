//! Tests for A11y bounded context reorganization
//!
//! These tests verify that the A11y bounded context properly consolidates
//! all accessibility-related functionality from the old architecture.

use crate::{CanvasLike, LayerLike, ShapeLike};
use archflow_core::EntityId;

// Mock Canvas type for testing
#[derive(Debug)]
pub struct MockCanvas {
    pub shapes: Vec<ShapeLike>,
    pub layers: Vec<LayerLike>,
}

impl MockCanvas {
    pub fn new() -> Self {
        Self {
            shapes: Vec::new(),
            layers: vec![
                LayerLike {
                    id: EntityId::new(),
                    name: "Layer 1".to_string(),
                    visible: true,
                },
                LayerLike {
                    id: EntityId::new(),
                    name: "Layer 2".to_string(),
                    visible: true,
                },
            ],
        }
    }

    pub fn add_shape(&mut self, shape: ShapeLike) {
        self.shapes.push(shape);
    }
}

impl CanvasLike for MockCanvas {
    fn all_shapes(&self) -> Vec<ShapeLike> {
        self.shapes.clone()
    }

    fn all_layers(&self) -> Vec<LayerLike> {
        self.layers.clone()
    }
}

use crate::{
    A11yBounds, A11yConfig, A11yManager, A11yVerbosity, FocusableType, LiveRegionType,
    NavigationDirection,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: Verifies that the a11y crate unifies accessibility modules
    #[test]
    fn test_a11y_crate_unified() {
        let manager = A11yManager::new();

        // Verify a11y manager has config
        let config = manager.config();
        assert!(config.enable_aria);
        assert!(config.enable_keyboard);

        // Verify we can set focus
        let mut manager = A11yManager::new();
        let id = EntityId::new();
        manager.set_focused(Some(id));
        assert_eq!(manager.focused(), Some(id));
    }

    /// Test: Verifies that a11y has proper keyboard navigation
    #[test]
    fn test_keyboard_navigation() {
        let mut manager = A11yManager::new();

        // Register focusable elements with specific IDs
        let id1 = EntityId::from_u128(1);
        let id2 = EntityId::from_u128(2);
        let id3 = EntityId::from_u128(3);

        manager.register_focusable(
            id1,
            FocusableType::Shape,
            "Shape 1",
            A11yBounds::new(0.0, 0.0, 50.0, 50.0),
        );

        manager.register_focusable(
            id2,
            FocusableType::Shape,
            "Shape 2",
            A11yBounds::new(100.0, 0.0, 50.0, 50.0),
        );

        manager.register_focusable(
            id3,
            FocusableType::Shape,
            "Shape 3",
            A11yBounds::new(200.0, 0.0, 50.0, 50.0),
        );

        // Initially, no element is focused
        assert_eq!(manager.focused(), None);

        // Test focus next - should focus first element (id1)
        manager.focus_next();
        assert_eq!(manager.focused(), Some(id1));

        // Test focus next again - should focus second element (id2)
        manager.focus_next();
        assert_eq!(manager.focused(), Some(id2));

        // Test focus previous - should go back to first element (id1)
        manager.focus_previous();
        assert_eq!(manager.focused(), Some(id1));
    }

    /// Test: Verifies spatial navigation
    #[test]
    fn test_spatial_navigation() {
        let mut manager = A11yManager::new();

        // Create elements in different positions with specific IDs
        let bottom_id = EntityId::from_u128(10);
        let middle_id = EntityId::from_u128(20);
        let top_id = EntityId::from_u128(30);

        manager.register_focusable(
            bottom_id,
            FocusableType::Shape,
            "Bottom",
            A11yBounds::new(0.0, 100.0, 50.0, 50.0),
        );

        manager.register_focusable(
            middle_id,
            FocusableType::Shape,
            "Middle",
            A11yBounds::new(0.0, 50.0, 50.0, 50.0),
        );

        manager.register_focusable(
            top_id,
            FocusableType::Shape,
            "Top",
            A11yBounds::new(0.0, 0.0, 50.0, 50.0),
        );

        // Focus middle element
        manager.set_focused(Some(middle_id));

        // Navigate up (should find top element)
        let new_focus = manager.navigate(NavigationDirection::Up);
        assert!(new_focus.is_some());
        assert_eq!(new_focus.unwrap(), top_id);
    }

    /// Test: Verifies A11y configuration
    #[test]
    fn test_a11y_config() {
        let config = A11yConfig::default();
        assert!(config.enable_aria);
        assert!(config.enable_keyboard);
        assert!(config.enable_screen_reader);
        assert_eq!(config.verbosity, A11yVerbosity::Normal);

        let mut config = A11yConfig::default();
        config.high_contrast_mode = true;
        config.verbosity = A11yVerbosity::Verbose;

        assert!(config.high_contrast_mode);
        assert_eq!(config.verbosity, A11yVerbosity::Verbose);
    }

    /// Test: Verifies key code conversion
    #[test]
    fn test_key_code_conversion() {
        use crate::KeyCode;

        // Test arrow keys
        assert_eq!(KeyCode::from(37u32), KeyCode::ArrowLeft);
        assert_eq!(KeyCode::from(38u32), KeyCode::ArrowUp);
        assert_eq!(KeyCode::from(39u32), KeyCode::ArrowRight);
        assert_eq!(KeyCode::from(40u32), KeyCode::ArrowDown);

        // Test letter keys
        assert_eq!(KeyCode::from(65u32), KeyCode::A);
        assert_eq!(KeyCode::from(90u32), KeyCode::Z);

        // Test number keys
        assert_eq!(KeyCode::from(48u32), KeyCode::Digit0);
        assert_eq!(KeyCode::from(57u32), KeyCode::Digit9);
    }

    /// Test: Verifies accessibility tree building
    #[test]
    fn test_build_a11y_tree() {
        let mut manager = A11yManager::new();
        let mut canvas = MockCanvas::new();

        canvas.add_shape(ShapeLike {
            id: EntityId::new(),
            shape_type: "Rectangle".to_string(),
            x: 100.0,
            y: 100.0,
            width: 50.0,
            height: 50.0,
        });

        // Build accessibility tree
        let tree = manager.build_tree(&canvas);

        assert_eq!(tree.role, "application");
        assert!(!tree.children.is_empty());
    }

    /// Test: Verifies announcements
    #[test]
    fn test_announcements() {
        let mut manager = A11yManager::new();

        manager.announce("Test announcement", LiveRegionType::Polite);
        let announcements = manager.get_announcements();

        assert_eq!(announcements.len(), 1);
        assert_eq!(announcements[0].text, "Test announcement");
        assert_eq!(announcements[0].priority, LiveRegionType::Polite);
    }

    /// Test: Verifies focusable element bounds
    #[test]
    fn test_a11y_bounds() {
        use crate::Vec2;

        let bounds = A11yBounds::new(10.0, 20.0, 100.0, 50.0);

        assert_eq!(bounds.x, 10.0);
        assert_eq!(bounds.y, 20.0);
        assert_eq!(bounds.width, 100.0);
        assert_eq!(bounds.height, 50.0);

        // Test center calculation
        let center = bounds.center();
        assert_eq!(center.x, 60.0);
        assert_eq!(center.y, 45.0);

        // Test contains
        assert!(bounds.contains(Vec2::new(50.0, 40.0)));
        assert!(!bounds.contains(Vec2::new(5.0, 10.0)));
    }
}
