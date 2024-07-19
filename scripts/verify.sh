#!/usr/bin/env bash

# Make sure we have the following are correct:
# - canister wasm hash
# - canister id
# - canister upgrade arg
# - proposal id
# - git hash

set -euxo pipefail

# fetch with get_proposal from the arguments passed

# get the new_canister_wasm, canister_id, and canister_upgrade_arg

# check if the canister_id is the same as the canister_id in the proposal

# check if the canister_upgrade_arg is the same as the canister_upgrade_arg in the proposal

# check if the canister_wasm is the same as the canister_wasm in the proposal

# make sure the proposal_id is the same as the proposal_id in the proposal

# make sure the git hash is the same as the git hash in the proposal

# make sure we have the right number of args
if [ "$#" -ne 1 ]; then
    echo "Usage: bazel run //scripts:verify -- <canister_hash>"
    exit 1
fi

CANISTER_PATH="$(readlink "$canister_path")"
SHA_256_SUM=$(sha256sum "$CANISTER_PATH" | cut -d ' ' -f 1)

# get the first arg
ARG_HASH_CANISTER=$1

if [ "$ARG_HASH_CANISTER" != "$SHA_256_SUM" ]; then
    echo "Hash mismatch: $ARG_HASH_CANISTER != $SHA_256_SUM"
    exit 1
else
    echo "Hash match: $ARG_HASH_CANISTER == $SHA_256_SUM"
fi
