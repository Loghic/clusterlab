//! Distance metrics. Currently only Euclidean; trait left in place
//! so other metrics (Manhattan, cosine) can slot in later.

use crate::geometry::{Point2, Point3};

/// A distance metric between two points.
pub trait Distance<P> {
    fn distance(&self, a: &P, b: &P) -> f32;

    /// Squared distance, used for comparisons (avoids a sqrt per call).
    fn distance_squared(&self, a: &P, b: &P) -> f32;
}

/// Euclidean distance.
#[derive(Debug, Default, Clone, Copy)]
pub struct Euclidean;

impl Distance<Point2> for Euclidean {
    fn distance(&self, a: &Point2, b: &Point2) -> f32 {
        a.distance(b)
    }
    fn distance_squared(&self, a: &Point2, b: &Point2) -> f32 {
        a.distance_squared(b)
    }
}

impl Distance<Point3> for Euclidean {
    fn distance(&self, a: &Point3, b: &Point3) -> f32 {
        a.distance(b)
    }
    fn distance_squared(&self, a: &Point3, b: &Point3) -> f32 {
        a.distance_squared(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn euclidean_2d() {
        let m = Euclidean;
        let a = Point2::new(0.0, 0.0);
        let b = Point2::new(3.0, 4.0);
        assert_relative_eq!(m.distance(&a, &b), 5.0);
        assert_relative_eq!(m.distance_squared(&a, &b), 25.0);
    }

    #[test]
    fn euclidean_3d() {
        let m = Euclidean;
        let a = Point3::new(0.0, 0.0, 0.0);
        let b = Point3::new(2.0, 3.0, 6.0);
        assert_relative_eq!(m.distance(&a, &b), 7.0);
    }

    #[test]
    fn euclidean_zero_at_same_point() {
        let m = Euclidean;
        let a = Point2::new(1.5, -2.5);
        assert_relative_eq!(m.distance(&a, &a), 0.0);
    }
}
