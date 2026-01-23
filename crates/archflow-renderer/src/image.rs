//! Image data implementation for the renderer

use super::{Image, PixelFormat};

/// Implementación simple de Image desde datos crudos
#[derive(Debug, Clone)]
pub struct ImageData {
    width: u32,
    height: u32,
    data: Vec<u8>,
    pixel_format: PixelFormat,
}

impl ImageData {
    /// Crear una nueva imagen desde datos RGBA
    pub fn new_rgba(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            width,
            height,
            data,
            pixel_format: PixelFormat::Rgba8,
        }
    }

    /// Crear una nueva imagen desde datos RGB
    pub fn new_rgb(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            width,
            height,
            data,
            pixel_format: PixelFormat::Rgb8,
        }
    }

    /// Crear una imagen placeholder
    pub fn placeholder(width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        let pixel = [128, 128, 128, 255];
        let data: Vec<u8> = pixel.iter().cycle().take(size).cloned().collect();
        Self {
            width,
            height,
            data,
            pixel_format: PixelFormat::Rgba8,
        }
    }

    /// Crear una imagen con color sólido
    pub fn solid_color(width: u32, height: u32, color: [u8; 4]) -> Self {
        let size = (width * height * 4) as usize;
        let mut data = Vec::with_capacity(size);
        for _ in 0..(width * height) {
            data.extend_from_slice(&color);
        }
        Self {
            width,
            height,
            data,
            pixel_format: PixelFormat::Rgba8,
        }
    }
}

impl Image for ImageData {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn data(&self) -> &[u8] {
        &self.data
    }

    fn pixel_format(&self) -> PixelFormat {
        self.pixel_format
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_data_creation() {
        let image = ImageData::placeholder(100, 100);
        assert_eq!(image.width(), 100);
        assert_eq!(image.height(), 100);
        assert_eq!(image.data().len(), 100 * 100 * 4);
    }

    #[test]
    fn test_solid_color_image() {
        let image = ImageData::solid_color(10, 10, [255, 0, 0, 255]);
        assert_eq!(image.width(), 10);
        assert_eq!(image.height(), 10);
        // Verify first pixel is red
        assert_eq!(image.data()[0], 255);
        assert_eq!(image.data()[1], 0);
        assert_eq!(image.data()[2], 0);
        assert_eq!(image.data()[3], 255);
    }
}
