// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Web - Lock-Free Input Ring Buffer via SharedArrayBuffer
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 7
//
// Lock-free input bridge between JavaScript (producer) and WASM (consumer):
// - SharedArrayBuffer for zero-copy data transfer
// - Atomic operations for lock-free synchronization
// - Event coalescing for reduced overhead
// - Support for multi-touch and keyboard input
// ═══════════════════════════════════════════════════════════════════════════════

#![no_std]
#![warn(missing_docs)]
#![warn(clippy::all)]

extern crate alloc;

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, Ordering};

/// Maximum number of simultaneous pointers (for multi-touch)
pub const MAX_POINTERS: usize = 8;

/// Event buffer capacity (events per frame)
pub const EVENT_CAPACITY: usize = 128;

/// Size of each RawInputEvent in bytes (for SharedArrayBuffer alignment)
pub const EVENT_SIZE: usize = 32;

/// Input event types
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputEventType {
    /// Pointer down (mouse button pressed, touch start)
    Down = 0,
    /// Pointer move (mouse move, touch move)
    Move = 1,
    /// Pointer up (mouse button released, touch end)
    Up = 2,
    /// Wheel scroll (mouse wheel, trackpad scroll)
    Wheel = 3,
    /// Key down (keyboard key pressed)
    KeyDown = 4,
    /// Key up (keyboard key released)
    KeyUp = 5,
}

/// Mouse button bitmask
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Buttons(pub u8);

impl Buttons {
    /// Left mouse button
    pub const LEFT: u8 = 0x01;
    /// Right mouse button
    pub const RIGHT: u8 = 0x02;
    /// Middle mouse button
    pub const MIDDLE: u8 = 0x04;

    pub fn new() -> Self {
        Self(0)
    }

    pub fn is_left_pressed(self) -> bool {
        self.0 & Self::LEFT != 0
    }

    pub fn is_right_pressed(self) -> bool {
        self.0 & Self::RIGHT != 0
    }

    pub fn is_middle_pressed(self) -> bool {
        self.0 & Self::MIDDLE != 0
    }
}

impl Default for Buttons {
    fn default() -> Self {
        Self::new()
    }
}

/// Keyboard modifier bitmask
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Modifiers(pub u8);

impl Modifiers {
    /// Shift key
    pub const SHIFT: u8 = 0x01;
    /// Control key
    pub const CTRL: u8 = 0x02;
    /// Alt key
    pub const ALT: u8 = 0x04;
    /// Meta key (Command on Mac, Windows key on Windows)
    pub const META: u8 = 0x08;

    pub fn new() -> Self {
        Self(0)
    }

    pub fn is_shift_pressed(self) -> bool {
        self.0 & Self::SHIFT != 0
    }

    pub fn is_ctrl_pressed(self) -> bool {
        self.0 & Self::CTRL != 0
    }

    pub fn is_alt_pressed(self) -> bool {
        self.0 & Self::ALT != 0
    }
}

impl Default for Modifiers {
    fn default() -> Self {
        Self::new()
    }
}

/// Raw input event for lock-free passing between JS and WASM
///
/// Layout is carefully designed to be 32 bytes (8-byte aligned)
/// for efficient SharedArrayBuffer access.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RawInputEvent {
    /// Timestamp from performance.now() for precise delta time
    pub timestamp: u64,

    /// Pointer identifier (for multi-touch support)
    pub pointer_id: u32,

    /// X coordinate in screen pixels
    pub x: f32,

    /// Y coordinate in screen pixels
    pub y: f32,

    /// Pointer pressure (0.0 to 1.0, for touch/pen support)
    pub pressure: f32,

    /// Event type (0=Down, 1=Move, 2=Up, 3=Wheel, 4=KeyDown)
    pub event_type: u8,

    /// Mouse button bitmask (Left=1, Right=2, Middle=4)
    pub buttons: u8,

    /// Keyboard modifier bitmask (Shift=1, Ctrl=2, Alt=4)
    pub modifiers: u8,

    /// Padding for 8-byte alignment
    _padding: u8,
}

impl RawInputEvent {
    /// Create a new raw input event
    pub fn new(
        timestamp: u64,
        pointer_id: u32,
        x: f32,
        y: f32,
        event_type: InputEventType,
        buttons: Buttons,
        modifiers: Modifiers,
    ) -> Self {
        Self {
            timestamp,
            pointer_id,
            x,
            y,
            pressure: 0.0,
            event_type: event_type as u8,
            buttons: buttons.0,
            modifiers: modifiers.0,
            _padding: 0,
        }
    }

