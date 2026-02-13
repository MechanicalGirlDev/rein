// Depth shader for shadow map generation

struct LightMatrixUniform {
    light_view_proj: mat4x4<f32>,
};

struct ModelUniform {
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> light: LightMatrixUniform;

@group(1) @binding(0)
var<uniform> model: ModelUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    let world_pos = model.model * vec4<f32>(input.position, 1.0);
    output.clip_position = light.light_view_proj * world_pos;

    return output;
}

// No fragment shader output needed for depth-only rendering
// The depth buffer is written automatically
@fragment
fn fs_main() {
    // Empty fragment shader - depth is written automatically
}
