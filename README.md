# clusterlab

[![CI](https://github.com/Loghic/clusterlab/actions/workflows/ci.yml/badge.svg)](https://github.com/Loghic/clusterlab/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/Loghic/clusterlab/branch/main/graph/badge.svg)](https://codecov.io/gh/Loghic/clusterlab)
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Interactive k-means clustering visualizer in Rust. Runs natively on
desktop and in the browser via WebAssembly. Animates each iteration so
you can watch centroids move, point assignments flip, and Voronoi cells
reshape — in 2D or 3D.

## Features

- **Live animation** of every k-means iteration with smooth interpolation.
- **2D and 3D modes**, each fully independent (no projection between them).
- **Pan, zoom, and click-to-add** in 2D; orbit camera (rotate + zoom) in 3D.
- **Voronoi visualization**: edges in 2D, bisector planes in 3D.
- **Run controls**: single step, auto-run with adjustable speed,
  "run to convergence", and loop mode that re-seeds and repeats forever.
- **Built-in datasets** that work in both 2D and 3D: random Gaussian blobs,
  two-moons (interlocking rings in 3D), and the Fisher iris dataset
  (PCA-projected in 3D).
- **Point editing** in 2D: add, delete, or hand-place centroids via the
  Edit tab in the control panel.
- **Pure-logic core** separated from rendering — easy to test and port.

## Quick start (desktop)

You need a stable Rust toolchain (1.75+ for native; 1.77+ for the wasm
build).

```bash
git clone https://github.com/Loghic/clusterlab.git
cd clusterlab
cargo run --release
```

On Linux you'll need a few system packages first:

```bash
sudo apt-get install libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev pkg-config
```

## Run in the browser (WebAssembly)

```bash
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown

mkdir -p dist
cp target/wasm32-unknown-unknown/release/clusterlab.wasm dist/
cp web/index.html dist/

# The macroquad JS loader must match the miniquad version in Cargo.lock.
# This project pins macroquad 0.4.14 (miniquad 0.4.8), and the bundle on
# `master` currently matches. If a version tag URL gives 404, fall back
# to `master`. Do NOT use the stale not-fl3.github.io hosted copy — that
# produces "Version mismatch: gl.js version is: 2, miniquad crate
# version is: 262144" at load time.
curl -fL -o dist/mq_js_bundle.js \
  https://raw.githubusercontent.com/not-fl3/macroquad/master/js/mq_js_bundle.js

# Any static server will do; for example:
cargo install basic-http-server
basic-http-server dist/
```

Then open <http://localhost:4000>.

## Controls

The control panel sits in the upper-left of the window. It has two tabs
(**Main** and **Edit**) and a collapse button (`[-]` / `[+]`) so you can
shrink it down to watch the animation.

### Main tab

| Section | Control | Purpose |
|---|---|---|
| Run | `k` slider | Number of clusters |
| Run | `Step` | Run one iteration |
| Run | `Auto run` / `Pause` | Run iterations on a timer |
| Run | `Run to conv.` | Fast steps until converged |
| Run | `auto s/step` slider | Time between auto-run steps |
| Run | `Loop ON/OFF` | After convergence, hold then re-seed centroids and repeat |
| Run | `hold s` slider | How long to hold the converged state in loop mode |
| Run | `Reset centroids (random)` | Re-randomize centroids; saves a snapshot |
| Run | `Clear (to initial)` | Restore the last snapshot |
| Data | `Blobs` / `Moons` / `Iris` | Switch dataset (works in 2D and 3D) |
| Data | `Regenerate (same dataset)` | New sample from the same source |
| Data | `Clear all points` | Wipe the canvas |
| View | `Switch to 2D` / `3D` | Toggle mode |
| View | `Show Voronoi` / `Hide` | Toggle cell boundaries (edges in 2D, planes in 3D) |
| View | `Reset view` | Reset the active camera |

### Edit tab (2D only)

Pick what a left-click on the canvas does:

- `Off` — clicks are ignored (default outside the Edit tab)
- `Add point` — adds a point at the cursor
- `Delete nearest` — removes the closest point within ~25 world units
- `Place centroid` — cycles through centroid indices 0..k

Snapshots are taken automatically after each placement so
`Clear (to initial)` works as expected.

### Mouse / scroll

| Action | Mode |
|---|---|
| Middle / right drag | 2D pan |
| Scroll | 2D zoom (cursor-anchored) |
| Right drag | 3D rotate (orbit) |
| Scroll | 3D zoom |
| Left click | 2D edit action (see Edit tab) |

## Development

```bash
# Format
cargo fmt --all

# Check formatting (no changes; same as CI)
cargo fmt --all -- --check

# Lint (CI runs this with -D warnings)
cargo clippy --all-targets --all-features -- -D warnings

# Test (89 tests at last count)
cargo test --all

# Install git pre-commit hooks (one-time)
pip install pre-commit
pre-commit install
```

The pre-commit hooks run `fmt --check`, `clippy -D warnings`, and the
library test suite on every commit. CI runs the same checks plus the
integration tests.

See [`AGENTS.md`](AGENTS.md) for project-specific conventions and
contribution rules (also useful for human contributors).

## Project layout

```
src/
├── lib.rs               re-exports the pure-logic modules
├── main.rs              binary entrypoint + wasm getrandom shim
├── kmeans/              algorithm, distance, centroid, state
├── geometry/            Point2/3, Voronoi (edges + 3D planes), sampling, PCA
├── datasets/            embedded Fisher iris + dataset loaders
├── animation/           lerp, smoothstep, timelines with hold phase
├── viz/                 macroquad rendering, cameras, UI
└── app/                 controller glue
tests/                   integration tests for the library
docs/                    architecture, algorithm, module graph, animation, datasets
web/                     wasm host HTML
```

## Architecture

The codebase is split into a **pure-logic library** (`kmeans`, `geometry`,
`animation`, `datasets`) and a **renderer + glue layer** (`viz`, `app`,
`main.rs`). Only the library is unit-tested; the renderer is exercised
by running the app. The boundary is enforced by making `viz` and `app`
the only modules allowed to import macroquad.

Diagrams in [`docs/`](docs/). See [`docs/architecture.md`](docs/architecture.md)
for the full picture.

## Empty cluster strategy

If a cluster ends up with no assigned points after the assignment step,
its centroid is re-seeded to a randomly chosen existing data point. This
keeps `k` stable and prevents NaN centroids. Details in
[`docs/algorithm.md`](docs/algorithm.md).

## Roadmap

- k-means++ initialization for better starting centroids.
- DBSCAN as a second algorithm in the same visualizer.
- Save / load point sets.
- Smooth point recoloring on reassignment (currently snaps).

## License

MIT. See [`LICENSE`](LICENSE).

Author: Matej Michalek.
