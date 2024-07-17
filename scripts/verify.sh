#!/usr/bin/env bash

# bazel
bazel build //water_neuron:waterneuron
sha256sum bazel-bin/water_neuron/waterneuron.wasm

# cargo
RUSTFLAGS="-C opt-level=z -C debuginfo=0" CARGO_PROFILE_RELEASE_LTO=true cargo build --target wasm32-unknown-unknown --release
sha256sum target/wasm32-unknown-unknown/release/water-neuron.wasm
