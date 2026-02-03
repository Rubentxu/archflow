// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Render - Render Pipelines
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 9
//
// Creates and manages the 4 specialized WebGPU render pipelines.
// Each pipeline is optimized for its specific use case to avoid
// SIMD divergence and maximize GPU efficiency.
//
// Features:
// - Shapes pipeline: SDF-based rendering for rectangles, circles, lines
// - Icons pipeline: Texture atlas lookup for icon rendering
// - Images pipeline: Texture2D array for PNG/jpeg images
// - Text pipeline: MTSDF text rendering with crisp edges
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(dead_code)]

use alloc::string::String;

use crate::shaders;
use crate::webgpu_context::WebGpuContext;

/// Render pipelines for the 4 specialized phases
///
/// Each pipeline is optimized for its specific render target:
/// - Shapes: SDF-based shapes with anti-aliasing
/// - Icons: Texture atlas lookup with bilinear filtering
/// - Images: Texture2D array for multiple images
/// - Text: MTSDF text with median calculation for crisp edges
pub struct RenderPipelines {
    /// Pipeline for SDF-based shape rendering
    pub shape_pipeline: wgpu::RenderPipeline,

    /// Pipeline for icon texture atlas rendering
    pub icon_pipeline: wgpu::RenderPipeline,

    /// Pipeline for texture2D array rendering
    pub image_pipeline: wgpu::RenderPipeline,

    /// Pipeline for MTSDF text rendering
    pub text_pipeline: wgpu::RenderPipeline,
}

impl RenderPipelines {
    /// Create all 4 render pipelines
    ///
    /// # Arguments
    /// * `context` - The WebGPU context with device and format
    ///
    /// # Returns
    /// `Result<RenderPipelines>` - The created pipelines or error
    ///
    /// # Errors
    /// - Shader compilation failed
    /// - Pipeline creation failed
    /// - Invalid shader code
    pub fn new(context: &WebGpuContext) -> Result<Self, String> {
        let device = &context.device;
        let format = context.swapchain_format();

        // Create all 4 pipelines
        let shape_pipeline = Self::create_shape_pipeline(device, format)?;
        let icon_pipeline = Self::create_icon_pipeline(device, format)?;
        let image_pipeline = Self::create_image_pipeline(device, format)?;
        let text_pipeline = Self::create_text_pipeline(device, format)?;

        Ok(Self {
            shape_pipeline,
            icon_pipeline,
            image_pipeline,
            text_pipeline,
        })
    }

    /// Create the SDF-based shape rendering pipeline
    fn create_shape_pipeline(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> Result<wgpu::RenderPipeline, String> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("SDF Shape Shader"),
            source: wgpu::ShaderSource::Wgsl(shaders::SHADER_SDF_SHAPES.into()),
        });

        // Create bind group layout for uniforms
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Uniform Bind Group Layout"),
            });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shape Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("SDF Shape Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Ok(pipeline)
    }

    /// Create the icon texture atlas pipeline
    fn create_icon_pipeline(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> Result<wgpu::RenderPipeline, String> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Icon Texture Shader"),
            source: wgpu::ShaderSource::Wgsl(shaders::SHADER_ICON_TEXTURE.into()),
        });

        // Create bind group layout for uniforms + texture
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                ],
                label: Some("Texture Bind Group Layout"),
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Icon Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Icon Texture Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Ok(pipeline)
    }

    /// Create the texture2D array pipeline for images
    fn create_image_pipeline(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> Result<wgpu::RenderPipeline, String> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Image Array Shader"),
            source: wgpu::ShaderSource::Wgsl(shaders::SHADER_IMAGE_ARRAY.into()),
        });

        // Create bind group layout for uniforms + texture2D array
        let texture_array_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                ],
                label: Some("Texture Array Bind Group Layout"),
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Image Render Pipeline Layout"),
            bind_group_layouts: &[&texture_array_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Image Array Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Ok(pipeline)
    }

    /// Create the MTSDF text rendering pipeline
    fn create_text_pipeline(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> Result<wgpu::RenderPipeline, String> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("MTSDF Text Shader"),
            source: wgpu::ShaderSource::Wgsl(shaders::SHADER_MTSDF_TEXT.into()),
        });

        // Create bind group layout for uniforms + texture
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                ],
                label: Some("Text Texture Bind Group Layout"),
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Text Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("MTSDF Text Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Ok(pipeline)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_constants_exist() {
        // Verify that shader constants are defined
        assert!(!shaders::SHADER_SDF_SHAPES.is_empty());
        assert!(!shaders::SHADER_ICON_TEXTURE.is_empty());
        assert!(!shaders::SHADER_IMAGE_ARRAY.is_empty());
        assert!(!shaders::SHADER_MTSDF_TEXT.is_empty());
    }

    #[test]
    fn test_shader_constants_contain_expected_elements() {
        // Verify shaders contain key markers
        assert!(shaders::SHADER_SDF_SHAPES.contains("struct VertexOutput"));
        assert!(shaders::SHADER_SDF_SHAPES.contains("@vertex"));
        assert!(shaders::SHADER_SDF_SHAPES.contains("@fragment"));

        assert!(shaders::SHADER_ICON_TEXTURE.contains("struct VertexOutput"));
        assert!(shaders::SHADER_IMAGE_ARRAY.contains("struct VertexOutput"));
        assert!(shaders::SHADER_MTSDF_TEXT.contains("fn median"));
    }
}
