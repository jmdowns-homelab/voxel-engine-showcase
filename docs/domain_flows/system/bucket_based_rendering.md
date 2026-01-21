# Bucket-Based Rendering Strategy

## Purpose
This document describes the bucket-based rendering strategy used in the voxel engine to optimize draw calls and enable efficient GPU-side culling.

## Architecture Overview
The rendering system uses a combination of the following components:
- **MeshBucketManager**: Manages allocation of fixed-size memory buckets
- **ChunkIndexState**: Tracks chunk positions and their GPU buffer indices
- **Indirect Buffers**: Store draw commands that can be executed by the GPU
- **Multi-Draw Indirect**: Enables drawing multiple objects with a single GPU command

## Bucket Allocation Flow

### 1. Initialization
- The system initializes with a fixed number of buckets per block side
- Each bucket has a fixed size (1024 vertices, 1536 indices)
- Buckets are organized by block side (FRONT, BACK, LEFT, RIGHT, TOP, BOTTOM)

### 2. Mesh Generation
1. When a chunk needs to be meshed:
   - The system checks if there are enough available buckets for the mesh data
   - If not, it unloads the least recently used chunks to free up buckets
   - The mesh is generated using greedy meshing

2. For each side of the mesh:
   - The mesh data is split into one or more buckets
   - Each bucket gets a portion of the vertices and indices
   - Indices are adjusted to be relative to the bucket's vertex offset

### 3. GPU Data Upload
1. For each bucket:
   - Vertex data is uploaded to the appropriate position in the vertex buffer
   - Index data is uploaded to the appropriate position in the index buffer
   - An indirect draw command is prepared and uploaded to the indirect buffer

2. The indirect draw command contains:
   - `index_count`: Number of indices to draw
   - `instance_count`: 1 (or 0 if culled)
   - `first_index`: Starting position in the index buffer
   - `base_vertex`: Starting position in the vertex buffer
   - `first_instance`: Always 0

## GPU-Side Culling
1. The vertex shader can cull entire buckets by setting `gl_InstanceIndex` to 0
2. The indirect buffer is updated to set `instance_count` to 0 for culled buckets
3. This allows the GPU to skip rendering of culled buckets without CPU intervention

## Performance Benefits
1. **Reduced CPU Overhead**: Fewer draw calls are needed
2. **Efficient Culling**: Entire buckets can be culled with minimal state changes
3. **Memory Efficiency**: Fixed-size buckets reduce memory fragmentation
4. **Better Cache Utilization**: Related geometry is stored contiguously in memory

## Example: Drawing a Frame
1. The renderer binds the vertex, index, and indirect buffers
2. It issues a single `multi_draw_indirect` call
3. The GPU processes each draw command in the indirect buffer
4. For each command:
   - If `instance_count` is 0, the bucket is skipped
   - Otherwise, the bucket is rendered with the specified parameters

## Related Components
- `MeshManager`: Coordinates the meshing and bucket allocation process
- `ChunkIndexState`: Maps chunk positions to GPU buffer indices
- `BufferState`: Manages GPU buffer resources
