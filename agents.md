# AGENTS.md

Instructions for AI coding agents working on this repo. Human contributors
should also skim it — most of the conventions are project-specific and
not obvious from the code.

## Build & test

Wrap commands in backticks so they can be copy-pasted directly.

- Native build: `cargo build --release`
- Run locally: `cargo run --release`
- Test (all targets): `cargo test --all`
- Format check: `cargo fmt --all -- --check`
- Lint (CI mode): `cargo clippy --all-targets --all-features -- -D warnings`
- WASM build: `cargo build --release --target wasm32-unknown-unknown`
- WASM dev (build + serve in one go): `./scripts/run-web.sh`

CI runs fmt-check + clippy-deny-warnings + tests. Pre-commit hooks run the
same locally. **Both must pass before any commit.**

System packages required on Linux for the desktop build:
`libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev pkg-config`.

## Hard rules

These are non-negotiable and break the build / loader if violated:

1. **No macroquad imports in pure-logic modules.** Only `viz/` and
   `main.rs` may use `macroquad::*`. All library modules (`kmeans`,
   `geometry`, `animation`, `datasets`, `app`, `world`) must stay pure
   Rust so they compile to wasm and stay testable. The `app::Controller`
   state machine in particular is library-side specifically so it can
   be unit-tested without macroquad — keep it that way. Re-check this
   constraint after moving any code.
2. **Never enable the `getrandom` `js` feature.** It transitively pulls
   `wasm-bindgen`, which is incompatible with macroquad's `mq_js_bundle.js`
   loader (`__wbindgen_placeholder__` errors at runtime). On wasm we use
   `getrandom`'s `custom` feature and register a tiny xorshift shim in
   `src/main.rs` instead.
3. **Never link `mq_js_bundle.js` from `not-fl3.github.io`, or from a
   version tag that doesn't match `macroquad` in `Cargo.lock`.** Both
   produce `Version mismatch: gl.js version is: 2, miniquad crate
   version is: 262144` at load time. This project pins macroquad 0.4.14
   (miniquad 0.4.8), and the bundle on `master` currently matches:
   `https://raw.githubusercontent.com/not-fl3/macroquad/master/js/mq_js_bundle.js`.
   If a `vX.Y.Z` tag URL gives 404 (some tags don't include `js/`), use
   `master`. If you bump macroquad and the bundle on `master` now
   tracks a newer miniquad than your Cargo.lock has, downgrade by
   pinning to the matching version tag. Update the URL in
   `web/index.html` (comment), `README.md`, `docs/wasm.md`, and
   `.github/workflows/wasm.yml` in the same commit.
4. **Don't delete `.cargo/config.toml`.** It contains
   `link-arg=--allow-undefined` for the wasm target, required since
   Rust 1.96 removed that flag from defaults. Without it, the wasm
   linker fails with "undefined symbol: glGenVertexArrays" and ~20
   similar errors. See `docs/wasm.md` for details.
5. **Every pure-logic function gets a test.** Anything in `kmeans/`,
   `geometry/`, `animation/`, or `datasets/` should have at least one
   unit test. Match-arm completeness and trivial getters are exempt.
6. **Never use `unwrap()` in non-test code** unless the invariant is
   documented inline (one-line comment justifying it). `expect()` with a
   reason string is acceptable. Use `?` or `match` everywhere else.

## Conventions

### Naming
- Tests: snake_case verbs describing the property tested
  (e.g. `centroid_of_symmetric_cloud_is_origin`). Avoid `test_X`.
- 2D/3D dual functions: same name with `_2d` / `_3d` suffix
  (e.g. `step_2d`, `step_3d`). Don't generic-ify them — the explicit
  duplication is intentional and keeps types readable.

### Floats
- Use `approx::assert_relative_eq!` for f32 equality in tests. Never
  `==` on floats.
- f32 throughout (not f64). The only place we cast to f64 is the
  voronoice API boundary because it requires it.

### Modules
- `pub use` re-exports go in each module's `mod.rs`, not in `lib.rs`.
- `lib.rs` only re-exports the four top-level modules (`animation`,
  `datasets`, `geometry`, `kmeans`). Adding more is fine, but keep the
  binary-only stuff (`viz`, `app`) out of `lib.rs`.

### UI
- The control panel API is data-in / data-out: `draw_panel` takes a
  `PanelInputs` and returns a `UiState`. Don't pass `&mut` into UI
  helpers; let `main.rs` apply the deltas. This keeps the UI layer
  free of controller mutations.
- macroquad's `widgets::Window::new(...).ui(&mut root_ui(), |ui| { ... })`
  needs `&mut root_ui()`, **not** `&mut *root_ui()`. Clippy will catch
  the deref but be aware that older docs sometimes show the dereffed
  form.

### Animation
- `Timeline` has two phases: `0..duration` is the lerp, `duration..duration+hold`
  is a static hold at `to`. `is_done()` only returns true after the hold.
  This is how Loop mode's hold-seconds slider works.
- `start_step()` clears `auto_run` on convergence only when `looping` is
  off. Don't break this — it's the fix for both the "Pause stuck after
  convergence" bug and the "Loop ON only runs one iteration" bug.

## Empty cluster strategy

If a cluster has zero assigned points after the assignment step, the
centroid is re-seeded to a randomly chosen existing data point. Pick
this strategy when extending the algorithm (e.g. k-means++). Do **not**
drop the cluster or set the centroid to the origin — both have been
tried and break invariants the UI relies on (cluster count stays at k,
no NaN centroids).

## Gotchas

- **Loop mode requires auto-run.** `set_looping(true)` automatically
  flips `auto_run = true`. Don't gate "auto run" on `!converged` in
  `main.rs` — let the controller manage that flag.
- **Disjoint field borrows work in `assign_2d` / `assign_3d`.** The
  loop reads `state.points` and writes `state.assignments[i]` — these
  are separate fields, so the borrow checker is fine with it. Don't
  refactor to introduce a local for `state.centroids` unless you have
  a reason.
- **`Color::new` is `const fn`** in macroquad. Either struct-literal
  syntax (`Color { r, g, b, a }`) or `Color::new(...)` works in
  `const` slices. Earlier comments in the code claimed otherwise; that
  was wrong.
- **`draw_sphere_wires` does not exist** in macroquad. For a wireframe
  look on a sphere, stack two `draw_sphere` calls with different
  radii and colors.
- **`mouse_delta_position` is unreliable on wasm.** The cameras track
  `mouse_position` themselves and compute deltas across frames.
- **2D Voronoi for `k=2`** is special-cased in `geometry/voronoi.rs`
  because voronoice / delaunator cannot triangulate two collinear
  sites. The custom path computes the perpendicular bisector and
  clips it with Liang-Barsky.
- **3D Voronoi planes are approximate.** We draw the bisector plane
  between every pair of centroids, clipped to the world cube. We
  don't trim planes against each other (that's a Sutherland-Hodgman
  problem). This is intentional; the result is "Voronoi-ish" and
  gets visually busy fast for `k >= 5`.
