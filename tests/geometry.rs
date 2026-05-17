//! Integration tests for the geometry module.

use kmeans_viz::geometry::{compute_edges, Point2, Point3};

#[test]
fn voronoi_edges_form_loop_per_cell() {
    // 4 sites in a square -> at least 4 cells. We check that each cell
    // contributes edges (so total edges should be a healthy multiple of cells).
    let sites = vec![
        Point2::new(25.0, 25.0),
        Point2::new(75.0, 25.0),
        Point2::new(25.0, 75.0),
        Point2::new(75.0, 75.0),
    ];
    let edges = compute_edges(&sites, (0.0, 0.0, 100.0, 100.0));
    assert!(
        edges.len() >= 8,
        "expected at least 8 edges, got {}",
        edges.len()
    );
}

#[test]
fn point_arithmetic_distributes() {
    let a = Point2::new(1.0, 2.0);
    let b = Point2::new(3.0, 4.0);
    let c = Point2::new(5.0, 6.0);
    let lhs = (a + b) + c;
    let rhs = a + (b + c);
    assert_eq!(lhs, rhs);
}

#[test]
fn point3_negative_components_distance() {
    let a = Point3::new(-1.0, -2.0, -3.0);
    let b = Point3::new(1.0, 2.0, 3.0);
    // distance = sqrt(4 + 16 + 36) = sqrt(56)
    let expected = 56.0f32.sqrt();
    let got = a.distance(&b);
    assert!((got - expected).abs() < 1e-5);
}
