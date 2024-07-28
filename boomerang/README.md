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
  │ get_staking_account_id(SNS) │                                    │            │            │      
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