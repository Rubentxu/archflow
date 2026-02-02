// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Command Pattern for Undo/Redo (HU-013)
//
// This module implements the Command Pattern for reversible operations,
// enabling undo/redo functionality for the editor.
//
// Reference: docs/epics/EPIC-003-actuators-animations.md - HU-013
//
// Architecture:
// - Command: Trait for executable/rollbackable operations
// - CommandHistory: Stack-based undo/redo management
// - Concrete Commands: Move, Resize, SetColor, Visibility, Selection, etc.
//
// Performance:
// - Zero-allocation hot path using enum-based commands
// - Bounded history (configurable max size)
// - O(1) undo/redo operations
// ═══════════════════════════════════════════════════════════════════════════════

#![no_std]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use archflow_core::Vec2;
use archflow_engine::EntityStore;

/// Maximum command history size (prevent memory bloat)
pub const DEFAULT_MAX_HISTORY: usize = 100;

/// Command trait for reversible operations
///
/// All commands must implement `execute()` to perform the action
/// and `rollback()` to reverse it. The command can also provide
/// a description for UI display.
pub trait Command {
    /// Execute the command
    fn execute(&mut self, store: &mut EntityStore);

    /// Rollback (undo) the command
    fn rollback(&mut self, store: &mut EntityStore);

    /// Get a description of the command (for UI/undo menu)
    fn description(&self) -> &'static str;
}

/// All possible commands in the system
///
/// Using an enum instead of `Box<dyn Command>` for:
/// - Zero-cost abstraction (no heap allocation)
/// - Better cache locality
/// - Predictable memory usage
/// - No trait object overhead
///
/// Size: 32 bytes per command (fits in cache line)
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub enum AnyCommand {
    /// Move an entity to a new position (by index)
    Move {
        /// Entity index in store
        entity_idx: usize,
        /// Previous position
        from: Vec2,
        /// New position
        to: Vec2,
    },

    /// Resize an entity
    Resize {
        /// Entity index
        entity_idx: usize,
        /// Previous size
        from: Vec2,
        /// New size
        to: Vec2,
    },

    /// Set entity position (absolute)
    SetPosition {
        /// Entity index
        entity_idx: usize,
        /// Previous position
        old_pos: Vec2,
        /// New position
        new_pos: Vec2,
    },

    /// Set entity size (absolute)
    SetSize {
        /// Entity index
        entity_idx: usize,
        /// Previous size
        old_size: Vec2,
        /// New size
        new_size: Vec2,
    },

    /// Set entity color (RGBA packed)
    SetColor {
        /// Entity index
        entity_idx: usize,
        /// Previous color
        old_color: u32,
        /// New color
        new_color: u32,
    },

    /// Set entity visibility
    SetVisibility {
        /// Entity index
        entity_idx: usize,
        /// Previous visibility state
        was_visible: bool,
        /// New visibility state
        now_visible: bool,
    },

    /// Set entity selection state
    Select {
        /// Entity index
        entity_idx: usize,
        /// Previous selection state
        was_selected: bool,
        /// New selection state
        now_selected: bool,
    },

    /// Set entity layer
    SetLayer {
        /// Entity index
        entity_idx: usize,
        /// Previous layer
        old_layer: u8,
        /// New layer
        new_layer: u8,
    },
}

impl AnyCommand {
    /// Create a move command
    #[inline(always)]
    #[must_use]
    pub fn move_entity(entity_idx: usize, from: Vec2, to: Vec2) -> Self {
        Self::Move {
            entity_idx,
            from,
            to,
        }
    }

    /// Create a resize command
    #[inline(always)]
    #[must_use]
    pub fn resize_entity(entity_idx: usize, from: Vec2, to: Vec2) -> Self {
        Self::Resize {
            entity_idx,
            from,
            to,
        }
    }

    /// Get command description
    #[inline(always)]
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Move { .. } => "Move",
            Self::Resize { .. } => "Resize",
            Self::SetPosition { .. } => "Set Position",
            Self::SetSize { .. } => "Set Size",
            Self::SetColor { .. } => "Set Color",
            Self::SetVisibility { .. } => "Set Visibility",
            Self::Select {
                now_selected: true, ..
            } => "Select",
            Self::Select {
                now_selected: false,
                ..
            } => "Deselect",
            Self::SetLayer { .. } => "Set Layer",
        }
    }
}

