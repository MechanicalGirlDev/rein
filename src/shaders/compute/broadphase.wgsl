// GPU broadphase collision detection.
// Tests all AABB pairs and outputs overlapping pairs.

struct AABB {
    min: vec3<f32>,
    entity_id: u32,
    max: vec3<f32>,
    _padding: u32,
};

struct CollisionPair {
    entity_a: u32,
    entity_b: u32,
};

@group(0) @binding(0) var<storage, read> aabbs: array<AABB>;
@group(0) @binding(1) var<storage, read_write> pairs: array<CollisionPair>;
@group(0) @binding(2) var<storage, read_write> pair_count: atomic<u32>;
@group(1) @binding(0) var<uniform> params: vec4<u32>; // x = num_bodies, y = max_pairs

@compute @workgroup_size(64)
fn cs_broadphase(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let count = params.x;
    let max_pairs = params.y;

    if (i >= count) {
        return;
    }

    let a = aabbs[i];

    for (var j = i + 1u; j < count; j++) {
        let b = aabbs[j];

        if (a.min.x <= b.max.x && a.max.x >= b.min.x
         && a.min.y <= b.max.y && a.max.y >= b.min.y
         && a.min.z <= b.max.z && a.max.z >= b.min.z) {
            let idx = atomicAdd(&pair_count, 1u);
            if (idx < max_pairs) {
                pairs[idx] = CollisionPair(a.entity_id, b.entity_id);
            }
        }
    }
}
