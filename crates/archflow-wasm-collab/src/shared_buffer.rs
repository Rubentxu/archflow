//! Shared Buffer for Zero-Copy Rust → JavaScript Communication
//!
//! This module provides zero-copy shared memory communication between Rust and JavaScript
//! using SharedArrayBuffer with bytemuck for POD struct sharing.
//!
//! # Architecture
//!
//! ```text
//! Rust (WASM)                          JavaScript
//!   |                                      |
//!   |-- RenderAttribute[POD] ---------->   |-- Float32Array view
//!   |-- Update in-place (zero-copy)       |
//!   |-- Pointer stable for JS             |
//! ```

use bytemuck::{Pod, Zeroable};

/// Render attribute shared between Rust and JavaScript.
///
/// This struct is carefully designed to be:
/// 1. **POD (Plain Old Data)**: Can be safely transmuted to bytes
/// 2. **Aligned**: Proper alignment for GPU/WebGL consumption
/// 3. **Fixed size**: Predictable memory layout for array indexing
///
/// Layout: 24 bytes total
/// - id: 8 bytes (u64)
/// - x: 4 bytes (f32)
/// - y: 4 bytes (f32)
/// - color: 4 bytes (RGBA u8)
/// - _padding: 4 bytes (alignment)
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug, PartialEq)]
pub struct RenderAttribute {
    /// Unique record identifier
    pub id: u64,
    /// X position in world coordinates
    pub x: f32,
    /// Y position in world coordinates
    pub y: f32,
    /// RGBA color (0-255 per channel)
    pub color: [u8; 4],
    /// Padding for 4-byte alignment
    pub _padding: [u8; 4],
}

impl RenderAttribute {
    /// Creates a zeroed render attribute (useful as default)
    pub fn zeroed() -> Self {
        Self {
            id: 0,
            x: 0.0,
            y: 0.0,
            color: [0, 0, 0, 0],
            _padding: [0; 4],
        }
    }

    /// Creates a render attribute from position and color
    pub fn new(id: u64, x: f32, y: f32, color: [u8; 4]) -> Self {
        Self {
            id,
            x,
            y,
            color,
            _padding: [0; 4],
        }
    }
}

/// Buffer compartido para comunicación Rust → JavaScript.
///
/// Uses a pre-allocated vector of `RenderAttribute` that is updated in-place,
/// providing zero-copy semantics when viewed from JavaScript.
///
/// # Example
///
/// ```rust,ignore
/// use archflow_wasm_collab::SharedBuffer;
///
/// let mut buffer = SharedBuffer::new(1000);
/// // Update with visible records
/// buffer.update(&visible_ids, &record_store);
/// // JavaScript can access via Float32Array view
/// ```
pub struct SharedBuffer {
    /// Pre-allocated render attributes (POD for zero-copy)
    render_buffer: Vec<RenderAttribute>,
    /// Maximum number of elements this buffer can hold
    max_elements: usize,
    /// Current number of valid elements
    len: usize,
}

impl SharedBuffer {
    /// Creates a new shared buffer with capacity for `max_elements`.
    ///
    /// # Arguments
    ///
    /// * `max_elements` - Maximum number of render attributes to buffer
    ///
    /// # Panics
    ///
    /// Panics if `max_elements` is 0 or exceeds reasonable limits.
    pub fn new(max_elements: usize) -> Self {
        assert!(max_elements > 0, "max_elements must be greater than 0");
        assert!(
            max_elements <= 1_000_000,
            "max_elements exceeds reasonable limits"
        );

        Self {
            render_buffer: vec![RenderAttribute::zeroed(); max_elements],
            max_elements,
            len: 0,
        }
    }

    /// Updates the buffer with visible elements from the record store.
    ///
    /// This method updates the buffer in-place without allocations,
    /// providing O(n) complexity where n is the number of visible elements.
    ///
    /// # Arguments
    ///
    /// * `visible_ids` - IDs of records that are currently visible
    /// * `store` - Record store containing the record data
    ///
    /// # Notes
    ///
    /// Elements beyond `visible_ids.len()` are zeroed out.
    pub fn update(
        &mut self,
        visible_ids: &[u64],
        get_record: impl Fn(u64) -> Option<(f32, f32, [u8; 4])>,
    ) {
        // Update visible elements
        for (i, &id) in visible_ids.iter().enumerate().take(self.max_elements) {
            if let Some((x, y, color)) = get_record(id) {
                self.render_buffer[i] = RenderAttribute {
                    id,
                    x,
                    y,
                    color,
                    _padding: [0; 4],
                };
            } else {
                // Zero out non-existent records
                self.render_buffer[i] = RenderAttribute::zeroed();
            }
        }

        // Zero out remaining elements
        for i in visible_ids.len().min(self.max_elements)..self.max_elements {
            self.render_buffer[i] = RenderAttribute::zeroed();
        }

        self.len = visible_ids.len().min(self.max_elements);
    }

