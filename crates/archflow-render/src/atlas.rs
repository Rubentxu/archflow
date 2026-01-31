// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Render - Texture Atlas with Shelf Packing
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 10
//
// Shelf Packing Algorithm:
// - O(shelves) insertion time - no full reorganization needed
// - Ideal for similarly-sized textures (like icon libraries)
// - Simple implementation and very fast
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(dead_code)]

use alloc::vec;
use alloc::vec::Vec;

/// Rectangle in the texture atlas
///
/// Represents the allocated region for a single texture/icon
/// with coordinates in texel space (0 to width/height)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AtlasRect {
    /// X coordinate of the rectangle's top-left corner (in texels)
    pub x: u32,

    /// Y coordinate of the rectangle's top-left corner (in texels)
    pub y: u32,

    /// Width of the rectangle (in texels)
    pub w: u32,

    /// Height of the rectangle (in texels)
    pub h: u32,
}

impl AtlasRect {
    /// Create a new rectangle
    #[inline(always)]
    pub const fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }

    /// Convert to normalized UV coordinates (0.0 to 1.0)
    ///
    /// Useful for GPU shader sampling
    #[inline(always)]
    pub fn to_uv(&self, atlas_width: u32, atlas_height: u32) -> [f32; 4] {
        let u_min = self.x as f32 / atlas_width as f32;
        let v_min = self.y as f32 / atlas_height as f32;
        let u_max = (self.x + self.w) as f32 / atlas_width as f32;
        let v_max = (self.y + self.h) as f32 / atlas_height as f32;
        [u_min, v_min, u_max - u_min, v_max - v_min]
    }

    /// Get the area of the rectangle
    #[inline(always)]
    pub const fn area(&self) -> u32 {
        self.w * self.h
    }

    /// Check if rectangle is empty (zero width or height)
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.w == 0 || self.h == 0
    }
}

/// Internal shelf structure for the Shelf Packing algorithm
///
/// Each shelf represents a horizontal row in the atlas
/// with a fixed height and horizontal placement cursor
#[derive(Clone, Debug)]
struct Shelf {
    /// Y coordinate where this shelf starts
    y_start: u32,

    /// Height of this shelf (determined by first texture placed)
    height: u32,

    /// Current X position for next placement in this shelf
    current_x: u32,
}

impl Shelf {
    /// Create a new shelf
    #[inline(always)]
    const fn new(y_start: u32, height: u32, first_width: u32) -> Self {
        Self {
            y_start,
            height,
            current_x: first_width,
        }
    }

    /// Check if a texture can fit in this shelf
    #[inline(always)]
    fn can_fit(&self, width: u32, height: u32, atlas_width: u32) -> bool {
        self.height >= height && (self.current_x + width) <= atlas_width
    }

    /// Get the remaining width in this shelf
    #[inline(always)]
    fn remaining_width(&self, atlas_width: u32) -> u32 {
        atlas_width.saturating_sub(self.current_x)
    }
}

/// Texture Atlas Packer using Shelf Packing algorithm
///
/// **Why Shelf Packing?**
/// - **O(shelves) insertion**: No need to reorganize entire atlas
/// - **Ideal for uniform textures**: Works best for similarly-sized items (icons, glyphs)
/// - **Simple and fast**: Easy to implement with minimal overhead
/// - **No fragmentation**: Each shelf is filled left-to-right, then we move to next
///
/// **Algorithm:**
/// 1. Try to place texture in existing shelf with sufficient height
/// 2. If none fits, create new shelf above the last one
/// 3. Returns None if atlas is full (no vertical space remaining)
///
/// **Use cases:**
/// - Icon atlases (AWS/Azure/GCP library icons)
/// - Font glyph atlases (MTSDF font rendering)
/// - UI element sprites
pub struct AtlasPacker {
    /// Total width of the atlas (in texels)
    width: u32,

    /// Total height of the atlas (in texels)
    height: u32,

    /// Vector of shelves, ordered from bottom to top
    shelves: Vec<Shelf>,

    /// Padding between textures to prevent "bleeding"
    /// (when linear filtering samples from neighboring textures)
    padding: u32,
}

impl AtlasPacker {
    /// Default padding to prevent texture bleeding (2 texels)
    pub const DEFAULT_PADDING: u32 = 2;

