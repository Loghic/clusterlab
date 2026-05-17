//! The k-means iteration loop and convergence check.
//!
//! Empty cluster strategy: re-seed an empty cluster's centroid to a
//! randomly chosen existing point. This keeps `k` stable and avoids
//! division by zero. Documented in the README.

use crate::geometry::{Point2, Point3};
use crate::kmeans::centroid::{mean_2d, mean_3d};
use crate::kmeans::distance::{Distance, Euclidean};
use crate::kmeans::state::{KMeansState2D, KMeansState3D};
use rand::seq::SliceRandom;
use rand::Rng;

/// Outcome of a single iteration step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StepOutcome {
    /// Centroids moved by more than the threshold; keep going.
    Moved,
    /// Centroids settled; algorithm has converged.
    Converged,
}

/// Run one iteration of k-means on a 2D state: assign points, then recompute
/// centroids. Returns `Converged` if no centroid moved more than `threshold`.
pub fn step_2d<R: Rng>(state: &mut KMeansState2D, threshold: f32, rng: &mut R) -> StepOutcome {
    assign_2d(state, &Euclidean);
    let max_shift = recompute_centroids_2d(state, rng);
    state.iteration += 1;
    if max_shift <= threshold {
        StepOutcome::Converged
    } else {
        StepOutcome::Moved
    }
}

/// Run one iteration of k-means on a 3D state.
pub fn step_3d<R: Rng>(state: &mut KMeansState3D, threshold: f32, rng: &mut R) -> StepOutcome {
    assign_3d(state, &Euclidean);
    let max_shift = recompute_centroids_3d(state, rng);
    state.iteration += 1;
    if max_shift <= threshold {
        StepOutcome::Converged
    } else {
        StepOutcome::Moved
    }
}

/// Run k-means to completion or until `max_iters` is hit.
pub fn run_to_completion_2d<R: Rng>(
    state: &mut KMeansState2D,
    max_iters: usize,
    threshold: f32,
    rng: &mut R,
) {
    for _ in 0..max_iters {
        if step_2d(state, threshold, rng) == StepOutcome::Converged {
            break;
        }
    }
}

/// Assign every point to its nearest centroid.
pub fn assign_2d<D: Distance<Point2>>(state: &mut KMeansState2D, metric: &D) {
    debug_assert!(
        !state.centroids.is_empty(),
        "cannot assign with zero centroids"
    );
    for (idx, p) in state.points.iter().enumerate() {
        let mut best = 0usize;
        let mut best_d = metric.distance_squared(p, &state.centroids[0]);
        for (ci, c) in state.centroids.iter().enumerate().skip(1) {
            let d = metric.distance_squared(p, c);
            if d < best_d {
                best_d = d;
                best = ci;
            }
        }
        state.assignments[idx] = best;
    }
}

/// 3D version of [`assign_2d`].
pub fn assign_3d<D: Distance<Point3>>(state: &mut KMeansState3D, metric: &D) {
    debug_assert!(
        !state.centroids.is_empty(),
        "cannot assign with zero centroids"
    );
    for (idx, p) in state.points.iter().enumerate() {
        let mut best = 0usize;
        let mut best_d = metric.distance_squared(p, &state.centroids[0]);
        for (ci, c) in state.centroids.iter().enumerate().skip(1) {
            let d = metric.distance_squared(p, c);
            if d < best_d {
                best_d = d;
                best = ci;
            }
        }
        state.assignments[idx] = best;
    }
}

/// Recompute centroids and return the largest distance any centroid moved.
/// Empty clusters get re-seeded to a random existing point.
fn recompute_centroids_2d<R: Rng>(state: &mut KMeansState2D, rng: &mut R) -> f32 {
    let k = state.centroids.len();
    let mut buckets: Vec<Vec<Point2>> = vec![Vec::new(); k];
    for (i, p) in state.points.iter().enumerate() {
        buckets[state.assignments[i]].push(*p);
    }

    let mut max_shift = 0.0f32;
    for (ci, bucket) in buckets.iter().enumerate() {
        let new_c = match mean_2d(bucket) {
            Some(c) => c,
            None => {
                // Empty cluster: re-seed to a random existing point.
                if let Some(&p) = state.points.choose(rng) {
                    p
                } else {
                    state.centroids[ci]
                }
            }
        };
        let shift = state.centroids[ci].distance(&new_c);
        if shift > max_shift {
            max_shift = shift;
        }
        state.centroids[ci] = new_c;
    }
    max_shift
}