    /// Gets a const pointer to the buffer data for JavaScript access.
    ///
    /// This pointer is stable across updates - the buffer is never reallocated.
    #[inline]
    pub fn get_ptr(&self) -> *const RenderAttribute {
        self.render_buffer.as_ptr()
    }

    /// Gets the current length of valid data in the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the maximum capacity of the buffer.
    #[inline]
    pub fn max_elements(&self) -> usize {
        self.max_elements
    }

    /// Returns whether the buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Gets the total byte size of the buffer.
    #[inline]
    pub fn byte_size(&self) -> usize {
        self.render_buffer.len() * std::mem::size_of::<RenderAttribute>()
    }

    /// Gets the raw buffer slice (for internal use).
    #[inline]
    pub fn as_slice(&self) -> &[RenderAttribute] {
        &self.render_buffer
    }

    /// Gets mutable access to the buffer (internal use only).
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [RenderAttribute] {
        &mut self.render_buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_attribute_pod() {
        let attr = RenderAttribute {
            id: 42,
            x: 100.5,
            y: 200.3,
            color: [255, 128, 64, 255],
            _padding: [0; 4],
        };

        // Verify POD - can be transmuted to bytes
        let bytes = bytemuck::bytes_of(&attr);
        assert_eq!(bytes.len(), std::mem::size_of::<RenderAttribute>());
        assert_eq!(bytes.len(), 24); // 8 + 4 + 4 + 4 + 4 = 24 bytes
    }

    #[test]
    fn test_render_attribute_alignment() {
        // Verify proper alignment for f32 access
        let attr = RenderAttribute::new(1, 1.0, 2.0, [255, 255, 255, 255]);
        assert_eq!(std::mem::size_of::<RenderAttribute>() % 4, 0);
        assert_eq!(std::mem::align_of::<RenderAttribute>(), 8);
    }

    #[test]
    fn test_shared_buffer_creation() {
        let buffer = SharedBuffer::new(100);
        assert_eq!(buffer.max_elements(), 100);
        assert!(buffer.is_empty());
        assert_eq!(buffer.byte_size(), 100 * 24);
    }

    #[test]
    fn test_shared_buffer_update() {
        let mut buffer = SharedBuffer::new(10);

        // Mock record getter
        let get_record = |id: u64| -> Option<(f32, f32, [u8; 4])> {
            match id {
                1 => Some((100.0, 200.0, [255, 0, 0, 255])),
                2 => Some((150.0, 250.0, [0, 255, 0, 255])),
                _ => None,
            }
        };

        buffer.update(&[1, 2, 999], get_record);

        assert_eq!(buffer.len(), 3);

        let slice = buffer.as_slice();
        assert_eq!(slice[0].id, 1);
        assert!((slice[0].x - 100.0).abs() < 0.01);
        assert!((slice[0].y - 200.0).abs() < 0.01);
        assert_eq!(slice[0].color, [255, 0, 0, 255]);

        assert_eq!(slice[1].id, 2);
        assert_eq!(slice[2].id, 0); // Non-existent record zeroed
    }

    #[test]
    fn test_pointer_stability() {
        let mut buffer = SharedBuffer::new(100);
        let ptr1 = buffer.get_ptr();

        // Update multiple times
        let get_record = |_: u64| Some((0.0, 0.0, [0, 0, 0, 0]));
        for _ in 0..10 {
            buffer.update(&(0..50).collect::<Vec<_>>(), &get_record);
        }

        let ptr2 = buffer.get_ptr();
        assert_eq!(ptr1, ptr2, "Pointer must remain stable after updates");
    }

    #[test]
    fn test_shared_buffer_bounds_exceeded() {
        let mut buffer = SharedBuffer::new(3);
        let get_record = |_: u64| Some((1.0, 2.0, [255, 255, 255, 255]));

        // Update with more elements than capacity
        buffer.update(&(0..10).collect::<Vec<_>>(), get_record);

        assert_eq!(buffer.len(), 3); // Capped at max_elements

        // Only first 3 should be updated
        let slice = buffer.as_slice();
        assert_eq!(slice[0].id, 0);
        assert_eq!(slice[1].id, 1);
        assert_eq!(slice[2].id, 2);
    }

    #[test]
    fn test_render_attribute_zeroed() {
        let attr = RenderAttribute::zeroed();
        assert_eq!(attr.id, 0);
        assert_eq!(attr.x, 0.0);
        assert_eq!(attr.y, 0.0);
        assert_eq!(attr.color, [0, 0, 0, 0]);
    }

    #[test]
    fn test_render_attribute_new() {
        let attr = RenderAttribute::new(42, 10.5, 20.5, [255, 128, 64, 255]);
        assert_eq!(attr.id, 42);
        assert!((attr.x - 10.5).abs() < 0.01);
        assert!((attr.y - 20.5).abs() < 0.01);
        assert_eq!(attr.color, [255, 128, 64, 255]);
    }
}
