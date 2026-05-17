//! State containers for an in-progress k-means run.

use crate::geometry::{Point2, Point3};

/// State of a 2D k-means run.
///
/// Invariant after any successful step: `assignments.len() == points.len()`
/// and every value in `assignments` is in `0..centroids.len()`.
#[derive(Debug, Clone)]
pub struct KMeansState2D {
    pub points: Vec<Point2>,
    pub centroids: Vec<Point2>,
    pub assignments: Vec<usize>,
    pub iteration: usize,
}

impl KMeansState2D {
    pub fn new(points: Vec<Point2>, centroids: Vec<Point2>) -> Self {
        let n = points.len();
        Self {
            points,
            centroids,
            assignments: vec![0; n],
            iteration: 0,
        }
    }

    pub fn k(&self) -> usize {
        self.centroids.len()
    }
}

/// State of a 3D k-means run.
#[derive(Debug, Clone)]
pub struct KMeansState3D {
    pub points: Vec<Point3>,
    pub centroids: Vec<Point3>,
    pub assignments: Vec<usize>,
    pub iteration: usize,
}

impl KMeansState3D {
    pub fn new(points: Vec<Point3>, centroids: Vec<Point3>) -> Self {
        let n = points.len();
        Self {
            points,
            centroids,
            assignments: vec![0; n],
            iteration: 0,
        }
    }

    pub fn k(&self) -> usize {
        self.centroids.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_2d_initial_assignments_match_points() {
        let pts = vec![Point2::new(1.0, 2.0), Point2::new(3.0, 4.0)];
        let cs = vec![Point2::new(0.0, 0.0)];
        let s = KMeansState2D::new(pts, cs);
        assert_eq!(s.assignments.len(), 2);
        assert_eq!(s.k(), 1);
        assert_eq!(s.iteration, 0);
    }

    #[test]
    fn state_3d_initial_assignments_match_points() {
        let pts = vec![Point3::new(1.0, 2.0, 3.0)];
        let cs = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0)];
        let s = KMeansState3D::new(pts, cs);
        assert_eq!(s.assignments.len(), 1);
        assert_eq!(s.k(), 2);
    }
}