    /// Create a new atlas packer with specified dimensions
    ///
    /// # Arguments
    /// * `width` - Atlas width in texels (should be power of 2 for GPU)
    /// * `height` - Atlas height in texels (should be power of 2 for GPU)
    ///
    /// # Example
    /// ```rust
    /// # use archflow_render::AtlasPacker;
    /// let mut packer = AtlasPacker::new(2048, 2048);
    /// ```
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            shelves: Vec::with_capacity(32), // Pre-allocate for typical icon counts
            padding: Self::DEFAULT_PADDING,
        }
    }

    /// Create a new atlas packer with custom padding
    ///
    /// # Arguments
    /// * `width` - Atlas width in texels
    /// * `height` - Atlas height in texels
    /// * `padding` - Padding between textures in texels
    pub fn with_padding(width: u32, height: u32, padding: u32) -> Self {
        Self {
            width,
            height,
            shelves: Vec::with_capacity(32),
            padding,
        }
    }

    /// Allocate space in the atlas for a texture
    ///
    /// Attempts to find space for a texture of the given dimensions.
    /// Returns `Some(AtlasRect)` with the allocated rectangle, or `None` if
    /// the atlas is full or if the texture has zero width/height.
    ///
    /// # Arguments
    /// * `w` - Width of the texture to allocate (in texels)
    /// * `h` - Height of the texture to allocate (in texels)
    ///
    /// # Returns
    /// * `Some(AtlasRect)` - Rectangle with allocated coordinates
    /// * `None` - Atlas is full, or texture has zero size
    ///
    /// # Example
    /// ```rust
    /// # use archflow_render::AtlasPacker;
    /// if let Some(rect) = packer.allocate(64, 64) {
    ///     // Texture allocated at (rect.x, rect.y)
    ///     // Upload texture data to GPU at this position
    /// }
    /// ```
    pub fn allocate(&mut self, w: u32, h: u32) -> Option<AtlasRect> {
        // Reject zero-sized textures
        if w == 0 || h == 0 {
            return None;
        }

        // Apply padding to prevent texture bleeding
        let needed_w = w.saturating_add(self.padding);
        let needed_h = h.saturating_add(self.padding);

        // 1. Try to fit in existing shelf
        for shelf in &mut self.shelves {
            if shelf.can_fit(needed_w, needed_h, self.width) {
                let rect = AtlasRect {
                    x: shelf.current_x,
                    y: shelf.y_start,
                    w,
                    h,
                };
                shelf.current_x += needed_w;
                return Some(rect);
            }
        }

        // 2. No shelf fits, create new shelf above the last one
        let y_start = self
            .shelves
            .last()
            .map(|s| s.y_start.saturating_add(s.height))
            .unwrap_or(0);

        // Check if we have vertical space for new shelf
        if y_start.saturating_add(needed_h) <= self.height {
            self.shelves.push(Shelf::new(y_start, needed_h, needed_w));
            Some(AtlasRect {
                x: 0,
                y: y_start,
                w,
                h,
            })
        } else {
            None // Atlas is full
        }
    }

    /// Clear all allocations and reset the packer
    ///
    /// This removes all shelves and prepares the packer for reuse.
    /// The capacity of the shelves vector is preserved.
    pub fn clear(&mut self) {
        self.shelves.clear();
    }

    /// Get the current number of shelves in the atlas
    #[inline(always)]
    pub fn shelf_count(&self) -> usize {
        self.shelves.len()
    }

    /// Get the atlas width
    #[inline(always)]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Get the atlas height
    #[inline(always)]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Calculate the current utilization ratio (0.0 to 1.0)
    ///
    /// Returns the fraction of atlas area that is currently used.
    /// Useful for monitoring when to create a new atlas.
    ///
    /// # Example
    /// ```rust
    /// # use archflow_render::AtlasPacker;
    /// # let mut packer = AtlasPacker::new(1024, 1024);
    /// if packer.utilization() > 0.9 {
    ///     // Atlas is 90% full, consider creating a new one
    /// }
    /// ```
    pub fn utilization(&self) -> f32 {
        if self.shelves.is_empty() {
            return 0.0;
        }

        let total_area = self.width as f32 * self.height as f32;
        let used_area: u32 = self.shelves.iter().map(|s| s.height * s.current_x).sum();

        used_area as f32 / total_area
    }

    /// Get the amount of free space remaining (in texels²)
    pub fn free_area(&self) -> u32 {
        let total_area = self.width * self.height;
        let used_area: u32 = self.shelves.iter().map(|s| s.height * s.current_x).sum();
        total_area.saturating_sub(used_area)
    }

    /// Get the highest Y coordinate used (next shelf would start here)
    pub fn current_height(&self) -> u32 {
        self.shelves
            .last()
            .map(|s| s.y_start.saturating_add(s.height))
            .unwrap_or(0)
    }

    /// Reset the packer and change dimensions
    ///
    /// This clears all allocations and sets new atlas dimensions.
    /// Useful when reusing a packer for a different atlas size.
    pub fn reset_with_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.clear();
    }
}

