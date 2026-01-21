//! # Input State
//!
//! This module defines the input state types used by the input manager.
//! It provides enums and structs for representing the state of input devices.

use std::collections::HashMap;
use winit::{
    event::{MouseButton, MouseScrollDelta},
    keyboard::KeyCode,
};

/// Represents the state of a key or button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawInputState {
    /// Key/button is not pressed
    NotPressed,
    /// Key/button was just pressed this frame
    Pressed,
    /// Key/button has been held down for multiple frames
    Held,
    /// Key/button was just released this frame
    Released,
}

impl Default for RawInputState {
    fn default() -> Self {
        Self::NotPressed
    }
}

impl RawInputState {
    /// Determines if the input is actively down (either pressed or held)
    pub fn is_active(&self) -> bool {
        matches!(self, RawInputState::Pressed | RawInputState::Held)
    }
    
    /// Determines if the input was just pressed this frame
    pub fn is_just_pressed(&self) -> bool {
        matches!(self, RawInputState::Pressed)
    }
    
    /// Determines if the input was just released this frame
    pub fn is_just_released(&self) -> bool {
        matches!(self, RawInputState::Released)
    }
    
    /// Updates the input state based on the previous and current raw states
    pub fn from_raw_states(previous: bool, current: bool) -> Self {
        match (previous, current) {
            (false, true) => RawInputState::Pressed,   // Wasn't pressed, now is = PRESSED
            (true, true) => RawInputState::Held,       // Was pressed, still is = HELD
            (true, false) => RawInputState::Released,  // Was pressed, now isn't = RELEASED
            (false, false) => RawInputState::NotPressed, // Wasn't pressed, still isn't = NOT PRESSED
        }
    }
}

/// A snapshot of the processed input states with state transitions.
///
/// This struct provides access to the processed state of all input devices,
/// with key and button states translated into RawInputState enum values.
pub struct ProcessedInputState {
    /// Current state of all tracked keyboard keys
    pub keyboard_states: HashMap<KeyCode, RawInputState>,
    
    /// Current state of mouse buttons
    pub mouse_button_states: HashMap<MouseButton, RawInputState>,
    
    /// Mouse movement delta since the last frame (x, y)
    pub mouse_delta: Option<(f64, f64)>,
}

impl ProcessedInputState {
    /// Gets the state of a keyboard key
    pub fn get_key_state(&self, key: KeyCode) -> RawInputState {
        self.keyboard_states.get(&key).copied().unwrap_or_default()
    }
    
    /// Gets the state of a mouse button
    pub fn get_mouse_button_state(&self, button: MouseButton) -> RawInputState {
        self.mouse_button_states.get(&button).copied().unwrap_or_default()
    }
    
    /// Gets the mouse movement delta since the last frame
    pub fn get_mouse_delta(&self) -> Option<(f64, f64)> {
        self.mouse_delta
    }
}

/// Tracks the state of mouse inputs including buttons, scroll, and movement.
pub struct MouseInput {
    /// Previous state of each mouse button (pressed/released)
    pub mouse_button_inputs_old: HashMap<MouseButton, bool>,
    /// Current state of each mouse button (pressed/released)
    pub mouse_button_inputs_new: HashMap<MouseButton, bool>,
    
    /// Accumulated scroll delta since the last frame
    pub mouse_scroll_delta: Option<MouseScrollDelta>,
    
    /// Mouse movement delta since the last frame (x, y)
    pub mouse_delta: Option<(f64, f64)>,
}
