//! clusterlab: pure logic crate root.
//!
//! This library contains the testable, render-agnostic parts of the project:
//! - [`geometry`] — 2D/3D points, Voronoi, planes, sampling.
//! - [`kmeans`] — clustering algorithm and state.
//! - [`animation`] — interpolation primitives and timelines.
//! - [`datasets`] — built-in real-world and synthetic datasets.
//! - [`world`] — shared world-space constants and `DatasetChoice` enum.
//! - [`app`] — controller state machine (no rendering).
//!
//! The binary in `src/main.rs` plus the modules under `src/viz/` consume
//! this library and add the macroquad-based UI on top.

pub mod animation;
pub mod app;
pub mod datasets;
pub mod geometry;
pub mod kmeans;
pub mod world;
