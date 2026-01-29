//! Command Pattern for Undo/Redo Operations
//!
//! This module implements the Command Pattern for tool operations, providing
//! a robust undo/redo system that integrates with the SDK's Canvas API.
//!
//! # Architecture
//!
//! - **Command**: Trait representing an executable operation
//! - **CommandExecutor**: Manages command execution and undo/redo history
//! - **CanvasCommand**: Commands for specific canvas operations
//!
//! # Note
//!
//! Due to the current Canvas API limitations (shapes get auto-generated IDs),
//! commands work with the actual IDs returned by Canvas operations.

pub mod clipboard_manager;
pub mod transform_commands;

pub use clipboard_manager::{ClipboardData, ClipboardManager, PasteResult};
pub use transform_commands::{ResizeShapeCommand, RotateShapeCommand};

use crate::canvas::{Canvas, ShapeChanges};
use crate::selection::SelectionDelta;
use archflow_core::{EntityId, Vec2};
use std::fmt;

/// Result type for command operations
pub type CommandResult<T> = Result<T, CommandError>;

/// Errors that can occur during command execution
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Cannot execute command: {0}")]
    ExecutionFailed(String),

    #[error("Cannot undo command: {0}")]
    UndoFailed(String),

    #[error("Cannot redo command: {0}")]
    RedoFailed(String),

    #[error("No command to undo")]
    NothingToUndo,

    #[error("No command to redo")]
    NothingToRedo,
}

