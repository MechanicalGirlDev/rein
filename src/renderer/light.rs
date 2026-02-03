//! Light types
//!
//! Provides various light types for 3D rendering.

use glam::Vec3;

/// Light type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightType {
    Ambient,
    Directional,
    Point,
    Spot,
}

/// Attenuation factors for point and spot lights.
#[derive(Debug, Clone, Copy)]
pub struct Attenuation {
    /// Constant attenuation factor (default: 1.0).
    pub constant: f32,
    /// Linear attenuation factor.
    pub linear: f32,
    /// Quadratic attenuation factor.
    pub quadratic: f32,
}

impl Attenuation {
    /// Create a new attenuation.
    pub fn new(constant: f32, linear: f32, quadratic: f32) -> Self {
        Self {
            constant,
            linear,
            quadratic,
        }
    }

    /// Default attenuation for a ~50 unit range.
    pub fn range_50() -> Self {
        Self::new(1.0, 0.09, 0.032)
    }

    /// Default attenuation for a ~20 unit range.
    pub fn range_20() -> Self {
        Self::new(1.0, 0.22, 0.20)
    }

    /// Default attenuation for a ~100 unit range.
    pub fn range_100() -> Self {
        Self::new(1.0, 0.045, 0.0075)
    }

    /// No attenuation (constant intensity).
    pub fn none() -> Self {
        Self::new(1.0, 0.0, 0.0)
    }

    /// Convert to array.
    pub fn to_array(&self) -> [f32; 3] {
        [self.constant, self.linear, self.quadratic]
    }
}

impl Default for Attenuation {
    fn default() -> Self {
        Self::range_50()
    }
}

/// Light uniform data for GPU.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniforms {
    /// Light direction or position (w = 0 for directional, 1 for point).
    pub direction_or_position: [f32; 4],
    /// Light color and intensity (rgb = color, a = intensity).
    pub color_intensity: [f32; 4],
    /// Attenuation factors for point lights (constant, linear, quadratic, unused).
    pub attenuation: [f32; 4],
}

/// Trait for light sources.
pub trait Light {
    /// Get the light type.
    fn light_type(&self) -> LightType;

    /// Get the light uniforms for GPU.
    fn uniforms(&self) -> LightUniforms;
}

/// Ambient light that illuminates all surfaces equally.
#[derive(Debug, Clone)]
pub struct AmbientLight {
    /// Light intensity (0.0 - 1.0).
    pub intensity: f32,
    /// Light color (RGB).
    pub color: [f32; 3],
}

impl AmbientLight {
    /// Create a new ambient light.
    pub fn new(intensity: f32, color: [f32; 3]) -> Self {
        Self { intensity, color }
    }

    /// Create a white ambient light.
    pub fn white(intensity: f32) -> Self {
        Self::new(intensity, [1.0, 1.0, 1.0])
    }
}

impl Light for AmbientLight {
    fn light_type(&self) -> LightType {
        LightType::Ambient
    }

