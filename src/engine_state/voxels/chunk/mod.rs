//! # Chunk Module
//!
//! This module provides the `Chunk` struct and related functionality for managing
//! 16x16x16 blocks of voxel data. It includes chunk generation algorithms and
//! utilities for working with chunk data.
//!
//! ## Memory Optimization
//! 
//! Chunks use a memory-efficient storage strategy to handle the potentially large
//! number of blocks in a voxel world:
//! - `solid_array`: A bit vector (1 bit per block) indicating which blocks are solid
//! - `blocks`: A vector containing only non-air blocks, in the order they appear in the chunk
//!
//! This approach provides significant memory savings because:
//! 1. Air blocks (which are common) only consume 1 bit each
//! 2. Only non-air blocks are stored in the `blocks` vector
//!
//! For example, in a chunk with only two solid blocks (one at each end), the storage would be:
//! - `solid_array`: `1000...0001` (CHUNK_SIZE bits)
//! - `blocks`: `[block1, block2]` (only 2 blocks stored)
//!
//! ### Performance Characteristics
//! - **Solidity Check**: O(1) - Just check the bit in `solid_array`
//! - **Block Lookup**: O(n) in worst case - Requires counting set bits up to the target position
//! - **Memory Usage**: ~1 bit per air block + sizeof(Block) per solid block + overhead

use bitvec::prelude::BitVec;
use cgmath::Point3;
use chunk_creation::ChunkCreationIterator;
use noise::NoiseFn;
use noise::Perlin;

use super::block::block_side::BlockSide;
use super::block::block_type::BlockType;
use super::block::Block;

mod chunk_creation;
pub mod chunk_iteration;

/// The dimension (width, height, depth) of a chunk in blocks.
pub const CHUNK_DIMENSION: i32 = 16;
/// The number of blocks in a single 2D plane of a chunk (CHUNK_DIMENSION²).
pub const CHUNK_PLANE_SIZE: i32 = CHUNK_DIMENSION * CHUNK_DIMENSION;
/// The total number of blocks in a chunk (CHUNK_DIMENSION³).
pub const CHUNK_SIZE: i32 = CHUNK_PLANE_SIZE * CHUNK_DIMENSION;
/// The dimension of a chunk including an extra layer of blocks on each side for neighbor lookups.
pub const CHUNK_DIMENSION_WRAPPED: usize = (CHUNK_DIMENSION + 2) as usize;
/// The number of blocks in a wrapped 2D chunk plane.
pub const CHUNK_PLANE_SIZE_WRAPPED: usize = CHUNK_DIMENSION_WRAPPED * CHUNK_DIMENSION_WRAPPED;
/// The total number of blocks in a wrapped chunk.
pub const CHUNK_SIZE_WRAPPED: usize = CHUNK_PLANE_SIZE_WRAPPED * CHUNK_DIMENSION_WRAPPED;

/// Represents a 16x16x16 collection of voxel blocks in the world.
///
/// Chunks are the fundamental unit of world data and are used to efficiently
/// manage and render the voxel world. Each chunk maintains its position in the
/// world and contains data about its blocks.
pub struct Chunk {
    /// The position of this chunk in chunk coordinates (not block coordinates).
    pub position: Point3<i32>,
    
    /// A bit vector where each bit represents whether the corresponding block is solid (1) or air (0).
    /// 
    /// The bits are stored in row-major order (x, then z, then y) and include padding for
    /// neighbor lookups (CHUNK_DIMENSION_WRAPPED in each dimension).
    /// 
    /// This provides O(1) solidity checks but requires O(n) time to look up a specific block
    /// in the blocks vector, where n is the number of solid blocks before the target block.
    pub solid_array: BitVec,
    
    /// Precomputed offsets for each Y plane in the chunk, used for rendering optimization.
    #[allow(dead_code)]
    pub offsets_at_plane: Vec<u32>,
    
    /// The actual block data for this chunk, containing only non-air blocks.
    /// 
    /// The blocks are stored in the order they appear in the chunk (row-major order),
    /// but only for positions where `solid_array` has a 1 bit. This means the index
    /// of a block in this vector is equal to the number of set bits before its position
    /// in the `solid_array`.
    /// 
    /// For example, if `solid_array` is `10101`, then `blocks` will contain 3 blocks,
    /// at the positions where bits are set to 1.
    pub blocks: Vec<Block>,
}

