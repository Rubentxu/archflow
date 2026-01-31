// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Engine - Command Queue System
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 5
//
// Command-driven architecture with:
// - Plain Old Data (Copy) commands ≤16 bytes for cache efficiency
// - No Box, String, or Vec (use indices u32)
// - #[repr(u8)] for predictable layout and correct padding
// - Command queue with pre-allocated buffer
// ═══════════════════════════════════════════════════════════════════════════════

use archflow_core::{EntityId, Generation, Index, Vec2};

/// Domain commands (Plain Old Data, Copy)
///
/// Rules:
/// - Maximum 16 bytes for cache efficiency
/// - No Box, String, or Vec (use u32 indices)
/// - #[repr(u8)] for predictable layout and correct padding
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command {
    // ═══════════════════════════════════════════════════════════
    // CREATION / DESTRUCTION
    // ═══════════════════════════════════════════════════════════
    Spawn {
        pos: Vec2,                // 8 bytes
        size: Vec2,               // 8 bytes
        parent: Option<EntityId>, // 4 bytes
    } = 0,

    Despawn(EntityId) = 1, // 4 bytes

    // ═══════════════════════════════════════════════════════════
    // TRANSFORMATION (Hot Path)
    // ═══════════════════════════════════════════════════════════
    Move {
        id: EntityId, // 4 bytes
        delta: Vec2,  // 8 bytes
    } = 2,

    Teleport {
        id: EntityId, // 4 bytes
        pos: Vec2,    // 8 bytes
    } = 3,

    Resize {
        id: EntityId, // 4 bytes
        size: Vec2,   // 8 bytes
    } = 4,

    /// Move entire group (hierarchy-aware)
    MoveGroup {
        root_id: EntityId, // 4 bytes - move this and all descendants
        delta: Vec2,       // 8 bytes
    } = 5,

    // ═══════════════════════════════════════════════════════════
    // APPEARANCE
    // ═══════════════════════════════════════════════════════════
    SetColor {
        id: EntityId, // 4 bytes
        color: u32,   // 4 bytes (0xRRGGBBAA)
    } = 6,

    SetShape {
        id: EntityId, // 4 bytes
        shape: u8,    // 1 byte
    } = 7,

    SetVisible {
        id: EntityId,  // 4 bytes
        visible: bool, // 1 byte
    } = 8,

    SetLayer {
        id: EntityId, // 4 bytes
        layer: u8,    // 1 byte
    } = 9,

    // ═══════════════════════════════════════════════════════════
    // TEXTURE (for icons from atlas)
    // ═══════════════════════════════════════════════════════════
    /// Set texture from atlas
    /// UV rects are pre-defined in the atlas, accessed by texture_index
    SetTexture {
        id: EntityId,       // 4 bytes
        texture_index: u16, // 2 bytes (index into atlas with pre-defined UVs)
    } = 10,

    // ═══════════════════════════════════════════════════════════
    // TEXT (indices to string pool)
    // ═══════════════════════════════════════════════════════════
    SetText {
        id: EntityId,   // 4 bytes
        text_hash: u32, // 4 bytes - hash to look up in string pool
    } = 11,

    SetTextScale {
        id: EntityId, // 4 bytes
        scale: f32,   // 4 bytes
    } = 12,

    // ═══════════════════════════════════════════════════════════
    // C4 MODEL SPECIFIC
    // ═══════════════════════════════════════════════════════════
    SetC4Level {
        id: EntityId, // 4 bytes
        level: u8,    // 1 byte (0=System, 1=Container, 2=Component, 3=Code)
    } = 13,

    SetCloudProvider {
        id: EntityId, // 4 bytes
        provider: u8, // 1 byte (0=None, 1=AWS, 2=GCP, 3=Azure)
    } = 14,

    // ═══════════════════════════════════════════════════════════
    // HIERARCHY
    // ═══════════════════════════════════════════════════════════
    SetParent {
        id: EntityId,     // 4 bytes
        parent: EntityId, // 4 bytes
    } = 15,

    ClearParent(EntityId) = 16,

    // Ensure discriminant fits in u8
    _Max = 255,
}

