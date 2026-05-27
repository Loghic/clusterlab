//! Binary entrypoint. Wires the controller to the macroquad event loop.

mod viz;

// Wasm `getrandom` shim — see Cargo.toml for context.
#[cfg(target_arch = "wasm32")]
mod wasm_rand_shim {
    use std::cell::Cell;
    thread_local! {
        static STATE: Cell<u64> = const { Cell::new(0x9E37_79B9_7F4A_7C15) };
    }

    fn next_u64() -> u64 {
        STATE.with(|s| {
            let mut x = s.get();
            if x == 0 {
                x = 0x9E37_79B9_7F4A_7C15;
            }
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            s.set(x);
            x
        })
    }

    pub fn fill_bytes(buf: &mut [u8]) -> Result<(), getrandom::Error> {
        let seed = (macroquad::miniquad::date::now() * 1e6) as u64;
        STATE.with(|s| s.set(s.get() ^ seed));

        let mut i = 0;
        while i < buf.len() {
            let chunk = next_u64().to_le_bytes();
            let n = (buf.len() - i).min(8);
            buf[i..i + n].copy_from_slice(&chunk[..n]);
            i += n;
        }
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
getrandom::register_custom_getrandom!(wasm_rand_shim::fill_bytes);

use clusterlab::app::{Controller, MAX_N_POINTS, MIN_N_POINTS};
use macroquad::prelude::*;
use viz::{
    camera::OrbitCamera,
    camera2d::Camera2D,
    scene_2d::draw_scene as draw_2d,
    scene_3d::draw_scene as draw_3d,
    ui::{
        draw_panel, EditMode, PanelInputs, PanelSettings, PANEL_HEIGHT_COLLAPSED,
        PANEL_HEIGHT_FULL, PANEL_WIDTH,
    },
};

/// World-space radius for "delete nearest point" hit detection.
const DELETE_RADIUS_WORLD: f32 = 25.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "k-means visualizer".to_owned(),
        window_width: 1100,
        window_height: 760,
        high_dpi: true,
        ..Default::default()
    }
}

fn seed_from_time() -> u64 {
    let t = miniquad::date::now();
    (t * 1_000_000.0) as u64
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut controller = Controller::new(seed_from_time(), 4);
    let mut camera_3d = OrbitCamera::default();
    let mut camera_2d = Camera2D::default();
    let mut panel_settings = PanelSettings::default();
    let mut auto_run_timer = 0.0f32;

    loop {
        let dt = get_frame_time();
        let just_finished = controller.tick(dt);
        if just_finished {
            auto_run_timer = 0.0;
        }

        clear_background(Color::new(0.10, 0.11, 0.14, 1.0));

        let (mx, my) = mouse_position();
        let panel_h = if panel_settings.collapsed {
            PANEL_HEIGHT_COLLAPSED
        } else {
            PANEL_HEIGHT_FULL
        };
        let pointer_on_panel = mx < PANEL_WIDTH + 10.0 && my < panel_h + 20.0;

        if controller.mode_3d {
            if !pointer_on_panel {
                camera_3d.handle_input();
            }
            let centroids = controller.display_centroids_3d();
            draw_3d(
                &camera_3d,
                &controller.state_3d.points,
                &controller.state_3d.assignments,
                &centroids,
                controller.show_voronoi,
            );
        } else {
            let camera_moved = camera_2d.handle_input(pointer_on_panel);

            let centroids = controller.display_centroids_2d();
            draw_2d(
                &camera_2d,
                &controller.state_2d.points,
                &controller.state_2d.assignments,
                &centroids,
                controller.show_voronoi,
            );

            // Left-click on the 2D canvas does whatever edit_mode says.
            // Off (default outside the Edit tab) means clicks are ignored,
            // so the Main tab stays free of editing side-effects.
            if !pointer_on_panel && !camera_moved && is_mouse_button_pressed(MouseButton::Left) {
                let p = camera_2d.screen_to_world(mx, my, screen_width(), screen_height());
                match panel_settings.edit_mode {
                    EditMode::Off => {}
                    EditMode::Add => controller.add_point_2d(p),
                    EditMode::Delete => {
                        controller.delete_nearest_point_2d(p, DELETE_RADIUS_WORLD);
                    }
                    EditMode::PlaceCentroid => controller.place_centroid_2d(p),
                }
            }
        }

        let inputs = PanelInputs {
            settings: panel_settings,
            k: controller.k,
            auto_run: controller.auto_run,
            auto_run_interval: controller.auto_run_interval,
            mode_3d: controller.mode_3d,
            show_voronoi: controller.show_voronoi,
            iteration: if controller.mode_3d {
                controller.state_3d.iteration
            } else {
                controller.state_2d.iteration
            },
            converged: controller.converged,
            looping: controller.looping,
            hold_seconds: controller.hold_seconds,
            point_count: controller.point_count(),
            n_points: controller.n_points,
            n_points_min: MIN_N_POINTS,
            n_points_max: MAX_N_POINTS,
        };
        let ui = draw_panel(inputs);
        panel_settings = ui.settings;

        if ui.k != controller.k {
            controller.set_k(ui.k);
        }
        if ui.step_clicked && !controller.is_animating() {
            controller.run_one_step();
        }
        if ui.run_to_convergence_clicked {
            controller.run_to_convergence();
        }
        if ui.auto_run != controller.auto_run {
            controller.toggle_auto_run(ui.auto_run);
            auto_run_timer = 0.0;
        }
        if (ui.auto_run_interval - controller.auto_run_interval).abs() > 1e-3 {
            controller.set_auto_run_interval(ui.auto_run_interval);
        }
        if ui.looping != controller.looping {
            controller.set_looping(ui.looping);
        }
        if (ui.hold_seconds - controller.hold_seconds).abs() > 1e-3 {
            controller.set_hold_seconds(ui.hold_seconds);
        }
        if ui.n_points != controller.n_points {
            controller.set_n_points(ui.n_points);
        }
        if ui.reset_clicked {
            controller.reset_centroids_random();
        }
        if ui.clear_to_initial_clicked {
            controller.clear_to_initial();
        }
        if ui.regen_points_clicked {
            controller.regenerate_points();
        }
        if ui.clear_points_clicked {
            controller.clear_points();
        }
        if ui.toggle_mode_clicked {
            controller.toggle_mode();
        }
        if ui.toggle_voronoi {
            controller.toggle_voronoi();
        }
        if ui.reset_camera_clicked {
            if controller.mode_3d {
                camera_3d = OrbitCamera::default();
            } else {
                camera_2d.reset();
            }
        }
        if let Some(choice) = ui.dataset_clicked {
            controller.load_dataset(choice);
            // Only reset the 2D camera when in 2D — otherwise we'd snap
            // it back even though the user is looking at the 3D scene.
            if !controller.mode_3d {
                camera_2d.reset();
            }
        }

        // Auto-run: fire a step periodically when not animating.
        // The controller turns auto_run off automatically on convergence,
        // so the "Pause" button label no longer gets stuck.
        if controller.auto_run && !controller.is_animating() && !controller.converged {
            auto_run_timer += dt;
            if auto_run_timer >= controller.auto_run_interval {
                controller.run_one_step();
                auto_run_timer = 0.0;
            }
        }

        next_frame().await;
    }
}
