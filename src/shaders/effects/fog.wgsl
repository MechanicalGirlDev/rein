// Fog post-processing shader
// Supports Linear, Exponential, and Exponential Squared fog modes

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct FogUniform {
    // rgb: fog color, a: mode (0=linear, 1=exp, 2=exp2)
    color_mode: vec4<f32>,
    // x: start, y: end, z: density, w: far plane
    params: vec4<f32>,
};

@group(0) @binding(0)
var color_texture: texture_2d<f32>;
@group(0) @binding(1)
var depth_texture: texture_depth_2d;
@group(0) @binding(2)
var color_sampler: sampler;
@group(0) @binding(3)
var depth_sampler: sampler;
@group(0) @binding(4)
var<uniform> fog: FogUniform;

// Convert depth buffer value to linear depth
fn linearize_depth(depth: f32, far: f32) -> f32 {
    // Assuming reverse-Z with infinite far plane or standard depth
    // For standard [0,1] depth buffer: linear = near * far / (far - depth * (far - near))
    // Simplified for far >> near: linear ≈ far * depth
    return far * depth;
}

// Calculate fog factor based on mode
fn calculate_fog_factor(distance: f32) -> f32 {
    let mode = i32(fog.color_mode.a);
    let start = fog.params.x;
    let end = fog.params.y;
    let density = fog.params.z;

    var fog_factor: f32;

    if mode == 0 {
        // Linear fog: (end - distance) / (end - start)
        fog_factor = (end - distance) / (end - start);
    } else if mode == 1 {
        // Exponential fog: exp(-density * distance)
        fog_factor = exp(-density * distance);
    } else {
        // Exponential squared fog: exp(-(density * distance)^2)
        let d = density * distance;
        fog_factor = exp(-d * d);
    }

    return clamp(fog_factor, 0.0, 1.0);
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(input.position.xy, 0.0, 1.0);
    output.uv = input.uv.xy;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let uv = input.uv;

    // Sample color and depth
    let color = textureSample(color_texture, color_sampler, uv);
    let depth = textureSample(depth_texture, depth_sampler, uv);

    // Convert depth to linear distance
    let far = fog.params.w;
    let distance = linearize_depth(depth, far);

    // Calculate fog factor (1 = no fog, 0 = full fog)
    let fog_factor = calculate_fog_factor(distance);

    // Get fog color
    let fog_color = fog.color_mode.rgb;

    // Mix original color with fog color
    let final_color = mix(fog_color, color.rgb, fog_factor);

    return vec4<f32>(final_color, color.a);
}
