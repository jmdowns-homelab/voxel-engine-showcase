//! Manages WebGPU bind groups and their layouts.
//!
//! This module handles the creation and management of WebGPU bind groups and
//! their corresponding layouts. It provides a centralized way to manage GPU resources
//! that need to be accessed by shaders, such as uniform buffers, textures, and samplers.

use std::{collections::HashMap, num::NonZeroU32};

use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Device, Features, Queue};

use crate::{
    core::StSystem,
    engine_state::{buffer_state::BufferState, camera_state::CAMERA_BUFFER_NAME},
};

use super::meshing::CHUNK_INDEX_BUFFER_NAME;

/// Manages WebGPU bind groups and their layouts.
///
/// This struct provides a centralized way to create, store, and retrieve
/// bind groups and their layouts. It handles the creation of common bind groups
/// used throughout the rendering pipeline, such as camera uniforms and textures.
pub struct BindGroupState {
    /// Map of bind group names to their WebGPU bind group objects
    bind_groups: HashMap<&'static str, wgpu::BindGroup>,
    /// Map of bind group layout names to their WebGPU bind group layout objects
    bind_group_layouts: HashMap<&'static str, wgpu::BindGroupLayout>,
}

/// Default dimension for texture atlases (width and height in pixels)
const TEXTURE_DIMENSION: u32 = 16;
/// Number of mip levels to generate for textures
const MIP_LEVEL: u32 = TEXTURE_DIMENSION.ilog2() + 1;
/// Number of textures in the texture atlas
const NUM_TEXTURES: u32 = 5;
/// Total length of texture data including all mip levels
const TEXTURE_LENGTH_WITH_MIPMAPS: u32 = 341;

impl BindGroupState {
    /// Creates a new `BindGroupState` instance with default bind groups.
    ///
    /// This initializes the following bind groups:
    /// - Camera uniforms
    /// - Texture atlas and sampler
    /// - Chunk index buffer
    ///
    /// # Arguments
    /// * `device` - The WebGPU device
    /// * `buffer_state` - Shared state for buffer management
    /// * `queue` - The WebGPU queue for resource uploads
    /// * `atlas_bytes` - Raw RGBA data for the texture atlas
    ///
    /// # Returns
    /// A new `BindGroupState` instance with all default bind groups created
    pub fn new(
        device: StSystem<Device>,
        buffer_state: StSystem<BufferState>,
        queue: StSystem<Queue>,
        atlas_bytes: Vec<u8>,
    ) -> Self {
        let mut bind_groups = HashMap::new();
        let mut bind_group_layouts = HashMap::new();

        let device = device.get();

        let (camera_bind_group, camera_bind_group_layout) =
            Self::generate_camera_bindgroups(&device, &buffer_state.get());

        bind_groups.insert(CAMERA_BIND_GROUP, camera_bind_group);
        bind_group_layouts.insert(CAMERA_BIND_GROUP_LAYOUT, camera_bind_group_layout);

        let (texture_bind_group, texture_bind_group_layout) =
            Self::generate_texture_bindgroups(&device, queue, atlas_bytes);

        bind_groups.insert(TEXTURE_BIND_GROUP, texture_bind_group);
        bind_group_layouts.insert(TEXTURE_BIND_GROUP_LAYOUT, texture_bind_group_layout);

        let (chunk_index_bind_group, chunk_index_bind_group_layout) =
            Self::generate_chunk_index_bindgroups(&device, &buffer_state.get());

        bind_groups.insert(CHUNK_INDEX_BIND_GROUP, chunk_index_bind_group);
        bind_group_layouts.insert(CHUNK_INDEX_BIND_GROUP_LAYOUT, chunk_index_bind_group_layout);

        Self {
            bind_groups,
            bind_group_layouts,
        }
    }

    /// Retrieves a bind group by name.
    ///
    /// # Arguments
    /// * `name` - The name of the bind group to retrieve
    ///
    /// # Returns
    /// A reference to the requested bind group
    ///
    /// # Panics
    /// Panics if no bind group with the given name exists
    pub fn get_bind_group(&self, name: &'static str) -> &wgpu::BindGroup {
        self.bind_groups.get(name).unwrap()
    }

