// Sprite billboard shader
//
// Renders quads that always face the camera.
// The vertex position stores the sprite center, and the normal field
// stores the corner offset (x, y) which is expanded in view space.

struct CameraUniform {
    view_proj: mat4x4<f32>,
    eye: vec4<f32>,
};

struct ModelUniform {
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> model: ModelUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) offset: vec3<f32>,
    @location(2) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    // Sprite center in world space
    let world_center = (model.model * vec4<f32>(input.position, 1.0)).xyz;

    // Camera right and up vectors from the view-projection matrix inverse
    // We compute them from the camera eye and an assumed up direction
    let forward = normalize(camera.eye.xyz - world_center);
    let world_up = vec3<f32>(0.0, 1.0, 0.0);
    let right = normalize(cross(world_up, forward));
    let up = cross(forward, right);

    // Expand the quad corners in view space
    let world_pos = world_center + right * input.offset.x + up * input.offset.y;

    output.clip_position = camera.view_proj * vec4<f32>(world_pos, 1.0);
    output.color = input.color;
    // UV from offset: map [-size, size] to [0, 1]
    output.uv = input.offset.xy * 0.5 + 0.5;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(input.color, 1.0);
}
