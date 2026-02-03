// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Text - Glyph Run Cache
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 12.2
//
// Caches text layout results to avoid re-shaping text on every frame.
// Uses LRU eviction policy to manage memory usage.
// ═══════════════════════════════════════════════════════════════════════════════

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use archflow_core::Vec2;

/// Maximum number of cached glyph runs
const MAX_CACHE_ENTRIES: usize = 256;

/// Glyph cache key: hash of text + font_size
type CacheKey = u64;

/// Flat glyph run for GPU upload
///
/// This structure is designed for direct GPU upload without additional processing.
#[repr(C)]
#[derive(Clone, Debug)]
pub struct FlatGlyphRun {
    /// Glyph positions in screen space [x, y]
    pub glyph_positions: Vec<[f32; 2]>,

    /// Glyph UV rectangles in the atlas [min_x, min_y, max_x, max_y]
    pub glyph_uvs: Vec<[f32; 4]>,

    /// Total number of glyphs
    pub total_glyphs: usize,
}

impl FlatGlyphRun {
    /// Create an empty glyph run
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            glyph_positions: Vec::new(),
            glyph_uvs: Vec::new(),
            total_glyphs: 0,
        }
    }

    /// Add a glyph to the run
    #[inline(always)]
    pub fn add_glyph(&mut self, position: Vec2, uv_rect: [f32; 4]) {
        self.glyph_positions.push([position.x, position.y]);
        self.glyph_uvs.push(uv_rect);
        self.total_glyphs += 1;
    }

    /// Clear all glyphs
    #[inline(always)]
    pub fn clear(&mut self) {
        self.glyph_positions.clear();
        self.glyph_uvs.clear();
        self.total_glyphs = 0;
    }

    /// Check if the run is empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.total_glyphs == 0
    }

    /// Get the number of glyphs
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.total_glyphs
    }
}

impl Default for FlatGlyphRun {
    fn default() -> Self {
        Self::new()
    }
}

/// LRU entry with access tracking
struct CacheEntry {
    /// The glyph run data
    glyph_run: FlatGlyphRun,

    /// Last access timestamp (for LRU eviction)
    last_access: u64,
}

/// Glyph run cache with LRU eviction
///
/// Caches shaped text layouts to avoid expensive text shaping on every frame.
///
/// **Note**: Due to borrow checker constraints, this cache returns cloned
/// glyph runs. In practice, this is acceptable since glyph runs are small
/// (typically < 100 glyphs) and the cache avoids expensive text shaping.
pub struct GlyphRunCache {
    /// Cached glyph runs
    cache: BTreeMap<CacheKey, CacheEntry>,

    /// Current timestamp for LRU tracking
    current_time: u64,
}

impl GlyphRunCache {
    /// Create a new glyph run cache
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            cache: BTreeMap::new(),
            current_time: 0,
        }
    }

    /// Get or compute a glyph run for the given text
    ///
    /// # Arguments
    /// * `text` - The text to shape
    /// * `font_size` - Font size in pixels
    /// * `compute_fn` - Function to compute the layout if not cached
    ///
    /// # Returns
    /// Cloned glyph run (to avoid borrow checker issues)
    pub fn get_or_compute<F>(&mut self, text: &str, font_size: f32, compute_fn: F) -> FlatGlyphRun
    where
        F: FnOnce(&str, f32) -> FlatGlyphRun,
    {
        let key = self.hash_text_and_scale(text, font_size);

        // Update access time
        self.current_time = self.current_time.wrapping_add(1);

        // Check if cached
        if let Some(entry) = self.cache.get_mut(&key) {
            entry.last_access = self.current_time;
            return entry.glyph_run.clone();
        }

        // Evict if cache is full
        if self.cache.len() >= MAX_CACHE_ENTRIES {
            self.evict_lru();
        }

        // Compute new layout
        let glyph_run = compute_fn(text, font_size);

        // Insert into cache
        self.cache.insert(
            key,
            CacheEntry {
                glyph_run: glyph_run.clone(),
                last_access: self.current_time,
            },
        );

        glyph_run
    }

    /// Hash text and scale for cache key
    #[inline(always)]
    fn hash_text_and_scale(&self, text: &str, font_size: f32) -> u64 {
        // Simple hash combining text bytes and font size
        let mut hash: u64 = 0xcbf29ce484222325;

        for byte in text.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }

        hash ^= font_size.to_bits() as u64;
        hash = hash.wrapping_mul(0x100000001b3);

        hash
    }

    /// Evict the least recently used entry
    fn evict_lru(&mut self) {
        let mut lru_key = None;
        let mut lru_time = u64::MAX;

        for (&key, entry) in &self.cache {
            if entry.last_access < lru_time {
                lru_time = entry.last_access;
                lru_key = Some(key);
            }
        }

        if let Some(key) = lru_key {
            self.cache.remove(&key);
        }
    }

    /// Clear all cached entries
    #[inline(always)]
    pub fn clear(&mut self) {
        self.cache.clear();
        self.current_time = 0;
    }

    /// Get the number of cached entries
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if the cache is empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Get cache statistics
    ///
    /// # Returns
    /// Number of cached entries
    #[inline(always)]
    pub fn stats(&self) -> usize {
        self.cache.len()
    }

    /// Prefill the cache with common text
    ///
    /// # Arguments
    /// * `texts` - List of (text, font_size) pairs to prefill
    /// * `compute_fn` - Function to compute layouts
    pub fn prefill<F>(&mut self, texts: &[(String, f32)], compute_fn: F)
    where
        F: Fn(&str, f32) -> FlatGlyphRun,
    {
        for (text, font_size) in texts {
            let _ = self.get_or_compute(text.as_str(), *font_size, &compute_fn);
        }
    }
}

