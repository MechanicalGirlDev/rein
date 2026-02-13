//! High-level rendering abstractions
//!
//! This module provides three-d style rendering with cameras, materials, and objects.

#[cfg(feature = "window")]
pub mod control;
pub mod culling;
pub mod geometry;
pub mod light;
pub mod material;
pub mod object;
pub mod shadow;
pub mod viewer;

#[cfg(feature = "window")]
pub use control::{FirstPersonControl, FlyControl, OrbitControl};
pub use culling::{Frustum, FrustumCuller, Intersection, Plane};
pub use geometry::{Aabb, Axes, BoundingBoxMesh, Geometry, InstancedMesh, LineStrip, Lines, Mesh};
pub use light::{AmbientLight, Attenuation, DirectionalLight, Light, PointLight, SpotLight};
pub use material::{
    ColorMaterial, DepthMaterial, GridMaterial, LineMaterial, Material, ModelUniform,
    NormalMaterial, PbrMaterial, PhongMaterial, UnlitMaterial,
};
pub use object::{Gm, Object};
pub use shadow::{DirectionalShadow, ShadowConfig, ShadowMap, ShadowUniform};
pub use viewer::{Camera, Projection, Viewer};
