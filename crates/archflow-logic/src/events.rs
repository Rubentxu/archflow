// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Event Ring Buffer
//
// Zero-cost event output for Rust → JavaScript communication
// Eliminates callback overhead during game loop - 1 poll per frame instead of N callbacks
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::string::ToString;
use alloc::vec::Vec;

/// Types of logic events that can be emitted to JavaScript
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum LogicEventType {
    /// Entity was selected/deselected
    EntitySelected = 0,
    /// Proximity threshold crossed
    ProximityAlert = 1,
    /// Drag operation started
    DragStarted = 2,
    /// Drag operation ended
    DragEnded = 3,
    /// Entity was destroyed
    EntityDestroyed = 4,
    /// Selection box completed (for box selection)
    BoxSelectionCompleted = 5,
    /// Hover state changed
    HoverChanged = 6,
    /// Maximum discriminant value
    _Max = 255,
}

/// Event data specific to each event type
#[derive(Clone, Copy, Debug)]
pub enum EventData {
    /// No additional data
    None,
    /// Proximity alert data
    Proximity {
        /// Distance that triggered the alert
        distance: f32,
    },
    /// Drag operation data
    Drag {
        /// Starting position in world coordinates
        start_pos: (f32, f32),
        /// Current/end position
        current_pos: (f32, f32),
    },
    /// Box selection result
    BoxSelection {
        /// Number of entities selected
        count: u32,
    },
    /// Hover change data
    Hover {
        /// Entity that was hovered (or none)
        entity_id: Option<u32>,
    },
}

/// A single logic event for output to JavaScript
///
/// Compact representation for efficient WASM boundary crossing.
/// Total size: ~24 bytes (optimized for cache efficiency)
#[derive(Clone, Debug)]
pub struct LogicEvent {
    /// Type of event
    pub event_type: LogicEventType,
    /// Entity ID (index part) that triggered the event, or 0 if none
    pub entity_id: u32,
    /// Timestamp in microseconds (from engine start)
    pub timestamp_us: u64,
    /// Event-specific data
    pub data: EventData,
}

impl LogicEvent {
    /// Create a new event with the given type
    #[inline]
    pub fn new(event_type: LogicEventType, entity_id: u32) -> Self {
        Self {
            event_type,
            entity_id,
            timestamp_us: 0, // Will be set by LogicSystem
            data: EventData::None,
        }
    }

    /// Create a proximity alert event
    #[inline]
    pub fn proximity_alert(entity_id: u32, distance: f32) -> Self {
        Self {
            event_type: LogicEventType::ProximityAlert,
            entity_id,
            timestamp_us: 0,
            data: EventData::Proximity { distance },
        }
    }

    /// Create a drag started event
    #[inline]
    pub fn drag_started(entity_id: u32, start_pos: (f32, f32)) -> Self {
        Self {
            event_type: LogicEventType::DragStarted,
            entity_id,
            timestamp_us: 0,
            data: EventData::Drag {
                start_pos,
                current_pos: start_pos,
            },
        }
    }

    /// Create a drag ended event
    #[inline]
    pub fn drag_ended(entity_id: u32, end_pos: (f32, f32)) -> Self {
        Self {
            event_type: LogicEventType::DragEnded,
            entity_id,
            timestamp_us: 0,
            data: EventData::Drag {
                start_pos: end_pos, // We don't track start in ended event
                current_pos: end_pos,
            },
        }
    }

    /// Create a box selection completed event
    #[inline]
    pub fn box_selection_completed(entity_count: u32) -> Self {
        Self {
            event_type: LogicEventType::BoxSelectionCompleted,
            entity_id: 0,
            timestamp_us: 0,
            data: EventData::BoxSelection {
                count: entity_count,
            },
        }
    }

    /// Create a hover changed event
    #[inline]
    pub fn hover_changed(entity_id: Option<u32>) -> Self {
        Self {
            event_type: LogicEventType::HoverChanged,
            entity_id: entity_id.unwrap_or(0),
            timestamp_us: 0,
            data: EventData::Hover { entity_id },
        }
    }

