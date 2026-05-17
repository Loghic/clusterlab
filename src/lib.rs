//! kmeans-viz: pure logic crate root.
//!
//! This library contains the testable, render-agnostic parts of the project:
//! - [`geometry`] — 2D/3D points, Voronoi, planes, sampling.
//! - [`kmeans`] — clustering algorithm and state.
//! - [`animation`] — interpolation primitives and timelines.
//! - [`datasets`] — built-in real-world and synthetic datasets.
//!
//! The binary in `src/main.rs` plus the modules under `src/viz/` and
//! `src/app/` consume this library and add the macroquad-based UI on top.

pub mod animation;
pub mod datasets;
pub mod geometry;
pub mod kmeans;
