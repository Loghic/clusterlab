//! Immediate-mode UI controls overlaid on top of the scene.
//!
//! Single panel with two tabs (`Main` and `Edit`) and a collapse button.
//! When collapsed, the panel shrinks to a thin strip with just the
//! iteration counter so you can watch the animation.

use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};

/// Dataset choices the user can pick from the menu. Works in 2D and 3D.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatasetChoice {
    BlobsRandom,
    Moons,
    Iris,
}

impl DatasetChoice {
    #[allow(dead_code)]
    pub fn label(self) -> &'static str {
        match self {
            DatasetChoice::BlobsRandom => "Random blobs",
            DatasetChoice::Moons => "Two moons",
            DatasetChoice::Iris => "Iris",
        }
    }
}

/// Editing sub-mode (lives in the Edit tab). Determines what a left-click does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditMode {
    /// "Inactive" — clicks do nothing. Default when not on the Edit tab.
    Off,
    Add,
    Delete,
    PlaceCentroid,
}

/// Which tab is currently shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Main,
    Edit,
}

/// Persistent settings owned by the host; passed in each frame and updated
/// from the panel state returned.
#[derive(Clone, Copy)]
pub struct PanelSettings {
    pub collapsed: bool,
    pub tab: Tab,
    pub edit_mode: EditMode,
}

impl Default for PanelSettings {
    fn default() -> Self {
        Self {
            collapsed: false,
            tab: Tab::Main,
            edit_mode: EditMode::Off,
        }
    }
}

/// State of UI inputs read each frame.
pub struct UiState {
    pub settings: PanelSettings,

    pub k: usize,
    pub step_clicked: bool,
    pub auto_run: bool,
    pub auto_run_interval: f32,
    pub run_to_convergence_clicked: bool,
    pub reset_clicked: bool,
    pub clear_to_initial_clicked: bool,
    pub regen_points_clicked: bool,
    pub toggle_mode_clicked: bool,
    pub toggle_voronoi: bool,
    pub looping: bool,
    pub hold_seconds: f32,
    pub reset_camera_clicked: bool,
    pub dataset_clicked: Option<DatasetChoice>,
    pub clear_points_clicked: bool,
}

pub const PANEL_WIDTH: f32 = 310.0;
pub const PANEL_HEIGHT_FULL: f32 = 600.0;
pub const PANEL_HEIGHT_COLLAPSED: f32 = 58.0;

pub struct PanelInputs {
    pub settings: PanelSettings,
    pub k: usize,
    pub auto_run: bool,
    pub auto_run_interval: f32,
    pub mode_3d: bool,
    pub show_voronoi: bool,
    pub iteration: usize,
    pub converged: bool,
    pub looping: bool,
    pub hold_seconds: f32,
    pub point_count: usize,
}

