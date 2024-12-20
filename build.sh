#!/usr/bin/env bash
set -euxo pipefail

MODE="remote"
CONTAINER_IMAGE="ghcr.io/waterneuron/waterneuron@sha256:66423f63a6b9110781c134cf643088856ca141a751bc33dea0e5b8a2d1470846"

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
    -w /waterneuron
    --userns=keep-id
    --mount type=bind,source=$(pwd),target=/waterneuron
    "$PODMAN_HASH"
    /usr/bin/bash
    -c
    "cargo canisters"
)

podman run "${PODMAN_ARGS[@]}"
