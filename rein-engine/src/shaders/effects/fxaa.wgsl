// FXAA (Fast Approximate Anti-Aliasing) shader
// Based on NVIDIA FXAA 3.11 Quality preset

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct FxaaUniform {
    // x: width, y: height, z: 1/width, w: 1/height
    texture_size: vec4<f32>,
};

@group(0) @binding(0)
var input_texture: texture_2d<f32>;
@group(0) @binding(1)
var input_sampler: sampler;
@group(0) @binding(2)
var<uniform> params: FxaaUniform;

// FXAA quality settings
const FXAA_EDGE_THRESHOLD: f32 = 0.166;
const FXAA_EDGE_THRESHOLD_MIN: f32 = 0.0833;
const FXAA_SUBPIX: f32 = 0.75;

// Convert RGB to luma (perceived brightness)
fn rgb_to_luma(rgb: vec3<f32>) -> f32 {
    return dot(rgb, vec3<f32>(0.299, 0.587, 0.114));
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
    let texel_size = params.texture_size.zw;

    // Sample center and neighboring pixels
    let center = textureSample(input_texture, input_sampler, uv);
    let luma_center = rgb_to_luma(center.rgb);

    // Sample the 4 direct neighbors
    let luma_n = rgb_to_luma(textureSample(input_texture, input_sampler, uv + vec2<f32>(0.0, -texel_size.y)).rgb);
    let luma_s = rgb_to_luma(textureSample(input_texture, input_sampler, uv + vec2<f32>(0.0, texel_size.y)).rgb);
    let luma_e = rgb_to_luma(textureSample(input_texture, input_sampler, uv + vec2<f32>(texel_size.x, 0.0)).rgb);
    let luma_w = rgb_to_luma(textureSample(input_texture, input_sampler, uv + vec2<f32>(-texel_size.x, 0.0)).rgb);

    // Find the maximum and minimum luma around the current pixel
    let luma_min = min(luma_center, min(min(luma_n, luma_s), min(luma_e, luma_w)));
    let luma_max = max(luma_center, max(max(luma_n, luma_s), max(luma_e, luma_w)));

    // Compute the contrast (delta)
    let luma_range = luma_max - luma_min;

    // If the contrast is lower than a threshold, don't apply AA
    if luma_range < max(FXAA_EDGE_THRESHOLD_MIN, luma_max * FXAA_EDGE_THRESHOLD) {
        return center;
    }

    // Sample the 4 corner neighbors
    let luma_nw = rgb_to_luma(textureSample(input_texture, input_sampler, uv + vec2<f32>(-texel_size.x, -texel_size.y)).rgb);
    let luma_ne = rgb_to_luma(textureSample(input_texture, input_sampler, uv + vec2<f32>(texel_size.x, -texel_size.y)).rgb);
    let luma_sw = rgb_to_luma(textureSample(input_texture, input_sampler, uv + vec2<f32>(-texel_size.x, texel_size.y)).rgb);
    let luma_se = rgb_to_luma(textureSample(input_texture, input_sampler, uv + vec2<f32>(texel_size.x, texel_size.y)).rgb);

    // Combine the four edges lumas
    let luma_ns = luma_n + luma_s;
    let luma_ew = luma_e + luma_w;

    // Same for corners
    let luma_nwne = luma_nw + luma_ne;
    let luma_swse = luma_sw + luma_se;

    // Compute horizontal and vertical gradients
    let edge_horz = abs(-2.0 * luma_w + luma_nwne) + abs(-2.0 * luma_center + luma_ns) * 2.0 + abs(-2.0 * luma_e + luma_swse);
    let edge_vert = abs(-2.0 * luma_n + (luma_nw + luma_ne)) + abs(-2.0 * luma_center + luma_ew) * 2.0 + abs(-2.0 * luma_s + (luma_sw + luma_se));

    // Is the edge horizontal or vertical?
    let is_horizontal = edge_horz >= edge_vert;

    // Select the two neighboring texels lumas in the opposite direction to the edge
    var luma1: f32;
    var luma2: f32;
    if is_horizontal {
        luma1 = luma_n;
        luma2 = luma_s;
    } else {
        luma1 = luma_w;
        luma2 = luma_e;
    }

    // Compute gradients in this direction
    let gradient1 = luma1 - luma_center;
    let gradient2 = luma2 - luma_center;

    // Which direction is the steepest?
    let is_1_steepest = abs(gradient1) >= abs(gradient2);

    // Gradient in the corresponding direction
    let gradient_scaled = 0.25 * max(abs(gradient1), abs(gradient2));

    // Average luma in the correct direction
    var luma_local_average: f32;
    if is_1_steepest {
        luma_local_average = 0.5 * (luma1 + luma_center);
    } else {
        luma_local_average = 0.5 * (luma2 + luma_center);
    }

    // Compute subpixel offset
    let luma_average_all = (1.0 / 12.0) * (2.0 * luma_ns + 2.0 * luma_ew + luma_nwne + luma_swse);
    let subpixel_offset1 = clamp(abs(luma_average_all - luma_center) / luma_range, 0.0, 1.0);
    let subpixel_offset2 = (-2.0 * subpixel_offset1 + 3.0) * subpixel_offset1 * subpixel_offset1;
    let subpixel_offset_final = subpixel_offset2 * subpixel_offset2 * FXAA_SUBPIX;

    // Step size
    var step_length: f32;
    if is_horizontal {
        step_length = texel_size.y;
    } else {
        step_length = texel_size.x;
    }

    // Correct for direction
    if is_1_steepest {
        step_length = -step_length;
    }

    // Shifted UV
    var uv_offset = uv;
    if is_horizontal {
        uv_offset.y += step_length * subpixel_offset_final;
    } else {
        uv_offset.x += step_length * subpixel_offset_final;
    }

    // Sample at the offset position
    let final_color = textureSample(input_texture, input_sampler, uv_offset);

    return final_color;
}
