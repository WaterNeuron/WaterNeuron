# WaterNeuron
Liquid staking protocol on the Internet Computer

## Building
-   We assume you are building on a Ubuntu +22.04 based machine.
-   Packages pre-requisites: `podman`, `git`
```bash
sudo apt-get install podman git
```

-   To test the canister hash of the current commit
```bash
./run.sh --build
```

-   To dev
```bash
./run.sh
```

## Verify
### [Proposal#9](https://dashboard.internetcomputer.org/sns/jmod6-4iaaa-aaaaq-aadkq-cai/proposal/9)

Spin up the local container
```bash
./run.sh --build
```

Verify the proposal
```bash
bazel run //scripts/verify:bin -- \
    --proposal-id 9 \
    --wasm-hash 18f89aafc01d87a5cb62be8f189e80f9810126e4603a820226436fe537039510 \
    --git-commit d81ded9adbee1475f8f63b125d23eec861c9d163 \
    --target-canister tsbvt-pyaaa-aaaar-qafva-cai \
    --canister water-neuron-canister \
    --upgrade-args '(LiquidArg)' '(variant{Init = record {wtn_ledger_id=principal "jcmow-hyaaa-aaaaq-aadlq-cai"; wtn_governance_id=principal "jfnic-kaaaa-aaaaq-aadla-cai"; nicp_ledger_id=principal "buwm7-7yaaa-aaaar-qagva-cai"}})'

```

### [Proposal#27](https://dashboard.internetcomputer.org/sns/jmod6-4iaaa-aaaaq-aadkq-cai/proposal/27)

Spin up the local container
```bash
./scripts/build.sh
```

Verify the proposal
```bash
bazel run //scripts/verify:bin -- \
    --proposal-id 27 \
    --wasm-hash e6e2b66179372f8806fc9e18f13c0b905285c0b07af455f8ecaedbd621f62d83 \
    --git-commit a8aad61870061619b14d61f3f9f1bb279dd03047 \
    --target-canister tsbvt-pyaaa-aaaar-qafva-cai \
    --canister water-neuron-canister \
    --upgrade-args '(LiquidArg)' '(variant{Upgrade})'
```
