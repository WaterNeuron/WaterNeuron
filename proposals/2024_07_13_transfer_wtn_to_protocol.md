TransferSnsTreasuryFunds

```
dfx canister call wtn_governance manage_neuron '(record {
    subaccount = blob "\6e\f8\6c\9b\56\61\50\ac\7a\b4\ce\ce\a6\a1\e7\8b\fd\e6\79\f5\97\3d\c5\0c\45\68\78\23\8c\1c\28\3c"; 
    command = opt variant {
        MakeProposal = record {
            url = "https://docs.waterneuron.fi/"; 
            title = "Transfer 100k WTN to the protocol";
            summary = "
The WaterNeuron protocol continuously scans the NNS for new proposals. Upon detecting a new proposal, it submits an SNS proposal. To function correctly, the protocol canister must have a neuron with WTN staked. Each time a proposal is rejected, 100 WTN are deducted from this neuron, providing a buffer for up to 1,000 rejections with a total of 100,000 WTN staked. This neuron is not eligible for any rewards.
            "; 
            action = opt variant { 
                TransferSnsTreasuryFunds = record { 
                    from_treasury = 2;
                    to_principal = opt principal "jfnic-kaaaa-aaaaq-aadla-cai";
                    to_subaccount = opt record {
                        subaccount = blob "\08\b2\52\52\24\f2\cf\91\0a\f7\52\5c\9d\99\2b\a6\e2\26\a7\f5\5a\65\eb\f8\f0\0c\32\90\2f\d8\c7\44";
                    };
                    memo = null;
                    amount_e8s = 100_000_00_000_000;
                }
            }
        }
    }
})' --network ic
```

```
https://nns.ic0.app/proposal/?u=jmod6-4iaaa-aaaaq-aadkq-cai&proposal=4
```