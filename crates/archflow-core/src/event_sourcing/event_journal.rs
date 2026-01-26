//! Event Journal - Undo/Redo functionality for document editing
//!
//! Provides:
//! - JournalEntry for tracking commands and their inverse events
//! - UndoRedoStack for managing undo/redo history
//! - EventJournal for coordinating the event journal

use crate::event_sourcing::event::SerializableTime;
use crate::EntityId;
use std::collections::VecDeque;
use thiserror::Error;

/// Error types for journal operations
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum JournalError {
    /// No events to undo
    #[error("No events to undo")]
    NoEventsToUndo,
    /// No events to redo
    #[error("No events to redo")]
    NoEventsToRedo,
}

/// An entry in the event journal
#[derive(Debug, Clone)]
pub struct JournalEntry {
    /// Unique identifier for this entry
    pub id: EntityId,
    /// The events that were applied
    pub events: Vec<super::event::DomainEvent>,
    /// The inverse events for undo
    pub inverse_events: Vec<super::event::DomainEvent>,
    /// When this entry was created
    pub timestamp: SerializableTime,
    /// Description for UI display
    pub description: String,
    /// Whether this entry is a batch
    pub is_batch: bool,
    /// Batch ID if this is part of a batch
    pub batch_id: Option<EntityId>,
}

impl JournalEntry {
    /// Create a new journal entry from events
    pub fn new(events: Vec<super::event::DomainEvent>, description: String) -> Self {
        let inverse_events: Vec<super::event::DomainEvent> =
            events.iter().map(|e| e.invert()).collect();
        Self {
            id: EntityId::new(),
            events,
            inverse_events,
            timestamp: SerializableTime::default(),
            description,
            is_batch: false,
            batch_id: None,
        }
    }

    /// Get the count of events in this entry
    pub fn len(&self) -> usize {
        self.events.len()
    }
}

/// Stack-based undo/redo history
#[derive(Debug, Clone)]
pub struct UndoRedoStack {
    undo_stack: VecDeque<JournalEntry>,
    redo_stack: VecDeque<JournalEntry>,
    max_history: usize,
}

impl UndoRedoStack {
    pub fn new(max_history: usize) -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            max_history,
        }
    }

    pub fn push(&mut self, entry: JournalEntry) {
        while self.undo_stack.len() >= self.max_history {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(entry);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> Result<JournalEntry, JournalError> {
        self.undo_stack
            .pop_back()
            .map(|entry| {
                self.redo_stack.push_back(entry.clone());
                entry
            })
            .ok_or(JournalError::NoEventsToUndo)
    }

    pub fn redo(&mut self) -> Result<JournalEntry, JournalError> {
        self.redo_stack
            .pop_back()
            .map(|entry| {
                self.undo_stack.push_back(entry.clone());
                entry
            })
            .ok_or(JournalError::NoEventsToRedo)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

/// Main event journal coordinating undo/redo with event store
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EventJournal {
    stack: UndoRedoStack,
    current_version: u64,
    snapshot_version: u64,
    document_id: EntityId,
}

impl EventJournal {
    pub fn new(document_id: EntityId, snapshot_version: u64) -> Self {
        Self {
            stack: UndoRedoStack::new(100),
            current_version: snapshot_version,
            snapshot_version,
            document_id,
        }
    }

    pub fn record(
        &mut self,
        events: Vec<super::event::DomainEvent>,
        description: String,
    ) -> Result<(), JournalError> {
        if events.is_empty() {
            return Ok(());
        }
        let entry = JournalEntry::new(events, description);
        self.stack.push(entry);
        self.current_version += 1;
        Ok(())
    }

    pub fn undo(&mut self) -> Result<JournalEntry, JournalError> {
        self.stack.undo().map(|entry| {
            self.current_version -= entry.len() as u64;
            entry
        })
    }

    pub fn redo(&mut self) -> Result<JournalEntry, JournalError> {
        self.stack.redo().map(|entry| {
            self.current_version += entry.len() as u64;
            entry
        })
    }

    pub fn can_undo(&self) -> bool {
        self.stack.can_undo()
    }
    pub fn can_redo(&self) -> bool {
        self.stack.can_redo()
    }
    pub fn current_version(&self) -> u64 {
        self.current_version
    }
    pub fn snapshot_version(&self) -> u64 {
        self.snapshot_version
    }
    pub fn changes_since_snapshot(&self) -> u64 {
        self.current_version - self.snapshot_version
    }

    pub fn clear(&mut self) {
        self.stack.clear();
        self.current_version = self.snapshot_version;
    }

    pub fn mark_snapshot(&mut self) {
        self.snapshot_version = self.current_version;
        self.stack.clear();
    }
}
