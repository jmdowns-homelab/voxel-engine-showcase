//! Texture handling for the rendering pipeline.
//!
//! This module provides functionality for creating and managing GPU textures,
//! including depth textures used in the rendering process.

/// Represents a GPU texture with associated view and sampler.
///
/// This struct wraps a WebGPU texture along with its view and sampler,
/// providing a convenient way to manage texture resources in the rendering pipeline.
pub struct Texture {
    /// The underlying WebGPU texture resource.
    #[allow(dead_code)]
    pub texture: wgpu::Texture,
    /// The texture view used for binding the texture to the pipeline.
    pub view: wgpu::TextureView,
    /// The sampler used for texture filtering and addressing.
    #[allow(dead_code)]
    pub sampler: wgpu::Sampler,
}

impl Texture {
    /// The texture format used for depth buffers.
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    /// Creates a new depth texture with the given configuration.
    ///
    /// # Arguments
    /// * `device` - The WebGPU device
    /// * `config` - The surface configuration containing dimensions
    /// * `label` - Debug label for the texture
    ///
    /// # Returns
    /// A new `Texture` instance configured as a depth buffer
    ///
    /// # Panics
    /// May panic if texture creation fails
    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };

        let texture = device.create_texture(&desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }
}
