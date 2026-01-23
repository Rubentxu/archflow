//! ArchFlow Core - Meta-crate con tipos base del dominio
//!
//! Este crate contiene los tipos fundamentales usados por todo el engine:
//! - Tipos geométricos (Vec2, Mat3, Rect, Transform)
//! - Identificadores (EntityId)
//! - Tipos de error y resultado
//! - Event Sourcing (Domain Events, Event Store, Journal)

// Módulos
mod color;
mod entity_id;
mod error;
mod event_sourcing;
mod rect;
mod transform;
mod types;

// Re-exports
pub use color::{Color, Hsla, Rgba};
pub use entity_id::{EntityId, EntityIdGenerator};
pub use error::{CoreError, CoreResult};
pub use rect::{Rect, Rect2D};
pub use transform::{Transform, Transform2D};
pub use types::{Mat3, Vec2};

// Event Sourcing exports
pub use event_sourcing::{
    Command, CommandResult, DocumentAggregate, DomainEvent, EventJournal, EventMetadata,
    EventStore, EventStoreError, JournalEntry, JournalError, Snapshot, SnapshotError,
    SnapshotManager, StoredEvent, UndoRedoStack,
};

// Re-export de uuid
pub use uuid::Uuid;
