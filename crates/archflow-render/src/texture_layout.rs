// WebGL2 Texture Alignment Utility
//
// This module provides utilities for calculating and managing texture memory alignment
// in WebGL2. While WebGL2 doesn't have the strict 256-byte alignment requirements of
// WebGPU's TexelCopyBufferLayout, it still has row alignment constraints based on
// GL_UNPACK_ALIGNMENT that must be respected for proper texture loading.
//
// Key concepts:
// - Row alignment: Each row of texture data must be aligned to 1, 2, 4, or 8 bytes
// - Row padding: May need to add padding bytes to the end of each row
// - Pixel size: Depends on the texture format (e.g., RGBA8 = 4 bytes/pixel)

use crate::error::{RenderError, RenderErrorKind};
use alloc::format;
use alloc::vec::Vec;

/// Pixel format used for texture data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// 8-bit red channel (1 byte/pixel)
    R8,
    /// 8-bit red and green channels (2 bytes/pixel)
    RG8,
    /// 8-bit red, green, and blue channels (3 bytes/pixel)
    RGB8,
    /// 8-bit RGBA (4 bytes/pixel)
    RGBA8,
    /// 16-bit floating point red (2 bytes/pixel)
    R16F,
    /// 16-bit floating point RG (4 bytes/pixel)
    RG16F,
    /// 16-bit floating point RGB (6 bytes/pixel)
    RGB16F,
    /// 16-bit floating point RGBA (8 bytes/pixel)
    RGBA16F,
    /// 32-bit floating point red (4 bytes/pixel)
    R32F,
    /// 32-bit floating point RG (8 bytes/pixel)
    RG32F,
    /// 32-bit floating point RGB (12 bytes/pixel)
    RGB32F,
    /// 32-bit floating point RGBA (16 bytes/pixel)
    RGBA32F,
}

impl PixelFormat {
    /// Returns the size of a single pixel in bytes
    pub fn pixel_size(&self) -> usize {
        match self {
            PixelFormat::R8 => 1,
            PixelFormat::RG8 => 2,
            PixelFormat::RGB8 => 3,
            PixelFormat::RGBA8 => 4,
            PixelFormat::R16F => 2,
            PixelFormat::RG16F => 4,
            PixelFormat::RGB16F => 6,
            PixelFormat::RGBA16F => 8,
            PixelFormat::R32F => 4,
            PixelFormat::RG32F => 8,
            PixelFormat::RGB32F => 12,
            PixelFormat::RGBA32F => 16,
        }
    }
}

/// Valid alignment values for WebGL2 pixel operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    One = 1,
    Two = 2,
    Four = 4,
    Eight = 8,
}

impl Alignment {
    /// Create from a raw value, validating it's 1, 2, 4, or 8
    pub fn new(value: i32) -> Result<Self, RenderError> {
        match value {
            1 => Ok(Alignment::One),
            2 => Ok(Alignment::Two),
            4 => Ok(Alignment::Four),
            8 => Ok(Alignment::Eight),
            _ => Err(RenderError::new(
                RenderErrorKind::InvalidTextureData,
                "Invalid alignment value: must be 1, 2, 4, or 8",
            )),
        }
    }
}

/// Layout information for texture data in a buffer
///
/// This structure calculates the proper row padding and alignment for texture
/// data to ensure it meets WebGL2's unpack alignment requirements.
///
/// While similar in concept to WebGPU's TexelCopyBufferLayout, WebGL2 has
/// different alignment constraints:
/// - WebGPU: bytes_per_row must be multiple of 256
/// - WebGL2: row bytes must be multiple of UNPACK_ALIGNMENT (1, 2, 4, or 8)
#[derive(Debug, Clone, Copy)]
pub struct TextureLayout {
    /// Size of a single pixel in bytes
    pub pixel_size: usize,
    /// Width of the texture in pixels
    pub width: usize,
    /// Height of the texture in pixels
    pub height: usize,
    /// Required row alignment (1, 2, 4, or 8)
    pub alignment: Alignment,
    /// Total bytes per row including padding
    pub bytes_per_row: usize,
    /// Total size of the texture data in bytes
    pub total_size: usize,
}

impl TextureLayout {
    /// Calculate the layout for texture data
    ///
    /// # Arguments
    /// * `format` - Pixel format of the texture
    /// * `width` - Width in pixels
    /// * `height` - Height in pixels
    /// * `alignment` - Required row alignment (default is 4)
    ///
    /// # Returns
    /// Calculated layout information
    pub fn new(format: PixelFormat, width: usize, height: usize, alignment: Alignment) -> Self {
        let pixel_size = format.pixel_size();
        let bytes_per_row = calculate_aligned_row_size(pixel_size, width, alignment as usize);
        let total_size = bytes_per_row * height;

        Self {
            pixel_size,
            width,
            height,
            alignment,
            bytes_per_row,
            total_size,
        }
    }

    /// Create with default alignment of 4 (WebGL2 default)
    pub fn with_default_alignment(format: PixelFormat, width: usize, height: usize) -> Self {
        Self::new(format, width, height, Alignment::Four)
    }

