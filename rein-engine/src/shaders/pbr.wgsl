// PBR (Physically Based Rendering) shader with metallic-roughness workflow

struct CameraUniform {
    view_proj: mat4x4<f32>,
    eye: vec4<f32>,
};

struct ModelUniform {
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
};

struct PbrUniform {
    base_color: vec4<f32>,
    emissive: vec4<f32>,
    metallic: f32,
    roughness: f32,
    ao: f32,
    _padding: f32,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> model: ModelUniform;

@group(2) @binding(0)
var<uniform> pbr: PbrUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) color: vec3<f32>,
};

const PI: f32 = 3.14159265359;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    let world_pos = model.model * vec4<f32>(input.position, 1.0);
    output.clip_position = camera.view_proj * world_pos;
    output.world_position = world_pos.xyz;
    output.world_normal = normalize((model.normal_matrix * vec4<f32>(input.normal, 0.0)).xyz);
    output.color = input.color;

    return output;
}

// Normal Distribution Function (GGX/Trowbridge-Reitz)
fn distribution_ggx(n_dot_h: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let denom = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
    return a2 / (PI * denom * denom);
}

// Geometry function (Schlick-GGX)
fn geometry_schlick_ggx(n_dot_v: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    return n_dot_v / (n_dot_v * (1.0 - k) + k);
}

// Smith's method for geometry term
fn geometry_smith(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
    let ggx1 = geometry_schlick_ggx(n_dot_v, roughness);
    let ggx2 = geometry_schlick_ggx(n_dot_l, roughness);
    return ggx1 * ggx2;
}

// Fresnel-Schlick approximation
fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (1.0 - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let N = normalize(input.world_normal);
    let V = normalize(camera.eye.xyz - input.world_position);

    // Material properties
    let albedo = pbr.base_color.rgb * input.color;
    let metallic = pbr.metallic;
    let roughness = max(pbr.roughness, 0.04); // Prevent zero roughness
    let ao = pbr.ao;

    // Calculate F0 (reflectance at normal incidence)
    let f0 = mix(vec3<f32>(0.04), albedo, metallic);

    // Simple directional light (sun-like)
    let light_dir = normalize(vec3<f32>(0.3, 1.0, 0.5));
    let light_color = vec3<f32>(1.0, 0.98, 0.95);
    let light_intensity = 2.0;

    // Calculate lighting
    let L = light_dir;
    let H = normalize(V + L);

    let n_dot_l = max(dot(N, L), 0.0);
    let n_dot_v = max(dot(N, V), 0.0);
    let n_dot_h = max(dot(N, H), 0.0);
    let v_dot_h = max(dot(V, H), 0.0);

    // Cook-Torrance BRDF
    let D = distribution_ggx(n_dot_h, roughness);
    let G = geometry_smith(n_dot_v, n_dot_l, roughness);
    let F = fresnel_schlick(v_dot_h, f0);

    // Specular contribution
    let numerator = D * G * F;
    let denominator = 4.0 * n_dot_v * n_dot_l + 0.0001;
    let specular = numerator / denominator;

    // Diffuse contribution (energy conservation)
    let kS = F;
    let kD = (1.0 - kS) * (1.0 - metallic);

    let diffuse = kD * albedo / PI;

    // Combine lighting
    let radiance = light_color * light_intensity;
    var Lo = (diffuse + specular) * radiance * n_dot_l;

    // Ambient lighting (simple)
    let ambient = vec3<f32>(0.03) * albedo * ao;

    // Emissive
    let emissive = pbr.emissive.rgb;

    // Final color
    var color = ambient + Lo + emissive;

    // Tone mapping (Reinhard)
    color = color / (color + vec3<f32>(1.0));

    // Gamma correction
    color = pow(color, vec3<f32>(1.0 / 2.2));

    return vec4<f32>(color, pbr.base_color.a);
}
