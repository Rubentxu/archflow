//! Clipboard Manager - Copy/Paste operations for shapes

use crate::canvas::{Canvas, Shape, ShapeChanges};
use crate::commands::{CommandError, CommandResult};
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

            // Restore all entity properties
            let fill_color = archflow_core::Color::rgba(
                entity.fill_color[0],
                entity.fill_color[1],
                entity.fill_color[2],
                entity.fill_color[3],
            );

            let stroke_color = entity
                .stroke_color
                .map(|c| archflow_core::Color::rgba(c[0], c[1], c[2], c[3]));

            let changes = ShapeChanges {
                x: None,
                y: None,
                width: None,
                height: None,
                rotation: Some(entity.rotation),
                fill_color: Some(fill_color),
                stroke_color: Some(stroke_color),
                stroke_width: Some(entity.stroke_width),
                opacity: Some(entity.opacity),
            };

            canvas.update_shape(new_id, changes);
            new_ids.push(new_id);
        }

        Ok(PasteResult { new_ids })
    }

    /// Cut entities to clipboard (copy + delete)
    pub fn cut(
        &mut self,
        canvas: &mut Canvas,
        entity_ids: &[EntityId],
    ) -> CommandResult<PasteResult> {
        // First copy to clipboard
        let result = self.copy(canvas, entity_ids)?;

        // Then delete the original entities
        for id in entity_ids {
            canvas.delete_shape(*id);
        }

        Ok(result)
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

    #[test]
    fn test_copy_multiple_entities() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id1 = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);
        let id2 = canvas.create_rectangle(200.0, 200.0, 60.0, 60.0);
        let id3 = canvas.create_rectangle(300.0, 300.0, 70.0, 70.0);

        let mut manager = ClipboardManager::new();
        manager.copy(&canvas, &[id1, id2, id3]).unwrap();

        assert_eq!(manager.len(), 3);
    }

    #[test]
    fn test_copy_includes_metadata() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);

        // Modify shape properties
        let changes = ShapeChanges {
            x: None,
            y: None,
            width: None,
            height: None,
            rotation: Some(45.0),
            fill_color: Some(archflow_core::Color::rgba(1.0, 0.0, 0.0, 1.0)),
            stroke_color: Some(Some(archflow_core::Color::rgba(0.0, 0.0, 0.0, 1.0))),
            stroke_width: Some(2.0),
            opacity: Some(0.8),
        };
        canvas.update_shape(id, changes);

        let mut manager = ClipboardManager::new();
        manager.copy(&canvas, &[id]).unwrap();

        let data = manager.clipboard.as_ref().unwrap();
        assert_eq!(data.entity_count, 1);
        assert_eq!(data.version, 1);
        assert!(data.timestamp > 0);

        let entity = &data.entities[0];
        assert_eq!(entity.rotation, 45.0);
        assert_eq!(entity.fill_color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(entity.stroke_color, Some([0.0, 0.0, 0.0, 1.0]));
        assert_eq!(entity.stroke_width, 2.0);
        assert_eq!(entity.opacity, 0.8);
    }

    #[test]
    fn test_paste_generates_new_ids() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let original_id = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);

        let mut manager = ClipboardManager::new();
        manager.copy(&canvas, &[original_id]).unwrap();
        let result = manager.paste(&mut canvas).unwrap();

        // Should have 2 shapes now
        assert_eq!(canvas.all_shapes().len(), 2);

        // New ID should be different from original
        let new_id = result.new_ids[0];
        assert_ne!(new_id, original_id);

        // Both shapes should exist
        assert!(canvas.get_shape(original_id).is_some());
        assert!(canvas.get_shape(new_id).is_some());
    }

    #[test]
    fn test_paste_multiple_times() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);

        let mut manager = ClipboardManager::new();
        manager.copy(&canvas, &[id]).unwrap();

        // Paste multiple times
        let result1 = manager.paste(&mut canvas).unwrap();
        let result2 = manager.paste(&mut canvas).unwrap();
        let result3 = manager.paste(&mut canvas).unwrap();

        // Should have 4 shapes now (1 original + 3 pasted)
        assert_eq!(canvas.all_shapes().len(), 4);

        // Each paste should generate unique IDs
        let ids: std::collections::HashSet<_> = result1
            .new_ids
            .iter()
            .chain(result2.new_ids.iter())
            .chain(result3.new_ids.iter())
            .collect();
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn test_cut_to_clipboard() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);

        let initial_count = canvas.all_shapes().len();

        let mut manager = ClipboardManager::new();
        manager.cut(&mut canvas, &[id]).unwrap();

        // Clipboard should have the entity
        assert_eq!(manager.len(), 1);

        // Entity should be deleted from canvas
        assert!(canvas.get_shape(id).is_none());
        assert_eq!(canvas.all_shapes().len(), initial_count - 1);
    }

    #[test]
    fn test_paste_restores_all_properties() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);

        // Set all properties
        let changes = ShapeChanges {
            x: None,
            y: None,
            width: None,
            height: None,
            rotation: Some(30.0),
            fill_color: Some(archflow_core::Color::rgba(0.2, 0.4, 0.6, 0.8)),
            stroke_color: Some(Some(archflow_core::Color::rgba(0.1, 0.2, 0.3, 0.9))),
            stroke_width: Some(3.5),
            opacity: Some(0.75),
        };
        canvas.update_shape(id, changes);

        let mut manager = ClipboardManager::new();
        manager.copy(&canvas, &[id]).unwrap();
        let result = manager.paste(&mut canvas).unwrap();

        let new_id = result.new_ids[0];
        let pasted_shape = canvas.get_shape(new_id).unwrap();

        assert_eq!(pasted_shape.rotation, 30.0);
        assert_eq!(pasted_shape.fill_color.r, 0.2);
        assert_eq!(pasted_shape.fill_color.g, 0.4);
        assert_eq!(pasted_shape.fill_color.b, 0.6);
        assert_eq!(pasted_shape.fill_color.a, 0.8);
        assert!(pasted_shape.stroke_color.is_some());
        assert_eq!(pasted_shape.stroke_width, 3.5);
        assert_eq!(pasted_shape.opacity, 0.75);
    }
}