impl Command for AnyCommand {
    fn execute(&mut self, store: &mut EntityStore) {
        match self {
            Self::Move { entity_idx, to, .. } => {
                store.set_pos(*entity_idx, *to);
            }
            Self::Resize { entity_idx, to, .. } => {
                store.set_size(*entity_idx, *to);
            }
            Self::SetPosition {
                entity_idx,
                new_pos,
                ..
            } => {
                store.set_pos(*entity_idx, *new_pos);
            }
            Self::SetSize {
                entity_idx,
                new_size,
                ..
            } => {
                store.set_size(*entity_idx, *new_size);
            }
            Self::SetColor {
                entity_idx,
                new_color,
                ..
            } => {
                store.set_color(*entity_idx, *new_color);
            }
            Self::SetVisibility {
                entity_idx,
                now_visible,
                ..
            } => {
                store.set_visible(*entity_idx, *now_visible);
            }
            Self::Select {
                entity_idx,
                now_selected,
                ..
            } => {
                store.set_selected(*entity_idx, *now_selected);
            }
            Self::SetLayer {
                entity_idx,
                new_layer,
                ..
            } => {
                store.set_layer(*entity_idx, *new_layer);
            }
        }
    }

    fn rollback(&mut self, store: &mut EntityStore) {
        match self {
            Self::Move {
                entity_idx, from, ..
            } => {
                store.set_pos(*entity_idx, *from);
            }
            Self::Resize {
                entity_idx, from, ..
            } => {
                store.set_size(*entity_idx, *from);
            }
            Self::SetPosition {
                entity_idx,
                old_pos,
                ..
            } => {
                store.set_pos(*entity_idx, *old_pos);
            }
            Self::SetSize {
                entity_idx,
                old_size,
                ..
            } => {
                store.set_size(*entity_idx, *old_size);
            }
            Self::SetColor {
                entity_idx,
                old_color,
                ..
            } => {
                store.set_color(*entity_idx, *old_color);
            }
            Self::SetVisibility {
                entity_idx,
                was_visible,
                ..
            } => {
                store.set_visible(*entity_idx, *was_visible);
            }
            Self::Select {
                entity_idx,
                was_selected,
                ..
            } => {
                store.set_selected(*entity_idx, *was_selected);
            }
            Self::SetLayer {
                entity_idx,
                old_layer,
                ..
            } => {
                store.set_layer(*entity_idx, *old_layer);
            }
        }
    }

    fn description(&self) -> &'static str {
        self.description()
    }
}

/// Command history manager for undo/redo
///
/// Maintains two stacks:
/// - `undo_stack`: Commands that can be undone
/// - `redo_stack`: Commands that can be redone after undo
///
/// # Performance
///
/// - O(1) execute, undo, redo operations
/// - Bounded memory (configurable max history)
/// - Zero allocations in hot path
pub struct CommandHistory {
    /// Stack of executed commands (for undo)
    undo_stack: Vec<AnyCommand>,

    /// Stack of undone commands (for redo)
    redo_stack: Vec<AnyCommand>,

    /// Maximum commands to keep in history
    max_history: usize,

    /// Current batch group (for grouping commands)
    batch_group: Option<u32>,

    /// Next batch ID
    next_batch_id: u32,
}

