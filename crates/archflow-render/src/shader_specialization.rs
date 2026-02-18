// ═══════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Render - Shader Specialization Constants
//
// Provides compile-time and runtime configuration for shaders across backends:
// - WebGPU: Uses WGSL override directives for dynamic configuration
// - WebGL2: Uses pre-processor defines via Naga compilation
//
// This enables optimal shaders per backend while sharing shader logic.
// ═══════════════════════════════════════════════════════════════════════════════════════

use alloc::vec::Vec;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

/// Configuration constants for shader compilation and runtime behavior.
///
/// These constants control which features are enabled in shaders for each backend.
/// WebGPU uses WGSL `override` directives that can be set at pipeline creation.
/// WebGL2 uses GLSL pre-processor defines compiled via Naga.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShaderConstants {
    /// Maximum number of lights in the scene (0 = no lighting)
    /// WebGPU: Can use override for dynamic values
    /// WebGL2: Fixed at compile time via define
    pub max_lights: u32,

    /// Enable shadow mapping (requires max_lights > 0)
    /// Shadows are expensive in WebGL2, disabled by default
    pub enable_shadows: bool,

    /// Enable anti-aliasing (MSAA or equivalent)
    /// Can be disabled for performance on low-end devices
    pub enable_aa: bool,

    /// Enable text rendering with MTSDF
    /// Can be disabled if text is not needed (reduces shader complexity)
    pub enable_text: bool,

    /// Enable texture atlas sampling
    /// Disable if only solid colors are used
    pub enable_textures: bool,

    /// Enable instanced rendering
    /// Should always be true for performance with many entities
    pub enable_instancing: bool,

    /// Atlas texture size (1024, 2048, 4096)
    /// Affects shader array sizes
    pub atlas_size: u32,

    /// Maximum number of instances per draw call
    /// WebGPU: Can be larger (64K+)
    /// WebGL2: Limited by MAX_VERTEX_ATTRIBS and implementation
    pub max_instances: u32,

    /// Enable debug visualizations (wireframes, bounds, etc.)
    /// Should be false in production
    pub enable_debug: bool,
}

impl Default for ShaderConstants {
    fn default() -> Self {
        Self {
            max_lights: 0,
            enable_shadows: false,
            enable_aa: true,
            enable_text: true,
            enable_textures: true,
            enable_instancing: true,
            atlas_size: 1024,
            max_instances: 100_000,
            enable_debug: false,
        }
    }
}

/// Predefined shader configurations for different backends and use cases.
impl ShaderConstants {
    /// WebGPU configuration with all features enabled.
    /// Optimized for performance with advanced effects.
    pub fn webgpu_full() -> Self {
        Self {
            max_lights: 4,
            enable_shadows: true,
            enable_aa: true,
            enable_text: true,
            enable_textures: true,
            enable_instancing: true,
            atlas_size: 2048,
            max_instances: 100_000,
            enable_debug: false,
        }
    }

    /// WebGPU configuration for high-performance scenarios.
    /// Disables expensive features like shadows.
    pub fn webgpu_performance() -> Self {
        Self {
            max_lights: 0,
            enable_shadows: false,
            enable_aa: true,
            enable_text: true,
            enable_textures: true,
            enable_instancing: true,
            atlas_size: 2048,
            max_instances: 100_000,
            enable_debug: false,
        }
    }

    /// WebGL2 configuration - simplified for compatibility.
    /// Disables expensive features that may not work well.
    pub fn webgl2() -> Self {
        Self {
            max_lights: 0,
            enable_shadows: false,
            enable_aa: true,
            enable_text: true,
            enable_textures: true,
            enable_instancing: true,
            atlas_size: 1024,
            max_instances: 10_000, // WebGL2 has lower limits
            enable_debug: false,
        }
    }

    /// Development configuration with debug features.
    pub fn debug() -> Self {
        Self {
            enable_debug: true,
            ..Default::default()
        }
    }

