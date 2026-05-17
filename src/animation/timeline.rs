//! Timeline tracks "from" and "to" centroid snapshots and an elapsed time,
//! producing interpolated frames for the renderer.
//!
//! A timeline has two phases:
//! 1. **Tweening**: `0 <= elapsed < duration`, interpolating from→to.
//! 2. **Holding**: `duration <= elapsed < duration + hold`, sitting at `to`.
//!
//! `is_done()` is true only after the hold phase. Set `hold = 0.0` for
//! the simple "tween then stop" behaviour.

use crate::animation::tween::{lerp_point2, lerp_point3, smoothstep};
use crate::geometry::{Point2, Point3};

/// Animation timeline for 2D centroids.
#[derive(Debug, Clone)]
pub struct Timeline2D {
    pub from: Vec<Point2>,
    pub to: Vec<Point2>,
    pub elapsed: f32,
    pub duration: f32,
    pub hold: f32,
}

impl Timeline2D {
    pub fn new(from: Vec<Point2>, to: Vec<Point2>, duration: f32) -> Self {
        Self::with_hold(from, to, duration, 0.0)
    }

    pub fn with_hold(from: Vec<Point2>, to: Vec<Point2>, duration: f32, hold: f32) -> Self {
        Self {
            from,
            to,
            elapsed: 0.0,
            duration: duration.max(1e-3),
            hold: hold.max(0.0),
        }
    }

    /// Advance the timeline by `dt` seconds.
    pub fn advance(&mut self, dt: f32) {
        self.elapsed = (self.elapsed + dt).min(self.duration + self.hold);
    }

    /// Returns true when both the tween and hold phases have played out.
    pub fn is_done(&self) -> bool {
        self.elapsed >= self.duration + self.hold
    }

    /// Returns true while in the hold phase (after the tween ends, before
    /// the timeline is fully done).
    pub fn is_holding(&self) -> bool {
        self.elapsed >= self.duration && !self.is_done()
    }

    /// Normalized tween progress in `[0, 1]`. Stays at 1.0 during the hold
    /// phase.
    pub fn progress(&self) -> f32 {
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }

    /// Interpolated centroid positions for the current frame.
    pub fn current(&self) -> Vec<Point2> {
        let t = smoothstep(self.progress());
        self.from
            .iter()
            .zip(self.to.iter())
            .map(|(a, b)| lerp_point2(*a, *b, t))
            .collect()
    }
}

/// Animation timeline for 3D centroids.
#[derive(Debug, Clone)]
pub struct Timeline3D {
    pub from: Vec<Point3>,
    pub to: Vec<Point3>,
    pub elapsed: f32,
    pub duration: f32,
    pub hold: f32,
}

impl Timeline3D {
    pub fn new(from: Vec<Point3>, to: Vec<Point3>, duration: f32) -> Self {
        Self::with_hold(from, to, duration, 0.0)
    }

    pub fn with_hold(from: Vec<Point3>, to: Vec<Point3>, duration: f32, hold: f32) -> Self {
        Self {
            from,
            to,
            elapsed: 0.0,
            duration: duration.max(1e-3),
            hold: hold.max(0.0),
        }
    }

    pub fn advance(&mut self, dt: f32) {
        self.elapsed = (self.elapsed + dt).min(self.duration + self.hold);
    }

    pub fn is_done(&self) -> bool {
        self.elapsed >= self.duration + self.hold
    }

    pub fn is_holding(&self) -> bool {
        self.elapsed >= self.duration && !self.is_done()
    }

    pub fn progress(&self) -> f32 {
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }

    pub fn current(&self) -> Vec<Point3> {
        let t = smoothstep(self.progress());
        self.from
            .iter()
            .zip(self.to.iter())
            .map(|(a, b)| lerp_point3(*a, *b, t))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeline2d_starts_at_from() {
        let tl = Timeline2D::new(
            vec![Point2::new(0.0, 0.0)],
            vec![Point2::new(10.0, 10.0)],
            1.0,
        );
        assert_eq!(tl.current()[0], Point2::new(0.0, 0.0));
        assert!(!tl.is_done());
    }

    #[test]
    fn timeline2d_ends_at_to() {
        let mut tl = Timeline2D::new(
            vec![Point2::new(0.0, 0.0)],
            vec![Point2::new(10.0, 10.0)],
            1.0,
        );
        tl.advance(1.5);
        assert!(tl.is_done());
        assert_eq!(tl.current()[0], Point2::new(10.0, 10.0));
    }

    #[test]
    fn timeline2d_progress_reaches_one() {
        let mut tl = Timeline2D::new(vec![Point2::ZERO], vec![Point2::new(1.0, 1.0)], 0.5);
        tl.advance(0.5);
        assert!((tl.progress() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn timeline_with_hold_holds_at_target() {
        let mut tl =
            Timeline2D::with_hold(vec![Point2::ZERO], vec![Point2::new(10.0, 10.0)], 0.5, 1.0);
        tl.advance(0.6);
        assert!(!tl.is_done(), "should still be holding");
        assert!(tl.is_holding(), "is_holding should be true");
        assert_eq!(tl.current()[0], Point2::new(10.0, 10.0));
        tl.advance(1.0);
        assert!(tl.is_done(), "should be done after hold");
        assert!(!tl.is_holding(), "is_holding should be false when done");
    }

    #[test]
    fn timeline_without_hold_is_done_immediately_after_tween() {
        let mut tl = Timeline2D::new(vec![Point2::ZERO], vec![Point2::new(1.0, 1.0)], 0.5);
        tl.advance(0.5);
        assert!(tl.is_done());
        assert!(!tl.is_holding());
    }

    #[test]
    fn timeline3d_endpoints() {
        let mut tl = Timeline3D::new(vec![Point3::ZERO], vec![Point3::new(2.0, 4.0, 6.0)], 1.0);
        assert_eq!(tl.current()[0], Point3::ZERO);
        tl.advance(2.0);
        assert_eq!(tl.current()[0], Point3::new(2.0, 4.0, 6.0));
    }

    #[test]
    fn timeline3d_hold_phase() {
        let mut tl = Timeline3D::with_hold(
            vec![Point3::ZERO],
            vec![Point3::new(1.0, 2.0, 3.0)],
            0.5,
            0.5,
        );
        tl.advance(0.6);
        assert!(tl.is_holding());
        assert_eq!(tl.current()[0], Point3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn timeline_handles_zero_duration_safely() {
        let tl = Timeline2D::new(vec![Point2::ZERO], vec![Point2::new(1.0, 1.0)], 0.0);
        let v = tl.current();
        assert!(v[0].x.is_finite() && v[0].y.is_finite());
    }

    #[test]
    fn negative_hold_clamped_to_zero() {
        let tl = Timeline2D::with_hold(vec![Point2::ZERO], vec![Point2::new(1.0, 1.0)], 0.5, -2.0);
        assert_eq!(tl.hold, 0.0);
    }
}
