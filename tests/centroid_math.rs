//! Integration tests for centroid math.

use kmeans_viz::geometry::{Point2, Point3};
use kmeans_viz::kmeans::{mean_2d, mean_3d};

#[test]
fn centroid_of_symmetric_cloud_is_origin() {
    let pts = vec![
        Point2::new(1.0, 0.0),
        Point2::new(-1.0, 0.0),
        Point2::new(0.0, 1.0),
        Point2::new(0.0, -1.0),
    ];
    let c = mean_2d(&pts).unwrap();
    assert!(c.x.abs() < 1e-6);
    assert!(c.y.abs() < 1e-6);
}

#[test]
fn centroid_shifts_with_outlier() {
    let tight = vec![Point2::new(0.0, 0.0); 9];
    let mut with_outlier = tight.clone();
    with_outlier.push(Point2::new(100.0, 100.0));
    let c_tight = mean_2d(&tight).unwrap();
    let c_out = mean_2d(&with_outlier).unwrap();
    // Single outlier in 10 points should pull centroid by 10 units.
    assert!((c_out.x - c_tight.x - 10.0).abs() < 1e-4);
}

#[test]
fn centroid_3d_symmetric() {
    let pts = vec![
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(-1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, -1.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(0.0, 0.0, -1.0),
    ];
    let c = mean_3d(&pts).unwrap();
    assert!(c.x.abs() < 1e-6);
    assert!(c.y.abs() < 1e-6);
    assert!(c.z.abs() < 1e-6);
}
