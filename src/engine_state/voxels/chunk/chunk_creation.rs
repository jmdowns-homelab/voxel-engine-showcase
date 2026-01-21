//! # Chunk Creation Module
//!
//! This module provides functionality for efficiently creating and populating chunks
//! using a memory-optimized storage strategy. It implements a builder pattern that
//! maintains the relationship between the solidity bit vector and the block storage.
//!
//! ## Memory Optimization
//! 
//! The `ChunkCreationIterator` builds chunks using two main components:
//! 1. A bit vector (`solid_array`) that tracks which positions contain solid blocks
//! 2. A vector (`blocks`) that stores only the non-air blocks
//!
//! This approach minimizes memory usage by:
//! - Using only 1 bit per air block (instead of storing full block data)
//! - Only storing actual block data for non-air blocks
//! - Maintaining efficient spatial locality for common operations

use bitvec::vec::BitVec;
use cgmath::Point3;

use crate::engine_state::voxels::block::{block_type::BlockType, Block};

use super::{Chunk, CHUNK_DIMENSION_WRAPPED, CHUNK_PLANE_SIZE_WRAPPED, CHUNK_SIZE_WRAPPED};

/// A builder for efficiently creating and populating chunks with optimized memory usage.
///
/// This struct implements a builder pattern that maintains the relationship between:
/// 1. The bit vector tracking solid blocks (`solid_array`)
/// 2. The vector storing actual block data (`blocks`)
///
/// The builder ensures that these two data structures remain consistent as blocks are added.
pub struct ChunkCreationIterator {
    /// The world position of the chunk being created
    position: Point3<i32>,
    /// Bit vector where each bit represents whether a block is solid (1) or air (0)
    /// 
    /// This is stored with padding (CHUNK_DIMENSION_WRAPPED) to simplify neighbor lookups.
    solid_array: BitVec,
    /// Precomputed offsets into the blocks vector for each Z plane
    /// 
    /// This is used to accelerate rendering by quickly finding where each
    /// horizontal plane's blocks begin in the blocks vector.
    offsets_at_plane: Vec<u32>,
    /// Vector containing only the non-air blocks, in the order they appear in the chunk
    /// 
    /// The index of a block in this vector corresponds to the number of set bits
    /// before its position in the `solid_array`.
    blocks: Vec<Block>,
    /// Current X position within the chunk (1..CHUNK_DIMENSION_WRAPPED-1)
    /// 
    /// Note the 1-based indexing to account for the padding in the solid array.
    local_x: usize,
    /// Current Y position within the chunk (1..CHUNK_DIMENSION_WRAPPED-1)
    local_y: usize,
    /// Current Z position within the chunk (1..CHUNK_DIMENSION_WRAPPED-1)
    local_z: usize,
    /// Current offset into the blocks vector (number of solid blocks added so far)
    block_offset: u32,
}

impl ChunkCreationIterator {
    /// Creates a new `ChunkCreationIterator` for building a chunk at the given position.
    ///
    /// # Arguments
    /// * `position` - The world position of the chunk to create
    ///
    /// # Returns
    /// A new `ChunkCreationIterator` ready to build a chunk
    pub fn new(position: Point3<i32>) -> Self {
        // Initialize the solid array with padding for the chunk boundaries
        let mut solid_array = BitVec::with_capacity(CHUNK_SIZE_WRAPPED);
        // Add padding for the first plane and first row
        for _ in 0..(CHUNK_PLANE_SIZE_WRAPPED + CHUNK_DIMENSION_WRAPPED + 1) {
            solid_array.push(false);
        }
        ChunkCreationIterator {
            position,
            solid_array,
            offsets_at_plane: Vec::new(),
            blocks: Vec::new(),
            local_x: 1,  // Start at (1,1,1) to leave room for boundary
            local_y: 1,
            local_z: 1,
            block_offset: 0,
        }
    }

    /// Finalizes the chunk creation and returns the constructed `Chunk`.
    ///
    /// # Returns
    /// The fully constructed `Chunk` with all added blocks
    pub fn return_chunk(self) -> Chunk {
        Chunk {
            position: self.position,
            solid_array: self.solid_array,
            offsets_at_plane: self.offsets_at_plane,
            blocks: self.blocks,
        }
    }

    /// Adds a block to the chunk at the current position and advances the position.
    ///
    /// This method maintains the relationship between the solidity bit vector
    /// and the block storage:
    /// - Updates the `solid_array` bit for the current position
    /// - Only adds non-air blocks to the `blocks` vector
    /// - Handles boundary conditions and padding
    ///
    /// # Arguments
    /// * `block_type` - The type of block to add
    ///
    /// # Note
    /// The method automatically handles the 1-block padding around the chunk
    /// (CHUNK_DIMENSION_WRAPPED) to simplify neighbor lookups during rendering.
    pub fn push_block_type(&mut self, block_type: BlockType) {
        let is_solid = block_type != BlockType::AIR;
        self.solid_array.push(is_solid);
        
        // Only store non-air blocks to save memory
        if is_solid {
            self.blocks.push(Block::new(block_type));
            self.block_offset += 1;
        }
        
        // Move to the next position
        self.local_x += 1;
        
        // Handle end of row (X boundary)
        if self.local_x == CHUNK_DIMENSION_WRAPPED - 1 {
            // Add padding at the end of the row
            self.solid_array.push(false);
            self.solid_array.push(false);
            
            // Move to the next row
            self.local_x = 1;
            self.local_y += 1;
            
            // Handle end of plane (Y boundary)
            if self.local_y == CHUNK_DIMENSION_WRAPPED - 1 {
                // Save the current block offset for this plane
                self.offsets_at_plane.push(self.block_offset);
                
                // Add padding at the end of the plane
                for _ in 0..2 * CHUNK_DIMENSION_WRAPPED {
                    self.solid_array.push(false);
                }
                
                // Move to the next plane
                self.local_y = 1;
                self.local_z += 1;
                
                // Handle end of chunk (Z boundary)
                if self.local_z == CHUNK_DIMENSION_WRAPPED - 1 {
                    // Add final padding
                    for _ in 0..CHUNK_PLANE_SIZE_WRAPPED - CHUNK_DIMENSION_WRAPPED - 1 {
                        self.solid_array.push(false);
                    }
                }
            }
        }
    }
}