/// Threshold above which Perlin noise is considered solid for terrain generation.
pub const PERLIN_POSITIVE_THRESHOLD: f64 = 0.2;
/// Threshold below which Perlin noise is considered empty for terrain generation.
pub const PERLIN_NEGATIVE_THRESHOLD: f64 = -0.2;
/// Scaling factor applied to world coordinates when sampling Perlin noise.
pub const PERLIN_SCALE_FACTOR: f64 = 0.02;

impl Chunk {
    /// Creates a new, completely empty chunk (all blocks are air).
    /// 
    /// # Arguments
    /// * `position` - The chunk coordinates of the new chunk
    /// 
    /// # Returns
    /// A new `Chunk` instance filled with air blocks.
    #[allow(dead_code)]
    pub fn empty(position: &Point3<i32>) -> Self {
        let mut cci = ChunkCreationIterator::new(*position);

        for _ in 0..CHUNK_SIZE {
            cci.push_block_type(BlockType::AIR);
        }

        cci.return_chunk()
    }

    /// Creates a new chunk with random blocks (for testing purposes).
    /// 
    /// # Arguments
    /// * `position` - The chunk coordinates of the new chunk
    /// 
    /// # Returns
    /// A new `Chunk` with randomly placed blocks.
    #[allow(dead_code)]
    pub fn random(position: &Point3<i32>) -> Self {
        let mut cci = ChunkCreationIterator::new(*position);

        let sparseness = 0.9;

        for _ in 0..CHUNK_SIZE {
            let random_value = fastrand::f64();
            if random_value < sparseness {
                cci.push_block_type(BlockType::AIR);
            } else {
                cci.push_block_type(BlockType::DIRT);
            }
        }

        cci.return_chunk()
    }

    /// Generates a chunk using Perlin noise for natural-looking terrain.
    /// 
    /// The terrain is generated by sampling 3D Perlin noise and applying thresholds
    /// to determine which blocks are solid. The result resembles natural terrain
    /// with caves and overhangs.
    /// 
    /// # Arguments
    /// * `position` - The chunk coordinates where the chunk will be placed
    /// 
    /// # Returns
    /// A new `Chunk` with terrain generated using Perlin noise.
    #[allow(dead_code)]
    pub fn perlin(position: &Point3<i32>) -> Self {
        let perlin = Perlin::new(0);
        let mut cci = ChunkCreationIterator::new(*position);

        for k in 0..CHUNK_DIMENSION {
            for j in 0..CHUNK_DIMENSION {
                for i in 0..CHUNK_DIMENSION {
                    let bposition = Point3::<i32>::new(
                        i + CHUNK_DIMENSION * position.x,
                        j + CHUNK_DIMENSION * position.y,
                        k + CHUNK_DIMENSION * position.z,
                    );
                    let perlin_sample =
                        perlin.get(Self::to_perlin_pos(bposition, PERLIN_SCALE_FACTOR));
                    if !(PERLIN_NEGATIVE_THRESHOLD..=PERLIN_POSITIVE_THRESHOLD).contains(&perlin_sample)
                    {
                        cci.push_block_type(BlockType::get_random_type());
                    } else {
                        cci.push_block_type(BlockType::AIR);
                    }
                }
            }
        }

        cci.return_chunk()
    }

    /// Converts chunk-relative block coordinates to world-space coordinates for Perlin noise sampling.
    /// 
    /// # Arguments
    /// * `pos` - The block position within the chunk
    /// * `scale_factor` - Scaling factor to apply to the world coordinates
    /// 
    /// # Returns
    /// An array of [x, y, z] coordinates scaled for Perlin noise sampling.
    fn to_perlin_pos(pos: Point3<i32>, scale_factor: f64) -> [f64; 3] {
        [
            (pos.x as f64 * scale_factor),
            (pos.y as f64 * scale_factor),
            (pos.z as f64 * scale_factor),
        ]
    }

    /// Creates a new chunk filled with solid blocks (for testing).
    /// 
    /// # Arguments
    /// * `position` - The chunk coordinates of the new chunk
    /// 
    /// # Returns
    /// A new `Chunk` completely filled with solid blocks.
    #[allow(dead_code)]
    pub fn solid(position: &Point3<i32>) -> Self {
        let mut cci = ChunkCreationIterator::new(*position);

        for _ in 0..CHUNK_SIZE {
            cci.push_block_type(BlockType::DIRT);
        }

        cci.return_chunk()
    }

