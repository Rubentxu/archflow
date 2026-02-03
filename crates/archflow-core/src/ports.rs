// ═══════════════════════════════════════════════════════════════════════════════
// Ports - Hexagonal Architecture Trait Definitions
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 2.2
//
// In Hexagonal Architecture (Ports & Adapters), ports define the interface
// between the domain and the outside world.
//
// Primary Ports (Input): Interfaces that external code uses to talk to the domain
// Secondary Ports (Output): Interfaces that the domain uses to talk to external services
//
// This file defines the core ports that all adapters must implement.
// ═══════════════════════════════════════════════════════════════════════════════

use crate::id::EntityId;

// Import types from std when available (for trait definitions)
#[cfg(feature = "std")]
use std::{boxed::Box, string::String, vec::Vec};

// ═══════════════════════════════════════════════════════════════════════════════
// PRIMARY PORTS (Input) - External code calls these
// ═══════════════════════════════════════════════════════════════════════════════

/// Entity Store Port - Primary interface for entity operations
///
/// This is the main port that the application layer uses to interact
/// with entities. Implementations must be thread-safe if used concurrently.
pub trait EntityStorePort {
    /// Error type for store operations
    type Error;

    /// Spawn a new entity at the given position
    fn spawn(
        &mut self,
        pos: crate::math::Vec2,
        size: crate::math::Vec2,
    ) -> Result<EntityId, Self::Error>;

    /// Despawn an entity (mark as dead, reusable slot)
    fn despawn(&mut self, id: EntityId) -> Result<(), Self::Error>;

    /// Check if an entity is alive
    fn is_alive(&self, id: EntityId) -> bool;

    /// Get the number of alive entities
    fn alive_count(&self) -> usize;

    /// Set the position of an entity
    fn set_position(&mut self, id: EntityId, pos: crate::math::Vec2) -> Result<(), Self::Error>;

    /// Get the position of an entity
    fn get_position(&self, id: EntityId) -> Option<crate::math::Vec2>;

    /// Set the size of an entity
    fn set_size(&mut self, id: EntityId, size: crate::math::Vec2) -> Result<(), Self::Error>;

    /// Get the size of an entity
    fn get_size(&self, id: EntityId) -> Option<crate::math::Vec2>;

    /// Set the color of an entity
    fn set_color(&mut self, id: EntityId, color: crate::math::Color) -> Result<(), Self::Error>;

    /// Get the color of an entity
    fn get_color(&self, id: EntityId) -> Option<crate::math::Color>;

    /// Set the visibility of an entity
    fn set_visible(&mut self, id: EntityId, visible: bool) -> Result<(), Self::Error>;

    /// Check if an entity is visible
    fn is_visible(&self, id: EntityId) -> bool;

    /// Set the selection state of an entity
    fn set_selected(&mut self, id: EntityId, selected: bool) -> Result<(), Self::Error>;

    /// Check if an entity is selected
    fn is_selected(&self, id: EntityId) -> bool;

    /// Set the parent of an entity (for hierarchy)
    fn set_parent(&mut self, id: EntityId, parent: Option<EntityId>) -> Result<(), Self::Error>;

    /// Get the parent of an entity
    fn get_parent(&self, id: EntityId) -> Option<EntityId>;

    /// Get children of an entity
    fn get_children(&self, id: EntityId) -> Vec<EntityId>;

    /// Set text content for an entity
    fn set_text(&mut self, id: EntityId, text: &str) -> Result<(), Self::Error>;

    /// Get text content of an entity
    fn get_text(&self, id: EntityId) -> Option<&str>;
}

/// Canvas Port - Primary interface for rendering operations
///
/// This port defines how the domain can interact with the rendering system.
/// Implementations can use WebGPU, Canvas2D, or other rendering backends.
pub trait CanvasPort {
    /// Error type for canvas operations
    type Error;

    /// Clear the canvas with the given color
    fn clear(&mut self, color: crate::math::Color) -> Result<(), Self::Error>;

    /// Begin a new frame
    fn begin_frame(&mut self) -> Result<(), Self::Error>;

    /// End the current frame and present
    fn end_frame(&mut self) -> Result<(), Self::Error>;

    /// Set the view-projection matrix
    fn set_view_projection(&mut self, matrix: crate::math::Mat4) -> Result<(), Self::Error>;

    /// Submit a batch of entities for rendering
    fn render_entities(&mut self, entities: &[EntityId]) -> Result<(), Self::Error>;

    /// Render a selection box
    fn render_selection(&mut self, bounds: crate::math::Rect) -> Result<(), Self::Error>;

    /// Render a connection line
    fn render_connection(
        &mut self,
        start: crate::math::Vec2,
        end: crate::math::Vec2,
    ) -> Result<(), Self::Error>;
}

/// Event Publisher Port - Primary interface for domain events
///
/// This port allows the domain to publish events to the outside world.
/// Implementations can use in-memory queues, message brokers, etc.
pub trait EventPublisher {
    /// Error type for event publishing
    type Error;

    /// Publish a domain event
    fn publish(&mut self, event: DomainEvent) -> Result<(), Self::Error>;

    /// Subscribe to events (returns a subscription ID)
    fn subscribe(&mut self, handler: Box<dyn EventHandler>) -> usize;

    /// Unsubscribe from events
    fn unsubscribe(&mut self, subscription_id: usize) -> Result<(), Self::Error>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECONDARY PORTS (Output) - Domain calls these
// ═══════════════════════════════════════════════════════════════════════════════

/// Command Executor Port - Secondary port for executing commands
///
/// The domain uses this port to execute commands (undoable actions).
pub trait CommandExecutor {
    /// Error type for command execution
    type Error;

    /// Execute a command
    fn execute(&mut self, command: Command) -> Result<CommandResult, Self::Error>;

