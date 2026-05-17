//! World-space constants and dataset selection.
//!
//! These items used to live in the rendering layer but are needed by the
//! controller too. Hoisting them to the library lets the controller be
//! pure-logic and unit-testable.

/// World-space bounds for 2D mode: (min_x, min_y, max_x, max_y).
/// The camera fits this rectangle to the screen at zoom = 1.0.
pub const WORLD_BOUNDS: (f32, f32, f32, f32) = (0.0, 0.0, 1000.0, 700.0);

/// Half-side of the centered cube used for 3D mode. The 3D scene spans
/// `(-HALF_EXTENT..=HALF_EXTENT)` on each axis.
pub const HALF_EXTENT: f32 = 5.0;

/// Convenience for the 3D bounds expected by the geometry generators.
/// Returns `(min_x, min_y, min_z, max_x, max_y, max_z)`.
pub fn bounds_3d() -> (f32, f32, f32, f32, f32, f32) {
    (
        -HALF_EXTENT,
        -HALF_EXTENT,
        -HALF_EXTENT,
        HALF_EXTENT,
        HALF_EXTENT,
        HALF_EXTENT,
    )
}

/// Dataset choices the user can pick from the menu. Works in 2D and 3D.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatasetChoice {
    BlobsRandom,
    Moons,
    Iris,
}

impl DatasetChoice {
    pub fn label(self) -> &'static str {
        match self {
            DatasetChoice::BlobsRandom => "Random blobs",
            DatasetChoice::Moons => "Two moons",
            DatasetChoice::Iris => "Iris",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds_3d_is_symmetric() {
        let (a, b, c, d, e, f) = bounds_3d();
        assert_eq!(a, -d);
        assert_eq!(b, -e);
        assert_eq!(c, -f);
    }

    #[test]
    fn world_bounds_2d_is_positive_quadrant() {
        let (x0, y0, x1, y1) = WORLD_BOUNDS;
        assert!(x0 < x1);
        assert!(y0 < y1);
    }

    #[test]
    fn dataset_choice_labels_are_distinct() {
        let labels = [
            DatasetChoice::BlobsRandom.label(),
            DatasetChoice::Moons.label(),
            DatasetChoice::Iris.label(),
        ];
        for i in 0..labels.len() {
            for j in (i + 1)..labels.len() {
                assert_ne!(labels[i], labels[j], "duplicate label");
            }
        }
    }
}
