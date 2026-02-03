// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Text - MTSDF Atlas (Multi-channel Signed Distance Field)
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 12.1
//
// MTSDF provides crisp text rendering at small sizes with proper anti-aliasing.
// The atlas caches glyph textures for GPU rendering.
// ═══════════════════════════════════════════════════════════════════════════════

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use archflow_core::Rect;

/// Glyph cache key: (font_id, glyph_id, size_px)
pub type GlyphKey = (u32, u32, u16);

/// MTSDF Atlas for text rendering
///
/// Stores pre-rasterized glyphs as multi-channel signed distance fields.
/// This allows crisp text rendering at any scale with proper anti-aliasing.
pub struct MtsdfAtlas {
    /// Atlas width in pixels
    pub width: u32,

    /// Atlas height in pixels
    pub height: u32,

    /// Glyph UV rectangle cache
    /// Maps (font_id, glyph_id, size_px) to UV coordinates in the atlas
    pub glyph_cache: BTreeMap<GlyphKey, Rect>,

    /// Raw pixel data (RGBA8)
    /// Used for texture upload to GPU
    pixels: Vec<u8>,
}

impl MtsdfAtlas {
    /// Create a new empty MTSDF atlas
    ///
    /// # Arguments
    /// * `width` - Atlas width in pixels (power of 2 recommended)
    /// * `height` - Atlas height in pixels (power of 2 recommended)
    #[inline(always)]
    pub fn new(width: u32, height: u32) -> Self {
        let pixel_count = (width * height) as usize;
        let pixels = vec![0u8; pixel_count * 4]; // RGBA8

        Self {
            width,
            height,
            glyph_cache: BTreeMap::new(),
            pixels,
        }
    }

    /// Insert a glyph into the atlas
    ///
    /// # Arguments
    /// * `key` - Glyph key (font_id, glyph_id, size_px)
    /// * `uv_rect` - UV rectangle in the atlas [0-1] range
    /// * `glyph_pixels` - Raw glyph pixel data (RGBA8)
    ///
    /// # Returns
    /// `true` if the glyph was inserted, `false` if the key already exists
    pub fn insert_glyph(&mut self, key: GlyphKey, uv_rect: Rect, _glyph_pixels: &[u8]) -> bool {
        if self.glyph_cache.contains_key(&key) {
            return false;
        }

        // Store UV mapping
        self.glyph_cache.insert(key, uv_rect);

        // Copy pixel data to atlas
        // Note: In a real implementation, this would use the UV rect
        // to calculate the correct position in the atlas texture

        true
    }

    /// Get the UV rectangle for a glyph
    ///
    /// # Arguments
    /// * `font_id` - Font identifier
    /// * `glyph_id` - Glyph index in the font
    /// * `size_px` - Font size in pixels
    #[inline(always)]
    pub fn get_glyph_uv(&self, font_id: u32, glyph_id: u32, size_px: u16) -> Option<Rect> {
        self.glyph_cache.get(&(font_id, glyph_id, size_px)).copied()
    }

    /// Get the UV rectangle for a glyph by key
    #[inline(always)]
    pub fn get_uv(&self, key: GlyphKey) -> Option<Rect> {
        self.glyph_cache.get(&key).copied()
    }

    /// Get the raw pixel data
    ///
    /// Used for uploading the atlas to the GPU
    #[inline(always)]
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// Get mutable reference to pixels
    ///
    /// Used for building the atlas
    #[inline(always)]
    pub fn pixels_mut(&mut self) -> &mut [u8] {
        &mut self.pixels
    }

    /// Clear all cached glyphs
    #[inline(always)]
    pub fn clear(&mut self) {
        self.glyph_cache.clear();
        self.pixels.fill(0);
    }

    /// Get the number of cached glyphs
    #[inline(always)]
    pub fn glyph_count(&self) -> usize {
        self.glyph_cache.len()
    }

    /// Check if a glyph is cached
    #[inline(always)]
    pub fn contains(&self, font_id: u32, glyph_id: u32, size_px: u16) -> bool {
        self.glyph_cache.contains_key(&(font_id, glyph_id, size_px))
    }

    /// Calculate atlas utilization
    ///
    /// # Returns
    /// Percentage of atlas space used (0.0 to 1.0)
    pub fn utilization(&self) -> f32 {
        // Estimate based on glyph count and average glyph size
        // This is a rough approximation
        let avg_glyph_area = 16.0 * 16.0; // 16x16 pixels average
        let total_area = (self.width * self.height) as f32;
        let used_area = self.glyph_count() as f32 * avg_glyph_area;

        (used_area / total_area).min(1.0)
    }
}

