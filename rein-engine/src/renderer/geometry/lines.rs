//! Line geometry for rendering lines and line strips
//!
//! Provides geometry types for line rendering.

use super::{Aabb, Geometry};
use crate::context::WgpuContext;
use crate::core::buffer::{IndexBuffer, VertexBuffer};
use crate::core::vertex::VertexPC;
use glam::Vec3;

/// A collection of line segments.
/// Each pair of consecutive vertices forms a line segment.
pub struct Lines {
    vertex_buffer: VertexBuffer,
    vertex_count: u32,
    aabb: Aabb,
}

impl Lines {
    /// Create lines from a list of segments.
    /// Each segment is defined by a start and end position with colors.
    pub fn new(
        ctx: &WgpuContext,
        segments: &[(Vec3, [f32; 4], Vec3, [f32; 4])],
        label: Option<&str>,
    ) -> Self {
        let mut vertices = Vec::with_capacity(segments.len() * 2);
        let mut positions = Vec::with_capacity(segments.len() * 2);

        for (start, start_color, end, end_color) in segments {
            vertices.push(VertexPC::new(start.to_array(), *start_color));
            vertices.push(VertexPC::new(end.to_array(), *end_color));
            positions.push(*start);
            positions.push(*end);
        }

        let vertex_buffer = VertexBuffer::new(ctx, &vertices, label);
        let aabb = Aabb::from_points(positions);

        Self {
            vertex_buffer,
            vertex_count: vertices.len() as u32,
            aabb,
        }
    }

    /// Create lines from vertex positions and colors.
    /// Vertices are paired: [v0, v1] forms line 0, [v2, v3] forms line 1, etc.
    pub fn from_vertices(ctx: &WgpuContext, vertices: &[VertexPC], label: Option<&str>) -> Self {
        let vertex_buffer = VertexBuffer::new(ctx, vertices, label);
        let aabb = Aabb::from_points(vertices.iter().map(|v| Vec3::from_array(v.position)));

        Self {
            vertex_buffer,
            vertex_count: vertices.len() as u32,
            aabb,
        }
    }

    /// Create lines with a single color.
    pub fn with_color(
        ctx: &WgpuContext,
        segments: &[(Vec3, Vec3)],
        color: [f32; 4],
        label: Option<&str>,
    ) -> Self {
        let mut vertices = Vec::with_capacity(segments.len() * 2);
        let mut positions = Vec::with_capacity(segments.len() * 2);

        for (start, end) in segments {
            vertices.push(VertexPC::new(start.to_array(), color));
            vertices.push(VertexPC::new(end.to_array(), color));
            positions.push(*start);
            positions.push(*end);
        }

        let vertex_buffer = VertexBuffer::new(ctx, &vertices, label);
        let aabb = Aabb::from_points(positions);

        Self {
            vertex_buffer,
            vertex_count: vertices.len() as u32,
            aabb,
        }
    }

    /// Get the vertex layout for line rendering.
    pub fn vertex_layout() -> wgpu::VertexBufferLayout<'static> {
        VertexPC::layout()
    }
}

impl Geometry for Lines {
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
        self.aabb
    }
}

/// A connected line strip where each vertex connects to the next.
pub struct LineStrip {
    vertex_buffer: VertexBuffer,
    vertex_count: u32,
    aabb: Aabb,
}

impl LineStrip {
    /// Create a line strip from positions and colors.
    pub fn new(ctx: &WgpuContext, points: &[(Vec3, [f32; 4])], label: Option<&str>) -> Self {
        let vertices: Vec<VertexPC> = points
            .iter()
            .map(|(pos, color)| VertexPC::new(pos.to_array(), *color))
            .collect();

        let vertex_buffer = VertexBuffer::new(ctx, &vertices, label);
        let aabb = Aabb::from_points(points.iter().map(|(p, _)| *p));

        Self {
            vertex_buffer,
            vertex_count: vertices.len() as u32,
            aabb,
        }
    }

    /// Create a line strip with a single color.
    pub fn with_color(
        ctx: &WgpuContext,
        points: &[Vec3],
        color: [f32; 4],
        label: Option<&str>,
    ) -> Self {
        let vertices: Vec<VertexPC> = points
            .iter()
            .map(|pos| VertexPC::new(pos.to_array(), color))
            .collect();

        let vertex_buffer = VertexBuffer::new(ctx, &vertices, label);
        let aabb = Aabb::from_points(points.iter().copied());

        Self {
            vertex_buffer,
            vertex_count: vertices.len() as u32,
            aabb,
        }
    }

    /// Create a circle in the XZ plane.
    pub fn circle(ctx: &WgpuContext, radius: f32, segments: u32, color: [f32; 4]) -> Self {
        let mut points = Vec::with_capacity(segments as usize + 1);

        for i in 0..=segments {
            let theta = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
            points.push(Vec3::new(radius * theta.cos(), 0.0, radius * theta.sin()));
        }

        Self::with_color(ctx, &points, color, Some("circle"))
    }

    /// Get the vertex layout for line strip rendering.
    pub fn vertex_layout() -> wgpu::VertexBufferLayout<'static> {
        VertexPC::layout()
    }
}

impl Geometry for LineStrip {
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
        self.aabb
    }
}
