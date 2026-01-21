//! # Engine State Module
//!
//! The core engine module that manages the state and functionality of the voxel engine.
//!
//! ## Key Components
//!
//! * `EngineState` - The main state container for the engine
//! * `buffer_state` - Manages GPU buffers for rendering
//! * `camera_state` - Handles camera positioning and movement
//! * `rendering` - Contains rendering systems and pipelines
//! * `task_management` - Manages asynchronous tasks and worker threads
//! * `voxels` - Handles voxel data, chunks, and world generation
//!
//! ## Architecture
//!
//! The engine state module follows a component-based architecture where each subsystem
//! is responsible for a specific aspect of the engine's functionality. The `EngineState`
//! struct serves as the central coordinator, maintaining references to all subsystems
//! and facilitating communication between them.
//!
//! ## Performance Considerations
//!
//! * Task-based parallelism for CPU-intensive operations
//! * Efficient memory management with resource pooling
//! * Optimized rendering pipelines for voxel geometry
//! * Chunk-based loading and unloading based on player position

use std::time::Duration;

use camera_state::{camera, CameraState, CameraUpdates};
use cgmath::Point3;
use log;
use rendering::MeshRendererManager;
use task_management::TaskManager;
use voxels::{
    block::block_side::BlockSide, tasks::chunk_generation_task::ChunkGenerationTask, world::World,
};
use web_time;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::keyboard::KeyCode;

use crate::{
    application_state::input_state::{ProcessedInputState, RawInputState},
    core::{
        injection_system::{MtInjectionSystem, StInjectionSystem},
        MtResource, StSystem,
    },
};

mod buffer_state;
mod camera_state;
mod rendering;
mod task_management;
mod voxels;

/// Constant defining the render distance in chunks
const RENDER_DISTANCE: usize = 2;

/// Flags controlling engine behavior and rendering options
#[derive(Default)]
pub struct EngineFlags {
    /// Whether the UI is currently visible
    pub ui_visible: bool,
    /// Whether the rectangle should be colored red (true) or gray (false)
    pub rectangle_red: bool,
}

/// The main state container for the voxel engine
///
/// This struct maintains references to all major subsystems and coordinates
/// their interactions. It handles input processing, task management, rendering,
/// and world updates.
///
/// # Examples
///
/// ```
/// let engine_state = EngineState::new(
///     surface,
///     surface_config,
///     device,
///     queue,
///     shader_string,
///     ui_shader_string,
///     atlas_rgba_bytes,
/// );
///
/// // Main game loop
/// loop {
///     engine_state.process_input(delta_time);
///     engine_state.process_tasks();
///     engine_state.render();
/// }
/// ```
pub struct EngineState {
    /// Camera state managing position, orientation and movement
    pub camera_state: CameraState,
    /// Current player actions derived from input
    pub player_actions: PlayerAction,
    /// Buffer state for managing GPU buffers
    pub buffer_state: StSystem<buffer_state::BufferState>,
    /// Manager for mesh rendering operations
    pub render_manager: MeshRendererManager,
    /// Task manager for asynchronous operations
    pub task_manager: TaskManager,
    /// The voxel world containing all chunk data
    pub world: MtResource<World>,
    /// Reference to the GPU device
    pub device: StSystem<Device>,
    /// Currently visible block sides for culling
    pub visible_sides: Vec<BlockSide>,
    /// Engine configuration flags
    flags: EngineFlags,
    /// Current chunk position of the player
    current_player_chunk_position: Point3<i32>,
    /// Reference to the GPU queue
    pub queue: StSystem<Queue>,
}