fn recompute_centroids_3d<R: Rng>(state: &mut KMeansState3D, rng: &mut R) -> f32 {
    let k = state.centroids.len();
    let mut buckets: Vec<Vec<Point3>> = vec![Vec::new(); k];
    for (i, p) in state.points.iter().enumerate() {
        buckets[state.assignments[i]].push(*p);
    }

    let mut max_shift = 0.0f32;
    for (ci, bucket) in buckets.iter().enumerate() {
        let new_c = match mean_3d(bucket) {
            Some(c) => c,
            None => {
                if let Some(&p) = state.points.choose(rng) {
                    p
                } else {
                    state.centroids[ci]
                }
            }
        };
        let shift = state.centroids[ci].distance(&new_c);
        if shift > max_shift {
            max_shift = shift;
        }
        state.centroids[ci] = new_c;
    }
    max_shift
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use rand::{rngs::StdRng, SeedableRng};

    /// Two well-separated clusters: algorithm should find their true centers.
    #[test]
    fn converges_on_two_separated_clusters() {
        let mut pts = Vec::new();
        // Cluster A around (0, 0)
        for &(x, y) in &[(0.0, 0.0), (0.1, 0.1), (-0.1, 0.0), (0.0, -0.1)] {
            pts.push(Point2::new(x, y));
        }
        // Cluster B around (10, 10)
        for &(x, y) in &[(10.0, 10.0), (10.1, 9.9), (9.9, 10.1), (10.0, 10.2)] {
            pts.push(Point2::new(x, y));
        }
        let centroids = vec![Point2::new(1.0, 1.0), Point2::new(9.0, 9.0)];
        let mut state = KMeansState2D::new(pts, centroids);
        let mut rng = StdRng::seed_from_u64(0);

        run_to_completion_2d(&mut state, 100, 1e-6, &mut rng);

        // The two final centroids should be near (0,0) and (10,10) in some order.
        let near_origin = state
            .centroids
            .iter()
            .any(|c| c.distance(&Point2::ZERO) < 0.3);
        let near_ten = state
            .centroids
            .iter()
            .any(|c| c.distance(&Point2::new(10.0, 10.0)) < 0.3);
        assert!(
            near_origin,
            "expected a centroid near (0,0): {:?}",
            state.centroids
        );
        assert!(
            near_ten,
            "expected a centroid near (10,10): {:?}",
            state.centroids
        );
    }

    #[test]
    fn invariant_every_point_assigned() {
        let pts = vec![
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 1.0),
            Point2::new(5.0, 5.0),
        ];
        let cs = vec![Point2::new(0.0, 0.0), Point2::new(5.0, 5.0)];
        let mut state = KMeansState2D::new(pts, cs);
        let mut rng = StdRng::seed_from_u64(0);
        step_2d(&mut state, 1e-6, &mut rng);
        assert_eq!(state.assignments.len(), state.points.len());
        for a in &state.assignments {
            assert!(
                *a < state.k(),
                "assignment {} out of range for k={}",
                a,
                state.k()
            );
        }
    }

    #[test]
    fn deterministic_given_seed() {
        let pts: Vec<Point2> = (0..20)
            .map(|i| Point2::new(i as f32, (i * 2) as f32))
            .collect();
        let cs = vec![
            Point2::new(0.0, 0.0),
            Point2::new(10.0, 20.0),
            Point2::new(20.0, 40.0),
        ];

        let mut a = KMeansState2D::new(pts.clone(), cs.clone());
        let mut ra = StdRng::seed_from_u64(123);
        run_to_completion_2d(&mut a, 50, 1e-6, &mut ra);

        let mut b = KMeansState2D::new(pts, cs);
        let mut rb = StdRng::seed_from_u64(123);
        run_to_completion_2d(&mut b, 50, 1e-6, &mut rb);

        assert_eq!(a.centroids, b.centroids);
        assert_eq!(a.assignments, b.assignments);
    }

    #[test]
    fn empty_cluster_gets_reseeded_not_nan() {
        // Put all points on one side; second centroid will end up empty.
        let pts = vec![
            Point2::new(0.0, 0.0),
            Point2::new(0.1, 0.0),
            Point2::new(0.0, 0.1),
        ];
        let cs = vec![Point2::new(0.0, 0.0), Point2::new(100.0, 100.0)];
        let mut state = KMeansState2D::new(pts, cs);
        let mut rng = StdRng::seed_from_u64(42);
        step_2d(&mut state, 1e-6, &mut rng);
        // No NaNs in centroids.
        for c in &state.centroids {
            assert!(c.x.is_finite() && c.y.is_finite(), "got NaN/inf in {:?}", c);
        }
    }

    #[test]
    fn iteration_counter_increments() {
        let pts = vec![Point2::new(0.0, 0.0), Point2::new(1.0, 1.0)];
        let cs = vec![Point2::new(0.5, 0.5)];
        let mut state = KMeansState2D::new(pts, cs);
        let mut rng = StdRng::seed_from_u64(0);
        assert_eq!(state.iteration, 0);
        step_2d(&mut state, 1e-6, &mut rng);
        assert_eq!(state.iteration, 1);
        step_2d(&mut state, 1e-6, &mut rng);
        assert_eq!(state.iteration, 2);
    }

    #[test]
    fn converged_when_centroids_settle() {
        // Single tight cluster; one centroid in the middle won't move.
        let pts = vec![
            Point2::new(0.0, 0.0),
            Point2::new(0.0, 0.0),
            Point2::new(0.0, 0.0),
        ];
        let cs = vec![Point2::new(0.0, 0.0)];
        let mut state = KMeansState2D::new(pts, cs);
        let mut rng = StdRng::seed_from_u64(0);
        let outcome = step_2d(&mut state, 1e-6, &mut rng);
        assert_eq!(outcome, StepOutcome::Converged);
    }

    #[test]
    fn assign_2d_picks_nearest() {
        let pts = vec![Point2::new(0.0, 0.0), Point2::new(10.0, 10.0)];
        let cs = vec![Point2::new(0.5, 0.5), Point2::new(9.0, 9.0)];
        let mut state = KMeansState2D::new(pts, cs);
        assign_2d(&mut state, &Euclidean);
        assert_eq!(state.assignments, vec![0, 1]);
    }

    #[test]
    fn step_3d_basic_run() {
        let pts: Vec<Point3> = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(0.1, 0.0, 0.0),
            Point3::new(10.0, 10.0, 10.0),
            Point3::new(10.1, 10.0, 10.0),
        ];
        let cs = vec![Point3::new(1.0, 1.0, 1.0), Point3::new(9.0, 9.0, 9.0)];
        let mut state = KMeansState3D::new(pts, cs);
        let mut rng = StdRng::seed_from_u64(0);
        for _ in 0..20 {
            if step_3d(&mut state, 1e-6, &mut rng) == StepOutcome::Converged {
                break;
            }
        }
        let near_origin = state
            .centroids
            .iter()
            .any(|c| c.distance(&Point3::ZERO) < 0.3);
        let near_ten = state
            .centroids
            .iter()
            .any(|c| c.distance(&Point3::new(10.0, 10.0, 10.0)) < 0.3);
        assert!(near_origin);
        assert!(near_ten);
    }

    #[test]
    fn single_centroid_means_all_points() {
        let pts = vec![
            Point2::new(0.0, 0.0),
            Point2::new(2.0, 0.0),
            Point2::new(0.0, 2.0),
            Point2::new(2.0, 2.0),
        ];
        let cs = vec![Point2::new(0.0, 0.0)];
        let mut state = KMeansState2D::new(pts, cs);
        let mut rng = StdRng::seed_from_u64(0);
        run_to_completion_2d(&mut state, 50, 1e-9, &mut rng);
        assert_relative_eq!(state.centroids[0].x, 1.0, epsilon = 1e-5);
        assert_relative_eq!(state.centroids[0].y, 1.0, epsilon = 1e-5);
    }
}
