// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Interaction - Input Processor (SharedArrayBuffer)
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 7
//
// Lock-free input processing via SharedArrayBuffer:
// - Ring buffer between JS and WASM
// - Zero-copy event access
// - Pointer event coalescing
// - Keyboard modifier tracking
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(unused_imports)]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use archflow_core::Vec2;

/// Maximum number of pointers to track
pub const MAX_POINTERS: u32 = 10;

/// Event capacity in the ring buffer
pub const EVENT_CAPACITY: usize = 256;

/// Size of each event in bytes
pub const EVENT_SIZE: usize = 32;

/// Event type enum (stored as u8)
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputEventType {
    /// Pointer down (mouse press, touch start)
    PointerDown = 0,
    /// Pointer move (mouse move, touch move)
    PointerMove = 1,
    /// Pointer up (mouse release, touch end)
    PointerUp = 2,
    /// Wheel (mouse wheel, trackpad scroll)
    Wheel = 3,
    /// Key down
    KeyDown = 4,
    /// Key up
    KeyUp = 5,
}

impl InputEventType {
    /// Convert from u8
    pub const fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::PointerDown,
            1 => Self::PointerMove,
            2 => Self::PointerUp,
            3 => Self::Wheel,
            4 => Self::KeyDown,
            5 => Self::KeyUp,
            _ => Self::PointerMove,
        }
    }
}

/// Mouse button state
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Buttons {
    /// Primary button (left mouse)
    pub primary: bool,

    /// Secondary button (right mouse)
    pub secondary: bool,

    /// Auxiliary button (middle mouse)
    pub auxiliary: bool,
}

impl Buttons {
    /// Create from u8 bitmask
    pub const fn from_u8(value: u8) -> Self {
        Self {
            primary: (value & 0x01) != 0,
            secondary: (value & 0x02) != 0,
            auxiliary: (value & 0x04) != 0,
        }
    }

    /// Convert to u8 bitmask
    pub const fn to_u8(self) -> u8 {
        let mut value = 0u8;
        if self.primary {
            value |= 0x01;
        }
        if self.secondary {
            value |= 0x02;
        }
        if self.auxiliary {
            value |= 0x04;
        }
        value
    }
}

/// Keyboard modifier state
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Modifiers {
    /// Shift key
    pub shift: bool,

    /// Control key
    pub ctrl: bool,

    /// Alt key
    pub alt: bool,

    /// Meta/Command key
    pub meta: bool,
}

impl Modifiers {
    /// Create from u8 bitmask
    pub const fn from_u8(value: u8) -> Self {
        Self {
            shift: (value & 0x01) != 0,
            ctrl: (value & 0x02) != 0,
            alt: (value & 0x04) != 0,
            meta: (value & 0x08) != 0,
        }
    }

    /// Convert to u8 bitmask
    pub const fn to_u8(self) -> u8 {
        let mut value = 0u8;
        if self.shift {
            value |= 0x01;
        }
        if self.ctrl {
            value |= 0x02;
        }
        if self.alt {
            value |= 0x04;
        }
        if self.meta {
            value |= 0x08;
        }
        value
    }
}

/// Raw input event from SharedArrayBuffer
///
/// This structure must match the layout on the JS side exactly.
/// It's designed to be 32 bytes for efficient cache line alignment.
#[repr(C, align(8))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RawInputEvent {
    /// Event timestamp (milliseconds since page load)
    pub timestamp: f64,

    /// Pointer ID (0 for mouse, 1+ for touch)
    pub pointer_id: u32,

    /// X position in screen coordinates
    pub x: f32,

    /// Y position in screen coordinates
    pub y: f32,

    /// Pressure (0.0 - 1.0) or wheel delta for wheel events
    pub pressure: f32,

    /// Event type (0-5)
    pub event_type: u8,

    /// Button state (bitmask)
    pub buttons: u8,

    /// Keyboard modifiers (bitmask)
    pub modifiers: u8,

    /// Reserved for future use
    _reserved: u8,
}

impl RawInputEvent {
    /// Get the event type
    pub const fn event_type_value(&self) -> InputEventType {
        InputEventType::from_u8(self.event_type)
    }

    /// Get the button state
    pub const fn button_state(&self) -> Buttons {
        Buttons::from_u8(self.buttons)
    }

    /// Get the modifier state
    pub const fn modifier_state(&self) -> Modifiers {
        Modifiers::from_u8(self.modifiers)
    }

