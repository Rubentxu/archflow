// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Plugins - SVG Rasterizer
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 14.3
//
// Rasterizes SVG icons to GPU texture atlas using resvg/tiny-skia.
// Integrates with AtlasPacker for efficient texture space usage.
// ═══════════════════════════════════════════════════════════════════════════════

extern crate alloc;

use alloc::vec::Vec;

use archflow_core::Rect;

/// SVG Rasterizer for GPU texture atlas
///
/// Parses SVG data and renders it to a texture atlas
/// using shelf-packing for efficient space usage.
pub struct SvgRasterizer {
    /// Atlas packer for texture space management
    pub packer: AtlasPacker,
    /// Whether the rasterizer is initialized
    pub initialized: bool,
}

impl SvgRasterizer {
    /// Create a new SVG rasterizer
    ///
    /// # Arguments
    /// * `width` - Atlas texture width
    /// * `height` - Atlas texture height
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            packer: AtlasPacker::new(width, height),
            initialized: true,
        }
    }

    /// Add an SVG to the atlas
    ///
    /// # Arguments
    /// * `svg_data` - The SVG data as a string
    /// * `size` - Target size for the rasterized icon
    ///
    /// # Returns
    /// UV rectangle in the atlas, or None if allocation failed
    pub fn add_svg(&mut self, svg_data: &str, size: u32) -> Option<Rect> {
        // 1. Allocate space in atlas
        let rect = self.packer.allocate(size, size)?;

        // 2. Parse SVG (stub for now - would use resvg in full implementation)
        // The actual implementation would:
        // let tree = usvg::Tree::from_data(svg_data.as_bytes(), &Default::default()).ok()?;

        // 3. Render to pixel buffer (in real impl, would upload to GPU)
        let _pixels = self.render_svg_stub(svg_data, size);

        // 4. In a real implementation, upload to GPU texture here
        // queue.write_texture(...);

        Some(Rect::new(
            rect.x as f32 / self.packer.width as f32,
            rect.y as f32 / self.packer.height as f32,
            (rect.x + rect.w) as f32 / self.packer.width as f32,
            (rect.y + rect.h) as f32 / self.packer.height as f32,
        ))
    }

    /// Stub renderer for SVG (placeholder)
    ///
    /// In production, this would use resvg to properly render SVG.
    /// For now, returns a simple colored rectangle.
    fn render_svg_stub(&self, _svg_data: &str, size: u32) -> Vec<u8> {
        // Create a simple test pattern
        let mut pixels = Vec::with_capacity((size * size * 4) as usize);

        for y in 0..size {
            for x in 0..size {
                // Create a simple gradient pattern
                let r = (x * 255 / size) as u8;
                let g = (y * 255 / size) as u8;
                let b = 128;
                let a = 255;

                pixels.push(r);
                pixels.push(g);
                pixels.push(b);
                pixels.push(a);
            }
        }

        pixels
    }

    /// Get current atlas utilization
    pub fn utilization(&self) -> f32 {
        self.packer.utilization()
    }

    /// Clear the atlas
    pub fn clear(&mut self) {
        self.packer = AtlasPacker::new(self.packer.width, self.packer.height);
    }
}

/// Atlas packer using shelf-packing algorithm
///
/// Implements the shelf-packing algorithm for efficient
/// rectangle allocation in a texture atlas.
pub struct AtlasPacker {
    /// Atlas width
    pub width: u32,
    /// Atlas height
    pub height: u32,
    /// Current shelves
    shelves: Vec<Shelf>,
    /// Padding between icons
    padding: u32,
}

/// A shelf in the atlas packer
#[derive(Clone, Debug)]
struct Shelf {
    /// Y position of this shelf
    y_start: u32,
    /// Height of this shelf
    height: u32,
    /// Current X position in this shelf
    current_x: u32,
}

impl AtlasPacker {
    /// Create a new atlas packer
    ///
    /// # Arguments
    /// * `width` - Atlas width
    /// * `height` - Atlas height
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            shelves: Vec::new(),
            padding: 2, // 2px padding between icons
        }
    }

    /// Allocate space for a rectangle
    ///
    /// # Arguments
    /// * `width` - Rectangle width
    /// * `height` - Rectangle height
    ///
    /// # Returns
    /// Allocated rectangle, or None if no space available
    pub fn allocate(&mut self, width: u32, height: u32) -> Option<PackedRect> {
        let padded_width = width + self.padding * 2;
        let padded_height = height + self.padding * 2;

        // Try to fit in existing shelf
        for shelf in &mut self.shelves {
            if shelf.height >= padded_height && shelf.current_x + padded_width <= self.width {
                let x = shelf.current_x + self.padding;
                let y = shelf.y_start + self.padding;

                shelf.current_x += padded_width;

                return Some(PackedRect {
                    x,
                    y,
                    w: width,
                    h: height,
                });
            }
        }

        // Need to create new shelf
        let new_y = if let Some(last) = self.shelves.last() {
            last.y_start + last.height
        } else {
            0
        };

        if new_y + padded_height <= self.height {
            let shelf = Shelf {
                y_start: new_y,
                height: padded_height,
                current_x: padded_width,
            };

            let x = self.padding;
            let y = new_y + self.padding;

            self.shelves.push(shelf);

            Some(PackedRect {
                x,
                y,
                w: width,
                h: height,
            })
        } else {
            None // No space available
        }
    }

    /// Calculate atlas utilization (0.0 to 1.0)
    pub fn utilization(&self) -> f32 {
        if self.shelves.is_empty() {
            return 0.0;
        }

        let used_height = self
            .shelves
            .last()
            .map(|s| s.y_start + s.height)
            .unwrap_or(0);
        let total_area = (self.width * self.height) as f32;

        if total_area == 0.0 {
            return 0.0;
        }

        let used_area = (self.width * used_height) as f32;
        used_area / total_area
    }

    /// Clear all allocations
    pub fn clear(&mut self) {
        self.shelves.clear();
    }
}

