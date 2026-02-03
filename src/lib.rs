//! Rein 3D Rendering Library
//!
//! A three-d style 3D rendering library built on wgpu.
//!
//! # Architecture
//!
//! The library is organized into layers:
//!
//! 1. **context** - Core wgpu wrapper (Device, Queue)
//! 2. **core** - Mid-level abstractions (buffers, textures, pipelines)
//! 3. **renderer** - High-level rendering (cameras, materials, objects, lights)
//! 4. **window** - Window management with winit (optional, feature = "window")
//! 5. **gui** - Text rendering (optional, feature = "gui")
//! 6. **urdf** - URDF robot model support
//!
//! # Example
//!
//! ```no_run
//! use rein::{Window, WindowSettings, FrameOutput};
//! use rein::renderer::{Camera, OrbitControl};
//! use rein::urdf::RobotModel;
//! use rein::core::ClearState;
//! use glam::Vec3;
//!
//! fn main() -> anyhow::Result<()> {
//!     let window = Window::new(WindowSettings::default().title("Robot Viewer"))?;
//!
//!     // State will be initialized in the render loop
//!     struct State {
//!         camera: Camera,
//!         control: OrbitControl,
//!         robot: Option<RobotModel>,
//!     }
//!
//!     // ...
//!     Ok(())
//! }
//! ```

pub mod context;
pub mod core;
pub mod effect;
pub mod renderer;
pub mod urdf;

#[cfg(feature = "window")]
pub mod window;

#[cfg(feature = "gui")]
pub mod gui;

// Re-export commonly used types
pub use context::WgpuContext;

pub use core::{
    BlendState, ClearState, CullState, DepthState, DepthTexture, IndexBuffer, InstanceBuffer,
    InstanceData, PipelineBuilder, RawUniformBuffer, RenderTarget, Texture2D, Texture2DArray,
    TextureCubeMap, UniformBuffer, VertexBuffer, VertexP, VertexPC, VertexPN, VertexPNUC,
};

pub use renderer::{
    Aabb, AmbientLight, Attenuation, Axes, BoundingBoxMesh, Camera, ColorMaterial, DepthMaterial,
    DirectionalLight, DirectionalShadow, FirstPersonControl, FlyControl, Frustum, FrustumCuller,
    Geometry, Gm, GridMaterial, InstancedMesh, Intersection, Light, LineMaterial, LineStrip, Lines,
    Material, Mesh, ModelUniform, NormalMaterial, Object, OrbitControl, PbrMaterial, PhongMaterial,
    Plane, PointLight, Projection, ShadowConfig, ShadowMap, ShadowUniform, SpotLight,
    UnlitMaterial, Viewer,
};

pub use urdf::{RobotModel, UrdfLoader};

pub use effect::{CopyEffect, Effect, EffectChain, FogEffect, FogMode, FullscreenQuad, FxaaEffect};

#[cfg(feature = "window")]
pub use window::{
    Event, FrameInput, FrameOutput, Key, Modifiers, MouseButton, Viewport, Window, WindowSettings,
    screen_target,
};

#[cfg(feature = "gui")]
pub use gui::{TextBuilder, TextRenderer};

// Re-export glam for convenience
pub use glam;
