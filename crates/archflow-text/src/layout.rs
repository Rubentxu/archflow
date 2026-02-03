// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Text - Text Layout System
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 12.2
//
// Integrates cosmic-text for text shaping and layout computation.
// Manages the FontSystem and provides layout computation.
// ═══════════════════════════════════════════════════════════════════════════════

extern crate alloc;

use archflow_core::Vec2;

use crate::cache::{FlatGlyphRun, GlyphRunCache};

/// Font identifier
pub type FontId = u32;

/// Default font size in pixels
pub const DEFAULT_FONT_SIZE: f32 = 14.0;

/// Text layout system
///
/// Integrates with cosmic-text for text shaping and layout computation.
pub struct TextLayoutSystem {
    /// Glyph run cache
    cache: GlyphRunCache,

    /// Default font ID
    default_font: FontId,

    /// Default font size
    default_size: f32,
}

impl TextLayoutSystem {
    /// Create a new text layout system
    #[inline(always)]
    pub const fn new(default_font: FontId, default_size: f32) -> Self {
        Self {
            cache: GlyphRunCache::new(),
            default_font,
            default_size,
        }
    }

    /// Create with defaults
    #[inline(always)]
    pub const fn with_defaults() -> Self {
        Self {
            cache: GlyphRunCache::new(),
            default_font: 0,
            default_size: DEFAULT_FONT_SIZE,
        }
    }

    /// Get or compute a glyph run for the given text
    ///
    /// # Arguments
    /// * `text` - The text to shape
    /// * `font_size` - Font size in pixels (uses default if None)
    ///
    /// # Returns
    /// Cloned glyph run from cache
    pub fn layout_text(&mut self, text: &str, font_size: Option<f32>) -> FlatGlyphRun {
        let size = font_size.unwrap_or(self.default_size);
        let font_id = self.default_font;

        self.compute_and_cache(text, font_id, size)
    }

    /// Get or compute a glyph run with specific font
    ///
    /// # Arguments
    /// * `text` - The text to shape
    /// * `font_id` - Font identifier
    /// * `font_size` - Font size in pixels
    ///
    /// # Returns
    /// Cloned glyph run from cache
    pub fn layout_text_with_font(
        &mut self,
        text: &str,
        font_id: FontId,
        font_size: f32,
    ) -> FlatGlyphRun {
        self.compute_and_cache(text, font_id, font_size)
    }

    /// Compute text layout and cache the result
    fn compute_and_cache(&mut self, text: &str, _font_id: FontId, font_size: f32) -> FlatGlyphRun {
        // Simple monospace layout (placeholder for cosmic-text integration)
        let advance = font_size * 0.6; // Approximate character width

        self.cache.get_or_compute(text, font_size, |text, _size| {
            let mut run = FlatGlyphRun::new();
            let mut x = 0.0;

            for _c in text.chars() {
                // UV rect would be fetched from MTSDF atlas based on the glyph
                let uv_rect = [0.0, 0.0, 0.1, 0.1]; // Placeholder
                run.add_glyph(Vec2::new(x, 0.0), uv_rect);
                x += advance;
            }

            run
        })
    }

    /// Clear the glyph cache
    #[inline(always)]
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics
    ///
    /// # Returns
    /// Number of cached entries
    #[inline(always)]
    pub fn cache_stats(&self) -> usize {
        self.cache.stats()
    }

    /// Set the default font
    #[inline(always)]
    pub fn set_default_font(&mut self, font_id: FontId) {
        self.default_font = font_id;
    }

    /// Set the default font size
    #[inline(always)]
    pub fn set_default_size(&mut self, size: f32) {
        self.default_size = size;
    }

    /// Get the default font ID
    #[inline(always)]
    pub const fn default_font(&self) -> FontId {
        self.default_font
    }

    /// Get the default font size
    #[inline(always)]
    pub const fn default_size(&self) -> f32 {
        self.default_size
    }
}

impl Default for TextLayoutSystem {
    fn default() -> Self {
        Self::with_defaults()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_creation() {
        let system = TextLayoutSystem::new(0, 16.0);

        assert_eq!(system.default_font(), 0);
        assert_eq!(system.default_size(), 16.0);
    }

    #[test]
    fn test_layout_default() {
        let system = TextLayoutSystem::default();

        assert_eq!(system.default_font(), 0);
        assert_eq!(system.default_size(), DEFAULT_FONT_SIZE);
    }

    #[test]
    fn test_layout_text() {
        let mut system = TextLayoutSystem::new(0, 14.0);
        let text = "Hello";

        let result = system.layout_text(text, None);

        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_layout_text_with_size() {
        let mut system = TextLayoutSystem::new(0, 14.0);
        let text = "Test";

        let result = system.layout_text(text, Some(20.0));

        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_layout_empty_text() {
        let mut system = TextLayoutSystem::new(0, 14.0);

        let result = system.layout_text("", None);

        assert_eq!(result.len(), 0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_cache_hit() {
        let mut system = TextLayoutSystem::new(0, 14.0);
        let text = "Cache";

        let _result1 = system.layout_text(text, None);
        let entries = system.cache_stats();
        assert!(entries > 0);

        // Second access should hit cache
        let result2 = system.layout_text(text, None);
        assert_eq!(result2.len(), 5);
    }

    #[test]
    fn test_clear_cache() {
        let mut system = TextLayoutSystem::new(0, 14.0);

        system.layout_text("Test", None);
        let entries = system.cache_stats();
        assert!(entries > 0);

        system.clear_cache();
        let entries = system.cache_stats();
        assert_eq!(entries, 0);
    }

    #[test]
    fn test_set_default_font() {
        let mut system = TextLayoutSystem::new(0, 14.0);

        system.set_default_font(5);
        assert_eq!(system.default_font(), 5);
    }

    #[test]
    fn test_set_default_size() {
        let mut system = TextLayoutSystem::new(0, 14.0);

        system.set_default_size(24.0);
        assert_eq!(system.default_size(), 24.0);
    }

    #[test]
    fn test_different_sizes() {
        let mut system = TextLayoutSystem::new(0, 14.0);

        let result1 = system.layout_text("A", Some(12.0));
        let result2 = system.layout_text("A", Some(24.0));

        // Different sizes = different cache entries
        assert_eq!(result1.len(), 1);
        assert_eq!(result2.len(), 1);
        assert_eq!(system.cache_stats(), 2);
    }

    #[test]
    fn test_cache_stats() {
        let mut system = TextLayoutSystem::new(0, 14.0);

        let entries = system.cache_stats();
        assert_eq!(entries, 0);

        system.layout_text("Test", None);
        system.layout_text("Hello", None);

        let entries = system.cache_stats();
        assert_eq!(entries, 2);
    }

    #[test]
    fn test_unicode_text() {
        let mut system = TextLayoutSystem::new(0, 14.0);

        let result = system.layout_text("Hello World", None);

        // Simple ASCII text test
        assert_eq!(result.len(), 11); // "Hello World" = 11 characters
    }

    #[test]
    fn test_multiline_text() {
        let mut system = TextLayoutSystem::new(0, 14.0);

        let result = system.layout_text("Line1\nLine2\nLine3", None);

        // Count each character including newlines: 5 + 1 + 5 + 1 + 5 = 17
        assert_eq!(result.len(), 17);
    }
}
