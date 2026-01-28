//! Event Sourcing module for ArchFlow SDK
//!
//! This module provides event sourcing infrastructure for undo/redo functionality
//! and collaborative editing. All canvas modifications are captured as events
//! and can be replayed to reconstruct state.

use crate::canvas::{
    CanvasOperation, Shape, ShapeGeometry, ShapeProperties, ShapeStyle, ShapeType,
};
use crate::layers::C4Level;
use archflow_core::{Color, EntityId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// A unique identifier for events
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub u64);

impl EventId {
    /// Creates a new event ID
    pub fn new() -> Self {
        Self(rand::random())
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a single atomic change to the canvas
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CanvasEvent {
    /// A shape was created
    ShapeCreated {
        /// ID of the created shape
        shape_id: EntityId,
        /// The shape data at creation time
        shape_data: ShapeData,
    },
    /// A shape was updated
    ShapeUpdated {
        /// ID of the updated shape
        shape_id: EntityId,
        /// The previous state of the shape
        previous: ShapeData,
        /// The new state of the shape
        current: ShapeData,
    },
    /// A shape was deleted
    ShapeDeleted {
        /// ID of the deleted shape
        shape_id: EntityId,
        /// The shape data before deletion
        shape_data: ShapeData,
    },
    /// Multiple shapes were modified in a batch
    Batch {
        /// Events contained in the batch
        events: Vec<CanvasEvent>,
    },
    /// Viewport changed
    ViewportChanged {
        /// Previous viewport state
        previous: ViewportSnapshot,
        /// New viewport state
        current: ViewportSnapshot,
    },
    /// Layer was created
    LayerCreated {
        /// ID of the created layer
        layer_id: EntityId,
        /// Layer name
        name: String,
        /// C4 level
        c4_level: C4Level,
    },
    /// Layer was deleted
    LayerDeleted {
        /// ID of the deleted layer
        layer_id: EntityId,
        /// Layer name
        name: String,
    },
    /// Layer visibility changed
    LayerVisibilityChanged {
        /// ID of the layer
        layer_id: EntityId,
        /// Previous visibility state
        previous: bool,
        /// New visibility state
        current: bool,
    },
    /// Layer opacity changed
    LayerOpacityChanged {
        /// ID of the layer
        layer_id: EntityId,
        /// Previous opacity
        previous: f32,
        /// New opacity
        current: f32,
    },
}

impl CanvasEvent {
    /// Returns the event ID if this is a batch
    pub fn is_batch(&self) -> bool {
        matches!(self, CanvasEvent::Batch { .. })
    }

    /// Unpacks a batch into individual events
    pub fn unpack_batch(self) -> Vec<CanvasEvent> {
        match self {
            CanvasEvent::Batch { events } => events,
            _ => vec![self],
        }
    }
}

/// Snapshot of shape data for serialization
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShapeData {
    /// Shape ID
    pub id: EntityId,
    /// Shape type
    pub shape_type: ShapeType,
    /// X position
    pub x: f32,
    /// Y position
    pub y: f32,
    /// Width
    pub width: f32,
    /// Height
    pub height: f32,
    /// Fill color
    pub fill_color: Color,
    /// Stroke color (if any)
    pub stroke_color: Option<Color>,
    /// Stroke width
    pub stroke_width: f32,
    /// Opacity
    pub opacity: f32,
    /// Rotation in degrees
    pub rotation: f32,
    /// Layer ID
    pub layer_id: EntityId,
    /// Whether selected
    pub selected: bool,
    /// Custom properties
    pub properties: HashMap<String, Value>,
}

impl From<Shape> for ShapeData {
    fn from(shape: Shape) -> Self {
        Self {
            id: shape.id,
            shape_type: shape.shape_type,
            x: shape.x,
            y: shape.y,
            width: shape.width,
            height: shape.height,
            fill_color: shape.fill_color,
            stroke_color: shape.stroke_color,
            stroke_width: shape.stroke_width,
            opacity: shape.opacity,
            rotation: shape.rotation,
            layer_id: shape.layer_id,
            selected: shape.selected,
            properties: HashMap::new(), // Convert ShapeProperties to HashMap if needed
        }
    }
}

impl From<ShapeData> for Shape {
    fn from(data: ShapeData) -> Self {
        let geometry =
            ShapeGeometry::from_components(data.x, data.y, data.width, data.height, data.rotation);
        let style = ShapeStyle::solid(data.fill_color, data.stroke_color, data.stroke_width);
        let opacity = data.opacity;
        Self {
            id: data.id,
            shape_type: data.shape_type,
            geometry,
            style: style.with_opacity(opacity),
            layer_id: data.layer_id,
            selected: data.selected,
            properties: ShapeProperties::new(),
            // Backwards compatibility fields
            x: data.x,
            y: data.y,
            width: data.width,
            height: data.height,
            rotation: data.rotation,
            fill_color: data.fill_color,
            stroke_color: data.stroke_color,
            stroke_width: data.stroke_width,
            opacity: data.opacity,
        }
    }
}

/// Snapshot of viewport state
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ViewportSnapshot {
    /// Offset X
    pub offset_x: f32,
    /// Offset Y
    pub offset_y: f32,
    /// Zoom level
    pub zoom: f32,
}

/// Metadata for an event
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Unique event ID
    pub id: EventId,
    /// Timestamp of the event
    pub timestamp: std::time::SystemTime,
    /// User or session that created the event
    pub author: String,
    /// Description of the change
    pub message: String,
    /// Hash of the previous event for chain verification
    pub previous_event_hash: Option<u64>,
}

impl EventMetadata {
    /// Creates new metadata for an event
    pub fn new(author: String, message: String) -> Self {
        Self {
            id: EventId::new(),
            timestamp: std::time::SystemTime::now(),
            author,
            message,
            previous_event_hash: None,
        }
    }
}

/// A complete event with its metadata
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecordedEvent {
    /// Event metadata
    pub metadata: EventMetadata,
    /// The actual event data
    pub event: CanvasEvent,
}

impl RecordedEvent {
    /// Creates a new recorded event
    pub fn new(event: CanvasEvent, author: String, message: String) -> Self {
        Self {
            metadata: EventMetadata::new(author, message),
            event,
        }
    }
}

/// Configuration for the event store
#[derive(Clone, Debug)]
pub struct EventStoreConfig {
    /// Maximum number of events to keep (0 = unlimited)
    pub max_events: usize,
    /// Maximum undo stack size
    pub max_undo_depth: usize,
    /// Whether to enable event compression
    pub enable_compression: bool,
}

impl Default for EventStoreConfig {
    fn default() -> Self {
        Self {
            max_events: 10000,
            max_undo_depth: 50,
            enable_compression: true,
        }
    }
}

/// Result of applying an event
#[derive(Debug, thiserror::Error)]
pub enum EventApplyError {
    #[error("Event validation failed: {0}")]
    ValidationError(String),
    #[error("Event application failed: {0}")]
    ApplicationError(String),
    #[error("Event not found: {0:?}")]
    EventNotFound(EventId),
}

/// Trait for objects that can apply events
pub trait EventHandler {
    /// Applies a canvas event to this handler
    fn apply_event(&mut self, event: &CanvasEvent) -> Result<(), EventApplyError>;
}

/// Event store for managing and replaying events
#[derive(Debug)]
pub struct EventStore {
    /// All recorded events
    events: Vec<RecordedEvent>,
    /// Configuration
    config: EventStoreConfig,
    /// Current event hash for chain verification
    current_hash: u64,
}

impl EventStore {
    /// Creates a new empty event store
    pub fn new(config: EventStoreConfig) -> Self {
        Self {
            events: Vec::new(),
            config,
            current_hash: 0,
        }
    }

