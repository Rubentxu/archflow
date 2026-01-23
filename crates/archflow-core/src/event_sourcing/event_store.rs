//! Event Store - Persistent storage for domain events
//!
//! Provides thread-safe storage and retrieval of domain events.
//! Events are stored in order and can be replayed to rebuild aggregate state.

use crate::event_sourcing::event::{DomainEvent, SerializableTime};
use crate::EntityId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// Error types for event store operations
#[derive(Debug, Error, Clone, PartialEq)]
pub enum EventStoreError {
    #[error("Aggregate not found: {0}")]
    AggregateNotFound(EntityId),

    #[error("Concurrency conflict: expected version {expected}, found {found}")]
    ConcurrencyConflict { expected: u64, found: u64 },

    #[error("Event serialization error: {0}")]
    SerializationError(String),

    #[error("Event deserialization error: {0}")]
    DeserializationError(String),

    #[error("IO error: {0}")]
    IoError(String),
}

/// A stored event with metadata for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "DomainEvent: Serialize + for<'a> Deserialize<'a>")]
pub struct StoredEvent {
    /// Unique identifier for this stored event
    pub id: EntityId,
    /// The aggregate this event belongs to
    pub aggregate_id: EntityId,
    /// Position in the event stream
    pub sequence_number: u64,
    /// The event data
    pub event: DomainEvent,
    /// Schema version for forward compatibility
    pub schema_version: u32,
    /// When this event was stored
    pub stored_at: SerializableTime,
}

impl StoredEvent {
    /// Create a new stored event
    pub fn new(
        id: EntityId,
        aggregate_id: EntityId,
        sequence_number: u64,
        event: DomainEvent,
    ) -> Self {
        Self {
            id,
            aggregate_id,
            sequence_number,
            event,
            schema_version: crate::event_sourcing::EVENT_SCHEMA_VERSION,
            stored_at: SerializableTime::default(),
        }
    }
}

/// In-memory event store for fast access
#[derive(Debug, Clone, Default)]
pub struct InMemoryEventStore {
    /// Events organized by aggregate ID
    aggregates: Arc<RwLock<HashMap<EntityId, Vec<StoredEvent>>>>,
    /// Event count per aggregate for quick access
    versions: Arc<RwLock<HashMap<EntityId, u64>>>,
}

