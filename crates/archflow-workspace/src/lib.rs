//! ArchFlow Workspace - Gestión de documento, undo/redo, selección
//!
//! Este crate contiene la lógica de alto nivel del engine:
//! - Document con event sourcing
//! - Undo/Redo manager
//! - Selección y comandos

use archflow_core::{EntityId, Vec2};

pub struct Document {
    /// Event journal para event sourcing
    event_journal: EventJournal,

    /// Undo/Redo manager
    undo_manager: UndoManager,

    /// Selección actual
    selection: SelectionState,
}

pub struct EventJournal {
    commits: Vec<Commit>,
    heads: std::collections::HashMap<String, usize>,
}

pub struct Commit {
    id: uuid::Uuid,
    events: Vec<DomainEvent>,
    author: String,
    timestamp: std::time::SystemTime,
    message: String,
}

#[derive(Clone)]
pub enum DomainEvent {
    EntityCreated {
        entity_id: EntityId,
    },
    EntityMoved {
        entity_id: EntityId,
        from: Vec2,
        to: Vec2,
    },
    EntityDeleted {
        entity_id: EntityId,
    },
    SelectionChanged {
        added: Vec<EntityId>,
        removed: Vec<EntityId>,
    },
}

pub struct UndoManager {
    undo_stack: Vec<uuid::Uuid>,
    redo_stack: Vec<uuid::Uuid>,
    max_undo_depth: usize,
}

pub struct SelectionState {
    selected: std::collections::HashSet<EntityId>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            event_journal: EventJournal::new(),
            undo_manager: UndoManager::new(100),
            selection: SelectionState::new(),
        }
    }

    pub fn create_entity(&mut self, id: EntityId) {
        let event = DomainEvent::EntityCreated { entity_id: id };
        self.event_journal.add_event(event.clone());
    }

    pub fn move_entity(&mut self, id: EntityId, from: Vec2, to: Vec2) {
        let event = DomainEvent::EntityMoved {
            entity_id: id,
            from,
            to,
        };
        self.event_journal.add_event(event);
    }

    pub fn undo(&mut self) -> Result<(), anyhow::Error> {
        self.undo_manager.undo()
    }

    pub fn redo(&mut self) -> Result<(), anyhow::Error> {
        self.undo_manager.redo()
    }
}

impl EventJournal {
    fn new() -> Self {
        Self {
            commits: Vec::new(),
            heads: std::collections::HashMap::new(),
        }
    }

    fn add_event(&mut self, _event: DomainEvent) {
        // Placeholder: agrupar eventos en commits
    }
}

impl UndoManager {
    fn new(max_depth: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_undo_depth: max_depth,
        }
    }

    fn undo(&mut self) -> Result<(), anyhow::Error> {
        Ok(())
    }

    fn redo(&mut self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

impl SelectionState {
    fn new() -> Self {
        Self {
            selected: std::collections::HashSet::new(),
        }
    }

    pub fn select(&mut self, id: EntityId) {
        self.selected.insert(id);
    }

    pub fn deselect(&mut self, id: EntityId) {
        self.selected.remove(&id);
    }

    pub fn clear(&mut self) {
        self.selected.clear();
    }

    pub fn is_selected(&self, id: EntityId) -> bool {
        self.selected.contains(&id)
    }

    pub fn selected_ids(&self) -> impl Iterator<Item = &EntityId> {
        self.selected.iter()
    }
}
