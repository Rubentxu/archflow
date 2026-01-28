//! Transform Commands - Commands for resize, rotate, and duplicate operations
//!
//! This module provides commands for:
//! - **ResizeShapeCommand**: Resize a shape
//! - **RotateShapeCommand**: Rotate a shape
//! - **DuplicateShapeCommand**: Duplicate shapes
//!
//! # Merge Support
//!
//! Commands support merging for continuous operations like drag-resize.

use crate::canvas::{Canvas, Shape, ShapeChanges};
use crate::commands::{Command, CommandError, CommandResult};
use crate::selection::SelectionDelta;
use archflow_core::{EntityId, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Command to resize a shape
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResizeShapeCommand {
    /// Shape ID
    shape_id: EntityId,
    /// Original bounds (min, max)
    original_bounds: (Vec2, Vec2),
    /// New bounds (min, max)
    new_bounds: (Vec2, Vec2),
    /// Whether the command has been executed
    executed: bool,
}

impl ResizeShapeCommand {
    /// Create a new resize command
    pub fn new(
        shape_id: EntityId,
        original_bounds: (Vec2, Vec2),
        new_bounds: (Vec2, Vec2),
    ) -> Self {
        Self {
            shape_id,
            original_bounds,
            new_bounds,
            executed: false,
        }
    }

    /// Get the shape ID
    pub fn shape_id(&self) -> EntityId {
        self.shape_id
    }

    /// Get original bounds
    pub fn original_bounds(&self) -> (Vec2, Vec2) {
        self.original_bounds
    }

    /// Get new bounds
    pub fn new_bounds(&self) -> (Vec2, Vec2) {
        self.new_bounds
    }

    /// Update the target bounds (for continuous operations)
    pub fn update_bounds(&mut self, new_bounds: (Vec2, Vec2)) {
        self.new_bounds = new_bounds;
    }
}

impl Command for ResizeShapeCommand {
    fn execute(&mut self, canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>> {
        let _shape = canvas
            .get_shape(self.shape_id)
            .ok_or_else(|| CommandError::ExecutionFailed("Shape not found".to_string()))?;

        let width = self.new_bounds.1.x - self.new_bounds.0.x;
        let height = self.new_bounds.1.y - self.new_bounds.0.y;
        let x = self.new_bounds.0.x;
        let y = self.new_bounds.0.y;

        let changes = ShapeChanges {
            x: Some(x),
            y: Some(y),
            width: Some(width),
            height: Some(height),
            rotation: None,
            fill_color: None,
            stroke_color: None,
            stroke_width: None,
            opacity: None,
        };

        canvas.update_shape(self.shape_id, changes);
        self.executed = true;

        Ok(None)
    }

    fn undo(&mut self, canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>> {
        let width = self.original_bounds.1.x - self.original_bounds.0.x;
        let height = self.original_bounds.1.y - self.original_bounds.0.y;
        let x = self.original_bounds.0.x;
        let y = self.original_bounds.0.y;

        let changes = ShapeChanges {
            x: Some(x),
            y: Some(y),
            width: Some(width),
            height: Some(height),
            rotation: None,
            fill_color: None,
            stroke_color: None,
            stroke_width: None,
            opacity: None,
        };

        canvas.update_shape(self.shape_id, changes);
        self.executed = false;

        Ok(None)
    }

    fn description(&self) -> &str {
        "Resize shape"
    }

    /// Merge with another resize command (for continuous drag operations)
    fn merge(&mut self, _other: &dyn Command) -> bool {
        // Note: Full merge support requires concrete type matching
        // which is not possible with dyn Command without RTTI
        false
    }
}

/// Command to rotate a shape
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotateShapeCommand {
    /// Shape ID
    shape_id: EntityId,
    /// Original rotation in degrees
    original_angle: f32,
    /// New rotation in degrees
    new_angle: f32,
    /// Center of rotation (for display purposes)
    center: Vec2,
    /// Whether the command has been executed
    executed: bool,
}

impl RotateShapeCommand {
    /// Create a new rotate command
    pub fn new(shape_id: EntityId, original_angle: f32, new_angle: f32, center: Vec2) -> Self {
        Self {
            shape_id,
            original_angle,
            new_angle,
            center,
            executed: false,
        }
    }

    /// Get the shape ID
    pub fn shape_id(&self) -> EntityId {
        self.shape_id
    }

    /// Get original angle
    pub fn original_angle(&self) -> f32 {
        self.original_angle
    }

    /// Get new angle
    pub fn new_angle(&self) -> f32 {
        self.new_angle
    }

    /// Update the target angle (for continuous operations)
    pub fn update_angle(&mut self, angle: f32) {
        self.new_angle = angle;
    }
}

impl Command for RotateShapeCommand {
    fn execute(&mut self, canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>> {
        let changes = ShapeChanges {
            x: None,
            y: None,
            width: None,
            height: None,
            rotation: Some(self.new_angle),
            fill_color: None,
            stroke_color: None,
            stroke_width: None,
            opacity: None,
        };

        canvas.update_shape(self.shape_id, changes);
        self.executed = true;

        Ok(None)
    }

    fn undo(&mut self, canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>> {
        let changes = ShapeChanges {
            x: None,
            y: None,
            width: None,
            height: None,
            rotation: Some(self.original_angle),
            fill_color: None,
            stroke_color: None,
            stroke_width: None,
            opacity: None,
        };

        canvas.update_shape(self.shape_id, changes);
        self.executed = false;

        Ok(None)
    }

    fn description(&self) -> &str {
        "Rotate shape"
    }

    /// Merge with another rotate command
    fn merge(&mut self, _other: &dyn Command) -> bool {
        // Note: Full merge support requires concrete type matching
        false
    }
}

/// Command to duplicate shapes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateShapeCommand {
    /// Source shape IDs
    source_ids: Vec<EntityId>,
    /// New shape IDs (created during execution)
    new_ids: Vec<EntityId>,
    /// Offset applied to each duplicate
    offset: Vec2,
    /// Original shape data for undo
    original_data: Vec<ShapeData>,
    /// Whether the command has been executed
    executed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ShapeData {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    rotation: f32,
    fill_color: archflow_core::Color,
    stroke_color: Option<archflow_core::Color>,
    stroke_width: f32,
    opacity: f32,
    shape_type: String,
}

impl DuplicateShapeCommand {
    /// Create a new duplicate command
    pub fn new(source_ids: &[EntityId], offset: Vec2) -> Self {
        Self {
            source_ids: source_ids.to_vec(),
            new_ids: Vec::new(),
            offset,
            original_data: Vec::new(),
            executed: false,
        }
    }

    /// Get the new shape IDs
    pub fn new_ids(&self) -> &[EntityId] {
        &self.new_ids
    }
}

impl Command for DuplicateShapeCommand {
    fn execute(&mut self, canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>> {
        self.original_data.clear();
        self.new_ids.clear();

        for source_id in &self.source_ids {
            if let Some(shape) = canvas.get_shape(*source_id) {
                // Capture original data
                let data = ShapeData {
                    x: shape.x,
                    y: shape.y,
                    width: shape.width,
                    height: shape.height,
                    rotation: shape.rotation,
                    fill_color: shape.fill_color,
                    stroke_color: shape.stroke_color,
                    stroke_width: shape.stroke_width,
                    opacity: shape.opacity,
                    shape_type: shape.shape_type.to_string(),
                };
                self.original_data.push(data);

                // Create duplicate with offset
                let new_id = canvas.create_rectangle(
                    shape.x + self.offset.x,
                    shape.y + self.offset.y,
                    shape.width,
                    shape.height,
                );
                self.new_ids.push(new_id);
            }
        }

        self.executed = true;

        Ok(None)
    }

    fn undo(&mut self, canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>> {
        // Delete the duplicated shapes
        for new_id in &self.new_ids {
            canvas.delete_shape(*new_id);
        }
        self.new_ids.clear();
        self.executed = false;

        Ok(None)
    }

    fn description(&self) -> &str {
        "Duplicate shape"
    }
}

/// Batch transform command for multiple shapes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchTransformCommand {
    /// Entity ID to original bounds mapping
    transforms: HashMap<EntityId, (Vec2, Vec2, f32)>, // (min, max, rotation)
    /// Target bounds and rotation for each entity
    targets: HashMap<EntityId, (Vec2, Vec2, f32)>,
    /// Whether the command has been executed
    executed: bool,
}

impl BatchTransformCommand {
    /// Create a new batch transform command
    pub fn new(
        transforms: HashMap<EntityId, (Vec2, Vec2, f32)>,
        targets: HashMap<EntityId, (Vec2, Vec2, f32)>,
    ) -> Self {
        Self {
            transforms,
            targets,
            executed: false,
        }
    }
}

impl Command for BatchTransformCommand {
    fn execute(&mut self, canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>> {
        for (id, (min, max, rotation)) in &self.targets {
            let width = max.x - min.x;
            let height = max.y - min.y;

            let changes = ShapeChanges {
                x: Some(min.x),
                y: Some(min.y),
                width: Some(width),
                height: Some(height),
                rotation: Some(*rotation),
                fill_color: None,
                stroke_color: None,
                stroke_width: None,
                opacity: None,
            };

            canvas.update_shape(*id, changes);
        }

        self.executed = true;
        Ok(None)
    }

    fn undo(&mut self, canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>> {
        for (id, (min, max, rotation)) in &self.transforms {
            let width = max.x - min.x;
            let height = max.y - min.y;

            let changes = ShapeChanges {
                x: Some(min.x),
                y: Some(min.y),
                width: Some(width),
                height: Some(height),
                rotation: Some(*rotation),
                fill_color: None,
                stroke_color: None,
                stroke_width: None,
                opacity: None,
            };

            canvas.update_shape(*id, changes);
        }

        self.executed = false;
        Ok(None)
    }

    fn description(&self) -> &str {
        "Batch transform"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resize_command_execute() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);

        let mut command = ResizeShapeCommand::new(
            id,
            (Vec2::new(100.0, 100.0), Vec2::new(150.0, 150.0)),
            (Vec2::new(100.0, 100.0), Vec2::new(200.0, 200.0)),
        );

        let result = command.execute(&mut canvas);
        assert!(result.is_ok());

        let shape = canvas.get_shape(id).unwrap();
        assert_eq!(shape.width, 100.0);
        assert_eq!(shape.height, 100.0);
    }

    #[test]
    fn test_resize_command_undo() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);

        let mut command = ResizeShapeCommand::new(
            id,
            (Vec2::new(100.0, 100.0), Vec2::new(150.0, 150.0)),
            (Vec2::new(100.0, 100.0), Vec2::new(200.0, 200.0)),
        );

        command.execute(&mut canvas).unwrap();
        command.undo(&mut canvas).unwrap();

        let shape = canvas.get_shape(id).unwrap();
        assert_eq!(shape.width, 50.0);
        assert_eq!(shape.height, 50.0);
    }

    #[test]
    fn test_resize_commands_merge_not_supported() {
        // Note: merge with dyn Command is not supported without RTTI
        // This test verifies the default merge behavior returns false
        let mut command1 = ResizeShapeCommand::new(
            EntityId::new(),
            (Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0)),
            (Vec2::new(0.0, 0.0), Vec2::new(150.0, 150.0)),
        );

        let command2 = ResizeShapeCommand::new(
            command1.shape_id(),
            (Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0)),
            (Vec2::new(0.0, 0.0), Vec2::new(200.0, 200.0)),
        );

        // Merge with dyn Command returns false (no RTTI for downcasting)
        let merged = command1.merge(&command2 as &dyn Command);
        assert!(!merged);
    }

    #[test]
    fn test_rotate_command_execute() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);

        let mut command = RotateShapeCommand::new(id, 0.0, 45.0, Vec2::new(125.0, 125.0));

        command.execute(&mut canvas).unwrap();

        let shape = canvas.get_shape(id).unwrap();
        assert_eq!(shape.rotation, 45.0);
    }

    #[test]
    fn test_rotate_command_undo() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);

        let mut command = RotateShapeCommand::new(id, 0.0, 45.0, Vec2::new(125.0, 125.0));

        command.execute(&mut canvas).unwrap();
        command.undo(&mut canvas).unwrap();

        let shape = canvas.get_shape(id).unwrap();
        assert_eq!(shape.rotation, 0.0);
    }

    #[test]
    fn test_rotate_commands_merge_not_supported() {
        // Note: merge with dyn Command is not supported without RTTI
        let mut command1 = RotateShapeCommand::new(EntityId::new(), 0.0, 45.0, Vec2::ZERO);
        let command2 = RotateShapeCommand::new(command1.shape_id(), 0.0, 90.0, Vec2::ZERO);

        // Merge with dyn Command returns false (no RTTI for downcasting)
        let merged = command1.merge(&command2 as &dyn Command);
        assert!(!merged);
    }

    #[test]
    fn test_duplicate_command_creates_new_entity() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);

        let initial_count = canvas.all_shapes().len();
        let mut command = DuplicateShapeCommand::new(&[id], Vec2::new(20.0, 20.0));

        command.execute(&mut canvas).unwrap();

        assert_eq!(canvas.all_shapes().len(), initial_count + 1);
    }

    #[test]
    fn test_duplicate_command_applies_offset() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);

        let mut command = DuplicateShapeCommand::new(&[id], Vec2::new(20.0, 20.0));
        command.execute(&mut canvas).unwrap();

        // Get the new shape ID from the command
        let new_id = command.new_ids()[0];
        let new_shape = canvas.get_shape(new_id).unwrap();
        assert_eq!(new_shape.x, 120.0); // 100 + 20
        assert_eq!(new_shape.y, 120.0); // 100 + 20
    }

    #[test]
    fn test_duplicate_command_undo() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);

        let mut command = DuplicateShapeCommand::new(&[id], Vec2::new(20.0, 20.0));

        let initial_count = canvas.all_shapes().len();
        command.execute(&mut canvas).unwrap();
        assert_eq!(canvas.all_shapes().len(), initial_count + 1);

        command.undo(&mut canvas).unwrap();
        assert_eq!(canvas.all_shapes().len(), initial_count);
    }

    #[test]
    fn test_duplicate_multiple_entities() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id1 = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);
        let id2 = canvas.create_rectangle(200.0, 200.0, 50.0, 50.0);

        let mut command = DuplicateShapeCommand::new(&[id1, id2], Vec2::new(20.0, 20.0));

        command.execute(&mut canvas).unwrap();

        // Should have 2 new shapes
        assert_eq!(canvas.all_shapes().len(), 4);
    }

    #[test]
    fn test_batch_transform_execute() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id1 = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);
        let id2 = canvas.create_rectangle(200.0, 200.0, 50.0, 50.0);

        let mut original = HashMap::new();
        original.insert(id1, (Vec2::new(100.0, 100.0), Vec2::new(150.0, 150.0), 0.0));
        original.insert(id2, (Vec2::new(200.0, 200.0), Vec2::new(250.0, 250.0), 0.0));

        let mut targets = HashMap::new();
        targets.insert(
            id1,
            (Vec2::new(110.0, 110.0), Vec2::new(160.0, 160.0), 15.0),
        );
        targets.insert(
            id2,
            (Vec2::new(210.0, 210.0), Vec2::new(260.0, 260.0), 30.0),
        );

        let mut command = BatchTransformCommand::new(original, targets);
        command.execute(&mut canvas).unwrap();

        let shape1 = canvas.get_shape(id1).unwrap();
        let shape2 = canvas.get_shape(id2).unwrap();

        assert_eq!(shape1.x, 110.0);
        assert_eq!(shape1.y, 110.0);
        assert_eq!(shape1.width, 50.0);
        assert_eq!(shape1.height, 50.0);
        assert_eq!(shape1.rotation, 15.0);

        assert_eq!(shape2.x, 210.0);
        assert_eq!(shape2.y, 210.0);
        assert_eq!(shape2.rotation, 30.0);
    }

    #[test]
    fn test_batch_transform_undo() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);

        let mut original = HashMap::new();
        original.insert(id, (Vec2::new(100.0, 100.0), Vec2::new(150.0, 150.0), 0.0));

        let mut targets = HashMap::new();
        targets.insert(id, (Vec2::new(200.0, 200.0), Vec2::new(250.0, 250.0), 45.0));

        let mut command = BatchTransformCommand::new(original, targets);
        command.execute(&mut canvas).unwrap();

        // Verify changed state
        let shape = canvas.get_shape(id).unwrap();
        assert_eq!(shape.x, 200.0);
        assert_eq!(shape.rotation, 45.0);

        // Undo
        command.undo(&mut canvas).unwrap();

        // Verify original state restored
        let shape = canvas.get_shape(id).unwrap();
        assert_eq!(shape.x, 100.0);
        assert_eq!(shape.rotation, 0.0);
    }

    #[test]
    fn test_batch_transform_multiple_entities() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id1 = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);
        let id2 = canvas.create_rectangle(300.0, 300.0, 50.0, 50.0);
        let id3 = canvas.create_rectangle(500.0, 500.0, 50.0, 50.0);

        let mut original = HashMap::new();
        original.insert(id1, (Vec2::new(100.0, 100.0), Vec2::new(150.0, 150.0), 0.0));
        original.insert(id2, (Vec2::new(300.0, 300.0), Vec2::new(350.0, 350.0), 0.0));
        original.insert(id3, (Vec2::new(500.0, 500.0), Vec2::new(550.0, 550.0), 0.0));

        let mut targets = HashMap::new();
        targets.insert(
            id1,
            (Vec2::new(110.0, 110.0), Vec2::new(160.0, 160.0), 10.0),
        );
        targets.insert(
            id2,
            (Vec2::new(310.0, 310.0), Vec2::new(360.0, 360.0), 20.0),
        );
        targets.insert(
            id3,
            (Vec2::new(510.0, 510.0), Vec2::new(560.0, 560.0), 30.0),
        );

        let mut command = BatchTransformCommand::new(original, targets);
        command.execute(&mut canvas).unwrap();

        assert_eq!(canvas.get_shape(id1).unwrap().rotation, 10.0);
        assert_eq!(canvas.get_shape(id2).unwrap().rotation, 20.0);
        assert_eq!(canvas.get_shape(id3).unwrap().rotation, 30.0);
    }
}
