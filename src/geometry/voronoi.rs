//! Voronoi diagram computation for 2D cluster boundaries.

use crate::geometry::point::Point2;
use voronoice::{BoundingBox, Point, VoronoiBuilder};

/// A single edge of the Voronoi diagram, as two endpoints.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoronoiEdge {
    pub from: Point2,
    pub to: Point2,
}

/// Compute Voronoi edges from a set of site points, clipped to `bounds`.
///
/// `bounds` is `(min_x, min_y, max_x, max_y)`. Returns an empty vector if
/// fewer than two sites are given.
///
/// For exactly 2 sites we compute the perpendicular bisector manually
/// (voronoice / delaunator can't triangulate with only 2 sites). For 3+
/// we delegate to voronoice.
pub fn compute_edges(sites: &[Point2], bounds: (f32, f32, f32, f32)) -> Vec<VoronoiEdge> {
    match sites.len() {
        0 | 1 => Vec::new(),
        2 => two_site_bisector(sites[0], sites[1], bounds)
            .into_iter()
            .collect(),
        _ => voronoice_edges(sites, bounds),
    }
}

/// Perpendicular bisector of the segment a–b, clipped to the rectangle
/// `(min_x, min_y, max_x, max_y)`. Returns the visible segment or `None`
/// if it doesn't intersect the rectangle (or `a == b`).
fn two_site_bisector(
    a: Point2,
    b: Point2,
    bounds: (f32, f32, f32, f32),
) -> Option<VoronoiEdge> {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    if dx.abs() < 1e-6 && dy.abs() < 1e-6 {
        return None; // coincident points have no bisector
    }
    let mid_x = (a.x + b.x) * 0.5;
    let mid_y = (a.y + b.y) * 0.5;

    // Bisector direction is perpendicular to (dx, dy).
    let dir_x = -dy;
    let dir_y = dx;

    clip_ray_to_rect(mid_x, mid_y, dir_x, dir_y, bounds)
}

/// Liang-Barsky line clipping for an infinite line through (px, py) with
/// direction (dx, dy), against the rectangle in `bounds`. Returns the
/// clipped segment endpoints or `None` if the line misses the rect.
fn clip_ray_to_rect(
    px: f32,
    py: f32,
    dx: f32,
    dy: f32,
    bounds: (f32, f32, f32, f32),
) -> Option<VoronoiEdge> {
    let (min_x, min_y, max_x, max_y) = bounds;
    let mut t_min = f32::NEG_INFINITY;
    let mut t_max = f32::INFINITY;

    for (p_axis, d_axis, lo, hi) in [(px, dx, min_x, max_x), (py, dy, min_y, max_y)] {
        if d_axis.abs() < 1e-9 {
            // Line parallel to this axis — must already be inside the slab.
            if p_axis < lo || p_axis > hi {
                return None;
            }
        } else {
            let t1 = (lo - p_axis) / d_axis;
            let t2 = (hi - p_axis) / d_axis;
            let (t_near, t_far) = if t1 < t2 { (t1, t2) } else { (t2, t1) };
            if t_near > t_min {
                t_min = t_near;
            }
            if t_far < t_max {
                t_max = t_far;
            }
            if t_min > t_max {
                return None;
            }
        }
    }

    Some(VoronoiEdge {
        from: Point2::new(px + dx * t_min, py + dy * t_min),
        to: Point2::new(px + dx * t_max, py + dy * t_max),
    })
}

