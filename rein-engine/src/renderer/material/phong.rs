//! Phong material with ambient, diffuse, and specular lighting

use super::color::ColorMaterial;
use super::traits::Material;
use crate::context::WgpuContext;
use crate::renderer::light::Light;
use crate::renderer::viewer::Viewer;
use glam::Mat4;

/// Phong material with ambient, diffuse, and specular components.
pub struct PhongMaterial {
    inner: ColorMaterial,
    /// Ambient color.
    pub ambient: [f32; 3],
    /// Diffuse color.
    pub diffuse: [f32; 3],
    /// Specular color.
    pub specular: [f32; 3],
    /// Shininess exponent.
    pub shininess: f32,
}

impl PhongMaterial {
    /// Create a new Phong material.
    pub fn new(
        ctx: &WgpuContext,
        format: wgpu::TextureFormat,
        ambient: [f32; 3],
        diffuse: [f32; 3],
        specular: [f32; 3],
        shininess: f32,
    ) -> anyhow::Result<Self> {
        let inner = ColorMaterial::new(ctx, format)?;
        Ok(Self {
            inner,
            ambient,
            diffuse,
            specular,
            shininess,
        })
    }
}

impl Material for PhongMaterial {
    fn pipeline(&self) -> &wgpu::RenderPipeline {
        self.inner.pipeline()
    }

    fn camera_bind_group(&self) -> &wgpu::BindGroup {
        self.inner.camera_bind_group()
    }

    fn model_bind_group(&self) -> &wgpu::BindGroup {
        self.inner.model_bind_group()
    }

    fn update_uniforms(
        &self,
        ctx: &WgpuContext,
        viewer: &dyn Viewer,
        model_matrix: Mat4,
        lights: &[&dyn Light],
    ) {
        self.inner
            .update_uniforms(ctx, viewer, model_matrix, lights);
    }
}
