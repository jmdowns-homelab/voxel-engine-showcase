//! # Block Side Module
//!
//! This module defines the different faces/sides of a voxel block.
//! It provides functionality for face culling and visibility determination.

use cgmath::Vector3;

/// Represents the six possible faces of a voxel block.
/// 
/// Each variant corresponds to a specific face and is assigned a unique integer value
/// for efficient storage and serialization. The values match the order expected
/// by the rendering system.
/// 
/// The order is: [FRONT, BACK, BOTTOM, TOP, LEFT, RIGHT]
#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum BlockSide {
    /// The front face (facing positive Z)
    FRONT = 0,
    
    /// The back face (facing negative Z)
    BACK = 1,
    
    /// The bottom face (facing negative Y)
    BOTTOM = 2,
    
    /// The top face (facing positive Y)
    TOP = 3,
    
    /// The left face (facing negative X)
    LEFT = 4,
    
    /// The right face (facing positive X)
    RIGHT = 5,
}

impl BlockSide {
    /// Returns an array containing all six block faces in a consistent order.
    /// 
    /// This is useful for iterating over all possible faces of a block.
    /// The order is: [FRONT, BACK, BOTTOM, TOP, LEFT, RIGHT]
    /// 
    /// # Returns
    /// An array containing all `BlockSide` variants.
    pub fn all() -> [BlockSide; 6] {
        [
            BlockSide::FRONT,
            BlockSide::BACK,
            BlockSide::BOTTOM,
            BlockSide::TOP,
            BlockSide::LEFT,
            BlockSide::RIGHT,
        ]
    }

    /// Determines which block faces are potentially visible from a given view direction.
    /// 
    /// This is used for face culling optimization in the rendering pipeline.
    /// A face is considered potentially visible if the view direction is within
    /// 90 degrees of the face's normal.
    /// 
    /// # Arguments
    /// * `view_vec` - The normalized view direction vector
    /// 
    /// # Returns
    /// A vector containing all potentially visible block faces, ordered by likelihood
    /// of being visible (most likely first).
    pub fn get_visible_sides(view_vec: Vector3<f32>) -> Vec<BlockSide> {
        // The cutoff is 1/√2, which is the cosine of 45 degrees.
        // This provides a good balance between culling efficiency and simplicity.
        const CUTOFF: f32 = std::f32::consts::FRAC_1_SQRT_2;
        let mut visible_sides = Vec::new();

        // Check each axis and add the appropriate faces based on view direction
        if view_vec.x > -CUTOFF {
            visible_sides.push(BlockSide::FRONT);
        }
        if view_vec.x < CUTOFF {
            visible_sides.push(BlockSide::BACK);
        }
        if view_vec.y > -CUTOFF {
            visible_sides.push(BlockSide::BOTTOM);
        }
        if view_vec.y < CUTOFF {
            visible_sides.push(BlockSide::TOP);
        }
        if view_vec.z > -CUTOFF {
            visible_sides.push(BlockSide::LEFT);
        }
        if view_vec.z < CUTOFF {
            visible_sides.push(BlockSide::RIGHT);
        }

        visible_sides
    }
}
