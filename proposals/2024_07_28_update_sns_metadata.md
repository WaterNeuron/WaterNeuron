```
dfx canister call jfnic-kaaaa-aaaaq-aadla-cai manage_neuron '(record {
    subaccount = blob "\6e\f8\6c\9b\56\61\50\ac\7a\b4\ce\ce\a6\a1\e7\8b\fd\e6\79\f5\97\3d\c5\0c\45\68\78\23\8c\1c\28\3c"; 
    command = opt variant {
        MakeProposal = record{
            url = "https://waterneuron.fi/stake/"; 
            title = "Update SNS metadata URL";
            summary = "
                Update URL.
            "; 
            action = opt variant { 
                ManageSnsMetadata = record { 
                    url = opt "https://waterneuron.fi/stake/"
                }
            }
        }
    }
})' --network ic
```


```
https://dashboard.internetcomputer.org/sns/jmod6-4iaaa-aaaaq-aadkq-cai/proposal/51
```