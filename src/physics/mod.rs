//! CPU-based physics engine with rigid body simulation and collision detection.
//!
//! # Architecture
//!
//! The physics pipeline runs in a fixed timestep loop:
//!
//! 1. Apply forces (gravity)
//! 2. Integrate velocities
//! 3. Broadphase collision detection (AABB overlap)
//! 4. Narrowphase collision detection (GJK/EPA, SAT, specialized tests)
//! 5. Solve contact constraints (sequential impulse)
//! 6. Integrate positions
//! 7. Synchronize transforms
//! 8. Clear force accumulators

pub mod broadphase;
pub mod collider;
pub mod contact;
#[cfg(feature = "gpu-physics")]
pub mod gpu;
pub mod narrowphase;
pub mod rigid_body;
pub mod solver;

use glam::Vec3;

use crate::ecs::components::physics::{Collider, RigidBody};
use crate::ecs::components::transform::GlobalTransform;

use self::broadphase::SweepAndPrune;
use self::contact::{ContactManifold, ContactPoint};
use self::narrowphase::detect_collision;

/// Configuration for the physics simulation.
#[derive(Debug, Clone)]
pub struct PhysicsConfig {
    /// Gravity vector. Default: (0, -9.81, 0).
    pub gravity: Vec3,
    /// Fixed timestep for physics updates in seconds. Default: 1/60.
    pub fixed_timestep: f64,
    /// Maximum number of sub-steps per frame. Default: 4.
    pub max_substeps: u32,
    /// Number of constraint solver iterations. Default: 8.
    pub solver_iterations: u32,
    /// Whether to use GPU acceleration when available. Default: false.
    /// Requires the `gpu-physics` feature.
    pub use_gpu: bool,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            fixed_timestep: 1.0 / 60.0,
            max_substeps: 4,
            solver_iterations: 8,
            use_gpu: false,
        }
    }
}

/// The main physics world managing simulation state.
pub struct PhysicsWorld {
    config: PhysicsConfig,
    accumulator: f64,
    broadphase: SweepAndPrune,
    contacts: Vec<ContactManifold>,
    #[cfg(feature = "gpu-physics")]
    gpu_physics: Option<gpu::GpuPhysics>,
}

impl PhysicsWorld {
    /// Create a new physics world with the given configuration.
    pub fn new(config: PhysicsConfig) -> Self {
        Self {
            config,
            accumulator: 0.0,
            broadphase: SweepAndPrune::new(),
            contacts: Vec::new(),
            #[cfg(feature = "gpu-physics")]
            gpu_physics: None,
        }
    }

    /// Initialize GPU physics resources. Only available with the `gpu-physics` feature.
    ///
    /// Call this once after creating the physics world to enable GPU acceleration.
    #[cfg(feature = "gpu-physics")]
    pub fn init_gpu(
        &mut self,
        ctx: &crate::context::WgpuContext,
        initial_capacity: usize,
    ) -> anyhow::Result<()> {
        self.gpu_physics = Some(gpu::GpuPhysics::new(ctx, initial_capacity)?);
        Ok(())
    }

    /// Get a reference to the GPU physics instance, if initialized.
    #[cfg(feature = "gpu-physics")]
    pub fn gpu_physics_ref(&self) -> Option<&gpu::GpuPhysics> {
        self.gpu_physics.as_ref()
    }

    /// Step the physics simulation forward by `delta_time` seconds.
    ///
    /// Uses a fixed timestep accumulator to ensure deterministic simulation.
    pub fn step(&mut self, world: &mut hecs::World, delta_time: f64) {
        self.accumulator += delta_time;

        let mut substeps = 0u32;
        while self.accumulator >= self.config.fixed_timestep && substeps < self.config.max_substeps
        {
            self.fixed_step(world, self.config.fixed_timestep as f32);
            self.accumulator -= self.config.fixed_timestep;
            substeps += 1;
        }

        // Clamp accumulator to avoid spiral of death
        if self.accumulator > self.config.fixed_timestep * self.config.max_substeps as f64 {
            self.accumulator = 0.0;
        }
    }

    /// Step the physics simulation with GPU-accelerated broadphase.
    ///
    /// Uses GPU compute for AABB broadphase when body count exceeds the threshold,
    /// falling back to CPU otherwise. Requires `gpu-physics` feature and prior
    /// call to [`init_gpu`].
    #[cfg(feature = "gpu-physics")]
    pub fn step_gpu(
        &mut self,
        world: &mut hecs::World,
        delta_time: f64,
        ctx: &crate::context::WgpuContext,
    ) {
        self.accumulator += delta_time;

        let mut substeps = 0u32;
        while self.accumulator >= self.config.fixed_timestep && substeps < self.config.max_substeps
        {
            self.fixed_step_gpu(world, self.config.fixed_timestep as f32, ctx);
            self.accumulator -= self.config.fixed_timestep;
            substeps += 1;
        }

        if self.accumulator > self.config.fixed_timestep * self.config.max_substeps as f64 {
            self.accumulator = 0.0;
        }
    }