impl Default for AtlasPacker {
    fn default() -> Self {
        Self::new(2048, 2048)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atlas_rect_creation() {
        let rect = AtlasRect::new(10, 20, 64, 32);
        assert_eq!(rect.x, 10);
        assert_eq!(rect.y, 20);
        assert_eq!(rect.w, 64);
        assert_eq!(rect.h, 32);
    }

    #[test]
    fn test_atlas_rect_area() {
        let rect = AtlasRect::new(0, 0, 64, 32);
        assert_eq!(rect.area(), 2048);
    }

    #[test]
    fn test_atlas_rect_is_empty() {
        assert!(AtlasRect::new(0, 0, 0, 32).is_empty());
        assert!(AtlasRect::new(0, 0, 64, 0).is_empty());
        assert!(!AtlasRect::new(0, 0, 64, 32).is_empty());
    }

    #[test]
    fn test_atlas_rect_to_uv() {
        let rect = AtlasRect::new(0, 0, 512, 256);
        let uv = rect.to_uv(2048, 2048);

        assert!((uv[0] - 0.0).abs() < 0.001); // u_min
        assert!((uv[1] - 0.0).abs() < 0.001); // v_min
        assert!((uv[2] - 0.25).abs() < 0.001); // w (512/2048)
        assert!((uv[3] - 0.125).abs() < 0.001); // h (256/2048)
    }

    #[test]
    fn test_packer_creation() {
        let packer = AtlasPacker::new(1024, 1024);
        assert_eq!(packer.width(), 1024);
        assert_eq!(packer.height(), 1024);
        assert_eq!(packer.shelf_count(), 0);
    }

    #[test]
    fn test_packer_default() {
        let packer = AtlasPacker::default();
        assert_eq!(packer.width(), 2048);
        assert_eq!(packer.height(), 2048);
    }

    #[test]
    fn test_single_allocation() {
        let mut packer = AtlasPacker::new(1024, 1024);

        let rect = packer.allocate(64, 64);
        assert!(rect.is_some());
        assert_eq!(rect.unwrap(), AtlasRect::new(0, 0, 64, 64));
        assert_eq!(packer.shelf_count(), 1);
    }

    #[test]
    fn test_multiple_same_size() {
        let mut packer = AtlasPacker::new(1024, 1024);

        let r1 = packer.allocate(64, 64);
        let r2 = packer.allocate(64, 64);
        let r3 = packer.allocate(64, 64);

        assert!(r1.is_some());
        assert!(r2.is_some());
        assert!(r3.is_some());

        // All should be in same shelf (same height)
        assert_eq!(packer.shelf_count(), 1);

        // Check x coordinates are spaced with padding
        assert_eq!(r1.unwrap().x, 0);
        assert_eq!(r2.unwrap().x, 66); // 64 + 2 padding
        assert_eq!(r3.unwrap().x, 132); // 66 + 64 + 2
    }

    #[test]
    fn test_different_heights() {
        let mut packer = AtlasPacker::new(1024, 1024);

        let r1 = packer.allocate(64, 32); // First shelf: height 32+2=34
        let r2 = packer.allocate(64, 64); // Won't fit in first shelf
        let r3 = packer.allocate(32, 64); // Fits in second shelf

        assert!(r1.is_some());
        assert!(r2.is_some());
        assert!(r3.is_some());

        assert_eq!(packer.shelf_count(), 2);
    }

    #[test]
    fn test_shelf_filling() {
        let mut packer = AtlasPacker::new(400, 256);

        // Fill first shelf with 128x64 textures
        // With padding (2): 130 width each, so 3 fit in 400 width
        let r1 = packer.allocate(128, 64); // Shelf 0: x=0, current_x=130
        let r2 = packer.allocate(128, 64); // Shelf 0: x=130, current_x=260
        let r3 = packer.allocate(128, 64); // Shelf 0: x=260, current_x=390
        let r4 = packer.allocate(128, 64); // Creates shelf 1 (390+130=520 > 400)

        assert!(r1.is_some());
        assert!(r2.is_some());
        assert!(r3.is_some());
        assert!(r4.is_some());

        // First shelf filled with 3 items, fourth creates second shelf
        assert_eq!(packer.shelf_count(), 2);

        // Verify positions
        assert_eq!(r1.unwrap().x, 0);
        assert_eq!(r2.unwrap().x, 130); // 128 + 2 padding
        assert_eq!(r3.unwrap().x, 260); // 130 + 128 + 2
        assert_eq!(r4.unwrap().x, 0); // New shelf starts at x=0
        assert!(r4.unwrap().y > 0); // New shelf is below first
    }

    #[test]
    fn test_atlas_full() {
        let mut packer = AtlasPacker::new(100, 100);

        // Fill exactly to capacity
        let r1 = packer.allocate(96, 96);
        assert!(r1.is_some());

        // Should not fit
        let r2 = packer.allocate(10, 10);
        assert!(r2.is_none());
    }

    #[test]
    fn test_utilization() {
        let mut packer = AtlasPacker::new(1024, 1024);

        assert_eq!(packer.utilization(), 0.0);

        packer.allocate(512, 512);
        // With padding: 514 x 514 = 264,196 texels²
        // Atlas: 1,048,576 texels²
        // ~25%
        let util = packer.utilization();
        assert!(util > 0.20 && util < 0.30);
    }

    #[test]
    fn test_free_area() {
        let mut packer = AtlasPacker::new(1024, 1024);
        let total = 1024 * 1024;

        assert_eq!(packer.free_area(), total);

        packer.allocate(512, 512);
        let used = 514 * 514; // With padding
        assert_eq!(packer.free_area(), total - used);
    }

    #[test]
    fn test_current_height() {
        let mut packer = AtlasPacker::new(1024, 1024);

        assert_eq!(packer.current_height(), 0);

        packer.allocate(100, 50);
        // First shelf: height 52 (50 + 2 padding)
        assert_eq!(packer.current_height(), 52);

        packer.allocate(100, 100);
        // Second shelf starts at y=52, height 102
        assert_eq!(packer.current_height(), 154);
    }

    #[test]
    fn test_clear() {
        let mut packer = AtlasPacker::new(1024, 1024);

        packer.allocate(64, 64);
        packer.allocate(128, 128);
        assert_eq!(packer.shelf_count(), 2);

        packer.clear();
        assert_eq!(packer.shelf_count(), 0);
        assert_eq!(packer.utilization(), 0.0);
    }

    #[test]
    fn test_custom_padding() {
        let mut packer = AtlasPacker::with_padding(1024, 1024, 4);

        let r1 = packer.allocate(64, 64);
        let r2 = packer.allocate(64, 64);

        assert_eq!(packer.shelf_count(), 1);
        // With 4 pixel padding: 64 + 4 = 68
        assert_eq!(r2.unwrap().x, 68);
    }

    #[test]
    fn test_reset_with_size() {
        let mut packer = AtlasPacker::new(1024, 1024);

        packer.allocate(64, 64);
        assert_eq!(packer.shelf_count(), 1);

        packer.reset_with_size(512, 512);
        assert_eq!(packer.width(), 512);
        assert_eq!(packer.height(), 512);
        assert_eq!(packer.shelf_count(), 0);
    }

    #[test]
    fn test_zero_size_allocation() {
        let mut packer = AtlasPacker::new(1024, 1024);

        // Zero size should return None
        assert!(packer.allocate(0, 64).is_none());
        assert!(packer.allocate(64, 0).is_none());
    }

    #[test]
    fn test_large_texture_fits() {
        let mut packer = AtlasPacker::new(2048, 2048);

        // Texture that nearly fills the atlas
        let rect = packer.allocate(2000, 2000);
        assert!(rect.is_some());
        assert_eq!(rect.unwrap().w, 2000);
        assert_eq!(rect.unwrap().h, 2000);
    }
}
