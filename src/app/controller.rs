//! Top-level controller: owns state and orchestrates the algorithm,
//! animation timeline, and dataset switching. No rendering — the binary
//! crate's `main.rs` reads controller state each frame and renders it
//! via macroquad.

use crate::animation::{Timeline2D, Timeline3D};
use crate::datasets::{iris_2d, iris_3d};
use crate::geometry::{
    generate_blobs_2d, generate_blobs_3d, generate_moons_2d, generate_moons_3d, Point2, Point3,
};
use crate::kmeans::{
    assign_2d, assign_3d, init_random_2d, init_random_3d, step_2d, step_3d, Euclidean,
    KMeansState2D, KMeansState3D, StepOutcome,
};
use crate::world::{bounds_3d, DatasetChoice, HALF_EXTENT, WORLD_BOUNDS};
use rand::{rngs::StdRng, SeedableRng};

const CONVERGENCE_THRESHOLD: f32 = 0.05;
const ANIMATION_DURATION: f32 = 0.5;
const FAST_ANIMATION_DURATION: f32 = 0.1;
const MAX_ITERS_RUN_TO_CONVERGENCE: usize = 200;
/// Default number of points generated for Blobs/Moons in either mode.
/// User-adjustable at runtime via the UI panel and clamped to this range.
pub const DEFAULT_N_POINTS: usize = 220;
pub const MIN_N_POINTS: usize = 10;
pub const MAX_N_POINTS: usize = 2000;
const NUM_BLOBS: usize = 4;
const BLOB_STD_2D: f32 = 55.0;
const BLOB_STD_3D: f32 = 0.55;
const MOONS_NOISE: f32 = 0.10;
const DEFAULT_AUTO_RUN_INTERVAL: f32 = 0.7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunMode {
    Idle,
    Single,
    Auto,
    Fast,
}

pub struct Controller {
    pub state_2d: KMeansState2D,
    pub state_3d: KMeansState3D,
    pub k: usize,
    pub mode_3d: bool,
    pub show_voronoi: bool,
    pub auto_run: bool,
    pub auto_run_interval: f32,
    pub converged: bool,
    pub looping: bool,
    pub hold_seconds: f32,
    pub current_dataset: DatasetChoice,
    /// Number of points to generate next time Blobs/Moons is (re)generated.
    /// The Iris dataset ignores this (its size is fixed by the data file).
    /// Changes here do NOT touch the currently displayed points; they take
    /// effect on the next "Regenerate" / dataset switch.
    pub n_points: usize,
    pub timeline_2d: Option<Timeline2D>,
    pub timeline_3d: Option<Timeline3D>,

    /// Snapshot of the centroids at the last "fresh" moment (program start,
    /// reset, dataset change, or k change). "Clear to initial" restores
    /// these.
    initial_centroids_2d: Vec<Point2>,
    initial_centroids_3d: Vec<Point3>,

    /// Index of the next centroid to place when in PlaceCentroid edit mode.
    /// Cycles through 0..k.
    pub next_centroid_to_place: usize,

    run_mode: RunMode,
    rng: StdRng,
}

impl Controller {
    pub fn new(seed: u64, k: usize) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let n_points = DEFAULT_N_POINTS;
        let pts_2d = generate_blobs_2d(&mut rng, n_points, NUM_BLOBS, BLOB_STD_2D, WORLD_BOUNDS);
        let pts_3d = generate_blobs_3d(&mut rng, n_points, NUM_BLOBS, BLOB_STD_3D, bounds_3d());
        let cs_2d = init_random_2d(&mut rng, k, WORLD_BOUNDS);
        let cs_3d = init_random_3d(&mut rng, k, bounds_3d());

