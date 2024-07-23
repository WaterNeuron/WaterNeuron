```
quill sns make-upgrade-canister-proposal 6ef86c9b566150ac7ab4cecea6a1e78bfde679f5973dc50c456878238c1c283c \
    --target-canister-id buwm7-7yaaa-aaaar-qagva-cai \
    --wasm-path "./ic-icrc1-ledger.wasm.gz" \
    --canister-upgrade-arg-path "./ledger_arg.bin" \
    --mode install \
    --summary "
# Install nICP ledger

The compressed canister WebAssembly module is built from commit \`a3831c87440df4821b435050c8a8fcb3745d86f6\`.

The compressed module hash is \`4264ce2952c4e9ff802d81a11519d5e3ffdaed4215d5831a6634e59efd72f7d8\`.

## Init args

You can verify the init arguments with the following commands:
\`\`\`
didc encode -d rs/rosetta-api/icrc1/ledger/ledger.did -t \`(InitArgs)\` \`(record { minting_account = record { owner = principal \"tsbvt-pyaaa-aaaar-qafva-cai\" }; fee_collector_account = opt record { owner = principal \"jfnic-kaaaa-aaaaq-aadla-cai\"; }; transfer_fee = 10_000; token_symbol = \"nICP\"; token_name = \"neuron ICP\"; metadata = vec { record { \"icrc1:logo\"; variant { Text = \"data:image/png;base64,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\" }} }; initial_balances = vec {}; archive_options = record { num_blocks_to_archive = 1000; trigger_threshold = 2000; max_message_size_bytes = null; cycles_for_archive_creation = opt 10_000_000_000_000; node_max_memory_size_bytes = opt 3_221_225_472; controller_id = principal \"jmod6-4iaaa-aaaaq-aadkq-cai\" } })\`
\`\`\`

Notes:
 - \`tsbvt-pyaaa-aaaar-qafva-cai\` is the protocol canister, responsible for minting.
 - \`jfnic-kaaaa-aaaaq-aadla-cai\` is the WTN governance canister.
 - The transfer fee is the same as ICP: 0.0001 nICP.
 - There are no initial balances: the protocol canister is responsible for minting all nICP.

The metadata contains the official nICP logo. You can copy the following bytes into your browser\`s address line and hit Enter to see the logo.
\`\`\`
data:image/png;base64,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
\`\`\`


## Wasm Verification
To build the wasm module yourself and verify its hash, run the following commands from the root of the ic repo:
\`\`\`
git fetch
git checkout a3831c87440df4821b435050c8a8fcb3745d86f6
./gitlab-ci/container/build-ic.sh -c
sha256sum artifacts/canisters/ic-icrc1-ledger.wasm.gz
\`\`\`
    " \
    --title "Install nICP ledger" \
    --url "https://github.com/dfinity/ic" \
    --hsm-slot 0 \
    --canister-ids-file ./sns_canister_ids.json > msg.json
```


```
didc encode -d candid/sns_ledger.did -t '(InitArgs)' '(record { 
    minting_account = record { 
        owner = principal "tsbvt-pyaaa-aaaar-qafva-cai" 
    }; 
    fee_collector_account = opt record {
        owner = principal "jfnic-kaaaa-aaaaq-aadla-cai";
    };
    transfer_fee = 10_000; 
    token_symbol = "nICP"; 
    token_name = "neuron ICP"; 
    metadata = vec { 
        record { "icrc1:logo"; variant { Text = "data:image/png;base64,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" }}
    };
    initial_balances = vec {}; 
    archive_options = record { 
        num_blocks_to_archive = 1000;
        trigger_threshold = 2000;
        max_message_size_bytes = null; 
        cycles_for_archive_creation = opt 10_000_000_000_000; 
        node_max_memory_size_bytes = opt 3_221_225_472; 
        controller_id = principal "jmod6-4iaaa-aaaaq-aadkq-cai" 
    } 
})' | xxd -r -p > ledger_arg.bin
```

```
https://dashboard.internetcomputer.org/sns/jmod6-4iaaa-aaaaq-aadkq-cai/proposal/5
```