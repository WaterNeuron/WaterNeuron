#!/usr/bin/env bash

# build
bazel build //water_neuron:waterneuron
RUSTFLAGS="-C embed-bitcode=yes -C opt-level=3 -C debug-assertions=no -C debuginfo=0 -C lto -C link-args=-z,stack-size=3145728 -C linker-plugin-lto" cargo build --target wasm32-unknown-unknown --release

# compute hashes
sha256sum bazel-bin/water_neuron/waterneuron.wasm
sha256sum target/wasm32-unknown-unknown/release/water-neuron.wasm