impl InMemoryEventStore {
    /// Create a new in-memory event store
    pub fn new() -> Self {
        Self {
            aggregates: Arc::new(RwLock::new(HashMap::new())),
            versions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Save events for an aggregate
    pub fn save_events(
        &mut self,
        aggregate_id: EntityId,
        events: &[DomainEvent],
        expected_version: u64,
    ) -> Result<Vec<StoredEvent>, EventStoreError> {
        let mut aggregates = self
            .aggregates
            .write()
            .map_err(|e| EventStoreError::SerializationError(e.to_string()))?;

        let current_events = aggregates.entry(aggregate_id).or_default();
        let current_version = current_events.len() as u64;

        if current_version != expected_version {
            return Err(EventStoreError::ConcurrencyConflict {
                expected: expected_version,
                found: current_version,
            });
        }

        let stored_events: Vec<StoredEvent> = events
            .iter()
            .enumerate()
            .map(|(i, event)| {
                StoredEvent::new(
                    EntityId::new(),
                    aggregate_id,
                    current_version + i as u64 + 1,
                    event.clone(),
                )
            })
            .collect();

        current_events.extend(stored_events.clone());

        let mut versions = self
            .versions
            .write()
            .map_err(|e| EventStoreError::SerializationError(e.to_string()))?;
        versions.insert(aggregate_id, current_events.len() as u64);

        Ok(stored_events)
    }

    /// Get all events for an aggregate
    pub fn get_events(
        &self,
        aggregate_id: EntityId,
        from_version: u64,
    ) -> Result<Vec<StoredEvent>, EventStoreError> {
        let aggregates = self
            .aggregates
            .read()
            .map_err(|e| EventStoreError::SerializationError(e.to_string()))?;

        if let Some(events) = aggregates.get(&aggregate_id) {
            let filtered: Vec<StoredEvent> = events
                .iter()
                .filter(|e| e.sequence_number > from_version)
                .cloned()
                .collect();
            Ok(filtered)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get the current version of an aggregate
    pub fn get_version(&self, aggregate_id: EntityId) -> Result<u64, EventStoreError> {
        let versions = self
            .versions
            .read()
            .map_err(|e| EventStoreError::SerializationError(e.to_string()))?;

        Ok(versions.get(&aggregate_id).copied().unwrap_or(0))
    }

    /// Check if an aggregate exists
    pub fn aggregate_exists(&self, aggregate_id: EntityId) -> bool {
        let aggregates = self.aggregates.read().unwrap();
        aggregates.contains_key(&aggregate_id)
    }

    /// Get all aggregate IDs
    pub fn aggregate_ids(&self) -> Vec<EntityId> {
        let aggregates = self.aggregates.read().unwrap();
        aggregates.keys().cloned().collect()
    }

    /// Clear all events (for testing)
    pub fn clear(&mut self) {
        let mut aggregates = self.aggregates.write().unwrap();
        let mut versions = self.versions.write().unwrap();
        aggregates.clear();
        versions.clear();
    }
}

/// File-based event store for persistence
#[derive(Debug, Clone)]
pub struct FileEventStore {
    /// In-memory store for fast access
    inner: InMemoryEventStore,
    /// Base directory for event files
    base_dir: PathBuf,
}

impl FileEventStore {
    /// Create a new file-based event store
    pub fn new(base_dir: PathBuf) -> Result<Self, EventStoreError> {
        // Create directory if it doesn't exist
        std::fs::create_dir_all(&base_dir).map_err(|e| {
            EventStoreError::IoError(format!("Failed to create base directory: {}", e))
        })?;

        let mut store = Self {
            inner: InMemoryEventStore::new(),
            base_dir,
        };

        // Load existing events
        store.load_events()?;

        Ok(store)
    }

    /// Load events from disk
    fn load_events(&mut self) -> Result<(), EventStoreError> {
        let entries = std::fs::read_dir(&self.base_dir)
            .map_err(|e| EventStoreError::IoError(e.to_string()))?;

        let mut event_files: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "events")
                    .unwrap_or(false)
            })
            .map(|e| e.path())
            .collect();

        event_files.sort();

        for file in event_files {
            self.load_event_file(&file)?;
        }

        Ok(())
    }

    /// Load events from a single file
    fn load_event_file(&mut self, path: &PathBuf) -> Result<(), EventStoreError> {
        let data =
            std::fs::read_to_string(path).map_err(|e| EventStoreError::IoError(e.to_string()))?;

        // Events are stored one per line as JSON
        for line in data.lines() {
            if line.trim().is_empty() {
                continue;
            }
            // Parse stored event
            let stored: StoredEvent = serde_json::from_str(line)
                .map_err(|e| EventStoreError::DeserializationError(e.to_string()))?;

            // Add to in-memory store
            self.inner.save_events(
                stored.aggregate_id,
                &[stored.event],
                self.inner.get_version(stored.aggregate_id)?,
            )?;
        }

        Ok(())
    }

    /// Get the current event file path
    fn get_current_file(&self) -> PathBuf {
        let base_name = chrono::Utc::now().format("%Y%m%d_%H%M%S.events");
        self.base_dir.join(base_name.to_string())
    }

    /// Persist events to file
    fn persist_events(&self, events: &[StoredEvent]) -> Result<(), EventStoreError> {
        if events.is_empty() {
            return Ok(());
        }

        let file_path = self.get_current_file();
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .map_err(|e| EventStoreError::IoError(e.to_string()))?;

        for event in events {
            let mut json = serde_json::to_string(event)
                .map_err(|e| EventStoreError::SerializationError(e.to_string()))?;
            json.push('\n');
            std::io::Write::write_all(&mut file, json.as_bytes())
                .map_err(|e| EventStoreError::IoError(e.to_string()))?;
        }

        Ok(())
    }
}

/// Trait for event store implementations
pub trait EventStore: Send + Sync {
    /// Save events and return stored events
    fn save_events(
        &mut self,
        aggregate_id: EntityId,
        events: &[DomainEvent],
        expected_version: u64,
    ) -> Result<Vec<StoredEvent>, EventStoreError>;

    /// Get events from a version onwards
    fn get_events(
        &self,
        aggregate_id: EntityId,
        from_version: u64,
    ) -> Result<Vec<StoredEvent>, EventStoreError>;

    /// Get the current version of an aggregate
    fn get_version(&self, aggregate_id: EntityId) -> Result<u64, EventStoreError>;
}

impl EventStore for InMemoryEventStore {
    fn save_events(
        &mut self,
        aggregate_id: EntityId,
        events: &[DomainEvent],
        expected_version: u64,
    ) -> Result<Vec<StoredEvent>, EventStoreError> {
        self.save_events(aggregate_id, events, expected_version)
    }

    fn get_events(
        &self,
        aggregate_id: EntityId,
        from_version: u64,
    ) -> Result<Vec<StoredEvent>, EventStoreError> {
        self.get_events(aggregate_id, from_version)
    }

    fn get_version(&self, aggregate_id: EntityId) -> Result<u64, EventStoreError> {
        self.get_version(aggregate_id)
    }
}

impl EventStore for FileEventStore {
    fn save_events(
        &mut self,
        aggregate_id: EntityId,
        events: &[DomainEvent],
        expected_version: u64,
    ) -> Result<Vec<StoredEvent>, EventStoreError> {
        let stored = self
            .inner
            .save_events(aggregate_id, events, expected_version)?;
        self.persist_events(&stored)?;
        Ok(stored)
    }

    fn get_events(
        &self,
        aggregate_id: EntityId,
        from_version: u64,
    ) -> Result<Vec<StoredEvent>, EventStoreError> {
        self.inner.get_events(aggregate_id, from_version)
    }

