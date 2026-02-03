//! Fullscreen quad for post-processing effects

use crate::context::WgpuContext;
use crate::core::buffer::VertexBuffer;
use crate::core::vertex::VertexPC;

/// A fullscreen quad for rendering post-processing effects.
pub struct FullscreenQuad {
    vertex_buffer: VertexBuffer,
}

impl FullscreenQuad {
    /// Create a new fullscreen quad.
    pub fn new(ctx: &WgpuContext) -> Self {
        // Fullscreen triangle (more efficient than quad)
        // Uses clip-space coordinates, UV computed in shader
        let vertices = vec![
            VertexPC::new([-1.0, -1.0, 0.0], [0.0, 1.0, 0.0, 0.0]), // UV stored in color for simplicity
            VertexPC::new([3.0, -1.0, 0.0], [2.0, 1.0, 0.0, 0.0]),
            VertexPC::new([-1.0, 3.0, 0.0], [0.0, -1.0, 0.0, 0.0]),
        ];

        let vertex_buffer = VertexBuffer::new(ctx, &vertices, Some("fullscreen quad"));

        Self { vertex_buffer }
    }

    /// Get the vertex buffer.
    pub fn vertex_buffer(&self) -> &VertexBuffer {
        &self.vertex_buffer
    }

    /// Draw the fullscreen quad.
    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice());
        render_pass.draw(0..3, 0..1);
    }
}
