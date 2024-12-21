#!/usr/bin/env bash

nix develop -i -k HOME -c bash -c "cargo canisters"
