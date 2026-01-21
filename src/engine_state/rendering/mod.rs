//! Rendering system for the voxel engine.
//!
//! This module contains the core rendering functionality, including mesh management,
//! pipeline setup, and the main render loop. It provides a high-level interface
//! for rendering 3D voxel-based graphics using WebGPU.

pub use meshing::MeshManager;
use pipeline_manager::PipelineManager;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};

use crate::core::{
    injection_system::{MtInjectionSystem, StInjectionSystem},
    StSystem,
};

use super::{
    buffer_state::BufferState, camera_state::camera, voxels::block::block_side::BlockSide,
};

mod bind_group_state;
pub mod meshing;
mod pipeline_manager;
mod query_manager;
mod raw_query_manager;
pub mod tasks;
mod texture;
mod vertex;
pub mod ui;

// Re-export commonly used types
pub use vertex::Vertex;

/// Manages the entire rendering pipeline for the voxel engine.
///
/// This struct is the main entry point for all rendering operations.
/// It manages the WebGPU surface, device, queue, and rendering pipeline.
pub struct MeshRendererManager {
    /// The WebGPU surface being rendered to
    pub surface: Surface<'static>,
    /// Configuration for the surface (size, format, etc.)
    pub surface_config: SurfaceConfiguration,
    /// The WebGPU device used for creating GPU resources
    pub device: StSystem<Device>,
    /// The WebGPU queue for submitting command buffers
    pub queue: StSystem<Queue>,
    /// Manages the rendering pipeline and shaders
    pub pipeline_manager: PipelineManager,
    /// Camera projection settings
    pub camera_projection: camera::Projection,
    /// Number of indirect draw commands to issue
    pub num_indirect_commands: u32,
}

impl MeshRendererManager {
    /// Creates a new `MeshRendererManager` instance.
    ///
    /// This initializes all the necessary WebGPU resources, including:
    /// - The rendering surface and swap chain
    /// - The graphics pipeline with vertex and fragment shaders
    /// - Texture atlases and samplers
    /// - Camera and projection matrices
    ///
    /// # Arguments
    /// * `surface` - The WebGPU surface to render to
    /// * `surface_config` - Configuration for the surface
    /// * `shader_string` - WGSL source code for the shaders
    /// * `ui_shader_string` - WGSL source code for the UI shaders
    /// * `atlas_rgba_bytes` - Raw RGBA data for the texture atlas
    /// * `camera_projection` - Initial camera projection settings
    /// * `mt_injection_system` - Multi-threaded dependency injection system
    /// * `st_injection_system` - Single-threaded dependency injection system
    ///
    /// # Returns
    /// A new `MeshRendererManager` instance with all rendering resources initialized
    pub fn new(
        surface: Surface<'static>,
        surface_config: SurfaceConfiguration,
        shader_string: String,
        ui_shader_string: String,
        atlas_rgba_bytes: Vec<u8>,
        camera_projection: camera::Projection,
        mt_injection_system: MtInjectionSystem,
        st_injection_system: StInjectionSystem,
    ) -> Self {
        let buffer_state = st_injection_system.get::<BufferState>().unwrap();

        let mesh_manager = MeshManager::new(buffer_state.clone());
        let num_indirect_commands = mesh_manager.get_number_indirect_commands();
        mt_injection_system.insert(mesh_manager);

        let device = st_injection_system.get::<Device>().unwrap();
        let queue = st_injection_system.get::<Queue>().unwrap();

        let pipeline_manager = PipelineManager::new(
            device.clone(),
            queue.clone(),
            &surface_config,
            surface_config.format,
            buffer_state.clone(),
            shader_string,
            ui_shader_string,
            atlas_rgba_bytes,
        );

        Self {
            surface,
            surface_config,
            device,
            queue,
            pipeline_manager,
            camera_projection,
            num_indirect_commands,
        }
    }

    /// Handles window resize events.
    ///
    /// Updates the surface configuration, camera projection, and pipeline
    /// to match the new window size.
    ///
    /// # Arguments
    /// * `size` - The new window size in physical pixels
    pub fn resize_surface(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        // Update surface dimensions
        self.surface_config.width = size.width;
        self.surface_config.height = size.height;
        
        // Reconfigure the surface with the new size
        self.surface
            .configure(&self.device.get(), &self.surface_config);
            
        // Update camera projection and pipeline for the new aspect ratio
        self.camera_projection.resize(size.width, size.height);
        self.pipeline_manager
            .resize(self.device.clone(), &self.surface_config);
    }

    /// Renders a new frame.
    ///
    /// This is the main rendering entry point that should be called once per frame.
    /// It handles all the necessary steps to render the current scene to the surface.
    ///
    /// # Arguments
    /// * `visible_sides` - List of block sides that should be rendered (used for face culling)
    /// * `ui_visible` - Whether UI elements should be rendered
    pub fn render(&mut self, visible_sides: &[BlockSide], ui_visible: bool) {
        self.pipeline_manager.render(
            &self.surface,
            self.device.clone(),
            self.queue.clone(),
            self.num_indirect_commands,
            visible_sides,
            ui_visible,
        );
    }
    
    /// Gets a reference to the UI mesh manager.
    ///
    /// # Returns
    /// A reference to the UI mesh manager for adding and updating UI elements
    pub fn ui_mesh_manager(&self) -> &StSystem<ui::UiMeshManager> {
        &self.pipeline_manager.ui_mesh_manager
    }
    
    /// Gets a mutable reference to the UI mesh manager.
    ///
    /// # Returns
    /// A mutable reference to the UI mesh manager for adding and updating UI elements
    pub fn ui_mesh_manager_mut(&mut self) -> &mut StSystem<ui::UiMeshManager> {
        &mut self.pipeline_manager.ui_mesh_manager
    }
}