impl Default for GlyphRunCache {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format; // For format! macro in no_std
    use alloc::string::String;
    use alloc::vec; // For vec! macro in no_std
    use archflow_core::Vec2;

    fn dummy_compute(text: &str, _font_size: f32) -> FlatGlyphRun {
        let mut run = FlatGlyphRun::new();
        for (i, _c) in text.chars().enumerate() {
            run.add_glyph(Vec2::new(i as f32 * 10.0, 0.0), [0.0, 0.0, 0.1, 0.1]);
        }
        run
    }

    #[test]
    fn test_cache_creation() {
        let cache = GlyphRunCache::new();

        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_default() {
        let cache = GlyphRunCache::default();

        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_get_or_compute() {
        let mut cache = GlyphRunCache::new();
        let text = "Hello";
        let font_size = 16.0;

        let result = cache.get_or_compute(text, font_size, dummy_compute);

        assert_eq!(result.len(), 5); // "Hello" has 5 characters
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_cache_hit() {
        let mut cache = GlyphRunCache::new();
        let text = "World";
        let font_size = 20.0;

        // First access - cache miss
        let _result1 = cache.get_or_compute(text, font_size, dummy_compute);
        assert_eq!(cache.len(), 1);

        // Second access - cache hit
        let result2 = cache.get_or_compute(text, font_size, dummy_compute);
        assert_eq!(cache.len(), 1); // No new entry

        // Result should be valid
        assert_eq!(result2.len(), 5);
    }

    #[test]
    fn test_different_keys() {
        let mut cache = GlyphRunCache::new();

        cache.get_or_compute("Test", 16.0, dummy_compute);
        cache.get_or_compute("Test", 24.0, dummy_compute);
        cache.get_or_compute("Other", 16.0, dummy_compute);

        // Different text or font size = different cache entries
        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn test_clear() {
        let mut cache = GlyphRunCache::new();

        cache.get_or_compute("ABC", 12.0, dummy_compute);
        cache.get_or_compute("DEF", 14.0, dummy_compute);

        assert_eq!(cache.len(), 2);

        cache.clear();

        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_flat_glyph_run() {
        let mut run = FlatGlyphRun::new();

        assert!(run.is_empty());
        assert_eq!(run.len(), 0);

        run.add_glyph(Vec2::new(0.0, 0.0), [0.0, 0.0, 0.1, 0.1]);
        run.add_glyph(Vec2::new(10.0, 0.0), [0.1, 0.0, 0.2, 0.1]);

        assert!(!run.is_empty());
        assert_eq!(run.len(), 2);
        assert_eq!(run.glyph_positions.len(), 2);
        assert_eq!(run.glyph_uvs.len(), 2);
    }

    #[test]
    fn test_flat_glyph_run_clear() {
        let mut run = FlatGlyphRun::new();

        run.add_glyph(Vec2::new(5.0, 5.0), [0.0, 0.0, 1.0, 1.0]);

        assert_eq!(run.len(), 1);

        run.clear();

        assert!(run.is_empty());
        assert_eq!(run.len(), 0);
    }

    #[test]
    fn test_lru_eviction() {
        let mut cache = GlyphRunCache::new();

        // Fill the cache
        for i in 0..MAX_CACHE_ENTRIES {
            let text = format!("Text{}", i);
            let _ = cache.get_or_compute(&text, 16.0, dummy_compute);
        }

        assert_eq!(cache.len(), MAX_CACHE_ENTRIES);

        // Add one more - should evict LRU
        let _ = cache.get_or_compute("New", 16.0, dummy_compute);

        assert_eq!(cache.len(), MAX_CACHE_ENTRIES);
    }

    #[test]
    fn test_prefill() {
        let mut cache = GlyphRunCache::new();

        let texts = vec![String::from("Hello"), String::from("World")];
        let sizes = vec![16.0, 20.0];
        let pairs: Vec<(String, f32)> = texts.into_iter().zip(sizes.into_iter()).collect();

        cache.prefill(&pairs, dummy_compute);

        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_empty_text() {
        let mut cache = GlyphRunCache::new();

        let result = cache.get_or_compute("", 16.0, dummy_compute);

        assert_eq!(result.len(), 0);
        assert!(result.is_empty());
    }
}
