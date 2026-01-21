//! Memory management for mesh data using a bucket-based allocation strategy.
//!
//! This module implements a memory management system that allocates fixed-size
//! buckets of GPU memory for storing mesh data. This approach provides several benefits:
//! - Reduces memory fragmentation by using fixed-size allocations
//! - Enables efficient GPU-side culling of geometry
//! - Reduces draw calls through multi-draw indirect
//! - Improves memory access patterns for better cache utilization
//!
//! # Bucket Organization
//! - Each bucket holds up to 1024 vertices and 1536 indices (1.5 indices per vertex)
//! - Buckets are organized by block side (FRONT, BACK, LEFT, RIGHT, TOP, BOTTOM)
//! - Multiple buffers are created per side (currently 1 per side)
//! - Each buffer contains 2048 buckets
//!
//! See also: [Bucket-Based Rendering Strategy](../../../../docs/domain_flows/system/bucket_based_rendering.md)

use std::collections::{HashMap, VecDeque};

use cgmath::Point3;
use wgpu::util::DrawIndexedIndirectArgs;

use crate::{
    engine_state::voxels::block::block_side::BlockSide,
    engine_state::rendering::Vertex
};

/// Represents a location within a bucket-based memory allocation.
///
/// This struct tracks the position of mesh data within the GPU buffers
/// using a bucket-based allocation strategy.
#[derive(Clone, Debug)]
pub struct BucketLocation {
    /// The buffer number this bucket belongs to
    #[allow(dead_code)]
    pub buffer_number: usize,
    /// Offset in the vertex buffer in bytes
    pub vertex_buffer_offset: u64,
    /// Offset in the index buffer in bytes
    pub index_buffer_offset: u64,
    /// Index for indirect drawing commands
    pub indirect_bucket_index: u64,
    /// The block side this bucket is associated with
    pub side: BlockSide,
}

/// Manages allocation and deallocation of mesh data in fixed-size buckets.
///
/// This manager uses a bucket-based approach to allocate memory for mesh data,
/// which helps reduce memory fragmentation and improves rendering performance.
pub struct MeshBucketManager {
    available_buckets: [VecDeque<BucketLocation>; 6],
    chunk_position_to_used_buckets: HashMap<Point3<i32>, Vec<BucketLocation>>,
}

impl MeshBucketManager {
    /// Number of buckets allocated per buffer
    const NUM_BUCKETS_PER_BUFFER: u64 = 2048;
    
    /// Maximum number of vertices that can be stored in a single bucket
    const NUM_VERTICES_PER_BUCKET: u64 = 1024;
    
    /// Maximum number of indices that can be stored in a single bucket (1.5x vertices)
    const NUM_INDICES_PER_BUCKET: u64 = (Self::NUM_VERTICES_PER_BUCKET * 3) / 2;

    /// Size of a vertex bucket in bytes
    const VERTEX_BUCKET_SIZE: u64 =
        Self::NUM_VERTICES_PER_BUCKET * std::mem::size_of::<Vertex>() as u64;
        
    /// Size of an index bucket in bytes
    const INDEX_BUCKET_SIZE: u64 = Self::NUM_INDICES_PER_BUCKET * std::mem::size_of::<u32>() as u64;

    pub fn new(num_buffers_per_side: usize) -> Self {
        let mut available_buckets = [
            VecDeque::new(),
            VecDeque::new(),
            VecDeque::new(),
            VecDeque::new(),
            VecDeque::new(),
            VecDeque::new(),
        ];
        for side in BlockSide::all() {
            for buffer_number in 0..num_buffers_per_side {
                for bucket_index in 0..Self::NUM_BUCKETS_PER_BUFFER {
                    available_buckets[side as usize].push_back(BucketLocation {
                        buffer_number,
                        vertex_buffer_offset: bucket_index * Self::VERTEX_BUCKET_SIZE,
                        index_buffer_offset: bucket_index * Self::INDEX_BUCKET_SIZE,
                        indirect_bucket_index: bucket_index,
                        side,
                    });
                }
            }
        }

        Self {
            available_buckets,
            chunk_position_to_used_buckets: HashMap::new(),
        }
    }

