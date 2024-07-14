```
dfx canister call jfnic-kaaaa-aaaaq-aadla-cai manage_neuron '(record {
    subaccount = blob "\6e\f8\6c\9b\56\61\50\ac\7a\b4\ce\ce\a6\a1\e7\8b\fd\e6\79\f5\97\3d\c5\0c\45\68\78\23\8c\1c\28\3c"; 
    command = opt variant {
        MakeProposal = record{
            url = "https://docs.waterneuron.fi/"; 
            title = "Register protocol canisters";
            summary = "Register the following canisters:
- `tsbvt-pyaaa-aaaar-qafva-cai`: WaterNeuron protocol canister.
- `buwm7-7yaaa-aaaar-qagva-cai`: nICP ledger canister.
- `btxkl-saaaa-aaaar-qagvq-cai`: nICP index canister.

All the canisters are on `pzp6e`, the fiduciary subnet.
            "; 
            action = opt variant { 
                RegisterDappCanisters = record { 
                    canister_ids = vec {
                        principal "tsbvt-pyaaa-aaaar-qafva-cai";
                        principal "buwm7-7yaaa-aaaar-qagva-cai";
                        principal "btxkl-saaaa-aaaar-qagvq-cai";
                    };
                }
            }
        }
    }
})' --network ic
```


```
https://dashboard.internetcomputer.org/sns/jmod6-4iaaa-aaaaq-aadkq-cai/proposal/2
```