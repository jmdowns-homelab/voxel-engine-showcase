# Chunk Memory Management

## Type
System

## Parent Flow
[Chunk Loading](./chunk_loading.md)

## Purpose
Manages the efficient allocation, caching, and deallocation of chunk data in both CPU and GPU memory. This system ensures optimal memory usage while maintaining performance for chunk loading and rendering.

## Implementation Details

### Memory Architecture
1. **CPU-Side Storage**
   - Uses `ChunkCreationIterator` for efficient chunk building
   - Implements bit-packed storage for air blocks
   - Maintains spatial partitioning for neighbor access

2. **GPU Memory Management**
   - Uses `MeshBucketManager` for organized GPU memory allocation
   - Implements bucket-based rendering to reduce draw calls
   - Manages buffer uploads through `BufferWriteCommand`s

3. **Caching Strategy**
   - Implements LRU caching for chunk data
   - Tracks chunk visibility and priority
   - Manages memory pressure through smart eviction

### Key Components
- **ChunkCreationIterator**: Optimizes chunk memory layout
- **MeshBucketManager**: Organizes GPU memory into fixed-size buckets
- **BufferState**: Manages GPU buffer lifecycle
- **ChunkIndexState**: Tracks chunk positions in GPU memory

## Performance Considerations
- Minimizes memory fragmentation through fixed-size buckets
- Reduces CPU-GPU synchronization points
- Implements efficient memory pooling
- Uses indirect drawing to reduce draw call overhead
- Implements level-of-detail for distant chunks

## Error Handling
- Handles GPU memory allocation failures gracefully
- Implements fallback strategies for memory-constrained systems
- Logs detailed memory usage statistics

## See Also
- `src/engine_state/rendering/meshing/bucket_manager.rs`
- `src/engine_state/rendering/meshing/chunk_index_state.rs`
- `src/engine_state/voxels/chunk/chunk_creation.rs`
- `src/engine_state/buffer_state/`
