# Setting up the mock minting account for e2e tests.
# We assume the the ./local_deploy.sh has been used.

dfx identity use icp-ident-RqOPnjj5ERjAEnwlvfKw
dfx ledger transfer --memo 0 --icp 1_000_000 90526bdfd692793cba1f96bde9079994ce4d40033746f04c12064ea599e2c274
dfx identity use default
intermediary=$( (dfx ledger account-id))
dfx identity use icp-ident-RqOPnjj5ERjAEnwlvfKw
dfx ledger transfer --memo 0 --icp 1_000_000 $intermediary
dfx identity use default
dfx canister call nns-ledger icrc2_approve '(record {spender= record{owner=principal "tsbvt-pyaaa-aaaar-qafva-cai";}; amount=100_000_000_000_000;})'
dfx canister call water_neuron icp_to_nicp '(record{amount_e8s=10_000_000_000_000})'
dfx canister call nicp_ledger icrc1_transfer '(record{to=record{owner=principal "syna7-6ipnd-myx4g-ia46u-nxwok-u5nrr-yxgpi-iang7-lvru2-i7n23-tqe"}; amount=9_000_000_000_000;})'