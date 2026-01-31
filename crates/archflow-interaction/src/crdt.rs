// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Interaction - CRDT Manager (Real-time Collaboration)
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 17
//
// CRDT-based multi-user collaboration with:
// - Lamport timestamps for total ordering
// - Conflict resolution for concurrent edits
// - Remote command processing
// ═══════════════════════════════════════════════════════════════════════════════

extern crate alloc;

use alloc::collections::VecDeque;
use alloc::vec::Vec;

use archflow_engine::{Command, EntityStore};

/// Remote command with metadata for synchronization
///
/// Contains the command along with origin user and Lamport timestamp
/// for total ordering across distributed systems.
#[derive(Clone, Debug)]
pub struct RemoteCommand {
    /// User ID who originated this command
    pub origin_user: u32,
    /// Lamport timestamp for total ordering
    pub timestamp: u64,
    /// The actual command to execute
    pub command: Command,
}

/// CRDT manager for real-time collaboration
///
/// Manages command synchronization across multiple users using
/// Lamport timestamps and conflict resolution strategies.
pub struct CrdtManager {
    /// This user's ID
    user_id: u32,
    /// Lamport logical clock
    lamport_clock: u64,
    /// Pending commands to process
    pending: VecDeque<RemoteCommand>,
}

impl CrdtManager {
    /// Create a new CRDT manager
    ///
    /// # Arguments
    /// * `user_id` - Unique identifier for this user
    pub fn new(user_id: u32) -> Self {
        Self {
            user_id,
            lamport_clock: 0,
            pending: VecDeque::new(),
        }
    }

    /// Get the current user ID
    #[inline]
    pub fn user_id(&self) -> u32 {
        self.user_id
    }

    /// Get the current Lamport clock value
    #[inline]
    pub fn lamport_clock(&self) -> u64 {
        self.lamport_clock
    }

    /// Apply a local command and prepare for broadcast
    ///
    /// # Arguments
    /// * `cmd` - The command to apply
    ///
    /// # Returns
    /// RemoteCommand ready to broadcast to other users
    pub fn apply_local(&mut self, cmd: Command) -> RemoteCommand {
        self.lamport_clock += 1;

        RemoteCommand {
            origin_user: self.user_id,
            timestamp: self.lamport_clock,
            command: cmd,
        }
    }

    /// Apply a remote command from another user
    ///
    /// # Arguments
    /// * `store` - Entity store to apply the command
    /// * `remote` - Remote command to apply
    pub fn apply_remote(&mut self, store: &mut EntityStore, remote: RemoteCommand) {
        // Update Lamport clock (max of both clocks + 1)
        self.lamport_clock = self.lamport_clock.max(remote.timestamp) + 1;

        // Apply command
        remote.command.execute(store);
    }

    /// Add a pending command to be processed later
    pub fn add_pending(&mut self, remote: RemoteCommand) {
        self.pending.push_back(remote);
    }

    /// Process all pending commands in timestamp order
    ///
    /// # Arguments
    /// * `store` - Entity store to apply commands to
    pub fn process_pending(&mut self, store: &mut EntityStore) {
        // Sort pending commands by timestamp
        let mut commands: Vec<_> = self.pending.drain(..).collect();
        commands.sort_by_key(|c| c.timestamp);

        for remote in commands {
            self.apply_remote(store, remote);
        }
    }

    /// Get the number of pending commands
    #[inline]
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Check if there are pending commands
    #[inline]
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    /// Clear all pending commands
    pub fn clear_pending(&mut self) {
        self.pending.clear();
    }

    /// Resolve conflict between concurrent commands
    ///
    /// Uses "Last Write Wins" strategy based on Lamport timestamps,
    /// with user ID as tie-breaker.
    ///
    /// # Arguments
    /// * `local_cmd` - Local command with metadata
    /// * `remote_cmd` - Remote command with metadata
    ///
    /// # Returns
    /// Conflict resolution decision
    pub fn resolve_conflict(
        &self,
        local_cmd: &RemoteCommand,
        remote_cmd: &RemoteCommand,
    ) -> ConflictResolution {
        match local_cmd.timestamp.cmp(&remote_cmd.timestamp) {
            core::cmp::Ordering::Greater => ConflictResolution::KeepLocal,
            core::cmp::Ordering::Less => ConflictResolution::UseRemote,
            core::cmp::Ordering::Equal => {
                // Tie-breaker: higher user ID wins (deterministic)
                match local_cmd.origin_user.cmp(&remote_cmd.origin_user) {
                    core::cmp::Ordering::Greater => ConflictResolution::KeepLocal,
                    _ => ConflictResolution::UseRemote,
                }
            }
        }
    }
}

