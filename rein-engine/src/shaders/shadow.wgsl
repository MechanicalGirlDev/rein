// Shadow map depth pass shader
// Renders depth from light's perspective for shadow mapping

struct DepthPassUniform {
    light_view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: DepthPassUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let world_pos = uniforms.model * vec4<f32>(input.position, 1.0);
    output.position = uniforms.light_view_proj * world_pos;
    return output;
}

// Fragment shader is minimal - we only care about depth
@fragment
fn fs_main(input: VertexOutput) {
    // Depth is written automatically by the depth attachment
    // No color output needed
}