    /// Creates a new event store with default config
    pub fn default() -> Self {
        Self::new(EventStoreConfig::default())
    }

    /// Appends a new event to the store
    pub fn append(&mut self, event: RecordedEvent) {
        // Calculate hash for chain verification
        self.current_hash = Self::calculate_hash(&event, self.current_hash);

        // Store the hash in metadata
        let hash = self.current_hash;
        self.events.push(event);

        // Trim old events if limit is set
        if self.config.max_events > 0 && self.events.len() > self.config.max_events {
            self.events
                .drain(0..(self.events.len() - self.config.max_events));
        }
    }

    /// Appends multiple events as a batch
    pub fn append_batch(&mut self, events: Vec<RecordedEvent>) {
        for event in events {
            self.append(event);
        }
    }

    /// Returns all events
    pub fn events(&self) -> &[RecordedEvent] {
        &self.events
    }

    /// Returns the number of events
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Returns whether the store is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Calculates a simple hash for chain verification
    fn calculate_hash(event: &RecordedEvent, previous_hash: u64) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        previous_hash.hash(&mut hasher);
        event.metadata.id.hash(&mut hasher);
        event.metadata.timestamp.hash(&mut hasher);

        hasher.finish()
    }

    /// Replays all events through a handler
    pub fn replay<H: EventHandler>(&self, handler: &mut H) -> Result<(), EventApplyError> {
        for recorded in &self.events {
            handler.apply_event(&recorded.event)?;
        }
        Ok(())
    }

