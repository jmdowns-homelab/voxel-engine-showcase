//! # Camera State Management
//!
//! This module handles all camera-related functionality including:
//! - Camera position and orientation tracking
//! - View and projection matrix calculations
//! - Player input processing for camera control
//! - Chunk visibility determination based on camera position
//!
//! ## Core Components
//! - `Camera`: Represents the camera's position and orientation in 3D space
//! - `CameraController`: Handles player input and updates camera state
//! - `Projection`: Manages the camera's projection matrix
//! - `CameraUniform`: GPU representation of camera data for shaders
//!
//! ## Key Features
//! - First-person camera controls (WASD, mouse look)
//! - Chunk-based position tracking for world interaction
//! - Efficient updates to GPU buffers
//! - Support for perspective projection

use camera::CameraController;
use cgmath::Point3;

use crate::core::StSystem;

use super::{
    buffer_state::BufferState,
    voxels::{block::block_side::BlockSide, chunk::CHUNK_DIMENSION},
    PlayerAction,
};

pub mod camera;

/// Manages the complete camera system including state, controls, and GPU resources.
///
/// This is the main interface for interacting with the camera system. It handles:
/// - Camera positioning and orientation
/// - Input processing
/// - GPU buffer updates
/// - Chunk-based position tracking
///
/// # Fields
/// - `camera`: The current camera state (position, orientation)
/// - `camera_uniform`: GPU-optimized camera data for shaders
/// - `camera_controller`: Handles player input and camera movement
/// - `buffer_state`: Manages GPU buffer state
pub struct CameraState {
    /// The current camera position and orientation
    pub camera: camera::Camera,
    /// GPU-optimized camera data for shaders
    pub camera_uniform: camera::CameraUniform,
    /// Handles player input and camera movement
    pub camera_controller: camera::CameraController,
    /// Manages GPU buffer state for camera data
    pub buffer_state: StSystem<BufferState>,
}

/// Name of the GPU buffer used for camera uniform data
pub const CAMERA_BUFFER_NAME: &str = "camera_buffer";

impl CameraState {
    /// Creates a new CameraState with default values.
    ///
    /// # Arguments
    /// * `buffer_state` - The buffer state system for GPU resource management
    /// * `projection` - The initial camera projection settings
    ///
    /// # Returns
    /// A new `CameraState` instance with the camera positioned at the origin
    pub fn new(buffer_state: StSystem<BufferState>, projection: &camera::Projection) -> Self {
        let camera_position = Point3::new(0.0, 0.0, 0.0);
        let camera = camera::Camera::new(camera_position, cgmath::Deg(0.0), cgmath::Deg(0.0));
        let camera_controller = CameraController::new(2.0, 2.0);

        let mut camera_uniform = camera::CameraUniform::new();
        camera_uniform.update_view_proj_and_pos(&camera, projection);

        let mut buffer_state_write = buffer_state.get_mut();

        buffer_state_write.create_buffer_init(
            CAMERA_BUFFER_NAME,
            wgpu::util::BufferInitDescriptor {
                label: Some(CAMERA_BUFFER_NAME),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        );

        CameraState {
            camera,
            camera_uniform,
            camera_controller,
            buffer_state: buffer_state.clone(),
        }
    }

    /// Processes player input actions and updates the camera controller state.
    ///
    /// # Arguments
    /// * `actions` - The player's input actions to process
    pub fn intake_actions(&mut self, actions: &PlayerAction) {
        self.camera_controller.intake_actions(actions);
    }

    /// Updates the camera state based on elapsed time and current projection.
    ///
    /// This method should be called every frame to:
    /// 1. Process any pending camera movements
    /// 2. Update the view and projection matrices
    /// 3. Update GPU buffers
    /// 4. Determine visible block faces based on camera orientation
    ///
    /// # Arguments
    /// * `dt` - Time elapsed since the last update
    /// * `projection` - Current camera projection settings
    ///
    /// # Returns
    /// - `Some(CameraUpdates)` if the camera position or orientation changed
    /// - `None` if no updates were needed
    pub fn update(
        &mut self,
        dt: web_time::Duration,
        projection: &camera::Projection,
    ) -> Option<CameraUpdates> {
        if self.camera_controller.has_updates() {
            self.camera
                .get_controller_updates_and_reset_controller(&mut self.camera_controller, dt);
            self.camera_uniform
                .update_view_proj_and_pos(&self.camera, projection);
            self.buffer_state.get_mut().write_buffer(
                CAMERA_BUFFER_NAME,
                0,
                bytemuck::cast_slice(&[self.camera_uniform]),
            );

            let normalized_player_direction_vec = self.camera.get_view_vec();
            let new_visible_sides = BlockSide::get_visible_sides(normalized_player_direction_vec);

            let player_position = self.camera.position;
            let player_chunk_x = (player_position.x / CHUNK_DIMENSION as f32).floor() as i32;
            let player_chunk_y = (player_position.y / CHUNK_DIMENSION as f32).floor() as i32;
            let player_chunk_z = (player_position.z / CHUNK_DIMENSION as f32).floor() as i32;
            let new_chunk_position = Point3::new(player_chunk_x, player_chunk_y, player_chunk_z);
            return Some(CameraUpdates {
                new_visible_sides,
                new_chunk_position,
            });
        }

        None
    }
}

/// Represents updates to the camera's state that affect game world interaction.
///
/// This is returned by `CameraState::update()` when the camera's position or
/// orientation has changed in a way that affects game state.
pub struct CameraUpdates {
    /// Block faces that are currently visible based on camera orientation
    pub new_visible_sides: Vec<BlockSide>,
    /// The current chunk position of the camera in world coordinates
    pub new_chunk_position: Point3<i32>,
}
