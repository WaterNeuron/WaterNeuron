# Boomerang Canister

The Boomerang canister is a helper canister that allows SNS DAOs to stake/unstake ICP easily.

We still recommand single individuals to use the DApp as it remains the simplest way to use the protocol. 

## Flow: ICP to nICP

Converting ICP from the treasury funds of the SNS is pretty straightforward.

```
┌───┐                      ┌─────────┐                          ┌──────────┐┌───────────┐┌───────────┐
│SNS│                      │Boomerang│                          │ICP ledger││WaterNeuron││nICP ledger│
└─┬─┘                      └────┬────┘                          └────┬─────┘└─────┬─────┘└─────┬─────┘
  │                             │                                    │            │            │      
  │   get_staking_account(SNS)  │                                    │            │            │      
  │────────────────────────────>│                                    │            │            │      
  │                             │                                    │            │            │      
  │   (*) TransferSnsTreasuryFunds(to Boomerang(sub SNS), amount)    │            │            │      
  │─────────────────────────────────────────────────────────────────>│            │            │      
  │                             │                                    │            │            │      
  │   notify_icp_deposit(SNS)   │                                    │            │            │      
  │────────────────────────────>│                                    │            │            │      
  │                             │                                    │            │            │      
  │                             │icrc2_balance_of(Boomerang(sub SNS))│            │            │      
  │                             │───────────────────────────────────>│            │            │      
  │                             │                                    │            │            │      
  │                             │ icrc2_approve(WaterNeuron, amount) │            │            │      
  │                             │───────────────────────────────────>│            │            │      
  │                             │                                    │            │            │      
  │                             │                  icp_to_nicp()     │            │            │      
  │                             │────────────────────────────────────────────────>│            │      
  │                             │                                    │            │            │      
  │     retreive_nicp(SNS)      │                                    │            │            │      
  │────────────────────────────>│                                    │            │            │      
  │                             │                                    │            │            │      
  │                             │             icrc2_balance_of(Boomerang(sub SNS))│            │      
  │                             │─────────────────────────────────────────────────────────────>│      
  │                             │                                    │            │            │      
  │                             │             icrc1_transfer(to SNS, nicp_amount) │            │      
  │                             │─────────────────────────────────────────────────────────────>│      
┌─┴─┐                      ┌────┴────┐                          ┌────┴─────┐┌─────┴─────┐┌─────┴─────┐
│SNS│                      │Boomerang│                          │ICP ledger││WaterNeuron││nICP ledger│
└───┘                      └─────────┘                          └──────────┘└───────────┘└───────────┘

```

## Flow: nICP to ICP

Step 1: Register a generic function to transfer nICP.

```
┌───┐                                                                                                        ┌───┐
│SNS│                                                                                                        │DAO│
└─┬─┘                                                                                                        └─┬─┘
  │                                                                                                            │  
  │(*) AddGenericNervousSystemFunction(target_canister nicp_ledger, method icrc1_transfer(to Boomerang, amount)│  
  │───────────────────────────────────────────────────────────────────────────────────────────────────────────>│  
┌─┴─┐                                                                                                        ┌─┴─┐
│SNS│                                                                                                        │DAO│
└───┘                                                                                                        └───┘

Step 2: Execute the previously registered function.

┌───┐                   ┌─────────┐                                ┌───────────┐┌───────────┐┌──────────┐
│SNS│                   │Boomerang│                                │nICP ledger││WaterNeuron││ICP ledger│
└─┬─┘                   └────┬────┘                                └─────┬─────┘└─────┬─────┘└────┬─────┘
  │                          │                                           │            │           │      
  │get_unstaking_account(SNS)│                                           │            │           │      
  │─────────────────────────>│                                           │            │           │      
  │                          │                                           │            │           │      
  │(*) ExecuteGenericNervousSystemFunction(to Boomerang(sub SNS), amount)│            │           │      
  │─────────────────────────────────────────────────────────────────────>│            │           │      
  │                          │                                           │            │           │      
  │ notify_nicp_deposit(SNS) │                                           │            │           │      
  │─────────────────────────>│                                           │            │           │      
  │                          │                                           │            │           │      
  │                          │   icrc2_balance_of(Boomerang(sub SNS))    │            │           │      
  │                          │──────────────────────────────────────────>│            │           │      
  │                          │                                           │            │           │      
  │                          │    icrc2_approve(WaterNeuron, amount)     │            │           │      
  │                          │──────────────────────────────────────────>│            │           │      
  │                          │                                           │            │           │      
  │                          │                     nicp_to_icp()         │            │           │      
  │                          │───────────────────────────────────────────────────────>│           │      
  │                          │                                           │            │           │      
  │  try_retreive_icp(SNS)   │                                           │            │           │      
  │─────────────────────────>│                                           │            │           │      
  │                          │                                           │            │           │      
  │                          │                icrc2_balance_of(Boomerang(sub SNS))    │           │      
  │                          │───────────────────────────────────────────────────────────────────>│      
  │                          │                                           │            │           │      
  │                          │                 icrc1_transfer(to SNS, icp_amount)     │           │      
  │                          │───────────────────────────────────────────────────────────────────>│      
┌─┴─┐                   ┌────┴────┐                                ┌─────┴─────┐┌─────┴─────┐┌────┴─────┐
│SNS│                   │Boomerang│                                │nICP ledger││WaterNeuron││ICP ledger│
└───┘                   └─────────┘                                └───────────┘└───────────┘└──────────┘

```

## Treasury proposals instructions
```
dfx canister call wtn_governance manage_neuron '(record {
    subaccount = blob "${NEURON_ID}"; 
    command = opt variant {
        MakeProposal = record {
            url = "https://docs.waterneuron.fi/"; 
            title = "Stake ICP Treasury with WaterNeuron";
            summary = "
This proposal to stake ICP treasury with WaterNeuron.
            "; 
            action = opt variant { 
                TransferSnsTreasuryFunds = record { 
                    from_treasury = 1;
                    to_principal = opt principal "daijl-2yaaa-aaaar-qag3a-cai";
                    to_subaccount = opt record {
                        subaccount = blob "${DAO_SUBACCOUNT}";
                    };
                    memo = null;
                    amount_e8s = $AMOUNT;
                }
            }
        }
    }
})' --network ic
```
