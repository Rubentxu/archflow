//! Tests for Editing bounded context reorganization
//!
//! These tests verify that the Editing bounded context properly consolidates
//! all editing-related functionality from the old architecture.

use crate::{
    Command, CommandError, CommandExecutor, CommandResult, EntityId, HistoryConfig, HistoryManager,
};

// Mock command for testing
struct MockCommand {
    executed: bool,
    undone: bool,
    entity_id: EntityId,
}

impl MockCommand {
    fn new(entity_id: EntityId) -> Self {
        Self {
            executed: false,
            undone: false,
            entity_id,
        }
    }
}

impl Command for MockCommand {
    fn execute(&mut self) -> CommandResult {
        self.executed = true;
        self.undone = false;
        Ok(())
    }

    fn undo(&mut self) -> CommandResult {
        if !self.executed {
            return Err(CommandError::InvalidOperation(
                "Cannot undo unexecuted command".to_string(),
            ));
        }
        self.undone = true;
        Ok(())
    }

    fn description(&self) -> String {
        "Mock command".to_string()
    }

    fn affected_entities(&self) -> Vec<EntityId> {
        vec![self.entity_id]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: Verifies command execution
    #[test]
    fn test_command_execute() {
        let id = EntityId::from_u128(1);
        let mut command = MockCommand::new(id);

        assert!(!command.executed);
        command.execute().unwrap();
        assert!(command.executed);
        assert!(!command.undone);
    }

    /// Test: Verifies command undo
    #[test]
    fn test_command_undo() {
        let id = EntityId::from_u128(1);
        let mut command = MockCommand::new(id);

        command.execute().unwrap();
        command.undo().unwrap();
        assert!(command.executed);
        assert!(command.undone);
    }

    /// Test: Verifies command undo without execute fails
    #[test]
    fn test_command_undo_without_execute_fails() {
        let id = EntityId::from_u128(1);
        let mut command = MockCommand::new(id);

        let result = command.undo();
        assert!(result.is_err());
    }

    /// Test: Verifies command executor
    #[test]
    fn test_command_executor() {
        let mut executor = CommandExecutor::new();
        let id = EntityId::from_u128(1);
        let command = Box::new(MockCommand::new(id));

        executor.execute(command).unwrap();
        assert_eq!(executor.history_len(), 1);
        assert!(executor.can_undo());
        assert!(!executor.can_redo());
    }

    /// Test: Verifies command executor undo
    #[test]
    fn test_command_executor_undo() {
        let mut executor = CommandExecutor::new();
        let id = EntityId::from_u128(1);
        let command = Box::new(MockCommand::new(id));

        executor.execute(command).unwrap();
        executor.undo().unwrap();

        assert_eq!(executor.history_len(), 0);
        assert!(!executor.can_undo());
        assert!(executor.can_redo());
    }

    /// Test: Verifies command executor redo
    #[test]
    fn test_command_executor_redo() {
        let mut executor = CommandExecutor::new();
        let id = EntityId::from_u128(1);
        let command = Box::new(MockCommand::new(id));

        executor.execute(command).unwrap();
        executor.undo().unwrap();
        executor.redo().unwrap();

        assert_eq!(executor.history_len(), 1);
        assert!(executor.can_undo());
        assert!(!executor.can_redo());
    }

    /// Test: Verifies command executor clears redo stack on new command
    #[test]
    fn test_command_executor_clears_redo_on_execute() {
        let mut executor = CommandExecutor::new();
        let id1 = EntityId::from_u128(1);
        let id2 = EntityId::from_u128(2);

        executor.execute(Box::new(MockCommand::new(id1))).unwrap();
        executor.undo().unwrap();
        assert!(executor.can_redo());

        // Execute new command should clear redo stack
        executor.execute(Box::new(MockCommand::new(id2))).unwrap();
        assert!(!executor.can_redo());
        // History should have 1 command (id2) since id1 was undone and redo cleared
        assert_eq!(executor.history_len(), 1);
    }

    /// Test: Verifies history manager creation
    #[test]
    fn test_history_manager_creation() {
        let manager = HistoryManager::new();
        assert_eq!(manager.config().max_history, 100);
        assert_eq!(manager.history_len(), 0);
    }

    /// Test: Verifies history manager configuration
    #[test]
    fn test_history_manager_config() {
        let mut manager = HistoryManager::new();
        let config = HistoryConfig {
            max_history: 50,
            auto_save: true,
            auto_save_interval_ms: 1000,
        };

        manager.set_config(config);
        assert_eq!(manager.config().max_history, 50);
        assert!(manager.config().auto_save);
    }

    /// Test: Verifies history manager execute
    #[test]
    fn test_history_manager_execute() {
        let mut manager = HistoryManager::new();
        let id = EntityId::from_u128(1);
        let command = Box::new(MockCommand::new(id));

        manager.execute(command).unwrap();
        assert_eq!(manager.history_len(), 1);
        assert_eq!(manager.snapshots().len(), 1);
        assert!(manager.can_undo());
    }

    /// Test: Verifies history manager undo/redo
    #[test]
    fn test_history_manager_undo_redo() {
        let mut manager = HistoryManager::new();
        let id = EntityId::from_u128(1);
        let command = Box::new(MockCommand::new(id));

        manager.execute(command).unwrap();
        manager.undo().unwrap();
        assert_eq!(manager.history_len(), 0);
        assert!(!manager.can_undo());
        assert!(manager.can_redo());

        manager.redo().unwrap();
        assert_eq!(manager.history_len(), 1);
        assert!(manager.can_undo());
        assert!(!manager.can_redo());
    }

    /// Test: Verifies history manager clears redo stack on execute
    #[test]
    fn test_history_manager_clears_redo_on_execute() {
        let mut manager = HistoryManager::new();
        let id1 = EntityId::from_u128(1);
        let id2 = EntityId::from_u128(2);

        manager.execute(Box::new(MockCommand::new(id1))).unwrap();
        manager.undo().unwrap();
        assert!(manager.can_redo());

        manager.execute(Box::new(MockCommand::new(id2))).unwrap();
        assert!(!manager.can_redo());
        // History should have 1 command (id2) since id1 was undone and redo cleared
        assert_eq!(manager.history_len(), 1);
    }

    /// Test: Verifies history manager max history limit
    #[test]
    fn test_history_manager_max_history() {
        let mut manager = HistoryManager::new();
        manager.set_config(HistoryConfig {
            max_history: 3,
            ..Default::default()
        });

        // Execute 5 commands
        for i in 1..=5 {
            let id = EntityId::from_u128(i);
            manager.execute(Box::new(MockCommand::new(id))).unwrap();
        }

        // Should only keep 3 due to max_history limit
        assert_eq!(manager.history_len(), 3);
    }

    /// Test: Verifies history manager clear
    #[test]
    fn test_history_manager_clear() {
        let mut manager = HistoryManager::new();
        let id = EntityId::from_u128(1);

        manager.execute(Box::new(MockCommand::new(id))).unwrap();
        manager.undo().unwrap();
        assert!(manager.can_redo());

        manager.clear();
        assert!(!manager.can_undo());
        assert!(!manager.can_redo());
        assert_eq!(manager.history_len(), 0);
    }
}
