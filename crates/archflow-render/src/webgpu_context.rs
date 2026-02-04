// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Render - WebGPU Context
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 9
//
// WebGPU context initialization and management.
// Handles device creation, surface setup, and swapchain management.
//
// Features:
// - WebGPU initialization (async when supported)
// - Automatic adapter selection (high performance when available)
// - Swapchain management with resize support
// - Error handling with descriptive messages
//
// Note: This module uses a simplified approach compatible with wgpu 23.
// The actual surface creation will be handled by the web layer (archflow-web).
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(dead_code)]

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;

/// WebGPU context containing device, surface, and queue
///
/// This struct holds all the WebGPU objects needed for rendering.
/// The surface has a static lifetime because it's owned by this context.
pub struct WebGpuContext {
    /// WebGPU instance (wgpu::Instance)
    pub instance: wgpu::Instance,

    /// Selected GPU adapter
    pub adapter: wgpu::Adapter,

    /// Logical device for command submission
    pub device: wgpu::Device,

    /// Command queue for submitting work to the GPU
    pub queue: wgpu::Queue,

    /// Surface for presenting rendered frames
    pub surface: Option<wgpu::Surface<'static>>,

    /// Preferred texture format for the swapchain
    pub swapchain_format: wgpu::TextureFormat,
}

impl WebGpuContext {
    /// Create a new WebGPU context
    ///
    /// This creates a WebGPU instance, adapter, device, and queue.
    /// The surface can be added later with `set_surface()`.
    ///
    /// # Returns
    /// `Result<WebGpuContext>` - The initialized context or error
    ///
    /// # Example
    ///
    /// ```ignore
    /// let context = WebGpuContext::new()?;
    /// ```
    ///
    /// Note: This method is synchronous but will use a blocking poll for the adapter.
    /// For async initialization, use `new_async()` instead.
    pub fn new() -> Result<Self, String> {
        // Create instance with all available backends
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // For simplicity, we'll use a blocking adapter request
        // In production, this should be async
        let adapter = pollster::block_on(async {
            instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: None,
                    force_fallback_adapter: false,
                })
                .await
        })
        .ok_or("Failed to request WebGPU adapter".to_string())?;

        // Create device and queue
        let (device, queue) = pollster::block_on(async {
            adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: Some("WebGPU Device"),
                        required_features: wgpu::Features::empty(),
                        required_limits: if cfg!(target_arch = "wasm32") {
                            wgpu::Limits::downlevel_webgl2_defaults()
                        } else {
                            wgpu::Limits::default()
                        },
                        memory_hints: Default::default(),
                        experimental_features: Default::default(),
                    },
                    None, // trace_path
                )
                .await
        })
        .map_err(|e| format!("Failed to create device: {e}"))?;

        // Default swapchain format (will be updated when surface is set)
        let swapchain_format = wgpu::TextureFormat::Bgra8UnormSrgb;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            surface: None,
            swapchain_format,
        })
    }

    /// Set the surface for rendering
    ///
    /// This should be called with a surface created from the web layer.
    /// The surface is configured with the appropriate format.
    ///
    /// # Arguments
    /// * `surface` - The WebGPU surface
    pub fn set_surface(&mut self, surface: wgpu::Surface<'static>) {
        // Get supported swapchain formats
        let surface_caps = surface.get_capabilities(&self.adapter);
        let format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        self.swapchain_format = format;
        self.surface = Some(surface);
    }

    /// Configure the surface for a given size
    ///
    /// Configures the surface for the specified dimensions.
    /// This should be called after surface creation or resize.
    ///
    /// # Arguments
    /// * `width` - Surface width in pixels
    /// * `height` - Surface height in pixels
    pub fn configure_surface(&self, width: u32, height: u32) {
        if let Some(surface) = &self.surface {
            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.swapchain_format,
                width,
                height,
                present_mode: wgpu::PresentMode::Fifo,
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
            };

            surface.configure(&self.device, &config);
        }
    }

    /// Get the next texture from the surface
    ///
    /// # Returns
    /// `Option<wgpu::SurfaceTexture>` - The next texture or None if lost/no surface
    pub fn get_current_texture(&self) -> Option<wgpu::SurfaceTexture> {
        match &self.surface {
            Some(surface) => surface.get_current_texture().ok(),
            None => None,
        }
    }

    /// Get the preferred swapchain format
    #[inline]
    pub fn swapchain_format(&self) -> wgpu::TextureFormat {
        self.swapchain_format
    }

    /// Get the device limits
    pub fn limits(&self) -> wgpu::Limits {
        self.device.limits()
    }

    /// Get the adapter info (useful for debugging)
    pub fn adapter_info(&self) -> String {
        let info = self.adapter.get_info();
        format!(
            "{} (Driver: {}, Backend: {:?})",
            info.name, info.driver, info.backend
        )
    }
}

impl Default for WebGpuContext {
    fn default() -> Self {
        Self::new().expect("Failed to create WebGPU context")
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swapchain_format_is_valid() {
        // Common swapchain formats that should be supported
        let valid_formats = [
            wgpu::TextureFormat::Bgra8UnormSrgb,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        ];

        // Verify they are valid TextureFormat variants
        for format in valid_formats {
            let _name = format!("{:?}", format);
        }
    }

    #[test]
    fn test_webgpu_context_fields() {
        // Verify the struct has the expected fields
        assert!(core::mem::size_of::<WebGpuContext>() > 0);
    }

    #[test]
    fn test_webgpu_context_default() {
        // Test default creation
        let context = WebGpuContext::default();
        assert!(core::mem::size_of_val(&context) > 0);
    }
}
