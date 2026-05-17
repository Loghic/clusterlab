//! 2D scene rendering: points, centroids, Voronoi edges, via Camera2D.

use crate::viz::camera2d::{Camera2D, WORLD_BOUNDS};
use kmeans_viz::geometry::{compute_edges, Point2};
use macroquad::prelude::*;

/// Palette for cluster colors. 12 entries — matches the maximum `k`
/// the UI slider allows, so colors never repeat in a normal run.
///
/// Hand-tuned for good contrast on the dark background and from each
/// other. If you add a 13th, double-check pairs of similar hues don't
/// land on adjacent cluster indices in typical k-means runs.
pub const PALETTE: &[Color] = &[
    Color { r: 0.95, g: 0.30, b: 0.30, a: 1.0 }, // red
    Color { r: 0.30, g: 0.65, b: 0.95, a: 1.0 }, // sky blue
    Color { r: 0.40, g: 0.85, b: 0.40, a: 1.0 }, // green
    Color { r: 0.95, g: 0.75, b: 0.20, a: 1.0 }, // yellow
    Color { r: 0.75, g: 0.40, b: 0.90, a: 1.0 }, // purple
    Color { r: 0.95, g: 0.55, b: 0.25, a: 1.0 }, // orange
    Color { r: 0.30, g: 0.85, b: 0.85, a: 1.0 }, // cyan
    Color { r: 0.95, g: 0.45, b: 0.65, a: 1.0 }, // pink
    Color { r: 0.60, g: 0.90, b: 0.20, a: 1.0 }, // lime
    Color { r: 0.95, g: 0.25, b: 0.95, a: 1.0 }, // magenta
    Color { r: 0.20, g: 0.55, b: 0.55, a: 1.0 }, // teal
    Color { r: 0.85, g: 0.70, b: 0.10, a: 1.0 }, // gold
];

pub fn cluster_color(idx: usize) -> Color {
    PALETTE[idx % PALETTE.len()]
}

/// Render Voronoi edges (if requested), the world frame, points, and centroids.
pub fn draw_scene(
    cam: &Camera2D,
    points: &[Point2],
    assignments: &[usize],
    centroids: &[Point2],
    show_voronoi: bool,
) {
    let sw = screen_width();
    let sh = screen_height();

    // World frame: draws the canonical 1000x700 rectangle so the user has
    // a reference of where data lives, even after panning/zooming.
    draw_world_frame(cam, sw, sh);

    if show_voronoi && centroids.len() >= 2 {
        let edges = compute_edges(centroids, WORLD_BOUNDS);
        for e in &edges {
            let (x1, y1) = cam.world_to_screen(e.from, sw, sh);
            let (x2, y2) = cam.world_to_screen(e.to, sw, sh);
            draw_line(x1, y1, x2, y2, 1.5, Color::new(0.55, 0.55, 0.6, 0.7));
        }
    }

    let scale = cam.scale(sw, sh);
    let point_r = (4.0 * scale.sqrt()).clamp(2.0, 12.0);
    let centroid_r = (11.0 * scale.sqrt()).clamp(6.0, 28.0);

    for (i, p) in points.iter().enumerate() {
        let (sx, sy) = cam.world_to_screen(*p, sw, sh);
        if sx < -10.0 || sx > sw + 10.0 || sy < -10.0 || sy > sh + 10.0 {
            continue;
        }
        let color = cluster_color(*assignments.get(i).unwrap_or(&0));
        draw_circle(sx, sy, point_r, color);
    }

    for (i, c) in centroids.iter().enumerate() {
        let (sx, sy) = cam.world_to_screen(*c, sw, sh);
        let color = cluster_color(i);
        draw_circle(sx, sy, centroid_r, BLACK);
        draw_circle(sx, sy, centroid_r * 0.82, color);
        draw_circle_lines(sx, sy, centroid_r, 1.8, WHITE);
    }
}

fn draw_world_frame(cam: &Camera2D, sw: f32, sh: f32) {
    let (min_x, min_y, max_x, max_y) = WORLD_BOUNDS;
    let (x1, y1) = cam.world_to_screen(Point2::new(min_x, min_y), sw, sh);
    let (x2, y2) = cam.world_to_screen(Point2::new(max_x, max_y), sw, sh);
    let col = Color::new(0.3, 0.3, 0.35, 0.6);
    draw_rectangle_lines(x1, y1, x2 - x1, y2 - y1, 1.0, col);
}
