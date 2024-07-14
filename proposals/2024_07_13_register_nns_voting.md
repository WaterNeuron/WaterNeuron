```
dfx canister call wtn_governance manage_neuron '(record {
    subaccount = blob "\6e\f8\6c\9b\56\61\50\ac\7a\b4\ce\ce\a6\a1\e7\8b\fd\e6\79\f5\97\3d\c5\0c\45\68\78\23\8c\1c\28\3c"; 
    command = opt variant {
        MakeProposal = record{
            url = "https://docs.waterneuron.fi/"; 
            title = "Add a custom SNS function to approve an NNS proposal";
            summary = "Approves a given NNS proposal. If no strict majority is reached, vote according to the majority opinion."; 
            action = opt variant { 
                AddGenericNervousSystemFunction = record { 
                    id = 1000;
                    name = "Vote on NNS proposal";
                    description = opt "Vote yes to a given NNS proposal. If there is no strict majority, vote for the majority outcome.";
                    function_type = opt variant {
                        GenericNervousSystemFunction = record {
                            validator_canister_id = opt principal "tsbvt-pyaaa-aaaar-qafva-cai";
                            target_canister_id = opt principal "tsbvt-pyaaa-aaaar-qafva-cai";
                            validator_method_name = opt "approve_proposal_validate";
                            target_method_name = opt "approve_proposal";
                        }
                    };
                }
            }
        }
    }
})' --network ic
```

```
https://dashboard.internetcomputer.org/sns/jmod6-4iaaa-aaaaq-aadkq-cai/proposal/3
```