        let mut state_2d = KMeansState2D::new(pts_2d, cs_2d.clone());
        let mut state_3d = KMeansState3D::new(pts_3d, cs_3d.clone());
        // Compute initial assignments so colours are populated from frame 0.
        if !state_2d.points.is_empty() {
            assign_2d(&mut state_2d, &Euclidean);
        }
        if !state_3d.points.is_empty() {
            assign_3d(&mut state_3d, &Euclidean);
        }

        Self {
            state_2d,
            state_3d,
            k,
            mode_3d: false,
            show_voronoi: true,
            auto_run: false,
            auto_run_interval: DEFAULT_AUTO_RUN_INTERVAL,
            converged: false,
            looping: false,
            hold_seconds: 1.5,
            current_dataset: DatasetChoice::BlobsRandom,
            n_points,
            timeline_2d: None,
            timeline_3d: None,
            initial_centroids_2d: cs_2d,
            initial_centroids_3d: cs_3d,
            next_centroid_to_place: 0,
            run_mode: RunMode::Idle,
            rng,
        }
    }

    pub fn set_k(&mut self, new_k: usize) {
        if new_k == self.k {
            return;
        }
        self.k = new_k.max(1);
        self.reset_centroids_random();
    }

    /// Random new centroids; sets these as the new "initial" snapshot.
    pub fn reset_centroids_random(&mut self) {
        self.state_2d.centroids = init_random_2d(&mut self.rng, self.k, WORLD_BOUNDS);
        self.state_3d.centroids = init_random_3d(&mut self.rng, self.k, bounds_3d());
        self.initial_centroids_2d = self.state_2d.centroids.clone();
        self.initial_centroids_3d = self.state_3d.centroids.clone();
        self.after_centroids_changed();
    }

    /// Restore centroids to whatever the last snapshot was (initial run or
    /// last reset). Iteration counters and converged flag are cleared.
    pub fn clear_to_initial(&mut self) {
        // Ensure snapshots are still consistent with the current k. If they
        // got out of sync (e.g. user changed k after a manual place), fall
        // back to a fresh random reset.
        if self.initial_centroids_2d.len() != self.k {
            self.initial_centroids_2d = init_random_2d(&mut self.rng, self.k, WORLD_BOUNDS);
        }
        if self.initial_centroids_3d.len() != self.k {
            self.initial_centroids_3d = init_random_3d(&mut self.rng, self.k, bounds_3d());
        }
        self.state_2d.centroids = self.initial_centroids_2d.clone();
        self.state_3d.centroids = self.initial_centroids_3d.clone();
        self.after_centroids_changed();
    }

    /// Common bookkeeping after any centroid change.
    fn after_centroids_changed(&mut self) {
        self.state_2d.iteration = 0;
        self.state_3d.iteration = 0;
        self.converged = false;
        self.auto_run = false; // stop auto-run so we don't fight the reset
        self.timeline_2d = None;
        self.timeline_3d = None;
        self.run_mode = RunMode::Idle;
        self.next_centroid_to_place = 0;
        // Refresh assignments so point colours match the new centroids.
        if !self.state_2d.points.is_empty() && !self.state_2d.centroids.is_empty() {
            assign_2d(&mut self.state_2d, &Euclidean);
        } else {
            self.state_2d.assignments = vec![0; self.state_2d.points.len()];
        }
        if !self.state_3d.points.is_empty() && !self.state_3d.centroids.is_empty() {
            assign_3d(&mut self.state_3d, &Euclidean);
        } else {
            self.state_3d.assignments = vec![0; self.state_3d.points.len()];
        }
    }

    /// Regenerate the active mode's points using the current dataset.
    pub fn regenerate_points(&mut self) {
        if self.mode_3d {
            self.regenerate_3d_for_current_dataset();
        } else {
            self.regenerate_2d_for_current_dataset();
        }
        // Snapshot centroids again so "clear to initial" gives a sensible
        // starting point for this fresh dataset.
        self.reset_centroids_random();
    }

    /// Load a dataset. Applies to the currently active mode (2D or 3D).
    pub fn load_dataset(&mut self, choice: DatasetChoice) {
        self.current_dataset = choice;
        if self.mode_3d {
            self.regenerate_3d_for_current_dataset();
        } else {
            self.regenerate_2d_for_current_dataset();
        }
        self.reset_centroids_random();
    }

    fn regenerate_2d_for_current_dataset(&mut self) {
        let pts = match self.current_dataset {
            DatasetChoice::BlobsRandom => generate_blobs_2d(
                &mut self.rng,
                self.n_points,
                NUM_BLOBS,
                BLOB_STD_2D,
                WORLD_BOUNDS,
            ),
            DatasetChoice::Moons => {
                generate_moons_2d(&mut self.rng, self.n_points, MOONS_NOISE, WORLD_BOUNDS)
            }
            // Iris is a fixed dataset (150 samples); n_points doesn't apply.
            DatasetChoice::Iris => iris_2d().points,
        };
        // Keep the documented `assignments.len() == points.len()` invariant.
        // `assign_2d` indexes `assignments[idx]` for every point, so it must
        // be sized to match BEFORE the next assign call. The actual values
        // will be overwritten by `after_centroids_changed` -> `assign_2d`.
        self.state_2d.assignments = vec![0; pts.len()];
        self.state_2d.points = pts;
        self.state_2d.iteration = 0;
    }

    fn regenerate_3d_for_current_dataset(&mut self) {
        let bounds = bounds_3d();
        let pts = match self.current_dataset {
            DatasetChoice::BlobsRandom => {
                generate_blobs_3d(&mut self.rng, self.n_points, NUM_BLOBS, BLOB_STD_3D, bounds)
            }
            DatasetChoice::Moons => {
                generate_moons_3d(&mut self.rng, self.n_points, MOONS_NOISE, bounds)
            }
            // Iris is a fixed dataset (150 samples); n_points doesn't apply.
            DatasetChoice::Iris => iris_3d(HALF_EXTENT).points,
        };
        self.state_3d.assignments = vec![0; pts.len()];
        self.state_3d.points = pts;
        self.state_3d.iteration = 0;
    }

    /// Set the target point count for future Blobs/Moons generations.
    /// Clamped to `[MIN_N_POINTS, MAX_N_POINTS]`. Does NOT regenerate the
    /// currently displayed points — that happens on the next "Regenerate"
    /// click or dataset switch. The Iris dataset is unaffected.
    pub fn set_n_points(&mut self, value: usize) {
        self.n_points = value.clamp(MIN_N_POINTS, MAX_N_POINTS);
    }

    /// Wipe all points (in the active mode). Useful before entering
    /// hand-placed-points mode.
    pub fn clear_points(&mut self) {
        if self.mode_3d {
            self.state_3d.points.clear();
            self.state_3d.assignments.clear();
            self.state_3d.iteration = 0;
        } else {
            self.state_2d.points.clear();
            self.state_2d.assignments.clear();
            self.state_2d.iteration = 0;
        }
        self.converged = false;
        self.auto_run = false;
        self.timeline_2d = None;
        self.timeline_3d = None;
        self.run_mode = RunMode::Idle;
    }

    pub fn run_one_step(&mut self) {
        self.start_step(ANIMATION_DURATION);
        if self.run_mode == RunMode::Idle {
            self.run_mode = RunMode::Single;
        }
    }

    fn start_step(&mut self, duration: f32) {
        if self.converged {
            return;
        }
        if self.mode_3d {
            if self.state_3d.points.is_empty() {
                return;
            }
            let from = self.state_3d.centroids.clone();
            let outcome = step_3d(&mut self.state_3d, CONVERGENCE_THRESHOLD, &mut self.rng);
            let to = self.state_3d.centroids.clone();
            let hold = if self.looping && outcome == StepOutcome::Converged {
                self.hold_seconds
            } else {
                0.0
            };
            self.timeline_3d = Some(Timeline3D::with_hold(from, to, duration, hold));
            if outcome == StepOutcome::Converged {
                self.converged = true;
                // Only clear auto_run when NOT looping. When looping, the
                // loop machinery in tick() handles the restart and we want
                // auto_run to keep firing.
                if !self.looping {
                    self.auto_run = false;
                }
            }
        } else {
            if self.state_2d.points.is_empty() {
                return;
            }
            let from = self.state_2d.centroids.clone();
            let outcome = step_2d(&mut self.state_2d, CONVERGENCE_THRESHOLD, &mut self.rng);
            let to = self.state_2d.centroids.clone();
            let hold = if self.looping && outcome == StepOutcome::Converged {
                self.hold_seconds
            } else {
                0.0
            };
            self.timeline_2d = Some(Timeline2D::with_hold(from, to, duration, hold));
            if outcome == StepOutcome::Converged {
                self.converged = true;
                if !self.looping {
                    self.auto_run = false;
                }
            }
        }
    }

    pub fn run_to_convergence(&mut self) {
        self.run_mode = RunMode::Fast;
        self.start_step(FAST_ANIMATION_DURATION);
    }

    pub fn toggle_auto_run(&mut self, value: bool) {
        self.auto_run = value;
        self.run_mode = if value { RunMode::Auto } else { RunMode::Idle };
    }

    pub fn set_auto_run_interval(&mut self, value: f32) {
        self.auto_run_interval = value.clamp(0.05, 5.0);
    }

    pub fn set_looping(&mut self, value: bool) {
        self.looping = value;
        // Loop is meaningless without something firing steps. When the
        // user turns Loop ON, also enable auto_run so it actually runs;
        // the auto_run interval slider controls how fast it progresses.
        if value {
            self.auto_run = true;
            self.run_mode = RunMode::Auto;
            // If we were already at converged from a previous run, restart
            // from a fresh random state so the loop has something to do.
            if self.converged {
                self.restart_for_loop();
            }
        }
    }

    pub fn set_hold_seconds(&mut self, value: f32) {
        self.hold_seconds = value.max(0.0);
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        let mut finished = false;
        if let Some(tl) = &mut self.timeline_2d {
            tl.advance(dt);
            if tl.is_done() {
                self.timeline_2d = None;
                finished = true;
            }
        }
        if let Some(tl) = &mut self.timeline_3d {
            tl.advance(dt);
            if tl.is_done() {
                self.timeline_3d = None;
                finished = true;
            }
        }

        if finished {
            match self.run_mode {
                RunMode::Fast => {
                    if self.converged {
                        if self.looping {
                            self.restart_for_loop();
                        } else {
                            self.run_mode = RunMode::Idle;
                        }
                    } else if self.state_2d.iteration + self.state_3d.iteration
                        >= MAX_ITERS_RUN_TO_CONVERGENCE
                    {
                        self.run_mode = RunMode::Idle;
                    } else {
                        self.start_step(FAST_ANIMATION_DURATION);
                    }
                }
                RunMode::Single | RunMode::Auto => {
                    if self.converged && self.looping {
                        self.restart_for_loop();
                    }
                }
                RunMode::Idle => {}
            }
        }
        finished
    }

    fn restart_for_loop(&mut self) {
        if self.mode_3d {
            self.state_3d.centroids = init_random_3d(&mut self.rng, self.k, bounds_3d());
            self.state_3d.iteration = 0;
        } else {
            self.state_2d.centroids = init_random_2d(&mut self.rng, self.k, WORLD_BOUNDS);
            self.state_2d.iteration = 0;
        }
        self.converged = false;
        // Refresh assignments so the loop starts with sensible colours.
        if !self.mode_3d && !self.state_2d.points.is_empty() {
            assign_2d(&mut self.state_2d, &Euclidean);
        }
        if self.mode_3d && !self.state_3d.points.is_empty() {
            assign_3d(&mut self.state_3d, &Euclidean);
        }
        // In Fast mode (Run to convergence), chain the next step immediately.
        // In Auto / Single mode let the auto_run timer in main.rs handle it,
        // so the user's "auto s/step" setting controls pacing.
        if self.run_mode == RunMode::Fast {
            self.start_step(FAST_ANIMATION_DURATION);
        }
    }

    pub fn display_centroids_2d(&self) -> Vec<Point2> {
        match &self.timeline_2d {
            Some(tl) => tl.current(),
            None => self.state_2d.centroids.clone(),
        }
    }

    pub fn display_centroids_3d(&self) -> Vec<Point3> {
        match &self.timeline_3d {
            Some(tl) => tl.current(),
            None => self.state_3d.centroids.clone(),
        }
    }

    pub fn is_animating(&self) -> bool {
        self.timeline_2d.is_some() || self.timeline_3d.is_some()
    }

    pub fn add_point_2d(&mut self, p: Point2) {
        let nearest = self.nearest_centroid_2d(p).unwrap_or(0);
        self.state_2d.points.push(p);
        self.state_2d.assignments.push(nearest);
        self.converged = false;
    }

    /// Delete the closest point to `p` if there is one within `radius` units.
    pub fn delete_nearest_point_2d(&mut self, p: Point2, radius: f32) -> bool {
        let mut best: Option<(usize, f32)> = None;
        for (i, q) in self.state_2d.points.iter().enumerate() {
            let d2 = q.distance_squared(&p);
            if best.map_or(true, |(_, b)| d2 < b) {
                best = Some((i, d2));
            }
        }
        if let Some((i, d2)) = best {
            if d2 <= radius * radius {
                self.state_2d.points.remove(i);
                if i < self.state_2d.assignments.len() {
                    self.state_2d.assignments.remove(i);
                }
                self.converged = false;
                return true;
            }
        }
        false
    }

    /// Place a centroid at `p`. Cycles through 0..k on successive calls.
    pub fn place_centroid_2d(&mut self, p: Point2) {
        if self.k == 0 {
            return;
        }
        let idx = self.next_centroid_to_place % self.k;
        if idx >= self.state_2d.centroids.len() {
            self.state_2d.centroids.resize(self.k, p);
        }
        self.state_2d.centroids[idx] = p;
        self.next_centroid_to_place = (idx + 1) % self.k;
        // Refresh assignments and snapshot as new initial.
        if !self.state_2d.points.is_empty() {
            assign_2d(&mut self.state_2d, &Euclidean);
        }
        self.initial_centroids_2d = self.state_2d.centroids.clone();
        self.converged = false;
        self.timeline_2d = None;
    }

    fn nearest_centroid_2d(&self, p: Point2) -> Option<usize> {
        self.state_2d
            .centroids
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                a.distance_squared(&p)
                    .partial_cmp(&b.distance_squared(&p))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)
    }

    pub fn toggle_mode(&mut self) {
        self.mode_3d = !self.mode_3d;
        self.converged = false;
        self.timeline_2d = None;
        self.timeline_3d = None;
        self.run_mode = RunMode::Idle;
        self.auto_run = false;
        // Refresh assignments so colors are right immediately after switch.
        if self.mode_3d {
            if !self.state_3d.points.is_empty() && !self.state_3d.centroids.is_empty() {
                assign_3d(&mut self.state_3d, &Euclidean);
            }
        } else if !self.state_2d.points.is_empty() && !self.state_2d.centroids.is_empty() {
            assign_2d(&mut self.state_2d, &Euclidean);
        }
    }

    pub fn toggle_voronoi(&mut self) {
        self.show_voronoi = !self.show_voronoi;
    }

    pub fn point_count(&self) -> usize {
        if self.mode_3d {
            self.state_3d.points.len()
        } else {
            self.state_2d.points.len()
        }
    }
}
