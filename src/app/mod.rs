//! Application glue: ties the pure library to the macroquad UI.

pub mod controller;

pub use controller::{Controller, DEFAULT_N_POINTS, MAX_N_POINTS, MIN_N_POINTS};
