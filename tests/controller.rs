//! State-machine tests for the `Controller`.
//!
//! These tests don't try to cover every line — they target the
//! interactions that actually broke during development and would
//! re-break if someone refactored carelessly: snapshot semantics
//! around `clear_to_initial`, the `Loop ON` auto-restart behaviour,
//! convergence flipping `auto_run`, and the point-edit operations.

use clusterlab::app::Controller;
use clusterlab::geometry::Point2;
use clusterlab::world::DatasetChoice;

const TEST_SEED: u64 = 42;
const K: usize = 3;

/// Helper: how many tick steps does it take for the current animation
/// to finish? We just pump 1-second ticks until `is_animating` returns
/// false. The default `ANIMATION_DURATION` is 0.5s, so 2 ticks is plenty.
fn drain_animation(c: &mut Controller) {
    for _ in 0..10 {
        if !c.is_animating() {
            break;
        }
        c.tick(1.0);
    }
}

#[test]
fn new_initializes_with_k_centroids_in_both_modes() {
    let c = Controller::new(TEST_SEED, K);
    assert_eq!(c.k, K);
    assert_eq!(c.state_2d.centroids.len(), K);
    assert_eq!(c.state_3d.centroids.len(), K);
    assert!(!c.mode_3d);
    assert!(!c.auto_run);
    assert!(!c.converged);
    assert!(!c.looping);
}

#[test]
fn set_k_resizes_centroids_and_resets_iteration() {
    let mut c = Controller::new(TEST_SEED, K);
    // Pretend we've run a step.
    c.state_2d.iteration = 5;
    c.set_k(5);
    assert_eq!(c.k, 5);
    assert_eq!(c.state_2d.centroids.len(), 5);
    assert_eq!(c.state_3d.centroids.len(), 5);
    assert_eq!(c.state_2d.iteration, 0);
    assert!(!c.converged);
}

#[test]
fn set_k_to_same_value_is_a_noop() {
    let mut c = Controller::new(TEST_SEED, K);
    let before = c.state_2d.centroids.clone();
    c.set_k(K);
    assert_eq!(c.state_2d.centroids, before);
}

#[test]
fn clear_to_initial_restores_snapshot_not_random() {
    let mut c = Controller::new(TEST_SEED, K);
    let snapshot = c.state_2d.centroids.clone();

    // Move centroids away from the snapshot.
    for cn in &mut c.state_2d.centroids {
        cn.x += 100.0;
        cn.y += 100.0;
    }
    assert_ne!(c.state_2d.centroids, snapshot);

    c.clear_to_initial();
    assert_eq!(
        c.state_2d.centroids, snapshot,
        "clear_to_initial must restore the saved snapshot exactly"
    );
}

#[test]
fn clear_to_initial_after_k_change_works_with_stale_snapshot() {
    // This is the bug we hit earlier: if the user changes k after a
    // manual centroid placement, the initial-centroids snapshot has the
    // old length. clear_to_initial must recover, not crash.
    let mut c = Controller::new(TEST_SEED, K);
    // Manually clobber the snapshot to simulate the inconsistent state.
    c.state_2d.centroids.truncate(2); // 2 != k=3
    c.clear_to_initial();
    assert_eq!(c.state_2d.centroids.len(), c.k);
}

#[test]
fn load_dataset_does_not_change_mode() {
    let mut c = Controller::new(TEST_SEED, K);
    assert!(!c.mode_3d);
    c.load_dataset(DatasetChoice::Iris);
    assert!(!c.mode_3d, "load_dataset must not flip 2D/3D mode");
    assert_eq!(c.current_dataset, DatasetChoice::Iris);
}

#[test]
fn clear_points_only_clears_active_mode() {
    let mut c = Controller::new(TEST_SEED, K);
    let n_3d_before = c.state_3d.points.len();
    assert!(!c.mode_3d, "test assumes 2D mode by default");
    c.clear_points();
    assert_eq!(c.state_2d.points.len(), 0);
    assert_eq!(
        c.state_3d.points.len(),
        n_3d_before,
        "clearing 2D must not touch 3D"
    );
}

#[test]
fn set_looping_on_enables_auto_run() {
    let mut c = Controller::new(TEST_SEED, K);
    assert!(!c.auto_run);
    c.set_looping(true);
    assert!(c.looping);
    assert!(
        c.auto_run,
        "turning Loop on should also flip auto_run on (otherwise nothing fires)"
    );
}

#[test]
fn set_looping_off_does_not_change_auto_run() {
    let mut c = Controller::new(TEST_SEED, K);
    c.set_looping(true);
    assert!(c.auto_run);
    c.set_looping(false);
    assert!(!c.looping);
    // Auto-run carries on; user has to pause it themselves.
    assert!(c.auto_run);
}

