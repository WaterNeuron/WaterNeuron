dfx identity use default

dfx nns install

cp /Users/ulyssekaz/sisyphe/wtn_svelte/wasm/governance-canister_test.wasm /Users/ulyssekaz/.cache/dfinity/versions/0.16.1/wasms/governance-canister_test.wasm
cp /Users/ulyssekaz/sisyphe/wtn_svelte/wasm/governance-canister_test.wasm /Users/ulyssekaz/.cache/dfinity/versions/0.16.1/wasms/governance-canister.wasm


dfx canister create water_neuron --specified-id bgq3g-tiaaa-aaaar-qagwa-cai
dfx canister create nicp_ledger --specified-id bbr5s-6qaaa-aaaar-qagwq-cai
dfx canister create wtn_ledger --specified-id jcmow-hyaaa-aaaaq-aadlq-cai

dfx deploy water_neuron --argument '(variant{Init=record{nicp_ledger_id=principal "bbr5s-6qaaa-aaaar-qagwq-cai"; wtn_ledger_id=principal "jcmow-hyaaa-aaaaq-aadlq-cai"; wtn_governance_id=principal "jfnic-kaaaa-aaaaq-aadla-cai"}})' --mode reinstall -y
dfx deploy nicp_ledger --argument '(variant { Init = record { minting_account = record { owner = principal "bgq3g-tiaaa-aaaar-qagwa-cai" }; feature_flags  = opt record { icrc2 = true }; decimals = opt 8; max_memo_length = opt 80; transfer_fee = 10_000; token_symbol = "NICP"; token_name = "Neuron Internet Computer"; metadata = vec {}; initial_balances = vec {} ; archive_options = record { num_blocks_to_archive = 1000; trigger_threshold = 2000; max_message_size_bytes = null; cycles_for_archive_creation = opt 1_000_000_000; node_max_memory_size_bytes = opt 3_221_225_472; controller_id = principal "mf7xa-laaaa-aaaar-qaaaa-cai"; } }})'
dfx deploy wtn_ledger --argument '(variant { Init = record { minting_account = record { owner = principal "bgq3g-tiaaa-aaaar-qagwa-cai" }; feature_flags  = opt record { icrc2 = true }; decimals = opt 8; max_memo_length = opt 80; transfer_fee = 10_000; token_symbol = "WTN"; token_name = "Water Neuron"; metadata = vec {}; initial_balances = vec {}; archive_options = record { num_blocks_to_archive = 1000; trigger_threshold = 2000; max_message_size_bytes = null; cycles_for_archive_creation = opt 1_000_000_000; node_max_memory_size_bytes = opt 3_221_225_472; controller_id = principal "mf7xa-laaaa-aaaar-qaaaa-cai"; } }})'