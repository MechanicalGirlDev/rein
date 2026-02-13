//! Bounding box visualization
//!
//! Provides wireframe rendering of axis-aligned bounding boxes.

use super::{Aabb, Geometry};
use crate::context::WgpuContext;
use crate::core::buffer::{IndexBuffer, VertexBuffer};
use crate::core::vertex::VertexPC;
use glam::Vec3;

/// Wireframe bounding box mesh for visualization.
pub struct BoundingBoxMesh {
    vertex_buffer: VertexBuffer,
    aabb: Aabb,
}

impl BoundingBoxMesh {
    /// Create a wireframe bounding box from an AABB.
    pub fn new(ctx: &WgpuContext, aabb: Aabb, color: [f32; 4]) -> Self {
        let vertices = Self::generate_vertices(&aabb, color);
        let vertex_buffer = VertexBuffer::new(ctx, &vertices, Some("bounding box"));

        Self {
            vertex_buffer,
            aabb,
        }
    }

    /// Create a wireframe bounding box from min/max points.
    pub fn from_min_max(ctx: &WgpuContext, min: Vec3, max: Vec3, color: [f32; 4]) -> Self {
        Self::new(ctx, Aabb::new(min, max), color)
    }

    /// Create a wireframe unit cube centered at origin.
    pub fn unit_cube(ctx: &WgpuContext, color: [f32; 4]) -> Self {
        Self::from_min_max(ctx, Vec3::splat(-0.5), Vec3::splat(0.5), color)
    }

    fn generate_vertices(aabb: &Aabb, color: [f32; 4]) -> Vec<VertexPC> {
        let min = aabb.min;
        let max = aabb.max;

        // 8 corners of the box
        let corners = [
            Vec3::new(min.x, min.y, min.z), // 0: ---
            Vec3::new(max.x, min.y, min.z), // 1: +--
            Vec3::new(min.x, max.y, min.z), // 2: -+-
            Vec3::new(max.x, max.y, min.z), // 3: ++-
            Vec3::new(min.x, min.y, max.z), // 4: --+
            Vec3::new(max.x, min.y, max.z), // 5: +-+
            Vec3::new(min.x, max.y, max.z), // 6: -++
            Vec3::new(max.x, max.y, max.z), // 7: +++
        ];

        // 12 edges of the box (each edge is 2 vertices)
        let edges = [
            // Bottom face
            (0, 1),
            (1, 3),
            (3, 2),
            (2, 0),
            // Top face
            (4, 5),
            (5, 7),
            (7, 6),
            (6, 4),
            // Vertical edges
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7),
        ];

        let mut vertices = Vec::with_capacity(24);
        for (i, j) in edges {
            vertices.push(VertexPC::new(corners[i].to_array(), color));
            vertices.push(VertexPC::new(corners[j].to_array(), color));
        }

        vertices
    }

    /// Get the vertex layout for bounding box rendering.
    pub fn vertex_layout() -> wgpu::VertexBufferLayout<'static> {
        VertexPC::layout()
    }
}

impl Geometry for BoundingBoxMesh {
    fn vertex_buffer(&self) -> &VertexBuffer {
        &self.vertex_buffer
    }

    fn index_buffer(&self) -> Option<&IndexBuffer> {
        None
    }

    fn draw_count(&self) -> u32 {
        24 // 12 edges * 2 vertices
    }

    fn aabb(&self) -> Aabb {
        self.aabb
    }
}
