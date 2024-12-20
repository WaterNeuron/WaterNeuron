#!/usr/bin/env bash

set -euxo pipefail

MODE="dev"
ARTIFACTS_DIR="$(pwd)/artifacts"

rm -rf "$ARTIFACTS_DIR"
mkdir -p "$ARTIFACTS_DIR"

while [[ $# -gt 0 ]]; do
    case $1 in
        --build)
            MODE="build"
            shift
            ;;
    esac
done

PODMAN_HASH=$(podman build -q -f Dockerfile .)

PODMAN_ARGS=(
    -it
    --rm
    -w /waterneuron
    --userns=keep-id
    --mount type=bind,source=$(pwd),target=/waterneuron
    --mount type=bind,source=${ARTIFACTS_DIR},target=/artifacts
    "$PODMAN_HASH"
)

if [[ "$MODE" == "build" ]]; then
    PODMAN_ARGS+=(
        /usr/bin/bash
        -c
        "cargo canisters"
    )
fi

podman run "${PODMAN_ARGS[@]}"
