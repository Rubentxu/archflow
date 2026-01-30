//! Command pattern for editing operations
//!
//! This module defines the Command trait and common commands for editing operations.

use crate::EntityId;
use serde::{Deserialize, Serialize};

/// Result of executing a command
pub type CommandResult = Result<(), CommandError>;

/// Error that can occur when executing a command
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Entity not found: {0}")]
    EntityNotFound(EntityId),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),
}

/// Trait for commands that can be executed and undone
pub trait Command: Send + Sync {
    /// Executes the command
    fn execute(&mut self) -> CommandResult;

    /// Undoes the command
    fn undo(&mut self) -> CommandResult;

    /// Returns a description of the command for display
    fn description(&self) -> String;

    /// Returns the entities affected by this command
    fn affected_entities(&self) -> Vec<EntityId>;
}

/// Executor for running commands with undo/redo support
pub struct CommandExecutor {
    /// Commands that have been executed and can be undone
    history: Vec<Box<dyn Command>>,
    /// Commands that were undone and can be redone
    redo_stack: Vec<Box<dyn Command>>,
    /// Maximum depth of the history
    max_history: usize,
}

impl CommandExecutor {
    /// Creates a new command executor
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 100,
        }
    }

    /// Sets the maximum history depth
    pub fn set_max_history(&mut self, max: usize) {
        self.max_history = max;
        // Trim history if needed
        if self.history.len() > max {
            self.history.drain(0..self.history.len() - max);
        }
    }

    /// Executes a command and adds it to the history
    pub fn execute(&mut self, mut command: Box<dyn Command>) -> CommandResult {
        // Execute the command
        command.execute()?;

        // Clear redo stack when executing a new command
        self.redo_stack.clear();

        // Add to history
        self.history.push(command);

        // Trim history if needed
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }

        Ok(())
    }

    /// Undoes the last command
    pub fn undo(&mut self) -> CommandResult {
        let mut command = self
            .history
            .pop()
            .ok_or_else(|| CommandError::InvalidOperation("No commands to undo".to_string()))?;

        command.undo()?;
        self.redo_stack.push(command);

        Ok(())
    }

    /// Redoes the last undone command
    pub fn redo(&mut self) -> CommandResult {
        let mut command = self
            .redo_stack
            .pop()
            .ok_or_else(|| CommandError::InvalidOperation("No commands to redo".to_string()))?;

        command.execute()?;
        self.history.push(command);

        Ok(())
    }

    /// Returns true if undo is available
    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    /// Returns true if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Clears all history
    pub fn clear(&mut self) {
        self.history.clear();
        self.redo_stack.clear();
    }

    /// Returns the number of commands in history
    pub fn history_len(&self) -> usize {
        self.history.len()
    }
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}
