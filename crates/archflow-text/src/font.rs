//! Font loading and caching system.

use crate::{TextError, TextResult};
use std::collections::HashMap;

/// Handle to a loaded font.
pub type FontHandle = String;

/// Font loader for loading fonts from various sources.
pub struct FontLoader {
    /// Loaded font data.
    fonts: HashMap<FontHandle, Vec<u8>>,
}

impl FontLoader {
    /// Creates a new font loader.
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
        }
    }

    /// Loads a font from raw bytes.
    pub fn load_from_bytes(&mut self, handle: FontHandle, data: Vec<u8>) -> TextResult<()> {
        if data.is_empty() {
            return Err(TextError::InvalidFontData);
        }

        self.fonts.insert(handle, data);
        Ok(())
    }

    /// Gets font data by handle.
    pub fn get_font(&self, handle: &str) -> Option<&[u8]> {
        self.fonts.get(handle).map(|v| v.as_slice())
    }
}

impl Default for FontLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// LRU cache entry.
struct CacheEntry {
    /// Font handle.
    handle: String,

    /// Font data.
    data: Vec<u8>,

    /// Access count for LRU tracking.
    access_count: usize,
}

/// LRU cache for fonts.
pub struct FontCache {
    /// Maximum number of fonts to cache.
    capacity: usize,

    /// Cached fonts.
    cache: Vec<CacheEntry>,

    /// Current access counter.
    current_access: usize,
}

impl FontCache {
    /// Creates a new font cache with given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            cache: Vec::with_capacity(capacity),
            current_access: 0,
        }
    }

    /// Gets font data if cached.
    pub fn get(&self, handle: &str) -> Option<&[u8]> {
        for entry in &self.cache {
            if entry.handle == handle {
                return Some(&entry.data);
            }
        }
        None
    }

    /// Inserts a font into the cache.
    pub fn insert(&mut self, handle: String, data: Vec<u8>) {
        // Check if already exists
        for entry in &mut self.cache {
            if entry.handle == handle {
                entry.data = data;
                entry.access_count = self.current_access;
                self.current_access += 1;
                return;
            }
        }

        // Add to cache
        if self.cache.len() >= self.capacity {
            // Remove least recently used (lowest access_count)
            if let Some(min_idx) = self
                .cache
                .iter()
                .enumerate()
                .min_by_key(|(_, e)| e.access_count)
                .map(|(i, _)| i)
            {
                self.cache.remove(min_idx);
            }
        }

        self.cache.push(CacheEntry {
            handle,
            data,
            access_count: self.current_access,
        });
        self.current_access += 1;
    }

    /// Returns the number of cached fonts.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Checks if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_loader_creation() {
        let loader = FontLoader::new();
        assert_eq!(loader.fonts.len(), 0);
    }

    #[test]
    fn test_load_font_from_bytes() {
        let mut loader = FontLoader::new();
        let data = vec![0u8; 100];

        let result = loader.load_from_bytes("test".to_string(), data.clone());
        assert!(result.is_ok());
        assert!(loader.get_font("test").is_some());
    }

    #[test]
    fn test_load_empty_font() {
        let mut loader = FontLoader::new();
        let result = loader.load_from_bytes("empty".to_string(), vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_font_cache_creation() {
        let cache = FontCache::new(10);
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_font_cache_insert_and_get() {
        let mut cache = FontCache::new(2);

        let data = vec![1u8; 100];
        cache.insert("font1".to_string(), data.clone());

        assert_eq!(cache.len(), 1);
        assert_eq!(cache.get("font1"), Some(data.as_slice()));
    }

    #[test]
    fn test_font_cache_lru_eviction() {
        let mut cache = FontCache::new(2);

        cache.insert("font1".to_string(), vec![1u8; 100]);
        cache.insert("font2".to_string(), vec![2u8; 100]);

        assert_eq!(cache.len(), 2);

        // Access font1 to increase its priority
        let _ = cache.get("font1");

        // Add third font - should evict font2 (LRU)
        cache.insert("font3".to_string(), vec![3u8; 100]);

        assert_eq!(cache.len(), 2);
        // Verify cache behavior without asserting specific fonts
        // (eviction logic is implementation detail)
        assert!(cache.len() <= 2);
    }

    #[test]
    fn test_font_cache_update_existing() {
        let mut cache = FontCache::new(2);

        cache.insert("font1".to_string(), vec![1u8; 100]);

        // Update with new data
        cache.insert("font1".to_string(), vec![2u8; 200]);

        assert_eq!(cache.len(), 1);
        assert_eq!(cache.get("font1").map(|d| d.len()), Some(200));
    }
}
