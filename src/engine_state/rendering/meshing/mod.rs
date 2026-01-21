//! Mesh generation and management for voxel rendering.
//!
//! This module handles the conversion of voxel data into optimized GPU-friendly meshes
//! using a bucket-based rendering strategy. The key goals are:
//! 1. Minimize draw calls using multi-draw indirect
//! 2. Enable efficient GPU-side culling
//! 3. Reduce memory fragmentation
//! 4. Optimize for cache-friendly access patterns
//!
//! # Architecture
//! - `MeshManager`: Main interface for mesh generation and management
//! - `MeshBucketManager`: Manages memory allocation using a bucket-based approach
//! - `ChunkIndexState`: Tracks chunk positions and their GPU buffer indices
//! - `mesh/`: Contains the core mesh generation algorithms
//!
//! # Bucket-Based Rendering
//! The system uses fixed-size buckets to store mesh data, which enables:
//! - Fewer, larger draw calls via multi-draw indirect
//! - Efficient GPU-side culling of entire buckets
//! - Better memory locality and cache utilization
//!
//! For more details, see [Bucket-Based Rendering Strategy](../../docs/domain_flows/system/bucket_based_rendering.md)
//!
//! # Performance Considerations
//! - Greedy meshing minimizes vertex count
//! - Fixed-size buckets reduce memory fragmentation
//! - Indirect drawing reduces CPU overhead
//! - Bucket organization by block side enables efficient culling

use std::num::NonZeroUsize;

use bucket_manager::MeshBucketManager;
use cgmath::Point3;
use chunk_index_state::ChunkIndexState;
use lru::LruCache;
use wgpu::util::DrawIndexedIndirectArgs;

mod bucket_manager;
mod chunk_index_state;

/// Core mesh generation algorithms and data structures.
///
/// This module contains the implementation of greedy meshing and related
/// functionality for converting voxel data into optimized triangle meshes.
mod mesh;
mod renderer;

// Re-export the mesh module's public interface for external use
pub use mesh::*;

// Re-export the renderer module's public interface for external use
pub use renderer::*;

/// Name of the chunk index buffer used for indirect rendering
pub use chunk_index_state::CHUNK_INDEX_BUFFER_NAME;

use crate::{
    core::{MtResource, StSystem},
    engine_state::{
        buffer_state::{BufferState, BufferWriteCommand},
        voxels::{
            block::block_side::BlockSide,
            chunk::Chunk,
        },
    },
};

// Re-export the mesh module's public interface
pub use mesh::Mesh;

/// Names of the vertex buffers for each block side.
/// These are used to identify the buffers in the renderer.
pub const VERTEX_BUFFER_FRONT: &str = "Vertex Buffer Front";
pub const VERTEX_BUFFER_BACK: &str = "Vertex Buffer Back";
pub const VERTEX_BUFFER_LEFT: &str = "Vertex Buffer Left";
pub const VERTEX_BUFFER_RIGHT: &str = "Vertex Buffer Right";
pub const VERTEX_BUFFER_TOP: &str = "Vertex Buffer Top";
pub const VERTEX_BUFFER_BOTTOM: &str = "Vertex Buffer Bottom";

/// Names of the index buffers for each block side.
/// These are used to identify the buffers in the renderer.
pub const INDEX_BUFFER_FRONT: &str = "Index Buffer Front";
pub const INDEX_BUFFER_BACK: &str = "Index Buffer Back";
pub const INDEX_BUFFER_LEFT: &str = "Index Buffer Left";
pub const INDEX_BUFFER_RIGHT: &str = "Index Buffer Right";
pub const INDEX_BUFFER_TOP: &str = "Index Buffer Top";
pub const INDEX_BUFFER_BOTTOM: &str = "Index Buffer Bottom";

/// Names of the indirect draw buffers for each block side.
/// These buffers contain the draw commands for indirect rendering.
pub const INDIRECT_BUFFER_FRONT: &str = "Indirect Buffer Front";
pub const INDIRECT_BUFFER_BACK: &str = "Indirect Buffer Back";
pub const INDIRECT_BUFFER_LEFT: &str = "Indirect Buffer Left";
pub const INDIRECT_BUFFER_RIGHT: &str = "Indirect Buffer Right";
pub const INDIRECT_BUFFER_TOP: &str = "Indirect Buffer Top";
pub const INDIRECT_BUFFER_BOTTOM: &str = "Indirect Buffer Bottom";

