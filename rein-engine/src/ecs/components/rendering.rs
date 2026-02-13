//! Rendering components for ECS entities.

use std::sync::Arc;

use glam::Vec3;

use crate::renderer::light::LightType;
use crate::renderer::viewer::Camera;

use super::super::bridge::MaterialResource;
use crate::renderer::geometry::Geometry;

/// Shared mesh resource handle.
pub struct MeshHandle(pub Arc<dyn Geometry + Send + Sync>);

/// Shared material resource handle.
pub struct MaterialHandle(pub Arc<dyn MaterialResource>);

/// Mesh + material rendering component. ECS equivalent of `Gm<G, M>`.
pub struct MeshRenderer {
    pub mesh: MeshHandle,
    pub material: MaterialHandle,
    pub visible: bool,
    pub cast_shadow: bool,
    pub receive_shadow: bool,
}

/// Light component.
pub struct LightComponent {
    pub light_type: LightType,
    pub color: Vec3,
    pub intensity: f32,
}

/// Camera component.
pub struct CameraComponent {
    pub camera: Camera,
    pub active: bool,
}

/// Marker for entities subject to frustum culling.
pub struct FrustumCullable;

/// Marker for entities that passed the culling test (updated each frame).
pub struct Visible;
