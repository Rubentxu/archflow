//! # Event Sourcing Module
//!
//! Provides event sourcing infrastructure for the demo.

use crate::entity_id::EntityId;
use crate::types::Vec2;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A command that triggers state changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub id: EntityId,
    pub command_type: String,
    pub payload: Vec<u8>,
}

/// Domain event representing something that happened
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    pub event_type: String,
    pub metadata: EventMetadata,
    pub data: Vec<u8>,
}

/// Metadata for domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub entity_id: EntityId,
    pub sequence: u64,
    pub causation_id: EntityId,
    pub timestamp: String,
}

impl EventMetadata {
    pub fn new(
        entity_id: EntityId,
        sequence: u64,
        causation_id: EntityId,
        timestamp: String,
    ) -> Self {
        Self {
            entity_id,
            sequence,
            causation_id,
            timestamp,
        }
    }
}

/// Event store for persisting events
#[derive(Debug, Default)]
pub struct EventStore {
    events: Vec<DomainEvent>,
}

impl EventStore {
    pub fn append(&mut self, event: DomainEvent) {
        self.events.push(event);
    }

    pub fn get_events(&self) -> &[DomainEvent] {
        &self.events
    }
}

/// Undo/Redo stack for command history
#[derive(Debug, Default)]
pub struct UndoRedoStack<T> {
    undo_stack: Vec<T>,
    redo_stack: Vec<T>,
}

impl<T: Clone> UndoRedoStack<T> {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn push(&mut self, item: T) {
        self.undo_stack.push(item);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> Option<T> {
        self.redo_stack.push(self.undo_stack.pop()?);
        self.redo_stack.last().cloned()
    }

    pub fn redo(&mut self) -> Option<T> {
        self.undo_stack.push(self.redo_stack.pop()?);
        self.undo_stack.last().cloned()
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

/// Document aggregate for the demo
#[derive(Debug)]
pub struct DocumentAggregate {
    pub id: EntityId,
    pub name: String,
    events: Vec<DomainEvent>,
}

impl DocumentAggregate {
    pub fn new(id: EntityId, name: String) -> Self {
        Self {
            id,
            name,
            events: Vec::new(),
        }
    }

    pub fn apply(&mut self, event: DomainEvent) {
        self.events.push(event);
    }

    pub fn get_events(&self) -> &[DomainEvent] {
        &self.events
    }
}

/// Shape primitive created event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimitiveCreatedData {
    pub primitive_id: EntityId,
    pub shape_type: String,
    pub position: Vec2,
    pub size: Vec2,
    pub color: String,
}

/// Shape moved event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimitiveMovedData {
    pub primitive_id: EntityId,
    pub from: Vec2,
    pub to: Vec2,
}
