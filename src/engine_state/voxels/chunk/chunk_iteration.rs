//! # Chunk Iteration Module
//!
//! This module provides an iterator for efficiently traversing all non-air blocks
//! in a chunk while respecting the memory-optimized storage format.
//!
//! ## Memory-Aware Iteration
//!
//! The `ChunkBlockIterator` is designed to work with the chunk's dual-storage format:
//! 1. It uses the `solid_array` bit vector to quickly skip over air blocks
//! 2. It maintains a separate index into the `blocks` vector that only contains non-air blocks
//! 3. It handles the chunk's boundary padding automatically
//!
//! This approach provides efficient iteration over only the solid blocks while
//! maintaining the spatial relationship between blocks.

use cgmath::Point3;

use crate::engine_state::voxels::block::Block;

use super::{Chunk, CHUNK_DIMENSION_WRAPPED, CHUNK_PLANE_SIZE_WRAPPED};

/// An iterator over all non-air blocks in a chunk.
///
/// This iterator efficiently traverses the chunk's blocks while respecting the
/// memory-optimized storage format. It maintains two main pieces of state:
/// 1. A position in the `solid_array` bit vector
/// 2. An index into the `blocks` vector containing actual block data
///
/// The iterator automatically skips air blocks and handles the chunk's boundary
/// padding, providing a clean interface to iterate over only the solid blocks.
pub struct ChunkBlockIterator<'a> {
    /// Reference to the chunk being iterated over
    chunk_ref: &'a Chunk,
    /// Current position in the solid array
    current_solid_offset: usize,
    /// Current position in the blocks vector
    current_block_offset: usize,
    /// Current X position within the chunk (1..CHUNK_DIMENSION_WRAPPED-1)
    local_x: usize,
    /// Current Y position within the chunk (1..CHUNK_DIMENSION_WRAPPED-1)
    local_y: usize,
    /// Current Z position within the chunk (1..CHUNK_DIMENSION_WRAPPED-1)
    local_z: usize,
}

impl<'a> ChunkBlockIterator<'a> {
    /// Creates a new `ChunkBlockIterator` for the given chunk.
    ///
    /// # Arguments
    /// * `chunk_ref` - A reference to the chunk to iterate over
    ///
    /// # Returns
    /// A new `ChunkBlockIterator` positioned at the first non-air block
    pub fn new(chunk_ref: &'a Chunk) -> Self {
        // Start after the initial padding (first plane + first row + first column)
        ChunkBlockIterator {
            chunk_ref,
            current_solid_offset: (1 + CHUNK_DIMENSION_WRAPPED + CHUNK_PLANE_SIZE_WRAPPED),
            current_block_offset: 0,
            local_x: 1,  // Start at (1,1,1) to skip boundary padding
            local_y: 1,
            local_z: 1,
        }
    }

    /// Gets the next non-air block in the chunk along with its position.
    ///
    /// This method efficiently finds the next solid block by:
    /// 1. Scanning the `solid_array` bit vector for the next set bit
    /// 2. Using the number of set bits encountered to index into the `blocks` vector
    /// 3. Automatically handling chunk boundaries and padding
    ///
    /// # Returns
    /// - `Some((position, block))` if another non-air block is found
    /// - `None` if there are no more blocks to iterate over
    ///
    /// # Performance
    /// - Best case: O(1) when the next block is solid
    /// - Worst case: O(n) when scanning through many air blocks
    ///   (where n is the number of bits scanned)
    pub fn get_next_block(&mut self) -> Option<(Point3<usize>, &Block)> {
        // Check if we've processed all blocks
        if self.current_block_offset >= self.chunk_ref.blocks.len() {
            return None;
        }

        // If we're at the end of a row, move to the next row
        if self.local_x == CHUNK_DIMENSION_WRAPPED - 1 {
            self.current_solid_offset += 2;  // Skip row padding
            self.local_x = 1;
            self.local_y += 1;
            
            // If we're at the end of a plane, move to the next plane
            if self.local_y == CHUNK_DIMENSION_WRAPPED - 1 {
                self.current_solid_offset += 2 * CHUNK_DIMENSION_WRAPPED;  // Skip plane padding
                self.local_y = 1;
                self.local_z += 1;
                
                // If we're at the end of the chunk, we're done
                if self.local_z == CHUNK_DIMENSION_WRAPPED - 1 {
                    return None;
                }
            }
        }

        // Iterate through the solid array until we find a solid block
        while self.current_solid_offset < self.chunk_ref.solid_array.len()
            && !self.chunk_ref.solid_array[self.current_solid_offset]
        {
            // Move to the next position
            self.local_x += 1;
            self.current_solid_offset += 1;
            
            // Handle end of row
            if self.local_x == CHUNK_DIMENSION_WRAPPED - 1 {
                self.current_solid_offset += 2;  // Skip row padding
                self.local_x = 1;
                self.local_y += 1;
                
                // Handle end of plane
                if self.local_y == CHUNK_DIMENSION_WRAPPED - 1 {
                    self.current_solid_offset += 2 * CHUNK_DIMENSION_WRAPPED;  // Skip plane padding
                    self.local_y = 1;
                    self.local_z += 1;
                    
                    // Handle end of chunk
                    if self.local_z == CHUNK_DIMENSION_WRAPPED - 1 {
                        return None;
                    }
                }
            }
        }
        
        // Get the current block and its position
        let block = &self.chunk_ref.blocks[self.current_block_offset];
        // Convert from 1-based to 0-based coordinates for the position
        let position = Point3::new(self.local_x - 1, self.local_y - 1, self.local_z - 1);

        // Prepare for the next iteration
        self.current_block_offset += 1;
        self.current_solid_offset += 1;
        self.local_x += 1;

        Some((position, block))
    }
}
