//! UI rendering module for the voxel engine.
//!
//! This module handles the rendering of UI elements on top of the 3D scene.
//! The UIRenderer is responsible for creating the pipeline, managing bind groups,
//! and implementing a render method for the pipeline manager to call.

use wgpu::{
    Device, Queue, RenderPipeline, ShaderModule, TextureFormat,
    BindGroupLayout, PipelineLayout, SurfaceConfiguration, DepthStencilState,
    RenderPass,
};
use crate::core::StSystem;
use crate::engine_state::buffer_state::BufferState;
use crate::engine_state::rendering::ui::manager::buffer_names::{UI_INDEX_BUFFER, UI_VERTEX_BUFFER};
use crate::engine_state::rendering::ui::UiMeshManager;
use super::primitives::UiVertex;

/// Manages UI rendering in the voxel engine.
///
/// This struct is responsible for setting up the UI rendering pipeline
/// and drawing UI elements on top of the 3D scene.
pub struct UiRenderer {
    /// The WebGPU render pipeline for UI elements
    render_pipeline: RenderPipeline,
    buffer_state: StSystem<BufferState>,
}

impl UiRenderer {
    /// Creates a new `UiRenderer` instance.
    ///
    /// # Arguments
    /// * `device` - The WebGPU device
    /// * `config` - Surface configuration
    /// * `format` - Texture format for the surface
    /// * `depth_stencil` - Optional depth stencil state
    /// * `ui_shader_source` - Source code for the UI shader
    /// * `buffer_state` - Reference to the buffer state for managing GPU buffers
    ///
    /// # Returns
    /// A new `UiRenderer` instance
    pub fn new(
        device: &Device,
        config: &SurfaceConfiguration,
        format: TextureFormat,
        depth_stencil: Option<DepthStencilState>,
        ui_shader_source: &str,
        buffer_state: StSystem<BufferState>,
    ) -> Self {
        // Create a simple shader for UI rendering
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("UI Shader"),
            source: wgpu::ShaderSource::Wgsl(ui_shader_source.into()),
        });

        // Create the render pipeline
        let render_pipeline = Self::create_render_pipeline(
            device,
            &shader,
            format,
            depth_stencil,
        );

        Self {
            render_pipeline,
            buffer_state,
        }
    }

    /// Creates a render pipeline for UI rendering.
    ///
    /// # Arguments
    /// * `device` - The WebGPU device
    /// * `shader` - The shader module containing vertex and fragment shaders
    /// * `format` - The texture format for the render target
    /// * `depth_stencil` - Optional depth stencil state
    ///
    /// # Returns
    /// A new render pipeline configured for UI rendering
    fn create_render_pipeline(
        device: &Device,
        shader: &ShaderModule,
        format: TextureFormat,
        depth_stencil: Option<DepthStencilState>,
    ) -> RenderPipeline {
        // Create a pipeline layout (no bind groups needed for basic UI)
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UI Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        // Define the vertex buffer layout
        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<UiVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position attribute
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Color attribute
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }];

        // Create the render pipeline
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &vertex_buffers,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
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
            depth_stencil,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        })
    }

    /// Renders the UI elements.
    ///
    /// # Arguments
    /// * `render_pass` - The render pass to draw with
    /// * `ui_mesh_manager` - The UI mesh manager containing the elements to render
    pub fn render<'a>(
        &self,
        render_pass: &mut RenderPass<'a>,
        ui_mesh_manager: StSystem<UiMeshManager>,
    ) {
        // Skip rendering if there are no UI elements
        if ui_mesh_manager.get().is_empty() {
            return;
        }

        // Set the pipeline for UI rendering
        render_pass.set_pipeline(&self.render_pipeline);
        
        // Set the vertex and index buffers from the UI mesh manager
        render_pass.set_vertex_buffer(
            0,
            self.buffer_state.get().get_buffer(UI_VERTEX_BUFFER).slice(..),
        );
        render_pass.set_index_buffer(
            self.buffer_state.get().get_buffer(UI_INDEX_BUFFER).slice(..),
            wgpu::IndexFormat::Uint32,
        );
        
        // Draw the UI elements
        render_pass.draw_indexed(0..ui_mesh_manager.get().get_index_count(), 0, 0..1);
    }
}
