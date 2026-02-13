//! UI System
//!
//! Provides a simple immediate mode UI system.

use crate::context::WgpuContext;
use crate::gui::primitive::PrimitiveRenderer;
use crate::gui::text::TextRenderer;
use crate::window::event::{Event, MouseButton};
use glam::Vec2;

/// UI Context managing draw calls and input state.
pub struct UiContext {
    primitive: PrimitiveRenderer,
    text: TextRenderer,
    mouse_pos: Vec2,
    mouse_down: bool,
    mouse_clicked: bool,
    viewport_width: u32,
    viewport_height: u32,
}

impl UiContext {
    /// Create a new UI context.
    pub fn new(ctx: &WgpuContext, format: wgpu::TextureFormat) -> Self {
        Self {
            primitive: PrimitiveRenderer::new(ctx, format),
            text: TextRenderer::new(ctx, format),
            mouse_pos: Vec2::ZERO,
            mouse_down: false,
            mouse_clicked: false,
            viewport_width: 0,
            viewport_height: 0,
        }
    }

    /// Update input state and prepare for a new frame.
    pub fn update(&mut self, events: &[Event], width: u32, height: u32) {
        self.viewport_width = width;
        self.viewport_height = height;
        self.mouse_clicked = false;

        for event in events {
            match event {
                Event::MouseMotion { position, .. } => {
                    self.mouse_pos = Vec2::new(position.0, position.1);
                }
                Event::MousePress { button: MouseButton::Left, .. } => {
                    self.mouse_down = true;
                }
                Event::MouseRelease { button: MouseButton::Left, .. } => {
                    self.mouse_down = false;
                    self.mouse_clicked = true;
                }
                _ => {}
            }
        }

        self.primitive.finish();
        self.text.begin_frame();
    }

    /// Render the UI to a render pass.
    pub fn render(&mut self, ctx: &WgpuContext, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) -> anyhow::Result<()> {
        // Render primitives
        self.primitive.prepare(ctx, self.viewport_width, self.viewport_height);

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("UI Primitives Pass"),
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
            self.primitive.render(&mut pass);
        }

        // Render text (overlay)
        self.text.render(ctx, encoder, view, self.viewport_width, self.viewport_height)?;

        Ok(())
    }

    /// Draw a text label.
    pub fn label(&mut self, text: &str, x: f32, y: f32, color: [f32; 4]) {
        self.text.draw_text(text, x, y, 14.0, color);
    }

    /// Draw a button.
    pub fn button(&mut self, text: &str, x: f32, y: f32, w: f32, h: f32) -> bool {
        let mouse_pos = self.mouse_pos;
        let hovered = mouse_pos.x >= x && mouse_pos.x <= x + w && mouse_pos.y >= y && mouse_pos.y <= y + h;
        let clicked = hovered && self.mouse_clicked;

        let color = if hovered {
            if self.mouse_down { [0.3, 0.3, 0.3, 1.0] } else { [0.4, 0.4, 0.4, 1.0] }
        } else {
            [0.2, 0.2, 0.2, 1.0]
        };

        self.primitive.draw_rect(x, y, w, h, color);

        let (text_w, text_h) = self.text.measure(text, 14.0);
        let text_x = x + (w - text_w) / 2.0;
        let text_y = y + (h - text_h) / 2.0;

        self.text.draw_text(text, text_x, text_y, 14.0, [1.0, 1.0, 1.0, 1.0]);

        clicked
    }

    /// Draw a checkbox.
    pub fn checkbox(&mut self, checked: bool, text: &str, x: f32, y: f32) -> bool {
        let size = 20.0;
        let mouse_pos = self.mouse_pos;
        let hovered = mouse_pos.x >= x && mouse_pos.x <= x + size && mouse_pos.y >= y && mouse_pos.y <= y + size;
        let clicked = hovered && self.mouse_clicked;

        let new_checked = if clicked { !checked } else { checked };

        let color = if hovered { [0.4, 0.4, 0.4, 1.0] } else { [0.2, 0.2, 0.2, 1.0] };
        self.primitive.draw_rect(x, y, size, size, color);

        if new_checked {
            let inner_size = size * 0.6;
            let offset = (size - inner_size) / 2.0;
            self.primitive.draw_rect(x + offset, y + offset, inner_size, inner_size, [0.8, 0.8, 0.8, 1.0]);
        }

        self.text.draw_text(text, x + size + 8.0, y + 2.0, 14.0, [1.0, 1.0, 1.0, 1.0]);

        new_checked
    }

    /// Draw a radio button.
    pub fn radio_button(&mut self, selected: bool, text: &str, x: f32, y: f32) -> bool {
        let size = 20.0;
        let mouse_pos = self.mouse_pos;
        let hovered = mouse_pos.x >= x && mouse_pos.x <= x + size && mouse_pos.y >= y && mouse_pos.y <= y + size;
        let clicked = hovered && self.mouse_clicked;

        let color = if hovered { [0.4, 0.4, 0.4, 1.0] } else { [0.2, 0.2, 0.2, 1.0] };
        self.primitive.draw_circle(x, y, size / 2.0, color);

        if selected {
            let inner_size = size * 0.6;
            let offset = (size - inner_size) / 2.0;
            self.primitive.draw_circle(x + offset, y + offset, inner_size / 2.0, [0.8, 0.8, 0.8, 1.0]);
        }

        self.text.draw_text(text, x + size + 8.0, y + 2.0, 14.0, [1.0, 1.0, 1.0, 1.0]);

        clicked
    }
}