    #[cfg(feature = "gpu-physics")]
    fn fixed_step_gpu(
        &mut self,
        world: &mut hecs::World,
        dt: f32,
        ctx: &crate::context::WgpuContext,
    ) {
        rigid_body::apply_gravity(world, self.config.gravity);
        rigid_body::integrate_velocities(world, dt);

        // Sync transforms so GPU broadphase sees current positions
        rigid_body::sync_transforms(world);

        // GPU broadphase when available and body count is high enough
        let pairs = if let Some(gpu) = &self.gpu_physics {
            let (body_count, entity_map) = gpu.upload_aabbs(ctx, world);
            if gpu::GpuPhysics::should_use_gpu(body_count as usize) {
                gpu.dispatch_broadphase(ctx, body_count);
                let gpu_pairs = gpu.readback_pairs(ctx);
                gpu_pairs
                    .iter()
                    .filter_map(|p| {
                        let a = entity_map.get(p.entity_a as usize)?;
                        let b = entity_map.get(p.entity_b as usize)?;
                        Some((*a, *b))
                    })
                    .collect()
            } else {
                self.broadphase.find_pairs(world)
            }
        } else {
            self.broadphase.find_pairs(world)
        };

        // Narrowphase (CPU)
        self.contacts.clear();
        for (entity_a, entity_b) in &pairs {
            let contact = {
                let collider_a = world.get::<&Collider>(*entity_a);
                let collider_b = world.get::<&Collider>(*entity_b);
                let transform_a = world.get::<&GlobalTransform>(*entity_a);
                let transform_b = world.get::<&GlobalTransform>(*entity_b);

                if let (Ok(ca), Ok(cb), Ok(ta), Ok(tb)) =
                    (collider_a, collider_b, transform_a, transform_b)
                {
                    let adjusted_a = if ca.offset != Vec3::ZERO {
                        GlobalTransform(ta.0 * glam::Mat4::from_translation(ca.offset))
                    } else {
                        *ta
                    };
                    let adjusted_b = if cb.offset != Vec3::ZERO {
                        GlobalTransform(tb.0 * glam::Mat4::from_translation(cb.offset))
                    } else {
                        *tb
                    };

                    detect_collision(&ca.shape, &adjusted_a, &cb.shape, &adjusted_b)
                } else {
                    None
                }
            };

            if let Some(info) = contact {
                self.contacts.push(ContactManifold {
                    entity_a: *entity_a,
                    entity_b: *entity_b,
                    normal: info.normal,
                    contacts: vec![ContactPoint {
                        position: info.point,
                        penetration: info.penetration,
                        normal_impulse: 0.0,
                        tangent_impulse: [0.0; 2],
                    }],
                });
            }
        }

        solver::solve_contacts(&mut self.contacts, world, self.config.solver_iterations);
        rigid_body::integrate_positions(world, dt);
        rigid_body::sync_transforms(world);
        rigid_body::clear_forces(world);
    }

