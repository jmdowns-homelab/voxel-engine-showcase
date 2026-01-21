//! Mesh data structures and operations for voxel rendering.
//!
//! This module provides the core data structures and algorithms for working with
//! meshes in the voxel engine. It handles the conversion from high-level voxel
//! data to GPU-friendly vertex and index buffers.

use crate::engine_state::voxels::{
    block::{block_side::BlockSide, Block, BlockTypeSize},
    chunk::Chunk,
};

use super::{face::Face, greedy};
use crate::engine_state::rendering::Vertex;

/// Represents a single side of a mesh with its associated vertices and indices.
///
/// Each `MeshSide` corresponds to one of the six possible block faces and contains
/// the vertex and index data needed to render that face.
#[derive(Debug)]
pub struct MeshSide {
    /// The vertex data for this mesh side
    pub vertices: Vec<Vertex>,
    /// The index data for this mesh side
    pub indices: Vec<u32>,
    /// The number of vertices in this mesh side
    pub len: u32,
    /// Which block side this mesh represents
    pub side: BlockSide,
}

impl MeshSide {
    /// Creates a new, empty `MeshSide` for the specified block side.
    ///
    /// # Arguments
    /// * `side` - The block side this mesh side represents
    ///
    /// # Returns
    /// A new `MeshSide` with empty vertex and index buffers.
    pub fn new(side: BlockSide) -> Self {
        MeshSide {
            vertices: Vec::new(),
            indices: Vec::new(),
            len: 0,
            side,
        }
    }
}

/// Represents a complete mesh for a voxel chunk with all six possible sides.
///
/// The mesh contains separate vertex and index buffers for each side of the blocks,
/// allowing for efficient rendering with face culling and material-specific rendering.
#[derive(Debug)]
pub struct Mesh {
    /// Array of mesh sides, indexed by `BlockSide` enum values.
    /// The order matches the `BlockSide` enum variant order.
    pub mesh: [MeshSide; 6],
}

impl Mesh {
    /// Creates a new, empty mesh with all sides initialized.
    ///
    /// # Returns
    /// A new `Mesh` with empty vertex and index buffers for all six sides.
    pub fn new() -> Self {
        Mesh {
            mesh: BlockSide::all()
                .into_iter()
                .map(MeshSide::new)
                .collect::<Vec<MeshSide>>()
                .try_into()
                .unwrap(),
        }
    }

    // pub fn greedy(chunk: &Chunk, index: u32) -> Self {
    //     greedy::greedy(chunk, index)
    // }

    #[allow(dead_code)]
    /// Generates a mesh for the specified chunk using greedy meshing for the given sides.
    ///
    /// # Arguments
    /// * `chunk` - The chunk to generate the mesh for
    /// * `index` - The index of this chunk in the world
    /// * `sides` - A list of block sides to generate mesh data for
    ///
    /// # Returns
    /// A new `Mesh` containing the generated geometry for the specified sides.
    pub fn greedy_sided(chunk: &Chunk, index: u32, sides: &Vec<BlockSide>) -> Self {
        greedy::greedy_sided(chunk, index, sides)
    }

    /// Adds vertices and indices to the mesh for each side.
    ///
    /// # Arguments
    /// * `block_vertices` - An array of vertex data for each block side
    /// * `block_indices` - An array of index data for each block side
    ///
    /// # Note
    /// The input arrays must have exactly 6 elements, one for each `BlockSide`.
    /// The indices will be adjusted to account for the existing vertices in the mesh.
    pub fn add_vertices(
        &mut self,
        mut block_vertices: [Vec<Vertex>; 6],
        block_indices: [Vec<u32>; 6],
    ) {
        for i in 0..6 {
            let current_vertices_len = self.mesh[i].vertices.len() as u32;
            self.mesh[i].vertices.append(&mut block_vertices[i]);
            let mut new_indices = block_indices[i]
                .iter()
                .map(|e| (e + current_vertices_len))
                .collect();
            self.mesh[i].indices.append(&mut new_indices);
            self.mesh[i].len += self.mesh[i].indices.len() as u32;
        }
    }

