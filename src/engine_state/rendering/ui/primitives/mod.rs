//! UI primitive elements for the voxel engine.
//!
//! This module defines the basic building blocks for UI elements like vertices
//! and common traits that all UI elements must implement.

mod rectangle;

use wgpu::{Device, Queue, RenderPass, Color};

pub use rectangle::UiRectangle;

/// Properties for updating UI elements.
///
/// This struct provides a flexible way to update UI element properties
/// without having to specify all properties every time.
#[derive(Debug, Clone)]
pub struct UiElementProperties {
    /// Position of the element (optional)
    pub position: Option<(f32, f32)>,
    /// Size of the element (optional)
    pub size: Option<(f32, f32)>,
    /// Color of the element (optional)
    pub color: Option<Color>,
}

impl UiElementProperties {
    /// Creates a new empty properties object.
    pub fn new() -> Self {
        Self {
            position: None,
            size: None,
            color: None,
        }
    }
    
    /// Sets the position property.
    pub fn with_position(mut self, position: (f32, f32)) -> Self {
        self.position = Some(position);
        self
    }
    
    /// Sets the size property.
    pub fn with_size(mut self, size: (f32, f32)) -> Self {
        self.size = Some(size);
        self
    }
    
    /// Sets the color property.
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

impl Default for UiElementProperties {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a vertex in a UI element.
///
/// UI vertices are used to define the geometry of user interface elements.
/// Each vertex contains position information in normalized device coordinates (NDC)
/// and color information for rendering. The UiVertex struct is designed to be
/// efficiently transferred to the GPU using the `bytemuck` crate for zero-copy
/// conversion to raw bytes.
///
/// # Memory Layout
///
/// The struct is marked with `#[repr(C)]` to ensure a consistent memory layout
/// across different platforms, which is essential for correct GPU buffer operations.
/// The total size is 28 bytes:
/// - `position`: 12 bytes (3 × f32)
/// - `color`: 16 bytes (4 × f32)
///
/// # GPU Representation
///
/// In the shader, this corresponds to:
/// ```wgsl
/// struct UiVertex {
///     @location(0) position: vec3<f32>,
///     @location(1) color: vec4<f32>,
/// }
/// ```
///
/// # Examples
///
/// ```rust
/// // Create a white vertex at the top-left corner of the screen
/// let vertex = UiVertex {
///     position: [-1.0, -1.0, 0.0],
///     color: [1.0, 1.0, 1.0, 1.0],
/// };
/// ```
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UiVertex {
    /// Position of the vertex in normalized device coordinates
    /// 
    /// The coordinates are in the range [-1.0, 1.0] where:
    /// - x: -1.0 is the left edge, 1.0 is the right edge
    /// - y: -1.0 is the top edge, 1.0 is the bottom edge
    /// - z: Used for depth ordering (typically 0.0 for UI elements)
    pub position: [f32; 3],
    
    /// Color of the vertex (RGBA)
    /// 
    /// Each component is in the range [0.0, 1.0]:
    /// - [0]: Red component
    /// - [1]: Green component
    /// - [2]: Blue component
    /// - [3]: Alpha component (transparency)
    pub color: [f32; 4],
}

/// Common trait for all UI elements.
pub trait UiElement {
    /// Gets the vertices for this UI element.
    fn get_vertices(&self) -> Vec<UiVertex>;
    
    /// Gets the indices for this UI element.
    ///
    /// # Arguments
    /// * `base_vertex` - The base vertex index to offset indices by
    fn get_indices(&self, base_vertex: u32) -> Vec<u32>;
    
    /// Gets the number of indices for this UI element.
    fn index_count(&self) -> u32;
    
    /// Gets the number of vertices for this UI element.
    fn vertex_count(&self) -> u32;
    
    /// Gets the vertex offset for this UI element in the shared buffer.
    fn get_vertex_offset(&self) -> u32;
    
    /// Sets the vertex offset for this UI element in the shared buffer.
    ///
    /// # Arguments
    /// * `offset` - The new vertex offset
    fn set_vertex_offset(&mut self, offset: u32);
    
    /// Updates the element with the given properties.
    ///
    /// # Arguments
    /// * `properties` - The properties to update
    ///
    /// # Returns
    /// `true` if any properties were updated, `false` otherwise
    fn update_properties(&mut self, properties: &UiElementProperties) -> bool;
}
