//! 2D rectangle geometry
//!
//! Provides a rectangle geometry for rendering 2D rectangles in 3D space.

use super::{Aabb, Geometry};
use crate::context::WgpuContext;
use crate::core::buffer::{IndexBuffer, VertexBuffer};
use crate::core::pipeline::Vertex;
use glam::Vec3;

/// A 2D rectangle geometry rendered in the XZ plane.
pub struct Rectangle {
    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,
    draw_count: u32,
    aabb: Aabb,
    width: f32,
    height: f32,
}

impl Rectangle {
    /// Create a new rectangle with the given width and height.
    pub fn new(ctx: &WgpuContext, width: f32, height: f32, color: [f32; 3]) -> Self {
        let (vertices, indices) = Self::generate(width, height, color);
        let vertex_buffer = VertexBuffer::new(ctx, &vertices, Some("rectangle vertices"));
        let index_buffer = IndexBuffer::new_u32(ctx, &indices, Some("rectangle indices"));
        let draw_count = indices.len() as u32;
        let aabb = Aabb::new(
            Vec3::new(-width / 2.0, 0.0, -height / 2.0),
            Vec3::new(width / 2.0, 0.0, height / 2.0),
        );

        Self {
            vertex_buffer,
            index_buffer,
            draw_count,
            aabb,
            width,
            height,
        }
    }

    /// Get the width.
    pub fn width(&self) -> f32 {
        self.width
    }

    /// Get the height.
    pub fn height(&self) -> f32 {
        self.height
    }

    fn generate(width: f32, height: f32, color: [f32; 3]) -> (Vec<Vertex>, Vec<u32>) {
        let hw = width / 2.0;
        let hh = height / 2.0;

        let vertices = vec![
            Vertex {
                position: [-hw, 0.0, -hh],
                normal: [0.0, 1.0, 0.0],
                color,
            },
            Vertex {
                position: [hw, 0.0, -hh],
                normal: [0.0, 1.0, 0.0],
                color,
            },
            Vertex {
                position: [hw, 0.0, hh],
                normal: [0.0, 1.0, 0.0],
                color,
            },
            Vertex {
                position: [-hw, 0.0, hh],
                normal: [0.0, 1.0, 0.0],
                color,
            },
        ];

        let indices = vec![0, 1, 2, 0, 2, 3];

        (vertices, indices)
    }
}

impl Geometry for Rectangle {
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