    /// Generates vertex data for a single face of a block.
    ///
    /// # Arguments
    /// * `face` - The face to generate vertices for
    /// * `chunk_coordinate_index` - The index of the chunk this face belongs to
    ///
    /// # Returns
    /// A vector of `Vertex` objects representing the four corners of the face.
    /// The vertices are ordered in a way that forms two triangles when combined
    /// with the indices from `generate_face_indices`.
    pub fn generate_face_vertices(face: &Face, chunk_coordinate_index: u32) -> Vec<Vertex> {
        let texture_indices =
            &Block::get_texture_indices_from_int(face.block_type_int as BlockTypeSize);
        let (texture_index, u_offset, v_offset) = match face.block_side {
            BlockSide::FRONT => (
                0,
                (face.lr.z - face.ll.z) as u8,
                (face.ul.y - face.ll.y) as u8,
            ),
            BlockSide::BACK => (
                1,
                (face.ll.z - face.lr.z) as u8,
                (face.ul.y - face.ll.y) as u8,
            ),
            BlockSide::LEFT => (
                2,
                (face.ll.x - face.lr.x) as u8,
                (face.ul.y - face.ll.y) as u8,
            ),
            BlockSide::RIGHT => (
                3,
                (face.lr.x - face.ll.x) as u8,
                (face.ul.y - face.ll.y) as u8,
            ),
            BlockSide::TOP => (
                4,
                (face.lr.z - face.ll.z) as u8,
                (face.ul.x - face.ll.x) as u8,
            ),
            BlockSide::BOTTOM => (
                5,
                (face.ll.z - face.lr.z) as u8,
                (face.ul.x - face.ll.x) as u8,
            ),
        };

        [
            Vertex::new(
                face.ll.cast::<i32>().unwrap(),
                texture_indices[texture_index],
                0,
                v_offset,
                chunk_coordinate_index,
            ),
            Vertex::new(
                face.lr.cast::<i32>().unwrap(),
                texture_indices[texture_index],
                u_offset,
                v_offset,
                chunk_coordinate_index,
            ),
            Vertex::new(
                face.ul.cast::<i32>().unwrap(),
                texture_indices[texture_index],
                0,
                0,
                chunk_coordinate_index,
            ),
            Vertex::new(
                face.ur.cast::<i32>().unwrap(),
                texture_indices[texture_index],
                u_offset,
                0,
                chunk_coordinate_index,
            ),
        ]
        .to_vec()
    }

    /// Generates index data for a face, adjusted by the number of previously generated faces.
    ///
    /// # Arguments
    /// * `num_faces_generated` - The number of faces that have been generated so far
    ///
    /// # Returns
    /// A vector of indices that form two triangles (6 indices total) for the face.
    /// The indices are adjusted to point to the correct vertices in the vertex buffer.
    pub fn generate_face_indices(num_faces_generated: u32) -> Vec<u32> {
        [
            (num_faces_generated * 4),
            1 + num_faces_generated * 4,
            3 + num_faces_generated * 4,
            (num_faces_generated * 4),
            3 + num_faces_generated * 4,
            2 + num_faces_generated * 4,
        ]
        .to_vec()
    }

    /// Gets the number of vertices for each side of the mesh.
    ///
    /// # Returns
    /// An array containing the vertex count for each `BlockSide` in the order defined by the `BlockSide` enum.
    pub fn get_vertex_lens(&self) -> [u64; 6] {
        [
            self.mesh[0].vertices.len() as u64,
            self.mesh[1].vertices.len() as u64,
            self.mesh[2].vertices.len() as u64,
            self.mesh[3].vertices.len() as u64,
            self.mesh[4].vertices.len() as u64,
            self.mesh[5].vertices.len() as u64,
        ]
    }
}
