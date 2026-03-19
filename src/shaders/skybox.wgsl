// Skybox shader - renders a cubemap as the background sky

struct SkyboxUniform {
    view_proj_no_translation: mat4x4<f32>,
    color_top: vec4<f32>,
    color_horizon: vec4<f32>,
    color_bottom: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> skybox: SkyboxUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_position: vec3<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.local_position = input.position;
    let pos = skybox.view_proj_no_translation * vec4<f32>(input.position, 1.0);
    // Set z = w so the skybox is always at the far plane
    output.clip_position = vec4<f32>(pos.xy, pos.w, pos.w);
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let dir = normalize(input.local_position);
    let y = dir.y;

    // Gradient sky based on vertical direction
    if y > 0.0 {
        let t = clamp(y, 0.0, 1.0);
        return mix(skybox.color_horizon, skybox.color_top, vec4<f32>(t, t, t, t));
    } else {
        let t = clamp(-y, 0.0, 1.0);
        return mix(skybox.color_horizon, skybox.color_bottom, vec4<f32>(t, t, t, t));
    }
}
