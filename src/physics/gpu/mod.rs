//! GPU-accelerated physics using compute shaders.
//!
//! Offloads broadphase collision detection and velocity/position integration
//! to the GPU for improved performance with large numbers of rigid bodies.
//!
//! # Strategy
//!
//! | Stage | CPU/GPU | Reason |
//! |-------|---------|--------|
//! | Velocity integration | GPU | Highly parallel (each body independent) |
//! | Position integration | GPU | Same |
//! | Broadphase AABB | GPU | Parallelizes O(n^2) pair testing |
//! | Narrowphase (GJK/EPA) | CPU | Branch-heavy, poor GPU fit |
//! | Contact solver | CPU | Sequential impulse is inherently serial |
//!
//! GPU offload is used when body count >= [`GPU_BODY_THRESHOLD`].

use glam::Vec3;

use crate::compute::{compute_workgroup_count, read_buffer_sync, ComputeDispatcher};
use crate::context::WgpuContext;
use crate::core::{ComputePipelineBuilder, StorageBuffer};
use crate::ecs::components::physics::{Collider, RigidBody, RigidBodyType};
use crate::ecs::components::transform::GlobalTransform;

/// Minimum number of bodies before GPU offload is used.
pub const GPU_BODY_THRESHOLD: usize = 256;

/// Maximum collision pairs the GPU broadphase can output.
const MAX_PAIRS: u32 = 65536;

/// Workgroup size matching the WGSL shaders.
const WORKGROUP_SIZE: u32 = 64;

/// GPU AABB data layout matching the broadphase shader.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuAabb {
    pub min: [f32; 3],
    pub entity_id: u32,
    pub max: [f32; 3],
    pub _padding: u32,
}

/// GPU collision pair output.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CollisionPair {
    pub entity_a: u32,
    pub entity_b: u32,
}

/// GPU body data layout matching the integrate shader.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuBody {
    pub position: [f32; 3],
    pub body_type: u32,
    pub linear_velocity: [f32; 3],
    pub mass: f32,
    pub angular_velocity: [f32; 3],
    pub gravity_scale: f32,
    pub force_accumulator: [f32; 3],
    pub linear_damping: f32,
    pub torque_accumulator: [f32; 3],
    pub angular_damping: f32,
    pub inertia_diag: [f32; 3],
    pub _padding: f32,
    pub rotation: [f32; 4],
}

/// GPU broadphase parameters.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct BroadphaseParams {
    num_bodies: u32,
    max_pairs: u32,
    _pad0: u32,
    _pad1: u32,
}

/// GPU integrate parameters.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct IntegrateParams {
    num_bodies: u32,
    dt: f32,
    gravity_x: f32,
    gravity_y: f32,
    gravity_z: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

/// GPU-accelerated physics engine.
///
/// Manages GPU buffers and compute pipelines for broadphase collision
/// detection and rigid body integration.
pub struct GpuPhysics {
    // Broadphase resources
    broadphase_pipeline: wgpu::ComputePipeline,
    aabb_buffer: StorageBuffer,
    pair_buffer: StorageBuffer,
    pair_count_buffer: StorageBuffer,
    broadphase_data_layout: wgpu::BindGroupLayout,
    broadphase_params_layout: wgpu::BindGroupLayout,

    // Integration resources
    integrate_vel_pipeline: wgpu::ComputePipeline,
    integrate_pos_pipeline: wgpu::ComputePipeline,
    body_buffer: StorageBuffer,
    integrate_data_layout: wgpu::BindGroupLayout,
    integrate_params_layout: wgpu::BindGroupLayout,

    // Capacity tracking
    max_bodies: usize,
}

