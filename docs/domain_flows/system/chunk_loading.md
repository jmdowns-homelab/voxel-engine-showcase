# Chunk Loading and Mesh Generation

## Type
System

## Purpose
Manages the lifecycle of chunks in the voxel world, from creation to rendering. This includes generating chunk data, creating optimized mesh geometry using greedy meshing, and managing GPU resources efficiently.

## Trigger
- Player moves into a new chunk area (detected by `World` system)
- New chunks come into the player's view distance
- Chunks are unloaded when they move out of view distance

## Systems Involved
- **World System**: Manages chunk positions and visibility
- **Chunk Manager**: Handles chunk lifecycle and state
- **Task System**: Processes chunk generation and mesh creation asynchronously
  - `ChunkGenerationTask`: Generates chunk data
  - `ChunkMeshGenerationTask`: Generates mesh data
- **Mesh Manager**: Manages GPU resources and mesh data using a bucket-based approach
- **Render System**: Handles the actual rendering of chunk meshes using indirect drawing

## Flow
1. **Chunk Loading Initiation**
   - `World` system detects player movement into new chunk coordinates
   - New chunks are identified based on view distance
   - `ChunkGenerationTask` is scheduled for new chunk positions

2. **Chunk Data Generation** (`ChunkGenerationTask`)
   - Creates a new `Chunk` at the specified position
   - Generates terrain data using noise functions
   - Adds the chunk to the world
   - Schedules `ChunkMeshGenerationTask` for the new chunk

3. **Mesh Generation** (`ChunkMeshGenerationTask`)
   - For each chunk needing a mesh:
     1. `MeshManager` checks if the chunk needs meshing
     2. Greedy meshing algorithm generates optimized mesh data
     3. Mesh data is prepared for GPU upload
     4. Buffer write commands are created for the render thread

4. **Rendering**
   - `MeshManager` organizes meshes into buckets by block face direction
   - Uses multi-draw indirect for efficient rendering
   - Implements frustum culling at the bucket level
   - Issues draw calls for visible chunks

5. **Chunk Unloading**
   - Chunks outside view distance are marked for unloading
   - GPU resources are released through `BufferWriteCommand`s
   - Chunk data is removed from the world

## Related Flows
- [Chunk Mesh Generation](./chunk_mesh_generation_subflow.md)
- [Chunk Data Generation](./chunk_data_generation_subflow.md)
- [Chunk Memory Management](./chunk_memory_management_subflow.md)

## Performance Considerations
- Uses a task-based system for parallel chunk generation and meshing
- Implements greedy meshing to minimize vertex count
- Employs a bucket-based rendering system to reduce draw calls
- Uses indirect drawing for efficient GPU utilization
- Implements LRU caching for chunk data
- Memory-efficient storage with bit-packed air block representation

## See Also
- `src/engine_state/voxels/chunk/` - Core chunk implementation
- `src/engine_state/rendering/meshing/` - Mesh generation and management
- `src/engine_state/rendering/tasks/` - Background tasks for chunk processing
- `src/engine_state/voxels/tasks/` - Chunk generation tasks
