struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) mode: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) @interpolate(flat) mode: u32,
};

struct ScreenSize {
    size: vec2<f32>,
    padding: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> screen: ScreenSize;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    let x = (input.position.x / screen.size.x) * 2.0 - 1.0;
    let y = 1.0 - (input.position.y / screen.size.y) * 2.0;

    output.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    output.uv = input.uv;
    output.color = input.color;
    output.mode = input.mode;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    if (input.mode == 1u) {
        let dist = distance(input.uv, vec2<f32>(0.5, 0.5));
        if (dist > 0.5) {
            discard;
        }
        // Simple AA
        let alpha = 1.0 - smoothstep(0.48, 0.5, dist);
        return vec4<f32>(input.color.rgb, input.color.a * alpha);
    }
    return input.color;
}
