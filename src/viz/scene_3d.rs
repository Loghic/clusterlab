//! 3D scene rendering: spheres for points/centroids and optional
//! bisector planes for a 3D Voronoi-like partition.

use crate::viz::camera::OrbitCamera;
use crate::viz::scene_2d::cluster_color;
use kmeans_viz::geometry::{bisector_planes, Point3};
use macroquad::prelude::*;

/// World bounds for 3D (centered cube around the origin).
pub const HALF_EXTENT: f32 = 5.0;

/// Draws ground grid, planes (if enabled), points, and centroids.
pub fn draw_scene(
    camera: &OrbitCamera,
    points: &[Point3],
    assignments: &[usize],
    centroids: &[Point3],
    show_planes: bool,
) {
    set_camera(&camera.camera3d());

    draw_grid(20, 0.5, Color::new(0.4, 0.4, 0.45, 1.0), Color::new(0.2, 0.2, 0.25, 1.0));

    if show_planes && centroids.len() >= 2 {
        draw_bisector_planes(centroids);
    }

    for (i, p) in points.iter().enumerate() {
        let color = cluster_color(*assignments.get(i).unwrap_or(&0));
        draw_sphere(vec3(p.x, p.y, p.z), 0.08, None, color);
    }

    for (i, c) in centroids.iter().enumerate() {
        let color = cluster_color(i);
        // Single solid colored sphere. The previous version drew a larger
        // black sphere then a smaller colored one inside — the inner
        // sphere was fully occluded so centroids appeared black.
        // Centroids stand out from data points by being ~3× their radius
        // (0.26 vs 0.08).
        draw_sphere(vec3(c.x, c.y, c.z), 0.26, None, color);
    }

    set_default_camera();
}

fn draw_bisector_planes(centroids: &[Point3]) {
    let planes = bisector_planes(centroids, HALF_EXTENT);
    for quad in &planes {
        // Tint each plane with a midpoint of its two cluster colors so it's
        // visually associated with the partition it represents.
        let ca = cluster_color(quad.pair.0);
        let cb = cluster_color(quad.pair.1);
        let fill = Color::new(
            (ca.r + cb.r) * 0.5,
            (ca.g + cb.g) * 0.5,
            (ca.b + cb.b) * 0.5,
            0.18,
        );
        let edge = Color::new(
            (ca.r + cb.r) * 0.5,
            (ca.g + cb.g) * 0.5,
            (ca.b + cb.b) * 0.5,
            0.7,
        );

        let c = quad.corners;
        // Two triangles for the fill.
        draw_triangle_filled_3d(c[0], c[1], c[2], fill);
        draw_triangle_filled_3d(c[0], c[2], c[3], fill);
        // Edges to make the plane outline visible.
        let p0 = vec3(c[0].x, c[0].y, c[0].z);
        let p1 = vec3(c[1].x, c[1].y, c[1].z);
        let p2 = vec3(c[2].x, c[2].y, c[2].z);
        let p3 = vec3(c[3].x, c[3].y, c[3].z);
        draw_line_3d(p0, p1, edge);
        draw_line_3d(p1, p2, edge);
        draw_line_3d(p2, p3, edge);
        draw_line_3d(p3, p0, edge);
    }
}

/// Draw a single triangle in 3D using `draw_line_3d` + a filled approximation.
/// macroquad has no `draw_triangle_3d`, so we fake a fill by drawing many
/// parallel lines between two edges.
fn draw_triangle_filled_3d(a: Point3, b: Point3, c: Point3, color: Color) {
    // Rasterize the triangle with N parallel line strips from edge a-b to a-c.
    const STRIPS: usize = 24;
    for i in 0..STRIPS {
        let t0 = i as f32 / STRIPS as f32;
        let t1 = (i + 1) as f32 / STRIPS as f32;
        let p0 = lerp3(a, b, t0);
        let p1 = lerp3(a, c, t0);
        let p2 = lerp3(a, b, t1);
        let p3 = lerp3(a, c, t1);
        // Two lines per strip form a thin quad fill.
        draw_line_3d(to_v(p0), to_v(p1), color);
        draw_line_3d(to_v(p0), to_v(p2), color);
        draw_line_3d(to_v(p1), to_v(p3), color);
    }
}

fn lerp3(a: Point3, b: Point3, t: f32) -> Point3 {
    Point3::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t, a.z + (b.z - a.z) * t)
}

fn to_v(p: Point3) -> Vec3 {
    vec3(p.x, p.y, p.z)
}
