//! # Application State Management
//!
//! This module handles the application's state management, including:
//! - Window and graphics initialization
//! - Input handling
//! - Application lifecycle events
//! - State transitions between initialization and running states

pub mod graphics_resources_builder;
pub mod input_manager;
pub mod input_state;

use std::sync::Arc;

use graphics_resources_builder::{Graphics, MaybeGraphics};
use input_manager::InputManager;

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::engine_state::EngineState;

/// The main application state container that manages the application's lifecycle.
/// 
/// This struct holds the current state of the application, including graphics resources,
/// input handling, and window management. It implements `ApplicationHandler` to handle
/// window and device events.
pub struct ApplicationState {
    /// The current graphics state, which may be uninitialized, initializing, or ready
    pub graphics: MaybeGraphics,
    
    /// The initialized application state, if the application has started
    pub state: Option<InitializedApplicationState>,
    
    /// Cached window size for web platforms during initialization
    pub web_window_size: Option<PhysicalSize<u32>>,
}

/// Represents the fully initialized and running state of the application.
/// 
/// This struct contains all the necessary components for the running application,
/// including the game engine state, window handle, and input management.
pub struct InitializedApplicationState {
    /// The core game engine state and logic
    pub engine_state: EngineState,
    
    /// Handle to the application window
    pub window: Arc<Window>,
    
    /// Manages input state and event processing
    pub input_manager: InputManager,
    
    /// Timestamp of the last frame for delta time calculations
    pub last_wait_time: web_time::Instant,
}

impl ApplicationState {
    /// Handles window resize events during the initialization phase.
    /// 
    /// This method updates the surface configuration and triggers application state initialization
    /// if all required resources are available.
    /// 
    /// # Arguments
    /// * `size` - The new size of the window in physical pixels
    fn resized(&mut self, size: PhysicalSize<u32>) {
        let MaybeGraphics::Graphics(gfx) = &mut self.graphics else {
            return;
        };

        gfx.surface_config.as_mut().unwrap().width = size.width;
        gfx.surface_config.as_mut().unwrap().height = size.height;
        gfx.surface.as_mut().unwrap().configure(
            gfx.device.as_ref().unwrap(),
            gfx.surface_config.as_ref().unwrap(),
        );
        self.initialize_application_state();
    }

    /// Initializes the application state with the required graphics resources.
    /// 
    /// This method transitions the application from the initialization phase to the running state
    /// by setting up the engine state with the provided graphics resources.
    fn initialize_application_state(&mut self) {
        if let MaybeGraphics::Graphics(gfx) = &mut self.graphics {
            let taken_gfx = std::mem::take(gfx);
            let window = taken_gfx.window.expect("Window is missing");
            let engine_state = EngineState::new(
                taken_gfx.surface.expect("Surface is missing"),
                taken_gfx
                    .surface_config
                    .expect("Surface configuration is missing"),
                taken_gfx.device.expect("Device is missing"),
                taken_gfx.queue.expect("Queue is missing"),
                taken_gfx.shader_file_string,
                taken_gfx.ui_shader_file_string,
                taken_gfx.atlas_bytes,
            );

            let window = window.clone();

            self.state = Some(InitializedApplicationState {
                engine_state,
                window,
                input_manager: InputManager::new(),
                last_wait_time: web_time::Instant::now(),
            });

            self.graphics = MaybeGraphics::Moved;
        }
    }
}

impl ApplicationHandler<Graphics> for ApplicationState {
    /// Handles window-related events such as resize, focus changes, and input events.
    /// 
    /// This method processes window events and delegates them to the appropriate handlers
    /// based on the current application state (initialized or uninitialized).
    /// 
    /// # Arguments
    /// * `event_loop` - Reference to the active event loop
    /// * `_window_id` - ID of the window that generated the event
    /// * `event` - The window event to process
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(state) = &mut self.state {
            let input_manager = &mut state.input_manager;
            let engine_state = &mut state.engine_state;

            input_manager.intake_input(&event);

            match event {
                WindowEvent::Resized(size) => {
                    engine_state.resize_surface(size);
                }
                WindowEvent::Focused(is_focused) => {
                    if !is_focused {
                        input_manager.reset_inputs();
                    }
                }
                WindowEvent::RedrawRequested => {
                    engine_state.render();
                }
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            ..
                        },
                    ..
                } => event_loop.exit(),
                _ => (),
            }
        } else {
            match event {
                WindowEvent::Resized(size) => {
                    self.web_window_size = Some(size);
                    self.resized(size);
                }
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            ..
                        },
                    ..
                } => event_loop.exit(),
                _ => (),
            }
        }
    }

    /// Handles device-level input events such as mouse motion.
    /// 
    /// This method processes raw device input and updates the application state accordingly.
    /// 
    /// # Arguments
    /// * `_event_loop` - Reference to the active event loop
    /// * `_device_id` - ID of the device that generated the event
    /// * `event` - The device event to process
    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let Some(state) = &mut self.state {
            if let DeviceEvent::MouseMotion { delta } = event {
                // Just update the input state without immediate processing
                state.input_manager.intake_mouse_motion(delta);
            }
        }
    }

    /// Called when the application is resumed after being suspended.
    /// 
    /// This method triggers the graphics initialization process if the application
    /// is in the uninitialized state with a graphics builder.
    /// 
    /// # Arguments
    /// * `event_loop` - Reference to the active event loop
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let MaybeGraphics::Builder(builder) = &mut self.graphics {
            builder.build_and_send(event_loop);
        }
    }

    /// Handles custom user events, specifically graphics initialization events.
    /// 
    /// This method processes the graphics initialization result and transitions the application
    /// to the running state if initialization is complete.
    /// 
    /// # Arguments
    /// * `_event_loop` - Reference to the active event loop
    /// * `graphics` - The initialized graphics resources
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, graphics: Graphics) {
        let is_surface_configured = graphics.is_surface_configured;

        self.graphics = MaybeGraphics::Graphics(graphics);

        if is_surface_configured {
            self.initialize_application_state();
        } else if let Some(size) = self.web_window_size {
            self.resized(size);
        }
    }

    /// Called before the event loop goes to sleep.
    /// 
    /// This method handles frame timing, input processing, and triggers rendering
    /// of the next frame.
    /// 
    /// # Arguments
    /// * `_event_loop` - Reference to the active event loop
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = &mut self.state {
            let now = web_time::Instant::now();
            let wait_dt = now - state.last_wait_time;

            if let Some(processed_input) = state.input_manager.get_and_reset_processed_input() {
                state.engine_state.set_input_commands(processed_input);
            }

            // Process input is now handled in RedrawRequested
            state.engine_state.process_input(wait_dt);
            
            state.last_wait_time = now;

            state.engine_state.process_tasks();
            state.window.request_redraw();
        }
    }
}
