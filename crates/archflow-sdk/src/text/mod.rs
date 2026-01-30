//! Text tool module for ArchFlow SDK
//!
//! Provides functionality for creating and editing text on the canvas:
//! - Text entity creation and management
//! - Content editing with edit mode
//! - Styling (font family, size, weight, color)
//! - Position and movement
//! - Serialization support
//! - Integration with undo/redo system

use crate::canvas::Canvas;
use crate::commands::{Command, CommandResult};
use crate::selection::SelectionDelta;
use archflow_core::{Color, EntityId, Vec2};
use serde::{Deserialize, Serialize};

/// Error type for text operations
#[derive(Debug, thiserror::Error)]
pub enum TextError {
    #[error("Text entity not found: {0}")]
    TextNotFound(EntityId),
    #[error("Invalid text content: {0}")]
    InvalidContent(String),
    #[error("Invalid font size: {0}")]
    InvalidFontSize(f32),
    #[error("Not in edit mode")]
    NotInEditMode,
    #[error("Already in edit mode")]
    AlreadyInEditMode,
    #[error("Shape is not a text entity")]
    NotATextEntity,
}

/// Type alias for text operation results
pub type TextResult<T> = Result<T, TextError>;

/// Text styling properties
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {
    /// Font family name
    pub font_family: String,
    /// Font size in pixels
    pub font_size: f32,
    /// Font weight (400 = normal, 700 = bold)
    pub font_weight: u16,
    /// Whether text is italic
    pub italic: bool,
    /// Text color
    pub color: Color,
    /// Text alignment
    pub alignment: TextAlignment,
    /// Line height multiplier
    pub line_height: f32,
    /// Letter spacing in pixels
    pub letter_spacing: f32,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_family: "Arial".to_string(),
            font_size: 16.0,
            font_weight: 400,
            italic: false,
            color: Color::rgb(0.0, 0.0, 0.0),
            alignment: TextAlignment::Left,
            line_height: 1.2,
            letter_spacing: 0.0,
        }
    }
}

impl TextStyle {
    /// Creates a new text style with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the font family
    pub fn with_font_family(mut self, family: impl Into<String>) -> Self {
        self.font_family = family.into();
        self
    }

    /// Sets the font size
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size.max(1.0);
        self
    }

    /// Sets the font weight
    pub fn with_font_weight(mut self, weight: u16) -> Self {
        self.font_weight = weight.clamp(100, 900);
        self
    }

    /// Sets italic style
    pub fn with_italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }

    /// Sets the text color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets the text alignment
    pub fn with_alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self
    }
}

/// Text alignment options
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TextAlignment {
    /// Left-aligned text
    Left,
    /// Center-aligned text
    Center,
    /// Right-aligned text
    Right,
    /// Justified text
    Justify,
}

impl Default for TextAlignment {
    fn default() -> Self {
        TextAlignment::Left
    }
}

/// Represents a text entity on the canvas
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextEntity {
    /// Shape ID reference
    pub shape_id: EntityId,
    /// Text content
    pub content: String,
    /// Text styling
    pub style: TextStyle,
    /// Whether the text is in edit mode
    pub edit_mode: bool,
    /// Cursor position in edit mode
    pub cursor_position: usize,
    /// Selection start (for text selection within the entity)
    pub selection_start: Option<usize>,
}

impl TextEntity {
    /// Creates a new text entity
    pub fn new(shape_id: EntityId, content: impl Into<String>, style: TextStyle) -> Self {
        let content = content.into();
        Self {
            shape_id,
            content,
            style,
            edit_mode: false,
            cursor_position: 0,
            selection_start: None,
        }
    }

    /// Enters edit mode
    pub fn enter_edit_mode(&mut self) -> TextResult<()> {
        if self.edit_mode {
            return Err(TextError::AlreadyInEditMode);
        }
        self.edit_mode = true;
        self.cursor_position = self.content.len();
        Ok(())
    }

    /// Exits edit mode
    pub fn exit_edit_mode(&mut self) -> TextResult<()> {
        if !self.edit_mode {
            return Err(TextError::NotInEditMode);
        }
        self.edit_mode = false;
        self.cursor_position = 0;
        self.selection_start = None;
        Ok(())
    }