/// Trait for commands that can be executed and undone.
pub trait Command: fmt::Debug + Send + Sync + AsAny {
    /// Executes the command.
    fn execute(&mut self, canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>>;

    /// Undoes the command.
    fn undo(&mut self, canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>>;

    /// Returns a description of what this command does.
    fn description(&self) -> &str;

    /// Try to merge with another command (for continuous operations like drag).
    /// Returns true if merge was successful.
    fn merge(&mut self, _other: &dyn Command) -> bool {
        false
    }
}

/// Trait for downcasting Command trait objects
pub trait AsAny {
    /// Returns the command as a trait object
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: Command + 'static> AsAny for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Manages command execution and undo/redo history.
#[derive(Debug)]
pub struct CommandExecutor {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
    max_history: usize,
}

impl CommandExecutor {
    pub fn new() -> Self {
        Self::with_limit(0)
    }

    pub fn with_limit(max_history: usize) -> Self {
        Self {
            undo_stack: Vec::with_capacity(max_history.max(64)),
            redo_stack: Vec::with_capacity(max_history.max(64)),
            max_history,
        }
    }

    pub fn execute(
        &mut self,
        canvas: &mut Canvas,
        mut command: Box<dyn Command>,
    ) -> CommandResult<Option<SelectionDelta>> {
        self.redo_stack.clear();
        let result = command.execute(canvas)?;
        self.undo_stack.push(command);

        if self.max_history > 0 && self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }

        Ok(result)
    }

    pub fn undo(&mut self, canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>> {
        let mut command = self.undo_stack.pop().ok_or(CommandError::NothingToUndo)?;
        let result = command.undo(canvas)?;
        self.redo_stack.push(command);
        Ok(result)
    }

    pub fn redo(&mut self, canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>> {
        let mut command = self.redo_stack.pop().ok_or(CommandError::NothingToRedo)?;
        let result = command.execute(canvas)?;
        self.undo_stack.push(command);
        Ok(result)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Command to create a rectangle shape.
#[derive(Debug)]
pub struct CreateRectangleCommand {
    position: Vec2,
    size: Vec2,
    created_id: Option<EntityId>,
    executed: bool,
}

impl CreateRectangleCommand {
    pub fn new(position: Vec2, size: Vec2) -> Self {
        Self {
            position,
            size,
            created_id: None,
            executed: false,
        }
    }
}

impl Command for CreateRectangleCommand {
    fn execute(&mut self, canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>> {
        let id =
            canvas.create_rectangle(self.position.x, self.position.y, self.size.x, self.size.y);
        self.created_id = Some(id);
        self.executed = true;
        Ok(None)
    }

    fn undo(&mut self, canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>> {
        if let Some(id) = self.created_id {
            canvas.delete_shape(id);
        }
        self.executed = false;
        Ok(None)
    }

    fn description(&self) -> &str {
        "Create rectangle"
    }
}

/// Command to delete a shape.
#[derive(Debug)]
pub struct DeleteShapeCommand {
    shape_id: EntityId,
    original_data: Option<ShapeOriginalData>,
    executed: bool,
}

#[derive(Debug, Clone)]
struct ShapeOriginalData {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    rotation: f32,
    fill_color: archflow_core::Color,
    stroke_color: Option<archflow_core::Color>,
    stroke_width: f32,
    opacity: f32,
}

impl DeleteShapeCommand {
    pub fn new(shape_id: EntityId) -> Self {
        Self {
            shape_id,
            original_data: None,
            executed: false,
        }
    }
}

impl Command for DeleteShapeCommand {
    fn execute(&mut self, canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>> {
        // Capture original data before deleting
        if let Some(shape) = canvas.get_shape(self.shape_id) {
            self.original_data = Some(ShapeOriginalData {
                x: shape.x,
                y: shape.y,
                width: shape.width,
                height: shape.height,
                rotation: shape.rotation,
                fill_color: shape.fill_color,
                stroke_color: shape.stroke_color,
                stroke_width: shape.stroke_width,
                opacity: shape.opacity,
            });
        }

        canvas.delete_shape(self.shape_id);
        self.executed = true;
        Ok(None)
    }

    fn undo(&mut self, canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>> {
        if let Some(data) = &self.original_data {
            // Recreate the shape
            let id = canvas.create_rectangle(data.x, data.y, data.width, data.height);

            // Restore all properties
            let changes = ShapeChanges {
                x: Some(data.x),
                y: Some(data.y),
                width: Some(data.width),
                height: Some(data.height),
                rotation: Some(data.rotation),
                fill_color: Some(data.fill_color),
                stroke_color: Some(data.stroke_color),
                stroke_width: Some(data.stroke_width),
                opacity: Some(data.opacity),
            };

            canvas.update_shape(id, changes);
        }

        self.executed = false;
        Ok(None)
    }

    fn description(&self) -> &str {
        "Delete shape"
    }
}

/// Command to move a shape.
#[derive(Debug)]
pub struct MoveShapeCommand {
    shape_id: EntityId,
    original_position: Vec2,
    new_position: Vec2,
    executed: bool,
}

impl MoveShapeCommand {
    pub fn new(shape_id: EntityId, original_position: Vec2, new_position: Vec2) -> Self {
        Self {
            shape_id,
            original_position,
            new_position,
            executed: false,
        }
    }
}

impl Command for MoveShapeCommand {
    fn execute(&mut self, canvas: &mut Canvas) -> CommandResult<Option<SelectionDelta>> {
        let changes = ShapeChanges {
            x: Some(self.new_position.x),
            y: Some(self.new_position.y),
            width: None,
            height: None,
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
        let changes = ShapeChanges {
            x: Some(self.original_position.x),
            y: Some(self.original_position.y),
            width: None,
            height: None,
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
        "Move shape"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_executor_create_undo_redo() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let mut executor = CommandExecutor::new();

        let command = CreateRectangleCommand::new(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        // Execute
        executor.execute(&mut canvas, Box::new(command)).unwrap();
        assert_eq!(canvas.all_shapes().len(), 1);
        assert!(executor.can_undo());
        assert!(!executor.can_redo());

        // Undo
        executor.undo(&mut canvas).unwrap();
        assert_eq!(canvas.all_shapes().len(), 0);
        assert!(!executor.can_undo());
        assert!(executor.can_redo());

        // Redo
        executor.redo(&mut canvas).unwrap();
        assert_eq!(canvas.all_shapes().len(), 1);
        assert!(executor.can_undo());
        assert!(!executor.can_redo());
    }

    #[test]
    fn test_move_shape_command() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);

        let original_pos = Vec2::new(100.0, 100.0);
        let new_pos = Vec2::new(200.0, 200.0);

        let mut executor = CommandExecutor::new();
        let command = MoveShapeCommand::new(id, original_pos, new_pos);

        // Execute move
        executor.execute(&mut canvas, Box::new(command)).unwrap();
        let shape = canvas.get_shape(id).unwrap();
        // update_shape updates x,y fields
        assert_eq!(shape.x, new_pos.x);
        assert_eq!(shape.y, new_pos.y);

        // Undo move
        executor.undo(&mut canvas).unwrap();
        let shape = canvas.get_shape(id).unwrap();
        assert_eq!(shape.x, original_pos.x);
        assert_eq!(shape.y, original_pos.y);
    }

    #[test]
    fn test_delete_shape_command() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let id = canvas.create_rectangle(100.0, 100.0, 50.0, 50.0);

        let mut executor = CommandExecutor::new();
        let command = DeleteShapeCommand::new(id);

        // Execute delete
        executor.execute(&mut canvas, Box::new(command)).unwrap();
        assert!(canvas.get_shape(id).is_none());

        // Undo delete (creates new shape with different ID but at same position)
        executor.undo(&mut canvas).unwrap();
        // After undo, we should have at least one shape (though ID may differ)
        assert!(canvas.all_shapes().len() > 0);
    }

    #[test]
    fn test_history_limit() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let mut executor = CommandExecutor::with_limit(3);

        // Execute 5 commands
        for i in 0..5 {
            let pos = Vec2::new(100.0 + i as f32, 100.0);
            let size = Vec2::new(50.0, 50.0);
            let command = CreateRectangleCommand::new(pos, size);
            executor.execute(&mut canvas, Box::new(command)).unwrap();
        }

        // Should only keep last 3
        assert_eq!(executor.undo_count(), 3);
    }

    #[test]
    fn test_multiple_operations() {
        let mut canvas = Canvas::new(800.0, 600.0);
        let mut executor = CommandExecutor::new();

        // Create two rectangles
        let cmd1 = CreateRectangleCommand::new(Vec2::new(50.0, 50.0), Vec2::new(30.0, 30.0));
        let cmd2 = CreateRectangleCommand::new(Vec2::new(150.0, 150.0), Vec2::new(40.0, 40.0));

        executor.execute(&mut canvas, Box::new(cmd1)).unwrap();
        executor.execute(&mut canvas, Box::new(cmd2)).unwrap();

        assert_eq!(canvas.all_shapes().len(), 2);
        assert_eq!(executor.undo_count(), 2);

        // Undo both
        executor.undo(&mut canvas).unwrap();
        executor.undo(&mut canvas).unwrap();

        assert_eq!(canvas.all_shapes().len(), 0);
        assert_eq!(executor.redo_count(), 2);

        // Redo both
        executor.redo(&mut canvas).unwrap();
        executor.redo(&mut canvas).unwrap();

        assert_eq!(canvas.all_shapes().len(), 2);
    }
}
