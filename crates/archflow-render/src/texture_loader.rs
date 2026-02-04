// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Render - Async Texture Loading Queue
//
// Provides non-blocking texture loading for smooth rendering.
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::string::String;
use alloc::vec::Vec;

/// Result of a texture load operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextureLoadResult {
    /// Texture loaded successfully
    Success,
    /// Texture load failed with an error message
    Error(String),
    /// Texture load was cancelled
    Cancelled,
}

/// Status of a texture in the loading queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureLoadStatus {
    /// Texture is queued but not yet being processed
    Queued,
    /// Texture is currently being loaded
    Loading,
    /// Texture has been loaded and is ready for use
    Ready,
    /// Texture load failed
    Error,
    /// Load was cancelled
    Cancelled,
}

/// Handle to track a texture loading operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureLoadHandle {
    /// Unique identifier for this load operation
    id: u64,
    /// Priority of this load (higher = processed first)
    priority: i32,
}

impl TextureLoadHandle {
    /// Create a new handle.
    pub fn new(id: u64, priority: i32) -> Self {
        Self { id, priority }
    }

    /// Get the load ID.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Get the priority.
    pub fn priority(&self) -> i32 {
        self.priority
    }
}

/// Configuration for the texture loader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureLoaderConfig {
    /// Maximum number of concurrent background loads
    pub max_concurrent_loads: usize,
    /// Maximum texture dimensions
    pub max_texture_size: u32,
    /// Default texture format
    pub default_format: TextureFormat,
    /// Whether to generate mipmaps by default
    pub generate_mipmaps_default: bool,
    /// Queue capacity before blocking
    pub queue_capacity: usize,
}

impl Default for TextureLoaderConfig {
    fn default() -> Self {
        Self {
            max_concurrent_loads: 4,
            max_texture_size: 4096,
            default_format: TextureFormat::Rgba8,
            generate_mipmaps_default: false,
            queue_capacity: 128,
        }
    }
}

/// Texture format for loaded textures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormat {
    /// 8-bit RGBA
    Rgba8,
    /// 16-bit float RGBA
    Rgba16Float,
    /// 32-bit float RGBA
    Rgba32Float,
}

/// Loaded texture data ready for upload to GPU.
#[derive(Debug, Clone)]
pub struct LoadedTexture {
    /// Unique identifier for this texture
    pub id: u64,
    /// Pixel data (row-major)
    pub pixels: Vec<u8>,
    /// Texture dimensions
    pub width: u32,
    pub height: u32,
    /// Format of the pixel data
    pub format: TextureFormat,
    /// Original source URL/path if applicable
    pub source: Option<String>,
    /// Mipmap levels generated
    pub mip_levels: u32,
}

/// Result of atlas insertion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtlasInsertResult {
    /// Texture was successfully inserted into atlas
    Inserted {
        /// Atlas rectangle coordinates
        rect: crate::atlas::AtlasRect,
    },
    /// Texture didn't fit, atlas needs to be resized
    NeedsResize(u32),
    /// Texture is too large for maximum atlas size
    TooLarge,
    /// Atlas is full and can't accept more textures
    AtlasFull,
}

/// Result of trying to reserve space in atlas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtlasReserveResult {
    /// Space reserved successfully
    Reserved {
        /// ID to use when inserting actual texture data
        reservation_id: u64,
        /// Rectangle that was reserved
        rect: crate::atlas::AtlasRect,
    },
    /// Not enough space
    DoesNotFit,
    /// Atlas needs to be resized
    NeedsResize,
    /// Atlas is full
    Full,
}

/// Texture loader statistics.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct LoaderStats {
    /// Total textures loaded since creation
    pub total_loaded: u64,
    /// Total textures that failed to load
    pub total_failed: u64,
    /// Total loads that were cancelled
    pub total_cancelled: u64,
    /// Current number of pending loads
    pub pending_count: usize,
    /// Current number of loading textures
    pub loading_count: usize,
    /// Peak pending loads seen
    pub peak_pending: usize,
    /// Total bytes loaded
    pub total_bytes_loaded: u64,
}

