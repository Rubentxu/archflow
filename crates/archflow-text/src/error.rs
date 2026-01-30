//! Error types for text rendering operations.

use std::fmt;

/// Result type for text operations.
pub type TextResult<T> = Result<T, TextError>;

/// Errors that can occur during text rendering operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextError {
    /// Font data is invalid or corrupted.
    InvalidFontData,

    /// Glyph not found in font.
    GlyphNotFound(u32),

    /// Font loading failed.
    FontLoadFailed(String),

    /// SDF generation failed.
    SDFGenerationFailed(String),

    /// Atlas is full and cannot fit more glyphs.
    AtlasFull,

    /// Invalid UTF-8 sequence in text.
    InvalidUtf8,

    /// Shaping operation failed.
    ShapingFailed(String),

    /// Other error with message.
    Other(String),
}

impl fmt::Display for TextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFontData => write!(f, "Invalid font data"),
            Self::GlyphNotFound(id) => write!(f, "Glyph not found: {}", id),
            Self::FontLoadFailed(msg) => write!(f, "Font load failed: {}", msg),
            Self::SDFGenerationFailed(msg) => write!(f, "SDF generation failed: {}", msg),
            Self::AtlasFull => write!(f, "Atlas is full"),
            Self::InvalidUtf8 => write!(f, "Invalid UTF-8 sequence"),
            Self::ShapingFailed(msg) => write!(f, "Shaping failed: {}", msg),
            Self::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for TextError {}
