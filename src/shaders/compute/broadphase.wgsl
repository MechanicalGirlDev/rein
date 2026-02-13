// GPU broadphase collision detection using spatial hash grid.
// Two-pass algorithm:
//   Pass 1 (cs_assign_cells): Compute cell IDs for each AABB
//   Pass 2 (cs_broadphase_spatial): Test pairs within the same cell

struct AABB {
    min: vec3<f32>,
    entity_id: u32,
    max: vec3<f32>,
    body_type: u32, // 0=Dynamic, 1=Static, 2=Kinematic
};

struct CollisionPair {
    entity_a: u32,
    entity_b: u32,
};

// Cell assignment output for each body.
// Each body can span up to 8 cells (2x2x2 corners of AABB).
struct CellAssignment {
    cell_hash: array<u32, 8>,
    num_cells: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

@group(0) @binding(0) var<storage, read> aabbs: array<AABB>;
@group(0) @binding(1) var<storage, read_write> pairs: array<CollisionPair>;
@group(0) @binding(2) var<storage, read_write> pair_count: atomic<u32>;
@group(0) @binding(3) var<storage, read_write> cell_assignments: array<CellAssignment>;

// params.x = num_bodies, params.y = max_pairs, params.z = cell_size_bits (as u32), params.w = unused
@group(1) @binding(0) var<uniform> params: vec4<u32>;

// Hash function for cell coordinates
fn hash_cell(cx: i32, cy: i32, cz: i32) -> u32 {
    // FNV-like hash for grid cells
    var h: u32 = 2166136261u;
    h = h ^ u32(cx + 32768);
    h = h * 16777619u;
    h = h ^ u32(cy + 32768);
    h = h * 16777619u;
    h = h ^ u32(cz + 32768);
    h = h * 16777619u;
    return h;
}

// Pass 1: Assign each body to its overlapping cells
@compute @workgroup_size(64)
fn cs_assign_cells(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let count = params.x;

    if (i >= count) {
        return;
    }

    let a = aabbs[i];
    let cell_size = bitcast<f32>(params.z);
    let inv_cell = 1.0 / cell_size;

    let min_cx = i32(floor(a.min.x * inv_cell));
    let min_cy = i32(floor(a.min.y * inv_cell));
    let min_cz = i32(floor(a.min.z * inv_cell));
    let max_cx = i32(floor(a.max.x * inv_cell));
    let max_cy = i32(floor(a.max.y * inv_cell));
    let max_cz = i32(floor(a.max.z * inv_cell));

    var assignment: CellAssignment;
    var idx: u32 = 0u;

    for (var cx = min_cx; cx <= max_cx && idx < 8u; cx++) {
        for (var cy = min_cy; cy <= max_cy && idx < 8u; cy++) {
            for (var cz = min_cz; cz <= max_cz && idx < 8u; cz++) {
                assignment.cell_hash[idx] = hash_cell(cx, cy, cz);
                idx++;
            }
        }
    }
    assignment.num_cells = idx;
    assignment._pad0 = 0u;
    assignment._pad1 = 0u;
    assignment._pad2 = 0u;

    cell_assignments[i] = assignment;
}

// Check AABB overlap
fn aabb_overlaps(a: AABB, b: AABB) -> bool {
    return a.min.x <= b.max.x && a.max.x >= b.min.x
        && a.min.y <= b.max.y && a.max.y >= b.min.y
        && a.min.z <= b.max.z && a.max.z >= b.min.z;
}

// Pass 2: Test pairs that share a cell
@compute @workgroup_size(64)
fn cs_broadphase_spatial(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let count = params.x;
    let max_pairs = params.y;

    if (i >= count) {
        return;
    }

    let a = aabbs[i];
    let ca = cell_assignments[i];

    // For each body j > i, check if they share a cell
    for (var j = i + 1u; j < count; j++) {
        let b = aabbs[j];

        // Skip static-static pairs
        if (a.body_type == 1u && b.body_type == 1u) {
            continue;
        }

        let cb = cell_assignments[j];

        // Check if any cell hashes match
        var share_cell = false;
        for (var ci = 0u; ci < ca.num_cells && !share_cell; ci++) {
            for (var cj = 0u; cj < cb.num_cells && !share_cell; cj++) {
                if (ca.cell_hash[ci] == cb.cell_hash[cj]) {
                    share_cell = true;
                }
            }
        }

        if (!share_cell) {
            continue;
        }

        // Verify actual AABB overlap
        if (aabb_overlaps(a, b)) {
            let idx = atomicAdd(&pair_count, 1u);
            if (idx < max_pairs) {
                pairs[idx] = CollisionPair(a.entity_id, b.entity_id);
            }
        }
    }
}

// Legacy fallback: O(n^2) brute-force broadphase
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

        if (aabb_overlaps(a, b)) {
            let idx = atomicAdd(&pair_count, 1u);
            if (idx < max_pairs) {
                pairs[idx] = CollisionPair(a.entity_id, b.entity_id);
            }
        }
    }
}