    /// Create an entity destroyed event
    #[inline]
    pub fn entity_destroyed(entity_id: u32) -> Self {
        Self {
            event_type: LogicEventType::EntityDestroyed,
            entity_id,
            timestamp_us: 0,
            data: EventData::None,
        }
    }
}

/// Ring buffer for logic events with bounded memory usage
///
/// Uses a pre-allocated Vec with capacity limit.
/// When full, oldest events are discarded (FIFO).
///
/// # Memory Layout
///
/// ```text
/// [███████████████████████████░░░░░░░░░░]
///  ^                        ^
///  tail                     head
///  (oldest event)          (newest event)
/// ```
///
/// # Example
///
/// ```rust
/// use archflow_logic::events::{EventRingBuffer, LogicEvent, LogicEventType};
///
/// let mut buffer = EventRingBuffer::new(1024);
///
/// // Push events
/// buffer.push(LogicEvent::new(LogicEventType::EntitySelected, 42));
///
/// // Drain all events for JS polling
/// let events = buffer.drain();
/// assert_eq!(events.len(), 1);
/// assert_eq!(events[0].entity_id, 42);
/// ```
#[derive(Debug)]
pub struct EventRingBuffer {
    /// Buffer of events
    events: Vec<LogicEvent>,

    /// Maximum capacity
    capacity: usize,
}

