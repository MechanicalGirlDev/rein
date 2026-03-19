//! Terrain geometry with height-map based generation
//!
//! Provides terrain mesh generation from a height function with height-based coloring.

use super::{Aabb, Geometry};
use crate::context::WgpuContext;
use crate::core::buffer::{IndexBuffer, RawUniformBuffer, VertexBuffer};
use crate::core::pipeline::{PipelineBuilder, Vertex};
use crate::core::render_states::{BlendState, CullState, DepthState};
use crate::renderer::light::Light;
use crate::renderer::material::traits::{Material, ModelUniform};
use crate::renderer::viewer::{CameraUniform, Viewer};
use glam::{Mat4, Vec3};

/// Level of detail for terrain generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainLod {
    /// High detail: full resolution.
    High,
    /// Medium detail: half resolution.
    Medium,
    /// Low detail: quarter resolution.
    Low,
}

impl TerrainLod {
    /// Get the step size multiplier for this LOD level.
    fn step_multiplier(self) -> u32 {
        match self {
            TerrainLod::High => 1,
            TerrainLod::Medium => 2,
            TerrainLod::Low => 4,
        }
    }
}

/// Terrain uniform data for GPU.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TerrainUniform {
    pub min_height: f32,
    pub max_height: f32,
    pub _padding0: f32,
    pub _padding1: f32,
    pub color_low: [f32; 4],
    pub color_high: [f32; 4],
}

/// A terrain mesh generated from a height function.
///
/// Inspired by three-d's Terrain, this generates a height-map based terrain mesh
/// with height-based coloring and basic LOD support.
pub struct Terrain {
    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,
    draw_count: u32,
    aabb: Aabb,
}

impl Terrain {
    /// Create a new terrain from a height function.
    ///
    /// - `width`: Total width of the terrain in world units.
    /// - `depth`: Total depth of the terrain in world units.
    /// - `resolution`: Number of vertices along each axis.
    /// - `height_fn`: Function that returns height given (x, z) world coordinates.
    /// - `lod`: Level of detail.
    pub fn new(
        ctx: &WgpuContext,
        width: f32,
        depth: f32,
        resolution: u32,
        height_fn: &dyn Fn(f32, f32) -> f32,
        lod: TerrainLod,
        color: [f32; 3],
    ) -> Self {
        let step = lod.step_multiplier();
        let effective_res = (resolution / step).max(2);

        let (vertices, indices, aabb) =
            Self::generate(width, depth, effective_res, height_fn, color);

        let vertex_buffer = VertexBuffer::new(ctx, &vertices, Some("terrain vertices"));
        let index_buffer = IndexBuffer::new_u32(ctx, &indices, Some("terrain indices"));
        let draw_count = indices.len() as u32;

        Self {
            vertex_buffer,
            index_buffer,
            draw_count,
            aabb,
        }
    }

    /// Create terrain from a heightmap array.
    ///
    /// - `width`, `depth`: World-space dimensions.
    /// - `heights`: Row-major heightmap of size `res_x * res_z`.
    /// - `res_x`, `res_z`: Resolution of the heightmap.
    pub fn from_heightmap(
        ctx: &WgpuContext,
        width: f32,
        depth: f32,
        heights: &[f32],
        res_x: u32,
        res_z: u32,
        color: [f32; 3],
    ) -> Self {
        let height_fn = |x: f32, z: f32| -> f32 {
            let u = (x / width + 0.5).clamp(0.0, 1.0);
            let v = (z / depth + 0.5).clamp(0.0, 1.0);
            let ix = ((u * (res_x - 1) as f32) as u32).min(res_x - 1);
            let iz = ((v * (res_z - 1) as f32) as u32).min(res_z - 1);
            heights[(iz * res_x + ix) as usize]
        };

        Self::new(
            ctx,
            width,
            depth,
            res_x.min(res_z),
            &height_fn,
            TerrainLod::High,
            color,
        )
    }