    fn uniforms(&self) -> LightUniforms {
        LightUniforms {
            direction_or_position: [0.0, 0.0, 0.0, 0.0],
            color_intensity: [self.color[0], self.color[1], self.color[2], self.intensity],
            attenuation: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

impl Default for AmbientLight {
    fn default() -> Self {
        Self::white(0.3)
    }
}

/// Directional light that illuminates from a direction.
#[derive(Debug, Clone)]
pub struct DirectionalLight {
    /// Light intensity (0.0 - 1.0+).
    pub intensity: f32,
    /// Light color (RGB).
    pub color: [f32; 3],
    /// Light direction (normalized).
    pub direction: Vec3,
}

impl DirectionalLight {
    /// Create a new directional light.
    pub fn new(intensity: f32, color: [f32; 3], direction: Vec3) -> Self {
        Self {
            intensity,
            color,
            direction: direction.normalize(),
        }
    }

    /// Create a white directional light.
    pub fn white(intensity: f32, direction: Vec3) -> Self {
        Self::new(intensity, [1.0, 1.0, 1.0], direction)
    }
}

impl Light for DirectionalLight {
    fn light_type(&self) -> LightType {
        LightType::Directional
    }

    fn uniforms(&self) -> LightUniforms {
        LightUniforms {
            direction_or_position: [self.direction.x, self.direction.y, self.direction.z, 0.0],
            color_intensity: [self.color[0], self.color[1], self.color[2], self.intensity],
            attenuation: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self::white(1.0, Vec3::new(-0.3, -1.0, -0.5))
    }
}

/// Point light that illuminates from a position with attenuation.
#[derive(Debug, Clone)]
pub struct PointLight {
    /// Light intensity.
    pub intensity: f32,
    /// Light color (RGB).
    pub color: [f32; 3],
    /// Light position.
    pub position: Vec3,
    /// Attenuation factors (constant, linear, quadratic).
    pub attenuation: [f32; 3],
}

impl PointLight {
    /// Create a new point light.
    pub fn new(intensity: f32, color: [f32; 3], position: Vec3, attenuation: [f32; 3]) -> Self {
        Self {
            intensity,
            color,
            position,
            attenuation,
        }
    }

    /// Create a white point light with default attenuation.
    pub fn white(intensity: f32, position: Vec3) -> Self {
        Self::new(intensity, [1.0, 1.0, 1.0], position, [1.0, 0.09, 0.032])
    }
}

impl Light for PointLight {
    fn light_type(&self) -> LightType {
        LightType::Point
    }

    fn uniforms(&self) -> LightUniforms {
        LightUniforms {
            direction_or_position: [self.position.x, self.position.y, self.position.z, 1.0],
            color_intensity: [self.color[0], self.color[1], self.color[2], self.intensity],
            attenuation: [
                self.attenuation[0],
                self.attenuation[1],
                self.attenuation[2],
                0.0,
            ],
        }
    }
}

impl Default for PointLight {
    fn default() -> Self {
        Self::white(1.0, Vec3::new(0.0, 2.0, 0.0))
    }
}

/// Spot light with cone-shaped illumination.
#[derive(Debug, Clone)]
pub struct SpotLight {
    /// Light intensity.
    pub intensity: f32,
    /// Light color (RGB).
    pub color: [f32; 3],
    /// Light position.
    pub position: Vec3,
    /// Light direction (normalized).
    pub direction: Vec3,
    /// Inner cone angle (radians) - full intensity within this cone.
    pub inner_angle: f32,
    /// Outer cone angle (radians) - light fades to zero at this angle.
    pub outer_angle: f32,
    /// Attenuation factors.
    pub attenuation: Attenuation,
}

impl SpotLight {
    /// Create a new spot light.
    pub fn new(
        intensity: f32,
        color: [f32; 3],
        position: Vec3,
        direction: Vec3,
        inner_angle: f32,
        outer_angle: f32,
        attenuation: Attenuation,
    ) -> Self {
        Self {
            intensity,
            color,
            position,
            direction: direction.normalize(),
            inner_angle,
            outer_angle,
            attenuation,
        }
    }

    /// Create a white spot light with default parameters.
    pub fn white(intensity: f32, position: Vec3, direction: Vec3) -> Self {
        Self::new(
            intensity,
            [1.0, 1.0, 1.0],
            position,
            direction,
            std::f32::consts::PI / 6.0, // 30 degrees
            std::f32::consts::PI / 4.0, // 45 degrees
            Attenuation::default(),
        )
    }

    /// Create a spot light with custom cone angles (in degrees).
    pub fn with_cone(
        intensity: f32,
        color: [f32; 3],
        position: Vec3,
        direction: Vec3,
        inner_degrees: f32,
        outer_degrees: f32,
    ) -> Self {
        Self::new(
            intensity,
            color,
            position,
            direction,
            inner_degrees.to_radians(),
            outer_degrees.to_radians(),
            Attenuation::default(),
        )
    }
}

impl Light for SpotLight {
    fn light_type(&self) -> LightType {
        LightType::Spot
    }

    fn uniforms(&self) -> LightUniforms {
        LightUniforms {
            direction_or_position: [self.position.x, self.position.y, self.position.z, 1.0],
            color_intensity: [self.color[0], self.color[1], self.color[2], self.intensity],
            attenuation: [
                self.attenuation.constant,
                self.attenuation.linear,
                self.attenuation.quadratic,
                0.0,
            ],
        }
    }
}

impl Default for SpotLight {
    fn default() -> Self {
        Self::white(1.0, Vec3::new(0.0, 5.0, 0.0), Vec3::new(0.0, -1.0, 0.0))
    }
}

/// Extended spot light uniform data for GPU.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpotLightUniform {
    /// Light position.
    pub position: [f32; 4],
    /// Light direction.
    pub direction: [f32; 4],
    /// Light color and intensity (rgb = color, a = intensity).
    pub color_intensity: [f32; 4],
    /// Cone angles and attenuation (inner_cos, outer_cos, linear, quadratic).
    pub cone_attenuation: [f32; 4],
}

impl SpotLight {
    /// Get extended uniforms for GPU (includes direction and cone data).
    pub fn extended_uniforms(&self) -> SpotLightUniform {
        SpotLightUniform {
            position: [self.position.x, self.position.y, self.position.z, 1.0],
            direction: [self.direction.x, self.direction.y, self.direction.z, 0.0],
            color_intensity: [self.color[0], self.color[1], self.color[2], self.intensity],
            cone_attenuation: [
                self.inner_angle.cos(),
                self.outer_angle.cos(),
                self.attenuation.linear,
                self.attenuation.quadratic,
            ],
        }
    }
}