    /// Calculate the padding bytes at the end of each row
    pub fn padding_per_row(&self) -> usize {
        self.bytes_per_row - (self.pixel_size * self.width)
    }

    /// Calculate the offset to a specific row
    pub fn row_offset(&self, row: usize) -> usize {
        row * self.bytes_per_row
    }

    /// Calculate the offset to a specific pixel
    pub fn pixel_offset(&self, x: usize, y: usize) -> Result<usize, RenderError> {
        if x >= self.width || y >= self.height {
            return Err(RenderError::new(
                RenderErrorKind::InvalidTextureData,
                "Pixel coordinates out of bounds",
            ));
        }
        Ok(y * self.bytes_per_row + x * self.pixel_size)
    }

    /// Check if the data size matches the expected size for this layout
    pub fn validate_data_size(&self, data_size: usize) -> bool {
        data_size >= self.total_size
    }
}

/// Calculate the aligned size of a row in bytes
///
/// This ensures that each row of texture data is properly aligned according
/// to the specified alignment value.
fn calculate_aligned_row_size(pixel_size: usize, width: usize, alignment: usize) -> usize {
    let unaligned_size = pixel_size * width;
    let remainder = unaligned_size % alignment;
    if remainder == 0 {
        unaligned_size
    } else {
        unaligned_size + (alignment - remainder)
    }
}

/// Calculate the optimal alignment for a given pixel format and width
///
/// This determines the smallest valid alignment that requires no padding,
/// which is most memory-efficient.
pub fn calculate_optimal_alignment(format: PixelFormat, width: usize) -> Alignment {
    let pixel_size = format.pixel_size();
    let row_size = pixel_size * width;

    // Any row_size is always divisible by 1, which requires no padding - this is optimal
    // Check larger alignments only if row_size is NOT divisible by smaller ones
    // (but we skip the % 1 check since it's always true and triggers a clippy warning)
    if row_size.is_multiple_of(4) && !row_size.is_multiple_of(8) {
        return Alignment::Four;
    }
    if row_size.is_multiple_of(2) && !row_size.is_multiple_of(4) {
        return Alignment::Two;
    }
    // Default to One (most optimal - no padding needed for any row_size)
    Alignment::One
}

