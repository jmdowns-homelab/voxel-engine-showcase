//! Rectangle UI primitive element.
//!
//! This module defines a simple rectangle UI element that can be positioned and sized on screen.

use wgpu::Color;

use super::{UiElement, UiVertex, UiElementProperties};

/// A simple rectangle UI element.
///
/// Represents a colored rectangle that can be positioned and sized on the screen.
/// The rectangle is defined by its position (in normalized device coordinates from -1 to 1)
/// and its size (also in normalized device coordinates).
pub struct UiRectangle {
    /// Position of the rectangle center in normalized device coordinates (-1 to 1)
    pub position: (f32, f32),
    /// Size of the rectangle in normalized device coordinates (0 to 2)
    pub size: (f32, f32),
    /// Color of the rectangle
    pub color: Color,
    /// Vertex offset in the shared buffer
    vertex_offset: u32,
}

impl UiRectangle {
    /// Creates a new rectangle with the given parameters.
    ///
    /// # Arguments
    /// * `position` - Center position of the rectangle in normalized device coordinates (-1 to 1)
    /// * `size` - Size of the rectangle in normalized device coordinates (0 to 2)
    /// * `color` - Color of the rectangle
    ///
    /// # Returns
    /// A new `UiRectangle` instance
    pub fn new(position: (f32, f32), size: (f32, f32), color: Color) -> Self {
        Self {
            position,
            size,
            color,
            vertex_offset: 0,
        }
    }
    
    /// Creates a rectangle centered on screen with the specified relative size.
    ///
    /// # Arguments
    /// * `relative_size` - Size as a fraction of the screen (0.0 to 1.0)
    /// * `color` - Color of the rectangle
    ///
    /// # Returns
    /// A new `UiRectangle` instance centered on screen
    pub fn centered(relative_size: (f32, f32), color: Color) -> Self {
        // Convert from relative size (0-1) to NDC size (-1 to 1)
        let size = (relative_size.0 * 2.0, relative_size.1 * 2.0);
        Self::new((0.0, 0.0), size, color)
    }
}

impl UiElement for UiRectangle {
    fn get_vertices(&self) -> Vec<UiVertex> {
        // Calculate the corners of the rectangle in NDC space
        let half_width = self.size.0 / 2.0;
        let half_height = self.size.1 / 2.0;
        
        let left = self.position.0 - half_width;
        let right = self.position.0 + half_width;
        let top = self.position.1 - half_height;
        let bottom = self.position.1 + half_height;
        
        let color = [
            self.color.r as f32,
            self.color.g as f32,
            self.color.b as f32,
            self.color.a as f32,
        ];
        
        // Define the vertices with position and color
        vec![
            UiVertex { position: [left, top, 0.0], color },     // Top-left
            UiVertex { position: [right, top, 0.0], color },    // Top-right
            UiVertex { position: [right, bottom, 0.0], color }, // Bottom-right
            UiVertex { position: [left, bottom, 0.0], color },  // Bottom-left
        ]
    }
    
    fn get_indices(&self, base_vertex: u32) -> Vec<u32> {
        // Define indices for two triangles forming the rectangle
        // Offset by base_vertex to account for position in the shared buffer
        vec![
            base_vertex + 0, base_vertex + 1, base_vertex + 2, // First triangle
            base_vertex + 0, base_vertex + 2, base_vertex + 3, // Second triangle
        ]
    }
    
    fn index_count(&self) -> u32 {
        6 // Two triangles = 6 indices
    }
    
    fn vertex_count(&self) -> u32 {
        4 // Four vertices for a rectangle
    }
    
    fn get_vertex_offset(&self) -> u32 {
        self.vertex_offset
    }
    
    fn set_vertex_offset(&mut self, offset: u32) {
        self.vertex_offset = offset;
    }
    
    fn update_properties(&mut self, properties: &UiElementProperties) -> bool {
        let mut updated = false;
        
        if let Some(position) = properties.position {
            self.position = position;
            updated = true;
        }
        
        if let Some(size) = properties.size {
            self.size = size;
            updated = true;
        }
        
        if let Some(color) = properties.color {
            self.color = color;
            updated = true;
        }
        
        updated
    }
}
