//! Shadow mapping module
//!
//! Provides shadow map generation and sampling for realistic shadows.

mod directional;

pub use directional::DirectionalShadow;

use crate::context::WgpuContext;
use crate::core::DepthTexture;
use glam::Mat4;

/// Shadow map configuration.
#[derive(Debug, Clone, Copy)]
pub struct ShadowConfig {
    /// Shadow map resolution (width and height).
    pub resolution: u32,
    /// Shadow bias to prevent shadow acne.
    pub bias: f32,
    /// Normal offset bias.
    pub normal_bias: f32,
    /// Enable PCF (Percentage Closer Filtering).
    pub pcf_enabled: bool,
    /// PCF kernel size (1 = 3x3, 2 = 5x5, etc.).
    pub pcf_radius: u32,
}

impl Default for ShadowConfig {
    fn default() -> Self {
        Self {
            resolution: 2048,
            bias: 0.005,
            normal_bias: 0.02,
            pcf_enabled: true,
            pcf_radius: 1,
        }
    }
}

/// Shadow map for a single light source.
pub struct ShadowMap {
    /// Depth texture for shadow map.
    pub depth_texture: DepthTexture,
    /// Light space matrix (view * projection).
    pub light_matrix: Mat4,
    /// Shadow configuration.
    pub config: ShadowConfig,
}

impl ShadowMap {
    /// Create a new shadow map.
    pub fn new(ctx: &WgpuContext, config: ShadowConfig) -> Self {
        let depth_texture = DepthTexture::new(
            ctx,
            config.resolution,
            config.resolution,
            Some("shadow map"),
        );

        Self {
            depth_texture,
            light_matrix: Mat4::IDENTITY,
            config,
        }
    }

    /// Get the depth texture view for shadow sampling.
    pub fn depth_view(&self) -> &wgpu::TextureView {
        self.depth_texture.view()
    }

    /// Get the shadow uniform data for shaders.
    pub fn uniform(&self) -> ShadowUniform {
        ShadowUniform {
            light_matrix: self.light_matrix.to_cols_array_2d(),
            bias: self.config.bias,
            normal_bias: self.config.normal_bias,
            pcf_radius: self.config.pcf_radius as f32,
            shadow_map_size: self.config.resolution as f32,
        }
    }
}

/// Shadow uniform data for GPU.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowUniform {
    /// Light space matrix (world to light clip space).
    pub light_matrix: [[f32; 4]; 4],
    /// Shadow bias.
    pub bias: f32,
    /// Normal offset bias.
    pub normal_bias: f32,
    /// PCF kernel radius.
    pub pcf_radius: f32,
    /// Shadow map size (for texel size calculation).
    pub shadow_map_size: f32,
}
