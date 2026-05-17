//! Orbit camera for 3D mode.
//!
//! Right-click and drag to rotate, scroll wheel to zoom.
//! We track the last mouse position manually instead of using
//! `mouse_delta_position`, which is more reliable on web.

use macroquad::prelude::*;

pub struct OrbitCamera {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    last_mouse: Vec2,
    was_rotating: bool,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            target: vec3(0.0, 0.0, 0.0),
            yaw: 0.6,
            pitch: 0.6,
            distance: 14.0,
            last_mouse: Vec2::ZERO,
            was_rotating: false,
        }
    }
}

impl OrbitCamera {
    /// Position derived from yaw, pitch, distance and target.
    pub fn position(&self) -> Vec3 {
        let x = self.distance * self.pitch.cos() * self.yaw.sin();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.pitch.cos() * self.yaw.cos();
        self.target + vec3(x, y, z)
    }

    /// Returns a macroquad `Camera3D` ready to be passed to `set_camera`.
    pub fn camera3d(&self) -> Camera3D {
        Camera3D {
            position: self.position(),
            up: vec3(0.0, 1.0, 0.0),
            target: self.target,
            ..Default::default()
        }
    }

    /// Process input: right-drag rotates, scroll zooms.
    pub fn handle_input(&mut self) {
        let (mx, my) = mouse_position();
        let mouse_now = Vec2::new(mx, my);

        let rotating = is_mouse_button_down(MouseButton::Right);
        if rotating {
            if self.was_rotating {
                let dx = mouse_now.x - self.last_mouse.x;
                let dy = mouse_now.y - self.last_mouse.y;
                // Rotation sensitivity: roughly half a turn per screen width.
                self.yaw -= dx * 0.01;
                self.pitch += dy * 0.01;
                // Clamp pitch to avoid gimbal flip / inverted view.
                let limit = std::f32::consts::FRAC_PI_2 - 0.05;
                self.pitch = self.pitch.clamp(-limit, limit);
            }
            self.was_rotating = true;
        } else {
            self.was_rotating = false;
        }
        self.last_mouse = mouse_now;

        let (_, scroll_y) = mouse_wheel();
        if scroll_y.abs() > 0.0 {
            // On web, scroll deltas are larger; we normalize to a fixed step.
            let step = if scroll_y > 0.0 { -0.5 } else { 0.5 };
            self.distance = (self.distance + step).clamp(2.0, 80.0);
        }
    }
}