/// Draw the control panel and return updated state.
pub fn draw_panel(inputs: PanelInputs) -> UiState {
    let mut settings = inputs.settings;
    let mut k_f32 = inputs.k as f32;
    let mut hold_local = inputs.hold_seconds;
    let mut interval_local = inputs.auto_run_interval;
    let mut auto_run_local = inputs.auto_run;
    let mut looping_local = inputs.looping;

    let mut step_clicked = false;
    let mut run_to_convergence_clicked = false;
    let mut reset_clicked = false;
    let mut clear_to_initial_clicked = false;
    let mut regen_clicked = false;
    let mut toggle_mode = false;
    let mut toggle_voronoi = false;
    let mut reset_camera_clicked = false;
    let mut dataset_clicked: Option<DatasetChoice> = None;
    let mut clear_points_clicked = false;

    let panel_h = if settings.collapsed {
        PANEL_HEIGHT_COLLAPSED
    } else {
        PANEL_HEIGHT_FULL
    };

    widgets::Window::new(hash!(), vec2(10.0, 10.0), vec2(PANEL_WIDTH, panel_h))
        .label("k-means controls")
        .titlebar(true)
        .ui(&mut root_ui(), |ui| {
            // Top row: collapse button + status.
            if ui.button(None, if settings.collapsed { "[+]" } else { "[-]" }) {
                settings.collapsed = !settings.collapsed;
            }
            ui.same_line(50.0);
            ui.label(
                None,
                &format!(
                    "iter {}  ({})",
                    inputs.iteration,
                    if inputs.converged {
                        "converged"
                    } else {
                        "running"
                    }
                ),
            );

            if settings.collapsed {
                return;
            }

            // Tab strip.
            ui.separator();
            for (tab, label) in [(Tab::Main, "Main"), (Tab::Edit, "Edit")] {
                let active = settings.tab == tab;
                let display = if active {
                    format!("[{label}]")
                } else {
                    label.to_string()
                };
                if ui.button(None, display.as_str()) {
                    settings.tab = tab;
                    // Leaving the Edit tab disables edit clicks entirely so
                    // they can't fire from outside the tab.
                    if tab == Tab::Main {
                        settings.edit_mode = EditMode::Off;
                    } else if settings.edit_mode == EditMode::Off {
                        // Default to Add when entering the tab for the first time.
                        settings.edit_mode = EditMode::Add;
                    }
                }
                ui.same_line(0.0);
            }
            ui.label(None, "");
            ui.separator();

            match settings.tab {
                Tab::Main => draw_main_tab(
                    ui,
                    &inputs,
                    &mut k_f32,
                    &mut auto_run_local,
                    &mut interval_local,
                    &mut looping_local,
                    &mut hold_local,
                    &mut step_clicked,
                    &mut run_to_convergence_clicked,
                    &mut reset_clicked,
                    &mut clear_to_initial_clicked,
                    &mut regen_clicked,
                    &mut clear_points_clicked,
                    &mut dataset_clicked,
                    &mut toggle_mode,
                    &mut toggle_voronoi,
                    &mut reset_camera_clicked,
                ),
                Tab::Edit => draw_edit_tab(ui, &inputs, &mut settings),
            }
        });

    UiState {
        settings,
        k: (k_f32.round() as usize).max(1),
        step_clicked,
        auto_run: auto_run_local,
        auto_run_interval: interval_local,
        run_to_convergence_clicked,
        reset_clicked,
        clear_to_initial_clicked,
        regen_points_clicked: regen_clicked,
        toggle_mode_clicked: toggle_mode,
        toggle_voronoi,
        looping: looping_local,
        hold_seconds: hold_local,
        reset_camera_clicked,
        dataset_clicked,
        clear_points_clicked,
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_main_tab(
    ui: &mut macroquad::ui::Ui,
    inputs: &PanelInputs,
    k_f32: &mut f32,
    auto_run_local: &mut bool,
    interval_local: &mut f32,
    looping_local: &mut bool,
    hold_local: &mut f32,
    step_clicked: &mut bool,
    run_to_convergence_clicked: &mut bool,
    reset_clicked: &mut bool,
    clear_to_initial_clicked: &mut bool,
    regen_clicked: &mut bool,
    clear_points_clicked: &mut bool,
    dataset_clicked: &mut Option<DatasetChoice>,
    toggle_mode: &mut bool,
    toggle_voronoi: &mut bool,
    reset_camera_clicked: &mut bool,
) {
    // --- Run ---
    ui.label(None, "Run");
    ui.slider(hash!(), "k", 1.0f32..12.0f32, k_f32);
    ui.label(None, &format!("k = {}", k_f32.round() as usize));

    if ui.button(None, "Step") {
        *step_clicked = true;
    }
    ui.same_line(70.0);
    if ui.button(None, if *auto_run_local { "Pause" } else { "Auto run" }) {
        *auto_run_local = !*auto_run_local;
    }
    ui.same_line(160.0);
    if ui.button(None, "Run to conv.") {
        *run_to_convergence_clicked = true;
    }

    ui.slider(hash!(), "auto s/step", 0.05f32..2.0f32, interval_local);

    if ui.button(
        None,
        if *looping_local {
            "Loop ON"
        } else {
            "Loop OFF"
        },
    ) {
        *looping_local = !*looping_local;
    }
    ui.slider(hash!(), "hold s", 0.0f32..5.0f32, hold_local);

    if ui.button(None, "Reset centroids (random)") {
        *reset_clicked = true;
    }
    if ui.button(None, "Clear (to initial)") {
        *clear_to_initial_clicked = true;
    }

    // --- Data ---
    ui.separator();
    ui.label(None, &format!("Data  (points: {})", inputs.point_count));
    if ui.button(None, "Blobs") {
        *dataset_clicked = Some(DatasetChoice::BlobsRandom);
    }
    ui.same_line(70.0);
    if ui.button(None, "Moons") {
        *dataset_clicked = Some(DatasetChoice::Moons);
    }
    ui.same_line(140.0);
    if ui.button(None, "Iris") {
        *dataset_clicked = Some(DatasetChoice::Iris);
    }
    if ui.button(None, "Regenerate (same dataset)") {
        *regen_clicked = true;
    }
    if ui.button(None, "Clear all points") {
        *clear_points_clicked = true;
    }

    // --- View ---
    ui.separator();
    ui.label(None, "View");
    if ui.button(
        None,
        if inputs.mode_3d {
            "Switch to 2D"
        } else {
            "Switch to 3D"
        },
    ) {
        *toggle_mode = true;
    }
    if ui.button(
        None,
        if inputs.show_voronoi {
            "Hide Voronoi"
        } else {
            "Show Voronoi"
        },
    ) {
        *toggle_voronoi = true;
    }
    if ui.button(None, "Reset view") {
        *reset_camera_clicked = true;
    }
    if inputs.mode_3d {
        ui.label(None, "3D: right drag = rotate, scroll = zoom");
    } else {
        ui.label(None, "2D: middle/right drag = pan");
        ui.label(None, "2D: scroll = zoom");
    }
}

fn draw_edit_tab(ui: &mut macroquad::ui::Ui, inputs: &PanelInputs, settings: &mut PanelSettings) {
    ui.label(None, "Edit clicks on canvas");
    if inputs.mode_3d {
        ui.label(None, "Point editing is 2D-only.");
        ui.label(None, "Switch to 2D from the Main tab.");
        // Keep edit_mode Off in 3D so clicks never fire.
        settings.edit_mode = EditMode::Off;
        return;
    }
    ui.separator();
    for (mode, label) in [
        (EditMode::Off, "Off (no action)"),
        (EditMode::Add, "Add point"),
        (EditMode::Delete, "Delete nearest"),
        (EditMode::PlaceCentroid, "Place centroid"),
    ] {
        let active = settings.edit_mode == mode;
        let display = if active {
            format!("[*] {label}")
        } else {
            format!("[ ] {label}")
        };
        if ui.button(None, display.as_str()) {
            settings.edit_mode = mode;
        }
    }
    ui.separator();
    match settings.edit_mode {
        EditMode::Off => {
            ui.label(None, "Click does nothing right now.");
            ui.label(None, "Pick an action above.");
        }
        EditMode::Add => {
            ui.label(None, "Click on canvas to add a point.");
            ui.label(None, "Joins the nearest centroid's cluster.");
        }
        EditMode::Delete => {
            ui.label(None, "Click near a point to delete it.");
            ui.label(None, "Only deletes within ~25 world units.");
        }
        EditMode::PlaceCentroid => {
            ui.label(None, "Click to place centroid #0;");
            ui.label(None, "next click places #1, etc.");
        }
    }
}
