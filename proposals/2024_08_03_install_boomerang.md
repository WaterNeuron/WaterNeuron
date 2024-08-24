```
quill sns make-upgrade-canister-proposal 6ef86c9b566150ac7ab4cecea6a1e78bfde679f5973dc50c456878238c1c283c \
    --target-canister-id daijl-2yaaa-aaaar-qag3a-cai \
    --wasm-path "./artifacts/boomerang.wasm.gz" \
    --canister-upgrade-arg-path "./boomerang.bin" \
    --mode install \
    --summary "
# Install Boomerang Module

This is a proposal to install the Boomerang module.

## Motivation

It's currently not trivial for SNS DAOs to stake ICP, with this module staking becomes straightforward.

## Init args

The args module hash is \`8bb1c9b38e1edcc9f6c2cee96ac61b1c11255345e6933d6ec9c632c6c5fc31d6\`.

\`\`\`
git fetch
git checkout 85b446d2e3d9b230dc6f29490643350a663abed1
didc encode -d boomerang/boomerang.did -t '(CanisterIds)' '(record { wtn_ledger_id = principal "jcmow-hyaaa-aaaaq-aadlq-cai"; icp_ledger_id = principal "ryjl3-tyaaa-aaaaa-aaaba-cai"; water_neuron_id = principal "tsbvt-pyaaa-aaaar-qafva-cai"; nicp_ledger_id = principal "buwm7-7yaaa-aaaar-qagva-cai"; })' | xxd -r -p > boomerang.bin
sha256sum boomerang.bin
\`\`\`

## Wasm Verification

The compressed canister WebAssembly module is built from commit \`85b446d2e3d9b230dc6f29490643350a663abed1\`.
The compressed module hash is \`a03f94ed89f096972495c24c3a2c1709831b94e96be412d288e763dc8bf57d66\`.
Target canister: \`daijl-2yaaa-aaaar-qag3a-cai\`.
To build the wasm module yourself and verify its hash, run the following commands from the root of the WaterNeuron repo:

\`\`\`
git fetch
git checkout 85b446d2e3d9b230dc6f29490643350a663abed1
./run.sh --build
\`\`\`
    " \
    --title "Install Boomerang Module" \
    --url "https://github.com/WaterNeuron/WaterNeuron" \
    --pem-file /home/leo/.config/dfx/identity/default/identity.pem \
    --canister-ids-file ./sns_canister_ids.json > msg.json
```

```
https://dashboard.internetcomputer.org/sns/jmod6-4iaaa-aaaaq-aadkq-cai/proposal/145
```

didc encode -d boomerang/boomerang.did -t '(CanisterIds)' '(record { wtn_ledger_id = principal "jcmow-hyaaa-aaaaq-aadlq-cai"; icp_ledger_id = principal "ryjl3-tyaaa-aaaaa-aaaba-cai"; water_neuron_id = principal "tsbvt-pyaaa-aaaar-qafva-cai"; nicp_ledger_id = principal "buwm7-7yaaa-aaaar-qagva-cai"; })'
