//! Clipboard Manager - Copy/Paste operations for shapes

use crate::canvas::{Canvas, Shape, ShapeChanges};
use crate::commands::{Command, CommandError, CommandResult};
use archflow_core::{EntityId, Vec2};
use serde::{Deserialize, Serialize};

/// Clipboard content data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardData {
    /// Version for future compatibility
    pub version: u32,
    /// Serialized entities
    pub entities: Vec<SerializedEntity>,
    /// Number of entities
    pub entity_count: usize,
    /// Creation timestamp
    pub timestamp: u64,
}

/// A serialized entity for clipboard storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedEntity {
    /// Original ID
    pub original_id: EntityId,
    /// New ID (generated on paste)
    pub new_id: Option<EntityId>,
    /// Position
    pub position: Vec2,
    /// Size
    pub size: Vec2,
    /// Rotation in degrees
    pub rotation: f32,
    /// Fill color as RGBA
    pub fill_color: [f32; 4],
    /// Stroke color as RGBA (None = no stroke)
    pub stroke_color: Option<[f32; 4]>,
    /// Stroke width
    pub stroke_width: f32,
    /// Opacity
    pub opacity: f32,
}

impl SerializedEntity {
    pub fn from_shape(shape: &Shape) -> Self {
        Self {
            original_id: shape.id,
            new_id: None,
            position: Vec2::new(shape.x, shape.y),
            size: Vec2::new(shape.width, shape.height),
            rotation: shape.rotation,
            fill_color: [
                shape.fill_color.r,
                shape.fill_color.g,
                shape.fill_color.b,
                shape.fill_color.a,
            ],
            stroke_color: shape.stroke_color.map(|c| [c.r, c.g, c.b, c.a]),
            stroke_width: shape.stroke_width,
            opacity: shape.opacity,
        }
    }
}

/// Result of a paste operation
#[derive(Debug, Clone)]
pub struct PasteResult {
    /// New entity IDs created
    pub new_ids: Vec<EntityId>,
}

/// Clipboard manager
#[derive(Debug, Clone)]
pub struct ClipboardManager {
    /// Current clipboard data
    clipboard: Option<ClipboardData>,
    /// Default paste offset
    default_offset: Vec2,
}

impl ClipboardManager {
    pub fn new() -> Self {
        Self {
            clipboard: None,
            default_offset: Vec2::new(20.0, 20.0),
        }
    }

    pub fn set_default_offset(&mut self, offset: Vec2) {
        self.default_offset = offset;
    }

    /// Copy entities to clipboard
    pub fn copy(&mut self, canvas: &Canvas, entity_ids: &[EntityId]) -> CommandResult<PasteResult> {
        let mut entities = Vec::new();

        for id in entity_ids {
            if let Some(shape) = canvas.get_shape(*id) {
                entities.push(SerializedEntity::from_shape(shape));
            }
        }

        let entity_count = entities.len();
        let data = ClipboardData {
            version: 1,
            entities,
            entity_count,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        self.clipboard = Some(data);

        Ok(PasteResult {
            new_ids: Vec::new(),
        })
    }

    /// Paste from clipboard
    pub fn paste(&mut self, canvas: &mut Canvas) -> CommandResult<PasteResult> {
        let data = self
            .clipboard
            .as_ref()
            .ok_or_else(|| CommandError::ExecutionFailed("Clipboard is empty".to_string()))?;

        let paste_offset = self.default_offset;
        let mut new_ids = Vec::new();

        for entity in &data.entities {
            let new_id = canvas.create_rectangle(
                entity.position.x + paste_offset.x,
                entity.position.y + paste_offset.y,
                entity.size.x,
                entity.size.y,
            );
            new_ids.push(new_id);
        }

        Ok(PasteResult { new_ids })
    }

    /// Get the number of entities in clipboard
    pub fn len(&self) -> usize {
        self.clipboard.as_ref().map(|d| d.entity_count).unwrap_or(0)
    }

    /// Check if clipboard is empty
    pub fn is_empty(&self) -> bool {
        self.clipboard.is_none()
    }

    /// Clear the clipboard
    pub fn clear(&mut self) {
        self.clipboard = None;
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_to_clipboard() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);

        let mut manager = ClipboardManager::new();
        let result = manager.copy(&canvas, &[id]).unwrap();

        assert!(!manager.is_empty());
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_paste_from_clipboard() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);

        let initial_count = canvas.all_shapes().len();

        let mut manager = ClipboardManager::new();
        manager.copy(&canvas, &[id]).unwrap();
        let result = manager.paste(&mut canvas).unwrap();

        assert_eq!(canvas.all_shapes().len(), initial_count + 1);
        assert_eq!(result.new_ids.len(), 1);
    }

    #[test]
    fn test_clipboard_clear() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);

        let mut manager = ClipboardManager::new();
        manager.copy(&canvas, &[id]).unwrap();
        assert!(!manager.is_empty());

        manager.clear();
        assert!(manager.is_empty());
    }
}
