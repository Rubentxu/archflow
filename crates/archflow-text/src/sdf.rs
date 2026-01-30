//! SDF (Signed Distance Field) generation and atlas management.
//!
//! # Foundation Implementation
//!
//! This module provides the structure for SDF texture generation.
//! Full SDF algorithms will be implemented when needed for production.

use std::collections::HashMap;

/// SDF generator for creating distance fields from glyphs.
pub struct SDFGenerator {
    /// Edge spread for SDF generation.
    spread: f32,
}

impl SDFGenerator {
    /// Creates a new SDF generator.
    pub fn new() -> Self {
        Self { spread: 4.0 }
    }

    /// Sets the edge spread for SDF generation.
    pub fn with_spread(mut self, spread: f32) -> Self {
        self.spread = spread;
        self
    }

    /// Generates SDF for a glyph outline.
    ///
    /// # Foundation Implementation
    ///
    /// Returns a placeholder SDF texture. Full SDF generation
    /// will implement the distance field algorithm when needed.
    pub fn generate_sdf(&self, _width: u32, _height: u32) -> SDFTexture {
        // Foundation: Return empty SDF texture
        // Full implementation will compute actual distance fields
        SDFTexture::new(64, 64)
    }
}

impl Default for SDFGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// SDF texture containing distance field data.
#[derive(Debug, Clone)]
pub struct SDFTexture {
    /// Width in pixels.
    pub width: u32,

    /// Height in pixels.
    pub height: u32,

    /// SDF data (normalized to [0, 1]).
    pub data: Vec<f32>,
}

impl SDFTexture {
    /// Creates a new SDF texture.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0.0; (width * height) as usize],
        }
    }

    /// Gets the SDF value at a specific pixel.
    pub fn get_pixel(&self, x: u32, y: u32) -> f32 {
        if x >= self.width || y >= self.height {
            return 0.0;
        }
        self.data[(y * self.width + x) as usize]
    }

    /// Sets the SDF value at a specific pixel.
    pub fn set_pixel(&mut self, x: u32, y: u32, value: f32) {
        if x < self.width && y < self.height {
            self.data[(y * self.width + x) as usize] = value;
        }
    }
}

/// SDF atlas containing multiple glyph textures.
pub struct SDFAtlas {
    /// Atlas texture width.
    pub width: u32,

    /// Atlas texture height.
    pub height: u32,

    /// Glyph-to-UV mapping.
    glyphs: HashMap<(char, u32), GlyphUV>,
}

/// UV rectangle for a glyph in the atlas.
#[derive(Debug, Clone, Copy)]
pub struct GlyphUV {
    /// X position in atlas.
    pub x: u32,

    /// Y position in atlas.
    pub y: u32,

    /// Width of glyph.
    pub width: u32,

    /// Height of glyph.
    pub height: u32,
}

impl SDFAtlas {
    /// Creates a new SDF atlas.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            glyphs: HashMap::new(),
        }
    }

    /// Gets the UV rectangle for a glyph.
    pub fn get_glyph_uv(&self, char: char, font_size: u32) -> Option<&GlyphUV> {
        self.glyphs.get(&(char, font_size))
    }

    /// Inserts a glyph into the atlas.
    pub fn insert_glyph(&mut self, char: char, font_size: u32, uv: GlyphUV) {
        self.glyphs.insert((char, font_size), uv);
    }

    /// Returns the number of cached glyphs.
    pub fn len(&self) -> usize {
        self.glyphs.len()
    }
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdf_generator_creation() {
        let generator = SDFGenerator::new();
        assert_eq!(generator.spread, 4.0);
    }

    #[test]
    fn test_sdf_generator_with_spread() {
        let generator = SDFGenerator::new().with_spread(8.0);
        assert_eq!(generator.spread, 8.0);
    }

    #[test]
    fn test_sdf_texture_creation() {
        let texture = SDFTexture::new(64, 64);
        assert_eq!(texture.width, 64);
        assert_eq!(texture.height, 64);
        assert_eq!(texture.data.len(), 4096);
    }

    #[test]
    fn test_sdf_texture_pixel_access() {
        let mut texture = SDFTexture::new(64, 64);

        texture.set_pixel(10, 20, 0.5);
        assert_eq!(texture.get_pixel(10, 20), 0.5);

        assert_eq!(texture.get_pixel(100, 100), 0.0); // Out of bounds
    }

    #[test]
    fn test_sdf_atlas_creation() {
        let atlas = SDFAtlas::new(2048, 2048);
        assert_eq!(atlas.width, 2048);
        assert_eq!(atlas.height, 2048);
    }

    #[test]
    fn test_sdf_atlas_glyph_lookup() {
        let mut atlas = SDFAtlas::new(2048, 2048);

        let uv = GlyphUV {
            x: 10,
            y: 20,
            width: 32,
            height: 32,
        };

        atlas.insert_glyph('A', 16, uv);

        assert!(atlas.get_glyph_uv('A', 16).is_some());
        assert!(atlas.get_glyph_uv('Z', 16).is_none());
    }

    #[test]
    fn test_sdf_atlas_length() {
        let mut atlas = SDFAtlas::new(2048, 2048);

        assert_eq!(atlas.len(), 0);

        atlas.insert_glyph(
            'A',
            16,
            GlyphUV {
                x: 0,
                y: 0,
                width: 32,
                height: 32,
            },
        );

        assert_eq!(atlas.len(), 1);
    }
}
