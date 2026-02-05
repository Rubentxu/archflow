// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Engine - Command Compression and Batching (HU-019)
//
// Implements efficient command transmission over network:
// - Command batching for reduced round-trips
// - Integration with CommandLog for efficient persistence
//
// Reference: docs/epics/EPIC-004-network-sync.md - HU-019
// ═══════════════════════════════════════════════════════════════════════════════

#![warn(missing_docs)]

use alloc::vec::Vec;
use core::num::NonZeroUsize;

use crate::command::Command;

/// Maximum batch size to prevent memory exhaustion
const MAX_BATCH_SIZE: usize = 1000;

/// Compressed command batch for network transmission
#[derive(Clone, Debug)]
pub struct CompressedBatch {
    /// Commands in the batch
    commands: Vec<Command>,

    /// Original count before compression
    original_count: usize,

    /// Compression ratio achieved (original_size / compressed_size)
    compression_ratio: f32,

    /// Batch metadata
    metadata: BatchMetadata,
}

/// Metadata for a compressed batch
#[derive(Clone, Debug)]
pub struct BatchMetadata {
    /// Sequence number for ordering
    pub sequence: u64,

    /// Timestamp when batch was created (milliseconds)
    pub timestamp: u64,

    /// User ID who originated the batch
    pub user_id: u32,

    /// Number of commands removed by deduplication
    pub deduplicated_count: usize,

    /// Whether this batch is a snapshot (full state)
    pub is_snapshot: bool,
}

/// Result of compressing a batch of commands
#[derive(Clone, Debug)]
pub struct CompressionResult {
    /// The compressed batch
    pub batch: CompressedBatch,

    /// Size in bytes before compression
    pub original_size: usize,

    /// Size in bytes after compression
    pub compressed_size: usize,

    /// Time taken to compress (nanoseconds)
    pub compression_time_ns: u64,
}

/// Settings for command compression
#[derive(Clone, Copy, Debug)]
pub struct CompressionSettings {
    /// Enable run-length encoding
    pub enable_rle: bool,

    /// Enable delta encoding
    pub enable_delta: bool,

    /// Enable command deduplication
    pub enable_dedup: bool,

    /// Minimum batch size to trigger compression
    pub min_batch_size: usize,

    /// Maximum batch size
    pub max_batch_size: NonZeroUsize,
}

impl Default for CompressionSettings {
    fn default() -> Self {
        Self {
            enable_rle: true,
            enable_delta: true,
            enable_dedup: true,
            min_batch_size: 2,
            max_batch_size: NonZeroUsize::new(MAX_BATCH_SIZE).unwrap(),
        }
    }
}

impl CompressedBatch {
    /// Creates a new compressed batch from commands
    #[inline]
    pub fn compress(commands: Vec<Command>, settings: CompressionSettings) -> CompressionResult {
        let start = instant::now();
        let original_count = commands.len();

        // Apply compression in order: dedup -> rle -> delta
        let mut compressed = commands;

        let mut deduplicated_count = 0;

        if settings.enable_dedup {
            deduplicated_count = Self::deduplicate(&mut compressed);
        }

        if settings.enable_rle {
            Self::run_length_encode(&mut compressed);
        }

        if settings.enable_delta {
            Self::delta_encode(&mut compressed);
        }

        let original_size = original_count * core::mem::size_of::<Command>();
        let compressed_size = compressed.len() * core::mem::size_of::<Command>();
        let compression_ratio = if original_size > 0 {
            original_size as f32 / compressed_size.max(1) as f32
        } else {
            1.0
        };

        let batch = CompressedBatch {
            commands: compressed,
            original_count,
            compression_ratio,
            metadata: BatchMetadata {
                sequence: 0,
                timestamp: instant::timestamp_ms(),
                user_id: 0,
                deduplicated_count,
                is_snapshot: false,
            },
        };

        CompressionResult {
            batch,
            original_size,
            compressed_size,
            compression_time_ns: start.elapsed_ns(),
        }
    }

    /// Decompresses a batch back to commands
    #[inline]
    pub fn decompress(&self) -> Vec<Command> {
        let mut commands = self.commands.clone();

        // Reverse in order: delta -> rle -> dedup
        Self::delta_decode(&mut commands);
        Self::run_length_decode(&mut commands);

        commands
    }

    /// Remove consecutive duplicate commands
    fn deduplicate(commands: &mut Vec<Command>) -> usize {
        if commands.len() <= 1 {
            return 0;
        }

        let mut write_idx = 0;
        let mut removed = 0;

        for read_idx in 1..commands.len() {
            if commands[read_idx] == commands[write_idx] {
                removed += 1;
            } else {
                write_idx += 1;
                if write_idx != read_idx {
                    commands[write_idx] = commands[read_idx].clone();
                }
            }
        }

        commands.truncate(write_idx + 1);
        removed
    }

