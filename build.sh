#!/usr/bin/env bash
set -euxo pipefail

MODE="remote"
CONTAINER_IMAGE="ghcr.io/waterneuron/waterneuron@sha256:97aa8f78871ddf40d4b457a81f14c171a1d4ef3dfc402fdd4de0b442c99a31d8"

while [[ $# -gt 0 ]]; do
    case $1 in
        --local)
            MODE="local"
            shift
            ;;
    esac
done

if [[ "$MODE" == "local" ]]; then
    PODMAN_HASH=$(podman build -q -f Containerfile .)
else
    podman pull "$CONTAINER_IMAGE"
    PODMAN_HASH="$CONTAINER_IMAGE"
fi

PODMAN_ARGS=(
    -it
    --rm
    --userns=keep-id
    --mount type=bind,source=$(pwd),target=/waterneuron
    "$PODMAN_HASH"
    /usr/bin/bash
    -c
    "cargo canisters"
)

podman run "${PODMAN_ARGS[@]}"
