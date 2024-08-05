#!/usr/bin/env bash

set -euxo pipefail

REPO_PATH="$(dirname "$(readlink "$WORKSPACE")")"
cd "$REPO_PATH"

find . -path ./.git -prune -o -type f -name "*.sh" -exec shfmt -w -i 4 -bn -ci {} \+
