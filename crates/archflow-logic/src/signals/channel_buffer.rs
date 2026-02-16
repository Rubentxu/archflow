// ═══════════════════════════════════════════════════════════════════════════════════════
// Signal Channel Buffer - FixedBitSet Implementation
//
// EPIC-LOGIC-003: Optimización de Canales
// Uses FixedBitSet for efficient double-buffered signal channel management
// ═══════════════════════════════════════════════════════════════════════════════════════

use fixedbitset::FixedBitSet;

/// Double-buffered signal channel using FixedBitSet for efficient bit operations.
///
/// This struct provides a high-performance signal channel buffer
/// that maintains two buffers (current and next) for concurrent read/write operations.
/// The swap() method atomically promotes the next buffer to current.
///
#[derive(Debug, Clone)]
pub struct SignalChannelBuffer {
    /// Current buffer for reading
    current: FixedBitSet,
    /// Next buffer for writing
    next: FixedBitSet,
    /// Maximum number of channels
    max_channels: usize,
}

impl SignalChannelBuffer {
    /// Creates a new SignalChannelBuffer with the specified maximum number of channels.
    #[inline(always)]
    #[must_use]
    pub fn new(max_channels: usize) -> Self {
        Self {
            current: FixedBitSet::with_capacity(max_channels),
            next: FixedBitSet::with_capacity(max_channels),
            max_channels,
        }
    }

    /// Writes a value to the specified channel in both buffers.
    /// This allows immediate reading while maintaining double-buffering semantics.
    #[inline(always)]
    pub fn write(&mut self, channel: u32, value: bool) {
        assert!(
            (channel as usize) < self.max_channels,
            "Channel index {} out of bounds (max: {})",
            channel,
            self.max_channels
        );
        // Write to both buffers for immediate reading (backward compatibility)
        self.current.set(channel as usize, value);
        self.next.set(channel as usize, value);
    }

    /// Swaps the current and next buffers.
    #[inline(always)]
    pub fn swap(&mut self) {
        core::mem::swap(&mut self.current, &mut self.next);
        self.next.clear();
    }

    /// Reads a value from the current buffer.
    #[inline(always)]
    #[must_use]
    pub fn read(&self, channel: u32) -> bool {
        assert!(
            (channel as usize) < self.max_channels,
            "Channel index {} out of bounds (max: {})",
            channel,
            self.max_channels
        );
        self.current[channel as usize]
    }

    /// Returns the maximum number of channels.
    #[inline(always)]
    #[must_use]
    pub fn max_channels(&self) -> usize {
        self.max_channels
    }

    /// Returns the number of active (true) channels in the current buffer.
    #[inline(always)]
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.current.count_ones(..self.max_channels)
    }

    /// Returns the number of inactive (false) channels in the current buffer.
    #[inline(always)]
    #[must_use]
    pub fn inactive_count(&self) -> usize {
        self.max_channels - self.current.count_ones(..self.max_channels)
    }

    /// Clears both buffers.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.current.clear();
        self.next.clear();
    }

    /// Returns a reference to the current buffer for advanced operations.
    #[inline(always)]
    #[must_use]
    pub fn current_buffer(&self) -> &FixedBitSet {
        &self.current
    }

    /// Returns a mutable reference to the next buffer for advanced operations.
    #[inline(always)]
    #[must_use]
    pub fn next_buffer(&mut self) -> &mut FixedBitSet {
        &mut self.next
    }
}

impl Default for SignalChannelBuffer {
    fn default() -> Self {
        Self::new(64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let buffer = SignalChannelBuffer::new(128);
        assert_eq!(buffer.max_channels(), 128);
        assert!(!buffer.read(0));
    }

    #[test]
    fn test_write_and_read() {
        let mut buffer = SignalChannelBuffer::new(8);
        buffer.write(0, true);
        buffer.write(1, true);
        buffer.write(2, false);

        assert!(buffer.read(0));
        assert!(buffer.read(1));
        assert!(!buffer.read(2));
    }

    #[test]
    fn test_swap() {
        let mut buffer = SignalChannelBuffer::new(4);
        // Write to current (immediately readable)
        buffer.write(0, true);
        assert!(buffer.read(0));
        
        // After swap, current gets next buffer's values
        // Since we wrote to both, value persists
        buffer.swap();
        assert!(buffer.read(0));
        
        // Clear and verify
        buffer.clear();
        assert!(!buffer.read(0));
    }

    #[test]
    fn test_swap_preserves_next_values() {
        let mut buffer = SignalChannelBuffer::new(4);
        buffer.write(0, true);
        buffer.write(1, true);
        buffer.swap();
        assert!(buffer.read(0));
        assert!(buffer.read(1));
        buffer.write(2, true);
        buffer.swap();
        assert!(!buffer.read(0));
        assert!(!buffer.read(1));
        assert!(buffer.read(2));
    }

    #[test]
    fn test_active_count() {
        let mut buffer = SignalChannelBuffer::new(8);
        assert_eq!(buffer.active_count(), 0);
        buffer.write(0, true);
        buffer.write(2, true);
        buffer.write(4, true);
        assert_eq!(buffer.active_count(), 3);
    }

    #[test]
    fn test_inactive_count() {
        let mut buffer = SignalChannelBuffer::new(4);
        assert_eq!(buffer.inactive_count(), 4);
        buffer.write(0, true);
        assert_eq!(buffer.inactive_count(), 3);
    }

    #[test]
    fn test_clear() {
        let mut buffer = SignalChannelBuffer::new(4);
        buffer.write(0, true);
        buffer.write(1, true);
        buffer.clear();
        assert!(!buffer.read(0));
        assert!(!buffer.read(1));
    }

    #[test]
    fn test_default() {
        let buffer = SignalChannelBuffer::default();
        assert_eq!(buffer.max_channels(), 64);
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn test_write_panics_on_invalid_channel() {
        let mut buffer = SignalChannelBuffer::new(4);
        buffer.write(4, true);
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn test_read_panics_on_invalid_channel() {
        let buffer = SignalChannelBuffer::new(4);
        buffer.read(4);
    }

    #[test]
    fn test_multiple_swaps() {
        let mut buffer = SignalChannelBuffer::new(4);
        buffer.write(0, true);
        buffer.swap();
        assert!(buffer.read(0));
        buffer.write(1, true);
        buffer.swap();
        assert!(!buffer.read(0));
        assert!(buffer.read(1));
        buffer.write(2, true);
        buffer.swap();
        assert!(!buffer.read(0));
        assert!(!buffer.read(1));
        assert!(buffer.read(2));
    }

    #[test]
    fn test_write_false_clears_channel() {
        let mut buffer = SignalChannelBuffer::new(4);
        buffer.write(0, true);
        assert!(buffer.read(0));
        buffer.write(0, false);
        buffer.swap();
        assert!(!buffer.read(0));
    }

    #[test]
    fn test_buffer_references() {
        let mut buffer = SignalChannelBuffer::new(4);
        buffer.write(0, true);
        let current = buffer.current_buffer();
        assert!(current[0]);
        let next = buffer.next_buffer();
        next.set(1, true);
        assert!(buffer.next_buffer()[1]);
    }
}