/// Pad texture data to meet alignment requirements
///
/// Takes raw pixel data and adds padding bytes to the end of each row
/// to ensure proper alignment for WebGL2 texture upload.
///
/// # Arguments
/// * `data` - Raw pixel data
/// * `format` - Pixel format
/// * `width` - Width in pixels
/// * `height` - Height in pixels
/// * `alignment` - Required row alignment
///
/// # Returns
/// Padded data as a Vec<u8>
pub fn pad_texture_data(
    data: &[u8],
    format: PixelFormat,
    width: usize,
    height: usize,
    alignment: Alignment,
) -> Result<Vec<u8>, RenderError> {
    let layout = TextureLayout::new(format, width, height, alignment);
    let pixel_size = format.pixel_size();
    let expected_unpadded_size = pixel_size * width * height;

    if data.len() != expected_unpadded_size {
        return Err(RenderError::new(
            RenderErrorKind::InvalidTextureData,
            &format!(
                "Data size mismatch: expected {}, got {}",
                expected_unpadded_size,
                data.len()
            ),
        ));
    }

    let padding = layout.padding_per_row();
    let mut padded_data = Vec::with_capacity(layout.total_size);

    for row in 0..height {
        let row_start = row * width * pixel_size;
        let row_end = row_start + width * pixel_size;
        padded_data.extend_from_slice(&data[row_start..row_end]);

        // Add padding to the end of the row
        if padding > 0 {
            padded_data.extend(core::iter::repeat(0).take(padding));
        }
    }

    Ok(padded_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_pixel_format_sizes() {
        assert_eq!(PixelFormat::R8.pixel_size(), 1);
        assert_eq!(PixelFormat::RG8.pixel_size(), 2);
        assert_eq!(PixelFormat::RGB8.pixel_size(), 3);
        assert_eq!(PixelFormat::RGBA8.pixel_size(), 4);
        assert_eq!(PixelFormat::RGBA16F.pixel_size(), 8);
        assert_eq!(PixelFormat::RGBA32F.pixel_size(), 16);
    }

    #[test]
    fn test_alignment_validation() {
        assert!(Alignment::new(1).is_ok());
        assert!(Alignment::new(2).is_ok());
        assert!(Alignment::new(4).is_ok());
        assert!(Alignment::new(8).is_ok());
        assert!(Alignment::new(3).is_err());
        assert!(Alignment::new(5).is_err());
        assert!(Alignment::new(16).is_err());
    }

    #[test]
    fn test_texture_layout_rgba8_no_padding() {
        let layout = TextureLayout::new(PixelFormat::RGBA8, 4, 4, Alignment::Four);

        assert_eq!(layout.pixel_size, 4);
        assert_eq!(layout.width, 4);
        assert_eq!(layout.height, 4);
        // 4 pixels * 4 bytes = 16, which is aligned to 4
        assert_eq!(layout.bytes_per_row, 16);
        assert_eq!(layout.total_size, 64);
        assert_eq!(layout.padding_per_row(), 0);
    }

    #[test]
    fn test_texture_layout_rgb8_with_padding() {
        let layout = TextureLayout::new(PixelFormat::RGB8, 3, 4, Alignment::Four);

        assert_eq!(layout.pixel_size, 3);
        assert_eq!(layout.width, 3);
        assert_eq!(layout.height, 4);
        // 3 pixels * 3 bytes = 9, need 1 byte padding to align to 4
        assert_eq!(layout.bytes_per_row, 12);
        assert_eq!(layout.total_size, 48);
        assert_eq!(layout.padding_per_row(), 3);
    }

    #[test]
    fn test_texture_layout_r8_with_alignment_4() {
        let layout = TextureLayout::new(PixelFormat::R8, 5, 3, Alignment::Four);

        assert_eq!(layout.pixel_size, 1);
        assert_eq!(layout.width, 5);
        assert_eq!(layout.height, 3);
        // 5 pixels * 1 byte = 5, need 3 bytes padding to align to 4
        assert_eq!(layout.bytes_per_row, 8);
        assert_eq!(layout.total_size, 24);
        assert_eq!(layout.padding_per_row(), 3);
    }

    #[test]
    fn test_texture_layout_with_alignment_1() {
        let layout = TextureLayout::new(PixelFormat::RGB8, 7, 5, Alignment::One);

        // With alignment 1, no padding needed
        assert_eq!(layout.bytes_per_row, 21);
        assert_eq!(layout.total_size, 105);
        assert_eq!(layout.padding_per_row(), 0);
    }

    #[test]
    fn test_row_offset() {
        let layout = TextureLayout::new(PixelFormat::RGBA8, 10, 10, Alignment::Four);

        assert_eq!(layout.row_offset(0), 0);
        assert_eq!(layout.row_offset(1), 40);
        assert_eq!(layout.row_offset(5), 200);
    }

    #[test]
    fn test_pixel_offset() {
        let layout = TextureLayout::new(PixelFormat::RGBA8, 10, 10, Alignment::Four);

        assert_eq!(layout.pixel_offset(0, 0).unwrap(), 0);
        assert_eq!(layout.pixel_offset(1, 0).unwrap(), 4);
        assert_eq!(layout.pixel_offset(0, 1).unwrap(), 40);
        // 3 * 40 (row 3 offset) + 5 * 4 (pixel 5 in row) = 120 + 20 = 140
        assert_eq!(layout.pixel_offset(5, 3).unwrap(), 140);
    }

    #[test]
    fn test_pixel_offset_out_of_bounds() {
        let layout = TextureLayout::new(PixelFormat::RGBA8, 10, 10, Alignment::Four);

        assert!(layout.pixel_offset(10, 0).is_err());
        assert!(layout.pixel_offset(0, 10).is_err());
        assert!(layout.pixel_offset(15, 5).is_err());
    }

    #[test]
    fn test_calculate_optimal_alignment() {
        // RGBA8 width 4: 4*4 = 16, optimal is 1
        assert_eq!(
            calculate_optimal_alignment(PixelFormat::RGBA8, 4),
            Alignment::One
        );

        // RGB8 width 3: 3*3 = 9, optimal is 1
        assert_eq!(
            calculate_optimal_alignment(PixelFormat::RGB8, 3),
            Alignment::One
        );

        // R8 width 5: 1*5 = 5, optimal is 1
        assert_eq!(
            calculate_optimal_alignment(PixelFormat::R8, 5),
            Alignment::One
        );
    }

    #[test]
    fn test_pad_texture_data_no_padding() {
        let data = vec![255u8; 64]; // 4x4 RGBA8, no padding needed
        let padded = pad_texture_data(&data, PixelFormat::RGBA8, 4, 4, Alignment::Four);

        assert!(padded.is_ok());
        let padded = padded.unwrap();
        assert_eq!(padded.len(), 64);
    }

    #[test]
    fn test_pad_texture_data_with_padding() {
        // 3x3 RGB8 = 27 bytes, with alignment 4 need 3 bytes padding per row = 36 total
        let data = vec![100u8; 27];
        let padded = pad_texture_data(&data, PixelFormat::RGB8, 3, 3, Alignment::Four);

        assert!(padded.is_ok());
        let padded = padded.unwrap();
        assert_eq!(padded.len(), 36);

        // Check that padding bytes are zero
        for row in 0..3 {
            let padding_start = row * 12 + 9;
            assert_eq!(padded[padding_start], 0);
            assert_eq!(padded[padding_start + 1], 0);
            assert_eq!(padded[padding_start + 2], 0);
        }
    }

    #[test]
    fn test_pad_texture_data_invalid_size() {
        let data = vec![0u8; 10];
        let result = pad_texture_data(&data, PixelFormat::RGBA8, 4, 4, Alignment::Four);

        assert!(result.is_err());
    }

    #[test]
    fn test_validate_data_size() {
        let layout = TextureLayout::new(PixelFormat::RGBA8, 4, 4, Alignment::Four);

        assert!(layout.validate_data_size(64));
        assert!(layout.validate_data_size(128));
        assert!(!layout.validate_data_size(32));
    }
}
