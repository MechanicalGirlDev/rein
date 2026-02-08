//! Broadphase collision detection using AABB overlap tests.

use glam::Vec3;

use crate::ecs::components::physics::{Collider, RigidBody, RigidBodyType};
use crate::ecs::components::transform::GlobalTransform;

use super::collider::PhysicsAabb;

/// Sweep-and-prune broadphase (currently O(n^2) pair-wise AABB test).
pub struct SweepAndPrune;

impl Default for SweepAndPrune {
    fn default() -> Self {
        Self
    }
}

impl SweepAndPrune {
    pub fn new() -> Self {
        Self
    }

    /// Find all pairs of entities whose AABBs overlap.
    ///
    /// Only returns pairs where at least one entity is dynamic.
    pub fn find_pairs(&self, world: &hecs::World) -> Vec<(hecs::Entity, hecs::Entity)> {
        // Collect all entities with colliders and their AABBs
        let mut entries: Vec<(hecs::Entity, PhysicsAabb, RigidBodyType)> = Vec::new();

        for (entity, (collider, transform, rb)) in world
            .query::<(&Collider, &GlobalTransform, &RigidBody)>()
            .iter()
        {
            if collider.is_sensor {
                continue;
            }
            // Offset the transform by the collider offset
            let mut adjusted_transform = *transform;
            if collider.offset != Vec3::ZERO {
                adjusted_transform.0 *= glam::Mat4::from_translation(collider.offset);
            }
            let aabb = collider.shape.compute_aabb(&adjusted_transform);
            entries.push((entity, aabb, rb.body_type));
        }

        let mut pairs = Vec::new();

        // O(n^2) brute force - sufficient for small numbers of entities
        for i in 0..entries.len() {
            for j in (i + 1)..entries.len() {
                let (entity_a, aabb_a, type_a) = &entries[i];
                let (entity_b, aabb_b, type_b) = &entries[j];

                // Skip static-static pairs
                if *type_a == RigidBodyType::Static && *type_b == RigidBodyType::Static {
                    continue;
                }

                if aabb_a.overlaps(aabb_b) {
                    pairs.push((*entity_a, *entity_b));
                }
            }
        }

        pairs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::components::physics::{ColliderShape, RigidBody};
    use crate::ecs::components::transform::{GlobalTransform, Transform};
    use glam::Mat4;

    #[test]
    fn test_broadphase_overlapping() {
        let mut world = hecs::World::new();

        // Two overlapping spheres
        let _a = world.spawn((
            Transform::from_position(Vec3::new(0.0, 0.0, 0.0)),
            GlobalTransform(Mat4::IDENTITY),
            RigidBody::new_dynamic(1.0),
            Collider {
                shape: ColliderShape::Sphere { radius: 1.0 },
                offset: Vec3::ZERO,
                is_sensor: false,
            },
        ));

        let _b = world.spawn((
            Transform::from_position(Vec3::new(1.0, 0.0, 0.0)),
            GlobalTransform(Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0))),
            RigidBody::new_dynamic(1.0),
            Collider {
                shape: ColliderShape::Sphere { radius: 1.0 },
                offset: Vec3::ZERO,
                is_sensor: false,
            },
        ));

        let broadphase = SweepAndPrune::new();
        let pairs = broadphase.find_pairs(&world);
        assert_eq!(pairs.len(), 1);
    }

    #[test]
    fn test_broadphase_no_overlap() {
        let mut world = hecs::World::new();

        // Two far apart spheres
        let _a = world.spawn((
            Transform::from_position(Vec3::ZERO),
            GlobalTransform(Mat4::IDENTITY),
            RigidBody::new_dynamic(1.0),
            Collider {
                shape: ColliderShape::Sphere { radius: 0.5 },
                offset: Vec3::ZERO,
                is_sensor: false,
            },
        ));

        let _b = world.spawn((
            Transform::from_position(Vec3::new(10.0, 0.0, 0.0)),
            GlobalTransform(Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0))),
            RigidBody::new_dynamic(1.0),
            Collider {
                shape: ColliderShape::Sphere { radius: 0.5 },
                offset: Vec3::ZERO,
                is_sensor: false,
            },
        ));

        let broadphase = SweepAndPrune::new();
        let pairs = broadphase.find_pairs(&world);
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_broadphase_static_static_skipped() {
        let mut world = hecs::World::new();

        // Two overlapping static bodies - should NOT be returned
        world.spawn((
            Transform::identity(),
            GlobalTransform(Mat4::IDENTITY),
            RigidBody::new_static(),
            Collider {
                shape: ColliderShape::Sphere { radius: 1.0 },
                offset: Vec3::ZERO,
                is_sensor: false,
            },
        ));

        world.spawn((
            Transform::identity(),
            GlobalTransform(Mat4::IDENTITY),
            RigidBody::new_static(),
            Collider {
                shape: ColliderShape::Sphere { radius: 1.0 },
                offset: Vec3::ZERO,
                is_sensor: false,
            },
        ));

        let broadphase = SweepAndPrune::new();
        let pairs = broadphase.find_pairs(&world);
        assert!(pairs.is_empty());
    }
}