    /// Gets events in a range
    pub fn range(&self, start: usize, end: usize) -> &[RecordedEvent] {
        &self.events[start.min(self.events.len())..end.min(self.events.len())]
    }

    /// Gets events since a specific event ID
    pub fn since(&self, event_id: EventId) -> &[RecordedEvent] {
        let pos = self.events.iter().position(|e| e.metadata.id == event_id);
        match pos {
            Some(p) => &self.events[p + 1..],
            None => &[],
        }
    }
}

/// Undo/Redo manager using event sourcing
#[derive(Debug)]
pub struct UndoManager {
    /// Undo stack (past states)
    undo_stack: Vec<EventSnapshot>,
    /// Redo stack (undone states)
    redo_stack: Vec<EventSnapshot>,
    /// Maximum undo depth
    max_undo_depth: usize,
    /// Current author for events
    current_author: String,
}

impl UndoManager {
    /// Creates a new UndoManager
    pub fn new(max_undo_depth: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_undo_depth,
            current_author: "user".to_string(),
        }
    }

    /// Creates a new UndoManager with default depth
    pub fn default() -> Self {
        Self::new(50)
    }

    /// Sets the current author for events
    pub fn set_author(&mut self, author: String) {
        self.current_author = author;
    }

    /// Saves the current state to the undo stack
    pub fn save_state(&mut self, event_store: &EventStore) {
        let snapshot = EventSnapshot::from_store(event_store);

        self.undo_stack.push(snapshot);

        // Clear redo stack on new action
        self.redo_stack.clear();

        // Trim undo stack if needed
        if self.undo_stack.len() > self.max_undo_depth {
            self.undo_stack.remove(0);
        }
    }

    /// Saves a single event to history
    pub fn save_event(&mut self, event: RecordedEvent) {
        let snapshot = EventSnapshot::from_event(event);

        self.undo_stack.push(snapshot);
        self.redo_stack.clear();

        if self.undo_stack.len() > self.max_undo_depth {
            self.undo_stack.remove(0);
        }
    }

    /// Checks if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Checks if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Returns the number of available undo operations
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Returns the number of available redo operations
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Undoes the last operation
    pub fn undo<H: EventHandler>(
        &mut self,
        event_store: &mut EventStore,
        handler: &mut H,
    ) -> Option<RecordedEvent> {
        if let Some(snapshot) = self.undo_stack.pop() {
            // Apply inverse operations from snapshot
            let inverse_events = snapshot.create_inverse(handler);

            // Store in redo stack
            self.redo_stack.push(snapshot);

            // Apply inverse events
            for event in &inverse_events {
                handler.apply_event(&event.event).ok();
                event_store.append(event.clone());
            }

            // Return the last undone event for UI feedback
            inverse_events.last().cloned()
        } else {
            None
        }
    }

    /// Redoes the last undone operation
    pub fn redo<H: EventHandler>(
        &mut self,
        event_store: &mut EventStore,
        handler: &mut H,
    ) -> Option<RecordedEvent> {
        if let Some(snapshot) = self.redo_stack.pop() {
            // Re-apply original events
            let events = snapshot.events.clone();

            // Store in undo stack
            self.undo_stack.push(snapshot);

            // Apply original events
            for event in &events {
                handler.apply_event(&event.event).ok();
                event_store.append(event.clone());
            }

            events.last().cloned()
        } else {
            None
        }
    }

    /// Clears all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

