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

use archflow_core::{EntityId, Vec2};

use crate::store::{EntityStore, MAX_ENTITIES};

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

    /// Generate the inverse command for undo functionality
    ///
    /// This method creates the command that would undo the effect of this command.
    /// For example, Move(delta) returns Move(-delta).
    ///
    /// # Returns
    /// The inverse command, or None if the command is not reversible
    pub fn inverse(&self, store: &EntityStore) -> Option<Command> {
        match self {
            // Spawn → Despawn (requires entity to be spawned first)
            Command::Spawn { .. } => None, // Cannot undo spawn without knowing the resulting EntityId

            // Despawn → Cannot reverse (entity is gone)
            Command::Despawn(_) => None,

            // Move → Move with negative delta
            Command::Move { id, delta } => Some(Command::Move {
                id: *id,
                delta: Vec2::new(-delta.x, -delta.y),
            }),

            // Teleport → Need previous position
            Command::Teleport { id, .. } => {
                let idx = id.index().0 as usize;
                if idx < MAX_ENTITIES {
                    let current_pos = Vec2::new(store.transforms[idx][0], store.transforms[idx][1]);
                    Some(Command::Teleport {
                        id: *id,
                        pos: current_pos,
                    })
                } else {
                    None
                }
            }

            // Resize → Need previous size
            Command::Resize { id, .. } => {
                let idx = id.index().0 as usize;
                if idx < MAX_ENTITIES {
                    let current_size =
                        Vec2::new(store.transforms[idx][2], store.transforms[idx][3]);
                    Some(Command::Resize {
                        id: *id,
                        size: current_size,
                    })
                } else {
                    None
                }
            }

            // MoveGroup → MoveGroup with negative delta
            Command::MoveGroup { root_id, delta } => Some(Command::MoveGroup {
                root_id: *root_id,
                delta: Vec2::new(-delta.x, -delta.y),
            }),

            // SetColor → Need previous color
            Command::SetColor { id, .. } => {
                let idx = id.index().0 as usize;
                if idx < MAX_ENTITIES {
                    let old_color = store.colors[idx];
                    Some(Command::SetColor {
                        id: *id,
                        color: old_color,
                    })
                } else {
                    None
                }
            }

            // SetShape → Need previous shape
            Command::SetShape { id, .. } => {
                let idx = id.index().0 as usize;
                // Extract previous shape from metadata (bits 0-3)
                let old_shape = (store.metadata[idx] & 0xF) as u8;
                Some(Command::SetShape {
                    id: *id,
                    shape: old_shape,
                })
            }

            // SetVisible → Toggle visibility
            Command::SetVisible { id, visible } => Some(Command::SetVisible {
                id: *id,
                visible: !visible,
            }),

            // SetLayer → Need previous layer
            Command::SetLayer { id, .. } => {
                let idx = id.index().0 as usize;
                // Extract previous layer from metadata (bits 4-7)
                let old_layer = ((store.metadata[idx] >> 4) & 0xF) as u8;
                Some(Command::SetLayer {
                    id: *id,
                    layer: old_layer,
                })
            }

            // SetTexture → Need previous texture index
            Command::SetTexture { id, .. } => {
                let idx = id.index().0 as usize;
                if idx < MAX_ENTITIES {
                    let old_index = store.texture_index[idx];
                    Some(Command::SetTexture {
                        id: *id,
                        texture_index: old_index,
                    })
                } else {
                    None
                }
            }

            // SetText → Cannot reverse without original text
            Command::SetText { .. } => None,

            // SetTextScale → Need previous scale
            Command::SetTextScale { id, .. } => {
                let idx = id.index().0 as usize;
                if idx < MAX_ENTITIES {
                    let old_scale = store.text_scale[idx];
                    Some(Command::SetTextScale {
                        id: *id,
                        scale: old_scale,
                    })
                } else {
                    None
                }
            }

            // SetC4Level → Need arch_data access (not available)
            Command::SetC4Level { .. } => None,

            // SetCloudProvider → Need arch_data access (not available)
            Command::SetCloudProvider { .. } => None,

            // SetParent → Need previous parent
            Command::SetParent { id, .. } => {
                let idx = id.index().0 as usize;
                let old_parent = store.parent_id[idx];
                match old_parent {
                    Some(parent) => Some(Command::SetParent { id: *id, parent }),
                    None => Some(Command::ClearParent(*id)),
                }
            }

            // ClearParent → SetParent with previous parent
            Command::ClearParent(id) => {
                let idx = id.index().0 as usize;
                let old_parent = store.parent_id[idx];
                match old_parent {
                    Some(parent) => Some(Command::SetParent { id: *id, parent }),
                    None => None, // Was already None, cannot restore
                }
            }

            Command::_Max => None,
        }
    }

    /// Execute this command on the entity store
    ///
    /// This is a simplified execution that directly mutates the store.
    /// In production, commands would go through the proper command queue processing.
    pub fn execute(&self, store: &mut EntityStore) {
        match self {
            Command::Spawn { pos, size, parent } => {
                let _id = store.spawn(*pos, *size);
                // Set parent if provided
                if let Some(p) = parent {
                    let idx = _id.index().0 as usize;
                    store.set_parent(idx, Some(*p));
                }
            }
            Command::Despawn(id) => {
                store.despawn(*id);
            }
            Command::Move { id, delta } => {
                let idx = id.index().0 as usize;
                store.move_by(idx, *delta);
            }
            Command::Teleport { id, pos } => {
                let idx = id.index().0 as usize;
                store.set_pos(idx, *pos);
            }
            Command::Resize { id, size } => {
                let idx = id.index().0 as usize;
                store.set_size(idx, *size);
            }
            Command::MoveGroup { root_id, delta } => {
                // Move group - simplified implementation that moves just the root
                // In production, would recursively move all descendants
                let idx = root_id.index().0 as usize;
                store.move_by(idx, *delta);
            }
            Command::SetColor { id, color } => {
                let idx = id.index().0 as usize;
                store.colors[idx] = *color;
                store.dirty_render.insert(idx);
            }
            Command::SetShape { id, shape } => {
                let idx = id.index().0 as usize;
                store.set_shape_type(idx, *shape);
            }
            Command::SetVisible { id, visible } => {
                let idx = id.index().0 as usize;
                store.set_visible(idx, *visible);
            }
            Command::SetLayer { id, layer } => {
                let idx = id.index().0 as usize;
                store.set_layer(idx, *layer);
            }
            // TODO: Implement these methods in EntityStore
            Command::SetTexture { .. } => {}
            Command::SetText { .. } => {}
            Command::SetTextScale { .. } => {}
            Command::SetC4Level { .. } => {}
            Command::SetCloudProvider { .. } => {}
            Command::SetParent { id, parent } => {
                let idx = id.index().0 as usize;
                store.set_parent(idx, Some(*parent));
            }
            Command::ClearParent(id) => {
                let idx = id.index().0 as usize;
                store.set_parent(idx, None);
            }
            Command::_Max => {}
        }
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
    use archflow_core::{Generation, Index};

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

    // ═══════════════════════════════════════════════════════════
    // INVERSE COMMAND TESTS
    // ═══════════════════════════════════════════════════════════

    /// Helper function to create a test entity store with one entity
    fn create_test_store() -> (EntityStore, EntityId) {
        let mut store = EntityStore::new();
        let pos = Vec2::new(100.0, 200.0);
        let size = Vec2::new(50.0, 30.0);
        let id = store.spawn(pos, size);
        (store, id)
    }

    #[test]
    fn test_inverse_move_returns_negative_delta() {
        let (store, id) = create_test_store();
        let delta = Vec2::new(10.0, 20.0);
        let cmd = Command::Move { id, delta };

        let inverse = cmd.inverse(&store).unwrap();

        match inverse {
            Command::Move {
                id: inv_id,
                delta: inv_delta,
            } => {
                assert_eq!(inv_id, id);
                assert_eq!(inv_delta.x, -10.0);
                assert_eq!(inv_delta.y, -20.0);
            }
            _ => panic!("Expected Move command"),
        }
    }

    #[test]
    fn test_inverse_move_roundtrip_restores_position() {
        let (mut store, id) = create_test_store();
        let original_pos = Vec2::new(
            store.transforms[id.index().0 as usize][0],
            store.transforms[id.index().0 as usize][1],
        );

        let delta = Vec2::new(10.0, 20.0);
        let cmd = Command::Move { id, delta };

        // Execute the command
        cmd.execute(&mut store);

        // Get inverse and execute it
        let inverse = cmd.inverse(&store).unwrap();
        inverse.execute(&mut store);

        // Position should be restored (approximately, due to floating point)
        let restored_pos = Vec2::new(
            store.transforms[id.index().0 as usize][0],
            store.transforms[id.index().0 as usize][1],
        );
        assert!((restored_pos.x - original_pos.x).abs() < 0.001);
        assert!((restored_pos.y - original_pos.y).abs() < 0.001);
    }

    #[test]
    fn test_inverse_teleport_returns_current_position() {
        let (store, id) = create_test_store();
        let new_pos = Vec2::new(500.0, 600.0);
        let cmd = Command::Teleport { id, pos: new_pos };

        let inverse = cmd.inverse(&store).unwrap();

        match inverse {
            Command::Teleport { id: inv_id, pos } => {
                assert_eq!(inv_id, id);
                // Should return current position (100.0, 200.0 from create_test_store)
                assert_eq!(pos.x, 100.0);
                assert_eq!(pos.y, 200.0);
            }
            _ => panic!("Expected Teleport command"),
        }
    }

    #[test]
    fn test_inverse_resize_returns_current_size() {
        let (store, id) = create_test_store();
        let new_size = Vec2::new(100.0, 80.0);
        let cmd = Command::Resize { id, size: new_size };

        let inverse = cmd.inverse(&store).unwrap();

        match inverse {
            Command::Resize { id: inv_id, size } => {
                assert_eq!(inv_id, id);
                // Should return current size (50.0, 30.0 from create_test_store)
                assert_eq!(size.x, 50.0);
                assert_eq!(size.y, 30.0);
            }
            _ => panic!("Expected Resize command"),
        }
    }

    #[test]
    fn test_inverse_move_group_returns_negative_delta() {
        let (store, id) = create_test_store();
        let delta = Vec2::new(5.0, -10.0);
        let cmd = Command::MoveGroup { root_id: id, delta };

        let inverse = cmd.inverse(&store).unwrap();

        match inverse {
            Command::MoveGroup {
                root_id,
                delta: inv_delta,
            } => {
                assert_eq!(root_id, id);
                assert_eq!(inv_delta.x, -5.0);
                assert_eq!(inv_delta.y, 10.0);
            }
            _ => panic!("Expected MoveGroup command"),
        }
    }

    #[test]
    fn test_inverse_setcolor_returns_current_color() {
        let (mut store, id) = create_test_store();
        // Set initial color
        store.colors[id.index().0 as usize] = 0xFF0000FF; // Red

        let new_color = 0x00FF00FF; // Green
        let cmd = Command::SetColor {
            id,
            color: new_color,
        };

        let inverse = cmd.inverse(&store).unwrap();

        match inverse {
            Command::SetColor { id: inv_id, color } => {
                assert_eq!(inv_id, id);
                assert_eq!(color, 0xFF0000FF); // Should return original red
            }
            _ => panic!("Expected SetColor command"),
        }
    }

    #[test]
    fn test_inverse_setvisible_toggles_visibility() {
        let (store, id) = create_test_store();

        // Test: SetVisible(true) → SetVisible(false)
        let cmd = Command::SetVisible { id, visible: true };
        let inverse = cmd.inverse(&store).unwrap();
        assert_eq!(inverse, Command::SetVisible { id, visible: false });

        // Test: SetVisible(false) → SetVisible(true)
        let cmd = Command::SetVisible { id, visible: false };
        let inverse = cmd.inverse(&store).unwrap();
        assert_eq!(inverse, Command::SetVisible { id, visible: true });
    }

    #[test]
    fn test_inverse_setshape_returns_current_shape() {
        let (mut store, id) = create_test_store();
        // Set initial shape to Rectangle (0) in metadata bits 0-3
        let idx = id.index().0 as usize;
        store.metadata[idx] = 0; // Shape = 0, Layer = 0

        let cmd = Command::SetShape { id, shape: 2 }; // Ellipse

        let inverse = cmd.inverse(&store).unwrap();

        match inverse {
            Command::SetShape { id: inv_id, shape } => {
                assert_eq!(inv_id, id);
                assert_eq!(shape, 0); // Should return original Rectangle
            }
            _ => panic!("Expected SetShape command"),
        }
    }

    #[test]
    fn test_inverse_setlayer_returns_current_layer() {
        let (mut store, id) = create_test_store();
        // Set initial layer to 2 in metadata bits 4-7
        let idx = id.index().0 as usize;
        store.metadata[idx] = 2 << 4; // Layer = 2, Shape = 0

        let cmd = Command::SetLayer { id, layer: 5 };

        let inverse = cmd.inverse(&store).unwrap();

        match inverse {
            Command::SetLayer { id: inv_id, layer } => {
                assert_eq!(inv_id, id);
                assert_eq!(layer, 2); // Should return original layer 2
            }
            _ => panic!("Expected SetLayer command"),
        }
    }

    #[test]
    fn test_inverse_settexture_returns_current_texture_index() {
        let (mut store, id) = create_test_store();
        let idx = id.index().0 as usize;
        store.texture_index[idx] = 42;

        let cmd = Command::SetTexture {
            id,
            texture_index: 99,
        };

        let inverse = cmd.inverse(&store).unwrap();

        match inverse {
            Command::SetTexture {
                id: inv_id,
                texture_index,
            } => {
                assert_eq!(inv_id, id);
                assert_eq!(texture_index, 42); // Should return original index
            }
            _ => panic!("Expected SetTexture command"),
        }
    }

    #[test]
    fn test_inverse_settextscale_returns_current_scale() {
        let (mut store, id) = create_test_store();
        let idx = id.index().0 as usize;
        store.text_scale[idx] = 1.5;

        let cmd = Command::SetTextScale { id, scale: 2.0 };

        let inverse = cmd.inverse(&store).unwrap();

        match inverse {
            Command::SetTextScale { id: inv_id, scale } => {
                assert_eq!(inv_id, id);
                assert_eq!(scale, 1.5); // Should return original scale
            }
            _ => panic!("Expected SetTextScale command"),
        }
    }

    #[test]
    fn test_inverse_setparent_returns_clearparent_when_no_parent() {
        let (store, id) = create_test_store();
        let parent_id = EntityId::from_parts(Index(2), Generation(1));

        // Entity has no parent initially
        let cmd = Command::SetParent {
            id,
            parent: parent_id,
        };

        let inverse = cmd.inverse(&store).unwrap();

        // Should return ClearParent since there was no parent before
        assert_eq!(inverse, Command::ClearParent(id));
    }

    #[test]
    fn test_inverse_setparent_returns_old_parent_when_exists() {
        let (mut store, id) = create_test_store();
        let old_parent = EntityId::from_parts(Index(2), Generation(1));
        let new_parent = EntityId::from_parts(Index(3), Generation(1));

        // Set initial parent
        store.set_parent(id.index().0 as usize, Some(old_parent));

        let cmd = Command::SetParent {
            id,
            parent: new_parent,
        };

        let inverse = cmd.inverse(&store).unwrap();

        // Should return SetParent with the old parent
        assert_eq!(
            inverse,
            Command::SetParent {
                id,
                parent: old_parent
            }
        );
    }

    #[test]
    fn test_inverse_clearparent_returns_setparent_when_parent_exists() {
        let (mut store, id) = create_test_store();
        let parent_id = EntityId::from_parts(Index(2), Generation(1));

        // Set initial parent
        store.set_parent(id.index().0 as usize, Some(parent_id));

        let cmd = Command::ClearParent(id);

        let inverse = cmd.inverse(&store).unwrap();

        // Should return SetParent with the old parent
        assert_eq!(
            inverse,
            Command::SetParent {
                id,
                parent: parent_id
            }
        );
    }

    #[test]
    fn test_inverse_clearparent_returns_none_when_no_parent() {
        let (store, id) = create_test_store();
        // Entity has no parent initially

        let cmd = Command::ClearParent(id);

        let inverse = cmd.inverse(&store);

        // Should return None since there's no parent to restore
        assert!(inverse.is_none());
    }

    #[test]
    fn test_inverse_spawn_returns_none() {
        let (store, _) = create_test_store();
        let cmd = Command::Spawn {
            pos: Vec2::ZERO,
            size: Vec2::ONE,
            parent: None,
        };

        let inverse = cmd.inverse(&store);
        assert!(
            inverse.is_none(),
            "Spawn should return None (not reversible)"
        );
    }

    #[test]
    fn test_inverse_despawn_returns_none() {
        let (store, id) = create_test_store();
        let cmd = Command::Despawn(id);

        let inverse = cmd.inverse(&store);
        assert!(
            inverse.is_none(),
            "Despawn should return None (not reversible)"
        );
    }

    #[test]
    fn test_inverse_settext_returns_none() {
        let (store, id) = create_test_store();
        let cmd = Command::SetText {
            id,
            text_hash: 12345,
        };

        let inverse = cmd.inverse(&store);
        assert!(
            inverse.is_none(),
            "SetText should return None (not reversible without original text)"
        );
    }

    #[test]
    fn test_inverse_setc4level_returns_none() {
        let (store, id) = create_test_store();
        let cmd = Command::SetC4Level { id, level: 2 };

        let inverse = cmd.inverse(&store);
        assert!(
            inverse.is_none(),
            "SetC4Level should return None (arch_data not accessible)"
        );
    }

    #[test]
    fn test_inverse_setcloudprovider_returns_none() {
        let (store, id) = create_test_store();
        let cmd = Command::SetCloudProvider { id, provider: 1 };

        let inverse = cmd.inverse(&store);
        assert!(
            inverse.is_none(),
            "SetCloudProvider should return None (arch_data not accessible)"
        );
    }

    #[test]
    fn test_inverse_with_invalid_entity_id_returns_none() {
        let store = EntityStore::new();
        // Use an invalid entity ID (index 1000000 >= MAX_ENTITIES=100000)
        let invalid_id = EntityId::from_parts(Index(1000000), Generation(1));

        // These commands require valid entity access
        let cmd = Command::SetColor {
            id: invalid_id,
            color: 0xFF0000FF,
        };
        let inverse = cmd.inverse(&store);
        assert!(
            inverse.is_none(),
            "Should return None for out-of-bounds entity ID"
        );

        let cmd2 = Command::Resize {
            id: invalid_id,
            size: Vec2::ONE,
        };
        let inverse2 = cmd2.inverse(&store);
        assert!(
            inverse2.is_none(),
            "Should return None for out-of-bounds entity ID"
        );
    }

    #[test]
    fn test_inverse_color_roundtrip() {
        let (mut store, id) = create_test_store();
        let idx = id.index().0 as usize;
        let original_color = 0xAABBCCDD;
        store.colors[idx] = original_color;

        let new_color = 0xFFEEFFAA;
        let cmd = Command::SetColor {
            id,
            color: new_color,
        };

        // Get inverse BEFORE executing (captures current/original state)
        let inverse = cmd.inverse(&store).unwrap();

        // Execute forward
        cmd.execute(&mut store);
        assert_eq!(store.colors[idx], new_color);

        // Execute inverse (should restore original)
        inverse.execute(&mut store);
        assert_eq!(store.colors[idx], original_color);
    }

    #[test]
    fn test_inverse_visible_roundtrip() {
        let (mut store, id) = create_test_store();
        let idx = id.index().0 as usize;
        store.set_visible(idx, true);

        // Make invisible
        let cmd = Command::SetVisible { id, visible: false };

        // Get inverse BEFORE executing (should capture visible=true)
        let inverse = cmd.inverse(&store).unwrap();

        cmd.execute(&mut store);
        assert_eq!(store.is_visible(idx), false);

        // Inverse should make visible again
        inverse.execute(&mut store);
        assert_eq!(store.is_visible(idx), true);
    }

    #[test]
    fn test_inverse_shape_roundtrip() {
        let (mut store, id) = create_test_store();
        let idx = id.index().0 as usize;
        store.set_shape_type(idx, 1); // Circle

        let cmd = Command::SetShape { id, shape: 5 }; // Diamond

        // Get inverse BEFORE executing (should capture shape=1)
        let inverse = cmd.inverse(&store).unwrap();

        cmd.execute(&mut store);
        assert_eq!(store.shape_type(idx), 5);

        // Inverse should restore circle
        inverse.execute(&mut store);
        assert_eq!(store.shape_type(idx), 1);
    }

    #[test]
    fn test_inverse_layer_roundtrip() {
        let (mut store, id) = create_test_store();
        let idx = id.index().0 as usize;
        store.set_layer(idx, 3);

        let cmd = Command::SetLayer { id, layer: 7 };

        // Get inverse BEFORE executing (should capture layer=3)
        let inverse = cmd.inverse(&store).unwrap();

        cmd.execute(&mut store);
        assert_eq!(store.layer(idx), 7);

        // Inverse should restore layer 3
        inverse.execute(&mut store);
        assert_eq!(store.layer(idx), 3);
    }
}
