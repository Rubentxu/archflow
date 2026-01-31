// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Interaction - History Manager (Undo/Redo)
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 16
//
// Command sourcing pattern for undo/redo functionality.
// Stores reversible commands with redo/undo pairs.
// ═══════════════════════════════════════════════════════════════════════════════

extern crate alloc;

use alloc::collections::VecDeque;
use alloc::vec::Vec;

use archflow_core::{EntityId, Vec2};
use archflow_engine::{Command, EntityStore};

/// Maximum depth for undo/redo stack
pub const DEFAULT_MAX_DEPTH: usize = 100;

/// History manager for undo/redo functionality
///
/// Uses command sourcing pattern - stores reversible commands
/// that can be executed and undone.
pub struct HistoryManager {
    /// Stack of undoable actions
    undo_stack: VecDeque<UndoEntry>,
    /// Stack of redoable actions
    redo_stack: VecDeque<UndoEntry>,
    /// Maximum depth of the undo stack
    max_depth: usize,
}

/// Entry in the undo/redo stack
///
/// Contains both the redo command (to apply forward) and
/// the undo command (to revert the action).
#[derive(Clone, Debug)]
pub struct UndoEntry {
    /// Command to redo the action
    pub redo: Command,
    /// Command to undo the action
    pub undo: Command,
}

impl HistoryManager {
    /// Create a new history manager
    ///
    /// # Arguments
    /// * `max_depth` - Maximum number of undo steps to keep
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo_stack: VecDeque::with_capacity(max_depth),
            redo_stack: VecDeque::with_capacity(max_depth),
            max_depth,
        }
    }

    /// Create a history manager with default max depth
    pub fn with_default_depth() -> Self {
        Self::new(DEFAULT_MAX_DEPTH)
    }

    /// Record a reversible action
    ///
    /// # Arguments
    /// * `redo` - Command to redo the action
    /// * `undo` - Command to undo the action
    ///
    /// # Note
    /// Clears the redo stack as a new action invalidates redo history
    pub fn record(&mut self, redo: Command, undo: Command) {
        // Enforce max depth by removing oldest entry if needed
        if self.undo_stack.len() >= self.max_depth {
            self.undo_stack.pop_front();
        }

        self.undo_stack.push_back(UndoEntry { redo, undo });
        self.redo_stack.clear(); // Invalidate redo on new action
    }

    /// Undo the last action
    ///
    /// # Arguments
    /// * `store` - Entity store to apply the undo command
    ///
    /// # Returns
    /// true if an action was undone, false if stack was empty
    pub fn undo(&mut self, store: &mut EntityStore) -> bool {
        if let Some(entry) = self.undo_stack.pop_back() {
            // Execute undo command
            entry.undo.execute(store);
            // Move to redo stack
            self.redo_stack.push_back(entry);
            true
        } else {
            false
        }
    }

    /// Redo the last undone action
    ///
    /// # Arguments
    /// * `store` - Entity store to apply the redo command
    ///
    /// # Returns
    /// true if an action was redone, false if stack was empty
    pub fn redo(&mut self, store: &mut EntityStore) -> bool {
        if let Some(entry) = self.redo_stack.pop_back() {
            // Execute redo command
            entry.redo.execute(store);
            // Move back to undo stack
            self.undo_stack.push_back(entry);
            true
        } else {
            false
        }
    }

    /// Check if undo is available
    #[inline]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    #[inline]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get the number of undoable actions
    #[inline]
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the number of redoable actions
    #[inline]
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self::with_default_depth()
    }
}

/// Helper to create undo/redo command pairs for common operations
pub struct HistoryCommands;

impl HistoryCommands {
    /// Create undo/redo pair for SetColor command
    ///
    /// # Arguments
    /// * `entity_id` - Entity to modify
    /// * `old_color` - Previous color (for undo)
    /// * `new_color` - New color (for redo)
    pub fn change_color(entity_id: EntityId, old_color: u32, new_color: u32) -> (Command, Command) {
        let redo = Command::SetColor {
            id: entity_id,
            color: new_color,
        };
        let undo = Command::SetColor {
            id: entity_id,
            color: old_color,
        };
        (redo, undo)
    }

    /// Create undo/redo pair for Move command
    ///
    /// # Arguments
    /// * `entity_id` - Entity to move
    /// * `delta` - Movement delta (for redo)
    pub fn move_delta(entity_id: EntityId, delta: Vec2) -> (Command, Command) {
        let redo = Command::Move {
            id: entity_id,
            delta,
        };
        let undo = Command::Move {
            id: entity_id,
            delta: -delta,
        };
        (redo, undo)
    }

    /// Create undo/redo pair for Resize command
    ///
    /// # Arguments
    /// * `entity_id` - Entity to resize
    /// * `old_size` - Previous size (for undo)
    /// * `new_size` - New size (for redo)
    pub fn resize(entity_id: EntityId, old_size: Vec2, new_size: Vec2) -> (Command, Command) {
        let redo = Command::Resize {
            id: entity_id,
            size: new_size,
        };
        let undo = Command::Resize {
            id: entity_id,
            size: old_size,
        };
        (redo, undo)
    }

    /// Create undo/redo pair for Teleport command
    ///
    /// # Arguments
    /// * `entity_id` - Entity to teleport
    /// * `old_pos` - Previous position (for undo)
    /// * `new_pos` - New position (for redo)
    pub fn teleport(entity_id: EntityId, old_pos: Vec2, new_pos: Vec2) -> (Command, Command) {
        let redo = Command::Teleport {
            id: entity_id,
            pos: new_pos,
        };
        let undo = Command::Teleport {
            id: entity_id,
            pos: old_pos,
        };
        (redo, undo)
    }

