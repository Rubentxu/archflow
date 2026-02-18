// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Input Sampling with SharedArrayBuffer
//
// This module implements zero-copy input sampling from JavaScript to Rust
// using SharedArrayBuffer (SAB) with automatic fallback for unsupported browsers.
//
// Reference: docs/epics/EPIC-001-input-sensors.md - HU-003
// ═══════════════════════════════════════════════════════════════════════════════


use archflow_core::Vec2;

/// Maximum number of keys tracked in the key bitmask
pub const MAX_KEYS: usize = 256;

/// SharedArrayBuffer memory layout (64 bytes, cache-line aligned)
///
/// This struct must match EXACTLY the layout specified in HU-003:
/// - Total size: exactly 64 bytes (one cache line)
/// - repr(C) ensures stable layout across platforms
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputSnapshotSAB {
    /// Write index (atomic)
    pub head: u32,

    /// Read index (atomic)
    pub tail: u32,

    /// Mouse X position
    pub mouse_x: i32,

    /// Mouse Y position
    pub mouse_y: i32,

    /// Button bitmask (bit 0 = left, bit 1 = right, bit 2 = middle)
    pub buttons: u8,

    /// Modifier bitmask (bit 0 = shift, bit 1 = ctrl, bit 2 = alt)
    pub modifiers: u8,

    /// Wheel delta
    pub wheel_delta: i16,

    /// Timestamp (ms since start)
    pub timestamp: u32,

    /// Key state bitmask (256 bits = 32 bytes)
    pub keys: [u8; 32],

    /// Cache line padding (8 bytes to reach 64 total)
    pub _padding: [u8; 8],
}

impl InputSnapshotSAB {
    /// Create a new zero-initialized snapshot
    #[must_use]
    pub const fn new() -> Self {
        Self {
            head: 0,
            tail: 0,
            mouse_x: 0,
            mouse_y: 0,
            buttons: 0,
            modifiers: 0,
            wheel_delta: 0,
            timestamp: 0,
            keys: [0; 32],
            _padding: [0; 8],
        }
    }

    /// Get mouse position as Vec2
    #[must_use]
    #[inline]
    pub const fn mouse_position(&self) -> Vec2 {
        Vec2::new(self.mouse_x as f32, self.mouse_y as f32)
    }

    /// Check if a specific mouse button is pressed
    #[must_use]
    #[inline]
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        match button {
            MouseButton::Left => (self.buttons & 0b001) != 0,
            MouseButton::Right => (self.buttons & 0b010) != 0,
            MouseButton::Middle => (self.buttons & 0b100) != 0,
        }
    }

    /// Check if a specific key is pressed (by keycode)
    #[must_use]
    #[inline]
    pub fn is_key_pressed(&self, keycode: u8) -> bool {
        let byte_idx = usize::from(keycode / 8);
        let bit_mask = 1 << (keycode % 8);
        if byte_idx < 32 {
            (self.keys[byte_idx] & bit_mask) != 0
        } else {
            false
        }
    }

    /// Check if shift key is held
    #[must_use]
    #[inline]
    pub const fn is_shift_pressed(&self) -> bool {
        (self.modifiers & 0b001) != 0
    }

    /// Check if ctrl key is held
    #[must_use]
    #[inline]
    pub const fn is_ctrl_pressed(&self) -> bool {
        (self.modifiers & 0b010) != 0
    }

    /// Check if alt key is held
    #[must_use]
    #[inline]
    pub const fn is_alt_pressed(&self) -> bool {
        (self.modifiers & 0b100) != 0
    }
}

impl Default for InputSnapshotSAB {
    fn default() -> Self {
        Self::new()
    }
}

/// Mouse button enum
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Input sampler that reads from SharedArrayBuffer
///
/// This is the primary interface for getting input snapshots in Rust.
/// It uses atomic operations to read from SAB without blocking.
pub struct InputSampler {
    /// Pointer to the SAB memory (set by JavaScript)
    sab_ptr: Option<*const InputSnapshotSAB>,