- **Iris in 3D uses PCA**, not a feature slice. The PCA code is in
  `datasets/mod.rs` and uses power iteration with deflation on a
  4×4 covariance matrix. No external linear-algebra dependency.

## PR / commit checklist

Before committing:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test --all` — every test passes
4. Updated tests for any pure-logic change
5. Updated `README.md` if user-facing controls changed
6. Updated this file if conventions changed

Commit messages: short imperative subject ("Fix Loop ON early stop"),
no "feat:" / "fix:" prefixes (we're not running conventional-commits
automation).

## Where to put new code

- New algorithm (e.g. k-means++): `src/kmeans/` next to `algorithm.rs`,
  re-exported from `kmeans/mod.rs`.
- New dataset: `src/datasets/` — embed the data as a `&str` constant
  in a submodule (mirror the iris setup) so wasm works without file IO.
- New rendering primitive: `src/viz/`, with `pub fn draw_*` functions
  taking world-space inputs.
- New 2D / 3D math helper: `src/geometry/` — must stay
  macroquad-free.
- New UI control: `src/viz/ui.rs`, threading the value through
  `PanelInputs` → `UiState` → `main.rs` → controller setter.

## README badges

`README.md` and `Cargo.toml` reference `https://github.com/Loghic/clusterlab`.
Four badges live at the top of README.md:

1. **CI**: `.github/workflows/ci.yml` runs fmt + clippy + tests on every
   push/PR to `main`.
2. **codecov**: `.github/workflows/coverage.yml` runs `cargo-llvm-cov`
   and uploads to Codecov. Public repos need no token; private repos
   need a `CODECOV_TOKEN` secret (already wired up in the workflow,
   just uncomment the token line).
3. **Rust version**: static shields.io badge pinned to `1.75+` (matches
   `rust-version` in Cargo.toml). Bump both if the MSRV moves.
4. **License**: static shields.io badge linking to `LICENSE`. We're MIT.

The CI badge auto-updates from GitHub Actions API. The codecov badge
shows "no data" until the first coverage upload completes — which
won't happen until you push the workflow to the GitHub default branch.
Don't panic if it looks broken on first push; it usually takes 1–2
minutes after CI completes.
