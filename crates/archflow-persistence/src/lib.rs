// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Persistence Layer - Document Serialization & Deserialization
//
// Architecture Reference: EPIC-WEB-012
//
// This crate provides:
// - JSON serialization for human-readable documents
// - Binary format for optimized large document handling
// - Compression support (gzip)
// - SpatialHash pre-building for O(1) queries from load time
// - Logic Bricks wiring persistence
// - Version migration support
// ═══════════════════════════════════════════════════════════════════════════════

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]

use std::io::Read;
use std::path::Path;

pub mod document;
pub mod error;
pub mod format;
pub mod logic;
pub mod spatial;
pub mod store;

// Re-exports at crate root for convenience
pub use document::{
    ArchitectureData, Document, DocumentMeta, EntityData, Migration, PropValue, Schema,
    SchemaVersion, ShapeTypeDef, SpatialIndexData, StoreSnapshot, TextData,
};
pub use error::{PersistenceError, PersistenceResult};
pub use format::{CompressionOption, Format, SerializationFormat};
pub use logic::{LogicWiringSerializer, SerializableWiring};
pub use spatial::SpatialHashBuilder;
pub use store::EntityMapper;

/// Configuration options for persistence operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct PersistenceOptions {
    /// Serialization format to use
    pub format: SerializationFormat,
    /// Compression option
    pub compression: CompressionOption,
    /// Whether to include pre-built spatial index
    pub include_spatial_index: bool,
    /// Whether to include Logic Bricks wiring
    pub include_logic_wiring: bool,
    /// Whether to pretty-print JSON (formatting only)
    pub pretty_print: bool,
}

impl Default for PersistenceOptions {
    fn default() -> Self {
        Self {
            format: SerializationFormat::Json,
            compression: CompressionOption::None,
            include_spatial_index: true,
            include_logic_wiring: true,
            pretty_print: false,
        }
    }
}

impl PersistenceOptions {
    /// Create new options with sensible defaults
    #[must_use]
    pub const fn new() -> Self {
        Self {
            format: SerializationFormat::Json,
            compression: CompressionOption::None,
            include_spatial_index: true,
            include_logic_wiring: true,
            pretty_print: false,
        }
    }

    /// Set the serialization format
    #[must_use]
    pub const fn with_format(mut self, format: SerializationFormat) -> Self {
        self.format = format;
        self
    }

    /// Set the compression option
    #[must_use]
    pub const fn with_compression(mut self, compression: CompressionOption) -> Self {
        self.compression = compression;
        self
    }

    /// Enable/disable spatial index inclusion
    #[must_use]
    pub const fn with_spatial_index(mut self, include: bool) -> Self {
        self.include_spatial_index = include;
        self
    }

    /// Enable/disable Logic Bricks wiring inclusion
    #[must_use]
    pub const fn with_logic_wiring(mut self, include: bool) -> Self {
        self.include_logic_wiring = include;
        self
    }

    /// Enable/disable pretty printing for JSON
    #[must_use]
    pub const fn with_pretty_print(mut self, pretty: bool) -> Self {
        self.pretty_print = pretty;
        self
    }
}

/// Main entry point for persistence operations
///
/// # Example
///
/// ```no_run
/// use archflow_persistence::{PersistenceEngine, PersistenceOptions};
///
/// # fn document() -> archflow_persistence::Document { todo!() }
/// let engine = PersistenceEngine::new(PersistenceOptions::default());
/// let doc = document();
///
/// // Save to file
/// # let path = std::path::Path::new("document.archflow");
/// engine.save(path, &doc).unwrap();
/// ```
pub struct PersistenceEngine {
    options: PersistenceOptions,
}

impl PersistenceEngine {
    /// Create a new persistence engine with default options
    #[must_use]
    pub fn new() -> Self {
        Self::with_options(PersistenceOptions::default())
    }

    /// Create a new persistence engine with custom options
    #[must_use]
    pub const fn with_options(options: PersistenceOptions) -> Self {
        Self { options }
    }

    /// Get the current options
    #[must_use]
    pub const fn options(&self) -> &PersistenceOptions {
        &self.options
    }