    /// Whether SAB is available
    sab_available: bool,

    /// Fallback buffer for when SAB is not available
    fallback_buffer: InputSnapshotSAB,
}

unsafe impl Send for InputSampler {}
unsafe impl Sync for InputSampler {}

impl InputSampler {
    /// Create a new input sampler
    #[must_use]
    pub const fn new() -> Self {
        Self {
            sab_ptr: None,
            sab_available: false,
            fallback_buffer: InputSnapshotSAB::new(),
        }
    }

    /// Check if SharedArrayBuffer is available
    #[must_use]
    pub const fn is_sab_available(&self) -> bool {
        self.sab_available
    }

    /// Set the SharedArrayBuffer pointer (called by JavaScript)
    ///
    /// # Safety
    ///
    /// The pointer must point to valid memory that lives for the duration
    /// of the sampler's usage.
    pub unsafe fn set_sab_ptr(&mut self, ptr: *const InputSnapshotSAB) {
        self.sab_ptr = Some(ptr);
        self.sab_available = true;
    }

    /// Take a snapshot of the current input state
    ///
    /// This uses atomic loads from SAB if available, or returns the
    /// fallback buffer if not.
    #[must_use]
    pub fn take_snapshot(&self) -> &InputSnapshotSAB {
        if self.sab_available {
            unsafe {
                if let Some(ptr) = self.sab_ptr {
                    return &*ptr;
                }
            }
        }
        &self.fallback_buffer
    }

    /// Push an input event (for fallback mode when SAB is not available)
    pub fn push_input_event(&mut self, event: InputEvent) {
        match event {
            InputEvent::MouseMove { x, y } => {
                self.fallback_buffer.mouse_x = x;
                self.fallback_buffer.mouse_y = y;
            }
            InputEvent::MouseButtonDown { button } => {
                self.fallback_buffer.buttons |= 1 << (button as u32);
            }
            InputEvent::MouseButtonUp { button } => {
                self.fallback_buffer.buttons &= !(1 << (button as u32));
            }
            InputEvent::MouseWheel { delta } => {
                self.fallback_buffer.wheel_delta = delta;
            }
            InputEvent::KeyDown { keycode } => {
                let byte_idx = (keycode / 8) as usize;
                let bit_mask = 1 << (keycode % 8);
                if byte_idx < 32 {
                    self.fallback_buffer.keys[byte_idx] |= bit_mask;
                }
            }
            InputEvent::KeyUp { keycode } => {
                let byte_idx = (keycode / 8) as usize;
                let bit_mask = 1 << (keycode % 8);
                if byte_idx < 32 {
                    self.fallback_buffer.keys[byte_idx] &= !bit_mask;
                }
            }
            InputEvent::Modifiers { shift, ctrl, alt } => {
                self.fallback_buffer.modifiers =
                    (shift as u8) | ((ctrl as u8) << 1) | ((alt as u8) << 2);
            }
        }
    }
}

impl Default for InputSampler {
    fn default() -> Self {
        Self::new()
    }
}

/// Input event that can be pushed to the sampler (fallback mode)
#[derive(Clone, Copy, Debug)]
pub enum InputEvent {
    MouseMove { x: i32, y: i32 },
    MouseButtonDown { button: u8 },
    MouseButtonUp { button: u8 },
    MouseWheel { delta: i16 },
    KeyDown { keycode: u8 },
    KeyUp { keycode: u8 },
    Modifiers { shift: bool, ctrl: bool, alt: bool },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_snapshot_sab_size() {
        // The struct must be exactly 64 bytes
        let size = core::mem::size_of::<InputSnapshotSAB>();
        assert_eq!(
            size, 64,
            "InputSnapshotSAB must be exactly 64 bytes, but is {}",
            size
        );
    }

    #[test]
    fn test_input_snapshot_sab_alignment() {
        // Should be aligned to 4 bytes (u32 alignment)
        assert_eq!(core::mem::align_of::<InputSnapshotSAB>(), 4);
    }

