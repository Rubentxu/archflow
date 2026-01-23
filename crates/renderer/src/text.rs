//! Text rendering module using cosmic-text
//!
//! Provides font handling, text shaping, and layout for 2D rendering.

use cosmic_text::{
    Metrics, SwashCache,
    fontdb::{Database, ID as FontId},
};
use std::fmt;

/// Text alignment options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

/// Font weight for text rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}

/// Font style variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyle {
    Normal,
    Italic,
}

/// Text wrapping mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextWrap {
    None,
    Word,
}

/// Text style configuration
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    pub font_family: String,
    pub font_size: f32,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub color: [u8; 4],
    pub alignment: TextAlignment,
    pub wrap: TextWrap,
    pub line_height: f32,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_family: "sans-serif".to_string(),
            font_size: 16.0,
            font_weight: FontWeight::Normal,
            font_style: FontStyle::Normal,
            color: [0, 0, 0, 255],
            alignment: TextAlignment::Left,
            wrap: TextWrap::Word,
            line_height: 1.2,
        }
    }
}

impl fmt::Display for TextStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TextStyle(font={}, size={}, weight={:?}, color={:?})",
            self.font_family, self.font_size, self.font_weight, self.color
        )
    }
}

/// Cached glyph for GPU rendering
#[derive(Debug, Clone)]
pub struct GlyphCacheEntry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub bytes: Vec<u8>,
}

/// Text buffer containing shaped text
#[derive(Debug, Clone)]
pub struct TextBuffer {
    /// The text content
    text: String,
    /// Text style applied
    style: TextStyle,
    /// Shaped lines
    lines: Vec<ShapedLine>,
    /// Minimum width
    min_width: f32,
    /// Actual width after layout
    width: f32,
    /// Total height after layout
    height: f32,
}

/// A single shaped line of text
#[derive(Debug, Clone)]
pub struct ShapedLine {
    text: String,
    width: f32,
    height: f32,
    glyph_count: usize,
}

/// Font loader and manager
#[derive(Debug)]
pub struct FontManager {
    font_db: Database,
    swash_cache: SwashCache,
    default_font_id: FontId,
}

impl FontManager {
    /// Creates a new font manager with system fonts
    pub fn new() -> Self {
        let mut font_db = Database::new();
        font_db.load_system_fonts();

        let swash_cache = SwashCache::new();

        let default_font_id = font_db
            .faces()
            .next()
            .map(|f| f.id)
            .unwrap_or(FontId::default());

        Self {
            font_db,
            swash_cache,
            default_font_id,
        }
    }

    /// Loads a font from memory
    pub fn load_font(&mut self, data: &[u8]) -> Option<FontId> {
        self.font_db.load_font_data(data.to_vec());
        self.font_db.faces().last().map(|f| f.id)
    }

    /// Returns the default font ID
    pub fn default_font(&self) -> FontId {
        self.default_font_id
    }

    /// Returns reference to the font database
    pub fn font_db(&self) -> &Database {
        &self.font_db
    }

    /// Returns mutable reference to the font database
    pub fn font_db_mut(&mut self) -> &mut Database {
        &mut self.font_db
    }

    /// Returns reference to the swash cache
    pub fn swash_cache(&self) -> &SwashCache {
        &self.swash_cache
    }

    /// Returns mutable reference to the swash cache
    pub fn swash_cache_mut(&mut self) -> &mut SwashCache {
        &mut self.swash_cache
    }
}

impl Default for FontManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Text renderer that handles layout and caching
#[derive(Debug)]
pub struct TextRenderer {
    font_manager: FontManager,
}

impl TextRenderer {
    /// Creates a new text renderer
    pub fn new() -> Self {
        Self {
            font_manager: FontManager::new(),
        }
    }

    /// Creates a text buffer from a string with default style
    pub fn create_text_buffer(&mut self, text: &str) -> TextBuffer {
        self.create_text_buffer_with_style(text, TextStyle::default())
    }

    /// Creates a text buffer with custom style
    pub fn create_text_buffer_with_style(&mut self, text: &str, style: TextStyle) -> TextBuffer {
        let font_id = self.find_font(&style.font_family);

        let metrics = Metrics::new(style.font_size, style.font_size * style.line_height);

        let lines: Vec<ShapedLine> = text
            .lines()
            .map(|line_text| {
                let width = self.measure_text_width(line_text, font_id, style.font_size);
                ShapedLine {
                    text: line_text.to_string(),
                    width,
                    height: style.font_size * style.line_height,
                    glyph_count: line_text.chars().count(),
                }
            })
            .collect();

        let min_width = lines.iter().map(|l| l.width).fold(0.0, f32::max);

        let total_height = lines.iter().map(|l| l.height).sum::<f32>();

        TextBuffer {
            text: text.to_string(),
            style,
            lines,
            min_width,
            width: min_width,
            height: total_height,
        }
    }

    /// Measures the width of text in pixels
    fn measure_text_width(&self, text: &str, _font_id: FontId, _font_size: f32) -> f32 {
        // Approximate width based on character count and average glyph width
        // A more accurate implementation would use the actual font metrics
        let avg_char_width = _font_size * 0.5;
        text.chars().map(|_| avg_char_width).sum()
    }