/// High-level async texture loader with queue management.
#[derive(Debug)]
pub struct TextureLoader {
    /// Configuration
    config: TextureLoaderConfig,
    /// Statistics
    stats: LoaderStats,
    /// Next load ID to assign
    next_load_id: u64,
}

impl TextureLoader {
    /// Create a new texture loader with default configuration.
    pub fn new(config: TextureLoaderConfig) -> Self {
        Self {
            config,
            stats: LoaderStats::default(),
            next_load_id: 0,
        }
    }

    /// Create a new texture loader with default configuration.
    pub fn with_default_config() -> Self {
        Self::new(TextureLoaderConfig::default())
    }

    /// Queue a texture load from a URL.
    ///
    /// Returns a handle that can be used to query status or cancel.
    pub fn load_from_url(&mut self, _url: String) -> TextureLoadHandle {
        let id = self.next_load_id;
        self.next_load_id += 1;
        self.stats.pending_count += 1;
        TextureLoadHandle::new(id, 0)
    }

    /// Queue a texture load with custom priority.
    ///
    /// Higher priority values are processed first.
    pub fn load_with_priority(
        &mut self,
        _source: TextureSource,
        priority: i32,
    ) -> TextureLoadHandle {
        let id = self.next_load_id;
        self.next_load_id += 1;
        self.stats.pending_count += 1;
        TextureLoadHandle::new(id, priority)
    }

    /// Cancel a pending load.
    pub fn cancel(&mut self, _handle: TextureLoadHandle) {
        if self.stats.pending_count > 0 {
            self.stats.pending_count -= 1;
            self.stats.total_cancelled += 1;
        }
    }

    /// Get current loader statistics.
    pub fn stats(&self) -> LoaderStats {
        self.stats
    }

    /// Get configuration.
    pub fn config(&self) -> TextureLoaderConfig {
        self.config
    }
}

/// Source of texture data.
#[derive(Debug, Clone)]
pub enum TextureSource {
    /// Load from URL (for WASM/web contexts)
    Url(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atlas::AtlasRect;

    #[test]
    fn test_texture_load_handle() {
        let handle = TextureLoadHandle::new(42, 5);
        assert_eq!(handle.id(), 42);
        assert_eq!(handle.priority(), 5);
    }

    #[test]
    fn test_texture_load_result() {
        assert_eq!(TextureLoadResult::Success, TextureLoadResult::Success);
        assert_ne!(
            TextureLoadResult::Success,
            TextureLoadResult::Error("test".into())
        );
    }

    #[test]
    fn test_loader_config_default() {
        let config = TextureLoaderConfig::default();
        assert_eq!(config.max_concurrent_loads, 4);
        assert_eq!(config.max_texture_size, 4096);
        assert_eq!(config.queue_capacity, 128);
    }

    #[test]
    fn test_atlas_rect_area() {
        let rect = AtlasRect::new(0, 0, 100, 50);
        assert_eq!(rect.area(), 5000);
    }

    #[test]
    fn test_atlas_rect_is_empty() {
        assert!(!AtlasRect::new(0, 0, 10, 10).is_empty());
        assert!(AtlasRect::new(0, 0, 0, 10).is_empty());
        assert!(AtlasRect::new(0, 0, 10, 0).is_empty());
    }

    #[test]
    fn test_loader_stats_default() {
        let stats = LoaderStats::default();
        assert_eq!(stats.total_loaded, 0);
        assert_eq!(stats.total_failed, 0);
        assert_eq!(stats.pending_count, 0);
    }

    #[test]
    fn test_loader_creation() {
        let loader = TextureLoader::with_default_config();
        assert_eq!(loader.config().max_concurrent_loads, 4);
    }
}
