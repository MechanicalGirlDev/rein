//! 2D circle geometry
//!
//! Provides a circle geometry for rendering 2D circles in 3D space.

use super::{Aabb, Geometry};
use crate::context::WgpuContext;
use crate::core::buffer::{IndexBuffer, VertexBuffer};
use crate::core::pipeline::Vertex;
use glam::Vec3;

/// A 2D circle geometry rendered in the XZ plane.
pub struct Circle {
    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,
    draw_count: u32,
    aabb: Aabb,
    radius: f32,
    segments: u32,
}

impl Circle {
    /// Create a new circle with the given radius and number of segments.
    pub fn new(ctx: &WgpuContext, radius: f32, segments: u32, color: [f32; 3]) -> Self {
        let (vertices, indices) = Self::generate(radius, segments, color);
        let vertex_buffer = VertexBuffer::new(ctx, &vertices, Some("circle vertices"));
        let index_buffer = IndexBuffer::new_u32(ctx, &indices, Some("circle indices"));
        let draw_count = indices.len() as u32;
        let aabb = Aabb::new(
            Vec3::new(-radius, 0.0, -radius),
            Vec3::new(radius, 0.0, radius),
        );

        Self {
            vertex_buffer,
            index_buffer,
            draw_count,
            aabb,
            radius,
            segments,
        }
    }

    /// Get the radius.
    pub fn radius(&self) -> f32 {
        self.radius
    }

    /// Get the number of segments.
    pub fn segments(&self) -> u32 {
        self.segments
    }

    fn generate(radius: f32, segments: u32, color: [f32; 3]) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::with_capacity(segments as usize + 1);
        let mut indices = Vec::with_capacity(segments as usize * 3);

        // Center vertex
        vertices.push(Vertex {
            position: [0.0, 0.0, 0.0],
            normal: [0.0, 1.0, 0.0],
            color,
        });

        // Edge vertices
        for i in 0..segments {
            let theta = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
            let x = radius * theta.cos();
            let z = radius * theta.sin();

            vertices.push(Vertex {
                position: [x, 0.0, z],
                normal: [0.0, 1.0, 0.0],
                color,
            });
        }

        // Triangle fan indices
        for i in 0..segments {
            indices.push(0); // center
            indices.push(1 + i);
            indices.push(1 + (i + 1) % segments);
        }

        (vertices, indices)
    }
}

impl Geometry for Circle {
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
