//! Centroid initialization and recomputation.

use crate::geometry::{Point2, Point3};
use rand::Rng;

/// Initialize `k` centroids by picking random points uniformly within `bounds`.
///
/// `bounds` is `(min_x, min_y, max_x, max_y)`. This is the simple v1 strategy;
/// k-means++ can replace this later.
pub fn init_random_2d<R: Rng>(rng: &mut R, k: usize, bounds: (f32, f32, f32, f32)) -> Vec<Point2> {
    let (min_x, min_y, max_x, max_y) = bounds;
    (0..k)
        .map(|_| Point2::new(rng.gen_range(min_x..=max_x), rng.gen_range(min_y..=max_y)))
        .collect()
}

/// 3D version of [`init_random_2d`].
pub fn init_random_3d<R: Rng>(
    rng: &mut R,
    k: usize,
    bounds: (f32, f32, f32, f32, f32, f32),
) -> Vec<Point3> {
    let (min_x, min_y, min_z, max_x, max_y, max_z) = bounds;
    (0..k)
        .map(|_| {
            Point3::new(
                rng.gen_range(min_x..=max_x),
                rng.gen_range(min_y..=max_y),
                rng.gen_range(min_z..=max_z),
            )
        })
        .collect()
}

/// Recompute the centroid of a cluster as the mean of its members.
///
/// Returns `None` if the cluster is empty (caller decides how to handle).
pub fn mean_2d(points: &[Point2]) -> Option<Point2> {
    if points.is_empty() {
        return None;
    }
    let n = points.len() as f32;
    let sum = points.iter().copied().fold(Point2::ZERO, |acc, p| acc + p);
    Some(sum / n)
}

/// 3D version of [`mean_2d`].
pub fn mean_3d(points: &[Point3]) -> Option<Point3> {
    if points.is_empty() {
        return None;
    }
    let n = points.len() as f32;
    let sum = points.iter().copied().fold(Point3::ZERO, |acc, p| acc + p);
    Some(sum / n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn mean_2d_of_known_points() {
        let pts = vec![
            Point2::new(0.0, 0.0),
            Point2::new(2.0, 0.0),
            Point2::new(0.0, 2.0),
            Point2::new(2.0, 2.0),
        ];
        let c = mean_2d(&pts).unwrap();
        assert_relative_eq!(c.x, 1.0);
        assert_relative_eq!(c.y, 1.0);
    }

    #[test]
    fn mean_2d_empty_returns_none() {
        assert!(mean_2d(&[]).is_none());
    }

    #[test]
    fn mean_2d_single_point() {
        let p = Point2::new(3.5, -2.0);
        let c = mean_2d(&[p]).unwrap();
        assert_eq!(c, p);
    }

    #[test]
    fn mean_3d_of_known_points() {
        let pts = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(6.0, 9.0, 12.0)];
        let c = mean_3d(&pts).unwrap();
        assert_relative_eq!(c.x, 3.0);
        assert_relative_eq!(c.y, 4.5);
        assert_relative_eq!(c.z, 6.0);
    }

    #[test]
    fn mean_3d_empty_returns_none() {
        assert!(mean_3d(&[]).is_none());
    }

    #[test]
    fn init_random_2d_produces_k_centroids_in_bounds() {
        let mut rng = StdRng::seed_from_u64(42);
        let cs = init_random_2d(&mut rng, 5, (0.0, 0.0, 10.0, 10.0));
        assert_eq!(cs.len(), 5);
        for c in cs {
            assert!(c.x >= 0.0 && c.x <= 10.0);
            assert!(c.y >= 0.0 && c.y <= 10.0);
        }
    }

    #[test]
    fn init_random_2d_is_deterministic_with_seed() {
        let mut a = StdRng::seed_from_u64(7);
        let mut b = StdRng::seed_from_u64(7);
        let c1 = init_random_2d(&mut a, 4, (0.0, 0.0, 100.0, 100.0));
        let c2 = init_random_2d(&mut b, 4, (0.0, 0.0, 100.0, 100.0));
        assert_eq!(c1, c2);
    }

    #[test]
    fn init_random_3d_produces_k_centroids_in_bounds() {
        let mut rng = StdRng::seed_from_u64(1);
        let cs = init_random_3d(&mut rng, 3, (-1.0, -1.0, -1.0, 1.0, 1.0, 1.0));
        assert_eq!(cs.len(), 3);
        for c in cs {
            assert!(c.x >= -1.0 && c.x <= 1.0);
            assert!(c.y >= -1.0 && c.y <= 1.0);
            assert!(c.z >= -1.0 && c.z <= 1.0);
        }
    }
}