fn voronoice_edges(sites: &[Point2], bounds: (f32, f32, f32, f32)) -> Vec<VoronoiEdge> {
    let (min_x, min_y, max_x, max_y) = bounds;
    let center_x = ((min_x + max_x) / 2.0) as f64;
    let center_y = ((min_y + max_y) / 2.0) as f64;
    let width = (max_x - min_x) as f64;
    let height = (max_y - min_y) as f64;

    let voro_sites: Vec<Point> = sites
        .iter()
        .map(|p| Point { x: p.x as f64, y: p.y as f64 })
        .collect();
    let bbox = BoundingBox::new(Point { x: center_x, y: center_y }, width, height);

    let diagram = match VoronoiBuilder::default()
        .set_sites(voro_sites)
        .set_bounding_box(bbox)
        .build()
    {
        Some(d) => d,
        None => return Vec::new(),
    };

    let mut edges = Vec::new();
    for cell in diagram.iter_cells() {
        let verts: Vec<&Point> = cell.iter_vertices().collect();
        let n = verts.len();
        if n < 2 {
            continue;
        }
        for (i, a) in verts.iter().enumerate() {
            let b = verts[(i + 1) % n];
            edges.push(VoronoiEdge {
                from: Point2::new(a.x as f32, a.y as f32),
                to: Point2::new(b.x as f32, b.y as f32),
            });
        }
    }
    edges
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_for_zero_sites() {
        let edges = compute_edges(&[], (0.0, 0.0, 100.0, 100.0));
        assert!(edges.is_empty());
    }

    #[test]
    fn empty_for_one_site() {
        let edges = compute_edges(&[Point2::new(50.0, 50.0)], (0.0, 0.0, 100.0, 100.0));
        assert!(edges.is_empty());
    }

    #[test]
    fn two_horizontal_sites_produce_vertical_bisector() {
        let sites = vec![Point2::new(25.0, 50.0), Point2::new(75.0, 50.0)];
        let edges = compute_edges(&sites, (0.0, 0.0, 100.0, 100.0));
        assert_eq!(edges.len(), 1);
        let e = edges[0];
        // Vertical line at x = 50.
        assert!((e.from.x - 50.0).abs() < 1e-3);
        assert!((e.to.x - 50.0).abs() < 1e-3);
        // Spans the full height (within bounds).
        let y_min = e.from.y.min(e.to.y);
        let y_max = e.from.y.max(e.to.y);
        assert!(y_min <= 0.5);
        assert!(y_max >= 99.5);
    }

    #[test]
    fn two_vertical_sites_produce_horizontal_bisector() {
        let sites = vec![Point2::new(50.0, 25.0), Point2::new(50.0, 75.0)];
        let edges = compute_edges(&sites, (0.0, 0.0, 100.0, 100.0));
        assert_eq!(edges.len(), 1);
        let e = edges[0];
        assert!((e.from.y - 50.0).abs() < 1e-3);
        assert!((e.to.y - 50.0).abs() < 1e-3);
    }

    #[test]
    fn two_diagonal_sites_produce_perpendicular_bisector() {
        let sites = vec![Point2::new(0.0, 0.0), Point2::new(100.0, 100.0)];
        let edges = compute_edges(&sites, (0.0, 0.0, 100.0, 100.0));
        assert_eq!(edges.len(), 1);
        // Bisector goes through (50, 50) perpendicular to direction (1, 1)
        // i.e. line y = -x + 100 — should run from (0, 100) to (100, 0).
        let e = edges[0];
        let touches_top_left = (e.from.x < 1.0 && e.from.y > 99.0)
            || (e.to.x < 1.0 && e.to.y > 99.0);
        let touches_bottom_right = (e.from.x > 99.0 && e.from.y < 1.0)
            || (e.to.x > 99.0 && e.to.y < 1.0);
        assert!(touches_top_left, "got {:?}", e);
        assert!(touches_bottom_right, "got {:?}", e);
    }

    #[test]
    fn coincident_two_sites_produce_no_edge() {
        let sites = vec![Point2::new(50.0, 50.0), Point2::new(50.0, 50.0)];
        let edges = compute_edges(&sites, (0.0, 0.0, 100.0, 100.0));
        assert!(edges.is_empty());
    }

    #[test]
    fn nonempty_for_three_sites() {
        let sites = vec![
            Point2::new(25.0, 25.0),
            Point2::new(75.0, 25.0),
            Point2::new(50.0, 75.0),
        ];
        let edges = compute_edges(&sites, (0.0, 0.0, 100.0, 100.0));
        assert!(edges.len() >= 3);
    }
}
