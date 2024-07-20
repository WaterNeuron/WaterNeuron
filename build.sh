#!/bin/bash

dfx build --network ic water_neuron
gzip ./.dfx/ic/canisters/water_neuron/water_neuron.wasm --force
shasum -a 256 ./.dfx/ic/canisters/water_neuron/water_neuron.wasm.gz