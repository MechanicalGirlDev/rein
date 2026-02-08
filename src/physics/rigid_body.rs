//! Rigid body integration functions.

use glam::{Quat, Vec3};

use crate::ecs::components::physics::{RigidBody, RigidBodyType};
use crate::ecs::components::transform::{GlobalTransform, Transform};

/// Apply gravity force to all dynamic rigid bodies.
pub fn apply_gravity(world: &mut hecs::World, gravity: Vec3) {
    for (_, rb) in world.query_mut::<&mut RigidBody>() {
        if rb.body_type == RigidBodyType::Dynamic && rb.mass > 0.0 {
            rb.force_accumulator += gravity * rb.mass * rb.gravity_scale;
        }
    }
}

/// Integrate velocities using semi-implicit Euler: v += (F/m) * dt.
pub fn integrate_velocities(world: &mut hecs::World, dt: f32) {
    for (_, rb) in world.query_mut::<&mut RigidBody>() {
        if rb.body_type != RigidBodyType::Dynamic || rb.mass <= 0.0 {
            continue;
        }

        let inv_mass = 1.0 / rb.mass;

        // Linear velocity: v += (F/m) * dt
        rb.linear_velocity += rb.force_accumulator * inv_mass * dt;

        // Angular velocity: omega += (tau / I) * dt
        // Using diagonal inertia approximation (inertia_tensor[0], [4], [8])
        let inv_inertia = Vec3::new(
            if rb.inertia_tensor[0] > 0.0 {
                1.0 / rb.inertia_tensor[0]
            } else {
                0.0
            },
            if rb.inertia_tensor[4] > 0.0 {
                1.0 / rb.inertia_tensor[4]
            } else {
                0.0
            },
            if rb.inertia_tensor[8] > 0.0 {
                1.0 / rb.inertia_tensor[8]
            } else {
                0.0
            },
        );
        rb.angular_velocity += rb.torque_accumulator * inv_inertia * dt;

        // Apply damping
        rb.linear_velocity *= (1.0 - rb.linear_damping).max(0.0);
        rb.angular_velocity *= (1.0 - rb.angular_damping).max(0.0);
    }
}

/// Integrate positions: p += v * dt, q += 0.5 * omega * q * dt.
pub fn integrate_positions(world: &mut hecs::World, dt: f32) {
    for (_, (rb, transform)) in world.query_mut::<(&RigidBody, &mut Transform)>() {
        if rb.body_type != RigidBodyType::Dynamic {
            continue;
        }

        // Update position
        transform.position += rb.linear_velocity * dt;

        // Update rotation using quaternion integration
        // q' = q + 0.5 * dt * omega_quat * q
        let omega = rb.angular_velocity;
        if omega.length_squared() > 1e-10 {
            let omega_quat = Quat::from_xyzw(omega.x, omega.y, omega.z, 0.0);
            let q_dot = omega_quat * transform.rotation * 0.5;
            transform.rotation = Quat::from_xyzw(
                transform.rotation.x + q_dot.x * dt,
                transform.rotation.y + q_dot.y * dt,
                transform.rotation.z + q_dot.z * dt,
                transform.rotation.w + q_dot.w * dt,
            )
            .normalize();
        }
    }
}

/// Synchronize RigidBody positions/rotations to Transform and GlobalTransform.
pub fn sync_transforms(world: &mut hecs::World) {
    for (_, (transform, global)) in world.query_mut::<(&Transform, &mut GlobalTransform)>() {
        global.0 = transform.to_matrix();
    }
}

/// Clear force and torque accumulators on all rigid bodies.
pub fn clear_forces(world: &mut hecs::World) {
    for (_, rb) in world.query_mut::<&mut RigidBody>() {
        rb.force_accumulator = Vec3::ZERO;
        rb.torque_accumulator = Vec3::ZERO;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_free_fall() {
        let mut world = hecs::World::new();

        let entity = world.spawn((
            Transform::from_position(Vec3::new(0.0, 10.0, 0.0)),
            GlobalTransform::default(),
            RigidBody::new_dynamic(1.0),
        ));

        let gravity = Vec3::new(0.0, -9.81, 0.0);
        let dt = 1.0 / 60.0;

        // Simulate 1 second (60 steps)
        for _ in 0..60 {
            apply_gravity(&mut world, gravity);
            integrate_velocities(&mut world, dt);
            integrate_positions(&mut world, dt);
            sync_transforms(&mut world);
            clear_forces(&mut world);
        }

        let transform = world.get::<&Transform>(entity).unwrap();

        // After 1 second of free fall from y=10: y = 10 - 0.5*9.81*1^2 ≈ 5.095
        // With damping and discrete steps, should be somewhere below 10
        assert!(
            transform.position.y < 10.0,
            "Body should have fallen: y = {}",
            transform.position.y
        );
        assert!(
            transform.position.y > 0.0,
            "Body should not have fallen too far in 1 second: y = {}",
            transform.position.y
        );

        // X and Z should be unchanged
        let eps = 1e-5;
        assert!(transform.position.x.abs() < eps);
        assert!(transform.position.z.abs() < eps);
    }

    #[test]
    fn test_static_body_unaffected() {
        let mut world = hecs::World::new();

        let entity = world.spawn((
            Transform::from_position(Vec3::new(0.0, 0.0, 0.0)),
            GlobalTransform::default(),
            RigidBody::new_static(),
        ));

        let gravity = Vec3::new(0.0, -9.81, 0.0);
        let dt = 1.0 / 60.0;

        for _ in 0..60 {
            apply_gravity(&mut world, gravity);
            integrate_velocities(&mut world, dt);
            integrate_positions(&mut world, dt);
            clear_forces(&mut world);
        }

        let transform = world.get::<&Transform>(entity).unwrap();
        assert_eq!(transform.position, Vec3::ZERO);
    }

    #[test]
    fn test_clear_forces() {
        let mut world = hecs::World::new();

        let entity = world.spawn((Transform::identity(), GlobalTransform::default(), {
            let mut rb = RigidBody::new_dynamic(1.0);
            rb.force_accumulator = Vec3::new(10.0, 20.0, 30.0);
            rb.torque_accumulator = Vec3::new(1.0, 2.0, 3.0);
            rb
        }));

        clear_forces(&mut world);

        let rb = world.get::<&RigidBody>(entity).unwrap();
        assert_eq!(rb.force_accumulator, Vec3::ZERO);
        assert_eq!(rb.torque_accumulator, Vec3::ZERO);
    }
}
