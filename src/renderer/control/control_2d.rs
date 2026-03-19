//! 2D camera control
//!
//! Provides pan and zoom control for 2D/orthographic views.

use crate::renderer::viewer::{Camera, Projection};
use crate::window::event::{Event, MouseButton};
use glam::Vec3;

/// 2D camera control for pan and zoom in an orthographic view.
///
/// Inspired by three-d's 2D control, this provides mouse-based panning
/// and scroll-based zooming for 2D scenes viewed with an orthographic camera.
pub struct Control2D {
    /// Pan speed multiplier.
    pub pan_speed: f32,
    /// Zoom speed multiplier.
    pub zoom_speed: f32,
    /// Minimum orthographic view width.
    pub min_width: f32,
    /// Maximum orthographic view width.
    pub max_width: f32,

    // Internal state
    left_mouse_pressed: bool,
}

impl Control2D {
    /// Create a new 2D control.
    pub fn new(min_width: f32, max_width: f32) -> Self {
        Self {
            pan_speed: 1.0,
            zoom_speed: 0.1,
            min_width,
            max_width,
            left_mouse_pressed: false,
        }
    }

    /// Handle events and update the camera.
    pub fn handle_events(&mut self, camera: &mut Camera, events: &mut [Event]) {
        for event in events.iter_mut() {
            if event.is_handled() {
                continue;
            }

            match event {
                Event::MousePress { button, .. } => {
                    if *button == MouseButton::Left {
                        self.left_mouse_pressed = true;
                    }
                }
                Event::MouseRelease { button, .. } => {
                    if *button == MouseButton::Left {
                        self.left_mouse_pressed = false;
                    }
                }
                Event::MouseMotion { delta, .. } => {
                    if self.left_mouse_pressed {
                        self.pan(camera, delta.0, delta.1);
                        event.set_handled();
                    }
                }
                Event::MouseWheel { delta, .. } => {
                    self.zoom(camera, delta.1);
                    event.set_handled();
                }
                _ => {}
            }
        }
    }

    /// Pan the camera in the XY plane.
    fn pan(&self, camera: &mut Camera, dx: f32, dy: f32) {
        let scale = match camera.projection {
            Projection::Orthographic { width, .. } => width * self.pan_speed * 0.001,
            Projection::Perspective { .. } => self.pan_speed * 0.01,
        };

        let offset = Vec3::new(-dx * scale, dy * scale, 0.0);
        camera.position += offset;
        camera.target += offset;
    }

    /// Zoom the camera by adjusting the orthographic width.
    fn zoom(&self, camera: &mut Camera, delta: f32) {
        if let Projection::Orthographic {
            ref mut width,
            ref mut height,
            ..
        } = camera.projection
        {
            let aspect = *width / *height;
            let new_width =
                (*width * (1.0 - delta * self.zoom_speed)).clamp(self.min_width, self.max_width);
            *width = new_width;
            *height = new_width / aspect;
        }
    }
}

impl Default for Control2D {
    fn default() -> Self {
        Self::new(1.0, 100.0)
    }
}