impl EngineState {
    /// Creates a new engine state with all subsystems initialized
    ///
    /// # Arguments
    ///
    /// * `surface` - The rendering surface
    /// * `surface_config` - Configuration for the rendering surface
    /// * `device` - The GPU device
    /// * `queue` - The GPU command queue
    /// * `shader_string` - WGPU shader code for the main renderer
    /// * `ui_shader_string` - WGPU shader code for UI rendering
    /// * `atlas_rgba_bytes` - Texture atlas data in RGBA format
    ///
    /// # Returns
    ///
    /// A fully initialized `EngineState` instance
    pub fn new(
        surface: Surface<'static>,
        surface_config: SurfaceConfiguration,
        device: Device,
        queue: Queue,
        shader_string: String,
        ui_shader_string: String,
        atlas_rgba_bytes: Vec<u8>,
    ) -> Self {
        let mt_injection_system = MtInjectionSystem::new();
        let st_injection_system = StInjectionSystem::new();

        let queue = st_injection_system.insert(queue);

        let device = st_injection_system.insert(device);

        let buffer_state = st_injection_system.insert(buffer_state::BufferState::new(
            device.clone(),
            queue.clone(),
        ));

        let camera_projection = camera::Projection::new(
            surface_config.width,
            surface_config.height,
            cgmath::Deg(45.0),
            0.1,
            1000.0,
        );

        let camera_state = CameraState::new(buffer_state.clone(), &camera_projection);

        let mut render_manager = MeshRendererManager::new(
            surface,
            surface_config,
            shader_string,
            ui_shader_string,
            atlas_rgba_bytes,
            camera_projection,
            mt_injection_system.clone(),
            st_injection_system.clone(),
        );

        let mut task_manager =
            TaskManager::new(4, st_injection_system.clone(), mt_injection_system.clone());

        let world = MtResource::new(World::new());

        let mut chunk_positions = Vec::new();

        let render_distance = RENDER_DISTANCE as i32;

        for x in -render_distance..render_distance {
            for y in -render_distance..render_distance {
                for z in -render_distance..render_distance {
                    chunk_positions.push(Point3::new(x, y, z));
                }
            }
        }

        for position in chunk_positions {
            task_manager.publish_task(Box::new(ChunkGenerationTask::new(world.clone(), position)));
        }

        // Add a centered rectangle that takes up the center quarter of the screen
        let light_grey = wgpu::Color {
            r: 0.784,
            g: 0.784,
            b: 0.784,
            a: 1.0,
        };
        render_manager.ui_mesh_manager().get_mut().add_centered_rectangle("centered_rect", (0.25, 0.25), light_grey);

        render_manager.ui_mesh_manager().get_mut().add_rectangle("top_rect", (-0.5, 0.5), (1.0, 0.05), light_grey);

        Self {
            camera_state,
            player_actions: PlayerAction::default(),
            buffer_state,
            render_manager,
            task_manager,
            world,
            device,
            visible_sides: BlockSide::all().to_vec(),
            flags: EngineFlags::default(),
            current_player_chunk_position: Point3::new(0, 0, 0),
            queue,
        }
    }

    /// Resizes the rendering surface when the window size changes
    ///
    /// # Arguments
    ///
    /// * `size` - The new physical size of the window
    pub fn resize_surface(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.render_manager.resize_surface(size);
    }

    /// Renders the current frame
    ///
    /// This method triggers the rendering pipeline to draw the current state
    /// of the world and UI to the screen.
    pub fn render(&mut self) {
        self.render_manager.render(&self.visible_sides, self.flags.ui_visible);
    }

    /// Processes completed and queued tasks
    ///
    /// This method should be called each frame to ensure that asynchronous
    /// tasks like chunk generation are processed.
    pub fn process_tasks(&mut self) {
        self.task_manager
            .process_completed_tasks(&self.buffer_state.get());
        self.task_manager.process_queued_tasks();
    }

    /// Processes input and updates the camera and world state
    ///
    /// # Arguments
    ///
    /// * `wait_duration` - The time elapsed since the last frame
    pub fn process_input(&mut self, wait_duration: web_time::Duration) {
        self.camera_state.intake_actions(&self.player_actions);
        
        // Handle rectangle color toggle
        if self.player_actions.toggle_rectangle_color {
            // Define colors for the toggle
            let color = if self.flags.rectangle_red {
                wgpu::Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 } // Bright red
            } else {
                wgpu::Color { r: 0.5, g: 0.5, b: 0.5, a: 1.0 } // Gray
            };

            self.render_manager.ui_mesh_manager().get_mut().remove_element("centered_rect");
            self.render_manager.ui_mesh_manager().get_mut().add_centered_rectangle("centered_rect", (0.25, 0.25), color);

            self.render_manager.ui_mesh_manager().get_mut().update_rectangle_color(
                "centered_rect",
                color
            );

        }
        
