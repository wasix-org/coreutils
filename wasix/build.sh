#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
[[ "$(cargo wasix --version)" == "cargo-wasix 0.1.33"* ]]
export RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals,-wide-arithmetic --remap-path-prefix=$HOME=/home/build --remap-path-prefix=$PWD=/src"
cargo wasix build --release --locked --no-default-features --features feat_wasix --bin coreutils
wasm-tools validate --features=-wide-arithmetic,-legacy-exceptions,-function-references,-gc \
  "${CARGO_TARGET_DIR:-target}/wasm32-wasmer-wasi/release/coreutils.wasm"
python3 wasix/package.py