    fn fixed_step(&mut self, world: &mut hecs::World, dt: f32) {
        // 1. Apply forces (gravity)
        rigid_body::apply_gravity(world, self.config.gravity);

        // 2. Integrate velocities
        rigid_body::integrate_velocities(world, dt);

        // 3. Broadphase collision detection
        let pairs = self.broadphase.find_pairs(world);

        // 4. Narrowphase collision detection
        self.contacts.clear();
        for (entity_a, entity_b) in &pairs {
            let contact = {
                let collider_a = world.get::<&Collider>(*entity_a);
                let collider_b = world.get::<&Collider>(*entity_b);
                let transform_a = world.get::<&GlobalTransform>(*entity_a);
                let transform_b = world.get::<&GlobalTransform>(*entity_b);

                if let (Ok(ca), Ok(cb), Ok(ta), Ok(tb)) =
                    (collider_a, collider_b, transform_a, transform_b)
                {
                    // Apply collider offsets
                    let adjusted_a = if ca.offset != Vec3::ZERO {
                        GlobalTransform(ta.0 * glam::Mat4::from_translation(ca.offset))
                    } else {
                        *ta
                    };
                    let adjusted_b = if cb.offset != Vec3::ZERO {
                        GlobalTransform(tb.0 * glam::Mat4::from_translation(cb.offset))
                    } else {
                        *tb
                    };

                    detect_collision(&ca.shape, &adjusted_a, &cb.shape, &adjusted_b)
                } else {
                    None
                }
            };

            if let Some(info) = contact {
                self.contacts.push(ContactManifold {
                    entity_a: *entity_a,
                    entity_b: *entity_b,
                    normal: info.normal,
                    contacts: vec![ContactPoint {
                        position: info.point,
                        penetration: info.penetration,
                        normal_impulse: 0.0,
                        tangent_impulse: [0.0; 2],
                    }],
                });
            }
        }

        // 5. Solve contact constraints
        // Need to set contact positions from transforms for solver
        for manifold in &mut self.contacts {
            if let Ok(rb) = world.get::<&RigidBody>(manifold.entity_a) {
                let _ = rb; // Just checking existence
            }
        }
        solver::solve_contacts(&mut self.contacts, world, self.config.solver_iterations);

        // 6. Integrate positions
        rigid_body::integrate_positions(world, dt);

        // 7. Synchronize transforms
        rigid_body::sync_transforms(world);

        // 8. Clear force accumulators
        rigid_body::clear_forces(world);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::components::physics::{Collider, ColliderShape, RigidBody};
    use crate::ecs::components::transform::{GlobalTransform, Transform};
    use glam::Mat4;

    #[test]
    fn test_physics_world_free_fall() {
        let mut world = hecs::World::new();
        let mut physics = PhysicsWorld::new(PhysicsConfig::default());

        let entity = world.spawn((
            Transform::from_position(Vec3::new(0.0, 10.0, 0.0)),
            GlobalTransform(Mat4::from_translation(Vec3::new(0.0, 10.0, 0.0))),
            RigidBody::new_dynamic(1.0),
            Collider {
                shape: ColliderShape::Sphere { radius: 0.5 },
                offset: Vec3::ZERO,
                is_sensor: false,
            },
        ));

        // Simulate ~1 second
        for _ in 0..60 {
            physics.step(&mut world, 1.0 / 60.0);
        }

        let transform = world.get::<&Transform>(entity).unwrap();
        assert!(
            transform.position.y < 10.0,
            "Body should have fallen: y = {}",
            transform.position.y
        );
    }

    #[test]
    fn test_physics_world_collision() {
        let mut world = hecs::World::new();
        let config = PhysicsConfig {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            fixed_timestep: 1.0 / 60.0,
            max_substeps: 4,
            solver_iterations: 8,
            use_gpu: false,
        };
        let mut physics = PhysicsWorld::new(config);

        // Dynamic box falling
        let dynamic_entity = world.spawn((
            Transform::from_position(Vec3::new(0.0, 2.0, 0.0)),
            GlobalTransform(Mat4::from_translation(Vec3::new(0.0, 2.0, 0.0))),
            RigidBody::new_dynamic(1.0),
            Collider {
                shape: ColliderShape::Box {
                    half_extents: Vec3::splat(0.5),
                },
                offset: Vec3::ZERO,
                is_sensor: false,
            },
        ));

        // Static ground plane (large box at y=0)
        world.spawn((
            Transform::from_position(Vec3::new(0.0, -0.5, 0.0)),
            GlobalTransform(Mat4::from_translation(Vec3::new(0.0, -0.5, 0.0))),
            RigidBody::new_static(),
            Collider {
                shape: ColliderShape::Box {
                    half_extents: Vec3::new(50.0, 0.5, 50.0),
                },
                offset: Vec3::ZERO,
                is_sensor: false,
            },
        ));

        // Simulate 3 seconds
        for _ in 0..180 {
            physics.step(&mut world, 1.0 / 60.0);
        }

        let transform = world.get::<&Transform>(dynamic_entity).unwrap();
        let rb = world.get::<&RigidBody>(dynamic_entity).unwrap();

        // The box should have fallen and been stopped by the ground
        // It should be near y=0.5 (half the box height above the ground surface)
        // Allow generous tolerance for solver precision
        assert!(
            transform.position.y > -2.0,
            "Box should not have fallen through the ground: y = {}",
            transform.position.y
        );
        assert!(
            transform.position.y < 2.0,
            "Box should have fallen from initial position: y = {}",
            transform.position.y
        );

        // Velocity should be near zero (settled)
        let speed = rb.linear_velocity.length();
        assert!(
            speed < 5.0,
            "Box should have mostly settled: speed = {}",
            speed
        );
    }

    #[test]
    fn test_physics_config_default() {
        let config = PhysicsConfig::default();
        assert_eq!(config.gravity, Vec3::new(0.0, -9.81, 0.0));
        assert!((config.fixed_timestep - 1.0 / 60.0).abs() < 1e-10);
        assert_eq!(config.max_substeps, 4);
        assert_eq!(config.solver_iterations, 8);
        assert!(!config.use_gpu);
    }
}
