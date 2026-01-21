//! # Chunk Generation Task
//!
//! This module defines the `ChunkGenerationTask` which handles asynchronous
//! generation of chunk data. This task is typically scheduled when new chunks
//! need to be generated as the player moves through the world.

use cgmath::Point3;

use crate::{
    core::MtResource,
    engine_state::{
        buffer_state::BufferWriteCommand,
        rendering::{tasks::chunk_mesh_generation_task::ChunkMeshGenerationTask, MeshManager},
        task_management::task::{Task, TaskResult},
        voxels::{block::block_side::BlockSide, chunk::Chunk, world::World},
    },
};

use crate::core::injection_system::{MtInjectionSystem, StInjectionSystem};

/// A task that generates chunk data asynchronously.
///
/// This task is responsible for:
/// 1. Generating the chunk data at the specified position
/// 2. Adding the chunk to the world
/// 3. Scheduling mesh generation for the chunk
pub struct ChunkGenerationTask {
    /// A thread-safe reference to the world where the chunk will be added
    world: MtResource<World>,
    /// The position of the chunk to generate (in chunk coordinates)
    position: Point3<i32>,
}

impl ChunkGenerationTask {
    /// Creates a new chunk generation task.
    ///
    /// # Arguments
    /// * `world` - A thread-safe reference to the world
    /// * `position` - The chunk coordinates where the chunk should be generated
    ///
    /// # Returns
    /// A new `ChunkGenerationTask` instance
    pub fn new(world: MtResource<World>, position: Point3<i32>) -> Self {
        ChunkGenerationTask { world, position }
    }
}

impl Task for ChunkGenerationTask {
    /// Executes the chunk generation task.
    ///
    /// This method is called by the task scheduler to perform the actual work.
    /// It generates the chunk and returns a result that will be processed on the main thread.
    ///
    /// # Returns
    /// A boxed `TaskResult` containing the generated chunk
    fn process(&self) -> Box<dyn TaskResult + Send> {
        // Generate the chunk data and add it to the world
        self.world.get_mut().add_chunk_at(self.position);

        // Return a result containing the generated chunk
        Box::new(ChunkGenerationTaskResult {
            chunk: self.world.get().get_chunk_at(self.position).unwrap(),
        })
    }
}

/// The result of a chunk generation task.
///
/// This contains the generated chunk data and is responsible for scheduling
/// any follow-up tasks (like mesh generation) that depend on the chunk data.
pub struct ChunkGenerationTaskResult {
    /// A thread-safe reference to the generated chunk
    chunk: MtResource<Chunk>,
}

impl TaskResult for ChunkGenerationTaskResult {
    /// Handles the result of chunk generation on the main thread.
    ///
    /// This method is called on the main thread after the chunk data has been generated.
    /// It schedules mesh generation for the chunk.
    ///
    /// # Arguments
    /// * `mt_injection_system` - The multi-threaded dependency injection system
    /// * `_st_injection_system` - The single-threaded dependency injection system (unused)
    ///
    /// # Returns
    /// A tuple containing:
    /// - A vector of follow-up tasks to schedule (mesh generation)
    /// - A vector of buffer write commands (empty in this case)
    fn handle_result(
        self: Box<Self>,
        mt_injection_system: &MtInjectionSystem,
        _st_injection_system: &StInjectionSystem,
    ) -> (Vec<Box<dyn Task>>, Vec<BufferWriteCommand>) {
        let mut tasks = Vec::new();

        // Schedule mesh generation for the chunk
        let mesh_generation_task: Box<dyn Task> = Box::new(ChunkMeshGenerationTask::new(
            mt_injection_system.get::<MeshManager>().unwrap(),
            self.chunk.clone(),
            vec![
                BlockSide::FRONT,
                BlockSide::BACK,
                BlockSide::LEFT,
                BlockSide::RIGHT,
                BlockSide::TOP,
                BlockSide::BOTTOM,
            ],
        ));

        tasks.push(mesh_generation_task);

        (tasks, Vec::new())
    }
}
