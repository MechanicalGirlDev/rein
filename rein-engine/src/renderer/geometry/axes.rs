//! Coordinate axes gizmo
//!
//! Provides a visual representation of coordinate axes (X=red, Y=green, Z=blue).

use super::{Aabb, Geometry};
use crate::context::WgpuContext;
use crate::core::buffer::{IndexBuffer, VertexBuffer};
use crate::core::vertex::VertexPC;
use glam::Vec3;

/// Coordinate axes gizmo with X (red), Y (green), Z (blue) axes.
pub struct Axes {
    vertex_buffer: VertexBuffer,
    vertex_count: u32,
    size: f32,
}

impl Axes {
    /// X-axis color (red)
    pub const X_COLOR: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
    /// Y-axis color (green)
    pub const Y_COLOR: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
    /// Z-axis color (blue)
    pub const Z_COLOR: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

    /// Create axes with the given size (length of each axis).
    pub fn new(ctx: &WgpuContext, size: f32) -> Self {
        let origin = [0.0, 0.0, 0.0];

        let vertices = vec![
            // X axis (red)
            VertexPC::new(origin, Self::X_COLOR),
            VertexPC::new([size, 0.0, 0.0], Self::X_COLOR),
            // Y axis (green)
            VertexPC::new(origin, Self::Y_COLOR),
            VertexPC::new([0.0, size, 0.0], Self::Y_COLOR),
            // Z axis (blue)
            VertexPC::new(origin, Self::Z_COLOR),
            VertexPC::new([0.0, 0.0, size], Self::Z_COLOR),
        ];

        let vertex_buffer = VertexBuffer::new(ctx, &vertices, Some("axes"));

        Self {
            vertex_buffer,
            vertex_count: 6,
            size,
        }
    }

    /// Create axes with arrowheads.
    pub fn with_arrows(ctx: &WgpuContext, size: f32, arrow_size: f32) -> Self {
        let origin = [0.0, 0.0, 0.0];
        let arrow_ratio = arrow_size / size;

        let vertices = vec![
            // X axis (red)
            VertexPC::new(origin, Self::X_COLOR),
            VertexPC::new([size, 0.0, 0.0], Self::X_COLOR),
            // X arrowhead
            VertexPC::new([size, 0.0, 0.0], Self::X_COLOR),
            VertexPC::new(
                [size - arrow_size, arrow_ratio * size * 0.3, 0.0],
                Self::X_COLOR,
            ),
            VertexPC::new([size, 0.0, 0.0], Self::X_COLOR),
            VertexPC::new(
                [size - arrow_size, -arrow_ratio * size * 0.3, 0.0],
                Self::X_COLOR,
            ),
            VertexPC::new([size, 0.0, 0.0], Self::X_COLOR),
            VertexPC::new(
                [size - arrow_size, 0.0, arrow_ratio * size * 0.3],
                Self::X_COLOR,
            ),
            VertexPC::new([size, 0.0, 0.0], Self::X_COLOR),
            VertexPC::new(
                [size - arrow_size, 0.0, -arrow_ratio * size * 0.3],
                Self::X_COLOR,
            ),
            // Y axis (green)
            VertexPC::new(origin, Self::Y_COLOR),
            VertexPC::new([0.0, size, 0.0], Self::Y_COLOR),
            // Y arrowhead
            VertexPC::new([0.0, size, 0.0], Self::Y_COLOR),
            VertexPC::new(
                [arrow_ratio * size * 0.3, size - arrow_size, 0.0],
                Self::Y_COLOR,
            ),
            VertexPC::new([0.0, size, 0.0], Self::Y_COLOR),
            VertexPC::new(
                [-arrow_ratio * size * 0.3, size - arrow_size, 0.0],
                Self::Y_COLOR,
            ),
            VertexPC::new([0.0, size, 0.0], Self::Y_COLOR),
            VertexPC::new(
                [0.0, size - arrow_size, arrow_ratio * size * 0.3],
                Self::Y_COLOR,
            ),
            VertexPC::new([0.0, size, 0.0], Self::Y_COLOR),
            VertexPC::new(
                [0.0, size - arrow_size, -arrow_ratio * size * 0.3],
                Self::Y_COLOR,
            ),
            // Z axis (blue)
            VertexPC::new(origin, Self::Z_COLOR),
            VertexPC::new([0.0, 0.0, size], Self::Z_COLOR),
            // Z arrowhead
            VertexPC::new([0.0, 0.0, size], Self::Z_COLOR),
            VertexPC::new(
                [arrow_ratio * size * 0.3, 0.0, size - arrow_size],
                Self::Z_COLOR,
            ),
            VertexPC::new([0.0, 0.0, size], Self::Z_COLOR),
            VertexPC::new(
                [-arrow_ratio * size * 0.3, 0.0, size - arrow_size],
                Self::Z_COLOR,
            ),
            VertexPC::new([0.0, 0.0, size], Self::Z_COLOR),
            VertexPC::new(
                [0.0, arrow_ratio * size * 0.3, size - arrow_size],
                Self::Z_COLOR,
            ),
            VertexPC::new([0.0, 0.0, size], Self::Z_COLOR),
            VertexPC::new(
                [0.0, -arrow_ratio * size * 0.3, size - arrow_size],
                Self::Z_COLOR,
            ),
        ];

        let vertex_buffer = VertexBuffer::new(ctx, &vertices, Some("axes with arrows"));

        Self {
            vertex_buffer,
            vertex_count: vertices.len() as u32,
            size,
        }
    }

    /// Get the size of the axes.
    pub fn size(&self) -> f32 {
        self.size
    }

    /// Get the vertex layout for axes rendering.
    pub fn vertex_layout() -> wgpu::VertexBufferLayout<'static> {
        VertexPC::layout()
    }
}

impl Geometry for Axes {
    fn vertex_buffer(&self) -> &VertexBuffer {
        &self.vertex_buffer
    }

    fn index_buffer(&self) -> Option<&IndexBuffer> {
        None
    }

    fn draw_count(&self) -> u32 {
        self.vertex_count
    }

    fn aabb(&self) -> Aabb {
        Aabb::new(Vec3::ZERO, Vec3::splat(self.size))
    }
}
