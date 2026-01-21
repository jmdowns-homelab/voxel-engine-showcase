//! UI manager for handling UI elements and their shared buffers.
//!
//! This module provides a manager for UI elements that handles the creation and
//! management of shared vertex and index buffers for efficient rendering.

use std::collections::{HashMap, BTreeSet};
use std::mem::size_of;
use crate::core::StSystem;

use crate::engine_state::buffer_state::BufferState;
use crate::engine_state::rendering::ui::manager::buffer_names::{UI_INDEX_BUFFER, UI_VERTEX_BUFFER};
use super::primitives::{UiElement, UiRectangle, UiVertex, UiElementProperties};

/// Buffer names used by the UI system
pub mod buffer_names {
    /// Name of the UI vertex buffer in the buffer state
    pub const UI_VERTEX_BUFFER: &str = "ui_vertex_buffer";
    /// Name of the UI index buffer in the buffer state
    pub const UI_INDEX_BUFFER: &str = "ui_index_buffer";
}

/// Maximum number of vertices the UI system can handle
const MAX_VERTICES: u32 = 8;
/// Maximum number of indices the UI system can handle
const MAX_INDICES: u32 = 12;

/// Manages UI elements and their shared buffers.
pub struct UiMeshManager {
    /// Named UI elements managed by this manager
    elements: HashMap<String, Box<dyn UiElement>>,
    /// Reference to the buffer state
    buffer_state: StSystem<BufferState>,
    /// Flag indicating if buffers have been created
    buffers_created: bool,
    /// Total number of vertices across all elements
    total_vertices: u32,
    /// Total number of indices across all elements
    total_indices: u32,
    /// Available vertex offsets for reuse (from removed elements)
    available_offsets: BTreeSet<(u32, u32)>, // (offset, size)
}

