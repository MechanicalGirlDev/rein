//! Post-processing effects
//!
//! Provides screen-space effects like FXAA and fog.

mod copy;
mod fog;
mod fullscreen;
mod fxaa;

pub use copy::CopyEffect;
pub use fog::{FogEffect, FogMode};
pub use fullscreen::FullscreenQuad;
pub use fxaa::FxaaEffect;

use crate::context::WgpuContext;
use crate::core::Texture2D;

/// Trait for post-processing effects.
pub trait Effect {
    /// Apply the effect to the input texture and write to the output.
    fn apply(
        &self,
        ctx: &WgpuContext,
        encoder: &mut wgpu::CommandEncoder,
        input: &wgpu::TextureView,
        output: &wgpu::TextureView,
    );
}

/// A chain of post-processing effects.
pub struct EffectChain {
    effects: Vec<Box<dyn Effect>>,
    intermediate_textures: Vec<Texture2D>,
}

impl EffectChain {
    /// Create a new empty effect chain.
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
            intermediate_textures: Vec::new(),
        }
    }

    /// Add an effect to the chain.
    pub fn add(&mut self, effect: Box<dyn Effect>) {
        self.effects.push(effect);
    }

    /// Ensure intermediate textures are allocated.
    pub fn ensure_textures(
        &mut self,
        ctx: &WgpuContext,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) {
        // Need n-1 intermediate textures for n effects
        let needed = if self.effects.len() > 1 {
            self.effects.len() - 1
        } else {
            0
        };

        while self.intermediate_textures.len() < needed {
            let tex = Texture2D::new(
                ctx,
                width,
                height,
                format,
                wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
                Some(&format!(
                    "effect chain intermediate {}",
                    self.intermediate_textures.len()
                )),
            );
            self.intermediate_textures.push(tex);
        }

        // Resize existing textures if needed
        for tex in &mut self.intermediate_textures {
            let (w, h) = tex.size();
            if w != width || h != height {
                *tex = Texture2D::new(
                    ctx,
                    width,
                    height,
                    format,
                    wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
                    Some("effect chain intermediate"),
                );
            }
        }
    }

    /// Apply all effects in sequence.
    pub fn apply(
        &self,
        ctx: &WgpuContext,
        encoder: &mut wgpu::CommandEncoder,
        input: &wgpu::TextureView,
        output: &wgpu::TextureView,
    ) {
        if self.effects.is_empty() {
            return;
        }

        if self.effects.len() == 1 {
            self.effects[0].apply(ctx, encoder, input, output);
            return;
        }

        // Chain effects through intermediate textures
        let mut current_input = input;
        for (i, effect) in self.effects.iter().enumerate() {
            let is_last = i == self.effects.len() - 1;
            let current_output = if is_last {
                output
            } else {
                self.intermediate_textures[i].view()
            };

            effect.apply(ctx, encoder, current_input, current_output);

            if !is_last {
                current_input = self.intermediate_textures[i].view();
            }
        }
    }

    /// Get the number of effects.
    pub fn len(&self) -> usize {
        self.effects.len()
    }

    /// Check if the chain is empty.
    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }

    /// Clear all effects.
    pub fn clear(&mut self) {
        self.effects.clear();
    }
}

impl Default for EffectChain {
    fn default() -> Self {
        Self::new()
    }
}