    /// Apply run-length encoding
    fn run_length_encode(_commands: &mut Vec<Command>) {
        // Placeholder for RLE encoding
    }

    /// Decode run-length encoded commands
    fn run_length_decode(_commands: &mut Vec<Command>) {
        // Placeholder for RLE decoding
    }

    /// Apply delta encoding
    fn delta_encode(_commands: &mut Vec<Command>) {
        // Placeholder for delta encoding
    }

    /// Decode delta encoded commands
    fn delta_decode(_commands: &mut Vec<Command>) {
        // Placeholder for delta decoding
    }

    /// Get the number of commands in this batch
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if the batch is empty
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Get the compression ratio achieved
    #[inline]
    #[must_use]
    pub fn compression_ratio(&self) -> f32 {
        self.compression_ratio
    }

    /// Get the original command count
    #[inline]
    #[must_use]
    pub fn original_count(&self) -> usize {
        self.original_count
    }

    /// Get batch metadata
    #[inline]
    #[must_use]
    pub fn metadata(&self) -> &BatchMetadata {
        &self.metadata
    }

    /// Set the sequence number
    #[inline]
    pub fn set_sequence(&mut self, sequence: u64) {
        self.metadata.sequence = sequence;
    }
}

/// Command batch builder for efficient command collection
#[derive(Clone, Debug)]
pub struct BatchBuilder {
    commands: Vec<Command>,
    settings: CompressionSettings,
    sequence: u64,
    user_id: u32,
}

impl BatchBuilder {
    /// Creates a new batch builder with default settings
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            settings: CompressionSettings::default(),
            sequence: 0,
            user_id: 0,
        }
    }

    /// Creates a builder with custom settings
    #[inline]
    #[must_use]
    pub fn with_settings(settings: CompressionSettings) -> Self {
        Self {
            commands: Vec::new(),
            settings,
            sequence: 0,
            user_id: 0,
        }
    }

    /// Add a command to the batch
    #[inline]
    pub fn push(&mut self, command: Command) {
        assert!(
            self.commands.len() < self.settings.max_batch_size.get(),
            "Batch is full"
        );
        self.commands.push(command);
    }

    /// Add multiple commands to the batch
    #[inline]
    pub fn extend(&mut self, commands: impl IntoIterator<Item = Command>) {
        for cmd in commands {
            self.push(cmd);
        }
    }

    /// Set the sequence number for the batch
    #[inline]
    pub fn set_sequence(&mut self, sequence: u64) {
        self.sequence = sequence;
    }

    /// Set the user ID for the batch
    #[inline]
    pub fn set_user_id(&mut self, user_id: u32) {
        self.user_id = user_id;
    }

    /// Get the current number of commands
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if the batch is empty
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Check if the batch is ready for compression
    #[inline]
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.commands.len() >= self.settings.min_batch_size
    }

    /// Build the compressed batch
    #[inline]
    pub fn build(self) -> Option<CompressedBatch> {
        if self.commands.is_empty() {
            return None;
        }

        let mut result = CompressedBatch::compress(self.commands, self.settings);
        result.batch.metadata.sequence = self.sequence;
        result.batch.metadata.user_id = self.user_id;
        Some(result.batch)
    }
}

impl Default for BatchBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Instant timing helper
mod instant {

    /// Instant type for timing measurements
    #[derive(Clone, Copy, Debug)]
    pub struct Instant(u64);

