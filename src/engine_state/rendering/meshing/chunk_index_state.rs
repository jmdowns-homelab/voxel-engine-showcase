//! Manages chunk-to-GPU index mapping for efficient rendering.
//!
//! This module handles the mapping between chunk positions and their corresponding
//! indices in GPU buffers. It's used to efficiently manage chunk visibility and
//! culling during rendering.


use std::collections::{HashMap, VecDeque};

use cgmath::Point3;

use crate::{
    core::StSystem,
    engine_state::{
        buffer_state::{BufferState, BufferWriteCommand},
        RENDER_DISTANCE,
    },
};

/// Name of the chunk index buffer used for indirect rendering
pub const CHUNK_INDEX_BUFFER_NAME: &str = "chunk_index_buffer";

/// Manages the mapping between chunk positions and GPU buffer indices.
///
/// This structure maintains the state needed to track which chunks are currently
/// loaded into GPU memory and their corresponding indices in the rendering pipeline.
pub struct ChunkIndexState {
    chunk_position_to_gpu_index: HashMap<Point3<i32>, u32>,
    available_chunk_indices: VecDeque<u32>,
}

const WORLD_DIMENSION: usize =
    (RENDER_DISTANCE * 2 + 1) * (RENDER_DISTANCE * 2 + 1) * (RENDER_DISTANCE * 2 + 1) * 2;

impl ChunkIndexState {
    pub fn new(buffer_state: StSystem<BufferState>) -> Self {
        buffer_state.get_mut().create_buffer(
            CHUNK_INDEX_BUFFER_NAME,
            wgpu::BufferDescriptor {
                label: Some(CHUNK_INDEX_BUFFER_NAME),
                size: (WORLD_DIMENSION * (3 * std::mem::size_of::<i32>())) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );

        let mut available_chunk_indices = VecDeque::new();

        for i in 0..WORLD_DIMENSION as u32 {
            available_chunk_indices.push_back(i);
        }

        Self {
            chunk_position_to_gpu_index: HashMap::new(),
            available_chunk_indices,
        }
    }

    pub fn unload_chunk_positions(&mut self, chunk_positions: &Vec<Point3<i32>>) {
        for pos in chunk_positions.iter() {
            if let Some(available_index) = self.chunk_position_to_gpu_index.remove(pos) {
                self.available_chunk_indices.push_back(available_index);
            }
        }
    }

    pub fn load_chunk_positions(
        &mut self,
        chunk_positions: &Vec<Point3<i32>>,
    ) -> Vec<BufferWriteCommand> {
        let mut commands = Vec::new();

        for pos in chunk_positions.iter() {
            let index = self.available_chunk_indices.pop_front().unwrap();

            self.chunk_position_to_gpu_index.insert(*pos, index);

            commands.push(BufferWriteCommand {
                name: format!("Chunk Position {:?} - Index {}", pos, index),
                buffer_name: CHUNK_INDEX_BUFFER_NAME,
                offset: index as u64 * 3 * std::mem::size_of::<i32>() as u64,
                data: Box::new([pos.x, pos.y, pos.z]),
            });
        }

        commands
    }

    pub fn can_allocate_index(&self) -> bool {
        !self.available_chunk_indices.is_empty()
    }

    pub fn get_index_for_position(&mut self, chunk_position: Point3<i32>) -> u32 {
        *self
            .chunk_position_to_gpu_index
            .get(&chunk_position)
            .unwrap()
    }
}
