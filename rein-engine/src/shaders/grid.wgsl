// Grid shader for ground plane

struct CameraUniform {
    view_proj: mat4x4<f32>,
    eye: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
};

@vertex
fn vs_grid(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let world_pos = vec4<f32>(input.position, 1.0);
    output.clip_position = camera.view_proj * world_pos;
    output.world_position = input.position;
    return output;
}

@fragment
fn fs_grid(input: VertexOutput) -> @location(0) vec4<f32> {
    // Grid pattern
    let grid_size = 0.1;
    let line_width = 0.005;

    let fx = abs(fract(input.world_position.x / grid_size + 0.5) - 0.5);
    let fz = abs(fract(input.world_position.z / grid_size + 0.5) - 0.5);

    let line = min(fx, fz);
    let alpha = 1.0 - smoothstep(0.0, line_width / grid_size, line);

    // Axis lines (thicker)
    let axis_width = 0.01;
    let ax = smoothstep(axis_width, 0.0, abs(input.world_position.x));
    let az = smoothstep(axis_width, 0.0, abs(input.world_position.z));

    var color = vec3<f32>(0.3, 0.3, 0.3);
    if ax > 0.5 {
        color = vec3<f32>(0.0, 0.0, 0.8);  // Z axis = blue
    }
    if az > 0.5 {
        color = vec3<f32>(0.8, 0.0, 0.0);  // X axis = red
    }

    let final_alpha = max(alpha * 0.5, max(ax, az) * 0.8);
    return vec4<f32>(color, final_alpha);
}
