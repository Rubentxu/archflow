// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Render - WGSL Shaders
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 9
//
// Multi-phase rendering shaders for WebGPU:
// - SDF-based shapes (rectangles, circles, lines)
// - Icon texture atlas
// - Image texture array
// - MTSDF text rendering
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]
#![warn(clippy::all)]

/// SDF Shapes shader for rendering rectangles, circles, ellipses, and lines
#[cfg(not(feature = "webgl2"))]
pub const SHADER_SDF_SHAPES: &str = include_str!("shaders/sdf_shapes.wgsl");

/// Icon texture shader for rendering from texture atlas
#[cfg(not(feature = "webgl2"))]
pub const SHADER_ICON_TEXTURE: &str = include_str!("shaders/icon_texture.wgsl");

/// Image array shader for rendering from texture2D array
#[cfg(not(feature = "webgl2"))]
pub const SHADER_IMAGE_ARRAY: &str = include_str!("shaders/image_array.wgsl");

/// MTSDF text shader for crisp text rendering at any size
#[cfg(not(feature = "webgl2"))]
pub const SHADER_MTSDF_TEXT: &str = include_str!("shaders/mtsdf_text.wgsl");

/// GLSL SDF Shapes shader (compiled by build.rs from WGSL)
#[cfg(feature = "webgl2")]
pub const SHADER_SDF_SHAPES: &str = include_str!(concat!(env!("OUT_DIR"), "/sdf_shapes.glsl"));

/// GLSL Icon texture shader (compiled by build.rs from WGSL)
#[cfg(feature = "webgl2")]
pub const SHADER_ICON_TEXTURE: &str = include_str!(concat!(env!("OUT_DIR"), "/icon_texture.glsl"));

/// GLSL Image array shader (compiled by build.rs from WGSL)
#[cfg(feature = "webgl2")]
pub const SHADER_IMAGE_ARRAY: &str = include_str!(concat!(env!("OUT_DIR"), "/image_array.glsl"));

/// GLSL MTSDF text shader (compiled by build.rs from WGSL)
#[cfg(feature = "webgl2")]
pub const SHADER_MTSDF_TEXT: &str = include_str!(concat!(env!("OUT_DIR"), "/mtsdf_text.glsl"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_sdf_shapes_exists() {
        assert!(!SHADER_SDF_SHAPES.is_empty());
        assert!(SHADER_SDF_SHAPES.contains("@vertex"));
        assert!(SHADER_SDF_SHAPES.contains("@fragment"));
    }

    #[test]
    fn test_shader_icon_texture_exists() {
        assert!(!SHADER_ICON_TEXTURE.is_empty());
        assert!(SHADER_ICON_TEXTURE.contains("@vertex"));
        assert!(SHADER_ICON_TEXTURE.contains("@fragment"));
    }

    #[test]
    fn test_shader_image_array_exists() {
        assert!(!SHADER_IMAGE_ARRAY.is_empty());
        assert!(SHADER_IMAGE_ARRAY.contains("@vertex"));
        assert!(SHADER_IMAGE_ARRAY.contains("@fragment"));
    }

    #[test]
    fn test_shader_mtsdf_text_exists() {
        assert!(!SHADER_MTSDF_TEXT.is_empty());
        assert!(SHADER_MTSDF_TEXT.contains("@vertex"));
        assert!(SHADER_MTSDF_TEXT.contains("@fragment"));
    }

    #[test]
    fn test_shader_sdf_shapes_contains_uniforms() {
        assert!(SHADER_SDF_SHAPES.contains("CameraUniforms"));
        assert!(SHADER_SDF_SHAPES.contains("view_projection"));
    }

    #[test]
    fn test_shader_mtsdf_contains_median_function() {
        assert!(SHADER_MTSDF_TEXT.contains("median"));
        assert!(SHADER_MTSDF_TEXT.contains("msdf"));
    }
}