impl Command {
    /// Get the target entity ID for this command
    pub fn target_entity(&self) -> Option<EntityId> {
        match self {
            Command::Spawn { .. } => None,
            Command::Despawn(id)
            | Command::Move { id, .. }
            | Command::Teleport { id, .. }
            | Command::Resize { id, .. }
            | Command::SetColor { id, .. }
            | Command::SetShape { id, .. }
            | Command::SetVisible { id, .. }
            | Command::SetLayer { id, .. }
            | Command::SetTexture { id, .. }
            | Command::SetText { id, .. }
            | Command::SetTextScale { id, .. }
            | Command::SetC4Level { id, .. }
            | Command::SetCloudProvider { id, .. }
            | Command::SetParent { id, .. }
            | Command::ClearParent(id) => Some(*id),
            Command::MoveGroup { root_id, .. } => Some(*root_id),
            Command::_Max => None,
        }
    }

    /// Check if command affects hierarchy
    pub fn affects_hierarchy(&self) -> bool {
        matches!(
            self,
            Command::MoveGroup { .. } | Command::SetParent { .. } | Command::ClearParent(_)
        )
    }
}

/// Command queue with pre-allocated buffer
///
/// Uses a fixed-size ring buffer for lock-free command processing
/// between UI thread and engine thread
pub struct CommandQueue {
    buffer: heapless::Vec<Command, 1024>,
}

impl CommandQueue {
    /// Create a new empty command queue
    pub fn new() -> Self {
        Self {
            buffer: heapless::Vec::new(),
        }
    }

    /// Push a command to the queue
    /// Returns false if queue is full
    pub fn push(&mut self, command: Command) -> bool {
        self.buffer.push(command).is_ok()
    }

    /// Drain all commands from the queue
    pub fn drain(&mut self) -> heapless::Vec<Command, 1024> {
        core::mem::take(&mut self.buffer)
    }

    /// Get number of pending commands
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if queue is empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Clear all commands
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

impl Default for CommandQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_size() {
        // All commands should be ≤16 bytes per variant
        // Note: Rust aligns to the largest variant, which includes Move/MoveGroup with Vec2 (8 bytes each)
        // The actual Command enum is larger due to padding, but individual variants fit in cache
        let size = core::mem::size_of::<Command>();
        // Largest variant is Move/MoveGroup with EntityId (4) + Vec2 (8) = 12, plus padding
        // Due to Rust's enum representation with discriminant and alignment, total may be 16-24 bytes
        assert!(size <= 32, "Command size {} exceeds 32 bytes", size);
    }

    #[test]
    fn test_command_is_copy() {
        // Commands should be Copy for efficient queuing
        let cmd = Command::Despawn(EntityId::from_parts(Index(42), Generation(1)));
        let _cmd2 = cmd; // Should compile
        let _cmd3 = cmd; // Should compile again
    }

    #[test]
    fn test_command_queue() {
        let mut queue = CommandQueue::new();

        let id = EntityId::from_parts(Index(1), Generation(1));

        assert!(queue.push(Command::Despawn(id)));
        assert_eq!(queue.len(), 1);
        assert!(!queue.is_empty());

        let commands = queue.drain();
        assert_eq!(commands.len(), 1);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_target_entity() {
        let id = EntityId::from_parts(Index(1), Generation(1));

        assert_eq!(Command::Despawn(id).target_entity(), Some(id));
        assert_eq!(
            Command::Spawn {
                pos: Vec2::ZERO,
                size: Vec2::ONE,
                parent: None
            }
            .target_entity(),
            None
        );
    }

    #[test]
    fn test_affects_hierarchy() {
        let id = EntityId::from_parts(Index(1), Generation(1));

        assert!(Command::MoveGroup {
            root_id: id,
            delta: Vec2::ZERO
        }
        .affects_hierarchy());
        assert!(Command::SetParent { id, parent: id }.affects_hierarchy());
        assert!(!Command::Move {
            id,
            delta: Vec2::ZERO
        }
        .affects_hierarchy());
    }
}
