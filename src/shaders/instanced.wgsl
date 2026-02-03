// Instanced mesh shader with per-instance transforms and colors

struct CameraUniform {
    view_proj: mat4x4<f32>,
    eye: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    // Per-vertex attributes
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
    // Per-instance attributes
    @location(4) instance_col0: vec4<f32>,
    @location(5) instance_col1: vec4<f32>,
    @location(6) instance_col2: vec4<f32>,
    @location(7) instance_col3: vec4<f32>,
    @location(8) instance_color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) color: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    // Reconstruct the instance transform matrix
    let instance_matrix = mat4x4<f32>(
        input.instance_col0,
        input.instance_col1,
        input.instance_col2,
        input.instance_col3
    );

    let world_pos = instance_matrix * vec4<f32>(input.position, 1.0);
    output.clip_position = camera.view_proj * world_pos;
    output.world_position = world_pos.xyz;

    // Calculate normal matrix (inverse transpose of upper-left 3x3)
    let normal_matrix = mat3x3<f32>(
        instance_matrix[0].xyz,
        instance_matrix[1].xyz,
        instance_matrix[2].xyz
    );
    output.world_normal = normalize(normal_matrix * input.normal);

    // Modulate vertex color with instance color
    output.color = vec4<f32>(input.color, 1.0) * input.instance_color;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Simple directional light
    let light_dir = normalize(vec3<f32>(0.3, 1.0, 0.5));
    let ambient = 0.3;

    let n = normalize(input.world_normal);
    let diffuse = max(dot(n, light_dir), 0.0);

    // Specular (Blinn-Phong)
    let view_dir = normalize(camera.eye.xyz - input.world_position);
    let half_dir = normalize(light_dir + view_dir);
    let specular = pow(max(dot(n, half_dir), 0.0), 32.0) * 0.3;

    let lighting = ambient + diffuse * 0.7 + specular;
    let final_color = input.color.rgb * lighting;

    return vec4<f32>(final_color, input.color.a);
}