/// A snapshot of events at a point in time
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EventSnapshot {
    /// Events in this snapshot
    events: Vec<RecordedEvent>,
    /// Hash of the snapshot
    hash: u64,
}

impl EventSnapshot {
    /// Creates a snapshot from an event store
    fn from_store(store: &EventStore) -> Self {
        let events = store.events().to_vec();
        let hash = Self::calculate_hash(&events);

        Self { events, hash }
    }

    /// Creates a snapshot from a single event
    fn from_event(event: RecordedEvent) -> Self {
        let events = vec![event.clone()];
        let hash = Self::calculate_hash(&events);

        Self { events, hash }
    }

    /// Calculates a hash for the snapshot
    fn calculate_hash(events: &[RecordedEvent]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        events.len().hash(&mut hasher);
        for event in events {
            event.metadata.id.hash(&mut hasher);
        }

        hasher.finish()
    }

    /// Creates inverse events to undo this snapshot
    fn create_inverse<H: EventHandler>(&self, _handler: &H) -> Vec<RecordedEvent> {
        // This is a simplified implementation
        // A full implementation would track the inverse of each event
        Vec::new()
    }
}

/// Builder for creating canvas events
#[derive(Debug, Default)]
pub struct EventBuilder {
    author: String,
}

impl EventBuilder {
    /// Creates a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the author
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }

    /// Creates a shape created event
    pub fn shape_created(&self, shape: &Shape) -> RecordedEvent {
        RecordedEvent::new(
            CanvasEvent::ShapeCreated {
                shape_id: shape.id,
                shape_data: ShapeData::from(shape.clone()),
            },
            self.author.clone(),
            format!("Created shape {}", shape.id),
        )
    }

    /// Creates a shape updated event
    pub fn shape_updated(
        &self,
        shape_id: EntityId,
        previous: &Shape,
        current: &Shape,
    ) -> RecordedEvent {
        RecordedEvent::new(
            CanvasEvent::ShapeUpdated {
                shape_id,
                previous: ShapeData::from(previous.clone()),
                current: ShapeData::from(current.clone()),
            },
            self.author.clone(),
            format!("Updated shape {}", shape_id),
        )
    }

    /// Creates a shape deleted event
    pub fn shape_deleted(&self, shape: &Shape) -> RecordedEvent {
        RecordedEvent::new(
            CanvasEvent::ShapeDeleted {
                shape_id: shape.id,
                shape_data: ShapeData::from(shape.clone()),
            },
            self.author.clone(),
            format!("Deleted shape {}", shape.id),
        )
    }

    /// Creates a batch event from multiple operations
    pub fn batch(&self, events: Vec<CanvasEvent>) -> RecordedEvent {
        let count = events.len();
        RecordedEvent::new(
            CanvasEvent::Batch { events },
            self.author.clone(),
            format!("Batch operation with {} events", count),
        )
    }
}