    fn generate(
        width: f32,
        depth: f32,
        resolution: u32,
        height_fn: &dyn Fn(f32, f32) -> f32,
        color: [f32; 3],
    ) -> (Vec<Vertex>, Vec<u32>, Aabb) {
        let mut vertices = Vec::with_capacity((resolution * resolution) as usize);
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        let hw = width / 2.0;
        let hd = depth / 2.0;

        // Generate vertex positions
        for iz in 0..resolution {
            for ix in 0..resolution {
                let u = ix as f32 / (resolution - 1) as f32;
                let v = iz as f32 / (resolution - 1) as f32;

                let x = -hw + u * width;
                let z = -hd + v * depth;
                let y = height_fn(x, z);

                min_y = min_y.min(y);
                max_y = max_y.max(y);

                // Normal will be calculated after all positions are known
                vertices.push(Vertex {
                    position: [x, y, z],
                    normal: [0.0, 1.0, 0.0],
                    color,
                });
            }
        }

        // Calculate normals using central differences
        for iz in 0..resolution {
            for ix in 0..resolution {
                let idx = (iz * resolution + ix) as usize;

                let left = if ix > 0 {
                    vertices[idx - 1].position[1]
                } else {
                    vertices[idx].position[1]
                };
                let right = if ix < resolution - 1 {
                    vertices[idx + 1].position[1]
                } else {
                    vertices[idx].position[1]
                };
                let down = if iz > 0 {
                    vertices[(idx as u32 - resolution) as usize].position[1]
                } else {
                    vertices[idx].position[1]
                };
                let up_val = if iz < resolution - 1 {
                    vertices[(idx as u32 + resolution) as usize].position[1]
                } else {
                    vertices[idx].position[1]
                };

                let dx = width / (resolution - 1) as f32;
                let dz = depth / (resolution - 1) as f32;

                let normal = Vec3::new(
                    (left - right) / (2.0 * dx),
                    1.0,
                    (down - up_val) / (2.0 * dz),
                )
                .normalize();

                vertices[idx].normal = [normal.x, normal.y, normal.z];
            }
        }

        // Generate indices
        let mut indices = Vec::with_capacity(((resolution - 1) * (resolution - 1) * 6) as usize);
        for iz in 0..resolution - 1 {
            for ix in 0..resolution - 1 {
                let top_left = iz * resolution + ix;
                let top_right = top_left + 1;
                let bottom_left = top_left + resolution;
                let bottom_right = bottom_left + 1;

                indices.push(top_left);
                indices.push(bottom_left);
                indices.push(top_right);

                indices.push(top_right);
                indices.push(bottom_left);
                indices.push(bottom_right);
            }
        }

        let aabb = Aabb::new(Vec3::new(-hw, min_y, -hd), Vec3::new(hw, max_y, hd));

        (vertices, indices, aabb)
    }
}

impl Geometry for Terrain {
    fn vertex_buffer(&self) -> &VertexBuffer {
        &self.vertex_buffer
    }

    fn index_buffer(&self) -> Option<&IndexBuffer> {
        Some(&self.index_buffer)
    }

    fn draw_count(&self) -> u32 {
        self.draw_count
    }

    fn aabb(&self) -> Aabb {
        self.aabb
    }
}

/// Material for terrain with height-based coloring.
///
/// Uses a custom shader that blends between a low-altitude color and a high-altitude
/// color based on vertex height.
pub struct TerrainMaterial {
    pipeline: wgpu::RenderPipeline,
    camera_buffer: RawUniformBuffer,
    camera_bind_group: wgpu::BindGroup,
    model_buffer: RawUniformBuffer,
    model_bind_group: wgpu::BindGroup,
    terrain_buffer: RawUniformBuffer,
    terrain_bind_group: wgpu::BindGroup,

    /// Low altitude color RGBA.
    pub color_low: [f32; 4],
    /// High altitude color RGBA.
    pub color_high: [f32; 4],
    /// Minimum height for color mapping.
    pub min_height: f32,
    /// Maximum height for color mapping.
    pub max_height: f32,
}