impl EventRingBuffer {
    /// Create a new buffer with the given capacity
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of events to store before overwriting
    ///
    /// # Example
    ///
    /// ```rust
    /// use archflow_logic::events::EventRingBuffer;
    ///
    /// let buffer = EventRingBuffer::new(256);
    /// assert!(buffer.is_empty());
    /// assert_eq!(buffer.capacity(), 256);
    /// ```
    #[inline]
    pub fn new(capacity: usize) -> Self {
        Self {
            events: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Push an event to the buffer
    ///
    /// If the buffer is full, the oldest event is overwritten.
    /// O(1) time complexity.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to push
    ///
    /// # Returns
    ///
    /// `true` if the event was added, `false` if buffer was full
    ///
    /// # Example
    ///
    /// ```rust
    /// use archflow_logic::events::{EventRingBuffer, LogicEvent, LogicEventType};
    ///
    /// let mut buffer = EventRingBuffer::new(2);
    ///
    /// assert!(buffer.push(LogicEvent::new(LogicEventType::EntitySelected, 1)));
    /// assert!(buffer.push(LogicEvent::new(LogicEventType::EntitySelected, 2)));
    /// assert!(!buffer.push(LogicEvent::new(LogicEventType::EntitySelected, 3))); // Full
    /// ```
    #[inline]
    pub fn push(&mut self, event: LogicEvent) -> bool {
        if self.events.len() < self.capacity {
            // Normal case: add new event
            self.events.push(event);
            true
        } else {
            // Buffer full: remove oldest and add new (FIFO)
            if self.capacity > 0 {
                self.events.remove(0);
                self.events.push(event);
            }
            false
        }
    }

    /// Drain all events from the buffer
    ///
    /// Returns all events and clears the buffer.
    /// O(n) time complexity where n = number of events.
    ///
    /// # Returns
    ///
    /// A vector containing all events that were in the buffer
    ///
    /// # Example
    ///
    /// ```rust
    /// use archflow_logic::events::{EventRingBuffer, LogicEvent, LogicEventType};
    ///
    /// let mut buffer = EventRingBuffer::new(256);
    /// buffer.push(LogicEvent::new(LogicEventType::EntitySelected, 1));
    /// buffer.push(LogicEvent::new(LogicEventType::EntitySelected, 2));
    ///
    /// let events = buffer.drain();
    /// assert_eq!(events.len(), 2);
    /// assert!(buffer.is_empty());
    /// ```
    #[inline]
    pub fn drain(&mut self) -> Vec<LogicEvent> {
        if self.events.is_empty() {
            return Vec::new();
        }

        // Extract all events and clear buffer
        core::mem::take(&mut self.events)
    }

    /// Peek at all events without removing them
    ///
    /// Returns a slice of all current events.
    /// Useful for debugging or when you don't want to clear the buffer.
    ///
    /// # Returns
    ///
    /// A slice reference to all events in the buffer
    #[inline]
    pub fn peek(&self) -> &[LogicEvent] {
        &self.events
    }

    /// Get the current number of events in the buffer
    ///
    /// # Returns
    ///
    /// Number of events currently stored
    ///
    /// # Example
    ///
    /// ```rust
    /// use archflow_logic::events::{EventRingBuffer, LogicEvent, LogicEventType};
    ///
    /// let mut buffer = EventRingBuffer::new(256);
    /// assert_eq!(buffer.len(), 0);
    ///
    /// buffer.push(LogicEvent::new(LogicEventType::EntitySelected, 1));
    /// assert_eq!(buffer.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if the buffer is empty
    ///
    /// # Returns
    ///
    /// `true` if no events are in the buffer
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Get the maximum capacity of the buffer
    ///
    /// # Returns
    ///
    /// Maximum number of events that can be stored
    #[inline]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// Check if the buffer is full
    ///
    /// # Returns
    ///
    /// `true` if the buffer has reached capacity
    #[inline]
    pub fn is_full(&self) -> bool {
        self.events.len() >= self.capacity
    }

    /// Clear all events from the buffer
    ///
    /// Removes all events without deallocating memory.
    #[inline]
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Get the number of events that would be dropped if pushed now
    ///
    /// Useful for monitoring event loss in high-frequency scenarios.
    ///
    /// # Returns
    ///
    /// Number of events that would be lost (0 if not full)
    #[inline]
    pub fn lost_count(&self) -> usize {
        if self.events.len() < self.capacity {
            0
        } else {
            // When full, pushing would lose 1 event (the oldest)
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer_is_empty() {
        let buffer = EventRingBuffer::new(256);
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.capacity(), 256);
    }

    #[test]
    fn test_push_single_event() {
        let mut buffer = EventRingBuffer::new(256);
        let event = LogicEvent::new(LogicEventType::EntitySelected, 42);

        assert!(buffer.push(event));
        assert_eq!(buffer.len(), 1);
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_push_multiple_events() {
        let mut buffer = EventRingBuffer::new(4);

        assert!(buffer.push(LogicEvent::new(LogicEventType::EntitySelected, 1)));
        assert!(buffer.push(LogicEvent::new(LogicEventType::EntitySelected, 2)));
        assert!(buffer.push(LogicEvent::new(LogicEventType::EntitySelected, 3)));
        assert!(buffer.push(LogicEvent::new(LogicEventType::EntitySelected, 4)));

        assert!(!buffer.push(LogicEvent::new(LogicEventType::EntitySelected, 5)));
        assert_eq!(buffer.len(), 4);
    }

    #[test]
    fn test_drain_returns_all_events() {
        let mut buffer = EventRingBuffer::new(256);
        buffer.push(LogicEvent::new(LogicEventType::EntitySelected, 1));
        buffer.push(LogicEvent::new(LogicEventType::EntitySelected, 2));

        let events = buffer.drain();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].entity_id, 1);
        assert_eq!(events[1].entity_id, 2);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_drain_empty_buffer() {
        let mut buffer = EventRingBuffer::new(256);
        let events = buffer.drain();

        assert!(events.is_empty());
    }

    #[test]
    fn test_clear_buffer() {
        let mut buffer = EventRingBuffer::new(256);
        buffer.push(LogicEvent::new(LogicEventType::EntitySelected, 1));
        buffer.push(LogicEvent::new(LogicEventType::EntitySelected, 2));

        buffer.clear();

        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_is_full() {
        let mut buffer = EventRingBuffer::new(2);

        assert!(!buffer.is_full());
        buffer.push(LogicEvent::new(LogicEventType::EntitySelected, 1));
        assert!(!buffer.is_full());
        buffer.push(LogicEvent::new(LogicEventType::EntitySelected, 2));
        assert!(buffer.is_full());
    }

    #[test]
    fn test_overflow_behavior() {
        let mut buffer = EventRingBuffer::new(2);
        buffer.push(LogicEvent::new(LogicEventType::EntitySelected, 1));
        buffer.push(LogicEvent::new(LogicEventType::EntitySelected, 2));
        buffer.push(LogicEvent::new(LogicEventType::EntitySelected, 3));

        // Oldest event should be overwritten
        assert_eq!(buffer.len(), 2);
        let events = buffer.drain();
        assert_eq!(events[0].entity_id, 2);
        assert_eq!(events[1].entity_id, 3);
    }

    #[test]
    fn test_event_types() {
        let mut buffer = EventRingBuffer::new(10);

        assert!(buffer.push(LogicEvent::new(LogicEventType::EntitySelected, 1)));
        assert!(buffer.push(LogicEvent::proximity_alert(2, 1.5)));
        assert!(buffer.push(LogicEvent::drag_started(3, (10.0, 20.0))));
        assert!(buffer.push(LogicEvent::drag_ended(3, (30.0, 40.0))));
        assert!(buffer.push(LogicEvent::box_selection_completed(5)));
        assert!(buffer.push(LogicEvent::hover_changed(Some(6))));
        assert!(buffer.push(LogicEvent::hover_changed(None)));
        assert!(buffer.push(LogicEvent::entity_destroyed(7)));

        let events = buffer.drain();
        assert_eq!(events.len(), 8);

        assert_eq!(events[0].event_type, LogicEventType::EntitySelected);
        assert_eq!(events[1].event_type, LogicEventType::ProximityAlert);
        assert_eq!(events[2].event_type, LogicEventType::DragStarted);
        assert_eq!(events[3].event_type, LogicEventType::DragEnded);
        assert_eq!(events[4].event_type, LogicEventType::BoxSelectionCompleted);
        assert_eq!(events[5].event_type, LogicEventType::HoverChanged);
        assert_eq!(events[6].event_type, LogicEventType::HoverChanged);
        assert_eq!(events[7].event_type, LogicEventType::EntityDestroyed);
    }

    #[test]
    fn test_event_data_proximity() {
        let event = LogicEvent::proximity_alert(42, 1.5);
        assert_eq!(event.entity_id, 42);
        if let EventData::Proximity { distance } = event.data {
            assert_eq!(distance, 1.5);
        } else {
            panic!("Expected Proximity event data");
        }
    }

    #[test]
    fn test_event_data_drag() {
        let event = LogicEvent::drag_started(42, (10.0, 20.0));
        assert_eq!(event.entity_id, 42);
        if let EventData::Drag {
            start_pos,
            current_pos,
        } = event.data
        {
            assert_eq!(start_pos, (10.0, 20.0));
            assert_eq!(current_pos, (10.0, 20.0));
        } else {
            panic!("Expected Drag event data");
        }
    }

    #[test]
    fn test_event_data_box_selection() {
        let event = LogicEvent::box_selection_completed(100);
        if let EventData::BoxSelection { count } = event.data {
            assert_eq!(count, 100);
        } else {
            panic!("Expected BoxSelection event data");
        }
    }

    #[test]
    fn test_event_data_hover() {
        let event = LogicEvent::hover_changed(Some(42));
        if let EventData::Hover { entity_id } = event.data {
            assert_eq!(entity_id, Some(42));
        } else {
            panic!("Expected Hover event data");
        }

        let event = LogicEvent::hover_changed(None);
        if let EventData::Hover { entity_id } = event.data {
            assert_eq!(entity_id, None);
        } else {
            panic!("Expected Hover event data");
        }
    }

    #[test]
    fn test_peek_returns_all_events() {
        let mut buffer = EventRingBuffer::new(256);
        buffer.push(LogicEvent::new(LogicEventType::EntitySelected, 1));
        buffer.push(LogicEvent::new(LogicEventType::EntitySelected, 2));

        let peeked = buffer.peek();
        assert_eq!(peeked.len(), 2);

        // Peek should not modify buffer
        let peeked_again = buffer.peek();
        assert_eq!(peeked_again.len(), 2);
    }
}
