//! Random sampling utilities for generating synthetic datasets.
//!
//! Implements Box–Muller transform for normal distributions so we don't
//! need an extra `rand_distr` dependency, and provides high-level helpers
//! for the common "blob" and "two moons" datasets used in clustering demos.

use crate::geometry::{Point2, Point3};
use rand::Rng;
use std::f32::consts::{PI, TAU};

/// Sample a single value from `N(mean, std_dev)` using Box–Muller.
///
/// One call discards one of the two values Box–Muller produces; that's
/// fine here — we're not in a hot loop.
pub fn sample_normal<R: Rng>(rng: &mut R, mean: f32, std_dev: f32) -> f32 {
    let u1: f32 = rng.gen_range(f32::EPSILON..1.0);
    let u2: f32 = rng.gen_range(0.0..1.0);
    let z = (-2.0 * u1.ln()).sqrt() * (TAU * u2).cos();
    mean + std_dev * z
}

/// Sample a 2D point from a Gaussian centered at `mean` with given std dev
/// (isotropic — same spread on both axes).
pub fn sample_normal_2d<R: Rng>(rng: &mut R, mean: Point2, std_dev: f32) -> Point2 {
    Point2::new(
        sample_normal(rng, mean.x, std_dev),
        sample_normal(rng, mean.y, std_dev),
    )
}

/// 3D version of [`sample_normal_2d`].
pub fn sample_normal_3d<R: Rng>(rng: &mut R, mean: Point3, std_dev: f32) -> Point3 {
    Point3::new(
        sample_normal(rng, mean.x, std_dev),
        sample_normal(rng, mean.y, std_dev),
        sample_normal(rng, mean.z, std_dev),
    )
}

/// Generate `n` 2D points distributed across `num_blobs` Gaussian clusters
/// whose centers are placed uniformly within `bounds`.
///
/// `bounds` is `(min_x, min_y, max_x, max_y)`. Points are clamped to bounds.
pub fn generate_blobs_2d<R: Rng>(
    rng: &mut R,
    n: usize,
    num_blobs: usize,
    std_dev: f32,
    bounds: (f32, f32, f32, f32),
) -> Vec<Point2> {
    let (min_x, min_y, max_x, max_y) = bounds;
    let w = max_x - min_x;
    let h = max_y - min_y;
    let margin_x = w * 0.15;
    let margin_y = h * 0.15;
    let centers: Vec<Point2> = (0..num_blobs.max(1))
        .map(|_| {
            Point2::new(
                rng.gen_range((min_x + margin_x)..(max_x - margin_x)),
                rng.gen_range((min_y + margin_y)..(max_y - margin_y)),
            )
        })
        .collect();
    (0..n)
        .map(|i| {
            let c = centers[i % centers.len()];
            let p = sample_normal_2d(rng, c, std_dev);
            Point2::new(p.x.clamp(min_x, max_x), p.y.clamp(min_y, max_y))
        })
        .collect()
}

/// 3D version of [`generate_blobs_2d`].
pub fn generate_blobs_3d<R: Rng>(
    rng: &mut R,
    n: usize,
    num_blobs: usize,
    std_dev: f32,
    bounds: (f32, f32, f32, f32, f32, f32),
) -> Vec<Point3> {
    let (min_x, min_y, min_z, max_x, max_y, max_z) = bounds;
    let margin = 0.15;
    let centers: Vec<Point3> = (0..num_blobs.max(1))
        .map(|_| {
            Point3::new(
                rng.gen_range(
                    (min_x + (max_x - min_x) * margin)..(max_x - (max_x - min_x) * margin),
                ),
                rng.gen_range(
                    (min_y + (max_y - min_y) * margin)..(max_y - (max_y - min_y) * margin),
                ),
                rng.gen_range(
                    (min_z + (max_z - min_z) * margin)..(max_z - (max_z - min_z) * margin),
                ),
            )
        })
        .collect();
    (0..n)
        .map(|i| {
            let c = centers[i % centers.len()];
            let p = sample_normal_3d(rng, c, std_dev);
            Point3::new(
                p.x.clamp(min_x, max_x),
                p.y.clamp(min_y, max_y),
                p.z.clamp(min_z, max_z),
            )
        })
        .collect()
}

/// Generate the classic "two moons" 2D dataset: two interleaving half-circles.
///
/// Mapped into `bounds`; k-means famously misclusters this since the moons
/// are not linearly separable. `noise` controls Gaussian noise std-dev as a
/// fraction of the smaller bound dimension.
pub fn generate_moons_2d<R: Rng>(
    rng: &mut R,
    n: usize,
    noise: f32,
    bounds: (f32, f32, f32, f32),
) -> Vec<Point2> {
    let (min_x, min_y, max_x, max_y) = bounds;
    let cx = (min_x + max_x) * 0.5;
    let cy = (min_y + max_y) * 0.5;
    let scale = ((max_x - min_x).min(max_y - min_y)) * 0.30;
    let noise_std = scale * noise;

    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let upper = i % 2 == 0;
        let t: f32 = rng.gen_range(0.0..1.0);
        let theta = t * PI;
        let (dx, dy) = if upper {
            // Upper moon: centered slightly left and up
            (theta.cos() - 0.5, theta.sin() - 0.25)
        } else {
            // Lower moon: mirrored, centered slightly right and down
            (-(theta.cos()) + 0.5, -(theta.sin()) + 0.25)
        };
        let nx = sample_normal(rng, 0.0, noise_std);
        let ny = sample_normal(rng, 0.0, noise_std);
        let px = (cx + dx * scale + nx).clamp(min_x, max_x);
        let py = (cy + dy * scale + ny).clamp(min_y, max_y);
        out.push(Point2::new(px, py));
    }
    out
}