    /// Create undo/redo pair for Spawn/Despawn
    ///
    /// # Arguments
    /// * `pos` - Spawn position
    /// * `size` - Spawn size
    /// * `entity_id` - Entity ID that was spawned (for undo)
    pub fn spawn(pos: Vec2, size: Vec2, entity_id: Option<EntityId>) -> (Command, Command) {
        let redo = Command::Spawn {
            pos,
            size,
            parent: None,
        };
        let undo = Command::Despawn(entity_id.unwrap_or(EntityId::new(0)));
        (redo, undo)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_manager_creation() {
        let manager = HistoryManager::new(50);
        assert_eq!(manager.undo_count(), 0);
        assert_eq!(manager.redo_count(), 0);
        assert!(!manager.can_undo());
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_history_manager_default() {
        let manager = HistoryManager::default();
        assert_eq!(manager.undo_count(), 0);
        assert_eq!(manager.max_depth, DEFAULT_MAX_DEPTH);
    }

    #[test]
    fn test_record_action() {
        let mut manager = HistoryManager::new(10);
        let entity = EntityId::new(1);
        let (redo, undo) = HistoryCommands::change_color(entity, 0xFF0000, 0x00FF00);

        manager.record(redo, undo);

        assert_eq!(manager.undo_count(), 1);
        assert!(manager.can_undo());
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_undo_clears_redo() {
        let mut manager = HistoryManager::new(10);
        let entity = EntityId::new(1);
        let (redo1, undo1) = HistoryCommands::change_color(entity, 0xFF0000, 0x00FF00);
        let (redo2, undo2) = HistoryCommands::change_color(entity, 0x00FF00, 0x0000FF);

        manager.record(redo1, undo1);
        manager.undo(&mut EntityStore::new());
        assert_eq!(manager.redo_count(), 1);

        // New action clears redo
        manager.record(redo2, undo2);
        assert_eq!(manager.redo_count(), 0);
    }

    #[test]
    fn test_max_depth_enforcement() {
        let mut manager = HistoryManager::new(3);

        for i in 0..5 {
            let entity = EntityId::new(i as u32);
            let (redo, undo) = HistoryCommands::change_color(entity, i, i + 1);
            manager.record(redo, undo);
        }

        // Should only keep 3 entries
        assert_eq!(manager.undo_count(), 3);
    }

    #[test]
    fn test_clear() {
        let mut manager = HistoryManager::new(10);
        let entity = EntityId::new(1);
        let (redo, undo) = HistoryCommands::change_color(entity, 0xFF0000, 0x00FF00);

        manager.record(redo, undo);
        assert_eq!(manager.undo_count(), 1);

        manager.clear();
        assert_eq!(manager.undo_count(), 0);
        assert_eq!(manager.redo_count(), 0);
    }

    #[test]
    fn test_change_color_command() {
        let entity = EntityId::new(1);
        let (redo, undo) = HistoryCommands::change_color(entity, 0xFF0000, 0x00FF00);

        // Verify redo command
        assert!(matches!(redo, Command::SetColor { id, color: 0x00FF00 } if id == entity));
        // Verify undo command
        assert!(matches!(undo, Command::SetColor { id, color: 0xFF0000 } if id == entity));
    }

    #[test]
    fn test_move_delta_command() {
        let entity = EntityId::new(1);
        let delta = Vec2::new(10.0, 20.0);
        let (redo, undo) = HistoryCommands::move_delta(entity, delta);

        assert!(matches!(redo, Command::Move { id, .. } if id == entity));
        assert!(matches!(undo, Command::Move { id, .. } if id == entity));
    }

    #[test]
    fn test_resize_command() {
        let entity = EntityId::new(1);
        let old_size = Vec2::new(50.0, 50.0);
        let new_size = Vec2::new(100.0, 100.0);
        let (redo, undo) = HistoryCommands::resize(entity, old_size, new_size);

        assert!(matches!(redo, Command::Resize { id, .. } if id == entity));
        assert!(matches!(undo, Command::Resize { id, .. } if id == entity));
    }

    #[test]
    fn test_teleport_command() {
        let entity = EntityId::new(1);
        let old_pos = Vec2::new(10.0, 10.0);
        let new_pos = Vec2::new(50.0, 50.0);
        let (redo, undo) = HistoryCommands::teleport(entity, old_pos, new_pos);

        assert!(matches!(redo, Command::Teleport { id, .. } if id == entity));
        assert!(matches!(undo, Command::Teleport { id, .. } if id == entity));
    }

    #[test]
    fn test_spawn_command() {
        let entity = EntityId::new(42);
        let pos = Vec2::new(100.0, 100.0);
        let size = Vec2::new(50.0, 50.0);
        let (redo, undo) = HistoryCommands::spawn(pos, size, Some(entity));

        assert!(matches!(redo, Command::Spawn { .. }));
        assert!(matches!(undo, Command::Despawn(id) if id == entity));
    }

    #[test]
    fn test_undo_entry_clone() {
        let entity = EntityId::new(1);
        let (redo, undo) = HistoryCommands::change_color(entity, 0xFF0000, 0x00FF00);
        let entry = UndoEntry { redo, undo };

        // Test Clone derive
        let _entry2 = entry.clone();
    }
}