    /// Returns true if in edit mode
    pub fn is_editing(&self) -> bool {
        self.edit_mode
    }

    /// Sets the text content
    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
        self.cursor_position = self.cursor_position.min(self.content.len());
    }

    /// Appends text at cursor position
    pub fn insert_text(&mut self, text: &str) {
        self.content.insert_str(self.cursor_position, text);
        self.cursor_position += text.len();
    }

    /// Deletes character before cursor
    pub fn backspace(&mut self) {
        if self.cursor_position > 0 {
            self.content.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    /// Moves cursor to the left
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Moves cursor to the right
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.content.len() {
            self.cursor_position += 1;
        }
    }

    /// Calculates the approximate width based on content and font size
    pub fn approximate_width(&self) -> f32 {
        // Rough approximation: each character is about 0.6 * font_size on average
        let char_width = self.style.font_size * 0.6;
        self.content.len() as f32 * char_width
    }

    /// Calculates the approximate height based on font size and line height
    pub fn approximate_height(&self) -> f32 {
        let line_count = self.content.matches('\n').count() as f32 + 1.0;
        self.style.font_size * self.style.line_height * line_count
    }
}

/// Manager for text entities
#[derive(Debug, Default)]
pub struct TextManager {
    /// All text entities indexed by shape ID
    texts: std::collections::HashMap<EntityId, TextEntity>,
    /// Currently editing text entity (if any)
    active_edit: Option<EntityId>,
}

impl TextManager {
    /// Creates a new text manager
    pub fn new() -> Self {
        Self {
            texts: std::collections::HashMap::new(),
            active_edit: None,
        }
    }

    /// Creates a new text entity on the canvas
    pub fn create_text(
        &mut self,
        canvas: &mut Canvas,
        x: f32,
        y: f32,
        content: impl Into<String>,
        style: Option<TextStyle>,
    ) -> EntityId {
        let content = content.into();
        let style = style.unwrap_or_default();

        // Create approximate dimensions
        let temp_text = TextEntity::new(EntityId::new(), content.clone(), style.clone());
        let width = temp_text.approximate_width();
        let height = temp_text.approximate_height();

        // Create the shape on canvas
        let shape_id = canvas.create_text(x, y, width, height);

        // Create the text entity
        let text_entity = TextEntity::new(shape_id, content, style);
        self.texts.insert(shape_id, text_entity);

        shape_id
    }

    /// Gets a text entity by shape ID
    pub fn get_text(&self, shape_id: EntityId) -> Option<&TextEntity> {
        self.texts.get(&shape_id)
    }

    /// Gets a mutable text entity by shape ID
    pub fn get_text_mut(&mut self, shape_id: EntityId) -> Option<&mut TextEntity> {
        self.texts.get_mut(&shape_id)
    }

    /// Removes a text entity
    pub fn remove_text(&mut self, shape_id: EntityId) -> Option<TextEntity> {
        if self.active_edit == Some(shape_id) {
            self.active_edit = None;
        }
        self.texts.remove(&shape_id)
    }

    /// Enters edit mode for a text entity
    pub fn enter_edit_mode(&mut self, shape_id: EntityId) -> TextResult<()> {
        // Exit any current edit mode
        if let Some(active_id) = self.active_edit {
            if let Some(text) = self.texts.get_mut(&active_id) {
                let _ = text.exit_edit_mode();
            }
        }

        let text = self
            .texts
            .get_mut(&shape_id)
            .ok_or(TextError::TextNotFound(shape_id))?;

        text.enter_edit_mode()?;
        self.active_edit = Some(shape_id);
        Ok(())
    }

    /// Exits edit mode
    pub fn exit_edit_mode(&mut self) -> TextResult<()> {
        if let Some(shape_id) = self.active_edit {
            let text = self
                .texts
                .get_mut(&shape_id)
                .ok_or(TextError::TextNotFound(shape_id))?;
            text.exit_edit_mode()?;
            self.active_edit = None;
        }
        Ok(())
    }