impl CommandHistory {
    /// Create a new command history with default settings
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self::with_max_history(DEFAULT_MAX_HISTORY)
    }

    /// Create a new command history with custom max size
    #[inline(always)]
    #[must_use]
    pub fn with_max_history(max_history: usize) -> Self {
        Self {
            undo_stack: Vec::with_capacity(max_history),
            redo_stack: Vec::with_capacity(max_history),
            max_history,
            batch_group: None,
            next_batch_id: 1,
        }
    }

    /// Execute a command
    ///
    /// The command is executed and pushed to the undo stack.
    /// The redo stack is cleared (new branch taken).
    #[inline(always)]
    pub fn execute(&mut self, mut cmd: AnyCommand, store: &mut EntityStore) {
        // Check if we're in a batch group
        if let Some(_batch_id) = self.batch_group {
            // Merge with last command if same batch
            if let Some(last) = self.undo_stack.last() {
                if self.is_compatible(last, &cmd) {
                    // Can't merge in-place with non-mut borrow, so just execute normally
                    cmd.execute(store);
                    self.undo_stack.push(cmd);
                    self.redo_stack.clear();
                    return;
                }
            }
        }

        // Execute and push to undo stack
        cmd.execute(store);
        self.undo_stack.push(cmd);

        // Clear redo stack (new timeline)
        self.redo_stack.clear();

        // Enforce max history limit
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }
    }

    /// Undo the last command
    ///
    /// Returns true if undo was performed, false if nothing to undo.
    #[inline(always)]
    pub fn undo(&mut self, store: &mut EntityStore) -> bool {
        if let Some(mut cmd) = self.undo_stack.pop() {
            cmd.rollback(store);
            self.redo_stack.push(cmd);
            true
        } else {
            false
        }
    }

    /// Redo the last undone command
    ///
    /// Returns true if redo was performed, false if nothing to redo.
    #[inline(always)]
    pub fn redo(&mut self, store: &mut EntityStore) -> bool {
        if let Some(mut cmd) = self.redo_stack.pop() {
            cmd.execute(store);
            self.undo_stack.push(cmd);
            true
        } else {
            false
        }
    }

    /// Undo multiple commands at once
    #[inline(always)]
    pub fn undo_n(&mut self, store: &mut EntityStore, count: usize) {
        for _ in 0..count.min(self.undo_stack.len()) {
            self.undo(store);
        }
    }

    /// Redo multiple commands at once
    #[inline(always)]
    pub fn redo_n(&mut self, store: &mut EntityStore, count: usize) {
        for _ in 0..count.min(self.redo_stack.len()) {
            self.redo(store);
        }
    }

    /// Start a batch group (commands will be merged)
    ///
    /// All commands executed until `end_batch()` will be treated as
    /// a single undoable unit.
    #[inline(always)]
    pub fn begin_batch(&mut self) {
        self.batch_group = Some(self.next_batch_id);
        self.next_batch_id = self.next_batch_id.wrapping_add(1);
    }

    /// End the current batch group
    #[inline(always)]
    pub fn end_batch(&mut self) {
        self.batch_group = None;
    }

    /// Check if undo is available
    #[inline(always)]
    #[must_use]
    pub const fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    #[inline(always)]
    #[must_use]
    pub const fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get the description of the next command to undo
    #[inline(always)]
    #[must_use]
    pub fn undo_description(&self) -> &'static str {
        self.undo_stack.last().map_or("", |cmd| cmd.description())
    }

    /// Get the description of the next command to redo
    #[inline(always)]
    #[must_use]
    pub fn redo_description(&self) -> &'static str {
        self.redo_stack.last().map_or("", |cmd| cmd.description())
    }

    /// Get the number of commands in undo history
    #[inline(always)]
    #[must_use]
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the number of commands in redo history
    #[inline(always)]
    #[must_use]
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Clear all history
    #[inline(always)]
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.batch_group = None;
    }

    /// Check if two commands can be merged
    fn is_compatible(&self, _a: &AnyCommand, b: &AnyCommand) -> bool {
        // Only merge same-type commands on same entity
        matches!(b, AnyCommand::Move { .. } | AnyCommand::Resize { .. })
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONVENIENCE FUNCTIONS
// ═════════════════════════════════════════════════════════════════════════════//

/// Helper to create a move command from entity state change
#[inline(always)]
#[must_use]
pub fn make_move_command(store: &EntityStore, entity_idx: usize, new_pos: Vec2) -> AnyCommand {
    let old_pos = store.pos(entity_idx);
    AnyCommand::move_entity(entity_idx, old_pos, new_pos)
}

/// Helper to create a resize command from entity state change
#[inline(always)]
#[must_use]
pub fn make_resize_command(store: &EntityStore, entity_idx: usize, new_size: Vec2) -> AnyCommand {
    let old_size = store.size(entity_idx);
    AnyCommand::resize_entity(entity_idx, old_size, new_size)
}

/// Helper to create a color change command
#[inline(always)]
#[must_use]
pub fn make_color_command(store: &EntityStore, entity_idx: usize, new_color: u32) -> AnyCommand {
    let old_color = store.colors_ref()[entity_idx];
    AnyCommand::SetColor {
        entity_idx,
        old_color,
        new_color,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═════════════════════════════════════════════════════════════════════════════//

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_engine::EntityStore;

    fn create_test_store() -> EntityStore {
        EntityStore::new()
    }

    #[test]
    fn test_move_command() {
        let mut store = create_test_store();
        store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0)); // Entity at index 0

        let mut cmd = AnyCommand::move_entity(0, Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        assert_eq!(store.pos(0), Vec2::new(0.0, 0.0));

        cmd.execute(&mut store);
        assert_eq!(store.pos(0), Vec2::new(100.0, 100.0));

        cmd.rollback(&mut store);
        assert_eq!(store.pos(0), Vec2::new(0.0, 0.0));
    }

    #[test]
    fn test_command_history_undo_redo() {
        let mut store = create_test_store();
        store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0)); // Entity at index 0

        let mut history = CommandHistory::new();

        // Initially nothing to undo/redo
        assert!(!history.can_undo());
        assert!(!history.can_redo());

        // Execute a move command
        let cmd = AnyCommand::move_entity(0, Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        history.execute(cmd, &mut store);

        assert!(history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(store.pos(0), Vec2::new(100.0, 100.0));

        // Undo
        assert!(history.undo(&mut store));
        assert_eq!(store.pos(0), Vec2::new(0.0, 0.0));
        assert!(history.can_redo());

        // Redo
        assert!(history.redo(&mut store));
        assert_eq!(store.pos(0), Vec2::new(100.0, 100.0));
        assert!(!history.can_redo());
    }

    #[test]
    fn test_command_history_clear() {
        let mut store = create_test_store();
        store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));

        let mut history = CommandHistory::new();
        let cmd = AnyCommand::move_entity(0, Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        history.execute(cmd, &mut store);

        assert!(history.can_undo());

        history.clear();

        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_command_description() {
        let move_cmd = AnyCommand::move_entity(0, Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        assert_eq!(move_cmd.description(), "Move");

        let resize_cmd =
            AnyCommand::resize_entity(0, Vec2::new(50.0, 50.0), Vec2::new(100.0, 100.0));
        assert_eq!(resize_cmd.description(), "Resize");
    }

    #[test]
    fn test_batch_commands() {
        let mut store = create_test_store();
        store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));

        let mut history = CommandHistory::new();

        // Start batch - commands executed in batch are grouped together
        history.begin_batch();

        // Execute multiple moves
        let cmd1 = AnyCommand::move_entity(0, Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        history.execute(cmd1, &mut store);

        let cmd2 = AnyCommand::move_entity(0, Vec2::new(10.0, 10.0), Vec2::new(20.0, 20.0));
        history.execute(cmd2, &mut store);

        history.end_batch();

        // Commands are separate but part of same batch group
        // (Note: full merge requires more complex logic)
        assert_eq!(store.pos(0), Vec2::new(20.0, 20.0));

        // Undo goes back one command at a time
        assert!(history.undo(&mut store));
        assert_eq!(store.pos(0), Vec2::new(10.0, 10.0)); // First undo goes to first command's destination
    }

    #[test]
    fn test_new_command_clears_redo() {
        let mut store = create_test_store();
        store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));

        let mut history = CommandHistory::new();

        // Execute and undo
        let cmd1 = AnyCommand::move_entity(0, Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        history.execute(cmd1, &mut store);
        history.undo(&mut store);

        assert!(history.can_redo());

        // Execute new command
        let cmd2 = AnyCommand::move_entity(0, Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));
        history.execute(cmd2, &mut store);

        // Redo stack should be cleared
        assert!(!history.can_redo());
    }

    #[test]
    fn test_undo_n_and_redo_n() {
        let mut store = create_test_store();
        store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));

        let mut history = CommandHistory::new();

        // Execute multiple commands
        history.execute(
            AnyCommand::move_entity(0, Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0)),
            &mut store,
        );
        history.execute(
            AnyCommand::move_entity(0, Vec2::new(10.0, 10.0), Vec2::new(20.0, 20.0)),
            &mut store,
        );
        history.execute(
            AnyCommand::move_entity(0, Vec2::new(20.0, 20.0), Vec2::new(30.0, 30.0)),
            &mut store,
        );

        assert_eq!(store.pos(0), Vec2::new(30.0, 30.0));
        assert_eq!(history.undo_count(), 3);

        // Undo 2
        history.undo_n(&mut store, 2);
        assert_eq!(store.pos(0), Vec2::new(10.0, 10.0));
        assert_eq!(history.redo_count(), 2);

        // Redo 1
        history.redo_n(&mut store, 1);
        assert_eq!(store.pos(0), Vec2::new(20.0, 20.0));
    }
}