#[test]
fn add_point_2d_assigns_to_nearest_centroid() {
    let mut c = Controller::new(TEST_SEED, K);
    // Force centroids to known positions so the nearest is unambiguous.
    c.state_2d.centroids = vec![
        Point2::new(0.0, 0.0),
        Point2::new(1000.0, 0.0),
        Point2::new(0.0, 700.0),
    ];

    let before_len = c.state_2d.points.len();
    c.add_point_2d(Point2::new(990.0, 10.0)); // clearly closest to centroid 1
    assert_eq!(c.state_2d.points.len(), before_len + 1);
    assert_eq!(
        c.state_2d.assignments[before_len], 1,
        "new point should be assigned to nearest centroid (index 1)"
    );
}

#[test]
fn delete_nearest_point_within_radius_removes_it() {
    let mut c = Controller::new(TEST_SEED, K);
    // Drop a marker we know we can find.
    c.add_point_2d(Point2::new(500.0, 350.0));
    let n = c.state_2d.points.len();

    let removed = c.delete_nearest_point_2d(Point2::new(501.0, 351.0), 25.0);
    assert!(removed, "should delete a nearby point");
    assert_eq!(c.state_2d.points.len(), n - 1);
}

#[test]
fn delete_nearest_point_outside_radius_returns_false() {
    let mut c = Controller::new(TEST_SEED, K);
    let n = c.state_2d.points.len();
    // Pick a far-away corner where the existing blobs almost certainly don't reach.
    let removed = c.delete_nearest_point_2d(Point2::new(-10000.0, -10000.0), 25.0);
    assert!(!removed, "must not delete anything outside the radius");
    assert_eq!(c.state_2d.points.len(), n, "point count unchanged");
}

#[test]
fn place_centroid_cycles_indices() {
    let mut c = Controller::new(TEST_SEED, K);
    assert_eq!(c.next_centroid_to_place, 0);
    c.place_centroid_2d(Point2::new(10.0, 10.0));
    assert_eq!(c.next_centroid_to_place, 1);
    c.place_centroid_2d(Point2::new(20.0, 20.0));
    assert_eq!(c.next_centroid_to_place, 2);
    c.place_centroid_2d(Point2::new(30.0, 30.0));
    assert_eq!(
        c.next_centroid_to_place, 0,
        "should wrap from k-1 back to 0"
    );
}

#[test]
fn place_centroid_updates_centroid_position() {
    let mut c = Controller::new(TEST_SEED, K);
    let target = Point2::new(123.0, 456.0);
    c.place_centroid_2d(target);
    assert_eq!(c.state_2d.centroids[0], target);
}

#[test]
fn run_to_convergence_eventually_sets_converged() {
    // The real algorithmic property: with non-pathological data, the
    // controller's run-to-convergence path must terminate with
    // `converged = true`. We pump ticks long enough that the Fast
    // animation chain has plenty of opportunities to fire.
    let mut c = Controller::new(TEST_SEED, K);
    c.run_to_convergence();
    // Each animation lasts FAST_ANIMATION_DURATION (0.1s). Pump 500
    // simulated frames at 0.05s each = 25s of game time. That's
    // far more than enough for blobs at k=3 to converge.
    for _ in 0..500 {
        if c.converged {
            break;
        }
        c.tick(0.05);
    }
    assert!(c.converged, "k-means on Gaussian blobs should converge");
}

#[test]
fn convergence_with_loop_on_triggers_restart_and_keeps_auto_run() {
    // After a converged step, if looping is enabled, the next tick
    // should restart with fresh random centroids and keep auto_run on.
    let mut c = Controller::new(TEST_SEED, K);
    c.set_looping(true);
    assert!(c.auto_run);

    // Force convergence by running to it first.
    c.run_to_convergence();
    for _ in 0..500 {
        if c.converged {
            break;
        }
        c.tick(0.05);
    }
    assert!(c.converged);

    let centroids_at_converge = c.state_2d.centroids.clone();

    // One more pump cycle to let the loop machinery fire.
    drain_animation(&mut c);
    for _ in 0..20 {
        c.tick(0.5);
        if !c.converged {
            break;
        }
    }

    assert!(
        c.auto_run,
        "Loop ON should keep auto_run alive after convergence"
    );
    assert!(!c.converged, "Loop should reset converged to false");
    assert_ne!(
        c.state_2d.centroids, centroids_at_converge,
        "Loop should re-seed centroids"
    );
}

#[test]
fn convergence_with_loop_off_clears_auto_run() {
    // Mirror image of the previous test: without Loop, hitting
    // convergence should pause auto_run.
    let mut c = Controller::new(TEST_SEED, K);
    c.toggle_auto_run(true);
    assert!(c.auto_run);
    assert!(!c.looping);

    c.run_to_convergence();
    for _ in 0..500 {
        if c.converged {
            break;
        }
        c.tick(0.05);
    }

    assert!(c.converged);
    assert!(
        !c.auto_run,
        "Without Loop, convergence must stop auto_run so the user isn't stuck"
    );
}