    /// Creates a new chunk with a checkerboard pattern (for testing).
    /// 
    /// The pattern alternates between solid and air blocks in a 3D grid.
    /// 
    /// # Arguments
    /// * `position` - The chunk coordinates of the new chunk
    /// 
    /// # Returns
    /// A new `Chunk` with a 3D checkerboard pattern.
    #[allow(dead_code)]
    pub fn checkerboard(position: &Point3<i32>) -> Self {
        let mut push_air = false;

        let mut cci = ChunkCreationIterator::new(*position);
        for i in 0..CHUNK_SIZE {
            if push_air {
                cci.push_block_type(BlockType::AIR);
            } else {
                cci.push_block_type(BlockType::DIRT);
            }

            push_air = !push_air;

            if (i + 1) % CHUNK_DIMENSION == 0 {
                push_air = !push_air
            }

            if (i + 1) % CHUNK_PLANE_SIZE == 0 {
                push_air = !push_air
            }
        }

        cci.return_chunk()
    }

    /// Gets a reference to the block at the specified chunk-relative coordinates.
    /// 
    /// # Arguments
    /// * `cx` - X coordinate within the chunk (0..CHUNK_DIMENSION)
    /// * `cy` - Y coordinate within the chunk (0..CHUNK_DIMENSION)
    /// * `cz` - Z coordinate within the chunk (0..CHUNK_DIMENSION)
    /// 
    /// # Returns
    /// A reference to the block at the specified coordinates.
    /// 
    /// # Panics
    /// Panics if the coordinates are out of bounds.
    pub fn _get_block_at(&self, cx: usize, cy: usize, cz: usize) -> &Block {
        let mut offset = self.offsets_at_plane[cz] as usize;
        for j in 0..cy {
            for i in 0..CHUNK_DIMENSION as usize {
                if self.is_block_solid(i, j, cz) {
                    offset += 1;
                }
            }
        }
        for i in 0..cx {
            if self.is_block_solid(i, cy, cz) {
                offset += 1;
            }
        }
        &self.blocks[offset]
    }

    /// Determines which faces of the block at (x,y,z) are adjacent to non-solid blocks.
    /// 
    /// This is used for face culling during rendering to avoid drawing faces that
    /// are occluded by adjacent solid blocks.
    /// 
    /// # Arguments
    /// * `x` - X coordinate within the chunk
    /// * `y` - Y coordinate within the chunk
    /// * `z` - Z coordinate within the chunk
    /// 
    /// # Returns
    /// An array of 6 booleans, where each boolean indicates if the corresponding
    /// face (in BlockSide order) is adjacent to a non-solid block and should be rendered.
    pub fn generate_adjacent_blocks(&self, x: usize, y: usize, z: usize) -> [bool; 6] {
        //This accounts for the chunk wrapping
        let i = x + 1;
        let j = y + 1;
        let k = z + 1;

        let mut adjacency_data = [false; 6];
        adjacency_data[BlockSide::FRONT as usize] = self.is_block_solid(i - 1, j, k);
        adjacency_data[BlockSide::BACK as usize] = self.is_block_solid(i + 1, j, k);
        adjacency_data[BlockSide::LEFT as usize] = self.is_block_solid(i, j, k - 1);
        adjacency_data[BlockSide::RIGHT as usize] = self.is_block_solid(i, j, k + 1);
        adjacency_data[BlockSide::TOP as usize] = self.is_block_solid(i, j + 1, k);
        adjacency_data[BlockSide::BOTTOM as usize] = self.is_block_solid(i, j - 1, k);
        adjacency_data
    }

    /// Checks if the block at the specified chunk-relative coordinates is solid.
    /// 
    /// # Arguments
    /// * `cx` - X coordinate within the chunk
    /// * `cy` - Y coordinate within the chunk
    /// * `cz` - Z coordinate within the chunk
    /// 
    /// # Returns
    /// `true` if the block is solid, `false` if it's air or out of bounds.
    pub fn is_block_solid(&self, cx: usize, cy: usize, cz: usize) -> bool {
        self.solid_array[cx
            + CHUNK_DIMENSION_WRAPPED * cy
            + CHUNK_PLANE_SIZE_WRAPPED * cz]
    }

    #[allow(dead_code)]
    /// Updates the solid state of a block in the solid array.
    /// 
    /// This is used to maintain the solid_array bit vector when blocks are
    /// added or removed from the chunk.
    /// 
    /// # Arguments
    /// * `cx` - X coordinate within the chunk
    /// * `cy` - Y coordinate within the chunk
    /// * `cz` - Z coordinate within the chunk
    /// * `solid_value` - The new solid state for the block
    pub fn update_solid_array(&mut self, cx: usize, cy: usize, cz: usize, solid_value: bool) {
        self.solid_array.set(
            cx + CHUNK_DIMENSION_WRAPPED * cy + CHUNK_PLANE_SIZE_WRAPPED * cz,
            solid_value,
        );
    }
}
