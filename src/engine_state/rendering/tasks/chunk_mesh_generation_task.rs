//! Task for generating mesh data for chunks in a background thread.
//!
//! This module contains the `ChunkMeshGenerationTask` which is responsible for
//! generating vertex and index data for chunks in a background thread. This helps
//! keep the main thread responsive while complex mesh generation is performed.

use crate::{
    core::{MtResource, MtSystem},
    engine_state::{
        buffer_state::BufferWriteCommand,
        rendering::meshing::MeshManager,
        task_management::task::{Task, TaskResult},
        voxels::{block::block_side::BlockSide, chunk::Chunk},
    },
};

use crate::core::injection_system::{MtInjectionSystem, StInjectionSystem};

/// A task that generates mesh data for a chunk in a background thread.
///
/// This task is responsible for:
/// 1. Checking if the chunk needs mesh generation
/// 2. Generating vertex and index data for the specified chunk sides
/// 3. Creating buffer write commands to upload the generated data to the GPU
pub struct ChunkMeshGenerationTask {
    /// Thread-safe reference to the mesh manager
    mesh_manager: MtSystem<MeshManager>,
    /// The chunk that needs mesh generation
    chunk: MtResource<Chunk>,
    /// Which block sides should have their meshes generated
    sides_to_generate: Vec<BlockSide>,
}

impl ChunkMeshGenerationTask {
    /// Creates a new chunk mesh generation task.
    ///
    /// # Arguments
    /// * `mesh_manager` - Thread-safe reference to the mesh manager
    /// * `chunk` - The chunk that needs mesh generation
    /// * `sides_to_generate` - List of block sides to generate meshes for
    ///
    /// # Returns
    /// A new `ChunkMeshGenerationTask` instance
    pub fn new(
        mesh_manager: MtSystem<MeshManager>,
        chunk: MtResource<Chunk>,
        sides_to_generate: Vec<BlockSide>,
    ) -> Self {
        ChunkMeshGenerationTask {
            mesh_manager,
            sides_to_generate,
            chunk,
        }
    }
}

impl Task for ChunkMeshGenerationTask {
    /// Processes the mesh generation task.
    ///
    /// This method is called in a background thread and performs the actual
    /// mesh generation. It checks if the chunk needs meshing and then generates
    /// the appropriate vertex and index data.
    ///
    /// # Returns
    /// A boxed `TaskResult` containing the buffer write commands needed to
    /// upload the generated mesh data to the GPU
    fn process(&self) -> Box<dyn TaskResult + Send> {
        if self
            .mesh_manager
            .get_mut()
            .is_chunk_meshed(self.chunk.get().position)
        {
            return Box::new(ChunkMeshGenerationTaskResult {
                write_commands: Vec::new(),
            });
        }

        let write_commands = self
            .mesh_manager
            .get_mut()
            .generate_mesh_for_chunk(self.chunk.clone(), &self.sides_to_generate);

        Box::new(ChunkMeshGenerationTaskResult { write_commands })
    }
}

/// The result of a chunk mesh generation task.
///
/// This struct contains the buffer write commands needed to upload the
/// generated mesh data to the GPU.
pub struct ChunkMeshGenerationTaskResult {
    /// List of buffer write commands to execute on the main thread
    write_commands: Vec<BufferWriteCommand>,
}

impl TaskResult for ChunkMeshGenerationTaskResult {
    /// Handles the result of the mesh generation task.
    ///
    /// This method is called on the main thread after the background task
    /// completes. It returns the buffer write commands that need to be
    /// executed to upload the generated mesh data to the GPU.
    ///
    /// # Arguments
    /// * `_mt_injection_system` - Multi-threaded dependency injection system (unused)
    /// * `_st_injection_system` - Single-threaded dependency injection system (unused)
    ///
    /// # Returns
    /// A tuple containing:
    /// 1. An empty vector (no follow-up tasks)
    /// 2. The list of buffer write commands to execute
    fn handle_result(
        self: Box<Self>,
        _mt_injection_system: &MtInjectionSystem,
        _st_injection_system: &StInjectionSystem,
    ) -> (Vec<Box<dyn Task>>, Vec<BufferWriteCommand>) {
        (Vec::new(), self.write_commands)
    }
}