    /// Get the position as a Vec2
    pub const fn position(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

/// Ring buffer for lock-free event sharing between JS and WASM
///
/// SharedArrayBuffer layout:
/// - 4 bytes: head index (write position)
/// - 4 bytes: tail index (read position)
/// - EVENT_SIZE * EVENT_CAPACITY bytes: event data
pub struct InputRingBuffer {
    /// Head index (write position)
    head: usize,

    /// Tail index (read position)
    tail: usize,

    /// Event data buffer
    data: Vec<RawInputEvent>,
}

impl InputRingBuffer {
    /// Create a new input ring buffer
    pub fn new() -> Self {
        Self {
            head: 0,
            tail: 0,
            data: vec![
                RawInputEvent {
                    timestamp: 0.0,
                    pointer_id: 0,
                    x: 0.0,
                    y: 0.0,
                    pressure: 0.0,
                    event_type: 0,
                    buttons: 0,
                    modifiers: 0,
                    _reserved: 0,
                };
                EVENT_CAPACITY
            ],
        }
    }

    /// Push an event to the buffer
    ///
    /// Returns true if successful, false if buffer is full
    pub fn push_event(&mut self, event: RawInputEvent) -> bool {
        let next_head = (self.head + 1) % EVENT_CAPACITY;

        // Check if buffer is full
        if next_head == self.tail {
            return false;
        }

        self.data[self.head] = event;
        self.head = next_head;
        true
    }

    /// Drain all events from the buffer
    ///
    /// Returns a vector of all pending events
    pub fn drain(&mut self) -> Vec<RawInputEvent> {
        let mut events = Vec::new();

        while self.tail != self.head {
            events.push(self.data[self.tail]);
            self.tail = (self.tail + 1) % EVENT_CAPACITY;
        }

        events
    }

    /// Get the number of events in the buffer
    pub fn len(&self) -> usize {
        if self.head >= self.tail {
            self.head - self.tail
        } else {
            EVENT_CAPACITY - self.tail + self.head
        }
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
    }
}

impl Default for InputRingBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Selection state for tracking dragged entities
#[derive(Clone, Debug, PartialEq)]
pub struct SelectionState {
    /// ID of the selected entity
    pub id: archflow_core::EntityId,

    /// Start position of the drag
    pub start_pos: Vec2,

    /// Current position
    pub current_pos: Vec2,
}

/// Input processor that handles all user input
pub struct InputProcessor {
    /// Ring buffer for incoming events
    ring_buffer: InputRingBuffer,

    /// Current selection state
    selection: Option<SelectionState>,

    /// Whether we're currently dragging
    is_dragging: bool,
}

impl InputProcessor {
    /// Create a new input processor
    pub fn new() -> Self {
        Self {
            ring_buffer: InputRingBuffer::new(),
            selection: None,
            is_dragging: false,
        }
    }

    /// Push an event to the input processor
    pub fn push_event(&mut self, event: RawInputEvent) -> bool {
        self.ring_buffer.push_event(event)
    }

    /// Drain all pending events
    pub fn drain_events(&mut self) -> Vec<RawInputEvent> {
        self.ring_buffer.drain()
    }

    /// Start a selection operation
    pub fn start_selection(&mut self, id: archflow_core::EntityId) {
        self.selection = Some(SelectionState {
            id,
            start_pos: Vec2::new(0.0, 0.0),
            current_pos: Vec2::new(0.0, 0.0),
        });
    }

    /// End the current selection
    pub fn end_selection(&mut self) {
        self.selection = None;
        self.is_dragging = false;
    }

    /// Get the current selection
    pub fn get_selection(&self) -> Option<&SelectionState> {
        self.selection.as_ref()
    }

    /// Check if we're currently dragging
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }

    /// Set the dragging state
    pub fn set_dragging(&mut self, dragging: bool) {
        self.is_dragging = dragging;
    }

    /// Get the ring buffer (for direct access from WASM bridge)
    pub fn ring_buffer(&mut self) -> &mut InputRingBuffer {
        &mut self.ring_buffer
    }
}

impl Default for InputProcessor {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_size() {
        // RawInputEvent should be exactly 32 bytes
        assert_eq!(core::mem::size_of::<RawInputEvent>(), 32);
        assert_eq!(core::mem::align_of::<RawInputEvent>(), 8);
    }