    /// Generate WGSL override declarations from this configuration.
    ///
    /// These are inserted at the top of WGSL shader files.
    /// Values can be overridden at pipeline creation time.
    pub fn to_wgsl_overrides(&self) -> alloc::string::String {
        let mut overrides = alloc::string::String::new();

        // Required: we must provide defaults for all overrides used in shaders
        overrides.push_str("// Shader specialization constants (WGSL override directives)\n");
        overrides.push_str(&alloc::format!(
            "override MAX_LIGHTS: u32 = {};\n",
            self.max_lights
        ));
        overrides.push_str(&alloc::format!(
            "override ENABLE_SHADOWS: bool = {};\n",
            if self.enable_shadows { "true" } else { "false" }
        ));
        overrides.push_str(&alloc::format!(
            "override ENABLE_AA: bool = {};\n",
            if self.enable_aa { "true" } else { "false" }
        ));
        overrides.push_str(&alloc::format!(
            "override ENABLE_TEXT: bool = {};\n",
            if self.enable_text { "true" } else { "false" }
        ));
        overrides.push_str(&alloc::format!(
            "override ENABLE_TEXTURES: bool = {};\n",
            if self.enable_textures {
                "true"
            } else {
                "false"
            }
        ));
        overrides.push_str(&alloc::format!(
            "override ENABLE_INSTANCING: bool = {};\n",
            if self.enable_instancing {
                "true"
            } else {
                "false"
            }
        ));
        overrides.push_str(&alloc::format!(
            "override ATLAS_SIZE: u32 = {};\n",
            self.atlas_size
        ));
        overrides.push_str(&alloc::format!(
            "override MAX_INSTANCES: u32 = {};\n",
            self.max_instances
        ));
        overrides.push_str(&alloc::format!(
            "override ENABLE_DEBUG: bool = {};\n",
            if self.enable_debug { "true" } else { "false" }
        ));

        overrides
    }

    /// Generate GLSL pre-processor defines from this configuration.
    ///
    /// These are used when compiling WGSL to GLSL via Naga.
    pub fn to_glsl_defines(&self) -> Vec<(alloc::string::String, alloc::string::String)> {
        let mut defines: Vec<(alloc::string::String, alloc::string::String)> = Vec::new();

        defines.push(("MAX_LIGHTS".into(), alloc::format!("{}", self.max_lights)));
        defines.push((
            "ENABLE_SHADOWS".into(),
            if self.enable_shadows {
                "1".into()
            } else {
                "0".into()
            },
        ));
        defines.push((
            "ENABLE_AA".into(),
            if self.enable_aa {
                "1".into()
            } else {
                "0".into()
            },
        ));
        defines.push((
            "ENABLE_TEXT".into(),
            if self.enable_text {
                "1".into()
            } else {
                "0".into()
            },
        ));
        defines.push((
            "ENABLE_TEXTURES".into(),
            if self.enable_textures {
                "1".into()
            } else {
                "0".into()
            },
        ));
        defines.push((
            "ENABLE_INSTANCING".into(),
            if self.enable_instancing {
                "1".into()
            } else {
                "0".into()
            },
        ));
        defines.push(("ATLAS_SIZE".into(), alloc::format!("{}", self.atlas_size)));
        defines.push((
            "MAX_INSTANCES".into(),
            alloc::format!("{}", self.max_instances),
        ));
        defines.push((
            "ENABLE_DEBUG".into(),
            if self.enable_debug {
                "1".into()
            } else {
                "0".into()
            },
        ));

        defines
    }
}

/// Backend-specific pipeline compilation options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PipelineOptions {
    /// Which backend this configuration is for
    pub backend: BackendType,

    /// Shader constants for this backend
    pub constants: ShaderConstants,

    /// Enable early Z-tests for better performance
    pub early_z: bool,

    /// Enable conservative rasterization (if supported)
    pub conservative_raster: bool,
}

/// Backend type for pipeline configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    /// WebGPU via wgpu
    WebGpu,
    /// WebGL2 via glow
    WebGl2,
    /// CPU fallback (Canvas 2D)
    Cpu,
}

