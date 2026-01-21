# Chunk Mesh Generation

## Type
System

## Parent Flow
[Chunk Loading](./chunk_loading.md)

## Purpose
Converts chunk voxel data into optimized mesh geometry using greedy meshing. This process generates efficient vertex and index buffers that are uploaded to the GPU using a bucket-based rendering system.

## Implementation Details

### Data Flow
1. **Input**: 
   - `Chunk` object containing voxel data
   - List of block sides to generate meshes for
   - Reference to the `MeshManager`

2. **Processing** (`ChunkMeshGenerationTask::process`):
   - For each block side that needs mesh generation:
     1. Check if the chunk needs meshing (may be cached)
     2. Use greedy meshing algorithm to merge coplanar faces
     3. Generate optimized vertex and index data
     4. Create buffer write commands for GPU upload

3. **Output**: 
   - `BufferWriteCommand` objects for the render thread
   - Updated mesh cache in `MeshManager`

### Key Components
- **Greedy Meshing**: Merges adjacent coplanar faces with the same texture
- **Bucket-Based Rendering**: Organizes meshes by block face direction for efficient rendering
- **Memory-Efficient Storage**: Uses bit-packed storage for air blocks
- **Indirect Drawing**: Enables efficient multi-draw calls

### Error Handling
- Invalid chunk data is skipped
- Failed mesh generation is logged and the chunk is marked for regeneration
- GPU resource allocation failures trigger cleanup and retry logic

## Dependencies
- **Chunk Data**: Source voxel information from `Chunk`
- **Mesh Manager**: Manages GPU resources and mesh state
- **Task System**: Handles background processing
- **Buffer State**: Manages GPU buffer uploads

## Performance Considerations
- Runs asynchronously on a background thread
- Uses greedy meshing to minimize vertex count
- Implements face culling to avoid generating hidden faces
- Batches draw calls using multi-draw indirect
- Uses LRU caching for chunk mesh data
- Implements efficient memory management with bucket-based allocation

## See Also
- `src/engine_state/rendering/meshing/mod.rs` - Main mesh management
- `src/engine_state/rendering/tasks/chunk_mesh_generation_task.rs` - Task implementation
- `src/engine_state/rendering/meshing/mesh/` - Core mesh generation algorithms
- `src/engine_state/rendering/meshing/bucket_manager.rs` - Bucket-based rendering