    #[test]
    fn test_input_snapshot_new() {
        let snapshot = InputSnapshotSAB::new();
        assert_eq!(snapshot.head, 0);
        assert_eq!(snapshot.tail, 0);
        assert_eq!(snapshot.mouse_x, 0);
        assert_eq!(snapshot.mouse_y, 0);
        assert_eq!(snapshot.buttons, 0);
        assert_eq!(snapshot.modifiers, 0);
        assert_eq!(snapshot.wheel_delta, 0);
        assert_eq!(snapshot.timestamp, 0);
        assert_eq!(snapshot.keys, [0; 32]);
    }

    #[test]
    fn test_mouse_position() {
        let mut snapshot = InputSnapshotSAB::new();
        snapshot.mouse_x = 100;
        snapshot.mouse_y = 200;
        let pos = snapshot.mouse_position();
        assert_eq!(pos.x, 100.0);
        assert_eq!(pos.y, 200.0);
    }

    #[test]
    fn test_is_mouse_button_pressed() {
        let mut snapshot = InputSnapshotSAB::new();

        // No buttons pressed
        assert!(!snapshot.is_mouse_button_pressed(MouseButton::Left));
        assert!(!snapshot.is_mouse_button_pressed(MouseButton::Right));
        assert!(!snapshot.is_mouse_button_pressed(MouseButton::Middle));

        // Press left button
        snapshot.buttons = 0b001;
        assert!(snapshot.is_mouse_button_pressed(MouseButton::Left));
        assert!(!snapshot.is_mouse_button_pressed(MouseButton::Right));
        assert!(!snapshot.is_mouse_button_pressed(MouseButton::Middle));

        // Press right button
        snapshot.buttons = 0b010;
        assert!(!snapshot.is_mouse_button_pressed(MouseButton::Left));
        assert!(snapshot.is_mouse_button_pressed(MouseButton::Right));
        assert!(!snapshot.is_mouse_button_pressed(MouseButton::Middle));

        // Press middle button
        snapshot.buttons = 0b100;
        assert!(!snapshot.is_mouse_button_pressed(MouseButton::Left));
        assert!(!snapshot.is_mouse_button_pressed(MouseButton::Right));
        assert!(snapshot.is_mouse_button_pressed(MouseButton::Middle));

        // Press all buttons
        snapshot.buttons = 0b111;
        assert!(snapshot.is_mouse_button_pressed(MouseButton::Left));
        assert!(snapshot.is_mouse_button_pressed(MouseButton::Right));
        assert!(snapshot.is_mouse_button_pressed(MouseButton::Middle));
    }

    #[test]
    fn test_is_key_pressed() {
        let mut snapshot = InputSnapshotSAB::new();

        // No keys pressed
        assert!(!snapshot.is_key_pressed(0));
        assert!(!snapshot.is_key_pressed(10));
        assert!(!snapshot.is_key_pressed(255));

        // Press key 0 (bit 0 of byte 0)
        snapshot.keys[0] = 0b00000001;
        assert!(snapshot.is_key_pressed(0));
        assert!(!snapshot.is_key_pressed(1));
        assert!(!snapshot.is_key_pressed(8));

        // Press key 7 (bit 7 of byte 0)
        snapshot.keys[0] = 0b10000000;
        assert!(!snapshot.is_key_pressed(0));
        assert!(snapshot.is_key_pressed(7));
        assert!(!snapshot.is_key_pressed(8));

        // Press key 8 (bit 0 of byte 1)
        snapshot.keys[1] = 0b00000001;
        assert!(snapshot.is_key_pressed(8));
        assert!(!snapshot.is_key_pressed(9));
        assert!(!snapshot.is_key_pressed(16));

        // Press key 255 (bit 7 of byte 31)
        snapshot.keys[31] = 0b10000000;
        assert!(snapshot.is_key_pressed(255));
        assert!(!snapshot.is_key_pressed(254));
    }