    /// Get the event type value
    pub fn event_type_value(&self) -> u8 {
        self.event_type
    }
}

impl Default for RawInputEvent {
    fn default() -> Self {
        Self {
            timestamp: 0,
            pointer_id: 0,
            x: 0.0,
            y: 0.0,
            pressure: 0.0,
            event_type: InputEventType::Move as u8,
            buttons: 0,
            modifiers: 0,
            _padding: 0,
        }
    }
}

/// Lock-free ring buffer for input events
///
/// This buffer is shared between JavaScript (producer) and WASM (consumer)
/// via SharedArrayBuffer, enabling zero-copy data transfer.
///
/// Memory layout (for SharedArrayBuffer):
/// ```text
/// +--------+--------+--------------------------+
/// | head   | tail   | data[EVENT_CAPACITY]    |
/// | u32    | u32    | RawInputEvent x 128     |
/// +--------+--------+--------------------------+
/// 0       4        8                         4104
/// ```
///
/// The buffer is lock-free using atomic operations on head and tail pointers.
pub struct InputRingBuffer {
    /// Head pointer - JS writes here (producer index)
    head: AtomicU32,

    /// Tail pointer - WASM reads here (consumer index)
    tail: AtomicU32,

    /// Event data buffer
    data: [RawInputEvent; EVENT_CAPACITY],
}

unsafe impl Send for InputRingBuffer {}
unsafe impl Sync for InputRingBuffer {}

impl InputRingBuffer {
    /// Create a new empty ring buffer
    pub fn new() -> Self {
        Self {
            head: AtomicU32::new(0),
            tail: AtomicU32::new(0),
            data: [RawInputEvent::default(); EVENT_CAPACITY],
        }
    }

    /// Push an event to the buffer (called from JS via wasm-bindgen)
    ///
    /// Returns false if buffer is full (backpressure should be applied)
    ///
    /// # Arguments
    /// * `event` - The event to push
    ///
    /// # Returns
    /// * `true` - Event was added successfully
    /// * `false` - Buffer is full, event was dropped
    pub fn push_event(&mut self, event: RawInputEvent) -> bool {
        let head = self.head.load(Ordering::Acquire) as usize;
        let tail = self.tail.load(Ordering::Acquire) as usize;
        let next = (head + 1) % EVENT_CAPACITY;

        // Check if buffer is full
        if next == tail {
            return false; // Apply backpressure
        }

        self.data[head] = event;
        self.head.store(next as u32, Ordering::Release);
        true
    }

    /// Drain all events from the buffer
    ///
    /// Called each frame from WASM to process all pending input events.
    /// Returns a Vec of all events and updates the tail pointer.
    pub fn drain(&mut self) -> Vec<RawInputEvent> {
        let tail = self.tail.load(Ordering::Acquire) as usize;
        let head = self.head.load(Ordering::Acquire) as usize;

        let mut result = Vec::new();
        let mut current = tail;

        while current != head {
            result.push(self.data[current]);
            current = (current + 1) % EVENT_CAPACITY;
        }

        self.tail.store(current as u32, Ordering::Release);
        result
    }

    /// Get the current number of pending events
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Acquire) as usize;
        let tail = self.tail.load(Ordering::Acquire) as usize;

        if head >= tail {
            head - tail
        } else {
            EVENT_CAPACITY - tail + head
        }
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all events (reset to empty state)
    pub fn clear(&mut self) {
        let head = self.head.load(Ordering::Acquire);
        self.tail.store(head, Ordering::Release);
    }
}

impl Default for InputRingBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// High-level input processor that consumes the ring buffer
///
/// This struct processes raw input events and converts them into
/// high-level actions (camera pan, zoom, entity selection, etc.).
pub struct InputProcessor {
    /// The underlying ring buffer
    buffer: InputRingBuffer,

    /// Current mouse position (in screen coordinates)
    mouse_pos: archflow_core::Vec2,

    /// Current mouse button state
    mouse_buttons: Buttons,

    /// Current keyboard modifiers
    modifiers: Modifiers,

    /// Last scroll position (for delta calculation)
    last_scroll_y: f32,
}