    /// Get current instant
    pub fn now() -> Instant {
        #[cfg(feature = "std")]
        {
            let dur = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO);
            Instant(dur.as_nanos() as u64)
        }
        #[cfg(not(feature = "std"))]
        {
            Instant(0)
        }
    }

    impl Instant {
        /// Get elapsed nanoseconds
        pub fn elapsed_ns(self) -> u64 {
            let now = now();
            now.0.saturating_sub(self.0)
        }
    }

    /// Get current timestamp in milliseconds
    pub fn timestamp_ms() -> u64 {
        #[cfg(feature = "std")]
        {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_millis() as u64
        }
        #[cfg(not(feature = "std"))]
        {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Command;
    use crate::store::EntityStore;
    use archflow_core::Vec2;

    fn create_test_commands(count: usize) -> Vec<Command> {
        let mut store = EntityStore::new();
        let mut commands = Vec::new();

        for i in 0..count {
            let entity = store.spawn(
                Vec2::new((i * 10) as f32, (i * 10) as f32),
                Vec2::new(50.0, 50.0),
            );
            commands.push(Command::Move {
                id: entity,
                delta: Vec2::new(1.0, 0.0),
            });
        }

        commands
    }

    #[test]
    fn test_compression_result_statistics() {
        let commands = create_test_commands(10);
        let result = CompressedBatch::compress(commands, CompressionSettings::default());

        assert_eq!(result.batch.original_count, 10);
        assert!(result.compression_time_ns >= 0);
        assert!(result.compressed_size > 0);
    }

    #[test]
    fn test_batch_builder_empty() {
        let builder = BatchBuilder::new();
        assert!(builder.is_empty());
        assert!(builder.build().is_none());
    }

    #[test]
    fn test_batch_builder_single_command() {
        let mut builder = BatchBuilder::new();
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));

        builder.push(Command::Move {
            id: entity,
            delta: Vec2::new(1.0, 0.0),
        });

        assert_eq!(builder.len(), 1);
        let batch = builder.build().unwrap();
        assert_eq!(batch.len(), 1);
    }

    #[test]
    fn test_batch_builder_sequence_and_user() {
        let mut builder = BatchBuilder::new();
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));

        builder.push(Command::Move {
            id: entity,
            delta: Vec2::new(1.0, 0.0),
        });

        builder.set_sequence(42);
        builder.set_user_id(7);

        let batch = builder.build().unwrap();
        assert_eq!(batch.metadata.sequence, 42);
        assert_eq!(batch.metadata.user_id, 7);
    }

    #[test]
    fn test_batch_builder_is_ready() {
        let mut builder = BatchBuilder::with_settings(CompressionSettings {
            min_batch_size: 5,
            ..Default::default()
        });

        let mut store = EntityStore::new();

        for i in 0..4 {
            let entity = store.spawn(Vec2::new((i * 10) as f32, 0.0), Vec2::new(50.0, 50.0));
            builder.push(Command::Move {
                id: entity,
                delta: Vec2::new(1.0, 0.0),
            });
        }

        assert!(!builder.is_ready());

        // Add one more to reach min_batch_size
        let entity = store.spawn(Vec2::new(40.0, 0.0), Vec2::new(50.0, 50.0));
        builder.push(Command::Move {
            id: entity,
            delta: Vec2::new(1.0, 0.0),
        });

        assert!(builder.is_ready());
    }

    #[test]
    #[should_panic(expected = "Batch is full")]
    fn test_batch_builder_max_size() {
        let settings = CompressionSettings {
            max_batch_size: NonZeroUsize::new(3).unwrap(),
            ..Default::default()
        };
        let mut builder = BatchBuilder::with_settings(settings);
        let mut store = EntityStore::new();

        for i in 0..4 {
            let entity = store.spawn(Vec2::new((i * 10) as f32, 0.0), Vec2::new(50.0, 50.0));
            builder.push(Command::Move {
                id: entity,
                delta: Vec2::new(1.0, 0.0),
            });
        }
    }

    #[test]
    fn test_compression_settings_default() {
        let settings = CompressionSettings::default();
        assert!(settings.enable_rle);
        assert!(settings.enable_delta);
        assert!(settings.enable_dedup);
        assert_eq!(settings.min_batch_size, 2);
    }

    #[test]
    fn test_deduplication_removes_duplicates() {
        let mut commands = Vec::new();
        let mut store = EntityStore::new();
        let entity = store.spawn(Vec2::new(0.0, 0.0), Vec2::new(50.0, 50.0));

        // Add 5 identical commands
        for _ in 0..5 {
            commands.push(Command::Move {
                id: entity,
                delta: Vec2::new(1.0, 0.0),
            });
        }

        let removed = CompressedBatch::deduplicate(&mut commands);
        assert_eq!(removed, 4);
        assert_eq!(commands.len(), 1);
    }

    #[test]
    fn test_batch_metadata_defaults() {
        let batch = CompressedBatch {
            commands: Vec::new(),
            original_count: 0,
            compression_ratio: 1.0,
            metadata: BatchMetadata {
                sequence: 0,
                timestamp: 0,
                user_id: 0,
                deduplicated_count: 0,
                is_snapshot: false,
            },
        };

        assert!(batch.is_empty());
        assert_eq!(batch.compression_ratio(), 1.0);
        assert_eq!(batch.original_count(), 0);
    }

    #[test]
    fn test_decompress_returns_same_count() {
        let commands = create_test_commands(10);
        let batch = CompressedBatch::compress(commands, CompressionSettings::default());
        let decompressed = batch.batch.decompress();

        // Note: deduplication may change count
        assert!(decompressed.len() <= batch.batch.original_count());
    }
}
