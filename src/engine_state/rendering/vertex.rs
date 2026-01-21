//! Vertex data structures and layouts for voxel rendering.
//!
//! This module defines the vertex format used for rendering voxels and provides utilities
//! for working with vertex data in the rendering pipeline.

use cgmath::Point3;

/// A vertex in the voxel rendering pipeline.
///
/// Represents a single point in 3D space with associated texture and chunk information.
/// The vertex is designed to be efficiently processed by the GPU and matches the
/// vertex shader's expected input layout.
///
/// # Memory Layout
/// - Position: 3x i32 (12 bytes)
/// - Texture Index: u32 (4 bytes)
/// - Texture Coordinates: [f32; 2] (8 bytes)
/// - Chunk Coordinate Index: u32 (4 bytes)
///
/// Total size: 28 bytes
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    /// X coordinate in world space
    x: i32,
    /// Y coordinate in world space
    y: i32,
    /// Z coordinate in world space
    z: i32,
    /// Index of the texture in the texture array
    texture_index: u32,
    /// UV texture coordinates (normalized 0.0-1.0)
    tex_coords: [f32; 2],
    /// Index into the chunk coordinate buffer for this vertex's chunk
    chunk_coordinate_index: u32,
}
impl Vertex {
    /// Creates a new vertex with the given parameters.
    ///
    /// # Arguments
    /// * `pos` - The 3D position of the vertex in world space
    /// * `texture_index` - Index of the texture in the texture array
    /// * `u` - U texture coordinate (0-255)
    /// * `v` - V texture coordinate (0-255)
    /// * `chunk_coordinate_index` - Index into the chunk coordinate buffer
    ///
    /// # Returns
    /// A new `Vertex` instance
    pub fn new(
        pos: Point3<i32>,
        texture_index: usize,
        u: u8,
        v: u8,
        chunk_coordinate_index: u32,
    ) -> Self {
        Vertex {
            x: pos.x,
            y: pos.y,
            z: pos.z,
            texture_index: texture_index as u32,
            tex_coords: [u as f32, v as f32],
            chunk_coordinate_index,
        }
    }

    /// Returns the vertex buffer layout description for the shader pipeline.
    ///
    /// This defines how the vertex data is laid out in memory and how it maps
    /// to the vertex shader's input attributes.
    ///
    /// # Returns
    /// A `wgpu::VertexBufferLayout` describing the vertex format
    ///
    /// # Shader Attributes
    /// - `location = 0`: position (i32, i32, i32)
    /// - `location = 3`: texture_index (u32)
    /// - `location = 4`: tex_coords (vec2<f32>)
    /// - `location = 5`: chunk_coordinate_index (u32)
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Sint32,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[u32; 1]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Sint32,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[u32; 2]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Sint32,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[u32; 3]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Uint32,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[u32; 4]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[u32; 6]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}