/// Converts a canvas operation to events
impl From<CanvasOperation> for CanvasEvent {
    fn from(op: CanvasOperation) -> Self {
        match op {
            CanvasOperation::CreateShape(shape) => CanvasEvent::ShapeCreated {
                shape_id: shape.id,
                shape_data: ShapeData::from(shape),
            },
            CanvasOperation::UpdateShape(id, previous, current) => CanvasEvent::ShapeUpdated {
                shape_id: id,
                previous: ShapeData::from(previous),
                current: ShapeData::from(current),
            },
            CanvasOperation::DeleteShape(id, shape) => CanvasEvent::ShapeDeleted {
                shape_id: id,
                shape_data: ShapeData::from(shape),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_id_creation() {
        let id1 = EventId::new();
        let id2 = EventId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_recorded_event_creation() {
        let shape = Shape::new_rectangle(0.0, 0.0, 100.0, 100.0);
        let event = CanvasEvent::ShapeCreated {
            shape_id: shape.id,
            shape_data: ShapeData::from(shape),
        };

        let recorded = RecordedEvent::new(event, "test_user".to_string(), "Test event".to_string());

        assert_eq!(recorded.metadata.author, "test_user");
        assert_eq!(recorded.metadata.message, "Test event");
        assert!(recorded.metadata.id != EventId::default());
    }

    #[test]
    fn test_event_store() {
        let mut store = EventStore::default();

        let shape = Shape::new_rectangle(0.0, 0.0, 100.0, 100.0);
        let event = RecordedEvent::new(
            CanvasEvent::ShapeCreated {
                shape_id: shape.id,
                shape_data: ShapeData::from(shape),
            },
            "test".to_string(),
            "Created shape".to_string(),
        );

        store.append(event.clone());

        assert_eq!(store.len(), 1);
        assert!(!store.is_empty());
    }

    #[test]
    fn test_undo_manager_limits() {
        let mut manager = UndoManager::new(3);

        for i in 0..5 {
            let shape = Shape::new_rectangle(i as f32, 0.0, 100.0, 100.0);
            let event = RecordedEvent::new(
                CanvasEvent::ShapeCreated {
                    shape_id: shape.id,
                    shape_data: ShapeData::from(shape),
                },
                "test".to_string(),
                format!("Event {}", i),
            );
            manager.save_event(event);
        }

        assert_eq!(manager.undo_count(), 3);
        assert_eq!(manager.redo_count(), 0);
    }

    #[test]
    fn test_event_builder() {
        let builder = EventBuilder::new().author("test_user");
        let shape = Shape::new_rectangle(0.0, 0.0, 100.0, 100.0);

        let event = builder.shape_created(&shape);

        assert_eq!(event.metadata.author, "test_user");
        match &event.event {
            CanvasEvent::ShapeCreated { shape_data, .. } => {
                assert_eq!(shape_data.width, 100.0);
                assert_eq!(shape_data.height, 100.0);
            }
            _ => panic!("Expected ShapeCreated event"),
        }
    }

    #[test]
    fn test_batch_event_unpack() {
        let shape1 = Shape::new_rectangle(0.0, 0.0, 100.0, 100.0);
        let shape2 = Shape::new_rectangle(50.0, 50.0, 50.0, 50.0);

        let batch = CanvasEvent::Batch {
            events: vec![
                CanvasEvent::ShapeCreated {
                    shape_id: shape1.id,
                    shape_data: ShapeData::from(shape1),
                },
                CanvasEvent::ShapeDeleted {
                    shape_id: shape2.id,
                    shape_data: ShapeData::from(shape2),
                },
            ],
        };

        let unpacked = batch.unpack_batch();
        assert_eq!(unpacked.len(), 2);
    }

    #[test]
    fn test_shape_data_conversion() {
        let original = Shape::new_ellipse(100.0, 100.0, 50.0, 75.0);
        let data = ShapeData::from(original.clone());
        let back = Shape::from(data);

        assert_eq!(original.x, back.x);
        assert_eq!(original.y, back.y);
        assert_eq!(original.width, back.width);
        assert_eq!(original.height, back.height);
        assert_eq!(original.shape_type, back.shape_type);
    }
}