    /// Export document to a byte vector using configured format
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails
    pub fn export_bytes(&self, document: &Document) -> PersistenceResult<Vec<u8>> {
        match self.options.format {
            SerializationFormat::Json => {
                let json_str = if self.options.pretty_print {
                    format::json::to_json_pretty(document)?
                } else {
                    format::json::to_json(document)?
                };
                let bytes = json_str.into_bytes();
                self.apply_compression(bytes)
            }
            SerializationFormat::Binary => {
                let bytes = format::binary::to_binary(document)?;
                self.apply_compression(bytes)
            }
        }
    }

    /// Import document from a byte vector (auto-detects format)
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails
    pub fn import_bytes(&self, data: &[u8]) -> PersistenceResult<Document> {
        let data = self.decompress(data)?;

        // Try to detect format from magic bytes or content
        if data.starts_with(b"{") || data.starts_with(b"[") {
            // Convert bytes to string for JSON parsing
            let json_str = std::str::from_utf8(&data)
                .map_err(|e| PersistenceError::Deserialization(e.to_string()))?;
            format::json::from_json(json_str)
        } else if data.len() >= 4 {
            // Check for binary format magic number
            format::binary::from_binary(&data)
        } else {
            Err(PersistenceError::InvalidFormat(
                "Cannot determine format from data".into(),
            ))
        }
    }

    /// Export document to JSON string (human-readable)
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails
    #[inline]
    pub fn export_json(&self, document: &Document) -> PersistenceResult<String> {
        if self.options.pretty_print {
            format::json::to_json_pretty(document)
        } else {
            format::json::to_json(document)
        }
    }

    /// Import document from JSON string
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails
    #[inline]
    pub fn import_json(&self, json: &str) -> PersistenceResult<Document> {
        format::json::from_json(json)
    }

    /// Export document to binary format (optimized)
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails
    #[inline]
    pub fn export_binary(&self, document: &Document) -> PersistenceResult<Vec<u8>> {
        format::binary::to_binary(document)
    }

    /// Import document from binary format
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails
    #[inline]
    pub fn import_binary(&self, data: &[u8]) -> PersistenceResult<Document> {
        format::binary::from_binary(data)
    }

    /// Apply compression if configured
    fn apply_compression(&self, data: Vec<u8>) -> PersistenceResult<Vec<u8>> {
        match self.options.compression {
            CompressionOption::None => Ok(data),
            CompressionOption::Gzip => {
                use flate2::Compression;
                use flate2::write::GzEncoder;

                let mut encoder = GzEncoder::new(std::vec::Vec::new(), Compression::default());
                encoder
                    .write_all(&data)
                    .map_err(|e| PersistenceError::Compression(e.to_string()))?;
                encoder
                    .finish()
                    .map_err(|e| PersistenceError::Compression(e.to_string()))
            }
        }
    }

    /// Decompress data if it's compressed
    fn decompress(&self, data: &[u8]) -> PersistenceResult<Vec<u8>> {
        // Check for gzip magic number
        if data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b {
            use flate2::read::GzDecoder;

            let mut decoder = GzDecoder::new(data);
            let mut decoded = Vec::new();
            decoder
                .read_to_end(&mut decoded)
                .map_err(|e| PersistenceError::Compression(e.to_string()))?;
            Ok(decoded)
        } else {
            Ok(data.to_vec())
        }
    }
}

impl Default for PersistenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WRITE TRAIT - For compression support
// ═══════════════════════════════════════════════════════════════════════════════

use std::io::Write;

/// Result of a save operation
#[derive(Debug, Clone)]
pub struct SaveResult {
    /// Number of entities saved
    pub entities_saved: u32,
    /// File size in bytes
    pub size_bytes: u64,
    /// Whether compression was applied
    pub compressed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_options_builder() {
        let opts = PersistenceOptions::new()
            .with_format(SerializationFormat::Binary)
            .with_compression(CompressionOption::Gzip)
            .with_pretty_print(true);

        assert_eq!(opts.format, SerializationFormat::Binary);
        assert_eq!(opts.compression, CompressionOption::Gzip);
        assert!(opts.pretty_print);
    }

    #[test]
    fn test_engine_creation() {
        let engine = PersistenceEngine::new();
        assert_eq!(engine.options().format, SerializationFormat::Json);
        assert_eq!(engine.options().compression, CompressionOption::None);
    }

    #[test]
    fn test_engine_with_options() {
        let opts = PersistenceOptions::new().with_format(SerializationFormat::Binary);
        let engine = PersistenceEngine::with_options(opts);
        assert_eq!(engine.options().format, SerializationFormat::Binary);
    }
}