impl Default for CrdtManager {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Conflict resolution strategy
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConflictResolution {
    /// Keep the local version
    KeepLocal,
    /// Use the remote version
    UseRemote,
    /// Merge both versions (not yet implemented)
    Merge,
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use archflow_core::{EntityId, Vec2};

    #[test]
    fn test_crdt_manager_creation() {
        let manager = CrdtManager::new(42);
        assert_eq!(manager.user_id(), 42);
        assert_eq!(manager.lamport_clock(), 0);
        assert_eq!(manager.pending_count(), 0);
    }

    #[test]
    fn test_crdt_manager_default() {
        let manager = CrdtManager::default();
        assert_eq!(manager.user_id(), 0);
    }

    #[test]
    fn test_apply_local_increments_clock() {
        let mut manager = CrdtManager::new(1);

        let cmd = Command::Despawn(EntityId::new(100));
        let remote = manager.apply_local(cmd);

        assert_eq!(manager.lamport_clock(), 1);
        assert_eq!(remote.origin_user, 1);
        assert_eq!(remote.timestamp, 1);
    }

    #[test]
    fn test_apply_local_multiple() {
        let mut manager = CrdtManager::new(1);

        manager.apply_local(Command::Despawn(EntityId::new(100)));
        assert_eq!(manager.lamport_clock(), 1);

        manager.apply_local(Command::Despawn(EntityId::new(101)));
        assert_eq!(manager.lamport_clock(), 2);

        manager.apply_local(Command::Despawn(EntityId::new(102)));
        assert_eq!(manager.lamport_clock(), 3);
    }

    #[test]
    fn test_apply_remote_updates_clock() {
        let mut manager = CrdtManager::new(1);
        let mut store = EntityStore::new();

        // First spawn an entity to get a valid ID
        let entity = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));

        let remote_cmd = RemoteCommand {
            origin_user: 2,
            timestamp: 10,
            command: Command::Despawn(entity),
        };

        manager.apply_remote(&mut store, remote_cmd);

        // Clock should be max(0, 10) + 1 = 11
        assert_eq!(manager.lamport_clock(), 11);
    }

    #[test]
    fn test_pending_commands() {
        let mut manager = CrdtManager::new(1);

        let remote = RemoteCommand {
            origin_user: 2,
            timestamp: 5,
            command: Command::Despawn(EntityId::new(100)),
        };

        manager.add_pending(remote);
        assert_eq!(manager.pending_count(), 1);
        assert!(manager.has_pending());
    }

    #[test]
    fn test_clear_pending() {
        let mut manager = CrdtManager::new(1);

        let remote = RemoteCommand {
            origin_user: 2,
            timestamp: 5,
            command: Command::Despawn(EntityId::new(100)),
        };

        manager.add_pending(remote);
        manager.clear_pending();

        assert_eq!(manager.pending_count(), 0);
        assert!(!manager.has_pending());
    }

    #[test]
    fn test_resolve_conflict_by_timestamp() {
        let manager = CrdtManager::new(1);

        let local = RemoteCommand {
            origin_user: 1,
            timestamp: 10,
            command: Command::Despawn(EntityId::new(100)),
        };

        let remote = RemoteCommand {
            origin_user: 2,
            timestamp: 5,
            command: Command::Despawn(EntityId::new(100)),
        };

        // Local has higher timestamp, should win
        assert_eq!(
            manager.resolve_conflict(&local, &remote),
            ConflictResolution::KeepLocal
        );
    }

    #[test]
    fn test_resolve_conflict_by_user_id() {
        let manager = CrdtManager::new(1);

        let local = RemoteCommand {
            origin_user: 2,
            timestamp: 10,
            command: Command::Despawn(EntityId::new(100)),
        };

        let remote = RemoteCommand {
            origin_user: 1,
            timestamp: 10,
            command: Command::Despawn(EntityId::new(100)),
        };

        // Same timestamp, higher user ID wins
        assert_eq!(
            manager.resolve_conflict(&local, &remote),
            ConflictResolution::KeepLocal
        );
    }

    #[test]
    fn test_resolve_conflict_remote_wins() {
        let manager = CrdtManager::new(1);

        let local = RemoteCommand {
            origin_user: 1,
            timestamp: 5,
            command: Command::Despawn(EntityId::new(100)),
        };

        let remote = RemoteCommand {
            origin_user: 2,
            timestamp: 10,
            command: Command::Despawn(EntityId::new(100)),
        };

        // Remote has higher timestamp
        assert_eq!(
            manager.resolve_conflict(&local, &remote),
            ConflictResolution::UseRemote
        );
    }

    #[test]
    fn test_remote_command_clone() {
        let cmd = RemoteCommand {
            origin_user: 1,
            timestamp: 100,
            command: Command::Despawn(EntityId::new(42)),
        };

        let _cmd2 = cmd.clone();
        // Test Clone derive
    }

    #[test]
    fn test_conflict_resolution_eq() {
        assert_eq!(ConflictResolution::KeepLocal, ConflictResolution::KeepLocal);
        assert_eq!(ConflictResolution::UseRemote, ConflictResolution::UseRemote);
        assert_ne!(ConflictResolution::KeepLocal, ConflictResolution::UseRemote);
    }
}