    /// Finds a font by family name
    fn find_font(&self, family: &str) -> FontId {
        // First try exact match
        for face in self.font_manager.font_db().faces() {
            if face.families.iter().any(|(f, _)| f == family) {
                return face.id;
            }
        }

        // Try sans-serif fallback
        if family != "sans-serif" {
            return self.find_font("sans-serif");
        }

        self.font_manager.default_font()
    }

    /// Returns the font manager
    pub fn font_manager(&self) -> &FontManager {
        &self.font_manager
    }

    /// Returns mutable font manager
    pub fn font_manager_mut(&mut self) -> &mut FontManager {
        &mut self.font_manager
    }

    /// Updates the text buffer with new text
    pub fn update_text_buffer(&mut self, buffer: &mut TextBuffer, new_text: &str) {
        *buffer = self.create_text_buffer_with_style(new_text, buffer.style.clone());
    }

    /// Updates the style of a text buffer
    pub fn update_text_style(&mut self, buffer: &mut TextBuffer, new_style: TextStyle) {
        *buffer = self.create_text_buffer_with_style(&buffer.text, new_style);
    }

    /// Returns the dimensions of a text buffer
    pub fn buffer_dimensions(&self, buffer: &TextBuffer) -> (f32, f32) {
        (buffer.width, buffer.height)
    }
}

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl TextBuffer {
    /// Returns the text content
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the text style
    pub fn style(&self) -> &TextStyle {
        &self.style
    }

    /// Returns the shaped lines
    pub fn lines(&self) -> &[ShapedLine] {
        &self.lines
    }

    /// Returns the width
    pub fn width(&self) -> f32 {
        self.width
    }

    /// Returns the height
    pub fn height(&self) -> f32 {
        self.height
    }

    /// Sets the layout width
    pub fn set_width(&mut self, width: f32) {
        self.width = width;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_renderer() -> TextRenderer {
        TextRenderer::new()
    }

    #[test]
    fn test_text_buffer_creation() {
        let mut renderer = create_test_renderer();
        let buffer = renderer.create_text_buffer("Hello, World!");

        assert_eq!(buffer.text(), "Hello, World!");
        assert!(buffer.width() > 0.0);
        assert!(buffer.height() > 0.0);
    }

    #[test]
    fn test_text_buffer_multiline() {
        let mut renderer = create_test_renderer();
        let buffer = renderer.create_text_buffer("Line 1\nLine 2\nLine 3");

        assert_eq!(buffer.lines().len(), 3);
        assert!(buffer.height() > buffer.lines()[0].height);
    }

    #[test]
    fn test_text_style_default() {
        let style = TextStyle::default();

        assert_eq!(style.font_family, "sans-serif");
        assert_eq!(style.font_size, 16.0);
        assert_eq!(style.font_weight, FontWeight::Normal);
        assert_eq!(style.font_style, FontStyle::Normal);
        assert_eq!(style.color, [0, 0, 0, 255]);
        assert_eq!(style.alignment, TextAlignment::Left);
        assert_eq!(style.line_height, 1.2);
    }

    #[test]
    fn test_text_style_custom() {
        let style = TextStyle {
            font_family: "serif".to_string(),
            font_size: 24.0,
            font_weight: FontWeight::Bold,
            font_style: FontStyle::Italic,
            color: [255, 0, 0, 255],
            alignment: TextAlignment::Center,
            wrap: TextWrap::Word,
            line_height: 1.5,
        };

        assert_eq!(style.font_family, "serif");
        assert_eq!(style.font_size, 24.0);
        assert_eq!(style.font_weight, FontWeight::Bold);
        assert_eq!(style.color, [255, 0, 0, 255]);
    }

    #[test]
    fn test_font_manager_creation() {
        let font_manager = FontManager::new();
        assert!(font_manager.font_db().faces().next().is_some());
    }

    #[test]
    fn test_text_renderer_default() {
        let renderer = TextRenderer::default();
        assert!(renderer.font_manager().font_db().faces().next().is_some());
    }

    #[test]
    fn test_empty_text_buffer() {
        let mut renderer = create_test_renderer();
        let buffer = renderer.create_text_buffer("");

        assert_eq!(buffer.text(), "");
        // Empty string creates 0 lines (lines() on empty string is empty)
        assert!(buffer.lines().is_empty() || buffer.lines().len() == 1);
    }

    #[test]
    fn test_text_style_display() {
        let style = TextStyle::default();
        let display = format!("{}", style);
        assert!(display.contains("TextStyle"));
        assert!(display.contains("sans-serif"));
    }

    #[test]
    fn test_text_alignment_variants() {
        assert_ne!(TextAlignment::Left, TextAlignment::Center);
        assert_ne!(TextAlignment::Center, TextAlignment::Right);
    }

    #[test]
    fn test_font_weight_variants() {
        assert_ne!(FontWeight::Thin, FontWeight::Bold);
        assert_ne!(FontWeight::Normal, FontWeight::Black);
    }

    #[test]
    fn test_update_text_buffer() {
        let mut renderer = create_test_renderer();
        let mut buffer = renderer.create_text_buffer("Original");
        renderer.update_text_buffer(&mut buffer, "Updated");

        assert_eq!(buffer.text(), "Updated");
    }

    #[test]
    fn test_buffer_dimensions() {
        let mut renderer = create_test_renderer();
        let buffer = renderer.create_text_buffer("Test");

        let (width, height) = renderer.buffer_dimensions(&buffer);
        assert_eq!(width, buffer.width());
        assert_eq!(height, buffer.height());
    }
}
