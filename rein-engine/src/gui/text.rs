//! Text rendering
//!
//! Provides text rendering using glyphon.

use crate::context::WgpuContext;
use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer as GlyphonTextRenderer,
};

/// Text renderer using glyphon.
pub struct TextRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    #[allow(dead_code)]
    cache: Cache,
    atlas: TextAtlas,
    renderer: GlyphonTextRenderer,
    buffer: Buffer,
    viewport: glyphon::Viewport,
}

impl TextRenderer {
    /// Create a new text renderer.
    pub fn new(ctx: &WgpuContext, format: wgpu::TextureFormat) -> Self {
        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&ctx.device);

        let mut atlas = TextAtlas::new(&ctx.device, &ctx.queue, &cache, format);

        let renderer = GlyphonTextRenderer::new(
            &mut atlas,
            &ctx.device,
            wgpu::MultisampleState::default(),
            None,
        );

        let buffer = Buffer::new(&mut font_system, Metrics::new(14.0, 18.0));

        let viewport = glyphon::Viewport::new(&ctx.device, &cache);

        Self {
            font_system,
            swash_cache,
            cache,
            atlas,
            renderer,
            buffer,
            viewport,
        }
    }

    /// Update the viewport size.
    pub fn resize(&mut self, ctx: &WgpuContext, width: u32, height: u32) {
        self.viewport
            .update(&ctx.queue, Resolution { width, height });
    }

    /// Set the text content.
    pub fn set_text(&mut self, text: &str, font_size: f32, line_height: f32) {
        self.buffer
            .set_metrics(&mut self.font_system, Metrics::new(font_size, line_height));
        self.buffer.set_text(
            &mut self.font_system,
            text,
            &Attrs::new().family(Family::Monospace),
            Shaping::Advanced,
            None,
        );
    }

    /// Shape the text at the given width.
    pub fn shape(&mut self, width: f32) {
        self.buffer
            .set_size(&mut self.font_system, Some(width), None);
        self.buffer.shape_until_scroll(&mut self.font_system, false);
    }

    /// Render the text to a render pass.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        ctx: &WgpuContext,
        view: &wgpu::TextureView,
        width: u32,
        height: u32,
        x: f32,
        y: f32,
        color: [f32; 4],
    ) -> anyhow::Result<()> {
        // Update viewport
        self.viewport
            .update(&ctx.queue, Resolution { width, height });

        let text_color = Color::rgba(
            (color[0] * 255.0) as u8,
            (color[1] * 255.0) as u8,
            (color[2] * 255.0) as u8,
            (color[3] * 255.0) as u8,
        );

        let text_areas = [TextArea {
            buffer: &self.buffer,
            left: x,
            top: y,
            scale: 1.0,
            bounds: TextBounds {
                left: 0,
                top: 0,
                right: width as i32,
                bottom: height as i32,
            },
            default_color: text_color,
            custom_glyphs: &[],
        }];

        self.renderer.prepare(
            &ctx.device,
            &ctx.queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            text_areas,
            &mut self.swash_cache,
        )?;

        let mut encoder = ctx.create_encoder(Some("text encoder"));
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("text render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            self.renderer
                .render(&self.atlas, &self.viewport, &mut pass)?;
        }

        ctx.submit([encoder.finish()]);

        Ok(())
    }

    /// Trim the atlas to free unused space.
    pub fn trim(&mut self) {
        self.atlas.trim();
    }
}

/// Helper for building text content with formatting.
pub struct TextBuilder {
    lines: Vec<String>,
}

impl TextBuilder {
    /// Create a new text builder.
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    /// Add a line of text.
    pub fn line(mut self, text: impl Into<String>) -> Self {
        self.lines.push(text.into());
        self
    }

    /// Add an empty line.
    pub fn blank(mut self) -> Self {
        self.lines.push(String::new());
        self
    }

    /// Add a separator line.
    pub fn separator(mut self, char: char, width: usize) -> Self {
        self.lines.push(char.to_string().repeat(width));
        self
    }

    /// Build the final text string.
    pub fn build(self) -> String {
        self.lines.join("\n")
    }
}

impl Default for TextBuilder {
    fn default() -> Self {
        Self::new()
    }
}
