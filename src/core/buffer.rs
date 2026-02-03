//! GPU buffer abstractions
//!
//! Provides typed wrappers for vertex, index, and uniform buffers.

use crate::context::WgpuContext;
use bytemuck::{Pod, Zeroable};
use std::marker::PhantomData;

/// A GPU buffer containing vertex data.
pub struct VertexBuffer {
    pub(crate) buffer: wgpu::Buffer,
    pub(crate) count: u32,
    pub(crate) stride: u64,
}

impl VertexBuffer {
    /// Create a new vertex buffer from a slice of vertices.
    pub fn new<V: Pod + Zeroable>(ctx: &WgpuContext, vertices: &[V], label: Option<&str>) -> Self {
        use wgpu::util::DeviceExt;
        let buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label,
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        Self {
            buffer,
            count: vertices.len() as u32,
            stride: std::mem::size_of::<V>() as u64,
        }
    }

    /// Get the raw wgpu buffer.
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    /// Get the number of vertices.
    pub fn count(&self) -> u32 {
        self.count
    }

    /// Get the stride (size of one vertex in bytes).
    pub fn stride(&self) -> u64 {
        self.stride
    }

    /// Create a buffer slice for the entire buffer.
    pub fn slice(&self) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(..)
    }
}

/// A GPU buffer containing index data.
pub struct IndexBuffer {
    pub(crate) buffer: wgpu::Buffer,
    pub(crate) count: u32,
    pub(crate) format: wgpu::IndexFormat,
}

impl IndexBuffer {
    /// Create a new index buffer from u16 indices.
    pub fn new_u16(ctx: &WgpuContext, indices: &[u16], label: Option<&str>) -> Self {
        use wgpu::util::DeviceExt;
        let buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label,
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        Self {
            buffer,
            count: indices.len() as u32,
            format: wgpu::IndexFormat::Uint16,
        }
    }

    /// Create a new index buffer from u32 indices.
    pub fn new_u32(ctx: &WgpuContext, indices: &[u32], label: Option<&str>) -> Self {
        use wgpu::util::DeviceExt;
        let buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label,
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        Self {
            buffer,
            count: indices.len() as u32,
            format: wgpu::IndexFormat::Uint32,
        }
    }

    /// Get the raw wgpu buffer.
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    /// Get the number of indices.
    pub fn count(&self) -> u32 {
        self.count
    }

    /// Get the index format (Uint16 or Uint32).
    pub fn format(&self) -> wgpu::IndexFormat {
        self.format
    }

    /// Create a buffer slice for the entire buffer.
    pub fn slice(&self) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(..)
    }
}

/// A typed GPU uniform buffer.
pub struct UniformBuffer<T> {
    pub(crate) buffer: wgpu::Buffer,
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) bind_group: wgpu::BindGroup,
    _marker: PhantomData<T>,
}

impl<T: Pod + Zeroable> UniformBuffer<T> {
    /// Create a new uniform buffer with initial data.
    pub fn new(ctx: &WgpuContext, data: &T, binding: u32, label: Option<&str>) -> Self {
        use wgpu::util::DeviceExt;

        let buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label,
                contents: bytemuck::bytes_of(data),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: label.map(|l| format!("{} layout", l)).as_deref(),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: label.map(|l| format!("{} bind group", l)).as_deref(),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            buffer,
            bind_group_layout,
            bind_group,
            _marker: PhantomData,
        }
    }

    /// Update the buffer contents.
    pub fn update(&self, ctx: &WgpuContext, data: &T) {
        ctx.queue
            .write_buffer(&self.buffer, 0, bytemuck::bytes_of(data));
    }

    /// Get the raw wgpu buffer.
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    /// Get the bind group layout.
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    /// Get the bind group.
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

/// Raw uniform buffer without type information (for dynamic usage).
pub struct RawUniformBuffer {
    pub(crate) buffer: wgpu::Buffer,
    pub(crate) size: u64,
}

impl RawUniformBuffer {
    /// Create a new raw uniform buffer with specified size.
    pub fn new(ctx: &WgpuContext, size: u64, label: Option<&str>) -> Self {
        let buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label,
            size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self { buffer, size }
    }

    /// Write data to the buffer.
    pub fn write<T: Pod>(&self, ctx: &WgpuContext, data: &T) {
        ctx.queue
            .write_buffer(&self.buffer, 0, bytemuck::bytes_of(data));
    }

    /// Get the raw wgpu buffer.
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    /// Get the buffer size.
    pub fn size(&self) -> u64 {
        self.size
    }
}
