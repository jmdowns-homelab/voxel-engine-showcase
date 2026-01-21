//! Manages the WebGPU render pipeline and associated rendering resources.
//!
//! This module is responsible for coordinating the rendering process by managing
//! specialized renderers, shared resources, and the render pass configuration.
//! It serves as the central orchestrator for rendering operations in the voxel engine.
//!
//! # Architecture
//!
//! The rendering system follows a modular design:
//!
//! - `PipelineManager`: Coordinates the overall rendering process
//! - `MeshingRenderer`: Handles voxel mesh rendering with its own pipeline
//! - `UiRenderer`: Manages UI element rendering with its own pipeline
//!
//! # Resource Management
//!
//! The pipeline manager initializes and maintains shared resources:
//!
//! - Bind groups for camera, textures, and chunk indices
//! - GPU buffer state for vertex, index, and indirect buffers
//! - Depth textures and other rendering resources
//!
//! # Performance Considerations
//!
//! - Uses GPU timestamp queries for performance profiling
//! - Delegates specialized rendering to dedicated renderer components
//! - Combines world and UI rendering in a single pass when possible

use log::error;
use wgpu::{
    Device, Queue, RenderPipeline, Surface, SurfaceConfiguration, TextureFormat,
};

use crate::{
    core::StSystem,
    engine_state::voxels::block::block_side::BlockSide,
};
use crate::engine_state::rendering::meshing::MeshingRenderer;
use super::{
    bind_group_state::{
        self, BindGroupState, CAMERA_BIND_GROUP, CAMERA_BIND_GROUP_LAYOUT, CHUNK_INDEX_BIND_GROUP,
        CHUNK_INDEX_BIND_GROUP_LAYOUT, TEXTURE_BIND_GROUP, TEXTURE_BIND_GROUP_LAYOUT,
    },
    query_manager::{self, QueryManager},
    texture,
    vertex::Vertex,
    MeshManager,
    super::buffer_state::BufferState,
    ui::{UiMeshManager, UiRenderer},
};

/// Manages the WebGPU rendering process and associated rendering resources.
///
/// This struct is responsible for coordinating the rendering process by:
/// 1. Managing specialized renderers (meshing, UI)
/// 2. Maintaining shared resources (bind groups, buffers)
/// 3. Configuring and executing render passes
///
/// It delegates the actual rendering operations to specialized renderer components.
pub struct PipelineManager {
    /// Manages GPU timing queries for performance profiling
    pub query_manager: QueryManager,
    /// Manages all bind groups used in the pipeline
    pub bind_group_state: StSystem<BindGroupState>,
    /// Shared state for buffer management
    pub buffer_state: StSystem<BufferState>,
    /// Depth texture used for depth testing
    pub depth_texture: texture::Texture,
    /// UI renderer for 2D interface elements
    pub ui_renderer: StSystem<UiRenderer>,
    /// UI mesh manager for handling UI elements and their shared buffers
    pub ui_mesh_manager: StSystem<UiMeshManager>,
    /// Meshing renderer for voxel meshes
    pub meshing_renderer: MeshingRenderer,
}