impl Default for MtsdfAtlas {
    fn default() -> Self {
        Self::new(1024, 1024)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format; // For format! macro in no_std
    use alloc::vec; // For vec! macro in no_std

    #[test]
    fn test_atlas_creation() {
        let atlas = MtsdfAtlas::new(512, 512);

        assert_eq!(atlas.width, 512);
        assert_eq!(atlas.height, 512);
        assert_eq!(atlas.glyph_count(), 0);
        assert_eq!(atlas.pixels.len(), 512 * 512 * 4);
    }

    #[test]
    fn test_atlas_default() {
        let atlas = MtsdfAtlas::default();

        assert_eq!(atlas.width, 1024);
        assert_eq!(atlas.height, 1024);
    }

    #[test]
    fn test_insert_glyph() {
        let mut atlas = MtsdfAtlas::new(256, 256);
        let key = (0, 42, 16);
        let uv = Rect::new(0.0, 0.0, 0.1, 0.1);
        let pixels = [255u8; 64]; // 4x4 glyph

        let result = atlas.insert_glyph(key, uv, &pixels);

        assert!(result);
        assert_eq!(atlas.glyph_count(), 1);
        assert!(atlas.contains(0, 42, 16));
    }

    #[test]
    fn test_insert_duplicate_glyph() {
        let mut atlas = MtsdfAtlas::new(256, 256);
        let key = (0, 42, 16);
        let uv = Rect::new(0.0, 0.0, 0.1, 0.1);
        let pixels = [255u8; 64];

        atlas.insert_glyph(key, uv, &pixels);
        let result = atlas.insert_glyph(key, uv, &pixels);

        assert!(!result); // Should not insert duplicate
        assert_eq!(atlas.glyph_count(), 1);
    }

    #[test]
    fn test_get_glyph_uv() {
        let mut atlas = MtsdfAtlas::new(256, 256);
        let key = (1, 100, 24);
        let uv = Rect::new(0.5, 0.5, 0.6, 0.6);
        let pixels = [255u8; 64];

        atlas.insert_glyph(key, uv, &pixels);

        let result = atlas.get_glyph_uv(1, 100, 24);
        assert_eq!(result, Some(uv));
    }

    #[test]
    fn test_get_uv_by_key() {
        let mut atlas = MtsdfAtlas::new(256, 256);
        let key = (2, 50, 12);
        let uv = Rect::new(0.2, 0.3, 0.4, 0.5);
        let pixels = [255u8; 64];

        atlas.insert_glyph(key, uv, &pixels);

        let result = atlas.get_uv(key);
        assert_eq!(result, Some(uv));
    }

    #[test]
    fn test_get_missing_glyph() {
        let atlas = MtsdfAtlas::new(256, 256);

        let result = atlas.get_glyph_uv(0, 999, 16);
        assert_eq!(result, None);
    }

    #[test]
    fn test_clear() {
        let mut atlas = MtsdfAtlas::new(256, 256);
        let key = (0, 1, 16);
        let uv = Rect::new(0.0, 0.0, 1.0, 1.0);
        let pixels = [255u8; 64];

        atlas.insert_glyph(key, uv, &pixels);
        assert_eq!(atlas.glyph_count(), 1);

        atlas.clear();
        assert_eq!(atlas.glyph_count(), 0);
        assert!(!atlas.contains(0, 1, 16));
    }

    #[test]
    fn test_contains() {
        let mut atlas = MtsdfAtlas::new(256, 256);
        let key = (5, 10, 20);
        let uv = Rect::new(0.0, 0.0, 1.0, 1.0);
        let pixels = [255u8; 64];

        assert!(!atlas.contains(5, 10, 20));

        atlas.insert_glyph(key, uv, &pixels);
        assert!(atlas.contains(5, 10, 20));
    }

    #[test]
    fn test_multiple_glyphs() {
        let mut atlas = MtsdfAtlas::new(512, 512);

        for i in 0..10 {
            let key = (0, i, 16);
            let uv = Rect::new(i as f32 * 0.1, 0.0, (i + 1) as f32 * 0.1, 0.1);
            let pixels = [255u8; 64];
            atlas.insert_glyph(key, uv, &pixels);
        }

        assert_eq!(atlas.glyph_count(), 10);

        for i in 0..10 {
            assert!(atlas.contains(0, i, 16));
        }
    }

    #[test]
    fn test_utilization() {
        let mut atlas = MtsdfAtlas::new(256, 256);

        assert_eq!(atlas.utilization(), 0.0);

        // Add some glyphs
        for i in 0..10 {
            let key = (0, i, 16);
            let uv = Rect::new(0.0, 0.0, 1.0, 1.0);
            let pixels = [255u8; 64];
            atlas.insert_glyph(key, uv, &pixels);
        }

        let util = atlas.utilization();
        assert!(util > 0.0);
        assert!(util <= 1.0);
    }

    #[test]
    fn test_different_font_sizes() {
        let mut atlas = MtsdfAtlas::new(512, 512);

        // Same glyph at different sizes
        let uv16 = Rect::new(0.0, 0.0, 0.1, 0.1);
        let uv32 = Rect::new(0.1, 0.0, 0.2, 0.2);
        let pixels = [255u8; 64];

        atlas.insert_glyph((0, 65, 16), uv16, &pixels);
        atlas.insert_glyph((0, 65, 32), uv32, &pixels);

        assert_eq!(atlas.glyph_count(), 2);
        assert!(atlas.contains(0, 65, 16));
        assert!(atlas.contains(0, 65, 32));
    }
}
