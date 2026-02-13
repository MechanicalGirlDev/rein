//! Object abstractions
//!
//! Provides the Object trait and Gm struct for combining geometry and materials.

use crate::context::WgpuContext;
use crate::renderer::geometry::{Aabb, Geometry};
use crate::renderer::light::Light;
use crate::renderer::material::Material;
use crate::renderer::viewer::Viewer;
use glam::Mat4;

/// Trait for renderable objects.
pub trait Object {
    /// Render the object.
    fn render(
        &self,
        ctx: &WgpuContext,
        viewer: &dyn Viewer,
        lights: &[&dyn Light],
        render_pass: &mut wgpu::RenderPass<'_>,
    );

    /// Get the axis-aligned bounding box in world space.
    fn aabb(&self) -> Aabb;

    /// Get the transform matrix.
    fn transform(&self) -> Mat4;

    /// Set the transform matrix.
    fn set_transform(&mut self, transform: Mat4);
}

/// A renderable object combining geometry and material.
pub struct Gm<G: Geometry, M: Material> {
    /// The geometry.
    pub geometry: G,
    /// The material.
    pub material: M,
    /// The transform matrix.
    pub transform: Mat4,
}

impl<G: Geometry, M: Material> Gm<G, M> {
    /// Create a new Gm.
    pub fn new(geometry: G, material: M) -> Self {
        Self {
            geometry,
            material,
            transform: Mat4::IDENTITY,
        }
    }

    /// Set the transform.
    pub fn with_transform(mut self, transform: Mat4) -> Self {
        self.transform = transform;
        self
    }

    /// Set the position.
    pub fn with_position(mut self, x: f32, y: f32, z: f32) -> Self {
        self.transform = Mat4::from_translation(glam::Vec3::new(x, y, z));
        self
    }

    /// Update the material uniforms.
    pub fn update_material(&self, ctx: &WgpuContext, viewer: &dyn Viewer, lights: &[&dyn Light]) {
        self.material
            .update_uniforms(ctx, viewer, self.transform, lights);
    }
}

impl<G: Geometry, M: Material> Object for Gm<G, M> {
    fn render(
        &self,
        ctx: &WgpuContext,
        viewer: &dyn Viewer,
        lights: &[&dyn Light],
        render_pass: &mut wgpu::RenderPass<'_>,
    ) {
        // Update uniforms
        self.material
            .update_uniforms(ctx, viewer, self.transform, lights);

        // Set pipeline and bind groups
        render_pass.set_pipeline(self.material.pipeline());
        render_pass.set_bind_group(0, self.material.camera_bind_group(), &[]);
        render_pass.set_bind_group(1, self.material.model_bind_group(), &[]);

        // Set vertex buffer
        render_pass.set_vertex_buffer(0, self.geometry.vertex_buffer().slice());

        // Draw
        if let Some(index_buffer) = self.geometry.index_buffer() {
            render_pass.set_index_buffer(index_buffer.slice(), index_buffer.format());
            render_pass.draw_indexed(0..self.geometry.draw_count(), 0, 0..1);
        } else {
            render_pass.draw(0..self.geometry.draw_count(), 0..1);
        }
    }

    fn aabb(&self) -> Aabb {
        let local_aabb = self.geometry.aabb();
        // Transform AABB corners to world space
        let corners = [
            self.transform.transform_point3(local_aabb.min),
            self.transform.transform_point3(glam::Vec3::new(
                local_aabb.max.x,
                local_aabb.min.y,
                local_aabb.min.z,
            )),
            self.transform.transform_point3(glam::Vec3::new(
                local_aabb.min.x,
                local_aabb.max.y,
                local_aabb.min.z,
            )),
            self.transform.transform_point3(glam::Vec3::new(
                local_aabb.min.x,
                local_aabb.min.y,
                local_aabb.max.z,
            )),
            self.transform.transform_point3(glam::Vec3::new(
                local_aabb.max.x,
                local_aabb.max.y,
                local_aabb.min.z,
            )),
            self.transform.transform_point3(glam::Vec3::new(
                local_aabb.max.x,
                local_aabb.min.y,
                local_aabb.max.z,
            )),
            self.transform.transform_point3(glam::Vec3::new(
                local_aabb.min.x,
                local_aabb.max.y,
                local_aabb.max.z,
            )),
            self.transform.transform_point3(local_aabb.max),
        ];
        Aabb::from_points(corners)
    }

    fn transform(&self) -> Mat4 {
        self.transform
    }

    fn set_transform(&mut self, transform: Mat4) {
        self.transform = transform;
    }
}