    pub fn can_allocate_buckets(&self, num_vertices_per_side: [u64; 6]) -> bool {
        for side in BlockSide::all() {
            let num_vertices = num_vertices_per_side[side as usize];
            let num_buckets_needed =
                num_vertices.div_ceil(Self::NUM_VERTICES_PER_BUCKET);

            if self.available_buckets[side as usize].len() < num_buckets_needed as usize {
                return false;
            }
        }

        true
    }

    pub fn allocate_buckets(
        &mut self,
        chunk_position: Point3<i32>,
        vertex_vec: Vec<Vertex>,
        index_vec: Vec<u32>,
        side: BlockSide,
    ) -> Vec<(BucketLocation, Vec<Vertex>, Vec<u32>)> {
        assert_eq!(
            index_vec.len(),
            vertex_vec.len() * 3 / 2,
            "Index vector must be 1.5 times the length of vertex vector"
        );

        let num_vertices = vertex_vec.len() as u64;
        let num_buckets_needed =
            num_vertices.div_ceil(Self::NUM_VERTICES_PER_BUCKET);

        let mut allocated_buckets = Vec::new();
        let mut used_buckets = Vec::new();

        let mut remaining_vertices = vertex_vec;
        let mut remaining_indices = index_vec;

        let mut current_vertex_count = 0;

        for _ in 0..num_buckets_needed {
            let bucket = self.available_buckets[side as usize].pop_front().unwrap();

            let vertex_count =
                (Self::NUM_VERTICES_PER_BUCKET as usize).min(remaining_vertices.len());
            let index_count = vertex_count * 3 / 2; // This will always fit since we maintain the 1.5x ratio

            let bucket_vertices: Vec<Vertex> = remaining_vertices.drain(..vertex_count).collect();
            let bucket_indices: Vec<u32> = remaining_indices
                .drain(..index_count)
                .collect::<Vec<u32>>()
                .into_iter()
                .map(|x| x - current_vertex_count)
                .collect();

            current_vertex_count += bucket_vertices.len() as u32;

            allocated_buckets.push((bucket.clone(), bucket_vertices, bucket_indices));
            used_buckets.push(bucket);
        }

        if let std::collections::hash_map::Entry::Vacant(e) = self
            .chunk_position_to_used_buckets.entry(chunk_position) {
            e.insert(used_buckets);
        } else {
            self.chunk_position_to_used_buckets
                .get_mut(&chunk_position)
                .unwrap()
                .extend(used_buckets);
        }

        allocated_buckets
    }

    pub fn deallocate_buckets(
        &mut self,
        chunk_positions: &Vec<Point3<i32>>,
    ) -> Vec<BucketLocation> {
        let mut buckets_deallocated = Vec::new();

        for chunk_position in chunk_positions {
            if let Some(available_buckets) =
                self.chunk_position_to_used_buckets.remove(chunk_position)
            {
                for bucket in available_buckets.iter() {
                    let side = bucket.side;
                    self.available_buckets[side as usize].push_back(bucket.clone());
                }
                buckets_deallocated.extend(available_buckets);
            }
        }

        buckets_deallocated
    }

    pub fn is_chunk_allocated(&mut self, chunk_position: Point3<i32>) -> bool {
        self.chunk_position_to_used_buckets
            .get(&chunk_position)
            .is_some()
    }

    pub fn get_number_vertices_per_bucket(&self) -> u64 {
        Self::NUM_VERTICES_PER_BUCKET
    }

    pub fn get_vertex_bucket_buffer_size(&self) -> u64 {
        Self::NUM_BUCKETS_PER_BUFFER * Self::VERTEX_BUCKET_SIZE
    }

    pub fn get_number_indices_per_bucket(&self) -> u64 {
        Self::NUM_INDICES_PER_BUCKET
    }

    pub fn get_index_bucket_buffer_size(&self) -> u64 {
        Self::NUM_BUCKETS_PER_BUFFER * Self::INDEX_BUCKET_SIZE
    }

    pub fn get_number_buckets_per_buffer(&self) -> u64 {
        Self::NUM_BUCKETS_PER_BUFFER
    }

    pub fn get_indirect_bucket_buffer_size(&self) -> u64 {
        Self::NUM_BUCKETS_PER_BUFFER * std::mem::size_of::<DrawIndexedIndirectArgs>() as u64
    }
}
