//! Pure geometric primitives. No rendering deps.

pub mod planes;
pub mod point;
pub mod sampling;
pub mod voronoi;

pub use planes::{bisector_planes, PlaneQuad};
pub use point::{Point2, Point3};
pub use sampling::{
    generate_blobs_2d, generate_blobs_3d, generate_moons_2d, generate_moons_3d, sample_normal,
    sample_normal_2d, sample_normal_3d,
};
pub use voronoi::{compute_edges, VoronoiEdge};