impl TerrainMaterial {
    /// Create a new terrain material with default colors.
    pub fn new(ctx: &WgpuContext, format: wgpu::TextureFormat) -> anyhow::Result<Self> {
        Self::with_params(
            ctx,
            format,
            [0.2, 0.5, 0.1, 1.0],  // green low
            [0.8, 0.8, 0.85, 1.0], // snow high
            0.0,
            10.0,
        )
    }

    /// Create a new terrain material with custom parameters.
    pub fn with_params(
        ctx: &WgpuContext,
        format: wgpu::TextureFormat,
        color_low: [f32; 4],
        color_high: [f32; 4],
        min_height: f32,
        max_height: f32,
    ) -> anyhow::Result<Self> {
        let shader = include_str!("../../shaders/terrain.wgsl");

        // Camera bind group layout (group 0)
        let camera_bind_group_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("terrain camera bind group layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        // Model bind group layout (group 1)
        let model_bind_group_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("terrain model bind group layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        // Terrain bind group layout (group 2)
        let terrain_bind_group_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("terrain params bind group layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let pipeline = PipelineBuilder::new(ctx)
            .label("terrain material pipeline")
            .shader(shader)
            .vertex_layout(Vertex::layout())
            .bind_group_layout(&camera_bind_group_layout)
            .bind_group_layout(&model_bind_group_layout)
            .bind_group_layout(&terrain_bind_group_layout)
            .color_format(format)
            .depth(DepthState::read_write())
            .blend(BlendState::Opaque)
            .cull(CullState::Back)
            .build()?;

        // Create camera uniform buffer
        let camera_buffer = RawUniformBuffer::new(
            ctx,
            std::mem::size_of::<CameraUniform>() as u64,
            Some("terrain camera uniform"),
        );

        let camera_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("terrain camera bind group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.buffer().as_entire_binding(),
            }],
        });

        // Create model uniform buffer
        let model_buffer = RawUniformBuffer::new(
            ctx,
            std::mem::size_of::<ModelUniform>() as u64,
            Some("terrain model uniform"),
        );

        let model_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("terrain model bind group"),
            layout: &model_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: model_buffer.buffer().as_entire_binding(),
            }],
        });

        // Create terrain uniform buffer
        let terrain_buffer = RawUniformBuffer::new(
            ctx,
            std::mem::size_of::<TerrainUniform>() as u64,
            Some("terrain params uniform"),
        );

        let terrain_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("terrain params bind group"),
            layout: &terrain_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: terrain_buffer.buffer().as_entire_binding(),
            }],
        });

        Ok(Self {
            pipeline,
            camera_buffer,
            camera_bind_group,
            model_buffer,
            model_bind_group,
            terrain_buffer,
            terrain_bind_group,
            color_low,
            color_high,
            min_height,
            max_height,
        })
    }

    /// Get the terrain bind group.
    pub fn terrain_bind_group(&self) -> &wgpu::BindGroup {
        &self.terrain_bind_group
    }
}

impl Material for TerrainMaterial {
    fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    fn camera_bind_group(&self) -> &wgpu::BindGroup {
        &self.camera_bind_group
    }

    fn model_bind_group(&self) -> &wgpu::BindGroup {
        &self.model_bind_group
    }

    fn update_uniforms(
        &self,
        ctx: &WgpuContext,
        viewer: &dyn Viewer,
        model_matrix: Mat4,
        _lights: &[&dyn Light],
    ) {
        let camera_uniform = CameraUniform::from_viewer(viewer);
        self.camera_buffer.write(ctx, &camera_uniform);

        let model_uniform = ModelUniform::from_matrix(model_matrix);
        self.model_buffer.write(ctx, &model_uniform);

        let terrain_uniform = TerrainUniform {
            min_height: self.min_height,
            max_height: self.max_height,
            _padding0: 0.0,
            _padding1: 0.0,
            color_low: self.color_low,
            color_high: self.color_high,
        };
        self.terrain_buffer.write(ctx, &terrain_uniform);
    }
}
