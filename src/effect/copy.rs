//! Copy effect for basic texture copying

use super::{Effect, FullscreenQuad};
use crate::context::WgpuContext;
use crate::core::pipeline::PipelineBuilder;
use crate::core::render_states::{BlendState, CullState};
use crate::core::vertex::VertexPC;

/// Simple copy effect that copies input to output.
pub struct CopyEffect {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    quad: FullscreenQuad,
}

impl CopyEffect {
    /// Create a new copy effect.
    pub fn new(ctx: &WgpuContext, format: wgpu::TextureFormat) -> anyhow::Result<Self> {
        let shader = include_str!("../shaders/effects/copy.wgsl");

        let bind_group_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("copy effect bind group layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let pipeline = PipelineBuilder::new(ctx)
            .label("copy effect pipeline")
            .shader(shader)
            .vertex_layout(VertexPC::layout())
            .bind_group_layout(&bind_group_layout)
            .color_format(format)
            .blend(BlendState::Opaque)
            .cull(CullState::None)
            .build()?;

        let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("copy effect sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let quad = FullscreenQuad::new(ctx);

        Ok(Self {
            pipeline,
            bind_group_layout,
            sampler,
            quad,
        })
    }

    fn create_bind_group(&self, ctx: &WgpuContext, input: &wgpu::TextureView) -> wgpu::BindGroup {
        ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("copy effect bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(input),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        })
    }
}

impl Effect for CopyEffect {
    fn apply(
        &self,
        ctx: &WgpuContext,
        encoder: &mut wgpu::CommandEncoder,
        input: &wgpu::TextureView,
        output: &wgpu::TextureView,
    ) {
        let bind_group = self.create_bind_group(ctx, input);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("copy effect pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        self.quad.draw(&mut render_pass);
    }
}