impl PipelineOptions {
    /// Create WebGPU pipeline options.
    pub fn webgpu(constants: ShaderConstants) -> Self {
        Self {
            backend: BackendType::WebGpu,
            constants,
            early_z: true,
            conservative_raster: false,
        }
    }

    /// Create WebGL2 pipeline options.
    pub fn webgl2() -> Self {
        Self {
            backend: BackendType::WebGl2,
            constants: ShaderConstants::webgl2(),
            early_z: true,
            conservative_raster: false,
        }
    }

    /// Create CPU/Canvas2D pipeline options.
    pub fn cpu() -> Self {
        Self {
            backend: BackendType::Cpu,
            constants: ShaderConstants::default(),
            early_z: false,
            conservative_raster: false,
        }
    }
}

/// Feature flags that can be queried at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeatureFlags {
    /// Supports WebGL2
    pub webgl2_available: bool,
    /// Supports float32 textures
    pub float_textures: bool,
    /// Supports instanced rendering
    pub instancing: bool,
    /// Maximum texture size
    pub max_texture_size: u32,
    /// Maximum vertex attribs
    pub max_vertex_attribs: u32,
}

impl FeatureFlags {
    /// Detect available features from browser capabilities.
    #[cfg(target_arch = "wasm32")]
    pub fn detect() -> Self {
        // WebGPU detection requires web-sys 0.4+ which has Gpu feature
        // For now, we detect WebGL2 and assume WebGPU is not available
        // This will be updated when web-sys 0.4+ is available
        let webgl2_available = {
            // Check for WebGL2 support
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Ok(canvas) = document
                        .create_element("canvas")
                        .map_err(|_| ())
                        .and_then(|el| el.dyn_into::<web_sys::HtmlCanvasElement>().map_err(|_| ()))
                    {
                        // get_context returns Result<Option<Object>, JsValue>
                        canvas.get_context("webgl2").ok().flatten().is_some()
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        };

        Self {
            webgl2_available,
            float_textures: true, // Most browsers support this
            instancing: true,     // WebGL2 supports instancing
            max_texture_size: 4096,
            max_vertex_attribs: 16,
        }
    }

    /// Create feature flags for native/testing.
    pub fn native() -> Self {
        Self {
            webgl2_available: true,
            float_textures: true,
            instancing: true,
            max_texture_size: 8192,
            max_vertex_attribs: 32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_constants_default() {
        let constants = ShaderConstants::default();
        assert_eq!(constants.max_lights, 0);
        assert!(!constants.enable_shadows);
        assert!(constants.enable_aa);
    }

    #[test]
    fn test_webgpu_config() {
        let constants = ShaderConstants::webgpu_full();
        assert_eq!(constants.max_lights, 4);
        assert!(constants.enable_shadows);
        assert_eq!(constants.atlas_size, 2048);
    }

    #[test]
    fn test_webgl2_config() {
        let constants = ShaderConstants::webgl2();
        assert_eq!(constants.max_lights, 0);
        assert!(!constants.enable_shadows);
        assert_eq!(constants.max_instances, 10_000);
    }

    #[test]
    fn test_wgsl_overrides_format() {
        let constants = ShaderConstants::webgpu_full();
        let overrides = constants.to_wgsl_overrides();

        assert!(overrides.contains("override MAX_LIGHTS: u32 = 4;"));
        assert!(overrides.contains("override ENABLE_SHADOWS: bool = true;"));
        assert!(overrides.contains("override ATLAS_SIZE: u32 = 2048;"));
    }

    #[test]
    fn test_glsl_defines_format() {
        let constants = ShaderConstants::webgl2();
        let defines = constants.to_glsl_defines();

        let max_lights = defines.iter().find(|(name, _)| name == "MAX_LIGHTS");
        assert_eq!(max_lights.map(|(_, v)| v.as_str()), Some("0"));

        let enable_shadows = defines.iter().find(|(name, _)| name == "ENABLE_SHADOWS");
        assert_eq!(enable_shadows.map(|(_, v)| v.as_str()), Some("0"));
    }
}