    /// Gets the currently active edit text ID
    pub fn active_edit(&self) -> Option<EntityId> {
        self.active_edit
    }

    /// Returns true if any text is being edited
    pub fn is_editing(&self) -> bool {
        self.active_edit.is_some()
    }

    /// Updates text content
    pub fn set_content(
        &mut self,
        shape_id: EntityId,
        content: impl Into<String>,
    ) -> TextResult<()> {
        let text = self
            .texts
            .get_mut(&shape_id)
            .ok_or(TextError::TextNotFound(shape_id))?;
        text.set_content(content);
        Ok(())
    }

    /// Updates text style
    pub fn set_style(&mut self, shape_id: EntityId, style: TextStyle) -> TextResult<()> {
        let text = self
            .texts
            .get_mut(&shape_id)
            .ok_or(TextError::TextNotFound(shape_id))?;
        text.style = style;
        Ok(())
    }

    /// Gets all text entities
    pub fn all_texts(&self) -> Vec<&TextEntity> {
        self.texts.values().collect()
    }

    /// Gets the number of text entities
    pub fn text_count(&self) -> usize {
        self.texts.len()
    }

    /// Checks if a shape is a text entity
    pub fn is_text(&self, shape_id: EntityId) -> bool {
        self.texts.contains_key(&shape_id)
    }

    /// Clears all texts (for testing)
    #[cfg(test)]
    pub fn clear(&mut self) {
        self.texts.clear();
        self.active_edit = None;
    }
}

/// Command to create a text entity
#[derive(Clone, Debug)]
pub struct CreateTextCommand {
    position: Vec2,
    content: String,
    style: TextStyle,
    created_shape_id: Option<EntityId>,
    executed: bool,
}

impl CreateTextCommand {
    /// Creates a new create text command
    pub fn new(position: Vec2, content: impl Into<String>, style: TextStyle) -> Self {
        Self {
            position,
            content: content.into(),
            style,
            created_shape_id: None,
            executed: false,
        }
    }

    /// Gets the created shape ID (if executed)
    pub fn shape_id(&self) -> Option<EntityId> {
        self.created_shape_id
    }
}

