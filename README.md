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
List of proposals changing the canister

### [Proposal#9](https://dashboard.internetcomputer.org/sns/jmod6-4iaaa-aaaaq-aadkq-cai/proposal/9)
```bash
bazel run //scripts/verify:bin --action_env=STABLE_GIT_COMMIT=d81ded9adbee1475f8f63b125d23eec861c9d163 -- \
    --proposal-id 9 \
    --wasm-hash 18f89aafc01d87a5cb62be8f189e80f9810126e4603a820226436fe537039510 \
    --git-commit d81ded9adbee1475f8f63b125d23eec861c9d163 \
    --target-canister tsbvt-pyaaa-aaaar-qafva-cai \
    --canister water-neuron-canister \
    --upgrade-args '(LiquidArg)' '(variant{Init = record {wtn_ledger_id=principal "jcmow-hyaaa-aaaaq-aadlq-cai"; wtn_governance_id=principal "jfnic-kaaaa-aaaaq-aadla-cai"; nicp_ledger_id=principal "buwm7-7yaaa-aaaar-qagva-cai"}})'

```

### [Proposal#27](https://dashboard.internetcomputer.org/sns/jmod6-4iaaa-aaaaq-aadkq-cai/proposal/27)
```bash
bazel run //scripts/verify:bin --action_env=STABLE_GIT_COMMIT=a8aad61870061619b14d61f3f9f1bb279dd03047 -- \
    --proposal-id 27 \
    --wasm-hash e6e2b66179372f8806fc9e18f13c0b905285c0b07af455f8ecaedbd621f62d83 \
    --git-commit a8aad61870061619b14d61f3f9f1bb279dd03047 \
    --target-canister tsbvt-pyaaa-aaaar-qafva-cai \
    --canister water-neuron-canister \
    --upgrade-args '(LiquidArg)' '(variant{Upgrade})'
```

### [Proposal#29](https://dashboard.internetcomputer.org/sns/jmod6-4iaaa-aaaaq-aadkq-cai/proposal/29)
```bash
bazel run //scripts/verify:bin --action_env=STABLE_GIT_COMMIT=f5aa7c12166c94d0f652fea2ee4527040c4b059f -- \
    --proposal-id 29 \
    --wasm-hash ec46abad688b9385a64bd70772af8c65b615e486b8bc0117ac59faa3139cf463 \
    --git-commit f5aa7c12166c94d0f652fea2ee4527040c4b059f \
    --target-canister tsbvt-pyaaa-aaaar-qafva-cai \
    --canister water-neuron-canister \
    --upgrade-args '(LiquidArg)' '(variant{Upgrade})'
```