        if let Some(CameraUpdates {
            new_visible_sides,
            new_chunk_position,
        }) = self
            .camera_state
            .update(wait_duration, &self.render_manager.camera_projection)
        {
            self.visible_sides = new_visible_sides;
            if self.current_player_chunk_position != new_chunk_position {
                // Need to be more intelligent about this, but for now just load the whole world
                let render_distance = RENDER_DISTANCE as i32;
                let mut chunks_to_load = Vec::new();
                for x in -render_distance..render_distance {
                    for y in -render_distance..render_distance {
                        for z in -render_distance..render_distance {
                            chunks_to_load.push(Point3::new(
                                new_chunk_position.x + x,
                                new_chunk_position.y + y,
                                new_chunk_position.z + z,
                            ));
                        }
                    }
                }

                // Load new chunks
                for chunk_pos in chunks_to_load {
                    self.task_manager
                        .publish_task(Box::new(ChunkGenerationTask::new(
                            self.world.clone(),
                            chunk_pos,
                        )));
                }

                self.current_player_chunk_position = new_chunk_position;
            }
        }

        if self.player_actions.get_device_details {
            log::error!("{:?}", self.device.get().features());
        }
    }

    /// Sets the input commands for the engine state.
    /// 
    /// # Arguments
    /// * `input` - The processed input state to use for setting commands
    pub fn set_input_commands(&mut self, input: ProcessedInputState) {
        let player_action = self.translate_processed_input(input);
        self.player_actions = player_action;
        
        // Log buffer information if requested
        if self.player_actions.get_buffer_data {
            log::error!(
                "Total allocated memory: {}",
                self.buffer_state.get().get_total_allocated_memory()
            );
            log::error!(
                "Total used memory: {}",
                self.buffer_state.get().get_total_used_memory()
            );
        }
    }

    /// Translates the processed input state into player actions.
    /// 
    /// # Arguments
    /// * `input` - The processed input state to translate
    /// 
    /// # Returns
    /// A PlayerAction struct with the appropriate actions set
    fn translate_processed_input(&mut self, input: ProcessedInputState) -> PlayerAction {
        let mut player_action = PlayerAction::default();

        // Movement actions - active if key is pressed or held
        player_action.move_forward = input.get_key_state(KeyCode::KeyW).is_active();
        player_action.move_backward = input.get_key_state(KeyCode::KeyS).is_active();
        player_action.move_left = input.get_key_state(KeyCode::KeyA).is_active();
        player_action.move_right = input.get_key_state(KeyCode::KeyD).is_active();
        player_action.move_up = input.get_key_state(KeyCode::Space).is_active();
        player_action.move_down = input.get_key_state(KeyCode::ShiftLeft).is_active();

        // Mouse rotation - active if left button is pressed or held & mouse has moved
        if input.get_mouse_delta().is_some() && input.get_mouse_button_state(winit::event::MouseButton::Left).is_active() {
            player_action.rotate_view = input.mouse_delta;
        }

        // I key action - only trigger on press, not hold
        if input.get_key_state(KeyCode::KeyI).is_just_pressed() {
            // Toggle UI visibility when 'I' key is pressed
            self.flags.ui_visible = !self.flags.ui_visible;
            player_action.toggle_ui_visibility = true;
        }
        
        // L key action - only trigger on press, not hold
        if input.get_key_state(KeyCode::KeyL).is_just_pressed() {
            // Toggle rectangle color when 'L' key is pressed
            self.flags.rectangle_red = !self.flags.rectangle_red;
            player_action.toggle_rectangle_color = true;
        }

        player_action
    }
}

/// Represents player actions derived from input
///
/// This struct contains flags for various player actions that can be
/// triggered by input, such as movement, camera control, and debug actions.
#[derive(Default)]
pub struct PlayerAction {
    /// Movement actions - true if key is pressed or held
    move_forward: bool,
    move_backward: bool,
    move_left: bool,
    move_right: bool,
    move_up: bool,
    move_down: bool,
    
    /// View rotation - Some if mouse is pressed or held
    rotate_view: Option<(f64, f64)>,
    
    /// Actions that should only trigger on key press, not hold
    get_buffer_data: bool,
    get_device_details: bool,
    toggle_ui_visibility: bool,
    toggle_rectangle_color: bool,
}
