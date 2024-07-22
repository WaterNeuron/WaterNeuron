```
quill sns make-upgrade-canister-proposal 6ef86c9b566150ac7ab4cecea6a1e78bfde679f5973dc50c456878238c1c283c \
    --target-canister-id tsbvt-pyaaa-aaaar-qafva-cai \
    --wasm-path "./waterneuron.wasm.gz" \
    --canister-upgrade-arg-path "./water_neuron_arg.bin" \
    --mode install \
    --summary "
# Install WaterNeuron Protocol in restricted mode

This is a proposal to install the WaterNeuron protocol with only members of SISYPHE being able to interact.

## Motivation

This proposal installs the WaterNeuron protocol in restricted mode.
 - Verify neurons are created and configured as expected.
 - Verify proposals mirroring is working.
 - Verify all the endpoints are correct.
 - Verify the frontend is working accordingly.
 - Add WaterNeuron to DefiLlama.
 - Start the transfer of ICP in the 8-year neuron.
 - Setup canister monitoring.

## Init args

The args module hash is \`c22e4b18f746af362906f7ee56264fcd3a7c38f5797618a4748f4e7b0c74215b\`.

\`\`\`
git fetch
git checkout d81ded9adbee1475f8f63b125d23eec861c9d163
cd water_neuron
didc encode -d water_neuron.did -t '(LiquidArg)' '(variant{Init = record {wtn_ledger_id=principal \"jcmow-hyaaa-aaaaq-aadlq-cai\"; wtn_governance_id=principal \"jfnic-kaaaa-aaaaq-aadla-cai\"; nicp_ledger_id=principal \"buwm7-7yaaa-aaaar-qagva-cai\"}})' | xxd -r -p > water_neuron_arg.bin
sha256sum water_neuron_arg.bin
\`\`\`

## Wasm Verification

The compressed canister WebAssembly module is built from commit \`d81ded9adbee1475f8f63b125d23eec861c9d163\`.
The compressed module hash is \`18f89aafc01d87a5cb62be8f189e80f9810126e4603a820226436fe537039510\`.
Target canister: \`tsbvt-pyaaa-aaaar-qafva-cai\`.

To build the wasm module yourself and verify its hash, run the following commands from the root of the water_neuron repo:
\`\`\`
git fetch
git checkout d81ded9adbee1475f8f63b125d23eec861c9d163
./scripts/build.sh --build
\`\`\`
    " \
    --title "Install WaterNeuron Protocol in restricted mode" \
    --url "https://github.com/WaterNeuron/WaterNeuron" \
    --hsm-slot 0 \
    --canister-ids-file ./sns_canister_ids.json > msg.json
```

```
https://dashboard.internetcomputer.org/sns/jmod6-4iaaa-aaaaq-aadkq-cai/proposal/9
```

didc encode -d water_neuron/water_neuron.did -t '(LiquidArg)' '(variant{Init = record {wtn_ledger_id=principal "jcmow-hyaaa-aaaaq-aadlq-cai"; wtn_governance_id=principal "jfnic-kaaaa-aaaaq-aadla-cai"; nicp_ledger_id=principal "buwm7-7yaaa-aaaar-qagva-cai"}})'