    fn get_version(&self, aggregate_id: EntityId) -> Result<u64, EventStoreError> {
        self.inner.get_version(aggregate_id)
    }
}

#[cfg(test)]
mod tests {
    use crate::event_sourcing::event::DomainEvent;
    use crate::event_sourcing::event::EventMetadata;
    use crate::event_sourcing::event_store::{
        EventStore, EventStoreError, InMemoryEventStore, StoredEvent,
    };
    use crate::EntityId;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::{Arc, RwLock};
    use thiserror::Error;

    #[test]
    fn test_save_and_load_events() {
        let mut store = InMemoryEventStore::new();
        let aggregate_id = EntityId::from_u128(1);

        let event1 = DomainEvent::PrimitiveCreated {
            metadata: EventMetadata::for_new_aggregate(aggregate_id, aggregate_id),
            primitive_id: EntityId::from_u128(100),
            primitive_type: "rectangle".to_string(),
            position: (0.0, 0.0),
            size: (100.0, 50.0),
        };

        let event2 = DomainEvent::PrimitiveMoved {
            metadata: EventMetadata::new(
                aggregate_id,
                2,
                aggregate_id,
                "PrimitiveMoved".to_string(),
            ),
            primitive_id: EntityId::from_u128(100),
            from: (0.0, 0.0),
            to: (10.0, 20.0),
        };

        let stored = store
            .save_events(aggregate_id, &[event1.clone()], 0)
            .expect("Failed to save first event");

        assert_eq!(stored.len(), 1);
        assert_eq!(store.get_version(aggregate_id).unwrap(), 1);

        let stored2 = store
            .save_events(aggregate_id, &[event2], 1)
            .expect("Failed to save second event");

        assert_eq!(stored2.len(), 1);
        assert_eq!(store.get_version(aggregate_id).unwrap(), 2);

        let events = store
            .get_events(aggregate_id, 0)
            .expect("Failed to get events");
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_concurrency_conflict() {
        let mut store = InMemoryEventStore::new();
        let aggregate_id = EntityId::from_u128(1);

        let event = DomainEvent::PrimitiveCreated {
            metadata: EventMetadata::for_new_aggregate(aggregate_id, aggregate_id),
            primitive_id: EntityId::from_u128(100),
            primitive_type: "rectangle".to_string(),
            position: (0.0, 0.0),
            size: (100.0, 50.0),
        };

        store
            .save_events(aggregate_id, &[event.clone()], 0)
            .expect("Failed to save");

        // Try to save with wrong version
        let result = store.save_events(aggregate_id, &[event], 0);

        assert!(matches!(
            result,
            Err(EventStoreError::ConcurrencyConflict { .. })
        ));
    }

    #[test]
    fn test_get_events_from_version() {
        let mut store = InMemoryEventStore::new();
        let aggregate_id = EntityId::from_u128(1);

        for i in 1..=5 {
            let event = DomainEvent::PrimitiveCreated {
                metadata: EventMetadata::new(
                    aggregate_id,
                    i as u64,
                    aggregate_id,
                    "PrimitiveCreated".to_string(),
                ),
                primitive_id: EntityId::from_u128(i),
                primitive_type: "rectangle".to_string(),
                position: (0.0, 0.0),
                size: (100.0, 50.0),
            };
            store
                .save_events(aggregate_id, &[event], (i - 1) as u64)
                .unwrap();
        }

        let events = store.get_events(aggregate_id, 3).unwrap();
        assert_eq!(events.len(), 2); // Events 4 and 5
    }

    #[test]
    fn test_event_serialization() {
        let aggregate_id = EntityId::from_u128(1);
        let event = DomainEvent::PrimitiveCreated {
            metadata: EventMetadata::for_new_aggregate(aggregate_id, aggregate_id),
            primitive_id: EntityId::from_u128(100),
            primitive_type: "rectangle".to_string(),
            position: (10.5, 20.5),
            size: (100.0, 50.0),
        };

        // Serialize
        let serialized = serde_json::to_string(&event).expect("Failed to serialize");

        // Deserialize
        let deserialized: DomainEvent =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        match deserialized {
            DomainEvent::PrimitiveCreated { position, size, .. } => {
                assert_eq!(position, (10.5, 20.5));
                assert_eq!(size, (100.0, 50.0));
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_event_invert() {
        let aggregate_id = EntityId::from_u128(1);
        let event = DomainEvent::PrimitiveMoved {
            metadata: EventMetadata::new(
                aggregate_id,
                1,
                aggregate_id,
                "PrimitiveMoved".to_string(),
            ),
            primitive_id: EntityId::from_u128(100),
            from: (0.0, 0.0),
            to: (10.0, 20.0),
        };

        let inverted = event.invert();

        match inverted {
            DomainEvent::PrimitiveMoved { to, from, .. } => {
                assert_eq!(from, (10.0, 20.0));
                assert_eq!(to, (0.0, 0.0));
            }
            _ => panic!("Wrong event type"),
        }
    }
}
