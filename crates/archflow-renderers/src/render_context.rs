//! WebGPU Render Context
//!
//! This module provides WebGPU-based rendering with instancing support
//! for high-performance 2D batch rendering.

use thiserror::Error;

use wgpu::{
    ColorTargetState, ColorWrites, CommandEncoderDescriptor, Device, FragmentState, Instance,
    Operations, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, Queue,
    RenderPipeline, RenderPipelineDescriptor, ShaderSource, Surface, SurfaceConfiguration,
    TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

use crate::batch_renderer::{BatchRenderer2D, InstanceRaw};

/// Error type for render context operations.
#[derive(Debug, Error)]
pub enum RenderContextError {
    #[error("Failed to create instance")]
    InstanceCreation,

    #[error("Failed to create surface: {0}")]
    SurfaceCreation(String),

    #[error("Failed to configure surface: {0}")]
    SurfaceConfiguration(String),

    #[error("Failed to create shader module")]
    ShaderCreation,

    #[error("Failed to create pipeline layout: {0}")]
    PipelineLayoutCreation(String),

    #[error("Failed to create render pipeline: {0}")]
    PipelineCreation(String),

    #[error("Failed to acquire next swap chain texture: {0}")]
    SwapChainError(String),

    #[error("Rendering error: {0}")]
    RenderError(String),
}

/// WebGPU Render Context for batch rendering.
///
/// Manages GPU resources and render pipeline for instanced 2D rendering.
#[derive(Debug)]
pub struct RenderContext {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    config: SurfaceConfiguration,
    pipeline: RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
}

impl RenderContext {
    /// Creates a new RenderContext from an existing surface.
    pub async fn from_surface(
        instance: Instance,
        surface: Surface<'static>,
        width: u32,
        height: u32,
    ) -> Result<Self, RenderContextError> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(RenderContextError::InstanceCreation)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("archflow-renderers device"),
                    required_features: wgpu::Features::INDIRECT_FIRST_INSTANCE,
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .map_err(|_| RenderContextError::InstanceCreation)?;

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        // Create pipeline
        let shader_source = ShaderSource::Wgsl(include_str!("shaders/batch.wgsl").into());
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("batch_renderer shader"),
            source: shader_source,
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("batch_renderer pipeline layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let vertex_buffers = &[
            VertexBufferLayout {
                array_stride: std::mem::size_of::<[f32; 2]>() as u64,
                step_mode: VertexStepMode::Vertex,
                attributes: &[VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                }],
            },
            VertexBufferLayout {
                array_stride: std::mem::size_of::<InstanceRaw>() as u64,
                step_mode: VertexStepMode::Instance,
                attributes: &[
                    VertexAttribute {
                        format: VertexFormat::Float32x4,
                        offset: 0,
                        shader_location: 1,
                    },
                    VertexAttribute {
                        format: VertexFormat::Float32x4,
                        offset: 16,
                        shader_location: 2,
                    },
                    VertexAttribute {
                        format: VertexFormat::Float32x4,
                        offset: 32,
                        shader_location: 3,
                    },
                    VertexAttribute {
                        format: VertexFormat::Float32x4,
                        offset: 48,
                        shader_location: 4,
                    },
                    VertexAttribute {
                        format: VertexFormat::Float32x4,
                        offset: 64,
                        shader_location: 5,
                    },
                ],
            },
        ];

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("batch_renderer pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: vertex_buffers,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                unclipped_depth: false,
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

        // Create buffers
        let vertices: [f32; 8] = [-0.5, -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5];
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("quad vertex buffer"),
            size: std::mem::size_of::<[f32; 8]>() as u64,
            usage: wgpu::BufferUsages::VERTEX,
            mapped_at_creation: true,
        });
        vertex_buffer
            .slice(..)
            .get_mapped_range_mut()
            .copy_from_slice(bytemuck::bytes_of(&vertices));
        vertex_buffer.unmap();

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("quad index buffer"),
            size: std::mem::size_of::<[u16; 6]>() as u64,
            usage: wgpu::BufferUsages::INDEX,
            mapped_at_creation: true,
        });
        index_buffer
            .slice(..)
            .get_mapped_range_mut()
            .copy_from_slice(bytemuck::bytes_of(&indices));
        index_buffer.unmap();

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instance buffer"),
            size: (10_000 * std::mem::size_of::<InstanceRaw>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(Self {
            device,
            queue,
            surface,
            config,
            pipeline,
            vertex_buffer,
            index_buffer,
            instance_buffer,
        })
    }

    /// Resizes the render surface.
    #[allow(dead_code)]
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    /// Renders the batched instances to the current frame.
    pub fn render(&mut self, batches: &BatchRenderer2D) -> Result<(), RenderContextError> {
        let frame = self.surface.get_current_texture().map_err(|e| {
            RenderContextError::SwapChainError(format!("Failed to get frame: {}", e))
        })?;

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("batch render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            // Iterate batches in deterministic order (BTreeMap guarantees sorted keys)
            for (_material_id, instances) in batches.iter_batches() {
                if instances.is_empty() {
                    continue;
                }

                // material_id is a MaterialId type for type safety
                // The sorted iteration ensures consistent draw call order

                self.queue
                    .write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(instances));

                let instance_size = std::mem::size_of::<InstanceRaw>() as u64;
                render_pass.set_vertex_buffer(
                    1,
                    self.instance_buffer
                        .slice(..(instances.len() as u64 * instance_size)),
                );

                render_pass.draw_indexed(0..6, 0, 0..instances.len() as u32);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();

        Ok(())
    }

    /// Returns a reference to the device.
    #[inline]
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Returns a reference to the queue.
    #[inline]
    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    /// Returns the current surface format.
    #[inline]
    pub fn surface_format(&self) -> TextureFormat {
        self.config.format
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_context_error_variants() {
        let errors: [RenderContextError; 5] = [
            RenderContextError::InstanceCreation,
            RenderContextError::SurfaceCreation("test".to_string()),
            RenderContextError::SurfaceConfiguration("test".to_string()),
            RenderContextError::PipelineLayoutCreation("test".to_string()),
            RenderContextError::PipelineCreation("test".to_string()),
        ];

        for error in errors {
            let msg = format!("{}", error);
            assert!(!msg.is_empty(), "Error message should not be empty");
        }
    }
}
