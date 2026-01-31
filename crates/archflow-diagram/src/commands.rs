// ═══════════════════════════════════════════════════════════════════════════════
// Diagram Commands - Domain Commands for C4 Diagrams
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 5
//
// Commands represent intentions to change the diagram domain.
// These are pure types that can be serialized and executed.
// ═══════════════════════════════════════════════════════════════════════════════

use archflow_core::{EntityId, Vec2};

/// Diagram commands - intentions to change a C4 diagram
///
/// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 5.1
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DiagramCommand {
    // ═══════════════════════════════════════════════════════════════════════════════
    // CREATION / DESTRUCTION
    // ═══════════════════════════════════════════════════════════════════════════════
    /// Spawn a new C4 entity
    Spawn {
        /// Position in world coordinates
        pos: Vec2,
        /// Size of the entity
        size: Vec2,
        /// Parent entity (for grouping/frames)
        parent: Option<EntityId>,
        /// C4 level (System, Container, Component, Code)
        c4_level: u8,
        /// Entity type (Person, SoftwareSystem, Container, etc.)
        entity_type: u8,
    },

    /// Despawn an entity (mark as dead)
    Despawn(EntityId),

    // ═══════════════════════════════════════════════════════════════════════════════
    // TRANSFORMATION
    // ═══════════════════════════════════════════════════════════════════════════════
    /// Move an entity by a delta
    Move {
        /// Entity to move
        id: EntityId,
        /// Delta in world coordinates
        delta: Vec2,
    },

    /// Teleport an entity to a specific position
    Teleport {
        /// Entity to teleport
        id: EntityId,
        /// New position
        pos: Vec2,
    },

    /// Resize an entity
    Resize {
        /// Entity to resize
        id: EntityId,
        /// New size
        size: Vec2,
    },

    /// Move a group of entities (hierarchy-aware)
    MoveGroup {
        /// Root entity of the group
        root_id: EntityId,
        /// Delta to apply to all children
        delta: Vec2,
    },

    // ═══════════════════════════════════════════════════════════════════════════════
    // STYLE
    // ═══════════════════════════════════════════════════════════════════════════════
    /// Set the color of an entity
    SetColor {
        /// Entity to recolor
        id: EntityId,
        /// New color (0xRRGGBBAA packed)
        color: u32,
    },

    /// Set the shape type of an entity
    SetShape {
        /// Entity to reshape
        id: EntityId,
        /// Shape type (0=Rect, 1=Circle, 2=Line, etc.)
        shape: u8,
    },

    // ═══════════════════════════════════════════════════════════════════════════════
    // TEXT / LABELS
    // ═══════════════════════════════════════════════════════════════════════════════
    /// Set the text content of an entity
    SetText {
        /// Entity to update
        id: EntityId,
        /// Hash of the text content (points to StringPool)
        text_hash: u64,
    },

    /// Set the text scale (font size)
    SetTextScale {
        /// Entity to update
        id: EntityId,
        /// New scale (1.0 = 12px base)
        scale: f32,
    },

    // ═══════════════════════════════════════════════════════════════════════════════
    // C4-SPECIFIC
    // ═══════════════════════════════════════════════════════════════════════════════
    /// Set the C4 level of an entity
    SetC4Level {
        /// Entity to update
        id: EntityId,
        /// New C4 level (0=System, 1=Container, 2=Component, 3=Code)
        level: u8,
    },

    /// Set the cloud provider for IaC export
    SetCloudProvider {
        /// Entity to update
        id: EntityId,
        /// Cloud provider (0=None, 1=AWS, 2=GCP, 3=Azure)
        provider: u8,
    },

    // ═══════════════════════════════════════════════════════════════════════════════
    // TEXTURE / ICONS
    // ═══════════════════════════════════════════════════════════════════════════════
    /// Set the texture (icon/image) for an entity
    SetTexture {
        /// Entity to update
        id: EntityId,
        /// Index in the texture atlas
        texture_index: u16,
        /// UV rectangle in the atlas [u, v, w, h]
        uv_rect: [f32; 4],
    },
}

impl DiagramCommand {
    /// Get the entity ID affected by this command, if any
    #[inline(always)]
    pub fn entity_id(&self) -> Option<EntityId> {
        match self {
            DiagramCommand::Spawn { .. } | DiagramCommand::MoveGroup { .. } => None,
            DiagramCommand::Despawn(id) => Some(*id),
            DiagramCommand::Move { id, .. }
            | DiagramCommand::Teleport { id, .. }
            | DiagramCommand::Resize { id, .. }
            | DiagramCommand::SetColor { id, .. }
            | DiagramCommand::SetShape { id, .. }
            | DiagramCommand::SetText { id, .. }
            | DiagramCommand::SetTextScale { id, .. }
            | DiagramCommand::SetC4Level { id, .. }
            | DiagramCommand::SetCloudProvider { id, .. }
            | DiagramCommand::SetTexture { id, .. } => Some(*id),
        }
    }

    /// Get the command discriminant (useful for serialization)
    #[inline(always)]
    pub fn discriminant(&self) -> u8 {
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_copy() {
        let cmd = DiagramCommand::SetColor {
            id: EntityId::new(123),
            color: 0xFF0000FF,
        };
        let copied = cmd;
        assert_eq!(cmd, copied);
    }

    #[test]
    fn test_entity_id_extraction() {
        let id = EntityId::new(456);
        assert_eq!(
            DiagramCommand::SetColor {
                id,
                color: 0xFF0000FF
            }
            .entity_id(),
            Some(id)
        );
        assert_eq!(DiagramCommand::MoveGroup {
            root_id: id,
            delta: Vec2::new(1.0, 2.0)
        }
        .entity_id()
        .is_none());
    }
}
