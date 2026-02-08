//! Entity Component System integration with hecs.

pub mod bridge;
pub mod components;
pub mod systems;

pub mod prelude {
    pub use super::bridge::*;
    pub use super::components::*;
    pub use super::systems::{culling_system, render_system, transform_system};
}
