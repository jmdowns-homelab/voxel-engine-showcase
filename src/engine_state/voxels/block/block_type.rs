//! # Block Type Module
//!
//! This module defines the different types of blocks in the voxel world.
//! It provides functionality for block type identification, conversion, and random generation.

use std::{
    fmt,
    hash::{Hash, Hasher},
};

use num_derive::FromPrimitive;

use super::BlockTypeSize;

/// Enumerates all possible block types in the voxel world.
///
/// Each variant represents a distinct type of block with its own properties
/// and behavior. The `FromPrimitive` derive allows conversion from integers,
/// which is useful for serialization and deserialization.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, FromPrimitive)]
pub enum BlockType {
    /// An air block, which is non-solid and transparent.
    AIR,
    
    /// A basic dirt block, used as a common building material.
    DIRT,
    
    /// A grass block with different textures on top and sides.
    /// The top is green, sides have grass on dirt, and bottom is plain dirt.
    GRASS,
    
    /// A wooden block with a bark texture on all sides.
    WOOD,
    
    /// A plain white block, often used for testing and UI elements.
    WHITE,
}

impl BlockType {
    /// Converts a `BlockTypeSize` to a `BlockType`.
    ///
    /// This is typically used when deserializing block data or converting
    /// from the compact storage format to the rich enum type.
    ///
    /// # Arguments
    /// * `btype` - The block type as a `BlockTypeSize`
    ///
    /// # Returns
    /// The corresponding `BlockType`
    ///
    /// # Panics
    /// Panics if the input value doesn't correspond to a valid `BlockType`.
    pub fn get_block_type_from_int(btype: BlockTypeSize) -> Self {
        let btype_option = num::FromPrimitive::from_u8(btype as BlockTypeSize);
        btype_option.unwrap()
    }
    
    /// Generates a random block type (excluding AIR).
    ///
    /// This is primarily used for testing and procedural generation.
    ///
    /// # Returns
    /// A random `BlockType` that is not `BlockType::AIR`
    pub fn get_random_type() -> Self {
        num::FromPrimitive::from_u8(fastrand::u8(1..4)).unwrap()
    }
}
// Implementation of PHF (Perfect Hash Function) traits for BlockType.
// These are used internally by the `phf` crate for static hash maps.

/// Implements `FmtConst` to allow formatting `BlockType` in const contexts.
/// This is used by the `phf` crate for compile-time map generation.
impl phf_shared::FmtConst for BlockType {
    fn fmt_const(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockType::{:?}", self)
    }
}

/// Implements `PhfHash` to provide a custom hashing strategy for `BlockType`.
/// This ensures that the hash matches the underlying integer representation.
impl phf_shared::PhfHash for BlockType {
    #[inline]
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        (*self as BlockTypeSize).hash(state);
    }
}

/// Implements `PhfBorrow` to allow using `BlockType` as a key in PHF maps.
/// This enables efficient lookups in compile-time generated maps.
impl phf_shared::PhfBorrow<BlockType> for BlockType {
    fn borrow(&self) -> &BlockType {
        self
    }
}
