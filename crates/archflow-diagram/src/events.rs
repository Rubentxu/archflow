// ═══════════════════════════════════════════════════════════════════════════════
// Diagram Events - Domain Events for C4 Diagrams
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 17
//
// Domain events represent facts that happened in the diagram domain.
// Unlike commands (intentions), events are immutable facts that can be
// stored, published, and used for synchronization (CRDT).
// ═══════════════════════════════════════════════════════════════════════════════

use crate::c4::{C4EntityType, C4Level, CloudProvider};
use archflow_core::{Color, EntityId, Vec2};

// ═══════════════════════════════════════════════════════════════════════════════
// DIAGRAM EVENTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Domain events for C4 diagram changes
///
/// Events are facts that happened,不可变 and should contain all relevant
/// information about what happened.
///
/// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 17
#[derive(Clone, Debug, PartialEq)]
pub enum DiagramEvent {
    // ═══════════════════════════════════════════════════════════════════════════════
    // ENTITY LIFECYCLE
    // ═══════════════════════════════════════════════════════════════════════════════
    /// An entity was spawned
    EntitySpawned {
        /// ID of the new entity
        id: EntityId,
        /// Position where it was spawned
        position: Vec2,
        /// Initial size
        size: Vec2,
        /// C4 level
        c4_level: C4Level,
        /// Entity type
        entity_type: C4EntityType,
    },

    /// An entity was despawned
    EntityDespawned {
        /// ID of the despawned entity
        id: EntityId,
    },

    // ═══════════════════════════════════════════════════════════════════════════════
    // TRANSFORMATION EVENTS
    // ═══════════════════════════════════════════════════════════════════════════════
    /// An entity was moved
    EntityMoved {
        /// ID of the moved entity
        id: EntityId,
        /// Old position
        old_position: Vec2,
        /// New position
        new_position: Vec2,
    },

    /// An entity was resized
    EntityResized {
        /// ID of the resized entity
        id: EntityId,
        /// Old size
        old_size: Vec2,
        /// New size
        new_size: Vec2,
    },

    /// An entity was teleported (absolute position change)
    EntityTeleported {
        /// ID of the teleported entity
        id: EntityId,
        /// Old position
        old_position: Vec2,
        /// New position
        new_position: Vec2,
    },

    // ═══════════════════════════════════════════════════════════════════════════════
    // STYLE EVENTS
    // ═══════════════════════════════════════════════════════════════════════════════
    /// An entity's color was changed
    EntityColorChanged {
        /// ID of the entity
        id: EntityId,
        /// Old color
        old_color: Color,
        /// New color
        new_color: Color,
    },

    /// An entity's shape was changed
    EntityShapeChanged {
        /// ID of the entity
        id: EntityId,
        /// Old shape type
        old_shape: u8,
        /// New shape type
        new_shape: u8,
    },

    /// An entity's texture was changed
    EntityTextureChanged {
        /// ID of the entity
        id: EntityId,
        /// Old texture index
        old_texture_index: Option<u16>,
        /// New texture index
        new_texture_index: Option<u16>,
    },

    // ═══════════════════════════════════════════════════════════════════════════════
    // SELECTION EVENTS
    // ═══════════════════════════════════════════════════════════════════════════════
    /// An entity was selected
    EntitySelected {
        /// ID of the selected entity
        id: EntityId,
    },

    /// An entity was deselected
    EntityDeselected {
        /// ID of the deselected entity
        id: EntityId,
    },

    /// Multiple entities were selected (marquee selection)
    EntitiesSelected {
        /// IDs of the selected entities
        ids: alloc::vec::Vec<EntityId>,
    },

    // ═══════════════════════════════════════════════════════════════════════════════
    // TEXT EVENTS
    // ═══════════════════════════════════════════════════════════════════════════════
    /// An entity's text content was changed
    TextChanged {
        /// ID of the entity
        id: EntityId,
        /// Old text hash
        old_text_hash: u64,
        /// New text hash
        new_text_hash: u64,
    },

    /// An entity's text scale was changed
    TextScaleChanged {
        /// ID of the entity
        id: EntityId,
        /// Old scale
        old_scale: f32,
        /// New scale
        new_scale: f32,
    },

    // ═══════════════════════════════════════════════════════════════════════════════
    // C4-SPECIFIC EVENTS
    // ═══════════════════════════════════════════════════════════════════════════════
    /// An entity's C4 level was changed
    C4LevelChanged {
        /// ID of the entity
        id: EntityId,
        /// Old level
        old_level: C4Level,
        /// New level
        new_level: C4Level,
    },

    /// An entity's cloud provider was changed
    CloudProviderChanged {
        /// ID of the entity
        id: EntityId,
        /// Old provider
        old_provider: CloudProvider,
        /// New provider
        new_provider: CloudProvider,
    },

    // ═══════════════════════════════════════════════════════════════════════════════
    // CONNECTION EVENTS
    // ═══════════════════════════════════════════════════════════════════════════════
    /// A connection was created between two entities
    ConnectionCreated {
        /// ID of the connection
        id: EntityId,
        /// Source entity
        source: EntityId,
        /// Target entity
        target: EntityId,
    },

