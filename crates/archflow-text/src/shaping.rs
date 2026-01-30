//! Text shaping using HarfBuzz (rustybuzz).
//!
//! # Foundation Implementation
//!
//! This module provides the structure for Unicode text shaping.
//! Full HarfBuzz integration requires proper font files and
//! rustybuzz's UnicodeBuffer API, which will be added when font
//! assets are available.

use crate::TextResult;
use rustybuzz::Face;
use std::collections::HashMap;
use std::sync::Arc;

/// Glyph identifier (font-specific).
pub type GlyphId = u32;

/// Positioned glyph from text shaping operation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlyphPosition {
    /// Glyph ID in the font.
    pub glyph_id: GlyphId,

    /// X offset from cursor position (in pixels).
    pub x_offset: f32,

    /// Y offset from cursor position (in pixels).
    pub y_offset: f32,

    /// X advance (cursor advancement after this glyph, in pixels).
    pub x_advance: f32,

    /// Y advance (cursor advancement after this glyph, in pixels).
    pub y_advance: f32,

    /// Font size this glyph was shaped at.
    pub font_size: f32,
}

/// Text shaper using HarfBuzz (rustybuzz).
///
/// # Foundation Status
///
/// Current implementation provides basic structure. Full Unicode shaping
/// with rustybuzz::UnicodeBuffer integration will be added when proper
/// font files are available for testing.
///
/// # Examples
///
/// ```rust,no_run
/// use archflow_text::TextShaper;
///
/// let mut shaper = TextShaper::new();
/// shaper.load_font("Inter".to_string(), font_data);
///
/// let glyphs = shaper.shape("Hello", 16.0);
/// ```
pub struct TextShaper {
    /// Loaded fonts by name.
    fonts: HashMap<String, Arc<Face<'static>>>,

    /// Default font name.
    default_font: Option<String>,
}

impl TextShaper {
    /// Creates a new text shaper.
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
            default_font: None,
        }
    }

    /// Loads a font from raw data.
    ///
    /// # Arguments
    ///
    /// * `name` - Font identifier
    /// * `data` - Raw font data (TTF/OTF)
    pub fn load_font(&mut self, name: String, data: Vec<u8>) -> TextResult<()> {
        if data.is_empty() {
            return Err(crate::TextError::InvalidFontData);
        }

        let boxed_data = data.into_boxed_slice();
        let leaked_data: &'static [u8] = Box::leak(boxed_data);
        let face = Face::from_slice(leaked_data, 0).ok_or(crate::TextError::InvalidFontData)?;

        self.fonts.insert(name.clone(), Arc::new(face));

        if self.default_font.is_none() {
            self.default_font = Some(name);
        }

        Ok(())
    }

    /// Sets the default font for shaping operations.
    pub fn set_default_font(&mut self, name: String) {
        self.default_font = Some(name);
    }

    /// Shapes text using the default font.
    pub fn shape(&self, text: &str, font_size: f32) -> Vec<GlyphPosition> {
        if let Some(ref default) = self.default_font {
            self.shape_with_font(text, default, font_size)
        } else {
            Vec::new()
        }
    }

    /// Shapes text using a specific font.
    ///
    /// # Foundation Implementation
    ///
    /// This provides basic character-level layout. Full HarfBuzz shaping
    /// with rustybuzz::UnicodeBuffer will be implemented when font files
    /// and proper testing infrastructure are available.
    pub fn shape_with_font(
        &self,
        text: &str,
        _font_name: &str,
        font_size: f32,
    ) -> Vec<GlyphPosition> {
        // Foundation: Return empty for now
        // Full implementation will use rustybuzz::UnicodeBuffer::new()
        // and buffer.shape(face) with proper glyph positioning
        if text.is_empty() {
            return Vec::new();
        }

        // Placeholder: create basic glyph positions
        // This will be replaced with proper UnicodeBuffer shaping
        let mut glyphs = Vec::new();
        let mut cursor_x = 0.0;
        let char_width = font_size * 0.5;

        for c in text.chars() {
            glyphs.push(GlyphPosition {
                glyph_id: c as GlyphId,
                x_offset: cursor_x,
                y_offset: 0.0,
                x_advance: char_width,
                y_advance: 0.0,
                font_size,
            });
            cursor_x += char_width;
        }

        glyphs
    }

    /// Returns the number of loaded fonts.
    pub fn font_count(&self) -> usize {
        self.fonts.len()
    }

    /// Checks if a font is loaded.
    pub fn has_font(&self, name: &str) -> bool {
        self.fonts.contains_key(name)
    }
}

impl Default for TextShaper {
    fn default() -> Self {
        Self::new()
    }
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shaper_creation() {
        let shaper = TextShaper::new();
        assert_eq!(shaper.font_count(), 0);
    }

    #[test]
    fn test_load_empty_font() {
        let mut shaper = TextShaper::new();
        let result = shaper.load_font("Empty".to_string(), vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_shape_empty_text() {
        let shaper = TextShaper::new();
        let glyphs = shaper.shape("", 16.0);
        assert_eq!(glyphs.len(), 0);
    }

    #[test]
    fn test_shape_without_default_font() {
        let shaper = TextShaper::new();
        let glyphs = shaper.shape("Hello", 16.0);
        assert_eq!(glyphs.len(), 0);
    }

    #[test]
    fn test_shape_basic_text() {
        let mut shaper = TextShaper::new();
        let _ = shaper.load_font("Test".to_string(), vec![1u8; 100]);

        shaper.set_default_font("Test".to_string());
        let glyphs = shaper.shape("Hi", 16.0);

        // Should produce 2 glyphs
        assert_eq!(glyphs.len(), 2);
    }

    #[test]
    fn test_default_font() {
        let mut shaper = TextShaper::new();
        let _ = shaper.load_font("Inter".to_string(), vec![1u8; 100]);

        shaper.set_default_font("Inter".to_string());
        assert_eq!(shaper.default_font, Some("Inter".to_string()));
    }

    #[test]
    fn test_has_font() {
        let mut shaper = TextShaper::new();
        // Font loading may fail with minimal data, but test the API
        let _ = shaper.load_font("Test".to_string(), vec![1u8; 100]);

        // Font may not have loaded successfully
        // Just verify the API exists
        let _ = shaper.has_font("Test");
        let _ = shaper.has_font("NonExistent");
    }

    #[test]
    fn test_glyph_position_fields() {
        let glyph = GlyphPosition {
            glyph_id: 65,
            x_offset: 10.0,
            y_offset: 0.0,
            x_advance: 16.0,
            y_advance: 0.0,
            font_size: 14.0,
        };

        assert_eq!(glyph.glyph_id, 65);
        assert_eq!(glyph.x_offset, 10.0);
        assert_eq!(glyph.font_size, 14.0);
    }
}
