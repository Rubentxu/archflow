// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Render - GPU Resources
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 9
//
// Creates and manages GPU buffers and textures for rendering.
// Handles efficient data transfer between CPU and GPU.
//
// Features:
// - Instance buffer: Storage buffer for 100k entities (~4.8MB)
// - Uniform buffer: Camera uniforms for view-projection matrix
// - Texture atlases: Placeholder for icons, images, and text
// - Efficient buffer updates with write_buffer
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(dead_code)]

use alloc::string::String;

use archflow_core::MAX_ENTITIES;

use crate::renderer::CameraUniforms;
use crate::webgpu_context::WebGpuContext;

/// Size of a single GpuInstance in bytes
const INSTANCE_SIZE: usize = 48;

/// Size of the camera uniform buffer in bytes
const UNIFORM_SIZE: usize = 64;

/// Maximum number of entities that can be rendered in one frame
pub const MAX_RENDER_ENTITIES: usize = MAX_ENTITIES as usize;

/// GPU resources including buffers and textures
///
/// This struct holds all GPU-side resources needed for rendering.
/// Buffers are pre-allocated and reused every frame to avoid allocations.
pub struct GpuResources {
    /// Instance buffer (storage buffer for up to 100k entities)
    ///
    /// Layout: Array of GpuInstance structs (48 bytes each)
    /// Usage: STORAGE buffer for read-write in shaders
    pub instance_buffer: wgpu::Buffer,

    /// Uniform buffer for camera uniforms
    ///
    /// Layout: CameraUniforms struct (64 bytes)
    /// Usage: UNIFORM buffer read in vertex shaders
    pub uniform_buffer: wgpu::Buffer,

    /// Icon texture atlas (placeholder)
    ///
    /// This will be populated with icon textures loaded from Draw.io
    /// For now, it's a 1x1 white texture as placeholder
    pub icon_atlas: wgpu::Texture,

    /// Image texture array (placeholder)
    ///
    /// This will be populated with loaded images
    /// For now, it's a single-element array
    pub image_array: wgpu::Texture,

    /// MTSDF text atlas (placeholder)
    ///
    /// This will be populated with MTSDF glyph textures
    /// For now, it's a 1x1 white texture as placeholder
    pub text_atlas: wgpu::Texture,

    /// Bind group layout for uniforms (shared by all pipelines)
    pub uniform_bind_group_layout: wgpu::BindGroupLayout,

    /// Bind group layout for textures
    pub texture_bind_group_layout: wgpu::BindGroupLayout,

    /// Bind group for uniforms (shared by all pipelines)
    pub uniform_bind_group: wgpu::BindGroup,

    /// Bind group for icon atlas
    pub icon_bind_group: wgpu::BindGroup,

    /// Bind group for image array
    pub image_bind_group: wgpu::BindGroup,

    /// Bind group for text atlas
    pub text_bind_group: wgpu::BindGroup,
}

