//! Pure k-means clustering. No rendering deps.
//!
//! Empty cluster strategy: when a cluster ends up with no assigned points,
//! its centroid is re-seeded to a randomly chosen existing data point. This
//! keeps `k` stable and avoids NaN centroids from division by zero.

pub mod algorithm;
pub mod centroid;
pub mod distance;
pub mod state;

pub use algorithm::{assign_2d, assign_3d, run_to_completion_2d, step_2d, step_3d, StepOutcome};
pub use centroid::{init_random_2d, init_random_3d, mean_2d, mean_3d};
pub use distance::{Distance, Euclidean};
pub use state::{KMeansState2D, KMeansState3D};
