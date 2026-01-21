// UI Shader for simple 2D elements

// Vertex shader
@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>
) -> VertexOutput {
    var output: VertexOutput;
    // Pass through the position for UI elements
    // UI elements are already in normalized device coordinates
    output.position = vec4<f32>(position.xyz, 1.0);
    // Pass the color to the fragment shader
    output.color = color;
    return output;
}

// Output structure for vertex shader
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Use the color passed from the vertex shader
    return in.color;
}
