//! Material trait and common types

use crate::context::WgpuContext;
use crate::renderer::light::Light;
use crate::renderer::viewer::Viewer;
use glam::Mat4;

/// Trait for materials that control surface appearance.
pub trait Material {
    /// Get the render pipeline.
    fn pipeline(&self) -> &wgpu::RenderPipeline;

    /// Get the camera bind group.
    fn camera_bind_group(&self) -> &wgpu::BindGroup;

    /// Get the model bind group.
    fn model_bind_group(&self) -> &wgpu::BindGroup;

    /// Update uniforms before rendering.
    fn update_uniforms(
        &self,
        ctx: &WgpuContext,
        viewer: &dyn Viewer,
        model_matrix: Mat4,
        lights: &[&dyn Light],
    );
}

/// Model uniform data for GPU.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelUniform {
    pub model: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 4]; 4],
}

impl ModelUniform {
    pub fn from_matrix(model: Mat4) -> Self {
        let normal_matrix = model.inverse().transpose();
        Self {
            model: model.to_cols_array_2d(),
            normal_matrix: normal_matrix.to_cols_array_2d(),
        }
    }
}
