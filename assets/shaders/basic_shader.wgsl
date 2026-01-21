struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) x: i32,
    @location(1) y: i32,
    @location(2) z: i32,
    @location(3) tex_index: u32,
    @location(4) tex_coords: vec2<f32>,
    @location(5) chunk_coordinate_index: u32,
}

struct VertexOutput {
    @builtin(position) clip_position:vec4<f32>,
    @location(0) @interpolate(flat) tex_index: u32,
    @location(1) tex_coords: vec2<f32>,
};

//This should be a compile-time constant
const chunk_position_size: i32 = 750;
struct ChunkPositions {
    chunk_positions: array<i32,chunk_position_size>
};
@group(2) @binding(0)
var<storage> chunkPositions: ChunkPositions;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var pos = vec4<f32>(f32(model.x), f32(model.y), f32(model.z), 1.0);
    let cci = model.chunk_coordinate_index;
    pos.x += f32(16 * chunkPositions.chunk_positions[3u*cci]);
    pos.y += f32(16 * chunkPositions.chunk_positions[3u*cci+1u]);
    pos.z += f32(16 * chunkPositions.chunk_positions[3u*cci+2u]);
    var out: VertexOutput;
    out.clip_position = camera.view_proj * pos;
    out.tex_index = model.tex_index;
    out.tex_coords = model.tex_coords;
    return out;
}

@group(1) @binding(0)
var diffuse_texture_array: texture_2d_array<f32>;
@group(1) @binding(1)
var sampler_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex_color = textureSample(diffuse_texture_array, sampler_diffuse, in.tex_coords, in.tex_index);
    return tex_color;
}