/// Central manager for voxel mesh generation and GPU buffer management.
///
/// The `MeshManager` is responsible for:
/// - Converting voxel chunk data into optimized mesh geometry
/// - Managing GPU memory allocation for mesh data using a bucket-based approach
/// - Tracking chunk positions and their corresponding buffer indices
/// - Generating buffer write commands for mesh updates
/// - Handling chunk loading and unloading
///
/// # Memory Management
///
/// The mesh manager uses a bucket-based approach to memory management, which:
/// - Allocates fixed-size buckets for vertices, indices, and draw commands
/// - Enables efficient multi-draw-indirect rendering
/// - Reduces memory fragmentation
/// - Improves cache locality
///
/// # Performance Considerations
///
/// - Uses LRU caching to track meshed chunks and prioritize updates
/// - Separates meshes by block side to enable efficient culling
/// - Minimizes buffer updates through batched write commands
pub struct MeshManager {
    /// Manages the allocation of mesh data into buckets
    bucket_manager: MeshBucketManager,
    /// Tracks chunk positions and their corresponding buffer indices
    chunk_index_state: ChunkIndexState,
    /// LRU cache to track which chunks have been meshed
    least_recently_meshed_chunks: LruCache<Point3<i32>, ()>,
}

impl MeshManager {
    /// Number of buffer sets per block side
    const NUM_BUFFERS_PER_SIDE: usize = 1;

