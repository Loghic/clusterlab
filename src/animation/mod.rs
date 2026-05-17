//! Animation primitives: tweening and timelines for smooth centroid motion.

pub mod timeline;
pub mod tween;

pub use timeline::{Timeline2D, Timeline3D};
pub use tween::{lerp_f32, lerp_point2, lerp_point3, smoothstep};
