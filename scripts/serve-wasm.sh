#!/usr/bin/env bash
set -euo pipefail
cargo build --release --target wasm32-unknown-unknown
mkdir -p dist
cp target/wasm32-unknown-unknown/release/kmeans-viz.wasm dist/
cp web/index.html dist/
if [ ! -f dist/mq_js_bundle.js ]; then
  curl -fL -o dist/mq_js_bundle.js \
    https://raw.githubusercontent.com/not-fl3/macroquad/master/js/mq_js_bundle.js
fi
basic-http-server dist/
