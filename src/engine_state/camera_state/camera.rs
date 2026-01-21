//! # Camera Implementation
//!
//! This module contains the core camera implementation including:
//! - Camera representation and transformations
//! - Projection matrix handling
//! - Camera controller for input processing
//! - GPU uniform buffer management
//!
//! ## Key Components
//! - `Camera`: Represents the camera's position and orientation in 3D space
//! - `Projection`: Manages perspective projection settings
//! - `CameraController`: Handles user input for camera movement
//! - `CameraUniform`: Packed data structure for GPU shaders

use cgmath::*;
use std::f32::consts::FRAC_PI_2;
use web_time::Duration;

use crate::engine_state::PlayerAction;

/// Transformation matrix to convert from OpenGL's coordinate system to WGPU's.
///
/// WGPU uses a coordinate system where:
/// - X is right
/// - Y is up
/// - Z is forward (unlike OpenGL where Z is backward)
/// - NDC (Normalized Device Coordinates) range from -1 to 1 in X and Y, and 0 to 1 in Z
///
/// This matrix performs two main transformations:
/// 1. Scales the Z coordinate from [-1, 1] to [-0.5, 0.5]
/// 2. Translates the Z coordinate from [-0.5, 0.5] to [0, 1]
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,  // Scale Z from [-1,1] to [-0.5,0.5]
    0.0, 0.0, 0.5, 1.0,  // Translate Z from [-0.5,0.5] to [0,1]
);

/// Safe limit for pitch to prevent gimbal lock
const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

/// Represents a first-person camera in 3D space.
///
/// The camera maintains its position and orientation in the world,
/// and provides methods for view matrix calculation and movement.
///
/// # Fields
/// - `position`: The camera's position in world space
/// - `yaw`: Horizontal rotation (around Y axis) in radians
/// - `pitch`: Vertical rotation (around X axis) in radians
/// - `view_x_vec`: Normalized vector pointing to the camera's right
/// - `view_y_vec`: Normalized vector pointing to the camera's up
/// - `view_z_vec`: Normalized vector pointing to the camera's forward
#[derive(Debug)]
pub struct Camera {
    /// The camera's position in world space
    pub position: Point3<f32>,
    /// Horizontal rotation (around Y axis) in radians
    pub yaw: Rad<f32>,
    /// Vertical rotation (around X axis) in radians
    pub pitch: Rad<f32>,
    /// Normalized vector pointing to the camera's right
    pub view_x_vec: cgmath::Vector3<f32>,
    /// Normalized vector pointing to the camera's up
    pub view_y_vec: cgmath::Vector3<f32>,
    /// Normalized vector pointing to the camera's forward
    pub view_z_vec: cgmath::Vector3<f32>,
}

