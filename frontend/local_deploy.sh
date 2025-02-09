dfx identity use default

dfx extension run nns install

dfx canister create water_neuron --specified-id tsbvt-pyaaa-aaaar-qafva-cai
dfx canister create nicp_ledger --specified-id buwm7-7yaaa-aaaar-qagva-cai
dfx canister create wtn_ledger --specified-id jcmow-hyaaa-aaaaq-aadlq-cai
dfx canister create internet_identity --specified-id iidmm-fiaaa-aaaaq-aadmq-cai
dfx canister create boomerang --specified-id daijl-2yaaa-aaaar-qag3a-cai

dfx identity use icp-ident-RqOPnjj5ERjAEnwlvfKw
# WaterNeuron canister requires 3 ICP to set up.
# WaterNeuron account id: 3a9c912b5f869ae01dcd0ae0ce773da1d893a7ff0f72e3a103349e68e96d7d28
dfx ledger transfer --memo 0 --icp 3 3a9c912b5f869ae01dcd0ae0ce773da1d893a7ff0f72e3a103349e68e96d7d28

dfx identity use default
dfx deploy water_neuron --argument '(variant{Init=record{nicp_ledger_id=principal "buwm7-7yaaa-aaaar-qagva-cai"; wtn_ledger_id=principal "jcmow-hyaaa-aaaaq-aadlq-cai"; wtn_governance_id=principal "jcmow-hyaaa-aaaaq-aadlq-cai"}})'
dfx deploy nicp_ledger --argument '(variant { Init = record { minting_account = record { owner = principal "tsbvt-pyaaa-aaaar-qafva-cai" }; feature_flags  = opt record { icrc2 = true }; decimals = opt 8; max_memo_length = opt 80; transfer_fee = 10_000; token_symbol = "NICP"; token_name = "Neuron Internet Computer"; metadata = vec {}; initial_balances = vec {} ; archive_options = record { num_blocks_to_archive = 1000; trigger_threshold = 2000; max_message_size_bytes = null; cycles_for_archive_creation = opt 1_000_000_000; node_max_memory_size_bytes = opt 3_221_225_472; controller_id = principal "mf7xa-laaaa-aaaar-qaaaa-cai"; } }})'
dfx deploy wtn_ledger --argument '(variant { Init = record { minting_account = record { owner = principal "tsbvt-pyaaa-aaaar-qafva-cai" }; feature_flags  = opt record { icrc2 = true }; decimals = opt 8; max_memo_length = opt 80; transfer_fee = 10_000; token_symbol = "WTN"; token_name = "Water Neuron"; metadata = vec {}; initial_balances = vec {}; archive_options = record { num_blocks_to_archive = 1000; trigger_threshold = 2000; max_message_size_bytes = null; cycles_for_archive_creation = opt 1_000_000_000; node_max_memory_size_bytes = opt 3_221_225_472; controller_id = principal "mf7xa-laaaa-aaaar-qaaaa-cai"; } }})'
dfx deploy boomerang --argument '(record { water_neuron_id= principal "tsbvt-pyaaa-aaaar-qafva-cai"; nicp_ledger_id= principal "buwm7-7yaaa-aaaar-qagva-cai"; wtn_ledger_id= principal "jcmow-hyaaa-aaaaq-aadlq-cai"; icp_ledger_id= principal "ryjl3-tyaaa-aaaaa-aaaba-cai"})'
dfx deploy internet_identity

dfx deploy --specified-id zftzm-qqaaa-aaaam-adxfa-cai sns_module --argument '(record {
icp_ledger_id = principal "ryjl3-tyaaa-aaaaa-aaaba-cai";
  start_ts = 1733511600;
  wtn_ledger_id = principal "jcmow-hyaaa-aaaaq-aadlq-cai";
  end_ts = 1734116400;
})'