    #[test]
    fn test_ring_buffer_creation() {
        let buffer = InputRingBuffer::new();
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_ring_buffer_push() {
        let mut buffer = InputRingBuffer::new();

        let event = RawInputEvent {
            timestamp: 1000.0,
            pointer_id: 0,
            x: 100.0,
            y: 200.0,
            pressure: 0.5,
            event_type: 0,
            buttons: 0,
            modifiers: 0,
            _reserved: 0,
        };

        assert!(buffer.push_event(event));
        assert_eq!(buffer.len(), 1);
    }

    #[test]
    fn test_ring_buffer_drain() {
        let mut buffer = InputRingBuffer::new();

        let event = RawInputEvent {
            timestamp: 1000.0,
            pointer_id: 0,
            x: 100.0,
            y: 200.0,
            pressure: 0.5,
            event_type: 0,
            buttons: 0,
            modifiers: 0,
            _reserved: 0,
        };

        buffer.push_event(event);
        let events = buffer.drain();

        assert_eq!(events.len(), 1);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_ring_buffer_wraparound() {
        let mut buffer = InputRingBuffer::new();
        buffer.tail = EVENT_CAPACITY - 1;
        buffer.head = EVENT_CAPACITY - 1;

        let event = RawInputEvent {
            timestamp: 1000.0,
            pointer_id: 0,
            x: 100.0,
            y: 200.0,
            pressure: 0.5,
            event_type: 0,
            buttons: 0,
            modifiers: 0,
            _reserved: 0,
        };

        buffer.push_event(event);
        assert_eq!(buffer.head, 0); // Wrapped around
    }

    #[test]
    fn test_buttons_bitmask() {
        let buttons = Buttons::from_u8(0x05);
        assert!(buttons.primary);
        assert!(!buttons.secondary);
        assert!(buttons.auxiliary);

        assert_eq!(buttons.to_u8(), 0x05);
    }

    #[test]
    fn test_modifiers_bitmask() {
        let modifiers = Modifiers::from_u8(0x0B);
        assert!(modifiers.shift);
        assert!(modifiers.ctrl);
        assert!(!modifiers.alt);
        assert!(modifiers.meta);

        assert_eq!(modifiers.to_u8(), 0x0B);
    }

    #[test]
    fn test_input_event_type() {
        assert_eq!(InputEventType::from_u8(0), InputEventType::PointerDown);
        assert_eq!(InputEventType::from_u8(1), InputEventType::PointerMove);
        assert_eq!(InputEventType::from_u8(2), InputEventType::PointerUp);
        assert_eq!(InputEventType::from_u8(3), InputEventType::Wheel);
    }

    #[test]
    fn test_input_processor_creation() {
        let processor = InputProcessor::new();
        assert!(processor.ring_buffer.is_empty());
        assert!(processor.get_selection().is_none());
        assert!(!processor.is_dragging());
    }

    #[test]
    fn test_input_processor_push_and_drain() {
        let mut processor = InputProcessor::new();

        let event = RawInputEvent {
            timestamp: 1000.0,
            pointer_id: 0,
            x: 100.0,
            y: 200.0,
            pressure: 0.5,
            event_type: 0,
            buttons: 0,
            modifiers: 0,
            _reserved: 0,
        };

        processor.push_event(event);
        let events = processor.drain_events();

        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_input_event_position() {
        let event = RawInputEvent {
            timestamp: 0.0,
            pointer_id: 0,
            x: 100.0,
            y: 200.0,
            pressure: 0.0,
            event_type: 0,
            buttons: 0,
            modifiers: 0,
            _reserved: 0,
        };

        let pos = event.position();
        assert_eq!(pos.x, 100.0);
        assert_eq!(pos.y, 200.0);
    }

    #[test]
    fn test_ring_buffer_capacity() {
        let mut buffer = InputRingBuffer::new();

        // Ring buffer can hold EVENT_CAPACITY - 1 items
        // because head == tail means empty, not full
        for i in 0..(EVENT_CAPACITY - 1) {
            let event = RawInputEvent {
                timestamp: i as f64,
                pointer_id: 0,
                x: 0.0,
                y: 0.0,
                pressure: 0.0,
                event_type: 0,
                buttons: 0,
                modifiers: 0,
                _reserved: 0,
            };
            assert!(buffer.push_event(event), "Failed at index {}", i);
        }

        // Buffer should be full now
        let event = RawInputEvent {
            timestamp: 0.0,
            pointer_id: 0,
            x: 0.0,
            y: 0.0,
            pressure: 0.0,
            event_type: 0,
            buttons: 0,
            modifiers: 0,
            _reserved: 0,
        };
        assert!(!buffer.push_event(event));
    }

    #[test]
    fn test_ring_buffer_push_drain() {
        let mut buffer = InputRingBuffer::new();

        for i in 0..10 {
            buffer.push_event(RawInputEvent {
                timestamp: i as f64,
                pointer_id: 0,
                x: 0.0,
                y: 0.0,
                pressure: 0.0,
                event_type: 0,
                buttons: 0,
                modifiers: 0,
                _reserved: 0,
            });
        }

        assert_eq!(buffer.len(), 10);

        let events = buffer.drain();
        assert_eq!(events.len(), 10);
        assert!(buffer.is_empty());
    }
}
