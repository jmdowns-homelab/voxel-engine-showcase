//! # Input Manager
//!
//! This module handles input processing for the application, including:
//! - Keyboard input state tracking
//! - Mouse input state tracking
//! - Input event processing
//! - Input state management

use std::collections::HashMap;

use winit::{
    event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use super::input_state::{MouseInput, ProcessedInputState, RawInputState};

const KEY_CODES: [KeyCode; 9] = [
    KeyCode::KeyW,
    KeyCode::KeyS,
    KeyCode::KeyA,
    KeyCode::KeyD,
    KeyCode::KeyR,
    KeyCode::KeyI,
    KeyCode::KeyL,
    KeyCode::Space,
    KeyCode::ShiftLeft,
];

/// Manages the state of all input devices and processes input events.
///
/// This struct maintains the current state of keyboard and mouse inputs
/// and provides methods to process input events from the windowing system.
pub struct InputManager {
    /// Previous state of all tracked keyboard keys
    pub keyboard_inputs_old: HashMap<KeyCode, bool>,
    /// Current state of all tracked keyboard keys
    pub keyboard_inputs_new: HashMap<KeyCode, bool>,
    
    /// Current state of mouse inputs
    pub mouse_inputs: MouseInput,
}

impl InputManager {
    /// Creates a new InputManager with default state.
    /// 
    /// Initializes all tracked keyboard keys to 'released' state and sets up
    /// empty mouse input state.
    /// 
    /// # Returns
    /// A new `InputManager` instance with default state.
    pub fn new() -> Self {
        let mut keyboard_inputs_old = HashMap::new();
        let mut keyboard_inputs_new = HashMap::new();
        for key_code in KEY_CODES {
            keyboard_inputs_old.insert(key_code, false);
            keyboard_inputs_new.insert(key_code, false);
        }

        let mouse_buttons = [
            MouseButton::Left,
            MouseButton::Right,
            MouseButton::Middle,
            MouseButton::Forward,
            MouseButton::Back,
        ];

        let mut mouse_button_inputs_old = HashMap::new();
        let mut mouse_button_inputs_new = HashMap::new();
        
        for button in mouse_buttons {
            mouse_button_inputs_old.insert(button, false);
            mouse_button_inputs_new.insert(button, false);
        }

        let mouse_inputs = MouseInput {
            mouse_button_inputs_old,
            mouse_button_inputs_new,
            mouse_scroll_delta: None,
            mouse_delta: None,
        };

        Self {
            keyboard_inputs_old,
            keyboard_inputs_new,
            mouse_inputs,
        }
    }

    /// Updates the old state with the current state to prepare for the next frame.
    /// 
    /// This should be called at the end of each frame to ensure that the "old" state
    /// is properly updated for the next frame's comparisons.
    pub fn move_old_states(&mut self) {
        // Update keyboard states
        for (key, new_state) in self.keyboard_inputs_new.iter() {
            if let Some(old_state) = self.keyboard_inputs_old.get_mut(key) {
                *old_state = *new_state;
            }
        }
        
        // Update mouse button states
        for (button, new_state) in self.mouse_inputs.mouse_button_inputs_new.iter() {
            if let Some(old_state) = self.mouse_inputs.mouse_button_inputs_old.get_mut(button) {
                *old_state = *new_state;
            }
        }
    }

    /// Processes a window event and updates internal input state.
    /// 
    /// Handles keyboard and mouse button events, updating the internal state.
    /// 
    /// # Arguments
    /// * `event` - The window event to process
    pub fn intake_input(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(key),
                        ..
                    },
                ..
            } => {
                if let Some(key_state) = self.keyboard_inputs_new.get_mut(key) {
                    *key_state = *state == ElementState::Pressed;
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.mouse_inputs.mouse_scroll_delta = Some(*delta);
            }
            WindowEvent::MouseInput { button, state, .. } => {
                if let Some(button_state) = self.mouse_inputs.mouse_button_inputs_new.get_mut(button) {
                    *button_state = *state == ElementState::Pressed;
                }
            }
            _ => {}
        }
    }

    /// Updates the mouse movement delta.
    /// 
    /// # Arguments
    /// * `delta` - The (x, y) delta of mouse movement since the last update
    pub fn intake_mouse_motion(&mut self, delta: (f64, f64)) {
        self.mouse_inputs.mouse_delta = Some(delta);
    }

    /// Creates a processed input state from the current raw boolean states.
    ///
    /// This translates the raw boolean states into RawInputState enum values
    /// that represent the state transitions (pressed, held, released, not pressed).
    ///
    /// # Returns
    /// A new `ProcessedInputState` with processed input states.
    pub fn create_processed_input_state(&mut self) -> ProcessedInputState {
        let mut keyboard_states = HashMap::new();
        let mut mouse_button_states = HashMap::new();
        
        // Process keyboard states
        for (key, &new_state) in self.keyboard_inputs_new.iter() {
            let old_state = self.keyboard_inputs_old.get(key).copied().unwrap_or(false);
            keyboard_states.insert(*key, RawInputState::from_raw_states(old_state, new_state));
        }
        
        // Process mouse button states
        for (button, &new_state) in self.mouse_inputs.mouse_button_inputs_new.iter() {
            let old_state = self.mouse_inputs.mouse_button_inputs_old.get(button).copied().unwrap_or(false);
            mouse_button_states.insert(*button, RawInputState::from_raw_states(old_state, new_state));
        }

        let mouse_delta = self.mouse_inputs.mouse_delta;
        
        ProcessedInputState {
            keyboard_states,
            mouse_button_states,
            mouse_delta,
        }
    }
    
    /// Returns the processed input state and resets internal state.
    /// 
    /// This method should be called to get the
    /// processed input state and reset the internal state for the next frame.
    /// 
    /// # Returns
    /// The processed input state, if available.
    pub fn get_and_reset_processed_input(&mut self) -> Option<ProcessedInputState> {
        let processed_input = Some(self.create_processed_input_state());
        self.reset_inputs();
        processed_input
    }

    /// Resets all input states to their default values.
    /// 
    /// This is typically called when the window loses focus to prevent
    /// stuck keys or buttons.
    pub fn reset_inputs(&mut self) {
        // Reset keyboard states
        self.move_old_states();
        
        // Reset other mouse state
        self.mouse_inputs.mouse_scroll_delta = None;
        self.mouse_inputs.mouse_delta = None;
    }
}
