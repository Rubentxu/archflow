//! History management for undo/redo operations

use crate::command::{Command, CommandError, CommandResult};
use crate::EntityId;
use std::collections::HashMap;

/// Configuration for history manager
#[derive(Clone, Debug)]
pub struct HistoryConfig {
    /// Maximum number of commands to keep in history
    pub max_history: usize,
    /// Whether to enable auto-save
    pub auto_save: bool,
    /// Auto-save interval in milliseconds (0 = disabled)
    pub auto_save_interval_ms: u64,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            max_history: 100,
            auto_save: false,
            auto_save_interval_ms: 0,
        }
    }
}

/// Snapshot of canvas state for undo/redo
#[derive(Clone, Debug)]
pub struct HistorySnapshot {
    /// Unique ID for this snapshot
    pub id: String,
    /// Timestamp of the snapshot
    pub timestamp: u64,
    /// Description of what changed
    pub description: String,
    /// Entities affected by this change
    pub affected_entities: Vec<EntityId>,
}

/// Manages the history of commands for undo/redo
pub struct HistoryManager {
    /// Configuration
    config: HistoryConfig,
    /// History of executed commands
    history: Vec<Box<dyn Command>>,
    /// Commands that were undone
    redo_stack: Vec<Box<dyn Command>>,
    /// Snapshots for each history entry
    snapshots: Vec<HistorySnapshot>,
    /// Snapshot counter for generating IDs
    snapshot_counter: u64,
}

impl HistoryManager {
    /// Creates a new history manager
    pub fn new() -> Self {
        Self {
            config: HistoryConfig::default(),
            history: Vec::new(),
            redo_stack: Vec::new(),
            snapshots: Vec::new(),
            snapshot_counter: 0,
        }
    }

    /// Sets the configuration
    pub fn set_config(&mut self, config: HistoryConfig) {
        self.config = config;
    }

    /// Gets the configuration
    pub fn config(&self) -> &HistoryConfig {
        &self.config
    }

    /// Executes a command and adds it to history
    pub fn execute(&mut self, mut command: Box<dyn Command>) -> CommandResult {
        // Execute the command
        command.execute()?;

        // Create snapshot
        let snapshot = HistorySnapshot {
            id: format!("snapshot-{}", self.snapshot_counter),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            description: command.description(),
            affected_entities: command.affected_entities(),
        };

        // Clear redo stack when executing a new command
        self.redo_stack.clear();

        // Add to history
        self.history.push(command);
        self.snapshots.push(snapshot);
        self.snapshot_counter += 1;

        // Trim history if needed
        if self.history.len() > self.config.max_history {
            self.history.remove(0);
            self.snapshots.remove(0);
        }

        Ok(())
    }

    /// Undoes the last command
    pub fn undo(&mut self) -> CommandResult {
        let mut command = self.history.pop()
            .ok_or_else(|| CommandError::InvalidOperation("No commands to undo".to_string()))?;

        command.undo()?;

        // Remove corresponding snapshot
        if !self.snapshots.is_empty() {
            self.snapshots.pop();
        }

        self.redo_stack.push(command);
        Ok(())
    }

    /// Redoes the last undone command
    pub fn redo(&mut self) -> CommandResult {
        let mut command = self.redo_stack.pop()
            .ok_or_else(|| CommandError::InvalidOperation("No commands to redo".to_string()))?;

        command.execute()?;

        // Recreate snapshot
        let snapshot = HistorySnapshot {
            id: format!("snapshot-{}", self.snapshot_counter),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            description: command.description(),
            affected_entities: command.affected_entities(),
        };

        self.history.push(command);
        self.snapshots.push(snapshot);
        self.snapshot_counter += 1;

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
        self.snapshots.clear();
    }

    /// Returns the number of commands in history
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Gets all snapshots
    pub fn snapshots(&self) -> &[HistorySnapshot] {
        &self.snapshots
    }
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self::new()
    }
}
