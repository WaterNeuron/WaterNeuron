```
quill sns make-upgrade-canister-proposal 6ef86c9b566150ac7ab4cecea6a1e78bfde679f5973dc50c456878238c1c283c \
    --target-canister-id btxkl-saaaa-aaaar-qagvq-cai \
    --wasm-path "./ic-icrc1-index-ng.wasm.gz" \
    --canister-upgrade-arg-path "./index_arg.bin" \
    --mode install \
    --summary "
# Install nICP Index Canister
The compressed canister WebAssembly module is built from commit \`a3831c87440df4821b435050c8a8fcb3745d86f6\`.
The compressed module hash is \`cac207cf438df8c9fba46d4445c097f05fd8228a1eeacfe0536b7e9ddefc5f1c\`.
Target canister: \`btxkl-saaaa-aaaar-qagvq-cai\`.
## Motivation
This is a proposal to install the nICP index canister.
## Init args
\`\`\`
git fetch
git checkout a3831c87440df4821b435050c8a8fcb3745d86f6
cd rs/rosetta-api/icrc1/index-ng
didc encode -d index-ng.did -t '(opt IndexArg)' '(opt variant{Init=record{ledger_id=principal \"buwm7-7yaaa-aaaar-qagva-cai\"}})'
\`\`\`
## Wasm Verification
To build the wasm module yourself and verify its hash, run the following commands from the root of the ic repo:
\`\`\`
git fetch
git checkout a3831c87440df4821b435050c8a8fcb3745d86f6
./gitlab-ci/container/build-ic.sh -c
sha256sum ./artifacts/canisters/ic-icrc1-index-ng.wasm.gz
\`\`\`
    " \
    --title "Install nICP Index" \
    --url "https://github.com/dfinity/ic" \
    --hsm-slot 0 \
    --canister-ids-file ./sns_canister_ids.json > msg.json
```

```
https://dashboard.internetcomputer.org/sns/jmod6-4iaaa-aaaaq-aadkq-cai/proposal/7
```