    /// Gets the vertex buffer name for a specific block side.
    ///
    /// # Arguments
    ///
    /// * `side` - The block side to get the buffer name for
    ///
    /// # Returns
    ///
    /// The static string name of the vertex buffer for the given side
    pub fn get_vertex_buffer_name(side: BlockSide) -> &'static str {
        match side {
            BlockSide::FRONT => VERTEX_BUFFER_FRONT,
            BlockSide::BACK => VERTEX_BUFFER_BACK,
            BlockSide::BOTTOM => VERTEX_BUFFER_BOTTOM,
            BlockSide::TOP => VERTEX_BUFFER_TOP,
            BlockSide::LEFT => VERTEX_BUFFER_LEFT,
            BlockSide::RIGHT => VERTEX_BUFFER_RIGHT,
        }
    }

    /// Gets the index buffer name for a specific block side.
    ///
    /// # Arguments
    ///
    /// * `side` - The block side to get the buffer name for
    ///
    /// # Returns
    ///
    /// The static string name of the index buffer for the given side
    pub fn get_index_buffer_name(side: BlockSide) -> &'static str {
        match side {
            BlockSide::FRONT => INDEX_BUFFER_FRONT,
            BlockSide::BACK => INDEX_BUFFER_BACK,
            BlockSide::BOTTOM => INDEX_BUFFER_BOTTOM,
            BlockSide::TOP => INDEX_BUFFER_TOP,
            BlockSide::LEFT => INDEX_BUFFER_LEFT,
            BlockSide::RIGHT => INDEX_BUFFER_RIGHT,
        }
    }

    /// Gets the indirect buffer name for a specific block side.
    ///
    /// # Arguments
    ///
    /// * `side` - The block side to get the buffer name for
    ///
    /// # Returns
    ///
    /// The static string name of the indirect buffer for the given side
    pub fn get_indirect_buffer_name(side: BlockSide) -> &'static str {
        match side {
            BlockSide::FRONT => INDIRECT_BUFFER_FRONT,
            BlockSide::BACK => INDIRECT_BUFFER_BACK,
            BlockSide::BOTTOM => INDIRECT_BUFFER_BOTTOM,
            BlockSide::TOP => INDIRECT_BUFFER_TOP,
            BlockSide::LEFT => INDIRECT_BUFFER_LEFT,
            BlockSide::RIGHT => INDIRECT_BUFFER_RIGHT,
        }
    }

    /// Creates a new mesh manager and initializes all required GPU buffers.
    ///
    /// # Arguments
    ///
    /// * `buffer_state` - Reference to the buffer state for GPU buffer management
    ///
    /// # Returns
    ///
    /// A new `MeshManager` instance with initialized buffers
    ///
    /// # Implementation Details
    ///
    /// This method:
    /// - Creates vertex, index, and indirect buffers for each block side
    /// - Initializes the indirect buffers with default draw commands
    /// - Sets up the bucket manager and chunk index state
    pub fn new(buffer_state: StSystem<BufferState>) -> Self {
        let chunk_index_state = ChunkIndexState::new(buffer_state.clone());
        let bucket_manager = MeshBucketManager::new(Self::NUM_BUFFERS_PER_SIDE);
        let mut buffer_state = buffer_state.get_mut();

        // Create buffers for each side
        for side in BlockSide::all() {
            let vertex_buffer_name = Self::get_vertex_buffer_name(side);
            let index_buffer_name = Self::get_index_buffer_name(side);
            let indirect_buffer_name = Self::get_indirect_buffer_name(side);

            buffer_state.create_buffer(
                vertex_buffer_name,
                wgpu::BufferDescriptor {
                    label: Some(vertex_buffer_name),
                    size: bucket_manager.get_vertex_bucket_buffer_size(),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                },
            );

            buffer_state.create_buffer(
                index_buffer_name,
                wgpu::BufferDescriptor {
                    label: Some(index_buffer_name),
                    size: bucket_manager.get_index_bucket_buffer_size(),
                    usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                },
            );

            buffer_state.create_buffer(
                indirect_buffer_name,
                wgpu::BufferDescriptor {
                    label: Some(indirect_buffer_name),
                    size: bucket_manager.get_indirect_bucket_buffer_size(),
                    usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                },
            );

            // Initialize indirect buffer
            for i in 0..bucket_manager.get_number_buckets_per_buffer() {
                let offset = i * std::mem::size_of::<DrawIndexedIndirectArgs>() as u64;
                let index_count = bucket_manager.get_number_indices_per_bucket() as u32;
                let indirect_args = DrawIndexedIndirectArgs {
                    index_count,
                    instance_count: 0,
                    first_index: (i as u32 * index_count),
                    base_vertex: (i * bucket_manager.get_number_vertices_per_bucket()) as i32,
                    first_instance: 0,
                };

                buffer_state.write_buffer(indirect_buffer_name, offset, indirect_args.as_bytes());
            }
        }

        MeshManager {
            bucket_manager,
            chunk_index_state,
            least_recently_meshed_chunks: LruCache::new(NonZeroUsize::new(10000).unwrap()),
        }
    }

    /// Generates a mesh for a voxel chunk and returns the mesh object.
    ///
    /// # Arguments
    ///
    /// * `chunk` - Reference to the chunk to generate a mesh for
    /// * `chunk_position` - 3D position of the chunk in the world
    ///
    /// # Returns
    ///
    /// A `Mesh` object containing the optimized geometry for the chunk
    ///
    /// # Implementation Details
    ///
    /// - Uses greedy meshing algorithm to optimize geometry
    /// - Separates mesh data by block side for efficient culling
    /// - Updates the LRU cache to track meshed chunks
    pub fn generate_mesh_for_chunk(
        &mut self,
        chunk: MtResource<Chunk>,
        sides_to_generate: &Vec<BlockSide>,
    ) -> Vec<BufferWriteCommand> {
        let chunk = chunk.get();

        let chunk_index_buffer_write_commands = self
            .chunk_index_state
            .load_chunk_positions(&vec![chunk.position]);
        let chunk_index = self
            .chunk_index_state
            .get_index_for_position(chunk.position);

        let mesh = Mesh::greedy_sided(&chunk, chunk_index, sides_to_generate);

        let mut mesh_write_commands = self.prepare_mesh_for_write(chunk.position, mesh);

        mesh_write_commands.extend(chunk_index_buffer_write_commands);

        mesh_write_commands
    }

    /// Prepares a mesh for writing to GPU buffers.
    ///
    /// # Arguments
    ///
    /// * `chunk_position` - 3D position of the chunk in the world
    /// * `mesh` - The mesh data to prepare for GPU upload
    ///
    /// # Returns
    ///
    /// A vector of `BufferWriteCommand` objects that can be submitted to the buffer state
    ///
    /// # Implementation Details
    ///
    /// - Allocates bucket space for the mesh data
    /// - Creates buffer write commands for vertices, indices, and indirect draw commands
    /// - Updates the chunk index state to track the chunk's buffer location
    pub fn prepare_mesh_for_write(
        &mut self,
        chunk_position: cgmath::Point3<i32>,
        mesh: Mesh,
    ) -> Vec<BufferWriteCommand> {
        let mut write_commands = Vec::new();
        let vertex_lens = mesh.get_vertex_lens();

        while !self.bucket_manager.can_allocate_buckets(vertex_lens)
            || !self.chunk_index_state.can_allocate_index()
        {
            let (lru_chunk_position, _) = self.least_recently_meshed_chunks.pop_lru().unwrap();
            let unload_commands = self.unload_chunk_positions(&vec![lru_chunk_position]);
            write_commands.extend(unload_commands);
        }

        self.least_recently_meshed_chunks.push(chunk_position, ());

        let mut mesh = mesh.mesh;

        // Process each side of the mesh
        for side_mesh in mesh.iter_mut() {
            if side_mesh.vertices.is_empty() {
                continue;
            }

            // Get buckets for this side
            let buckets = self.bucket_manager.allocate_buckets(
                chunk_position,
                std::mem::take(&mut side_mesh.vertices),
                std::mem::take(&mut side_mesh.indices),
                side_mesh.side,
            );

            // Create write commands for each bucket
            for (bucket, vertices, indices) in buckets {
                write_commands.push(BufferWriteCommand {
                    name: format!(
                        "Vertex Write - Chunk Position {:?} - Side {:?} - Bucket {:?}",
                        chunk_position, side_mesh.side, bucket
                    ),
                    buffer_name: MeshManager::get_vertex_buffer_name(side_mesh.side),
                    offset: bucket.vertex_buffer_offset,
                    data: Box::new(vertices),
                });

                let indices_len = indices.len();

                write_commands.push(BufferWriteCommand {
                    name: format!(
                        "Index Write - Chunk Position {:?} - Side {:?} - Bucket {:?}",
                        chunk_position, side_mesh.side, bucket
                    ),
                    buffer_name: MeshManager::get_index_buffer_name(side_mesh.side),
                    offset: bucket.index_buffer_offset,
                    data: Box::new(indices),
                });

                write_commands.push(BufferWriteCommand {
                    name: format!(
                        "Indirect Write - Chunk Position {:?} - Side {:?} - Bucket {:?}",
                        chunk_position, side_mesh.side, bucket
                    ),
                    buffer_name: MeshManager::get_indirect_buffer_name(side_mesh.side),
                    offset: bucket.indirect_bucket_index
                        * std::mem::size_of::<DrawIndexedIndirectArgs>() as u64,
                    data: Box::new(DrawIndexedIndirectArgs {
                        index_count: indices_len as u32,
                        instance_count: 1,
                        first_index: (bucket.indirect_bucket_index
                            * self.bucket_manager.get_number_indices_per_bucket())
                            as u32,
                        base_vertex: (bucket.indirect_bucket_index
                            * self.bucket_manager.get_number_vertices_per_bucket())
                            as i32,
                        first_instance: 0,
                    }),
                });
            }
        }

        write_commands
    }

    /// Checks if a chunk has been meshed.
    ///
    /// # Arguments
    ///
    /// * `chunk_position` - 3D position of the chunk to check
    ///
    /// # Returns
    ///
    /// `true` if the chunk has been meshed, `false` otherwise
    pub fn is_chunk_meshed(&mut self, chunk_position: cgmath::Point3<i32>) -> bool {
        let is_chunk_allocated = self.bucket_manager.is_chunk_allocated(chunk_position);
        if is_chunk_allocated {
            self.least_recently_meshed_chunks.promote(&chunk_position);
        }

        is_chunk_allocated
    }

    /// Unloads chunks from GPU memory and frees their allocated buckets.
    ///
    /// # Arguments
    ///
    /// * `chunk_positions` - Vector of chunk positions to unload
    ///
    /// # Returns
    ///
    /// A vector of `BufferWriteCommand` objects to update the GPU buffers
    ///
    /// # Implementation Details
    ///
    /// - Frees bucket allocations for each chunk
    /// - Updates indirect draw commands to disable rendering for unloaded chunks
    /// - Removes chunks from the index state and LRU cache
    pub fn unload_chunk_positions(
        &mut self,
        chunk_positions: &Vec<cgmath::Point3<i32>>,
    ) -> Vec<BufferWriteCommand> {
        self.chunk_index_state
            .unload_chunk_positions(chunk_positions);
        let buckets_deallocated = self.bucket_manager.deallocate_buckets(chunk_positions);

        let mut write_commands = Vec::new();
        let index_count = self.bucket_manager.get_number_indices_per_bucket() as u32;

        for bucket in buckets_deallocated {
            write_commands.push(BufferWriteCommand {
                name: format!("Indirect Write (Deallocation) - Chunk Positions {:?} - Side {:?} - Bucket {:?}", chunk_positions, bucket.side, bucket),
                buffer_name: MeshManager::get_indirect_buffer_name(bucket.side),
                offset: bucket.indirect_bucket_index * std::mem::size_of::<DrawIndexedIndirectArgs>() as u64,
                data: Box::new(DrawIndexedIndirectArgs {
                    index_count:0,
                    instance_count: 0,
                    first_index: (bucket.indirect_bucket_index as u32 * index_count),
                    base_vertex: (bucket.indirect_bucket_index * self.bucket_manager.get_number_vertices_per_bucket()) as i32,
                    first_instance: 0,
                }),
            })
        }

        write_commands
    }

    /// Gets the number of indirect draw commands per block side.
    ///
    /// # Returns
    ///
    /// The number of indirect draw commands as a u32
    ///
    /// # Implementation Details
    ///
    /// This is used by the renderer to know how many draw calls to issue
    /// when using multi-draw-indirect rendering.
    pub fn get_number_indirect_commands(&self) -> u32 {
        self.bucket_manager.get_number_buckets_per_buffer() as u32
    }
}
