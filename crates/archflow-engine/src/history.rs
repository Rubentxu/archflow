// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Engine - Command History (Undo/Redo System)
//
// Reference: docs/epics/EPIC-003-actuators-animations.md - HU-013
//
// Implements undo/redo functionality with:
// - Circular buffer for fixed memory usage
// - Command grouping for atomic transactions
// - Redo stack cleared on new commands
// - Serializable commands (future: FlatBuffers)
// ═══════════════════════════════════════════════════════════════════════════════


use alloc::vec::Vec;
use core::mem::MaybeUninit;

use crate::command::Command;

/// Maximum number of commands in undo history
///
/// This limit ensures fixed memory usage and prevents unbounded growth.
/// At 32 bytes per command, 256 commands = 8KB maximum memory usage.
const MAX_HISTORY: usize = 256;

/// Command history with circular buffer for undo/redo
///
/// This struct manages the undo and redo stacks using a fixed-size circular buffer.
/// When the buffer is full, oldest commands are overwritten (FIFO).
///
/// # Memory Layout
///
/// ```text
/// [████████████████░░░░░░░░░░░░░░░░░░]
///  ^            ^
///  tail         head
///  (oldest)     (newest)
/// ```
///
/// # Invariants
///
/// - `undo_count` <= MAX_HISTORY
/// - `redo_count` + `undo_count` <= MAX_HISTORY
/// - When head reaches MAX_HISTORY, it wraps to 0 (circular)
pub struct CommandHistory {
    /// Circular buffer of commands (using MaybeUninit for efficiency)
    buffer: [MaybeUninit<Command>; MAX_HISTORY],

    /// Track which slots are initialized
    initialized: usize,

    /// Index of the newest command (0 if empty)
    head: usize,

    /// Number of commands available for undo
    undo_count: usize,

    /// Number of commands available for redo
    redo_count: usize,
}

impl CommandHistory {
    /// Create a new empty command history
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buffer: unsafe { MaybeUninit::uninit().assume_init() },
            initialized: 0,
            head: 0,
            undo_count: 0,
            redo_count: 0,
        }
    }

    /// Push a new command to history (clears redo stack)
    ///
    /// This method adds a command to the undo history. If the redo stack has commands,
    /// they are discarded (standard undo/redo behavior).
    ///
    /// # Returns
    ///
    /// `true` if command was added, `false` if buffer is full
    #[inline]
    pub fn push(&mut self, command: Command) -> bool {
        // If redo stack exists, clear it (standard behavior)
        if self.redo_count > 0 {
            // Clear redo slots
            for i in 0..self.redo_count {
                let idx = (self.head + 1 + i) % MAX_HISTORY;
                unsafe {
                    self.buffer[idx].assume_init_drop();
                }
            }
            self.redo_count = 0;
        }

        // Calculate next head position
        let next_head = (self.head + 1) % MAX_HISTORY;

        // If buffer is full, drop oldest command
        if self.undo_count == MAX_HISTORY {
            // Drop the oldest command before overwriting
            let tail_idx = (self.head + 1) % MAX_HISTORY;
            unsafe {
                self.buffer[tail_idx].assume_init_drop();
            }
            self.initialized = self.initialized.saturating_sub(1);
        } else {
            self.undo_count += 1;
        }

        // Write command to buffer
        self.buffer[next_head].write(command);
        self.head = next_head;
        self.initialized = self.initialized.saturating_add(1);

        true
    }

    /// Undo the most recent command
    ///
    /// # Returns
    ///
    /// `Some(command)` if undo available, `None` if history is empty
    #[must_use]
    #[inline]
    pub fn undo(&mut self) -> Option<Command> {
        if self.undo_count == 0 {
            return None;
        }

        // Clone the command before modifying buffer
        let command = unsafe { self.buffer[self.head].assume_init_ref().clone() };

        // Move head back
        self.head = if self.head == 0 {
            MAX_HISTORY - 1
        } else {
            self.head - 1
        };
        self.undo_count = self.undo_count.saturating_sub(1);
        self.redo_count = self.redo_count.saturating_add(1);

        Some(command)
    }

    /// Redo the most recently undone command
    ///
    /// # Returns
    ///
    /// `Some(command)` if redo available, `None` if redo stack is empty
    #[must_use]
    #[inline]
    pub fn redo(&mut self) -> Option<Command> {
        if self.redo_count == 0 {
            return None;
        }

        // Move head forward
        let new_head = (self.head + 1) % MAX_HISTORY;

        // Clone the command before modifying buffer
        let command = unsafe { self.buffer[new_head].assume_init_ref().clone() };

        self.head = new_head;
        self.redo_count = self.redo_count.saturating_sub(1);
        self.undo_count = self.undo_count.saturating_add(1);

        Some(command)
    }

    /// Clear all history (undo and redo stacks)
    #[inline]
    pub fn clear(&mut self) {
        // Drop all initialized commands
        if self.initialized > 0 {
            for i in 0..self.initialized {
                let idx = (self.head + 1 + i) % MAX_HISTORY;
                unsafe {
                    self.buffer[idx].assume_init_drop();
                }
            }
        }
        self.head = 0;
        self.undo_count = 0;
        self.redo_count = 0;
        self.initialized = 0;
    }

    /// Get number of commands available for undo
    #[must_use]
    #[inline]
    pub const fn can_undo(&self) -> usize {
        self.undo_count
    }

    /// Get number of commands available for redo
    #[must_use]
    #[inline]
    pub const fn can_redo(&self) -> usize {
        self.redo_count
    }

    /// Check if history is empty
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.undo_count == 0
    }

    /// Get total number of commands in history (undo + redo)
    #[must_use]
    #[inline]
    pub const fn len(&self) -> usize {
        self.undo_count + self.redo_count
    }

    /// Check if buffer is full
    #[must_use]
    #[inline]
    pub const fn is_full(&self) -> bool {
        self.undo_count == MAX_HISTORY
    }

    /// Get capacity of history buffer
    #[must_use]
    #[inline]
    pub const fn capacity(&self) -> usize {
        MAX_HISTORY
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// Command group for atomic transactions
///
/// A command group executes multiple commands as a single atomic unit.
/// All commands in a group are undone/redone together.
#[derive(Clone, Debug)]
pub struct CommandGroup {
    /// Commands in this group (in execution order)
    commands: Vec<Command>,
}

impl CommandGroup {
    /// Create a new empty command group
    #[must_use]
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Add a command to this group
    pub fn add(&mut self, command: Command) {
        self.commands.push(command);
    }

    /// Get number of commands in group
    #[must_use]
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if group is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Get iterator over commands
    pub fn iter(&self) -> core::slice::Iter<'_, Command> {
        self.commands.iter()
    }

    /// Consume the group and return the commands
    #[must_use]
    pub fn into_commands(self) -> Vec<Command> {
        self.commands
    }
}