impl PipelineManager {
    /// Creates a new `PipelineManager` instance.
    ///
    /// # Arguments
    /// * `device` - The WebGPU device
    /// * `queue` - The WebGPU queue for buffer operations
    /// * `config` - Surface configuration containing size and format
    /// * `texture_format` - The texture format to use for rendering
    /// * `buffer_state` - Shared state for buffer management
    /// * `shader_string` - The WGSL shader source code for mesh rendering
    /// * `ui_shader_string` - The UI WGSL shader source code
    /// * `atlas_rgba_bytes` - Raw RGBA data for the texture atlas
    ///
    /// # Returns
    /// A new `PipelineManager` instance with all rendering resources initialized
    ///
    /// # Implementation Details
    ///
    /// - Initializes shared bind groups and buffer state
    /// - Creates specialized renderers for different rendering tasks
    /// - Sets up performance profiling via query manager
    pub fn new(
        device: StSystem<Device>,
        queue: StSystem<Queue>,
        config: &SurfaceConfiguration,
        texture_format: TextureFormat,
        buffer_state: StSystem<BufferState>,
        shader_string: String,
        ui_shader_string: String,
        atlas_rgba_bytes: Vec<u8>,
    ) -> Self {
        let bind_group_state = StSystem::new(Box::new(BindGroupState::new(
            device.clone(),
            buffer_state.clone(),
            queue.clone(),
            atlas_rgba_bytes,
        )));

        let device_ref = device.get();

        let query_manager = QueryManager::new(&device_ref, buffer_state.clone());

        let depth_texture =
            texture::Texture::create_depth_texture(&device_ref, config, "DEPTH TEXTURE");

        let depth_stencil = Some(wgpu::DepthStencilState {
            format: texture::Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        });

        // Create UI mesh manager
        let ui_mesh_manager = StSystem::new(Box::new(UiMeshManager::new(buffer_state.clone())));
        
        // Create UI renderer
        let ui_renderer = StSystem::new(Box::new(UiRenderer::new(
            &device_ref, 
            config, 
            texture_format, 
            depth_stencil.clone(), 
            &ui_shader_string,
            buffer_state.clone(),
        )));
        
        // Create the meshing renderer with its own render pipeline
        let meshing_renderer = MeshingRenderer::new(
            device.clone(),
            buffer_state.clone(),
            &shader_string,
            texture_format,
            bind_group_state.clone(),
            depth_stencil.clone(),
        );
        
        Self {
            query_manager,
            bind_group_state,
            buffer_state,
            depth_texture,
            ui_renderer,
            ui_mesh_manager,
            meshing_renderer,
        }
    }

    /// Renders a frame to the given surface.
    ///
    /// This method handles the complete rendering pipeline execution for a single frame:
    /// 1. Acquires the next frame from the surface
    /// 2. Sets up performance measurement via timestamp queries
    /// 3. Creates a render pass with appropriate attachments
    /// 4. Delegates rendering to specialized renderers:
    ///    - MeshingRenderer for voxel meshes
    ///    - UiRenderer for UI elements
    /// 5. Submits commands to the GPU and presents the frame
    /// 6. Collects performance metrics
    ///
    /// # Arguments
    /// * `surface` - The target surface to render to
    /// * `device` - The WebGPU device for creating GPU resources
    /// * `queue` - The WebGPU queue for command submission
    /// * `number_indirect_commands` - Number of indirect draw commands to issue (one per chunk)
    /// * `visible_sides` - List of block sides that should be rendered (for face culling)
    /// * `ui_visible` - Flag indicating whether UI elements should be rendered
    ///
    /// # Performance Considerations
    /// * Delegates efficient batch rendering to specialized renderers
    /// * Uses GPU timestamp queries to measure rendering performance
    /// * Combines world and UI rendering in a single pass when possible
    ///
    /// # Panics
    /// Panics if the surface texture cannot be acquired or if the render pass encounters an error
    pub fn render(
        &mut self,
        surface: &Surface,
        device: StSystem<Device>,
        queue: StSystem<Queue>,
        number_indirect_commands: u32,
        visible_sides: &[BlockSide],
        ui_visible: bool,
    ) {
        let frame = match surface.get_current_texture() {
            Ok(frame) => frame,
            Err(err) => {
                error!("Error getting current frame: {:?}", err);
                panic!();
            }
        };

        let timestamp_writes = self.query_manager.request_timestamp_writes();

        let view = frame.texture.create_view(&Default::default());
        let mut encoder = device.get().create_command_encoder(&Default::default());
        {
            let depth_stencil_attachment = Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            });
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment,
                timestamp_writes,
                ..Default::default()
            });
            // Render voxel meshes using the meshing renderer
            self.meshing_renderer.render(&mut rpass, visible_sides, number_indirect_commands);

            // Render UI elements in the same render pass if they should be visible
            if ui_visible {
                self.ui_renderer.get().render(&mut rpass, self.ui_mesh_manager.clone());
            }
        }

        self.query_manager.request_gpu_query(&mut encoder);
        let command_buffer = encoder.finish();
        queue.get().submit([command_buffer]);
        frame.present();

        let _ = self.query_manager.request_read_results(queue);
    }

    /// Handles window resize events by recreating the depth texture.
    ///
    /// # Arguments
    /// * `device` - The WebGPU device
    /// * `config` - The new surface configuration containing the updated size
    pub fn resize(&mut self, device: StSystem<Device>, config: &SurfaceConfiguration) {
        self.depth_texture =
            texture::Texture::create_depth_texture(&device.get(), config, "DEPTH TEXTURE");
    }
}
