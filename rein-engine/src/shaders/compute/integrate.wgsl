// GPU velocity and position integration for rigid bodies.
//
// Each body is represented as a contiguous block of data:
//   [0..3]   position (xyz) + body_type (w: 0=dynamic, 1=static, 2=kinematic)
//   [4..7]   linear_velocity (xyz) + mass (w)
//   [8..11]  angular_velocity (xyz) + gravity_scale (w)
//   [12..15] force_accumulator (xyz) + linear_damping (w)
//   [16..19] torque_accumulator (xyz) + angular_damping (w)
//   [20..23] inertia_diag (xyz) + _padding (w)
//   [24..27] rotation quaternion (xyzw)

struct Body {
    position: vec3<f32>,
    body_type: u32,       // 0 = Dynamic, 1 = Static, 2 = Kinematic
    linear_velocity: vec3<f32>,
    mass: f32,
    angular_velocity: vec3<f32>,
    gravity_scale: f32,
    force_accumulator: vec3<f32>,
    linear_damping: f32,
    torque_accumulator: vec3<f32>,
    angular_damping: f32,
    inertia_diag: vec3<f32>,
    _padding: f32,
    rotation: vec4<f32>,
};

struct Params {
    num_bodies: u32,
    dt: f32,
    gravity_x: f32,
    gravity_y: f32,
    gravity_z: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
};

@group(0) @binding(0) var<storage, read_write> bodies: array<Body>;
@group(1) @binding(0) var<uniform> params: Params;

@compute @workgroup_size(64)
fn cs_integrate_velocities(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.num_bodies) {
        return;
    }

    var body = bodies[i];

    // Skip non-dynamic bodies
    if (body.body_type != 0u || body.mass <= 0.0) {
        return;
    }

    let inv_mass = 1.0 / body.mass;
    let gravity = vec3<f32>(params.gravity_x, params.gravity_y, params.gravity_z);

    // Apply gravity
    body.force_accumulator += gravity * body.mass * body.gravity_scale;

    // Integrate linear velocity: v += (F/m) * dt
    body.linear_velocity += body.force_accumulator * inv_mass * params.dt;

    // Integrate angular velocity: omega += (tau / I) * dt
    let inv_inertia = vec3<f32>(
        select(0.0, 1.0 / body.inertia_diag.x, body.inertia_diag.x > 0.0),
        select(0.0, 1.0 / body.inertia_diag.y, body.inertia_diag.y > 0.0),
        select(0.0, 1.0 / body.inertia_diag.z, body.inertia_diag.z > 0.0),
    );
    body.angular_velocity += body.torque_accumulator * inv_inertia * params.dt;

    // Apply damping
    body.linear_velocity *= max(1.0 - body.linear_damping, 0.0);
    body.angular_velocity *= max(1.0 - body.angular_damping, 0.0);

    // Clear force accumulators
    body.force_accumulator = vec3<f32>(0.0, 0.0, 0.0);
    body.torque_accumulator = vec3<f32>(0.0, 0.0, 0.0);

    bodies[i] = body;
}

@compute @workgroup_size(64)
fn cs_integrate_positions(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.num_bodies) {
        return;
    }

    var body = bodies[i];

    // Skip non-dynamic bodies
    if (body.body_type != 0u) {
        return;
    }

    // Integrate position: p += v * dt
    body.position += body.linear_velocity * params.dt;

    // Integrate rotation: q' = q + 0.5 * dt * omega_quat * q
    let omega = body.angular_velocity;
    let omega_len_sq = dot(omega, omega);
    if (omega_len_sq > 1e-10) {
        let omega_quat = vec4<f32>(omega.x, omega.y, omega.z, 0.0);
        let q = body.rotation;
        // quaternion multiply: omega_quat * q
        let qm = vec4<f32>(
            omega_quat.w * q.x + omega_quat.x * q.w + omega_quat.y * q.z - omega_quat.z * q.y,
            omega_quat.w * q.y - omega_quat.x * q.z + omega_quat.y * q.w + omega_quat.z * q.x,
            omega_quat.w * q.z + omega_quat.x * q.y - omega_quat.y * q.x + omega_quat.z * q.w,
            omega_quat.w * q.w - omega_quat.x * q.x - omega_quat.y * q.y - omega_quat.z * q.z,
        );
        let q_dot = qm * 0.5;
        var new_q = q + q_dot * params.dt;
        // Normalize
        let len = length(new_q);
        if (len > 0.0) {
            new_q = new_q / len;
        }
        body.rotation = new_q;
    }

    bodies[i] = body;
}
