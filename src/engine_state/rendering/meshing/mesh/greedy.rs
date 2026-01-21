//! Greedy meshing implementation for voxel rendering.
//!
//! This module implements the greedy meshing algorithm which combines adjacent coplanar
//! faces with the same texture into larger quads, significantly reducing the number of
//! vertices and draw calls needed to render a voxel world.

use log::info;
use web_time::Instant;

use crate::engine_state::voxels::{
    block::block_side::BlockSide,
    chunk::{chunk_iteration::ChunkBlockIterator, Chunk, CHUNK_DIMENSION},
};

use super::{face::Face, mesh::Mesh};

/// Gets the appropriate boundary coordinate from a face based on the merge direction.
///
/// # Arguments
/// * `face` - The face to get the boundary from
/// * `is_y_merging` - Whether we're merging in the Y direction (true) or X/Z direction (false)
///
/// # Returns
/// The appropriate coordinate value based on the face orientation and merge direction.
fn get_boundary_from_face(face: &Face, is_y_merging: bool) -> usize {
    if is_y_merging {
        match face.block_side {
            BlockSide::FRONT => face.ul.y,
            BlockSide::BACK => face.ul.y,
            BlockSide::LEFT => face.ul.x,
            BlockSide::RIGHT => face.ur.x,
            BlockSide::TOP => face.ur.x,
            BlockSide::BOTTOM => face.ur.x,
        }
    } else {
        match face.block_side {
            BlockSide::FRONT => face.ul.z,
            BlockSide::BACK => face.ul.z,
            BlockSide::LEFT => face.ul.z,
            BlockSide::RIGHT => face.ul.z,
            BlockSide::TOP => face.ul.x,
            BlockSide::BOTTOM => face.ul.x,
        }
    }
}

/// Performs the core greedy meshing algorithm by merging faces in the current layer
/// with those in the previous layer where possible.
///
/// # Arguments
/// * `current_layer` - The current layer of faces being processed
/// * `before_layer` - The previous layer of faces to merge with
/// * `faces_to_make` - Output vector for the resulting merged faces
/// * `side` - The block side being processed
///
/// # Note
/// This function modifies the input vectors in-place for efficiency.
fn greedy_merge_and_modify_vecs(
    current_layer: &mut Vec<Vec<Face>>,
    before_layer: &mut Vec<Vec<Face>>,
    faces_to_make: &mut Vec<Face>,
    side: BlockSide,
) {
    for layer_index in 0..CHUNK_DIMENSION as usize {
        match (
            before_layer[layer_index].len(),
            current_layer[layer_index].len(),
        ) {
            (0, _) => {
                before_layer[layer_index].append(&mut current_layer[layer_index]);
            }
            (_, 0) => {
                for before_face in before_layer[layer_index].drain(..) {
                    faces_to_make.push(before_face);
                }
            }
            (before_len, current_len) => {
                let mut before_index = 0;
                let mut current_index = 0;

                while before_index < before_len && current_index < current_len {
                    let before_face = before_layer[layer_index][before_index];
                    let current_face = current_layer[layer_index][current_index];
                    let merged_face_option = match side {
                        BlockSide::FRONT => before_face.merge_right(&current_face),
                        BlockSide::BACK => before_face.merge_left(&current_face),
                        BlockSide::LEFT => before_face.merge_up(&current_face),
                        BlockSide::RIGHT => before_face.merge_up(&current_face),
                        BlockSide::TOP => before_face.merge_right(&current_face),
                        BlockSide::BOTTOM => before_face.merge_left(&current_face),
                    };
                    if let Some(merged_face) = merged_face_option {
                        current_layer[layer_index][current_index] = merged_face;
                        before_index += 1;
                        current_index += 1;
                    } else {
                        let (before_boundary, current_boundary) = (
                            get_boundary_from_face(&before_face, true),
                            get_boundary_from_face(&current_face, true),
                        );
                        if before_boundary == current_boundary {
                            faces_to_make.push(before_layer[layer_index][before_index]);
                            before_index += 1;
                            current_index += 1;
                        } else if before_boundary < current_boundary {
                            while before_index < before_len
                                && get_boundary_from_face(
                                    &before_layer[layer_index][before_index],
                                    true,
                                ) < current_boundary
                            {
                                faces_to_make.push(before_layer[layer_index][before_index]);
                                before_index += 1;
                            }
                        } else if before_boundary > current_boundary {
                            while current_index < current_len
                                && get_boundary_from_face(
                                    &current_layer[layer_index][current_index],
                                    true,
                                ) < before_boundary
                            {
                                current_index += 1;
                            }
                            if current_index == current_len {
                                faces_to_make.push(before_layer[layer_index][before_index]);
                                before_index += 1;
                            }
                        }
                    }
                }

                for i in before_index..before_len {
                    faces_to_make.push(before_layer[layer_index][i]);
                }

                before_layer[layer_index].clear();
                before_layer[layer_index].append(&mut current_layer[layer_index]);
            }
        }
    }
}