impl Command for CreateTextCommand {
    fn execute(&mut self, canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>> {
        let shape_id = canvas.create_text(self.position.x, self.position.y, 100.0, 20.0);
        self.created_shape_id = Some(shape_id);
        self.executed = true;
        Ok(None)
    }

    fn undo(&mut self, _canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>> {
        if let Some(_id) = self.created_shape_id {
            // In a real implementation, we'd remove the shape from canvas
            // For now, we just mark as not executed
        }
        self.executed = false;
        Ok(None)
    }

    fn description(&self) -> &str {
        "Create text"
    }
}

/// Command to update text content
#[derive(Clone, Debug)]
pub struct UpdateTextContentCommand {
    shape_id: EntityId,
    new_content: String,
    old_content: String,
    executed: bool,
}

impl UpdateTextContentCommand {
    /// Creates a new update text content command
    pub fn new(shape_id: EntityId, new_content: impl Into<String>) -> Self {
        Self {
            shape_id,
            new_content: new_content.into(),
            old_content: String::new(),
            executed: false,
        }
    }
}

impl Command for UpdateTextContentCommand {
    fn execute(&mut self, _canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>> {
        // Store old content would require access to TextManager
        // In real implementation, this would update the text entity
        self.executed = true;
        Ok(None)
    }

    fn undo(&mut self, _canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>> {
        // Restore old content would require access to TextManager
        self.executed = false;
        Ok(None)
    }

    fn description(&self) -> &str {
        "Update text content"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_style_default() {
        let style = TextStyle::default();
        assert_eq!(style.font_family, "Arial");
        assert_eq!(style.font_size, 16.0);
        assert_eq!(style.font_weight, 400);
        assert!(!style.italic);
    }

    #[test]
    fn test_text_style_builder() {
        let style = TextStyle::new()
            .with_font_family("Helvetica")
            .with_font_size(24.0)
            .with_font_weight(700)
            .with_italic(true)
            .with_color(Color::rgb(1.0, 0.0, 0.0))
            .with_alignment(TextAlignment::Center);

        assert_eq!(style.font_family, "Helvetica");
        assert_eq!(style.font_size, 24.0);
        assert_eq!(style.font_weight, 700);
        assert!(style.italic);
        assert_eq!(style.color, Color::rgb(1.0, 0.0, 0.0));
        assert_eq!(style.alignment, TextAlignment::Center);
    }

    #[test]
    fn test_text_entity_creation() {
        let shape_id = EntityId::new();
        let text = TextEntity::new(shape_id, "Hello, World!", TextStyle::default());

        assert_eq!(text.content, "Hello, World!");
        assert!(!text.edit_mode);
        assert_eq!(text.cursor_position, 0);
    }

    #[test]
    fn test_text_entity_edit_mode() {
        let shape_id = EntityId::new();
        let mut text = TextEntity::new(shape_id, "Hello", TextStyle::default());

        text.enter_edit_mode().unwrap();
        assert!(text.edit_mode);
        assert_eq!(text.cursor_position, 5);

        text.exit_edit_mode().unwrap();
        assert!(!text.edit_mode);
        assert_eq!(text.cursor_position, 0);
    }

    #[test]
    fn test_text_entity_edit_mode_already_in_edit() {
        let shape_id = EntityId::new();
        let mut text = TextEntity::new(shape_id, "Hello", TextStyle::default());

        text.enter_edit_mode().unwrap();
        let result = text.enter_edit_mode();
        assert!(matches!(result, Err(TextError::AlreadyInEditMode)));
    }

    #[test]
    fn test_text_entity_insert_text() {
        let shape_id = EntityId::new();
        let mut text = TextEntity::new(shape_id, "Hello", TextStyle::default());
        text.enter_edit_mode().unwrap();

        text.insert_text(" World");
        assert_eq!(text.content, "Hello World");
        assert_eq!(text.cursor_position, 11);
    }

    #[test]
    fn test_text_entity_backspace() {
        let shape_id = EntityId::new();
        let mut text = TextEntity::new(shape_id, "Hello", TextStyle::default());
        text.enter_edit_mode().unwrap();

        text.backspace();
        assert_eq!(text.content, "Hell");
        assert_eq!(text.cursor_position, 4);
    }

    #[test]
    fn test_text_entity_move_cursor() {
        let shape_id = EntityId::new();
        let mut text = TextEntity::new(shape_id, "Hello", TextStyle::default());
        text.enter_edit_mode().unwrap();

        text.move_cursor_left();
        assert_eq!(text.cursor_position, 4);

        text.move_cursor_right();
        assert_eq!(text.cursor_position, 5);
    }

    #[test]
    fn test_text_manager_creation() {
        let manager = TextManager::new();
        assert_eq!(manager.text_count(), 0);
        assert!(!manager.is_editing());
    }

    #[test]
    fn test_text_manager_create_text() {
        let mut manager = TextManager::new();
        let mut canvas = Canvas::new(800.0, 600.0);

        let shape_id = manager.create_text(&mut canvas, 100.0, 200.0, "Test text", None);

        assert_eq!(manager.text_count(), 1);
        assert!(manager.is_text(shape_id));

        let text = manager.get_text(shape_id).unwrap();
        assert_eq!(text.content, "Test text");
    }

    #[test]
    fn test_text_manager_edit_mode() {
        let mut manager = TextManager::new();
        let mut canvas = Canvas::new(800.0, 600.0);

        let shape_id = manager.create_text(&mut canvas, 100.0, 200.0, "Test", None);

        manager.enter_edit_mode(shape_id).unwrap();
        assert!(manager.is_editing());
        assert_eq!(manager.active_edit(), Some(shape_id));

        let text = manager.get_text(shape_id).unwrap();
        assert!(text.is_editing());
    }

    #[test]
    fn test_text_manager_exit_edit_mode() {
        let mut manager = TextManager::new();
        let mut canvas = Canvas::new(800.0, 600.0);

        let shape_id = manager.create_text(&mut canvas, 100.0, 200.0, "Test", None);

        manager.enter_edit_mode(shape_id).unwrap();
        manager.exit_edit_mode().unwrap();

        assert!(!manager.is_editing());
        assert_eq!(manager.active_edit(), None);
    }

    #[test]
    fn test_text_manager_switch_edit_mode() {
        let mut manager = TextManager::new();
        let mut canvas = Canvas::new(800.0, 600.0);

        let id1 = manager.create_text(&mut canvas, 100.0, 100.0, "Text 1", None);
        let id2 = manager.create_text(&mut canvas, 100.0, 200.0, "Text 2", None);

        manager.enter_edit_mode(id1).unwrap();
        assert_eq!(manager.active_edit(), Some(id1));

        manager.enter_edit_mode(id2).unwrap();
        assert_eq!(manager.active_edit(), Some(id2));

        // First text should have exited edit mode
        let text1 = manager.get_text(id1).unwrap();
        assert!(!text1.is_editing());
    }

    #[test]
    fn test_text_manager_remove_text() {
        let mut manager = TextManager::new();
        let mut canvas = Canvas::new(800.0, 600.0);

        let shape_id = manager.create_text(&mut canvas, 100.0, 200.0, "Test", None);
        manager.enter_edit_mode(shape_id).unwrap();

        let removed = manager.remove_text(shape_id);
        assert!(removed.is_some());
        assert_eq!(manager.text_count(), 0);
        assert!(!manager.is_editing());
        assert!(!manager.is_text(shape_id));
    }

    #[test]
    fn test_text_manager_set_content() {
        let mut manager = TextManager::new();
        let mut canvas = Canvas::new(800.0, 600.0);

        let shape_id = manager.create_text(&mut canvas, 100.0, 200.0, "Old content", None);
        manager.set_content(shape_id, "New content").unwrap();

        let text = manager.get_text(shape_id).unwrap();
        assert_eq!(text.content, "New content");
    }

    #[test]
    fn test_text_manager_set_style() {
        let mut manager = TextManager::new();
        let mut canvas = Canvas::new(800.0, 600.0);

        let shape_id = manager.create_text(&mut canvas, 100.0, 200.0, "Test", None);
        let new_style = TextStyle::new().with_font_size(24.0);

        manager.set_style(shape_id, new_style.clone()).unwrap();

        let text = manager.get_text(shape_id).unwrap();
        assert_eq!(text.style.font_size, 24.0);
    }

    #[test]
    fn test_text_manager_not_found() {
        let mut manager = TextManager::new();
        let nonexistent_id = EntityId::new();

        let result = manager.enter_edit_mode(nonexistent_id);
        assert!(matches!(result, Err(TextError::TextNotFound(_))));
    }

    #[test]
    fn test_text_alignment_variants() {
        assert_ne!(TextAlignment::Left, TextAlignment::Center);
        assert_ne!(TextAlignment::Center, TextAlignment::Right);
        assert_ne!(TextAlignment::Right, TextAlignment::Justify);
    }

    #[test]
    fn test_text_approximate_dimensions() {
        let shape_id = EntityId::new();
        let text = TextEntity::new(shape_id, "Hello", TextStyle::default());

        let width = text.approximate_width();
        let height = text.approximate_height();

        assert!(width > 0.0);
        assert!(height > 0.0);

        // Longer text should be wider
        let long_text = TextEntity::new(shape_id, "Hello World This Is Long", TextStyle::default());
        assert!(long_text.approximate_width() > width);
    }

    #[test]
    fn test_create_text_command() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let mut cmd = CreateTextCommand::new(Vec2::new(100.0, 200.0), "Test", TextStyle::default());

        cmd.execute(&mut canvas).unwrap();
        assert!(cmd.shape_id().is_some());
        assert!(cmd.executed);
    }

    #[test]
    fn test_font_size_validation() {
        let style = TextStyle::new().with_font_size(-10.0);
        // Font size should be clamped to minimum
        assert_eq!(style.font_size, 1.0);
    }

    #[test]
    fn test_font_weight_validation() {
        let style = TextStyle::new().with_font_weight(50);
        assert_eq!(style.font_weight, 100);

        let style = TextStyle::new().with_font_weight(1000);
        assert_eq!(style.font_weight, 900);
    }
}
