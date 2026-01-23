//! Integration test: Renderer with Core and ECS
//!
//! Tests that renderer works correctly with core geometry and ECS components.

use archflow_core::geometry::Vec2;
use archflow_ecs::Color;
use archflow_renderer::{FontManager, TextRenderer, TextStyle};

// Test that text rendering works with core geometry types
#[test]
fn test_text_with_vec2_positions() {
    let mut renderer = TextRenderer::new();

    // Create text at a position that would come from core geometry
    let position = Vec2::new(100.0, 200.0);
    let _x = position.x();
    let _y = position.y();

    // Create a text buffer
    let buffer = renderer.create_text_buffer("Position-based text");

    // Verify the buffer was created
    assert!(!buffer.text().is_empty());
    assert!(buffer.width() > 0.0);
    assert!(buffer.height() > 0.0);
}

// Test that text style colors work with ECS colors
#[test]
fn test_text_style_with_ecs_colors() {
    let mut renderer = TextRenderer::new();

    // ECS Color converted to text color
    let ecs_color = Color::new(1.0, 0.0, 0.0, 1.0);
    let text_color = [
        (ecs_color.r * 255.0) as u8,
        (ecs_color.g * 255.0) as u8,
        (ecs_color.b * 255.0) as u8,
        (ecs_color.a * 255.0) as u8,
    ];

    // Create text style matching ECS color
    let style = TextStyle {
        color: text_color,
        font_size: 16.0,
        ..TextStyle::default()
    };

    let buffer = renderer.create_text_buffer_with_style("Red text", style);

    // Verify the style was applied
    assert_eq!(buffer.style().color, text_color);
}

// Test font manager with system fonts
#[test]
fn test_font_manager_system_fonts() {
    let font_manager = FontManager::new();

    // Should have loaded at least one system font
    let faces: Vec<_> = font_manager.font_db().faces().collect();
    assert!(!faces.is_empty(), "No system fonts loaded");
}

// Test text buffer dimensions match expected geometry
#[test]
fn test_text_dimensions_match_geometry() {
    let mut renderer = TextRenderer::new();

    let short_text = renderer.create_text_buffer("Hi");
    let long_text = renderer.create_text_buffer("This is a longer text");

    // Longer text should have greater or equal width
    assert!(
        long_text.width() >= short_text.width(),
        "Longer text should have greater width"
    );

    // Multiline text should have greater height
    let multiline = renderer.create_text_buffer("Line 1\nLine 2\nLine 3");
    assert!(
        multiline.height() > short_text.height(),
        "Multiline text should have greater height"
    );
}

// Test that text renderer can update buffers
#[test]
fn test_text_buffer_updates() {
    let mut renderer = TextRenderer::new();
    let mut buffer = renderer.create_text_buffer("Original");

    // Update with new text
    renderer.update_text_buffer(&mut buffer, "Updated");

    assert_eq!(buffer.text(), "Updated");
}

// Test that text style can be updated
#[test]
fn test_text_style_updates() {
    let mut renderer = TextRenderer::new();
    let mut buffer = renderer.create_text_buffer("Styled text");

    let new_style = TextStyle {
        font_size: 24.0,
        font_family: "serif".to_string(),
        ..TextStyle::default()
    };

    renderer.update_text_style(&mut buffer, new_style.clone());

    assert_eq!(buffer.style().font_size, 24.0);
    assert_eq!(buffer.style().font_family, "serif");
}

// Test buffer dimensions calculation
#[test]
fn test_buffer_dimensions() {
    let mut renderer = TextRenderer::new();
    let buffer = renderer.create_text_buffer("Test dimensions");

    let (width, height) = renderer.buffer_dimensions(&buffer);

    assert_eq!(width, buffer.width());
    assert_eq!(height, buffer.height());
}

// Test empty and whitespace text
#[test]
fn test_whitespace_text() {
    let mut renderer = TextRenderer::new();

    let empty = renderer.create_text_buffer("");
    assert!(empty.width() >= 0.0);

    let spaces = renderer.create_text_buffer("   ");
    assert!(spaces.width() >= 0.0);

    let newlines = renderer.create_text_buffer("\n\n");
    assert!(newlines.height() >= 0.0);
}

// Test that text rendering doesn't panic with special characters
#[test]
fn test_special_characters() {
    let mut renderer = TextRenderer::new();

    // Should handle various special characters
    let special = renderer.create_text_buffer("Hello! @#$% Unicode: éñ 🎉");
    assert!(special.width() > 0.0);
    assert!(special.height() > 0.0);
}

// Test that multiple buffers can coexist
#[test]
fn test_multiple_buffers() {
    let mut renderer = TextRenderer::new();

    let buffer1 = renderer.create_text_buffer("First");
    let buffer2 = renderer.create_text_buffer("Second");
    let buffer3 = renderer.create_text_buffer("Third");

    assert_eq!(buffer1.text(), "First");
    assert_eq!(buffer2.text(), "Second");
    assert_eq!(buffer3.text(), "Third");
}

// Test that text works with Vec2 operations
#[test]
fn test_text_position_calculations() {
    let pos1 = Vec2::new(0.0, 0.0);
    let pos2 = Vec2::new(100.0, 200.0);

    // Distance between positions
    let dist = pos1.distance_to(pos2);
    assert!((dist - 223.606).abs() < 1.0); // sqrt(100^2 + 200^2) ≈ 223.6

    // Linear interpolation for text positioning
    let mid = Vec2::lerp(pos1, pos2, 0.5);
    assert!((mid.x() - 50.0).abs() < 0.001);
    assert!((mid.y() - 100.0).abs() < 0.001);
}
