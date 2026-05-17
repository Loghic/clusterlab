//! 2D camera with pan (middle/right drag) and zoom (scroll).
//!
//! Maintains a separate world space from the screen so that the user can
//! place points anywhere with the mouse and still navigate around them.

use clusterlab::geometry::Point2;
pub use clusterlab::world::WORLD_BOUNDS;
use macroquad::prelude::*;

pub struct Camera2D {
    /// World-space point currently at the screen center.
    pub center: Vec2,
    /// Multiplier on the base "fit" zoom. 1.0 = fit world to screen.
    pub zoom: f32,
    last_mouse: Vec2,
    was_panning: bool,
}

impl Default for Camera2D {
    fn default() -> Self {
        let (min_x, min_y, max_x, max_y) = WORLD_BOUNDS;
        Self {
            center: Vec2::new((min_x + max_x) * 0.5, (min_y + max_y) * 0.5),
            zoom: 1.0,
            last_mouse: Vec2::ZERO,
            was_panning: false,
        }
    }
}

impl Camera2D {
    /// Pixels-per-world-unit at current zoom.
    pub fn scale(&self, screen_w: f32, screen_h: f32) -> f32 {
        let (min_x, min_y, max_x, max_y) = WORLD_BOUNDS;
        let world_w = max_x - min_x;
        let world_h = max_y - min_y;
        let fit = (screen_w / world_w).min(screen_h / world_h);
        fit * self.zoom
    }

    /// World point → screen pixel.
    pub fn world_to_screen(&self, p: Point2, screen_w: f32, screen_h: f32) -> (f32, f32) {
        let s = self.scale(screen_w, screen_h);
        let sx = (p.x - self.center.x) * s + screen_w * 0.5;
        let sy = (p.y - self.center.y) * s + screen_h * 0.5;
        (sx, sy)
    }

    /// Screen pixel → world point (inverse of [`world_to_screen`]).
    pub fn screen_to_world(&self, sx: f32, sy: f32, screen_w: f32, screen_h: f32) -> Point2 {
        let s = self.scale(screen_w, screen_h);
        Point2::new(
            (sx - screen_w * 0.5) / s + self.center.x,
            (sy - screen_h * 0.5) / s + self.center.y,
        )
    }

    /// Handle scroll-to-zoom and middle/right-drag-to-pan. Returns true if
    /// the camera moved this frame (caller can use this to suppress
    /// click-to-add when the user was actually dragging).
    pub fn handle_input(&mut self, blocked_by_ui: bool) -> bool {
        let (mx, my) = mouse_position();
        let mouse = Vec2::new(mx, my);
        let mut moved = false;

        // Pan with middle or right mouse button.
        let panning = !blocked_by_ui
            && (is_mouse_button_down(MouseButton::Middle)
                || is_mouse_button_down(MouseButton::Right));
        if panning {
            if self.was_panning {
                let s = self.scale(screen_width(), screen_height()).max(1e-6);
                let delta = mouse - self.last_mouse;
                self.center.x -= delta.x / s;
                self.center.y -= delta.y / s;
                if delta.length_squared() > 0.0 {
                    moved = true;
                }
            }
            self.was_panning = true;
        } else {
            self.was_panning = false;
        }
        self.last_mouse = mouse;

        // Zoom with scroll wheel, anchoring on the mouse position so the
        // point under the cursor stays put as zoom changes.
        if !blocked_by_ui {
            let (_, wheel_y) = mouse_wheel();
            if wheel_y.abs() > 0.0 {
                let sw = screen_width();
                let sh = screen_height();
                let world_before = self.screen_to_world(mx, my, sw, sh);
                let factor = if wheel_y > 0.0 { 1.15 } else { 1.0 / 1.15 };
                self.zoom = (self.zoom * factor).clamp(0.1, 20.0);
                let world_after = self.screen_to_world(mx, my, sw, sh);
                // Re-anchor so the cursor's world point doesn't drift.
                self.center.x += world_before.x - world_after.x;
                self.center.y += world_before.y - world_after.y;
                moved = true;
            }
        }

        moved
    }

    /// Reset to the default fit view.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