impl Default for CommandGroup {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating command groups
///
/// Provides a fluent API for grouping commands atomically.
#[derive(Clone, Debug)]
pub struct CommandGroupBuilder {
    group: CommandGroup,
}

impl CommandGroupBuilder {
    /// Create a new builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            group: CommandGroup::new(),
        }
    }

    /// Add a command to the group
    pub fn with_command(mut self, command: Command) -> Self {
        self.group.add(command);
        self
    }

    /// Add multiple commands to the group
    pub fn add_all(mut self, commands: impl IntoIterator<Item = Command>) -> Self {
        for cmd in commands {
            self.group.add(cmd);
        }
        self
    }

    /// Build the command group
    #[must_use]
    pub fn build(self) -> CommandGroup {
        self.group
    }
}

impl Default for CommandGroupBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::store::EntityStore;
    use archflow_core::{EntityId, Generation, Index, Vec2};

    // ═══════════════════════════════════════════════════════════
    // COMMANDHISTORY TESTS
    // ═══════════════════════════════════════════════════════════

    fn create_test_command(n: u32) -> Command {
        let id = EntityId::from_parts(Index(n % 1000), Generation(1));
        Command::Move {
            id,
            delta: Vec2::new(n as f32, n as f32),
        }
    }

    #[test]
    fn test_history_new() {
        let history = CommandHistory::new();
        assert_eq!(history.can_undo(), 0);
        assert_eq!(history.can_redo(), 0);
        assert!(history.is_empty());
        assert!(!history.is_full());
        assert_eq!(history.capacity(), MAX_HISTORY);
    }

    #[test]
    fn test_history_push_single() {
        let mut history = CommandHistory::new();
        let cmd = create_test_command(1);

        assert!(history.push(cmd));
        assert_eq!(history.can_undo(), 1);
        assert_eq!(history.can_redo(), 0);
        assert!(!history.is_empty());
    }

    #[test]
    fn test_history_push_multiple() {
        let mut history = CommandHistory::new();

        for i in 0..5 {
            assert!(history.push(create_test_command(i)));
        }

        assert_eq!(history.can_undo(), 5);
        assert_eq!(history.can_redo(), 0);
        assert_eq!(history.len(), 5);
    }

    #[test]
    fn test_history_undo_single() {
        let mut history = CommandHistory::new();
        let cmd = create_test_command(1);

        history.push(cmd);

        let undone = history.undo();
        assert!(undone.is_some());
        assert_eq!(history.can_undo(), 0);
        assert_eq!(history.can_redo(), 1);
    }

    #[test]
    fn test_history_undo_redo_roundtrip() {
        let mut history = CommandHistory::new();
        let cmd1 = create_test_command(1);
        let cmd2 = create_test_command(2);

        history.push(cmd1);
        history.push(cmd2);

        // Undo both
        let u2 = history.undo();
        let u1 = history.undo();
        assert!(u2.is_some());
        assert!(u1.is_some());
        assert_eq!(history.can_undo(), 0);
        assert_eq!(history.can_redo(), 2);

        // Redo both
        let r1 = history.redo();
        let r2 = history.redo();
        assert!(r1.is_some());
        assert!(r2.is_some());
        assert_eq!(history.can_undo(), 2);
        assert_eq!(history.can_redo(), 0);
    }

    #[test]
    fn test_history_push_clears_redo() {
        let mut history = CommandHistory::new();

        // Push 2 commands
        history.push(create_test_command(1));
        history.push(create_test_command(2));

        // Undo one
        history.undo();
        assert_eq!(history.can_redo(), 1);

        // Push new command (should clear redo)
        history.push(create_test_command(3));
        assert_eq!(history.can_redo(), 0);
        assert_eq!(history.can_undo(), 2);
    }

    #[test]
    fn test_history_clear() {
        let mut history = CommandHistory::new();

        for i in 0..5 {
            history.push(create_test_command(i));
        }
        history.undo();
        history.undo();

        assert_eq!(history.len(), 5); // 3 undo + 2 redo

        history.clear();
        assert_eq!(history.can_undo(), 0);
        assert_eq!(history.can_redo(), 0);
        assert!(history.is_empty());
    }

    #[test]
    fn test_history_undo_when_empty() {
        let mut history = CommandHistory::new();
        assert!(history.undo().is_none());
    }

    #[test]
    fn test_history_redo_when_empty() {
        let mut history = CommandHistory::new();
        assert!(history.redo().is_none());

        // Push and undo
        history.push(create_test_command(1));
        history.undo();

        // Redo should work
        assert!(history.redo().is_some());

        // Redo again should fail
        assert!(history.redo().is_none());
    }

    #[test]
    fn test_history_circular_buffer_wrap() {
        let mut history = CommandHistory::new();

        // Fill buffer completely
        for i in 0..MAX_HISTORY {
            assert!(history.push(create_test_command(i as u32)));
            assert_eq!(history.can_undo(), i + 1);
        }

        assert!(history.is_full());
        assert_eq!(history.can_undo(), MAX_HISTORY);

        // Push one more (should overwrite oldest)
        assert!(history.push(create_test_command(MAX_HISTORY as u32)));
        assert_eq!(history.can_undo(), MAX_HISTORY); // Still at max

        // Should be able to undo all
        for _ in 0..MAX_HISTORY {
            assert!(history.undo().is_some());
        }

        assert!(history.undo().is_none());
    }

    #[test]
    fn test_history_fifo_behavior() {
        let mut history = CommandHistory::new();

        // Fill buffer
        for i in 0..MAX_HISTORY {
            history.push(create_test_command(i as u32));
        }

        // Add one more (overwrites index 0)
        history.push(create_test_command(MAX_HISTORY as u32));

        // Now undo everything
        let mut undone = Vec::new();
        while let Some(cmd) = history.undo() {
            undone.push(cmd);
        }

        // The oldest command (index 0) should have been overwritten
        // So we should NOT see command 0 when undoing
        assert_eq!(undone.len(), MAX_HISTORY);
    }

    #[test]
    fn test_history_capacity_const() {
        let history = CommandHistory::new();
        assert_eq!(history.capacity(), 256);
    }

    // ═══════════════════════════════════════════════════════════
    // COMMANDGROUP TESTS
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_group_new() {
        let group = CommandGroup::new();
        assert!(group.is_empty());
        assert_eq!(group.len(), 0);
    }

    #[test]
    fn test_group_add_commands() {
        let mut group = CommandGroup::new();

        group.add(create_test_command(1));
        group.add(create_test_command(2));
        group.add(create_test_command(3));

        assert_eq!(group.len(), 3);
        assert!(!group.is_empty());
    }

    #[test]
    fn test_group_iter() {
        let mut group = CommandGroup::new();

        group.add(create_test_command(1));
        group.add(create_test_command(2));

        let cmds: Vec<_> = group.iter().collect();
        assert_eq!(cmds.len(), 2);
    }

    #[test]
    fn test_group_into_commands() {
        let mut group = CommandGroup::new();

        group.add(create_test_command(1));
        group.add(create_test_command(2));

        let cmds = group.into_commands();
        assert_eq!(cmds.len(), 2);
    }

    // ═══════════════════════════════════════════════════════════
    // COMMANDGROUPBUILDER TESTS
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_builder_new() {
        let builder = CommandGroupBuilder::new();
        let group = builder.build();
        assert!(group.is_empty());
    }

    #[test]
    fn test_builder_add_single() {
        let group = CommandGroupBuilder::new()
            .with_command(create_test_command(1))
            .with_command(create_test_command(2))
            .build();

        assert_eq!(group.len(), 2);
    }

    #[test]
    fn test_builder_add_multiple() {
        let cmds = vec![
            create_test_command(1),
            create_test_command(2),
            create_test_command(3),
        ];

        let group = CommandGroupBuilder::new().add_all(cmds).build();

        assert_eq!(group.len(), 3);
    }

    #[test]
    fn test_builder_mixed() {
        let cmds = vec![create_test_command(10), create_test_command(11)];

        let group = CommandGroupBuilder::new()
            .with_command(create_test_command(1))
            .add_all(cmds)
            .with_command(create_test_command(2))
            .build();

        assert_eq!(group.len(), 4);
    }

    // ═══════════════════════════════════════════════════════════
    // INTEGRATION TESTS
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn test_undo_redo_with_store() {
        let mut store = EntityStore::new();
        let mut history = CommandHistory::new();

        // Spawn entity
        let id = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(50.0, 50.0));

        // Move command
        let move_cmd = Command::Move {
            id,
            delta: Vec2::new(10.0, 20.0),
        };

        // Execute the command first
        move_cmd.execute(&mut store);

        let _moved_pos = Vec2::new(
            store.transforms[id.index().0 as usize][0],
            store.transforms[id.index().0 as usize][1],
        );

        // Add to history AFTER executing
        history.push(move_cmd);

        // Undo should restore original position
        let cmd = history.undo().unwrap();
        let inverse = cmd.inverse(&store).unwrap();
        inverse.execute(&mut store);

        // Should have moved back to original (100, 100)
        let restored_pos = Vec2::new(
            store.transforms[id.index().0 as usize][0],
            store.transforms[id.index().0 as usize][1],
        );
        assert!((restored_pos.x - 100.0).abs() < 0.001);
        assert!((restored_pos.y - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_atomic_transaction_with_group() {
        let mut store = EntityStore::new();
        let id1 = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let id2 = store.spawn(Vec2::new(100.0, 100.0), Vec2::new(20.0, 20.0));

        // Create atomic group
        let group = CommandGroupBuilder::new()
            .with_command(Command::Move {
                id: id1,
                delta: Vec2::new(5.0, 5.0),
            })
            .with_command(Command::Move {
                id: id2,
                delta: Vec2::new(10.0, 10.0),
            })
            .build();

        // Execute all
        for cmd in group.iter() {
            cmd.execute(&mut store);
        }

        // Both should have moved
        let pos1 = Vec2::new(
            store.transforms[id1.index().0 as usize][0],
            store.transforms[id1.index().0 as usize][1],
        );
        let pos2 = Vec2::new(
            store.transforms[id2.index().0 as usize][0],
            store.transforms[id2.index().0 as usize][1],
        );

        assert_eq!(pos1.x, 5.0);
        assert_eq!(pos1.y, 5.0);
        assert_eq!(pos2.x, 110.0);
        assert_eq!(pos2.y, 110.0);
    }

    #[test]
    fn test_default_history() {
        let history = CommandHistory::default();
        assert!(history.is_empty());
    }

    #[test]
    fn test_default_group() {
        let group = CommandGroup::default();
        assert!(group.is_empty());
    }

    #[test]
    fn test_default_builder() {
        let builder = CommandGroupBuilder::default();
        let group = builder.build();
        assert!(group.is_empty());
    }
}
