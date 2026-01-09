```
quill sns make-upgrade-canister-proposal 85ff8b442cca2eb2943fe74127085745f16d95b0d539993cd093f682f774dca8 \
    --target-canister-id tsbvt-pyaaa-aaaar-qafva-cai \
    --wasm-path "./water_neuron.wasm.gz" \
    --canister-upgrade-arg-path "./water_neuron_arg.bin" \
    --mode upgrade \
    --summary "
# Upgrade WaterNeuron Protocol

This is a proposal to upgrade the WaterNeuron protocol that updates the did file to match latest changes.

## Upgrade args

The args module hash is \`9b723b5ed323ebb32f08d9ea96f10523987b95e4fe72bfb3e23f41578bbf7972\`.

\`\`\`
git fetch
git checkout cd865e97772ff93f7d10b939d86f0b2fd0e52eff
cd water_neuron
didc encode -d water_neuron/water_neuron.did -t '(LiquidArg)' '(variant{Upgrade})' | xxd -r -p > water_neuron_arg.bin
sha256sum water_neuron_arg.bin
\`\`\`

## Wasm Verification

The compressed canister WebAssembly module is built from commit \`cd865e97772ff93f7d10b939d86f0b2fd0e52eff\`.
The compressed module hash is \`6dc0b6aaa3f51a8dc1f6378eacfced768f0ce39684a14d88fb230ddaf03c6910\`.
Target canister: \`tsbvt-pyaaa-aaaar-qafva-cai\`.

To build the wasm module yourself and verify its hash, run the following commands from the root of the water_neuron repo:
\`\`\`
git fetch
git checkout cd865e97772ff93f7d10b939d86f0b2fd0e52eff
./build.sh
\`\`\`
    " \
    --title "Upgrade WaterNeuron Protocol" \
    --url "https://github.com/WaterNeuron/WaterNeuron" \
    --pem-file ~/.config/dfx/identity/default/identity.pem \
    --canister-ids-file ./sns_canister_ids.json > msg.json
```
didc encode -d water_neuron/water_neuron.did -t '(LiquidArg)' '(variant{Init = record {wtn_ledger_id=principal "jcmow-hyaaa-aaaaq-aadlq-cai"; wtn_governance_id=principal "jfnic-kaaaa-aaaaq-aadla-cai"; nicp_ledger_id=principal "buwm7-7yaaa-aaaar-qagva-cai"}})'