// /// Generates a mesh for the entire chunk using greedy meshing on all sides.
// ///
// /// # Arguments
// /// * `chunk` - The chunk to generate the mesh for
// /// * `index` - The index of the chunk in the world
// ///
// /// # Returns
// /// A new `Mesh` containing the greedy-meshed geometry for all sides of the chunk.
// pub fn greedy(chunk: &Chunk, index: u32) -> Mesh {
//     greedy_sided(chunk, index, &vec![
//         BlockSide::FRONT,
//         BlockSide::BACK,
//         BlockSide::LEFT,
//         BlockSide::RIGHT,
//         BlockSide::TOP,
//         BlockSide::BOTTOM
//     ])
// }

/// Generates a mesh for the specified sides of a chunk using greedy meshing.
///
/// # Arguments
/// * `chunk` - The chunk to generate the mesh for
/// * `index` - The index of the chunk in the world
/// * `sides` - A list of block sides to generate mesh data for
///
/// # Returns
/// A new `Mesh` containing the greedy-meshed geometry for the specified sides.
///
/// # Performance
/// The greedy meshing algorithm runs in O(n) time where n is the number of voxels in the chunk.
/// It significantly reduces the number of vertices compared to naive meshing by combining
/// adjacent coplanar faces with the same texture.
pub fn greedy_sided(chunk: &Chunk, index: u32, sides: &Vec<BlockSide>) -> Mesh {
    let mut mesh = Mesh::new();
    let mut cbi = ChunkBlockIterator::new(chunk);

    let mut side_layers = vec![vec![Vec::new(); CHUNK_DIMENSION as usize]; 6];
    let mut side_before_layers = vec![vec![Vec::new(); CHUNK_DIMENSION as usize]; 6];

    let mut faces_to_make = Vec::new();

    let mut current_x;
    let mut current_y = 0;
    let mut current_z = 0;

    let contains_front = sides.contains(&BlockSide::FRONT);
    let contains_back = sides.contains(&BlockSide::BACK);
    let contains_left = sides.contains(&BlockSide::LEFT);
    let contains_right = sides.contains(&BlockSide::RIGHT);
    let contains_top = sides.contains(&BlockSide::TOP);
    let contains_bottom = sides.contains(&BlockSide::BOTTOM);

    while let Some((position, block)) = cbi.get_next_block() {
        let i = position.x;
        let j = position.y;
        let k = position.z;

        if current_y < j {
            if contains_left {
                greedy_merge_and_modify_vecs(
                    &mut side_layers[BlockSide::LEFT as usize],
                    &mut side_before_layers[BlockSide::LEFT as usize],
                    &mut faces_to_make,
                    BlockSide::LEFT,
                );
            }
            if contains_right {
                greedy_merge_and_modify_vecs(
                    &mut side_layers[BlockSide::RIGHT as usize],
                    &mut side_before_layers[BlockSide::RIGHT as usize],
                    &mut faces_to_make,
                    BlockSide::RIGHT,
                );
            }
        }
        if current_z < k {
            if contains_front {
                greedy_merge_and_modify_vecs(
                    &mut side_layers[BlockSide::FRONT as usize],
                    &mut side_before_layers[BlockSide::FRONT as usize],
                    &mut faces_to_make,
                    BlockSide::FRONT,
                );
            }
            if contains_back {
                greedy_merge_and_modify_vecs(
                    &mut side_layers[BlockSide::BACK as usize],
                    &mut side_before_layers[BlockSide::BACK as usize],
                    &mut faces_to_make,
                    BlockSide::BACK,
                );
            }
            if contains_top {
                greedy_merge_and_modify_vecs(
                    &mut side_layers[BlockSide::TOP as usize],
                    &mut side_before_layers[BlockSide::TOP as usize],
                    &mut faces_to_make,
                    BlockSide::TOP,
                );
            }
            if contains_bottom {
                greedy_merge_and_modify_vecs(
                    &mut side_layers[BlockSide::BOTTOM as usize],
                    &mut side_before_layers[BlockSide::BOTTOM as usize],
                    &mut faces_to_make,
                    BlockSide::BOTTOM,
                );
            }
        }
        current_x = i;
        current_y = j;
        current_z = k;
        let adjacent_blocks_data = Chunk::generate_adjacent_blocks(chunk, i, j, k);

        for side in sides.iter() {
            if !adjacent_blocks_data[*side as usize] {
                let side_face = Face::new(
                    current_x,
                    current_y,
                    current_z,
                    block.block_type as usize,
                    *side,
                );
                let orientation_index = match *side {
                    BlockSide::FRONT => current_x,
                    BlockSide::BACK => current_x,
                    BlockSide::LEFT => current_z,
                    BlockSide::RIGHT => current_z,
                    BlockSide::TOP => current_y,
                    BlockSide::BOTTOM => current_y,
                };
                match side_layers[*side as usize][orientation_index].last() {
                    Some(face) => {
                        let merged_option = match *side {
                            BlockSide::FRONT => face.merge_up(&side_face),
                            BlockSide::BACK => face.merge_up(&side_face),
                            BlockSide::LEFT => face.merge_left(&side_face),
                            BlockSide::RIGHT => face.merge_right(&side_face),
                            BlockSide::TOP => face.merge_up(&side_face),
                            BlockSide::BOTTOM => face.merge_up(&side_face),
                        };
                        if let Some(merged_face) = merged_option {
                            side_layers[*side as usize][orientation_index].pop();
                            side_layers[*side as usize][orientation_index].push(merged_face);
                        } else {
                            side_layers[*side as usize][orientation_index].push(side_face);
                        }
                    }
                    None => {
                        side_layers[*side as usize][orientation_index].push(side_face);
                    }
                }
            }
        }
    }

    if contains_front {
        greedy_merge_and_modify_vecs(
            &mut side_layers[BlockSide::FRONT as usize],
            &mut side_before_layers[BlockSide::FRONT as usize],
            &mut faces_to_make,
            BlockSide::FRONT,
        );
    }
    if contains_back {
        greedy_merge_and_modify_vecs(
            &mut side_layers[BlockSide::BACK as usize],
            &mut side_before_layers[BlockSide::BACK as usize],
            &mut faces_to_make,
            BlockSide::BACK,
        );
    }
    if contains_left {
        greedy_merge_and_modify_vecs(
            &mut side_layers[BlockSide::LEFT as usize],
            &mut side_before_layers[BlockSide::LEFT as usize],
            &mut faces_to_make,
            BlockSide::LEFT,
        );
    }
    if contains_right {
        greedy_merge_and_modify_vecs(
            &mut side_layers[BlockSide::RIGHT as usize],
            &mut side_before_layers[BlockSide::RIGHT as usize],
            &mut faces_to_make,
            BlockSide::RIGHT,
        );
    }
    if contains_top {
        greedy_merge_and_modify_vecs(
            &mut side_layers[BlockSide::TOP as usize],
            &mut side_before_layers[BlockSide::TOP as usize],
            &mut faces_to_make,
            BlockSide::TOP,
        );
    }
    if contains_bottom {
        greedy_merge_and_modify_vecs(
            &mut side_layers[BlockSide::BOTTOM as usize],
            &mut side_before_layers[BlockSide::BOTTOM as usize],
            &mut faces_to_make,
            BlockSide::BOTTOM,
        );
    }

    for i in 0..6 {
        for face_vec in side_before_layers[i].iter() {
            for face in face_vec {
                faces_to_make.push(*face);
            }
        }
        for face_vec in side_layers[i].iter() {
            for face in face_vec {
                faces_to_make.push(*face);
            }
        }
    }

    let mut vertex_vec = [
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ];
    let mut index_vec = [
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ];
    let mut num_faces_generated = [0; 6];

    for face in faces_to_make {
        let face_index = face.block_side as usize;
        vertex_vec[face_index].extend(Mesh::generate_face_vertices(&face, index));
        index_vec[face_index].extend(Mesh::generate_face_indices(num_faces_generated[face_index]));
        num_faces_generated[face_index] += 1;
    }

    mesh.add_vertices(vertex_vec, index_vec);

    mesh
}
