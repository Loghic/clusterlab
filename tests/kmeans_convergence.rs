//! Integration tests: end-to-end k-means runs on known datasets.

use clusterlab::geometry::{Point2, Point3};
use clusterlab::kmeans::{
    run_to_completion_2d, step_3d, KMeansState2D, KMeansState3D, StepOutcome,
};
use rand::{rngs::StdRng, SeedableRng};

#[test]
fn three_well_separated_clusters_2d() {
    let mut pts = Vec::new();
    // Cluster A around (0, 0)
    for i in 0..15 {
        let f = i as f32 * 0.01;
        pts.push(Point2::new(f, -f));
    }
    // Cluster B around (50, 0)
    for i in 0..15 {
        let f = i as f32 * 0.01;
        pts.push(Point2::new(50.0 + f, f));
    }
    // Cluster C around (25, 50)
    for i in 0..15 {
        let f = i as f32 * 0.01;
        pts.push(Point2::new(25.0 + f, 50.0 - f));
    }

    let centroids = vec![
        Point2::new(5.0, 5.0),
        Point2::new(45.0, 5.0),
        Point2::new(20.0, 45.0),
    ];
    let mut state = KMeansState2D::new(pts, centroids);
    let mut rng = StdRng::seed_from_u64(0);

    run_to_completion_2d(&mut state, 100, 1e-4, &mut rng);

    let targets = [
        Point2::new(0.0, 0.0),
        Point2::new(50.0, 0.0),
        Point2::new(25.0, 50.0),
    ];
    for t in &targets {
        let found = state.centroids.iter().any(|c| c.distance(t) < 1.0);
        assert!(
            found,
            "expected a centroid near {t:?}; got {:?}",
            state.centroids
        );
    }
}

#[test]
fn invariant_holds_across_many_steps_2d() {
    let pts: Vec<Point2> = (0..100)
        .map(|i| Point2::new((i % 10) as f32 * 5.0, (i / 10) as f32 * 5.0))
        .collect();
    let cs = vec![
        Point2::new(0.0, 0.0),
        Point2::new(50.0, 50.0),
        Point2::new(25.0, 25.0),
    ];
    let mut state = KMeansState2D::new(pts, cs);
    let mut rng = StdRng::seed_from_u64(7);

    for _ in 0..30 {
        run_to_completion_2d(&mut state, 1, 1e-6, &mut rng);
        assert_eq!(state.assignments.len(), state.points.len());
        for &a in &state.assignments {
            assert!(a < state.k());
        }
        for c in &state.centroids {
            assert!(c.x.is_finite() && c.y.is_finite());
        }
    }
}

#[test]
fn converges_in_few_iterations_2d() {
    let mut pts = Vec::new();
    for i in 0..10 {
        let f = i as f32 * 0.001;
        pts.push(Point2::new(f, f));
        pts.push(Point2::new(100.0 + f, 100.0 + f));
    }
    let cs = vec![Point2::new(0.5, 0.5), Point2::new(99.5, 99.5)];
    let mut state = KMeansState2D::new(pts, cs);
    let mut rng = StdRng::seed_from_u64(0);
    run_to_completion_2d(&mut state, 50, 1e-4, &mut rng);
    // With such well-separated, near-identical clusters convergence is fast.
    assert!(state.iteration < 10, "took {} iterations", state.iteration);
}

#[test]
fn three_clusters_3d() {
    let mut pts = Vec::new();
    for i in 0..10 {
        let f = i as f32 * 0.01;
        pts.push(Point3::new(f, f, f));
        pts.push(Point3::new(20.0 + f, 0.0, 0.0));
        pts.push(Point3::new(0.0, 20.0, 0.0));
    }
    let cs = vec![
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(19.0, 1.0, 1.0),
        Point3::new(1.0, 19.0, 1.0),
    ];
    let mut state = KMeansState3D::new(pts, cs);
    let mut rng = StdRng::seed_from_u64(0);

    for _ in 0..100 {
        if step_3d(&mut state, 1e-4, &mut rng) == StepOutcome::Converged {
            break;
        }
    }

    let targets = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(20.0, 0.0, 0.0),
        Point3::new(0.0, 20.0, 0.0),
    ];
    for t in &targets {
        let found = state.centroids.iter().any(|c| c.distance(t) < 1.0);
        assert!(
            found,
            "expected a centroid near {t:?}; got {:?}",
            state.centroids
        );
    }
}
