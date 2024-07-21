#!/usr/bin/env bash 

set -euxo pipefail

PODMAN_HASH=$(podman build -q -f Dockerfile)

podman run -it --rm -w /waterneuron \
    --userns=keep-id \
    --mount type=bind,source="${HOME}",target="${HOME}" \
    --mount type=bind,source="$(pwd)",target="/waterneuron" \
    "$PODMAN_HASH"