/// Generate a "two interlocking rings" 3D dataset, the 3D analogue of
/// the two-moons problem. One ring lies in the xy-plane, the other in
/// the yz-plane, threaded through it so neither is linearly separable
/// from the other.
pub fn generate_moons_3d<R: Rng>(
    rng: &mut R,
    n: usize,
    noise: f32,
    bounds: (f32, f32, f32, f32, f32, f32),
) -> Vec<Point3> {
    let (min_x, min_y, min_z, max_x, max_y, max_z) = bounds;
    let cx = (min_x + max_x) * 0.5;
    let cy = (min_y + max_y) * 0.5;
    let cz = (min_z + max_z) * 0.5;
    let radius = (max_x - min_x)
        .min(max_y - min_y)
        .min(max_z - min_z)
        * 0.30;
    let noise_std = radius * noise;

    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let upper = i % 2 == 0;
        let theta: f32 = rng.gen_range(0.0..TAU);
        let (x, y, z) = if upper {
            // Ring 1: in the xy-plane, offset by -radius/2 in x.
            (theta.cos() * radius - radius * 0.5, theta.sin() * radius, 0.0)
        } else {
            // Ring 2: in the yz-plane, offset by +radius/2 in x.
            (radius * 0.5, theta.cos() * radius, theta.sin() * radius)
        };
        let nx = sample_normal(rng, 0.0, noise_std);
        let ny = sample_normal(rng, 0.0, noise_std);
        let nz = sample_normal(rng, 0.0, noise_std);
        out.push(Point3::new(
            (cx + x + nx).clamp(min_x, max_x),
            (cy + y + ny).clamp(min_y, max_y),
            (cz + z + nz).clamp(min_z, max_z),
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn normal_distribution_mean_is_close() {
        let mut rng = StdRng::seed_from_u64(0);
        let n = 5000;
        let mean: f32 =
            (0..n).map(|_| sample_normal(&mut rng, 10.0, 2.0)).sum::<f32>() / n as f32;
        // With 5000 samples, mean should be within ~0.3 of 10.0
        assert!((mean - 10.0).abs() < 0.5, "got mean {mean}");
    }

    #[test]
    fn normal_distribution_std_is_close() {
        let mut rng = StdRng::seed_from_u64(0);
        let n = 5000;
        let samples: Vec<f32> = (0..n).map(|_| sample_normal(&mut rng, 0.0, 3.0)).collect();
        let mean: f32 = samples.iter().sum::<f32>() / n as f32;
        let var: f32 =
            samples.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / (n - 1) as f32;
        let std = var.sqrt();
        // Should be within ~10% of 3.0
        assert!((std - 3.0).abs() < 0.4, "got std {std}");
    }

    #[test]
    fn blobs_2d_produces_n_points_in_bounds() {
        let mut rng = StdRng::seed_from_u64(42);
        let pts = generate_blobs_2d(&mut rng, 100, 3, 20.0, (0.0, 0.0, 1000.0, 700.0));
        assert_eq!(pts.len(), 100);
        for p in &pts {
            assert!((0.0..=1000.0).contains(&p.x));
            assert!((0.0..=700.0).contains(&p.y));
        }
    }

    #[test]
    fn blobs_3d_produces_n_points_in_bounds() {
        let mut rng = StdRng::seed_from_u64(0);
        let pts = generate_blobs_3d(&mut rng, 60, 4, 0.3, (-5.0, -5.0, -5.0, 5.0, 5.0, 5.0));
        assert_eq!(pts.len(), 60);
        for p in &pts {
            assert!((-5.0..=5.0).contains(&p.x));
            assert!((-5.0..=5.0).contains(&p.y));
            assert!((-5.0..=5.0).contains(&p.z));
        }
    }

    #[test]
    fn moons_produces_n_points_in_bounds() {
        let mut rng = StdRng::seed_from_u64(0);
        let pts = generate_moons_2d(&mut rng, 100, 0.1, (0.0, 0.0, 1000.0, 700.0));
        assert_eq!(pts.len(), 100);
        for p in &pts {
            assert!((0.0..=1000.0).contains(&p.x));
            assert!((0.0..=700.0).contains(&p.y));
        }
    }

    #[test]
    fn moons_3d_produces_n_points_in_bounds() {
        let mut rng = StdRng::seed_from_u64(0);
        let pts = generate_moons_3d(&mut rng, 80, 0.05, (-5.0, -5.0, -5.0, 5.0, 5.0, 5.0));
        assert_eq!(pts.len(), 80);
        for p in &pts {
            assert!((-5.0..=5.0).contains(&p.x));
            assert!((-5.0..=5.0).contains(&p.y));
            assert!((-5.0..=5.0).contains(&p.z));
        }
    }

    #[test]
    fn blobs_2d_deterministic_with_seed() {
        let mut a = StdRng::seed_from_u64(7);
        let mut b = StdRng::seed_from_u64(7);
        let bounds = (0.0, 0.0, 100.0, 100.0);
        let p1 = generate_blobs_2d(&mut a, 20, 3, 5.0, bounds);
        let p2 = generate_blobs_2d(&mut b, 20, 3, 5.0, bounds);
        assert_eq!(p1, p2);
    }
}