impl GpuPhysics {
    /// Create a new GPU physics instance.
    ///
    /// Allocates GPU buffers and compiles compute shaders.
    pub fn new(ctx: &WgpuContext, initial_capacity: usize) -> anyhow::Result<Self> {
        let max_bodies = initial_capacity.max(256);

        // --- Broadphase pipeline ---
        let broadphase_data_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("broadphase data layout"),
                    entries: &[
                        // AABBs (read)
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        // Pairs (read_write)
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        // Pair count (read_write)
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let broadphase_params_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("broadphase params layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let broadphase_shader = include_str!("../../shaders/compute/broadphase.wgsl");
        let broadphase_pipeline = ComputePipelineBuilder::new(ctx)
            .label("broadphase compute")
            .shader(broadphase_shader)
            .entry_point("cs_broadphase")
            .bind_group_layout(&broadphase_data_layout)
            .bind_group_layout(&broadphase_params_layout)
            .build()?;

        // Broadphase buffers
        let aabb_size = (max_bodies * std::mem::size_of::<GpuAabb>()) as u64;
        let aabb_buffer = StorageBuffer::new(ctx, aabb_size, Some("aabb buffer"));

        let pair_size = (MAX_PAIRS as usize * std::mem::size_of::<CollisionPair>()) as u64;
        let pair_buffer = StorageBuffer::new(ctx, pair_size, Some("pair buffer"));

        // pair_count: single u32, atomic
        let pair_count_buffer = StorageBuffer::new(ctx, 4, Some("pair count buffer"));

        // --- Integration pipelines ---
        let integrate_data_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("integrate data layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let integrate_params_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("integrate params layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let integrate_shader = include_str!("../../shaders/compute/integrate.wgsl");
        let integrate_vel_pipeline = ComputePipelineBuilder::new(ctx)
            .label("integrate velocities compute")
            .shader(integrate_shader)
            .entry_point("cs_integrate_velocities")
            .bind_group_layout(&integrate_data_layout)
            .bind_group_layout(&integrate_params_layout)
            .build()?;

        let integrate_pos_pipeline = ComputePipelineBuilder::new(ctx)
            .label("integrate positions compute")
            .shader(integrate_shader)
            .entry_point("cs_integrate_positions")
            .bind_group_layout(&integrate_data_layout)
            .bind_group_layout(&integrate_params_layout)
            .build()?;

        // Body buffer
        let body_size = (max_bodies * std::mem::size_of::<GpuBody>()) as u64;
        let body_buffer = StorageBuffer::new(ctx, body_size, Some("body buffer"));

        Ok(Self {
            broadphase_pipeline,
            aabb_buffer,
            pair_buffer,
            pair_count_buffer,
            broadphase_data_layout,
            broadphase_params_layout,
            integrate_vel_pipeline,
            integrate_pos_pipeline,
            body_buffer,
            integrate_data_layout,
            integrate_params_layout,
            max_bodies,
        })
    }

    /// Upload AABB data from the ECS world to the GPU for broadphase.
    ///
    /// Returns the number of bodies uploaded, and the entity ID mapping.
    pub fn upload_aabbs(&self, ctx: &WgpuContext, world: &hecs::World) -> (u32, Vec<hecs::Entity>) {
        let mut aabbs = Vec::new();
        let mut entity_map = Vec::new();

        for (entity, (collider, transform, rb)) in world
            .query::<(&Collider, &GlobalTransform, &RigidBody)>()
            .iter()
        {
            if collider.is_sensor {
                continue;
            }
            // Skip static-only pairs will be handled by skipping in results
            let mut adjusted_transform = *transform;
            if collider.offset != Vec3::ZERO {
                adjusted_transform.0 *= glam::Mat4::from_translation(collider.offset);
            }
            let aabb = collider.shape.compute_aabb(&adjusted_transform);

            let idx = aabbs.len() as u32;
            aabbs.push(GpuAabb {
                min: aabb.min.into(),
                entity_id: idx,
                max: aabb.max.into(),
                _padding: rb.body_type as u32,
            });
            entity_map.push(entity);
        }

        if !aabbs.is_empty() && aabbs.len() <= self.max_bodies {
            self.aabb_buffer.write(ctx, &aabbs);
        }

        // Reset pair count to 0
        self.pair_count_buffer.write(ctx, &[0u32]);

        (aabbs.len() as u32, entity_map)
    }

    /// Dispatch the GPU broadphase compute shader.
    pub fn dispatch_broadphase(&self, ctx: &WgpuContext, body_count: u32) {
        if body_count == 0 {
            return;
        }

        let params = BroadphaseParams {
            num_bodies: body_count,
            max_pairs: MAX_PAIRS,
            _pad0: 0,
            _pad1: 0,
        };

        // Create params uniform buffer
        use wgpu::util::DeviceExt;
        let params_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("broadphase params"),
                contents: bytemuck::bytes_of(&params),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let data_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("broadphase data"),
            layout: &self.broadphase_data_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.aabb_buffer.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.pair_buffer.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.pair_count_buffer.buffer().as_entire_binding(),
                },
            ],
        });