    /// Undo the last command
    fn undo(&mut self) -> Result<(), Self::Error>;

    /// Redo the last undone command
    fn redo(&mut self) -> Result<(), Self::Error>;

    /// Check if undo is available
    fn can_undo(&self) -> bool;

    /// Check if redo is available
    fn can_redo(&self) -> bool;
}

/// Asset Loader Port - Secondary port for loading assets
///
/// The domain uses this port to load icons, images, fonts, etc.
pub trait AssetLoader {
    /// Error type for asset loading
    type Error;

    /// Load an icon from bytes (SVG, PNG, etc.)
    fn load_icon(&mut self, id: &str, data: &[u8]) -> Result<usize, Self::Error>;

    /// Load a font from bytes
    fn load_font(&mut self, id: &str, data: &[u8]) -> Result<usize, Self::Error>;

    /// Get an icon by ID
    fn get_icon(&self, id: &str) -> Option<usize>;

    /// Get a font by ID
    fn get_font(&self, id: &str) -> Option<usize>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// DOMAIN EVENTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Domain events represent facts that happened in the domain
///
/// Events are immutable and should contain all relevant information
/// about what happened.
#[derive(Clone, Debug, PartialEq)]
pub enum DomainEvent {
    /// An entity was spawned
    EntitySpawned {
        /// The unique identifier of the spawned entity
        id: EntityId,
    },

    /// An entity was despawned
    EntityDespawned {
        /// The unique identifier of the despawned entity
        id: EntityId,
    },

    /// An entity was moved
    EntityMoved {
        /// The unique identifier of the moved entity
        id: EntityId,
        /// The previous position before the move
        old_position: crate::math::Vec2,
        /// The new position after the move
        new_position: crate::math::Vec2,
    },

    /// An entity was resized
    EntityResized {
        /// The unique identifier of the resized entity
        id: EntityId,
        /// The previous size before the resize
        old_size: crate::math::Vec2,
        /// The new size after the resize
        new_size: crate::math::Vec2,
    },

    /// An entity's color was changed
    EntityColorChanged {
        /// The unique identifier of the entity whose color changed
        id: EntityId,
        /// The previous color before the change
        old_color: crate::math::Color,
        /// The new color after the change
        new_color: crate::math::Color,
    },

    /// An entity was selected
    EntitySelected {
        /// The unique identifier of the selected entity
        id: EntityId,
    },

    /// An entity was deselected
    EntityDeselected {
        /// The unique identifier of the deselected entity
        id: EntityId,
    },

    /// A connection was created
    ConnectionCreated {
        /// The unique identifier of the connection entity
        id: EntityId,
        /// The source entity of the connection
        source: EntityId,
        /// The target entity of the connection
        target: EntityId,
    },

    /// A connection was deleted
    ConnectionDeleted {
        /// The unique identifier of the deleted connection
        id: EntityId,
    },

    /// Text was changed
    TextChanged {
        /// The unique identifier of the entity whose text changed
        id: EntityId,
        /// The previous text content before the change
        old_text: String,
        /// The new text content after the change
        new_text: String,
    },
}

// ═══════════════════════════════════════════════════════════════════════════════
// COMMANDS
// ═══════════════════════════════════════════════════════════════════════════════

/// Commands represent intentions to change the domain
///
/// Unlike events (which are facts), commands represent desired changes.
/// Commands can be executed, undone, and redone.
///
/// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 5
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command {
    /// Spawn a new entity
    Spawn {
        /// The position at which to spawn the entity
        pos: crate::math::Vec2,
        /// The size of the entity to spawn
        size: crate::math::Vec2,
        /// Optional parent entity for hierarchical organization
        parent: Option<EntityId>,
    },

    /// Despawn an entity
    Despawn(
        /// The unique identifier of the entity to despawn
        EntityId,
    ),

    /// Move an entity by a delta
    Move {
        /// The unique identifier of the entity to move
        id: EntityId,
        /// The delta vector by which to move the entity
        delta: crate::math::Vec2,
    },

    /// Teleport an entity to a position
    Teleport {
        /// The unique identifier of the entity to teleport
        id: EntityId,
        /// The absolute position to teleport the entity to
        pos: crate::math::Vec2,
    },

    /// Resize an entity
    Resize {
        /// The unique identifier of the entity to resize
        id: EntityId,
        /// The new size for the entity
        size: crate::math::Vec2,
    },

    /// Set the color of an entity
    SetColor {
        /// The unique identifier of the entity whose color to set
        id: EntityId,
        /// The new color value (ARGB format)
        color: u32,
    },

    /// Set the shape type of an entity
    SetShape {
        /// The unique identifier of the entity whose shape to set
        id: EntityId,
        /// The shape type identifier
        shape: u8,
    },
}

/// Result of executing a command
#[derive(Clone, Debug, PartialEq)]
pub enum CommandResult {
    /// Command executed successfully
    Success,

    /// Command execution failed
    Failed(String),

    /// Command executed with events to publish
    SuccessWithEvents(Vec<DomainEvent>),
}

// ═══════════════════════════════════════════════════════════════════════════════
// EVENT HANDLER
// ═══════════════════════════════════════════════════════════════════════════════

/// Handler for domain events
pub trait EventHandler: Send + Sync {
    /// Handle a domain event
    fn handle(&mut self, event: &DomainEvent);
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_event_clone() {
        let event = DomainEvent::EntitySpawned {
            id: EntityId::new(123),
        };
        let cloned = event.clone();
        assert_eq!(event, cloned);
    }

    #[test]
    fn test_command_copy() {
        let cmd = Command::SetColor {
            id: EntityId::new(456),
            color: 0xFF0000FF,
        };
        let copied = cmd;
        assert_eq!(cmd, copied);
    }
}
