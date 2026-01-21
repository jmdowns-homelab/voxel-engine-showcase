//! # Block Module
//!
//! This module provides the core block-related functionality for the voxel engine.
//! It includes block type definitions, block face handling, and block data structures.

use block_type::BlockType;

pub mod block_side;
pub mod block_type;

/// The underlying integer type used to represent block types in memory.
/// This is used for efficient storage and serialization of block data.
pub type BlockTypeSize = u8;

/// Maps each block type to its corresponding texture indices for each face.
/// 
/// The outer array is indexed by `BlockType` as a `usize`.
/// The inner array contains 6 texture indices, one for each face in the order:
/// [Front, Back, Bottom, Top, Left, Right]
pub static BLOCK_TYPE_TO_TEXTURE_INDICES: [[usize; 6]; 4] = [
    [0, 0, 0, 0, 0, 0], // WOOD (all sides use texture 0)
    [1, 1, 1, 1, 1, 1], // DIRT (all sides use texture 1)
    [4, 4, 4, 4, 4, 4], // WHITE (all sides use texture 4)
    [2, 2, 2, 2, 3, 1], // GRASS (top: 3, bottom: 1, sides: 2)
];
// phf::Map<BlockType, [usize; 6]> =
// ::phf::Map {
//     key: 2126027241312876569,
//     disps: &[
//         (1, 0),
//     ],
//     entries: &[
//         (BlockType::WOOD, [0, 0, 0, 0, 0, 0]),
//         (BlockType::DIRT, [1, 1, 1, 1, 1, 1]),
//         (BlockType::WHITE, [4, 4, 4, 4, 4, 4]),
//         (BlockType::GRASS, [2, 2, 2, 2, 3, 1]),
//     ],
// };

/// Represents a single voxel block in the world.
///
/// This is a lightweight structure that stores only the essential block data.
/// The actual block properties are looked up from the block type.
///
/// # Memory Layout
/// The `#[repr(C)]` attribute ensures a consistent memory layout for GPU interoperability.
/// The block type is stored as a compact `BlockTypeSize` for memory efficiency.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct Block {
    /// The type of this block, encoded as a `BlockTypeSize` for compact storage.
    pub block_type: BlockTypeSize,
}

impl Block {
    /// Creates a new block of the specified type.
    ///
    /// # Arguments
    /// * `block_type` - The type of block to create
    ///
    /// # Returns
    /// A new `Block` instance with the specified type.
    pub fn new(block_type: BlockType) -> Self {
        Block {
            block_type: block_type as BlockTypeSize,
        }
    }

    /// Gets the texture indices for all faces of a block given its type as an integer.
    ///
    /// This is a convenience method that looks up the texture indices from the
    /// `BLOCK_TYPE_TO_TEXTURE_INDICES` array.
    ///
    /// # Arguments
    /// * `btype_int` - The block type as a `BlockTypeSize`
    ///
    /// # Returns
    /// An array of 6 texture indices, one for each face of the block.
    pub fn get_texture_indices_from_int(btype_int: BlockTypeSize) -> [usize; 6] {
        let block_type = BlockType::get_block_type_from_int(btype_int);
        BLOCK_TYPE_TO_TEXTURE_INDICES[block_type as usize]
    }
}
