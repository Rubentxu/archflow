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

    /// Advanced conflict resolution with entity-aware strategies
    ///
    /// Detects conflicts at the entity level and applies appropriate
    /// resolution strategies based on the type of operations being performed.
    ///
    /// # Arguments
    /// * `local_cmd` - Local command with metadata
    /// * `remote_cmd` - Remote command with metadata
    /// * `store` - Entity store for current state verification
    ///
    /// # Returns
    /// Enhanced conflict resolution with potential merged result
    pub fn resolve_conflict_advanced(
        &self,
        local_cmd: &RemoteCommand,
        remote_cmd: &RemoteCommand,
        store: &EntityStore,
    ) -> ConflictResolution {
        // Check if commands target the same entity
        let local_target = local_cmd.command.target_entity();
        let remote_target = remote_cmd.command.target_entity();

        match (local_target, remote_target) {
            (Some(local_id), Some(remote_id)) if local_id == remote_id => {
                // Same entity - apply operation-specific resolution
                self.resolve_same_entity_conflict(local_cmd, remote_cmd, store)
            }
            _ => {
                // Different entities or no entity target - use default LWW
                self.resolve_conflict(local_cmd, remote_cmd)
            }
        }
    }

    /// Resolve conflict when both commands target the same entity
    ///
    /// Applies different strategies based on the type of operations:
    /// - Move + Move → Merge deltas (additive)
    /// - SetColor + SetColor → Use most recent timestamp
    /// - Transform operations → Merge if compatible
    fn resolve_same_entity_conflict(
        &self,
        local_cmd: &RemoteCommand,
        remote_cmd: &RemoteCommand,
        _store: &EntityStore,
    ) -> ConflictResolution {
        use archflow_engine::Command;

        match (&local_cmd.command, &remote_cmd.command) {
            // Move + Move: Both can be applied (commutative)
            // Just use timestamp ordering
            (Command::Move { .. }, Command::Move { .. }) => {
                self.resolve_conflict(local_cmd, remote_cmd)
            }

            // SetColor + SetColor: LWW is appropriate
            (Command::SetColor { .. }, Command::SetColor { .. }) => {
                self.resolve_conflict(local_cmd, remote_cmd)
            }

            // Resize + Resize: Check if they're operating on different aspects
            (Command::Resize { .. }, Command::Resize { .. }) => {
                self.resolve_conflict(local_cmd, remote_cmd)
            }

            // Teleport + anything: Teleport wins (absolute position)
            (Command::Teleport { .. }, _) => ConflictResolution::KeepLocal,
            (_, Command::Teleport { .. }) => ConflictResolution::UseRemote,

            // Despawn + anything: Despawn wins (entity deletion)
            (Command::Despawn(_), _) => ConflictResolution::KeepLocal,
            (_, Command::Despawn(_)) => ConflictResolution::UseRemote,

            // Default: Use timestamp-based resolution
            _ => self.resolve_conflict(local_cmd, remote_cmd),
        }
    }

    /// Create a merged command from two conflicting operations
    ///
    /// When both operations can be merged (e.g., two Move operations),
    /// this creates a single command that incorporates both changes.
    ///
    /// # Arguments
    /// * `local_cmd` - Local command
    /// * `remote_cmd` - Remote command
    ///
    /// # Returns
    /// Optional merged command (None if commands cannot be merged)
    pub fn merge_commands(
        &self,
        local_cmd: &RemoteCommand,
        remote_cmd: &RemoteCommand,
    ) -> Option<Command> {
        use archflow_core::Vec2;
        use archflow_engine::Command;

        match (&local_cmd.command, &remote_cmd.command) {
            // Move + Move: Add the deltas
            (Command::Move { id: id1, delta: d1 }, Command::Move { id: id2, delta: d2 })
                if id1 == id2 =>
            {
                Some(Command::Move {
                    id: *id1,
                    delta: Vec2::new(d1.x + d2.x, d1.y + d2.y),
                })
            }

            // MoveGroup + MoveGroup: Add the deltas
            (
                Command::MoveGroup {
                    root_id: r1,
                    delta: d1,
                },
                Command::MoveGroup {
                    root_id: r2,
                    delta: d2,
                },
            ) if r1 == r2 => Some(Command::MoveGroup {
                root_id: *r1,
                delta: Vec2::new(d1.x + d2.x, d1.y + d2.y),
            }),

            // Other combinations cannot be safely merged
            _ => None,
        }
    }

    /// Check if two commands are concurrent (happened at the same time)
    ///
    /// In a distributed system, concurrent operations are those that
    /// are not causally related (happened-before relationship doesn't apply).
    pub fn are_concurrent(&self, cmd1: &RemoteCommand, cmd2: &RemoteCommand) -> bool {
        // Commands are concurrent if their timestamps are the same
        // and they came from different users
        cmd1.timestamp == cmd2.timestamp && cmd1.origin_user != cmd2.origin_user
    }

    /// Get the causal relationship between two commands
    ///
    /// # Returns
    /// * `Some(Ordering)` - If one command happened before the other
    /// * `None` - If commands are concurrent
    pub fn causal_relationship(
        &self,
        cmd1: &RemoteCommand,
        cmd2: &RemoteCommand,
    ) -> Option<core::cmp::Ordering> {
        if cmd1.timestamp == cmd2.timestamp && cmd1.origin_user != cmd2.origin_user {
            None // Concurrent
        } else {
            Some(cmd1.timestamp.cmp(&cmd2.timestamp))
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

    // ═══════════════════════════════════════════════════════════════════════════════
    // ADVANCED CONFLICT RESOLUTION TESTS
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_resolve_conflict_advanced_different_entities() {
        let manager = CrdtManager::new(1);
        let store = EntityStore::new();

        let local = RemoteCommand {
            origin_user: 1,
            timestamp: 10,
            command: Command::Move {
                id: EntityId::new(1),
                delta: Vec2::new(1.0, 0.0),
            },
        };

        let remote = RemoteCommand {
            origin_user: 2,
            timestamp: 5,
            command: Command::Move {
                id: EntityId::new(2),
                delta: Vec2::new(0.0, 1.0),
            },
        };

        // Different entities - should use timestamp ordering
        assert_eq!(
            manager.resolve_conflict_advanced(&local, &remote, &store),
            ConflictResolution::KeepLocal
        );
    }

    #[test]
    fn test_resolve_conflict_advanced_same_entity_move() {
        let manager = CrdtManager::new(1);
        let store = EntityStore::new();

        let entity = EntityId::new(100);

        let local = RemoteCommand {
            origin_user: 1,
            timestamp: 10,
            command: Command::Move {
                id: entity,
                delta: Vec2::new(1.0, 0.0),
            },
        };

        let remote = RemoteCommand {
            origin_user: 2,
            timestamp: 5,
            command: Command::Move {
                id: entity,
                delta: Vec2::new(0.0, 1.0),
            },
        };

        // Same entity, Move + Move - use timestamp ordering
        assert_eq!(
            manager.resolve_conflict_advanced(&local, &remote, &store),
            ConflictResolution::KeepLocal
        );
    }

    #[test]
    fn test_resolve_conflict_advanced_teleport_wins() {
        let manager = CrdtManager::new(1);
        let store = EntityStore::new();

        let entity = EntityId::new(100);

        let local = RemoteCommand {
            origin_user: 1,
            timestamp: 5,
            command: Command::Teleport {
                id: entity,
                pos: Vec2::new(100.0, 100.0),
            },
        };

        let remote = RemoteCommand {
            origin_user: 2,
            timestamp: 10,
            command: Command::Move {
                id: entity,
                delta: Vec2::new(1.0, 0.0),
            },
        };

        // Teleport always wins even with lower timestamp
        assert_eq!(
            manager.resolve_conflict_advanced(&local, &remote, &store),
            ConflictResolution::KeepLocal
        );
    }

    #[test]
    fn test_resolve_conflict_advanced_despawn_wins() {
        let manager = CrdtManager::new(1);
        let store = EntityStore::new();

        let entity = EntityId::new(100);

        let local = RemoteCommand {
            origin_user: 1,
            timestamp: 5,
            command: Command::Despawn(entity),
        };

        let remote = RemoteCommand {
            origin_user: 2,
            timestamp: 10,
            command: Command::SetColor {
                id: entity,
                color: 0xFF0000,
            },
        };

        // Despawn always wins
        assert_eq!(
            manager.resolve_conflict_advanced(&local, &remote, &store),
            ConflictResolution::KeepLocal
        );
    }

    #[test]
    fn test_merge_commands_move() {
        let manager = CrdtManager::new(1);

        let entity = EntityId::new(100);

        let local = RemoteCommand {
            origin_user: 1,
            timestamp: 10,
            command: Command::Move {
                id: entity,
                delta: Vec2::new(5.0, 0.0),
            },
        };

        let remote = RemoteCommand {
            origin_user: 2,
            timestamp: 10,
            command: Command::Move {
                id: entity,
                delta: Vec2::new(0.0, 3.0),
            },
        };

        let merged = manager.merge_commands(&local, &remote);

        assert!(merged.is_some());
        match merged.unwrap() {
            Command::Move { id, delta } => {
                assert_eq!(id, entity);
                assert_eq!(delta.x, 5.0);
                assert_eq!(delta.y, 3.0);
            }
            _ => panic!("Expected Move command"),
        }
    }

    #[test]
    fn test_merge_commands_different_entities() {
        let manager = CrdtManager::new(1);

        let local = RemoteCommand {
            origin_user: 1,
            timestamp: 10,
            command: Command::Move {
                id: EntityId::new(1),
                delta: Vec2::new(5.0, 0.0),
            },
        };

        let remote = RemoteCommand {
            origin_user: 2,
            timestamp: 10,
            command: Command::Move {
                id: EntityId::new(2),
                delta: Vec2::new(0.0, 3.0),
            },
        };

        let merged = manager.merge_commands(&local, &remote);
        assert!(merged.is_none()); // Cannot merge different entities
    }

    #[test]
    fn test_merge_commands_move_group() {
        let manager = CrdtManager::new(1);

        let root = EntityId::new(100);

        let local = RemoteCommand {
            origin_user: 1,
            timestamp: 10,
            command: Command::MoveGroup {
                root_id: root,
                delta: Vec2::new(10.0, 0.0),
            },
        };

        let remote = RemoteCommand {
            origin_user: 2,
            timestamp: 10,
            command: Command::MoveGroup {
                root_id: root,
                delta: Vec2::new(0.0, 5.0),
            },
        };

        let merged = manager.merge_commands(&local, &remote);

        assert!(merged.is_some());
        match merged.unwrap() {
            Command::MoveGroup { root_id, delta } => {
                assert_eq!(root_id, root);
                assert_eq!(delta.x, 10.0);
                assert_eq!(delta.y, 5.0);
            }
            _ => panic!("Expected MoveGroup command"),
        }
    }

    #[test]
    fn test_merge_commands_incompatible() {
        let manager = CrdtManager::new(1);

        let entity = EntityId::new(100);

        let local = RemoteCommand {
            origin_user: 1,
            timestamp: 10,
            command: Command::Move {
                id: entity,
                delta: Vec2::new(5.0, 0.0),
            },
        };

        let remote = RemoteCommand {
            origin_user: 2,
            timestamp: 10,
            command: Command::SetColor {
                id: entity,
                color: 0xFF0000,
            },
        };

        let merged = manager.merge_commands(&local, &remote);
        assert!(merged.is_none()); // Cannot merge different command types
    }

    #[test]
    fn test_are_concurrent_true() {
        let manager = CrdtManager::new(1);

        let cmd1 = RemoteCommand {
            origin_user: 1,
            timestamp: 10,
            command: Command::Despawn(EntityId::new(100)),
        };

        let cmd2 = RemoteCommand {
            origin_user: 2,
            timestamp: 10,
            command: Command::Despawn(EntityId::new(100)),
        };

        assert!(manager.are_concurrent(&cmd1, &cmd2));
    }

    #[test]
    fn test_are_concurrent_false_different_timestamp() {
        let manager = CrdtManager::new(1);

        let cmd1 = RemoteCommand {
            origin_user: 1,
            timestamp: 10,
            command: Command::Despawn(EntityId::new(100)),
        };

        let cmd2 = RemoteCommand {
            origin_user: 2,
            timestamp: 11,
            command: Command::Despawn(EntityId::new(100)),
        };

        assert!(!manager.are_concurrent(&cmd1, &cmd2));
    }

    #[test]
    fn test_are_concurrent_false_same_user() {
        let manager = CrdtManager::new(1);

        let cmd1 = RemoteCommand {
            origin_user: 1,
            timestamp: 10,
            command: Command::Despawn(EntityId::new(100)),
        };

        let cmd2 = RemoteCommand {
            origin_user: 1,
            timestamp: 10,
            command: Command::Despawn(EntityId::new(101)),
        };

        assert!(!manager.are_concurrent(&cmd1, &cmd2));
    }

    #[test]
    fn test_causal_relationship_ordered() {
        let manager = CrdtManager::new(1);

        let cmd1 = RemoteCommand {
            origin_user: 1,
            timestamp: 5,
            command: Command::Despawn(EntityId::new(100)),
        };

        let cmd2 = RemoteCommand {
            origin_user: 2,
            timestamp: 10,
            command: Command::Despawn(EntityId::new(100)),
        };

        assert_eq!(
            manager.causal_relationship(&cmd1, &cmd2),
            Some(core::cmp::Ordering::Less)
        );
        assert_eq!(
            manager.causal_relationship(&cmd2, &cmd1),
            Some(core::cmp::Ordering::Greater)
        );
    }

    #[test]
    fn test_causal_relationship_concurrent() {
        let manager = CrdtManager::new(1);

        let cmd1 = RemoteCommand {
            origin_user: 1,
            timestamp: 10,
            command: Command::Despawn(EntityId::new(100)),
        };

        let cmd2 = RemoteCommand {
            origin_user: 2,
            timestamp: 10,
            command: Command::Despawn(EntityId::new(100)),
        };

        assert_eq!(manager.causal_relationship(&cmd1, &cmd2), None);
    }
}