    /// A connection was deleted
    ConnectionDeleted {
        /// ID of the deleted connection
        id: EntityId,
    },

    /// A connection was rerouted
    ConnectionRerouted {
        /// ID of the connection
        id: EntityId,
        /// Old source
        old_source: EntityId,
        /// New source
        new_source: EntityId,
        /// Old target
        old_target: EntityId,
        /// New target
        new_target: EntityId,
    },

    // ═══════════════════════════════════════════════════════════════════════════════
    // HIERARCHY EVENTS
    // ═══════════════════════════════════════════════════════════════════════════════
    /// An entity's parent was changed
    ParentChanged {
        /// ID of the entity
        id: EntityId,
        /// Old parent
        old_parent: Option<EntityId>,
        /// New parent
        new_parent: Option<EntityId>,
    },

    /// A group was created
    GroupCreated {
        /// ID of the group entity
        id: EntityId,
        /// Initial children
        children: alloc::vec::Vec<EntityId>,
    },
}

impl DiagramEvent {
    /// Get the entity ID affected by this event, if any
    #[inline(always)]
    pub fn entity_id(&self) -> Option<EntityId> {
        match self {
            DiagramEvent::EntitySpawned { id, .. }
            | DiagramEvent::EntityDespawned { id }
            | DiagramEvent::EntityMoved { id, .. }
            | DiagramEvent::EntityResized { id, .. }
            | DiagramEvent::EntityTeleported { id, .. }
            | DiagramEvent::EntityColorChanged { id, .. }
            | DiagramEvent::EntityShapeChanged { id, .. }
            | DiagramEvent::EntityTextureChanged { id, .. }
            | DiagramEvent::EntitySelected { id }
            | DiagramEvent::EntityDeselected { id }
            | DiagramEvent::TextChanged { id, .. }
            | DiagramEvent::TextScaleChanged { id, .. }
            | DiagramEvent::C4LevelChanged { id, .. }
            | DiagramEvent::CloudProviderChanged { id, .. }
            | DiagramEvent::ConnectionCreated { id, .. }
            | DiagramEvent::ConnectionDeleted { id }
            | DiagramEvent::ConnectionRerouted { id, .. }
            | DiagramEvent::ParentChanged { id, .. }
            | DiagramEvent::GroupCreated { id, .. } => Some(*id),
            DiagramEvent::EntitiesSelected { .. } => None,
        }
    }

    /// Get a human-readable name for this event type
    #[inline(always)]
    pub fn type_name(&self) -> &'static str {
        match self {
            DiagramEvent::EntitySpawned { .. } => "EntitySpawned",
            DiagramEvent::EntityDespawned { .. } => "EntityDespawned",
            DiagramEvent::EntityMoved { .. } => "EntityMoved",
            DiagramEvent::EntityResized { .. } => "EntityResized",
            DiagramEvent::EntityTeleported { .. } => "EntityTeleported",
            DiagramEvent::EntityColorChanged { .. } => "EntityColorChanged",
            DiagramEvent::EntityShapeChanged { .. } => "EntityShapeChanged",
            DiagramEvent::EntityTextureChanged { .. } => "EntityTextureChanged",
            DiagramEvent::EntitySelected { .. } => "EntitySelected",
            DiagramEvent::EntityDeselected { .. } => "EntityDeselected",
            DiagramEvent::EntitiesSelected { .. } => "EntitiesSelected",
            DiagramEvent::TextChanged { .. } => "TextChanged",
            DiagramEvent::TextScaleChanged { .. } => "TextScaleChanged",
            DiagramEvent::C4LevelChanged { .. } => "C4LevelChanged",
            DiagramEvent::CloudProviderChanged { .. } => "CloudProviderChanged",
            DiagramEvent::ConnectionCreated { .. } => "ConnectionCreated",
            DiagramEvent::ConnectionDeleted { .. } => "ConnectionDeleted",
            DiagramEvent::ConnectionRerouted { .. } => "ConnectionRerouted",
            DiagramEvent::ParentChanged { .. } => "ParentChanged",
            DiagramEvent::GroupCreated { .. } => "GroupCreated",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_id_extraction() {
        let id = EntityId::new(123);
        assert_eq!(DiagramEvent::EntitySelected { id }.entity_id(), Some(id));
        assert_eq!(
            DiagramEvent::EntitiesSelected {
                ids: alloc::vec::Vec::new()
            }
            .entity_id(),
            None
        );
    }

    #[test]
    fn test_event_type_name() {
        assert_eq!(
            DiagramEvent::EntitySpawned {
                id: EntityId::new(1),
                position: Vec2::ZERO,
                size: Vec2::new(10.0, 10.0),
                c4_level: C4Level::System,
                entity_type: C4EntityType::SoftwareSystem
            }
            .type_name(),
            "EntitySpawned"
        );
    }

    #[test]
    fn test_event_clone() {
        let event = DiagramEvent::EntitySelected {
            id: EntityId::new(456),
        };
        let cloned = event.clone();
        assert_eq!(event, cloned);
    }
}