/// Packed rectangle in atlas
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackedRect {
    /// X position
    pub x: u32,
    /// Y position
    pub y: u32,
    /// Width
    pub w: u32,
    /// Height
    pub h: u32,
}

impl Default for SvgRasterizer {
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
    fn test_svg_rasterizer_creation() {
        let rasterizer = SvgRasterizer::new(1024, 1024);
        assert!(rasterizer.initialized);
        assert_eq!(rasterizer.packer.width, 1024);
        assert_eq!(rasterizer.packer.height, 1024);
    }

    #[test]
    fn test_svg_rasterizer_default() {
        let rasterizer = SvgRasterizer::default();
        assert_eq!(rasterizer.packer.width, 2048);
        assert_eq!(rasterizer.packer.height, 2048);
    }

    #[test]
    fn test_add_svg() {
        let mut rasterizer = SvgRasterizer::new(1024, 1024);
        let svg_data = "<svg><rect width='100' height='100'/></svg>";

        let result = rasterizer.add_svg(svg_data, 64);
        assert!(result.is_some());

        let rect = result.unwrap();
        assert!(rect.min.x >= 0.0);
        assert!(rect.min.x < 1.0);
        assert!(rect.min.y >= 0.0);
        assert!(rect.min.y < 1.0);
    }

    #[test]
    fn test_add_multiple_svgs() {
        let mut rasterizer = SvgRasterizer::new(1024, 1024);
        let svg_data = "<svg></svg>";

        // Add multiple icons
        for _ in 0..10 {
            let result = rasterizer.add_svg(svg_data, 64);
            assert!(result.is_some());
        }
    }

    #[test]
    fn test_utilization() {
        let mut rasterizer = SvgRasterizer::new(1024, 1024);
        assert_eq!(rasterizer.utilization(), 0.0);

        let svg_data = "<svg></svg>";
        rasterizer.add_svg(svg_data, 64);

        // Should have some utilization now
        assert!(rasterizer.utilization() > 0.0);
        assert!(rasterizer.utilization() < 1.0);
    }

    #[test]
    fn test_clear() {
        let mut rasterizer = SvgRasterizer::new(1024, 1024);
        let svg_data = "<svg></svg>";

        rasterizer.add_svg(svg_data, 64);
        assert!(rasterizer.utilization() > 0.0);

        rasterizer.clear();
        assert_eq!(rasterizer.utilization(), 0.0);
    }

    #[test]
    fn test_atlas_packer_allocation() {
        let mut packer = AtlasPacker::new(1024, 1024);

        let rect1 = packer.allocate(64, 64);
        assert!(rect1.is_some());
        assert_eq!(rect1.unwrap().w, 64);

        let rect2 = packer.allocate(128, 128);
        assert!(rect2.is_some());
        assert_eq!(rect2.unwrap().w, 128);
    }

    #[test]
    fn test_atlas_packer_overflow() {
        let mut packer = AtlasPacker::new(100, 100);

        // This should fail - too big
        let rect = packer.allocate(200, 200);
        assert!(rect.is_none());
    }

    #[test]
    fn test_atlas_packer_utilization() {
        let mut packer = AtlasPacker::new(1024, 1024);
        assert_eq!(packer.utilization(), 0.0);

        packer.allocate(512, 64);
        assert!(packer.utilization() > 0.0);
    }

    #[test]
    fn test_packed_rect() {
        let rect = PackedRect {
            x: 10,
            y: 20,
            w: 64,
            h: 64,
        };

        assert_eq!(rect.x, 10);
        assert_eq!(rect.y, 20);
        assert_eq!(rect.w, 64);
        assert_eq!(rect.h, 64);
    }

    #[test]
    fn test_shelf_creation() {
        let shelf = Shelf {
            y_start: 100,
            height: 64,
            current_x: 128,
        };

        assert_eq!(shelf.y_start, 100);
        assert_eq!(shelf.height, 64);
        assert_eq!(shelf.current_x, 128);
    }

    #[test]
    fn test_render_svg_stub() {
        let rasterizer = SvgRasterizer::new(256, 256);
        let pixels = rasterizer.render_svg_stub("<svg></svg>", 64);

        // Should have RGBA data
        assert_eq!(pixels.len(), (64 * 64 * 4) as usize);
    }
}
