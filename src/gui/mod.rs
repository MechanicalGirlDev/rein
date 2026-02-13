//! GUI module
//!
//! Provides UI rendering capabilities including text and 2D primitives.

pub mod primitive;
pub mod text;
pub mod ui;

pub use primitive::PrimitiveRenderer;
pub use text::{TextBuilder, TextRenderer};
pub use ui::UiContext;