        let params_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("broadphase params"),
            layout: &self.broadphase_params_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buffer.as_entire_binding(),
            }],
        });

        let dispatcher = ComputeDispatcher::new(ctx);
        let workgroups = compute_workgroup_count(body_count, WORKGROUP_SIZE);
        dispatcher.dispatch(
            &self.broadphase_pipeline,
            &[&data_bind_group, &params_bind_group],
            [workgroups, 1, 1],
            Some("broadphase"),
        );
    }

    /// Read back broadphase collision pairs from the GPU.
    ///
    /// Returns pairs as (entity_index_a, entity_index_b). Use the entity map
    /// from [`upload_aabbs`] to convert indices to `hecs::Entity`.
    pub fn readback_pairs(&self, ctx: &WgpuContext) -> Vec<CollisionPair> {
        // Read the pair count first
        let count_data: Vec<u32> = read_buffer_sync(ctx, self.pair_count_buffer.buffer(), 4);
        let pair_count = count_data.first().copied().unwrap_or(0).min(MAX_PAIRS);

        if pair_count == 0 {
            return Vec::new();
        }

        let read_size = (pair_count as usize * std::mem::size_of::<CollisionPair>()) as u64;
        let pairs: Vec<CollisionPair> = read_buffer_sync(ctx, self.pair_buffer.buffer(), read_size);

        pairs.into_iter().take(pair_count as usize).collect()
    }

    /// Upload rigid body data for GPU integration.
    ///
    /// Returns the body count and entity mapping.
    pub fn upload_bodies(
        &self,
        ctx: &WgpuContext,
        world: &hecs::World,
    ) -> (u32, Vec<hecs::Entity>) {
        let mut bodies = Vec::new();
        let mut entity_map = Vec::new();

        for (entity, (rb, transform)) in world
            .query::<(&RigidBody, &crate::ecs::components::transform::Transform)>()
            .iter()
        {
            let body_type = match rb.body_type {
                RigidBodyType::Dynamic => 0u32,
                RigidBodyType::Static => 1,
                RigidBodyType::Kinematic => 2,
            };

            bodies.push(GpuBody {
                position: transform.position.into(),
                body_type,
                linear_velocity: rb.linear_velocity.into(),
                mass: rb.mass,
                angular_velocity: rb.angular_velocity.into(),
                gravity_scale: rb.gravity_scale,
                force_accumulator: rb.force_accumulator.into(),
                linear_damping: rb.linear_damping,
                torque_accumulator: rb.torque_accumulator.into(),
                angular_damping: rb.angular_damping,
                inertia_diag: [
                    rb.inertia_tensor[0],
                    rb.inertia_tensor[4],
                    rb.inertia_tensor[8],
                ],
                _padding: 0.0,
                rotation: [
                    transform.rotation.x,
                    transform.rotation.y,
                    transform.rotation.z,
                    transform.rotation.w,
                ],
            });
            entity_map.push(entity);
        }

        if !bodies.is_empty() && bodies.len() <= self.max_bodies {
            self.body_buffer.write(ctx, &bodies);
        }

        (bodies.len() as u32, entity_map)
    }

    /// Dispatch GPU velocity integration.
    pub fn dispatch_integrate_velocities(
        &self,
        ctx: &WgpuContext,
        body_count: u32,
        dt: f32,
        gravity: Vec3,
    ) {
        if body_count == 0 {
            return;
        }

        let params = IntegrateParams {
            num_bodies: body_count,
            dt,
            gravity_x: gravity.x,
            gravity_y: gravity.y,
            gravity_z: gravity.z,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        };

        self.dispatch_integrate(ctx, &self.integrate_vel_pipeline, body_count, &params);
    }

    /// Dispatch GPU position integration.
    pub fn dispatch_integrate_positions(&self, ctx: &WgpuContext, body_count: u32, dt: f32) {
        if body_count == 0 {
            return;
        }

        let params = IntegrateParams {
            num_bodies: body_count,
            dt,
            gravity_x: 0.0,
            gravity_y: 0.0,
            gravity_z: 0.0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        };

        self.dispatch_integrate(ctx, &self.integrate_pos_pipeline, body_count, &params);
    }

    fn dispatch_integrate(
        &self,
        ctx: &WgpuContext,
        pipeline: &wgpu::ComputePipeline,
        body_count: u32,
        params: &IntegrateParams,
    ) {
        use wgpu::util::DeviceExt;
        let params_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("integrate params"),
                contents: bytemuck::bytes_of(params),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let data_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("integrate data"),
            layout: &self.integrate_data_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.body_buffer.buffer().as_entire_binding(),
            }],
        });

        let params_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("integrate params"),
            layout: &self.integrate_params_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buffer.as_entire_binding(),
            }],
        });

        let dispatcher = ComputeDispatcher::new(ctx);
        let workgroups = compute_workgroup_count(body_count, WORKGROUP_SIZE);
        dispatcher.dispatch(
            pipeline,
            &[&data_bind_group, &params_bind_group],
            [workgroups, 1, 1],
            Some("integrate"),
        );
    }

    /// Read back integrated body data and sync to the ECS world.
    pub fn download_bodies(
        &self,
        ctx: &WgpuContext,
        world: &mut hecs::World,
        body_count: u32,
        entity_map: &[hecs::Entity],
    ) {
        if body_count == 0 {
            return;
        }

        let read_size = (body_count as usize * std::mem::size_of::<GpuBody>()) as u64;
        let gpu_bodies: Vec<GpuBody> = read_buffer_sync(ctx, self.body_buffer.buffer(), read_size);

        for (gpu_body, entity) in gpu_bodies.iter().zip(entity_map.iter()) {
            if let Ok((rb, transform)) = world.query_one_mut::<(
                &mut RigidBody,
                &mut crate::ecs::components::transform::Transform,
            )>(*entity)
            {
                // Sync back velocities and forces
                rb.linear_velocity = Vec3::from(gpu_body.linear_velocity);
                rb.angular_velocity = Vec3::from(gpu_body.angular_velocity);
                rb.force_accumulator = Vec3::from(gpu_body.force_accumulator);
                rb.torque_accumulator = Vec3::from(gpu_body.torque_accumulator);

                // Sync back position and rotation
                transform.position = Vec3::from(gpu_body.position);
                transform.rotation = glam::Quat::from_xyzw(
                    gpu_body.rotation[0],
                    gpu_body.rotation[1],
                    gpu_body.rotation[2],
                    gpu_body.rotation[3],
                );
            }
        }
    }

    /// Check if GPU offload should be used based on body count.
    pub fn should_use_gpu(body_count: usize) -> bool {
        body_count >= GPU_BODY_THRESHOLD
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_aabb_layout() {
        assert_eq!(std::mem::size_of::<GpuAabb>(), 32);
    }

    #[test]
    fn test_collision_pair_layout() {
        assert_eq!(std::mem::size_of::<CollisionPair>(), 8);
    }

    #[test]
    fn test_gpu_body_layout() {
        assert_eq!(std::mem::size_of::<GpuBody>(), 112);
    }

    #[test]
    fn test_broadphase_params_layout() {
        assert_eq!(std::mem::size_of::<BroadphaseParams>(), 16);
    }

    #[test]
    fn test_integrate_params_layout() {
        assert_eq!(std::mem::size_of::<IntegrateParams>(), 32);
    }

    #[test]
    fn test_gpu_threshold() {
        assert!(!GpuPhysics::should_use_gpu(100));
        assert!(GpuPhysics::should_use_gpu(256));
        assert!(GpuPhysics::should_use_gpu(1000));
    }
}
