# Shader Documentation

This directory contains WebGPU Shading Language (WGSL) shaders for the voxel engine's rendering pipeline.

## Shader Files

### 1. `basic_shader.wgsl`

A basic shader for rendering voxels with texture array support.

#### Features:
- Vertex shader that transforms voxel positions using chunk-relative coordinates
- Supports texture arrays for efficient texture sampling
- Implements chunk-based positioning system

#### Bind Groups:
- **Group 0 (Uniform)**: Camera view-projection matrix
  - Binding 0: `camera` - Camera view-projection matrix
- **Group 1 (Texture)**: Texture resources
  - Binding 0: `diffuse_texture_array` - Array of 2D textures
  - Binding 1: `sampler_diffuse` - Texture sampler
- **Group 2 (Storage)**: Chunk positions
  - Binding 0: `chunkPositions` - Array of chunk positions

#### Vertex Attributes:
- Position (x, y, z): i32
- Texture Index: u32
- Texture Coordinates: vec2<f32>
- Chunk Coordinate Index: u32

---

### 2. `basic_shader_texture_binding_array.wgsl`

A variant of the basic shader that uses texture binding arrays instead of texture arrays.

#### Key Differences from `basic_shader.wgsl`:
- Uses `binding_array<texture_2d<f32>>` instead of `texture_2d_array<f32>`
- Different texture sampling approach in the fragment shader
- Potentially better performance on some hardware due to different binding model

#### Bind Groups:
- Same as `basic_shader.wgsl` but with different texture binding:
  - Binding 0: `diffuse_texture_array` - Binding array of 2D textures

#### When to Use:
- Use this shader if your target hardware performs better with texture binding arrays
- Switch to the other shader if you encounter compatibility issues

## Common Structures

### Camera Uniform
```wgsl
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
```

### Vertex Input
```wgsl
struct VertexInput {
    @location(0) x: i32,
    @location(1) y: i32,
    @location(2) z: i32,
    @location(3) tex_index: u32,
    @location(4) tex_coords: vec2<f32>,
    @location(5) chunk_coordinate_index: u32,
}
```

### Chunk Positions
```wgsl
const chunk_position_size: i32 = 750;
struct ChunkPositions {
    chunk_positions: array<i32, chunk_position_size>
};
```

## Performance Considerations
- The shaders are optimized for batch rendering of voxels
- Chunk-based positioning minimizes the number of draw calls
- Texture arrays/binding arrays reduce state changes between draws

## See Also
- [WebGPU Shading Language (WGSL) Specification](https://www.w3.org/TR/WGSL/)
- [WebGPU API Documentation](https://gpuweb.github.io/gpuweb/)

## Version History
- Initial version: Basic shader implementation with texture array support
- Added: Texture binding array variant for improved hardware compatibility
