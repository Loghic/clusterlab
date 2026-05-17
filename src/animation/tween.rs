//! Interpolation and easing functions for animation.

use crate::geometry::{Point2, Point3};

/// Linear interpolation: `t = 0.0` returns `a`, `t = 1.0` returns `b`.
/// `t` is clamped to `[0.0, 1.0]`.
pub fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    a + (b - a) * t
}

/// Lerp between two 2D points.
pub fn lerp_point2(a: Point2, b: Point2, t: f32) -> Point2 {
    Point2::new(lerp_f32(a.x, b.x, t), lerp_f32(a.y, b.y, t))
}

/// Lerp between two 3D points.
pub fn lerp_point3(a: Point3, b: Point3, t: f32) -> Point3 {
    Point3::new(
        lerp_f32(a.x, b.x, t),
        lerp_f32(a.y, b.y, t),
        lerp_f32(a.z, b.z, t),
    )
}

/// Smoothstep easing: makes motion ease in and out instead of linear.
/// `t` clamped to `[0.0, 1.0]`.
pub fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn lerp_at_zero_returns_a() {
        assert_relative_eq!(lerp_f32(5.0, 10.0, 0.0), 5.0);
    }

    #[test]
    fn lerp_at_one_returns_b() {
        assert_relative_eq!(lerp_f32(5.0, 10.0, 1.0), 10.0);
    }

    #[test]
    fn lerp_at_half_returns_midpoint() {
        assert_relative_eq!(lerp_f32(0.0, 10.0, 0.5), 5.0);
    }

    #[test]
    fn lerp_clamps_below_zero() {
        assert_relative_eq!(lerp_f32(0.0, 10.0, -1.0), 0.0);
    }

    #[test]
    fn lerp_clamps_above_one() {
        assert_relative_eq!(lerp_f32(0.0, 10.0, 2.0), 10.0);
    }

    #[test]
    fn lerp_point2_endpoints() {
        let a = Point2::new(0.0, 0.0);
        let b = Point2::new(10.0, 20.0);
        assert_eq!(lerp_point2(a, b, 0.0), a);
        assert_eq!(lerp_point2(a, b, 1.0), b);
        assert_eq!(lerp_point2(a, b, 0.5), Point2::new(5.0, 10.0));
    }

    #[test]
    fn lerp_point3_endpoints() {
        let a = Point3::new(0.0, 0.0, 0.0);
        let b = Point3::new(2.0, 4.0, 6.0);
        assert_eq!(lerp_point3(a, b, 0.0), a);
        assert_eq!(lerp_point3(a, b, 1.0), b);
        assert_eq!(lerp_point3(a, b, 0.5), Point3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn smoothstep_endpoints() {
        assert_relative_eq!(smoothstep(0.0), 0.0);
        assert_relative_eq!(smoothstep(1.0), 1.0);
    }

    #[test]
    fn smoothstep_midpoint() {
        assert_relative_eq!(smoothstep(0.5), 0.5);
    }

    #[test]
    fn smoothstep_monotonic() {
        let mut prev = smoothstep(0.0);
        for i in 1..=10 {
            let t = i as f32 / 10.0;
            let v = smoothstep(t);
            assert!(v >= prev, "smoothstep not monotonic at t={t}");
            prev = v;
        }
    }
}