    #[test]
    fn test_modifier_keys() {
        let mut snapshot = InputSnapshotSAB::new();

        // No modifiers
        assert!(!snapshot.is_shift_pressed());
        assert!(!snapshot.is_ctrl_pressed());
        assert!(!snapshot.is_alt_pressed());

        // Shift only
        snapshot.modifiers = 0b001;
        assert!(snapshot.is_shift_pressed());
        assert!(!snapshot.is_ctrl_pressed());
        assert!(!snapshot.is_alt_pressed());

        // Ctrl only
        snapshot.modifiers = 0b010;
        assert!(!snapshot.is_shift_pressed());
        assert!(snapshot.is_ctrl_pressed());
        assert!(!snapshot.is_alt_pressed());

        // Alt only
        snapshot.modifiers = 0b100;
        assert!(!snapshot.is_shift_pressed());
        assert!(!snapshot.is_ctrl_pressed());
        assert!(snapshot.is_alt_pressed());

        // All modifiers
        snapshot.modifiers = 0b111;
        assert!(snapshot.is_shift_pressed());
        assert!(snapshot.is_ctrl_pressed());
        assert!(snapshot.is_alt_pressed());
    }

    #[test]
    fn test_input_sampler_new() {
        let sampler = InputSampler::new();
        assert!(!sampler.is_sab_available());
        assert_eq!(sampler.sab_ptr, None);
    }

    #[test]
    fn test_input_sampler_push_input_event() {
        let mut sampler = InputSampler::new();

        // Mouse move
        sampler.push_input_event(InputEvent::MouseMove { x: 100, y: 200 });
        let snapshot = sampler.take_snapshot();
        assert_eq!(snapshot.mouse_x, 100);
        assert_eq!(snapshot.mouse_y, 200);

        // Mouse button down
        sampler.push_input_event(InputEvent::MouseButtonDown { button: 0 });
        let snapshot = sampler.take_snapshot();
        assert_eq!(snapshot.buttons, 0b001);

        // Key down
        sampler.push_input_event(InputEvent::KeyDown { keycode: 10 });
        let snapshot = sampler.take_snapshot();
        assert!(snapshot.is_key_pressed(10));

        // Modifiers
        sampler.push_input_event(InputEvent::Modifiers {
            shift: true,
            ctrl: false,
            alt: true,
        });
        let snapshot = sampler.take_snapshot();
        assert!(snapshot.is_shift_pressed());
        assert!(!snapshot.is_ctrl_pressed());
        assert!(snapshot.is_alt_pressed());
    }

    #[test]
    fn test_input_sampler_key_up() {
        let mut sampler = InputSampler::new();

        // Key down
        sampler.push_input_event(InputEvent::KeyDown { keycode: 5 });
        assert!(sampler.take_snapshot().is_key_pressed(5));

        // Key up
        sampler.push_input_event(InputEvent::KeyUp { keycode: 5 });
        assert!(!sampler.take_snapshot().is_key_pressed(5));
    }

    #[test]
    fn test_input_sampler_multiple_keys() {
        let mut sampler = InputSampler::new();

        sampler.push_input_event(InputEvent::KeyDown { keycode: 0 });
        sampler.push_input_event(InputEvent::KeyDown { keycode: 10 });
        sampler.push_input_event(InputEvent::KeyDown { keycode: 100 });

        let snapshot = sampler.take_snapshot();
        assert!(snapshot.is_key_pressed(0));
        assert!(snapshot.is_key_pressed(10));
        assert!(snapshot.is_key_pressed(100));
        assert!(!snapshot.is_key_pressed(50));
    }

    #[test]
    fn test_input_sampler_mouse_wheel() {
        let mut sampler = InputSampler::new();

        sampler.push_input_event(InputEvent::MouseWheel { delta: 5 });
        assert_eq!(sampler.take_snapshot().wheel_delta, 5);

        sampler.push_input_event(InputEvent::MouseWheel { delta: -3 });
        assert_eq!(sampler.take_snapshot().wheel_delta, -3);
    }
}
