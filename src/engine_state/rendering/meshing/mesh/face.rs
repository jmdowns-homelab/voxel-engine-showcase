use cgmath::Point3;

use crate::engine_state::voxels::block::block_side::BlockSide;

/// Represents a single quad face of a voxel in the mesh.
///
/// A face is defined by four corner points (lower-left, lower-right, upper-right, upper-left)
/// and contains information about the block type and which side of the block it represents.
/// This is used by the greedy meshing algorithm to combine adjacent coplanar faces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Face {
    /// Lower-right corner of the face in chunk coordinates
    pub lr: Point3<usize>,
    /// Lower-left corner of the face in chunk coordinates
    pub ll: Point3<usize>,
    /// Upper-right corner of the face in chunk coordinates
    pub ur: Point3<usize>,
    /// Upper-left corner of the face in chunk coordinates
    pub ul: Point3<usize>,
    /// The block type as an integer, used for texture mapping
    pub block_type_int: usize,
    /// Which side of the block this face represents
    pub block_side: BlockSide,
}

impl Face {
    /// Creates a new face for a voxel at the given coordinates.
    ///
    /// # Arguments
    /// * `i`, `j`, `k` - The coordinates of the voxel in chunk space
    /// * `block_type_int` - The type of the block, used for texture mapping
    /// * `block_side` - Which side of the block this face represents
    ///
    /// # Returns
    /// A new `Face` instance with the specified properties and properly calculated vertices
    /// based on the block side.
    pub fn new(i: usize, j: usize, k: usize, block_type_int: usize, block_side: BlockSide) -> Self {
        match block_side {
            BlockSide::FRONT => Face {
                ll: Point3::new(i, j, k),
                lr: Point3::new(i, j, k + 1),
                ul: Point3::new(i, j + 1, k),
                ur: Point3::new(i, j + 1, k + 1),
                block_type_int,
                block_side,
            },

            BlockSide::BACK => Face {
                ll: Point3::new(i + 1, j, k + 1),
                lr: Point3::new(i + 1, j, k),
                ul: Point3::new(i + 1, j + 1, k + 1),
                ur: Point3::new(i + 1, j + 1, k),
                block_type_int,
                block_side,
            },

            BlockSide::BOTTOM => Face {
                ll: Point3::new(i, j, k + 1),
                lr: Point3::new(i, j, k),
                ul: Point3::new(i + 1, j, k + 1),
                ur: Point3::new(i + 1, j, k),
                block_type_int,
                block_side,
            },

            BlockSide::TOP => Face {
                ll: Point3::new(i, j + 1, k),
                lr: Point3::new(i, j + 1, k + 1),
                ul: Point3::new(i + 1, j + 1, k),
                ur: Point3::new(i + 1, j + 1, k + 1),
                block_type_int,
                block_side,
            },

            BlockSide::LEFT => Face {
                ll: Point3::new(i + 1, j, k),
                lr: Point3::new(i, j, k),
                ul: Point3::new(i + 1, j + 1, k),
                ur: Point3::new(i, j + 1, k),
                block_type_int,
                block_side,
            },

            BlockSide::RIGHT => Face {
                ll: Point3::new(i, j, k + 1),
                lr: Point3::new(i + 1, j, k + 1),
                ul: Point3::new(i, j + 1, k + 1),
                ur: Point3::new(i + 1, j + 1, k + 1),
                block_type_int,
                block_side,
            },
        }
    }

    /// Attempts to merge this face with another face that is directly above it.
    ///
    /// # Arguments
    /// * `other` - The face to merge with (should be directly above this face)
    ///
    /// # Returns
    /// `Some(merged_face)` if the faces can be merged, or `None` if they cannot be merged.
    ///
    /// # Note
    /// Faces can only be merged if they have the same block type and their edges align perfectly.
    pub fn merge_up(&self, other: &Face) -> Option<Face> {
        if self.block_type_int == other.block_type_int && self.ul == other.ll && self.ur == other.lr
        {
            return Some(Face {
                ul: other.ul,
                ur: other.ur,
                ll: self.ll,
                lr: self.lr,
                block_side: self.block_side,
                block_type_int: self.block_type_int,
            });
        }

        None
    }

    /// Attempts to merge this face with another face that is directly to its right.
    ///
    /// # Arguments
    /// * `other` - The face to merge with (should be directly to the right of this face)
    ///
    /// # Returns
    /// `Some(merged_face)` if the faces can be merged, or `None` if they cannot be merged.
    ///
    /// # Note
    /// Faces can only be merged if they have the same block type and their edges align perfectly.
    pub fn merge_right(&self, other: &Face) -> Option<Face> {
        if self.block_type_int == other.block_type_int && self.lr == other.ll && self.ur == other.ul
        {
            return Some(Face {
                ul: self.ul,
                ur: other.ur,
                ll: self.ll,
                lr: other.lr,
                block_side: self.block_side,
                block_type_int: self.block_type_int,
            });
        }

        None
    }

    /// Attempts to merge this face with another face that is directly to its left.
    ///
    /// # Arguments
    /// * `other` - The face to merge with (should be directly to the left of this face)
    ///
    /// # Returns
    /// `Some(merged_face)` if the faces can be merged, or `None` if they cannot be merged.
    ///
    /// # Note
    /// Faces can only be merged if they have the same block type and their edges align perfectly.
    pub fn merge_left(&self, other: &Face) -> Option<Face> {
        if self.block_type_int == other.block_type_int && self.ll == other.lr && self.ul == other.ur
        {
            return Some(Face {
                ul: other.ul,
                ur: self.ur,
                ll: other.ll,
                lr: self.lr,
                block_side: self.block_side,
                block_type_int: self.block_type_int,
            });
        }

        None
    }
}