impl InputProcessor {
    /// Create a new input processor
    pub fn new() -> Self {
        Self {
            buffer: InputRingBuffer::new(),
            mouse_pos: archflow_core::Vec2::ZERO,
            mouse_buttons: Buttons::new(),
            modifiers: Modifiers::new(),
            last_scroll_y: 0.0,
        }
    }

    /// Get a reference to the underlying ring buffer
    pub fn buffer(&mut self) -> &mut InputRingBuffer {
        &mut self.buffer
    }

    /// Process all pending events
    ///
    /// This should be called each frame to consume input events
    /// and update the internal state.
    pub fn process_events(&mut self) -> Vec<RawInputEvent> {
        self.buffer.drain()
    }

    /// Get the current mouse position
    pub fn mouse_pos(&self) -> archflow_core::Vec2 {
        self.mouse_pos
    }

    /// Get the current mouse button state
    pub fn mouse_buttons(&self) -> Buttons {
        self.mouse_buttons
    }

    /// Get the current keyboard modifiers
    pub fn modifiers(&self) -> Modifiers {
        self.modifiers
    }

    /// Check if left mouse button is pressed
    pub fn is_left_mouse_pressed(&self) -> bool {
        self.mouse_buttons.is_left_pressed()
    }
}

impl Default for InputProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_size() {
        assert_eq!(core::mem::size_of::<RawInputEvent>(), 32);
    }

    #[test]
    fn test_ring_buffer_push_drain() {
        let mut buffer = InputRingBuffer::new();

        let event = RawInputEvent::new(
            1000,
            0,
            100.0,
            200.0,
            InputEventType::Move,
            Buttons::new(),
            Modifiers::new(),
        );

        assert!(buffer.is_empty());
        assert!(buffer.push_event(event));
        assert_eq!(buffer.len(), 1);
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_ring_buffer_drain() {
        let mut buffer = InputRingBuffer::new();

        buffer.push_event(RawInputEvent::new(
            1000,
            0,
            10.0,
            20.0,
            InputEventType::Down,
            Buttons::new(),
            Modifiers::new(),
        ));
        buffer.push_event(RawInputEvent::new(
            2000,
            0,
            30.0,
            40.0,
            InputEventType::Move,
            Buttons::new(),
            Modifiers::new(),
        ));

        let events = buffer.drain();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].x, 10.0);
        assert_eq!(events[1].x, 30.0);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_ring_buffer_capacity() {
        let mut buffer = InputRingBuffer::new();

        // Fill the buffer one event at a time
        let mut pushed = 0;
        for i in 0..EVENT_CAPACITY {
            if buffer.push_event(RawInputEvent::new(
                i as u64,
                0,
                0.0,
                0.0,
                InputEventType::Move,
                Buttons::new(),
                Modifiers::new(),
            )) {
                pushed += 1;
            } else {
                // Should not fail before reaching capacity
                break;
            }
        }

        // Should have pushed all events
        assert!(
            pushed >= EVENT_CAPACITY - 1,
            "Only pushed {} of {} events",
            pushed,
            EVENT_CAPACITY
        );

        // Buffer should be full or nearly full
        assert!(buffer.len() >= EVENT_CAPACITY - 1);

        // Next event should fail (buffer full)
        assert!(!buffer.push_event(RawInputEvent::new(
            999,
            0,
            0.0,
            0.0,
            InputEventType::Move,
            Buttons::new(),
            Modifiers::new(),
        )));
    }

    #[test]
    fn test_ring_buffer_wraparound() {
        let mut buffer = InputRingBuffer::new();

        // Fill and drain to test wraparound
        for _ in 0..2 {
            // Push some events
            for i in 0..10 {
                buffer.push_event(RawInputEvent::new(
                    i as u64,
                    0,
                    i as f32,
                    0.0,
                    InputEventType::Move,
                    Buttons::new(),
                    Modifiers::new(),
                ));
            }

            // Drain them
            let count = buffer.drain().len();
            assert_eq!(count, 10);
        }
    }

    #[test]
    fn test_buttons() {
        let buttons = Buttons(0x05); // Left + Middle
        assert!(buttons.is_left_pressed());
        assert!(buttons.is_middle_pressed());
        assert!(!buttons.is_right_pressed());
    }

    #[test]
    fn test_modifiers() {
        let modifiers = Modifiers(0x03); // Shift + Ctrl
        assert!(modifiers.is_shift_pressed());
        assert!(modifiers.is_ctrl_pressed());
        assert!(!modifiers.is_alt_pressed());
    }
}
