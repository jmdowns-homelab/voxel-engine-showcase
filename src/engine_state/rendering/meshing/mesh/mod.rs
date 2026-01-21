//! Mesh generation and manipulation for voxel rendering.
//!
//! This module provides the core functionality for converting voxel data into optimized
//! GPU-friendly mesh representations. It implements greedy meshing algorithms to reduce
//! the number of vertices and faces by combining coplanar faces with the same material.
//!
//! # Architecture
//! - [`Mesh`]: The main structure representing a complete mesh with vertices and indices
//! - [`Face`]: Represents a single face of a voxel with its vertices and properties
//! - Greedy meshing: Algorithm to optimize the mesh by merging adjacent coplanar faces
//!
//! # Usage
//! ```no_run
//! use crate::engine_state::{
//!     rendering::meshing::mesh::{Mesh, Face},
//!     voxels::{chunk::Chunk, block::block_side::BlockSide}
//! };
//! use cgmath::Point3;
//!
//! // Create a new mesh for a chunk
//! let chunk = Chunk::new(Point3::new(0, 0, 0));
//! let mesh = Mesh::greedy_sided(&chunk, 0, &BlockSide::all());
//! ```
//!
//! # Performance Considerations
//! - Uses greedy meshing to minimize the number of vertices and indices
//! - Optimized for chunk-based rendering with batched draw calls
//! - Reduces memory usage by reusing vertex data where possible

mod face;
mod greedy;
mod mesh;

pub use face::Face;
pub use greedy::greedy_sided;
pub use mesh::*;
