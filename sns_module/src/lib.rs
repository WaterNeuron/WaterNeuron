use candid::{Nat, Principal};
use ic_base_types::PrincipalId;
use ic_nervous_system_common::ledger::compute_neuron_staking_subaccount;
use icp_ledger::{AccountIdentifier, Subaccount};
use icrc_ledger_client_cdk::{CdkRuntime, ICRC1Client};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::TransferArg;
use icrc_ledger_types::icrc1::transfer::TransferError;
use sha2::{Digest, Sha256};

pub mod memory;

pub async fn transfer(
    from_subaccount: Option<[u8; 32]>,
    to: impl Into<Account>,
    amount: Nat,
    fee: Option<Nat>,
    ledger_canister_id: Principal,
) -> Result<u64, TransferError> {
    let client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id,
    };
    let block_index = client
        .transfer(TransferArg {
            from_subaccount,
            to: to.into(),
            fee,
            created_at_time: None,
            memo: None,
            amount,
        })
        .await
        .map_err(|e| TransferError::GenericError {
            error_code: (Nat::from(e.0 as u32)),
            message: (e.1),
        })??;
    Ok(block_index.0.try_into().unwrap())
}

pub fn derive_staking(principal: Principal) -> [u8; 32] {
    const DOMAIN: &[u8] = b"STAKE-ICP";

    let mut hasher = Sha256::new();
    hasher.update(DOMAIN);
    hasher.update(principal.as_slice());
    hasher.finalize().into()
}
