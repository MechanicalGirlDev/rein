//! Core rendering abstractions
//!
//! This module provides mid-level abstractions over wgpu primitives.

pub mod buffer;
pub mod instance;
pub mod pipeline;
pub mod render_states;
pub mod render_target;
pub mod texture;
pub mod vertex;

pub use buffer::{IndexBuffer, RawUniformBuffer, StorageBuffer, UniformBuffer, VertexBuffer};
pub use instance::{InstanceBuffer, InstanceData};
pub use pipeline::{ComputePipelineBuilder, PipelineBuilder};
pub use render_states::{BlendState, ClearState, CullState, DepthState};
pub use render_target::RenderTarget;
pub use texture::{DepthTexture, Texture2D, Texture2DArray, TextureCubeMap};
pub use vertex::{VertexP, VertexPC, VertexPN, VertexPNUC};