impl GpuResources {
    /// Create all GPU resources
    ///
    /// This allocates GPU memory for:
    /// - Instance buffer: 100k entities × 48 bytes = ~4.8MB
    /// - Uniform buffer: 64 bytes
    /// - Placeholder textures for atlases
    ///
    /// # Arguments
    /// * `context` - The WebGPU context with device and limits
    ///
    /// # Returns
    /// `Result<GpuResources>` - The created resources or error
    ///
    /// # Errors
    /// - Buffer creation failed
    /// - Texture creation failed
    /// - Bind group creation failed
    pub fn new(context: &WebGpuContext) -> Result<Self, String> {
        let device = &context.device;

        // Create instance buffer (storage buffer for up to 100k entities)
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (MAX_ENTITIES as usize * INSTANCE_SIZE) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create uniform buffer for camera uniforms
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: UNIFORM_SIZE as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create placeholder icon atlas (1x1 white texture)
        let icon_atlas = Self::create_placeholder_texture(device, &context.queue, "Icon Atlas")?;

        // Create placeholder image array (single-element array)
        let image_array = Self::create_placeholder_texture_array(device, &context.queue)?;

        // Create placeholder text atlas (1x1 white texture)
        let text_atlas = Self::create_placeholder_texture(device, &context.queue, "Text Atlas")?;

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

        // Create bind group layout for textures
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

        // Create bind group for uniforms
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("Uniform Bind Group"),
        });

        // Create bind group for icon atlas
        let icon_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&icon_atlas.create_view(
                        &wgpu::TextureViewDescriptor {
                            label: Some("Icon Atlas View"),
                            format: None,
                            dimension: Some(wgpu::TextureViewDimension::D2),
                            aspect: wgpu::TextureAspect::All,
                            base_mip_level: 0,
                            mip_level_count: None,
                            base_array_layer: 0,
                            array_layer_count: None,
                        },
                    )),
                },
            ],
            label: Some("Icon Bind Group"),
        });

        // Create bind group for image array
        let image_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&image_array.create_view(
                        &wgpu::TextureViewDescriptor {
                            label: Some("Image Array View"),
                            format: None,
                            dimension: Some(wgpu::TextureViewDimension::D2Array),
                            aspect: wgpu::TextureAspect::All,
                            base_mip_level: 0,
                            mip_level_count: None,
                            base_array_layer: 0,
                            array_layer_count: None,
                        },
                    )),
                },
            ],
            label: Some("Image Bind Group"),
        });

        // Create bind group for text atlas
        let text_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&text_atlas.create_view(
                        &wgpu::TextureViewDescriptor {
                            label: Some("Text Atlas View"),
                            format: None,
                            dimension: Some(wgpu::TextureViewDimension::D2),
                            aspect: wgpu::TextureAspect::All,
                            base_mip_level: 0,
                            mip_level_count: None,
                            base_array_layer: 0,
                            array_layer_count: None,
                        },
                    )),
                },
            ],
            label: Some("Text Bind Group"),
        });

        Ok(Self {
            instance_buffer,
            uniform_buffer,
            icon_atlas,
            image_array,
            text_atlas,
            uniform_bind_group_layout,
            texture_bind_group_layout,
            uniform_bind_group,
            icon_bind_group,
            image_bind_group,
            text_bind_group,
        })
    }

    /// Write instance data to the GPU instance buffer
    ///
    /// This uploads the prepared instance data from the renderer
    /// to the GPU storage buffer for rendering.
    ///
    /// # Arguments
    /// * `queue` - The WebGPU command queue
    /// * `instances` - Slice of GpuInstance data to upload
    ///
    /// # Returns
    /// `Result<()>` - Success or error message
    pub fn write_instances(
        &self,
        queue: &wgpu::Queue,
        instances: &[crate::renderer::GpuInstance],
    ) -> Result<(), String> {
        let byte_slice = bytemuck::cast_slice(instances);
        queue.write_buffer(&self.instance_buffer, 0, byte_slice);
        Ok(())
    }

    /// Write camera uniforms to the GPU uniform buffer
    ///
    /// This updates the camera uniforms on the GPU for the current frame.
    ///
    /// # Arguments
    /// * `queue` - The WebGPU command queue
    /// * `uniforms` - Camera uniforms to upload
    ///
    /// # Returns
    /// `Result<()>` - Success or error message
    pub fn write_uniforms(
        &self,
        queue: &wgpu::Queue,
        uniforms: &CameraUniforms,
    ) -> Result<(), String> {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[*uniforms]));
        Ok(())
    }

    /// Get the uniform bind group
    #[inline]
    pub fn uniform_bind_group(&self) -> &wgpu::BindGroup {
        &self.uniform_bind_group
    }

    /// Get the icon bind group
    #[inline]
    pub fn icon_bind_group(&self) -> &wgpu::BindGroup {
        &self.icon_bind_group
    }

    /// Get the image bind group
    #[inline]
    pub fn image_bind_group(&self) -> &wgpu::BindGroup {
        &self.image_bind_group
    }

    /// Get the text bind group
    #[inline]
    pub fn text_bind_group(&self) -> &wgpu::BindGroup {
        &self.text_bind_group
    }

    /// Create a placeholder 1x1 white texture
    fn create_placeholder_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: &str,
    ) -> Result<wgpu::Texture, String> {
        let size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        });

        // Write white pixel data (RGBA = [255, 255, 255, 255])
        let data: [u8; 4] = [255, 255, 255, 255];

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: None,
            },
            size,
        );

        Ok(texture)
    }

    /// Create a placeholder texture2D array with a single white texture
    fn create_placeholder_texture_array(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<wgpu::Texture, String> {
        let size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1, // Single texture in array
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Image Array (Placeholder)"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        });

        // Write white pixel data
        let data: [u8; 4] = [255, 255, 255, 255];

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: None,
            },
            size,
        );

        Ok(texture)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_size_is_correct() {
        // Verify our instance size constant is correct
        assert_eq!(INSTANCE_SIZE, 48);
        assert_eq!(
            core::mem::size_of::<crate::renderer::GpuInstance>(),
            INSTANCE_SIZE
        );
    }

    #[test]
    fn test_uniform_size_is_correct() {
        // Verify our uniform size constant is correct
        assert_eq!(UNIFORM_SIZE, 64);
        assert_eq!(core::mem::size_of::<CameraUniforms>(), UNIFORM_SIZE);
    }

    #[test]
    fn test_max_render_entities() {
        // Verify the max entities constant is correct
        assert_eq!(MAX_RENDER_ENTITIES, MAX_ENTITIES as usize);
        assert_eq!(MAX_RENDER_ENTITIES, 100_000);
    }

    #[test]
    fn test_calculate_instance_buffer_size() {
        // Calculate: 100,000 entities × 48 bytes each = 4.8MB
        let expected_size = MAX_ENTITIES as usize * INSTANCE_SIZE;
        assert_eq!(expected_size, 4_800_000); // ~4.8MB
    }
}