    /// Retrieves a bind group layout by name.
    ///
    /// # Arguments
    /// * `name` - The name of the bind group layout to retrieve
    ///
    /// # Returns
    /// A reference to the requested bind group layout
    ///
    /// # Panics
    /// Panics if no bind group layout with the given name exists
    pub fn get_bind_group_layout(&self, name: &'static str) -> &wgpu::BindGroupLayout {
        self.bind_group_layouts.get(name).unwrap()
    }

    /// Creates bind groups for camera uniforms.
    ///
    /// This sets up the bind group layout and bind group for camera uniforms
    /// that will be used in the vertex and fragment shaders.
    ///
    /// # Arguments
    /// * `device` - The WebGPU device
    /// * `buffer_state` - Shared state for buffer management
    ///
    /// # Returns
    /// A tuple containing the bind group and its layout
    fn generate_camera_bindgroups(
        device: &Device,
        buffer_state: &BufferState,
    ) -> (BindGroup, BindGroupLayout) {
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some(CAMERA_BIND_GROUP_LAYOUT),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer_state.get_entire_binding(CAMERA_BUFFER_NAME),
            }],
            label: Some(CAMERA_BIND_GROUP),
        });

        (camera_bind_group, camera_bind_group_layout)
    }

    /// Creates bind groups for texture arrays using WebGPU's bind group array feature.
    ///
    /// This method is used when the hardware supports binding texture arrays directly.
    /// It creates a texture array with mipmapping and a corresponding sampler.
    ///
    /// # Arguments
    /// * `device` - The WebGPU device
    /// * `queue` - The WebGPU queue for uploading texture data
    /// * `atlas_rgba_bytes` - Raw RGBA data for the texture atlas
    ///
    /// # Returns
    /// A tuple containing the bind group and its layout
    fn generate_texture_bindgroups_with_binding_texture_array(
        device: &Device,
        queue: StSystem<Queue>,
        atlas_rgba_bytes: Vec<u8>,
    ) -> (BindGroup, BindGroupLayout) {
        let mut block_texture_vec = Vec::new();
        let mut block_texture_view_vec = Vec::new();
        let mut block_texture_view_reference_vec = Vec::new();

        let texture_size = wgpu::Extent3d {
            width: TEXTURE_DIMENSION,
            height: TEXTURE_DIMENSION,
            depth_or_array_layers: 1,
        };

        let texture_descriptor = wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: MIP_LEVEL,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: None,
            view_formats: &[],
        };

        for i in 0..NUM_TEXTURES as usize {
            let texture_range = i * TEXTURE_LENGTH_WITH_MIPMAPS as usize * 4
                ..(i + 1) * TEXTURE_LENGTH_WITH_MIPMAPS as usize * 4;
            let block_texture = device.create_texture_with_data(
                &queue.get(),
                &wgpu::TextureDescriptor {
                    label: Some(&format!("Texture {i}")),
                    ..texture_descriptor
                },
                wgpu::util::TextureDataOrder::LayerMajor,
                &atlas_rgba_bytes[texture_range],
            );

            let block_texture_view =
                block_texture.create_view(&wgpu::TextureViewDescriptor::default());

            block_texture_vec.push(block_texture);
            block_texture_view_vec.push(block_texture_view);
        }

        for block_texture_view in block_texture_view_vec.iter() {
            block_texture_view_reference_vec.push(block_texture_view);
        }

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let texture_array_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: NonZeroU32::new(NUM_TEXTURES),
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let texture_array_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_array_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(
                        &block_texture_view_reference_vec[..],
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("texture_bind_group"),
        });

        (texture_array_bind_group, texture_array_bind_group_layout)
    }

    /// Creates bind groups for textures without using WebGPU's bind group array feature.
    ///
    /// This fallback method is used when the hardware doesn't support binding texture arrays
    /// directly. It creates individual textures and combines them into an array in the shader.
    ///
    /// # Arguments
    /// * `device` - The WebGPU device
    /// * `queue` - The WebGPU queue for uploading texture data
    /// * `atlas_rgba_bytes` - Raw RGBA data for the texture atlas
    ///
    /// # Returns
    /// A tuple containing the bind group and its layout
    fn generate_texture_bindgroups_without_binding_texture_array(
        device: &Device,
        queue: StSystem<Queue>,
        atlas_rgba_bytes: Vec<u8>,
    ) -> (BindGroup, BindGroupLayout) {
        let texture_size = wgpu::Extent3d {
            width: TEXTURE_DIMENSION,
            height: TEXTURE_DIMENSION,
            depth_or_array_layers: NUM_TEXTURES,
        };

        let texture_descriptor = wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: MIP_LEVEL,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: None,
            view_formats: &[],
        };

        let onion_texture = device.create_texture_with_data(
            &queue.get(),
            &wgpu::TextureDescriptor {
                label: Some("Onion Texture"),
                ..texture_descriptor
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &atlas_rgba_bytes[0..NUM_TEXTURES as usize * TEXTURE_LENGTH_WITH_MIPMAPS as usize * 4],
        );

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let texture_array_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let texture_array_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_array_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &onion_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("texture_bind_group"),
        });

        (texture_array_bind_group, texture_array_bind_group_layout)
    }

    /// Creates texture bind groups, automatically selecting the appropriate method.
    ///
    /// This method checks the device features and selects either the texture array
    /// or fallback implementation based on hardware support.
    ///
    /// # Arguments
    /// * `device` - The WebGPU device
    /// * `queue` - The WebGPU queue for uploading texture data
    /// * `atlas_bytes` - Raw RGBA data for the texture atlas
    ///
    /// # Returns
    /// A tuple containing the bind group and its layout
    fn generate_texture_bindgroups(
        device: &Device,
        queue: StSystem<Queue>,
        atlas_bytes: Vec<u8>,
    ) -> (BindGroup, BindGroupLayout) {
        if device.features().contains(Features::TEXTURE_BINDING_ARRAY) {
            Self::generate_texture_bindgroups_with_binding_texture_array(
                device,
                queue,
                atlas_bytes,
            )
        } else {
            Self::generate_texture_bindgroups_without_binding_texture_array(
                device,
                queue,
                atlas_bytes,
            )
        }
    }

    /// Creates bind groups for chunk index buffers.
    ///
    /// This sets up the bind group layout and bind group for chunk index
    /// buffers that are used in indirect rendering of chunks.
    ///
    /// # Arguments
    /// * `device` - The WebGPU device
    /// * `buffer_state` - Shared state for buffer management
    ///
    /// # Returns
    /// A tuple containing the bind group and its layout
    fn generate_chunk_index_bindgroups(
        device: &Device,
        buffer_state: &BufferState,
    ) -> (BindGroup, BindGroupLayout) {
        let chunk_index_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some(CHUNK_INDEX_BIND_GROUP_LAYOUT),
            });

        let chunk_index_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &chunk_index_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer_state.get_entire_binding(CHUNK_INDEX_BUFFER_NAME),
            }],
            label: Some(CHUNK_INDEX_BIND_GROUP),
        });

        (chunk_index_bind_group, chunk_index_bind_group_layout)
    }
}

/// Name of the camera bind group
pub const CAMERA_BIND_GROUP: &str = "camera_bind_group";
/// Name of the camera bind group layout
pub const CAMERA_BIND_GROUP_LAYOUT: &str = "camera_bind_group_layout";
/// Name of the texture bind group
pub const TEXTURE_BIND_GROUP: &str = "texture_bind_group";
/// Name of the texture bind group layout
pub const TEXTURE_BIND_GROUP_LAYOUT: &str = "texture_bind_group_layout";
/// Name of the chunk index bind group
pub const CHUNK_INDEX_BIND_GROUP: &str = "chunk_index_bind_group";
/// Name of the chunk index bind group layout
pub const CHUNK_INDEX_BIND_GROUP_LAYOUT: &str = "chunk_index_bind_group_layout";