impl Camera {
    /// Creates a new camera with the specified position and orientation.
    ///
    /// # Arguments
    /// * `position` - Initial position of the camera in world space. Can be any type that converts to `Point3<f32>`.
    /// * `yaw` - Initial yaw (horizontal rotation around Y axis). Can be any type that converts to `Rad<f32>`.
    /// * `pitch` - Initial pitch (vertical rotation around X axis). Can be any type that converts to `Rad<f32>`.
    ///
    /// # Returns
    /// A new `Camera` instance with the specified position and orientation.
    ///
    /// # Example
    /// ```rust
    /// use cgmath::{Point3, Deg};
    /// let camera = Camera::new(
    ///     Point3::new(0.0, 0.0, 0.0),  // Position at origin
    ///     Deg(0.0),                     // Facing along positive Z
    ///     Deg(0.0),                     // Level horizon
    /// );
    /// ```
    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        let yaw_rad = yaw.into();
        let pitch_rad = pitch.into();
        let (yaw_sin, yaw_cos) = yaw_rad.sin_cos();
        let (pitch_sin, pitch_cos) = pitch_rad.sin_cos();
        Self {
            position: position.into(),
            yaw: yaw_rad,
            pitch: pitch_rad,
            view_x_vec: cgmath::Vector3::new(yaw_cos, pitch_sin, yaw_sin).normalize(),
            view_y_vec: cgmath::Vector3::new(-pitch_sin, pitch_cos, 0.0).normalize(),
            view_z_vec: cgmath::Vector3::new(-yaw_sin, 0.0, -yaw_cos).normalize(),
        }
    }

    /// Gets the camera's forward direction vector.
    ///
    /// This is a normalized vector pointing in the direction the camera is facing.
    ///
    /// # Returns
    /// A normalized 3D vector representing the camera's forward direction
    pub fn get_view_vec(&self) -> cgmath::Vector3<f32> {
        self.view_x_vec
    }

    /// Calculates the view matrix for this camera.
    ///
    /// The view matrix transforms world coordinates to view (camera) space.
    ///
    /// # Returns
    /// A 4x4 view matrix that can be used for rendering
    pub fn calc_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_to_rh(
            self.position,
            Vector3::new(self.yaw.0.cos(), self.pitch.0.sin(), self.yaw.0.sin()).normalize(),
            Vector3::unit_y(),
        )
    }

    /// Updates the camera's position and orientation based on controller input.
    ///
    /// # Arguments
    /// * `controller` - The camera controller containing input state
    /// * `dt` - Time elapsed since the last update
    ///
    /// # Notes
    /// - Handles movement (WASD, space/shift for up/down)
    /// - Handles rotation (mouse look)
    /// - Updates all camera vectors to maintain orthonormality
    pub fn get_controller_updates_and_reset_controller(
        &mut self,
        controller: &mut CameraController,
        dt: Duration,
    ) {
        let dt = dt.as_secs_f32();

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = self.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        self.position += forward
            * (controller.amount_forward - controller.amount_backward)
            * controller.speed
            * dt;
        self.position +=
            right * (controller.amount_right - controller.amount_left) * controller.speed * dt;

        // Move in/out (zoom)
        let (pitch_sin, pitch_cos) = self.pitch.0.sin_cos();
        let scrollward =
            Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        self.position +=
            scrollward * controller.scroll * controller.speed * controller.sensitivity * dt;
        controller.scroll = 0.0;

        // Move up/down
        self.position.y += (controller.amount_up - controller.amount_down) * controller.speed * dt;

        // Rotate
        self.yaw += Rad(controller.rotate_horizontal) * controller.sensitivity * dt;
        self.pitch += Rad(-controller.rotate_vertical) * controller.sensitivity * dt;

        // Reset controller state
        controller.rotate_horizontal = 0.0;
        controller.rotate_vertical = 0.0;
        controller.amount_up = 0.0;
        controller.amount_down = 0.0;
        controller.amount_left = 0.0;
        controller.amount_right = 0.0;
        controller.amount_forward = 0.0;
        controller.amount_backward = 0.0;

        // Clamp pitch to prevent gimbal lock
        if self.pitch < -Rad(SAFE_FRAC_PI_2) {
            self.pitch = -Rad(SAFE_FRAC_PI_2);
        } else if self.pitch > Rad(SAFE_FRAC_PI_2) {
            self.pitch = Rad(SAFE_FRAC_PI_2);
        }

        // Update camera vectors
        self.view_x_vec.x = self.pitch.cos() * self.yaw.cos();
        self.view_x_vec.y = self.pitch.sin();
        self.view_x_vec.z = self.pitch.cos() * self.yaw.sin();
        self.view_x_vec = self.view_x_vec.normalize();

        self.view_y_vec.x = -self.pitch.sin() * self.yaw.cos();
        self.view_y_vec.y = self.pitch.cos();
        self.view_y_vec.z = -self.pitch.sin() * self.yaw.sin();
        self.view_y_vec = self.view_y_vec.normalize();

        self.view_z_vec.x = -self.yaw.sin();
        self.view_z_vec.z = self.yaw.cos();
        self.view_z_vec = self.view_z_vec.normalize();
    }
}

/// Represents a camera's projection matrix and related parameters.
///
/// This handles the perspective projection used to render the 3D scene.
/// It manages the aspect ratio, field of view, and near/far clipping planes.
#[derive(Debug)]
pub struct Projection {
    /// Aspect ratio (width / height)
    aspect: f32,
    /// Vertical field of view in radians
    fovy: Rad<f32>,
    /// Near clipping plane distance
    znear: f32,
    /// Far clipping plane distance
    zfar: f32,
}

