//! 3D Voronoi bisector planes.
//!
//! True 3D Voronoi *cells* are convex polyhedra and computing them is
//! non-trivial. For visualization we cheat: between every pair of centroids
//! we draw the perpendicular bisector plane, clipped to the world bounding
//! cube. This isn't strictly the Voronoi tessellation (it doesn't trim
//! planes against other planes), but it's visually close enough to give a
//! sense of the partition — and far cheaper.

use crate::geometry::Point3;

/// A planar quad to render as a translucent face.
#[derive(Debug, Clone, Copy)]
pub struct PlaneQuad {
    /// Four corners in CCW order viewed from the +normal side.
    pub corners: [Point3; 4],
    /// Cluster index pair that produced this bisector (lower, higher).
    pub pair: (usize, usize),
}

/// Compute one bisector plane between each pair of centroids, clipped to
/// the axis-aligned cube `(-half..=half)` on each axis.
///
/// Returns quads suitable for `draw_plane`-like rendering. Empty if there
/// are fewer than two centroids.
pub fn bisector_planes(centroids: &[Point3], half: f32) -> Vec<PlaneQuad> {
    if centroids.len() < 2 {
        return Vec::new();
    }
    let mut out = Vec::new();
    for i in 0..centroids.len() {
        for j in (i + 1)..centroids.len() {
            if let Some(quad) = bisector_quad(centroids[i], centroids[j], half) {
                out.push(PlaneQuad {
                    corners: quad,
                    pair: (i, j),
                });
            }
        }
    }
    out
}

/// Compute a single bisector quad between two points.
///
/// The plane passes through the midpoint with normal `b - a`. We pick two
/// in-plane axes perpendicular to the normal and place corners at a fixed
/// radius (the world diagonal), then clip the quad to the world cube by
/// sampling — good enough for visualization.
fn bisector_quad(a: Point3, b: Point3, half: f32) -> Option<[Point3; 4]> {
    let mid = Point3::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5, (a.z + b.z) * 0.5);
    let n = Point3::new(b.x - a.x, b.y - a.y, b.z - a.z);
    let n_len = (n.x * n.x + n.y * n.y + n.z * n.z).sqrt();
    if n_len < 1e-6 {
        // Coincident centroids — bisector is undefined.
        return None;
    }
    let n = Point3::new(n.x / n_len, n.y / n_len, n.z / n_len);

    // Build an orthonormal basis (u, v) perpendicular to n.
    // Pick a helper axis that isn't parallel to n.
    let helper = if n.x.abs() < 0.9 {
        Point3::new(1.0, 0.0, 0.0)
    } else {
        Point3::new(0.0, 1.0, 0.0)
    };
    let u = cross(n, helper);
    let u = normalize(u)?;
    let v = cross(n, u);
    let v = normalize(v)?;

    // Use the world diagonal as the quad radius — guarantees the quad spans
    // the world cube, then we clamp corners to the cube to clip the visible
    // area roughly. This is an approximation; precise polygon clipping
    // against the cube would require Sutherland–Hodgman.
    let r = half * 3.0;
    let raw = [
        offset(mid, u, -r, v, -r),
        offset(mid, u, r, v, -r),
        offset(mid, u, r, v, r),
        offset(mid, u, -r, v, r),
    ];
    Some([
        clamp_to_cube(raw[0], half),
        clamp_to_cube(raw[1], half),
        clamp_to_cube(raw[2], half),
        clamp_to_cube(raw[3], half),
    ])
}

fn cross(a: Point3, b: Point3) -> Point3 {
    Point3::new(
        a.y * b.z - a.z * b.y,
        a.z * b.x - a.x * b.z,
        a.x * b.y - a.y * b.x,
    )
}

fn normalize(p: Point3) -> Option<Point3> {
    let len = (p.x * p.x + p.y * p.y + p.z * p.z).sqrt();
    if len < 1e-6 {
        None
    } else {
        Some(Point3::new(p.x / len, p.y / len, p.z / len))
    }
}

fn offset(c: Point3, u: Point3, su: f32, v: Point3, sv: f32) -> Point3 {
    Point3::new(
        c.x + u.x * su + v.x * sv,
        c.y + u.y * su + v.y * sv,
        c.z + u.z * su + v.z * sv,
    )
}

fn clamp_to_cube(p: Point3, half: f32) -> Point3 {
    Point3::new(
        p.x.clamp(-half, half),
        p.y.clamp(-half, half),
        p.z.clamp(-half, half),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_for_fewer_than_two() {
        assert!(bisector_planes(&[], 5.0).is_empty());
        assert!(bisector_planes(&[Point3::ZERO], 5.0).is_empty());
    }

    #[test]
    fn one_plane_for_two_centroids() {
        let cs = vec![Point3::new(-1.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)];
        let planes = bisector_planes(&cs, 5.0);
        assert_eq!(planes.len(), 1);
        // Midpoint at origin -> every corner should have x ≈ 0 (the bisector
        // is the yz-plane at x = 0).
        for c in planes[0].corners {
            assert!(c.x.abs() < 1e-4, "expected x ≈ 0, got {}", c.x);
        }
    }

    #[test]
    fn three_planes_for_three_centroids() {
        let cs = vec![
            Point3::new(-2.0, 0.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
            Point3::new(0.0, 3.0, 0.0),
        ];
        let planes = bisector_planes(&cs, 5.0);
        assert_eq!(planes.len(), 3); // C(3, 2) = 3
    }

    #[test]
    fn coincident_centroids_produce_no_plane() {
        let cs = vec![Point3::new(1.0, 1.0, 1.0), Point3::new(1.0, 1.0, 1.0)];
        let planes = bisector_planes(&cs, 5.0);
        assert!(planes.is_empty());
    }

    #[test]
    fn corners_stay_inside_cube() {
        let cs = vec![Point3::new(-1.5, 0.5, -0.5), Point3::new(1.5, -0.5, 0.5)];
        let half = 3.0;
        let planes = bisector_planes(&cs, half);
        for quad in &planes {
            for c in quad.corners {
                assert!(c.x.abs() <= half + 1e-4);
                assert!(c.y.abs() <= half + 1e-4);
                assert!(c.z.abs() <= half + 1e-4);
            }
        }
    }
}
