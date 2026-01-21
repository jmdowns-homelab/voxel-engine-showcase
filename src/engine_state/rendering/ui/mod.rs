//! UI rendering system for the voxel engine.
//!
//! This module contains components for rendering 2D user interface elements
//! on top of the 3D voxel world. It provides simple primitives like rectangles
//! that can be positioned on screen.

mod renderer;
mod primitives;
mod manager;

pub use renderer::UiRenderer;
pub use primitives::{UiVertex, UiElement, UiRectangle};
pub use manager::UiMeshManager;
