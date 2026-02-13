// GPU narrowphase collision detection for specialized shape pairs.
// Handles sphere-sphere and box-sphere tests in parallel on the GPU.
// GJK/EPA pairs are left for CPU processing.

struct CollisionPair {
    entity_a: u32,
    entity_b: u32,
};

struct NarrowphaseResult {
    entity_a: u32,
    entity_b: u32,
    _pad0: u32,
    _pad1: u32,
    normal: vec3<f32>,
    penetration: f32,
    point: vec3<f32>,
    has_contact: u32,
};

// Shape data packed for GPU:
// type 0 = sphere: data.x = radius
// type 1 = box:    data.xyz = half_extents
struct ShapeData {
    position: vec3<f32>,
    shape_type: u32,
    data: vec4<f32>,
    // Box axes (only used for box shapes)
    axis_x: vec3<f32>,
    scale_x: f32,
    axis_y: vec3<f32>,
    scale_y: f32,
    axis_z: vec3<f32>,
    scale_z: f32,
};

@group(0) @binding(0) var<storage, read> pairs: array<CollisionPair>;
@group(0) @binding(1) var<storage, read> shapes: array<ShapeData>;
@group(0) @binding(2) var<storage, read_write> results: array<NarrowphaseResult>;
@group(1) @binding(0) var<uniform> params: vec4<u32>; // x = num_pairs

fn sphere_sphere_test(a: ShapeData, b: ShapeData) -> NarrowphaseResult {
    var result: NarrowphaseResult;
    result.has_contact = 0u;

    let diff = b.position - a.position;
    let dist_sq = dot(diff, diff);
    let radius_a = a.data.x * max(a.scale_x, max(a.scale_y, a.scale_z));
    let radius_b = b.data.x * max(b.scale_x, max(b.scale_y, b.scale_z));
    let min_dist = radius_a + radius_b;

    if (dist_sq >= min_dist * min_dist) {
        return result;
    }

    let dist = sqrt(dist_sq);
    var normal: vec3<f32>;
    if (dist > 1e-6) {
        normal = diff / dist;
    } else {
        normal = vec3<f32>(0.0, 1.0, 0.0);
    }

    result.normal = normal;
    result.penetration = min_dist - dist;
    result.point = a.position + normal * (radius_a - result.penetration * 0.5);
    result.has_contact = 1u;
    return result;
}

fn box_sphere_test(box_shape: ShapeData, sphere_shape: ShapeData) -> NarrowphaseResult {
    var result: NarrowphaseResult;
    result.has_contact = 0u;

    let sphere_center = sphere_shape.position;
    let box_center = box_shape.position;
    let world_radius = sphere_shape.data.x * max(sphere_shape.scale_x, max(sphere_shape.scale_y, sphere_shape.scale_z));

    let box_axes = array<vec3<f32>, 3>(
        box_shape.axis_x,
        box_shape.axis_y,
        box_shape.axis_z,
    );
    let scaled_half = vec3<f32>(
        box_shape.data.x * box_shape.scale_x,
        box_shape.data.y * box_shape.scale_y,
        box_shape.data.z * box_shape.scale_z,
    );

    let diff = sphere_center - box_center;
    let local = vec3<f32>(
        dot(diff, box_axes[0]),
        dot(diff, box_axes[1]),
        dot(diff, box_axes[2]),
    );

    let clamped = clamp(local, -scaled_half, scaled_half);
    let closest_world = box_center
        + box_axes[0] * clamped.x
        + box_axes[1] * clamped.y
        + box_axes[2] * clamped.z;

    let to_sphere = sphere_center - closest_world;
    let dist_sq = dot(to_sphere, to_sphere);

    if (dist_sq >= world_radius * world_radius) {
        return result;
    }

    let dist = sqrt(dist_sq);

    if (dist < 1e-6) {
        // Sphere center inside box - find minimum penetration axis
        var min_pen = 1e30;
        var normal = vec3<f32>(0.0, 1.0, 0.0);
        for (var i = 0u; i < 3u; i++) {
            let pen_pos = scaled_half[i] - local[i];
            let pen_neg = scaled_half[i] + local[i];
            if (pen_pos < min_pen) {
                min_pen = pen_pos;
                normal = box_axes[i];
            }
            if (pen_neg < min_pen) {
                min_pen = pen_neg;
                normal = -box_axes[i];
            }
        }
        result.normal = normal;
        result.penetration = min_pen + world_radius;
        result.point = sphere_center - normal * world_radius;
        result.has_contact = 1u;
        return result;
    }

    result.normal = to_sphere / dist;
    result.penetration = world_radius - dist;
    result.point = closest_world;
    result.has_contact = 1u;
    return result;
}

@compute @workgroup_size(64)
fn cs_narrowphase(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let num_pairs = params.x;

    if (i >= num_pairs) {
        return;
    }

    let pair = pairs[i];
    let shape_a = shapes[pair.entity_a];
    let shape_b = shapes[pair.entity_b];

    var result: NarrowphaseResult;
    result.entity_a = pair.entity_a;
    result.entity_b = pair.entity_b;
    result._pad0 = 0u;
    result._pad1 = 0u;
    result.has_contact = 0u;

    // Dispatch based on shape types
    if (shape_a.shape_type == 0u && shape_b.shape_type == 0u) {
        // sphere-sphere
        result = sphere_sphere_test(shape_a, shape_b);
        result.entity_a = pair.entity_a;
        result.entity_b = pair.entity_b;
    } else if (shape_a.shape_type == 1u && shape_b.shape_type == 0u) {
        // box-sphere
        result = box_sphere_test(shape_a, shape_b);
        result.entity_a = pair.entity_a;
        result.entity_b = pair.entity_b;
    } else if (shape_a.shape_type == 0u && shape_b.shape_type == 1u) {
        // sphere-box (swap and flip normal)
        result = box_sphere_test(shape_b, shape_a);
        result.entity_a = pair.entity_a;
        result.entity_b = pair.entity_b;
        result.normal = -result.normal;
    }
    // Other combos: has_contact stays 0, CPU will handle via GJK/EPA

    results[i] = result;
}