impl UiMeshManager {
    /// Creates a new UI manager.
    ///
    /// # Arguments
    /// * `buffer_state` - Reference to the buffer state for managing GPU buffers
    ///
    /// # Returns
    /// A new UI manager instance
    pub fn new(buffer_state: StSystem<BufferState>) -> Self {
        buffer_state.get_mut().create_buffer(
            UI_VERTEX_BUFFER,
            wgpu::BufferDescriptor {
                label: Some(UI_VERTEX_BUFFER),
                size: size_of::<UiVertex>() as u64 * MAX_VERTICES as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );

        buffer_state.get_mut().create_buffer(
            UI_INDEX_BUFFER,
            wgpu::BufferDescriptor {
                label: Some(UI_INDEX_BUFFER),
                size: size_of::<u32>() as u64 * MAX_INDICES as u64,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );

        Self {
            elements: HashMap::new(),
            buffer_state,
            buffers_created: true, // Buffers are created in the constructor
            total_vertices: 0,
            total_indices: 0,
            available_offsets: BTreeSet::new(),
        }
    }
    
    /// Finds a suitable vertex offset for the given number of vertices.
    ///
    /// This method tries to find an available slot in previously freed space,
    /// or returns the current end of the buffer if no suitable space is found.
    ///
    /// # Arguments
    /// * `vertex_count` - Number of vertices needed
    ///
    /// # Returns
    /// The vertex offset to use
    fn find_vertex_offset(&mut self, vertex_count: u32) -> u32 {
        // Try to find a suitable slot in the available offsets
        if let Some(&(offset, size)) = self.available_offsets
            .iter()
            .find(|&&(_, size)| size >= vertex_count) {
            
            // Remove this slot from available offsets
            self.available_offsets.remove(&(offset, size));
            
            // If there's leftover space, add it back to available offsets
            if size > vertex_count {
                self.available_offsets.insert((offset + vertex_count, size - vertex_count));
            }
            
            return offset;
        }
        
        // No suitable slot found, use the end of the buffer
        self.total_vertices
    }
    
    /// Adds a UI element to the manager with the given name.
    ///
    /// # Arguments
    /// * `name` - Unique identifier for the UI element
    /// * `element` - The UI element to add
    ///
    /// # Returns
    /// `true` if the element was added successfully, `false` if an element with the same name already exists
    pub fn add_element(
        &mut self, 
        name: &str, 
        mut element: Box<dyn UiElement>
    ) -> bool {
        if self.elements.contains_key(name) {
            return false;
        }
        
        // Get vertex and index counts
        let vertex_count = element.vertex_count();
        let index_count = element.index_count();
        
        // Find a suitable vertex offset
        let vertex_offset = self.find_vertex_offset(vertex_count);
        
        // Set the vertex offset for the element
        element.set_vertex_offset(vertex_offset);
        
        // Get vertices and indices
        let vertices = element.get_vertices();
        let indices = element.get_indices(vertex_offset);
        
        // Write vertices to the buffer
        let vertex_byte_offset = (vertex_offset as usize) * size_of::<UiVertex>();
        self.buffer_state.get_mut().write_buffer(
            UI_VERTEX_BUFFER,
            vertex_byte_offset as u64,
            bytemuck::cast_slice(&vertices)
        );

        let index_offset = (vertex_offset * 3 / 2);
        
        // Write indices to the buffer
        let index_byte_offset = (index_offset as usize) * size_of::<u32>();
        self.buffer_state.get_mut().write_buffer(
            UI_INDEX_BUFFER,
            index_byte_offset as u64,
            bytemuck::cast_slice(&indices)
        );
        
        // Update totals
        if vertex_offset + vertex_count > self.total_vertices {
            self.total_vertices = vertex_offset + vertex_count;
        }
        if index_offset + index_count > self.total_indices {
            self.total_indices = index_offset + index_count;
        }
        
        // Add the element to the collection
        self.elements.insert(name.to_string(), element);
        
        true
    }
    
    /// Adds a centered rectangle to the UI.
    ///
    /// # Arguments
    /// * `name` - Unique identifier for the UI element
    /// * `relative_size` - Size as a fraction of the screen (0.0 to 1.0)
    /// * `color` - Color of the rectangle
    ///
    /// # Returns
    /// `true` if the element was added successfully, `false` if an element with the same name already exists
    pub fn add_centered_rectangle(
        &mut self,
        name: &str,
        relative_size: (f32, f32),
        color: wgpu::Color
    ) -> bool {
        let rectangle = UiRectangle::centered(relative_size, color);
        self.add_element(name, Box::new(rectangle))
    }
    
    /// Adds a rectangle to the UI at a specific position.
    ///
    /// # Arguments
    /// * `name` - Unique identifier for the UI element
    /// * `position_lower_left` - Position of the lower left corner as a fraction of the screen (0.0 to 1.0)
    /// * `relative_size` - Size as a fraction of the screen (0.0 to 1.0)
    /// * `color` - Color of the rectangle
    ///
    /// # Returns
    /// `true` if the element was added successfully, `false` if an element with the same name already exists
    pub fn add_rectangle(
        &mut self,
        name: &str,
        position_lower_left: (f32, f32),
        relative_size: (f32, f32),
        color: wgpu::Color
    ) -> bool {
        let rectangle = UiRectangle::new(position_lower_left, relative_size, color);
        self.add_element(name, Box::new(rectangle))
    }
    
    /// Updates the color of an existing rectangle UI element.
    ///
    /// # Arguments
    /// * `name` - Name of the rectangle element to update
    /// * `color` - New color for the rectangle
    ///
    /// # Returns
    /// `true` if the element was found and updated, `false` otherwise
    pub fn update_rectangle_color(
        &mut self,
        name: &str,
        color: wgpu::Color
    ) -> bool {
        // Create properties with just the color update
        let properties = super::primitives::UiElementProperties {
            position: None,
            size: None,
            color: Some(color),
        };
        
        // Update the element
        self.update_element(name, properties)
    }
    
    /// Updates an existing UI element with new properties.
    ///
    /// This method efficiently updates only the affected element without rebuilding all buffers.
    ///
    /// # Arguments
    /// * `name` - Name of the element to update
    /// * `properties` - New properties to apply to the element
    ///
    /// # Returns
    /// `true` if the element was found and updated, `false` otherwise
    pub fn update_element(
        &mut self,
        name: &str,
        properties: UiElementProperties,
    ) -> bool {
        // Find the element
        if let Some(element) = self.elements.get_mut(name) {

            element.update_properties(&properties);

            // Get the updated vertices
            let vertices = element.get_vertices();
            
            // Write the updated vertices to the buffer
            let vertex_offset = element.get_vertex_offset();
            let vertex_byte_offset = (vertex_offset as usize) * size_of::<UiVertex>();
            self.buffer_state.get_mut().write_buffer(
                UI_VERTEX_BUFFER,
                vertex_byte_offset as u64,
                bytemuck::cast_slice(&vertices)
            );
            
            true
        } else {
            false
        }
    }
    
    /// Removes a UI element by name.
    ///
    /// # Arguments
    /// * `name` - Name of the element to remove
    ///
    /// # Returns
    /// `true` if the element was found and removed, `false` otherwise
    pub fn remove_element(&mut self, name: &str) -> bool {
        if let Some(element) = self.elements.remove(name) {
            // Get the vertex offset and count
            let vertex_offset = element.get_vertex_offset();
            let vertex_count = element.vertex_count();
            
            // Add the freed space to available offsets
            self.available_offsets.insert((vertex_offset, vertex_count));
            
            // Note: We don't update the index buffer here because it would require
            // shifting all subsequent indices. Instead, we just keep track of the
            // total number of indices and reuse the space when adding new elements.
            
            true
        } else {
            false
        }
    }
    
    /// Gets a slice of the vertex buffer for rendering.
    
    /// Gets the total number of indices for rendering.
    ///
    /// # Returns
    /// The total number of indices in the buffer
    pub fn get_index_count(&self) -> u32 {
        self.total_indices
    }
    
    /// Checks if there are any UI elements to render.
    ///
    /// # Returns
    /// `true` if there are no elements, `false` otherwise
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}
