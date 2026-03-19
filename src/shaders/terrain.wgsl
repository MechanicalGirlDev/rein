// Terrain shader with height-based coloring

struct CameraUniform {
    view_proj: mat4x4<f32>,
    eye: vec4<f32>,
};

struct ModelUniform {
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
};

struct TerrainUniform {
    min_height: f32,
    max_height: f32,
    _padding0: f32,
    _padding1: f32,
    color_low: vec4<f32>,
    color_high: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> model: ModelUniform;

@group(2) @binding(0)
var<uniform> terrain: TerrainUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) height_factor: f32,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    let world_pos = model.model * vec4<f32>(input.position, 1.0);
    output.clip_position = camera.view_proj * world_pos;
    output.world_position = world_pos.xyz;
    output.world_normal = normalize((model.normal_matrix * vec4<f32>(input.normal, 0.0)).xyz);

    let height_range = terrain.max_height - terrain.min_height;
    if height_range > 0.0 {
        output.height_factor = clamp((input.position.y - terrain.min_height) / height_range, 0.0, 1.0);
    } else {
        output.height_factor = 0.5;
    }

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(0.3, 1.0, 0.5));
    let ambient = 0.3;

    let n = normalize(input.world_normal);
    let diffuse = max(dot(n, light_dir), 0.0);

    let view_dir = normalize(camera.eye.xyz - input.world_position);
    let half_dir = normalize(light_dir + view_dir);
    let specular = pow(max(dot(n, half_dir), 0.0), 16.0) * 0.15;

    let lighting = ambient + diffuse * 0.7 + specular;

    let t = input.height_factor;
    let base_color = mix(terrain.color_low, terrain.color_high, vec4<f32>(t, t, t, t));

    return vec4<f32>(base_color.rgb * lighting, 1.0);
}