impl Projection {
    /// Creates a new projection with the given parameters.
    ///
    /// # Arguments
    /// * `width` - Viewport width in pixels
    /// * `height` - Viewport height in pixels
    /// * `fovy` - Vertical field of view (can be any type convertible to `Rad<f32>`)
    /// * `znear` - Near clipping plane distance
    /// * `zfar` - Far clipping plane distance
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        let aspect = width as f32 / height as f32;
        let fovy: Rad<f32> = fovy.into();
        Self {
            aspect,
            fovy,
            znear,
            zfar,
        }
    }

    /// Updates the projection's aspect ratio for viewport resizing.
    ///
    /// # Arguments
    /// * `width` - New viewport width in pixels
    /// * `height` - New viewport height in pixels
    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    /// Calculates the projection matrix.
    ///
    /// Combines the perspective projection with the OpenGL to WGPU coordinate system transform.
    ///
    /// # Returns
    /// A 4x4 projection matrix ready for use in shaders
    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

/// Handles camera movement and rotation based on user input.
///
/// This struct tracks the current state of movement keys and mouse input,
/// and applies them to the camera when updated.
#[derive(Debug)]
pub struct CameraController {
    // Movement amounts (normalized)
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    
    // Rotation amounts (in radians)
    rotate_horizontal: f32,
    rotate_vertical: f32,
    
    // Zoom/scroll amount
    scroll: f32,
    
    // Configuration
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    /// Creates a new camera controller with the given speed and sensitivity.
    ///
    /// # Arguments
    /// * `speed` - Base movement speed in units per second
    /// * `sensitivity` - Mouse look sensitivity multiplier
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    /// Processes player actions and updates controller state accordingly.
    ///
    /// # Arguments
    /// * `actions` - The player's input actions to process
    pub fn intake_actions(&mut self, actions: &PlayerAction) {
        if actions.move_forward {
            self.amount_forward = self.speed;
        }
        if actions.move_backward {
            self.amount_backward = self.speed;
        }
        if actions.move_left {
            self.amount_left = self.speed;
        }
        if actions.move_right {
            self.amount_right = self.speed;
        }
        if actions.move_up {
            self.amount_up = self.speed;
        }
        if actions.move_down {
            self.amount_down = self.speed;
        }
        if let Some((delta_x, delta_y)) = actions.rotate_view {
            if delta_x.abs() > 0.5 {
                self.rotate_horizontal = (delta_x as f32) * self.sensitivity;
            }
            if delta_y.abs() > 0.5 {
                self.rotate_vertical = (delta_y as f32) * self.sensitivity;
            }
        }
    }

    /// Checks if there are any pending updates that would affect the camera.
    ///
    /// # Returns
    /// `true` if there are pending updates, `false` otherwise
    pub fn has_updates(&self) -> bool {
        self.amount_forward > 0.0
            || self.amount_backward > 0.0
            || self.amount_left > 0.0
            || self.amount_right > 0.0
            || self.amount_up > 0.0
            || self.amount_down > 0.0
            || self.rotate_horizontal != 0.0
            || self.rotate_vertical != 0.0
    }
}

/// GPU-friendly representation of camera data for shaders.
///
/// This struct is used to pass camera data to the GPU in a format that matches
/// the layout expected by the shaders.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly so we'll have to conver the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
    view_proj_inverse: [[f32; 4]; 4],
    position: [f32; 4],
}

impl CameraUniform {
    /// Creates a new camera uniform with identity matrices and zero position.
    pub fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
            view_proj_inverse: cgmath::Matrix4::identity().into(),
            position: [0.0, 0.0, 0.0, 0.0],
        }
    }

    /// Updates the view-projection matrix and position based on the current camera state.
    ///
    /// # Arguments
    /// * `camera` - The camera to get view matrix and position from
    /// * `projection` - The projection to use
    pub fn update_view_proj_and_pos(&mut self, camera: &Camera, projection: &Projection) {
        let viewproj = projection.calc_matrix() * camera.calc_matrix();
        self.view_proj = (viewproj).into();
        self.view_proj_inverse = (viewproj.invert().unwrap()).into();
        let pos3: [f32; 3] = camera.position.into();

        self.position = [pos3[0], pos3[1], pos3[2], 0.0];
    }
}
