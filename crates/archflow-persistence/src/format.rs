// ═══════════════════════════════════════════════════════════════════════════════
// Format Definitions - JSON and Binary serialization formats
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]
#![allow(clippy::module_name_repetitions)]

pub mod binary;
pub mod json;

/// Serialization format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum SerializationFormat {
    /// JSON format (human-readable)
    #[default]
    Json,
    /// Binary format (optimized for size/speed)
    Binary,
}

/// Compression options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum CompressionOption {
    /// No compression
    #[default]
    None,
    /// Gzip compression
    Gzip,
}

/// Format auto-detection result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// JSON format detected
    Json,
    /// Binary format detected
    Binary,
    /// Gzip-compressed data (format unknown until decompressed)
    Gzip,
    /// Unknown format
    Unknown,
}

impl Format {
    /// Detect format from magic bytes or content inspection
    #[must_use]
    pub fn detect(data: &[u8]) -> Self {
        if data.len() < 2 {
            return Self::Unknown;
        }

        // Check for gzip magic number
        if data[0] == 0x1f && data[1] == 0x8b {
            return Self::Gzip;
        }

        // Check for JSON (starts with { or [)
        let first = data[0];
        if first == b'{' || first == b'[' {
            return Self::Json;
        }

        // Check for ArchFlow binary magic number (0xAF 0x01)
        if data.len() >= 4 && data[0] == 0xAF && data[1] == 0x01 {
            return Self::Binary;
        }

        Self::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detect_json() {
        let json = b"{\"version\":1}";
        assert_eq!(Format::detect(json), Format::Json);
    }

    #[test]
    fn test_format_detect_json_array() {
        let json = b"[1,2,3]";
        assert_eq!(Format::detect(json), Format::Json);
    }

    #[test]
    fn test_format_detect_gzip() {
        // Gzip magic number: 0x1f 0x8b
        let gzip = [0x1f, 0x8b, 0x08, 0x00];
        assert_eq!(Format::detect(&gzip), Format::Gzip);
    }

    #[test]
    fn test_format_detect_binary() {
        // ArchFlow binary magic number: 0xAF 0x01
        let binary = [0xAF, 0x01, 0x00, 0x01];
        assert_eq!(Format::detect(&binary), Format::Binary);
    }

    #[test]
    fn test_format_detect_unknown() {
        let unknown = b"XYZ";
        assert_eq!(Format::detect(unknown), Format::Unknown);
    }

    #[test]
    fn test_format_detect_empty() {
        let empty: [u8; 0] = [];
        assert_eq!(Format::detect(&empty), Format::Unknown);
    }

    #[test]
    fn test_serialization_format_default() {
        assert_eq!(SerializationFormat::default(), SerializationFormat::Json);
    }

    #[test]
    fn test_compression_option_default() {
        assert_eq!(CompressionOption::default(), CompressionOption::None);
    }
}
