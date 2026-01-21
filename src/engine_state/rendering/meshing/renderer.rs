//! Meshing renderer module for the voxel engine.
//!
//! This module handles the rendering of voxel meshes using a bucket-based approach
//! for efficient multi-draw-indirect rendering.
//!
//! # Architecture
//!
//! The meshing renderer is responsible for:
//! 1. Creating and managing its own render pipeline
//! 2. Setting up the appropriate bind groups for rendering
//! 3. Executing multi-draw-indirect commands for each visible block side
//!
//! # Performance Considerations
//!
//! - Uses multi-draw-indirect for efficient batch rendering of chunks
//! - Implements face culling based on visible_sides to reduce overdraw
//! - Organizes rendering by block side for optimal GPU utilization

use wgpu::{
    Device, Queue, RenderPipeline, RenderPass, ShaderModule, TextureFormat, SurfaceConfiguration,
};

use crate::{
    core::StSystem,
    engine_state::{
        buffer_state::BufferState,
        rendering::bind_group_state::{BindGroupState, CAMERA_BIND_GROUP_LAYOUT, TEXTURE_BIND_GROUP_LAYOUT, CHUNK_INDEX_BIND_GROUP_LAYOUT},
        voxels::{
            block::block_side::BlockSide,
        },
    },
};
use crate::engine_state::rendering::bind_group_state::{CAMERA_BIND_GROUP, CHUNK_INDEX_BIND_GROUP, TEXTURE_BIND_GROUP};
use crate::engine_state::rendering::Vertex;
use super::MeshManager;

/// Manages mesh rendering in the voxel engine.
///
/// This struct is responsible for rendering voxel meshes using
/// a bucket-based approach with multi-draw-indirect for efficiency.
/// It encapsulates the complete rendering pipeline for voxel meshes,
/// including pipeline creation, bind group setup, and draw commands.
pub struct MeshingRenderer {
    /// The WebGPU render pipeline for mesh rendering
    render_pipeline: RenderPipeline,
    /// Shared state for buffer management
    buffer_state: StSystem<BufferState>,
    /// Shared state for bind group management
    bind_group_state: StSystem<BindGroupState>,
}

impl MeshingRenderer {
    /// Creates a new `MeshingRenderer` instance.
    ///
    /// # Arguments
    /// * `device` - The WebGPU device
    /// * `buffer_state` - Shared state for buffer management
    /// * `shader_string` - The WGSL shader source code
    /// * `texture_format` - The texture format to use for rendering
    /// * `bind_group_state` - State for managing bind groups
    /// * `depth_stencil` - Optional depth stencil state
    ///
    /// # Returns
    /// A new `MeshingRenderer` instance with initialized render pipeline
    ///
    /// # Implementation Details
    ///
    /// - Creates a specialized render pipeline for mesh rendering
    /// - Sets up the appropriate pipeline layout with necessary bind groups
    /// - Configures vertex and fragment shaders for voxel rendering
    pub fn new(
        device: StSystem<Device>,
        buffer_state: StSystem<BufferState>,
        shader_string: &str,
        texture_format: TextureFormat,
        bind_group_state: StSystem<BindGroupState>,
        depth_stencil: Option<wgpu::DepthStencilState>,
    ) -> Self {
        let device_ref = device.get();
        
        // Create the pipeline layout
        let pipeline_layout = device_ref.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Mesh Render Pipeline Layout"),
            bind_group_layouts: &[
                bind_group_state.get().get_bind_group_layout(CAMERA_BIND_GROUP_LAYOUT),
                bind_group_state.get().get_bind_group_layout(TEXTURE_BIND_GROUP_LAYOUT),
                bind_group_state.get().get_bind_group_layout(CHUNK_INDEX_BIND_GROUP_LAYOUT),
            ],
            push_constant_ranges: &[],
        });
        
        // Create the shader module
        let shader = device_ref.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Mesh Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_string.into()),
        });
        
        // Create the render pipeline
        let render_pipeline = device_ref.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Mesh Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });
        
        Self {
            render_pipeline,
            buffer_state,
            bind_group_state: bind_group_state.clone(),
        }
    }

    /// Renders all visible mesh sides using multi-draw-indirect.
    ///
    /// # Arguments
    /// * `render_pass` - The render pass to use for rendering
    /// * `visible_sides` - List of block sides that should be rendered (for face culling)
    /// * `number_indirect_commands` - Number of indirect draw commands to issue (one per chunk)
    ///
    /// # Implementation Details
    ///
    /// - Sets the pipeline and bind groups for rendering
    /// - Iterates through all block sides and renders only the visible ones
    /// - Uses multi-draw-indirect for efficient batch rendering
    /// - Accesses vertex, index, and indirect buffers from the buffer state
    pub fn render<'a, 'b>(
        &'a self,
        render_pass: &mut RenderPass<'b>,
        visible_sides: &[BlockSide],
        number_indirect_commands: u32,
    ) where 'a: 'b {
        // Set the pipeline
        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_bind_group(
            0,
            self.bind_group_state.get().get_bind_group(CAMERA_BIND_GROUP),
            &[],
        );
        render_pass.set_bind_group(
            1,
            self.bind_group_state.get().get_bind_group(TEXTURE_BIND_GROUP),
            &[],
        );
        render_pass.set_bind_group(
            2,
            self.bind_group_state.get().get_bind_group(CHUNK_INDEX_BIND_GROUP),
            &[],
        );
        
        // Render all visible sides using multi-draw-indirect
        for side in BlockSide::all() {
            if !visible_sides.contains(&side) {
                continue;
            }

            let vertex_buffer_name = MeshManager::get_vertex_buffer_name(side);
            let index_buffer_name = MeshManager::get_index_buffer_name(side);
            let indirect_buffer_name = MeshManager::get_indirect_buffer_name(side);

            render_pass.set_vertex_buffer(
                0,
                self.buffer_state
                    .get()
                    .get_buffer(vertex_buffer_name)
                    .slice(..),
            );
            render_pass.set_index_buffer(
                self.buffer_state
                    .get()
                    .get_buffer(index_buffer_name)
                    .slice(..),
                wgpu::IndexFormat::Uint32,
            );
            render_pass.multi_draw_indexed_indirect(
                self.buffer_state.get().get_buffer(indirect_buffer_name),
                0,
                number_indirect_commands,
            );
        }
    }
    
    /// Gets the render pipeline.
    ///
    /// # Returns
    /// Reference to the render pipeline
    pub fn get_render_pipeline(&self) -> &RenderPipeline {
        &self.render_pipeline
    }
}