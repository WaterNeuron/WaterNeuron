use crate::state::read_state;
use candid::{CandidType, Nat, Principal};
use icrc_ledger_client_cdk::{CdkRuntime, ICRC1Client};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::{TransferArg, TransferError};
use serde::Deserialize;
use sha2::{Digest, Sha256};

pub mod memory;
pub mod state;

#[cfg(test)]
pub mod state_machine;

pub const E8S: u64 = 100_000_000;
pub const NANOS: u64 = 1_000_000_000;
pub const MIN_DEPOSIT_AMOUNT: u64 = 10 * E8S;

#[derive(CandidType, Deserialize)]
pub struct Status {
    pub participants: usize,
    pub total_icp_deposited: u64,
    pub time_left: u64,
    pub start_at: u64,
    pub end_at: u64,
    pub minimum_deposit_amount: u64,
}

pub fn is_swap_available() -> Result<(), String> {
    let time = ic_cdk::api::time() / NANOS;
    let (start_ts, end_ts) = read_state(|s| (s.start_ts, s.end_ts));
    if time < start_ts {
        return Err(format!("Swap didn't start yet, starting at {start_ts}"));
    }
    if time > end_ts {
        return Err(format!("Swap ended at {end_ts}"));
    }
    Ok(())
}

pub fn is_distribution_available() -> bool {
    let time = ic_cdk::api::time() / NANOS;
    let end_ts = read_state(|s| s.end_ts);
    time > end_ts
}

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

pub async fn balance_of(
    of: impl Into<Account>,
    ledger_canister_id: Principal,
) -> Result<u64, String> {
    let client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id,
    };
    let balance = client
        .balance_of(of.into())
        .await
        .map_err(|(code, e)| format!("{code} - {e}"))?;
    Ok(balance.0.try_into().unwrap())
}

pub fn derive_staking(principal: Principal) -> [u8; 32] {
    const DOMAIN: &[u8] = b"STAKE-ICP";

    let mut hasher = Sha256::new();
    hasher.update(DOMAIN);
    hasher.update(principal.as_slice());
    hasher.finalize().into()
}

pub fn dispatch_tokens(wtn_tokens: u64, balances: Vec<(Principal, u64)>) -> Vec<(Principal, u64)> {
    let total_tracked: u64 = balances.iter().map(|(_, tokens)| tokens).sum();
    let mut result: Vec<(Principal, u64)> = vec![];
    for (owner, balance) in balances {
        let wtn_share = balance as f64 / total_tracked as f64;
        let wtn_share_amount = (wtn_share * wtn_tokens as f64) as u64;
        result.push((owner, wtn_share_amount));
    }
    result
}

#[test]
fn should_dispatch_tokens() {
    let token_to_dispatch = 1_000_000 * E8S;
    let balances = vec![
        (
            Principal::from_text("vkp32-xurde-i5td6-chrbx-2b5p2-bogyg-qbckl-74ebs-xwvzo-jrwib-mqe")
                .unwrap(),
            25,
        ),
        (
            Principal::from_text("wlgcb-f7wlw-yvrlc-vvo7n-j7t4u-zair7-suih4-zvw7m-b7uwv-tapcl-sqe")
                .unwrap(),
            25,
        ),
        (
            Principal::from_text("aqbuz-ghmx7-hsjcb-hudle-m2olh-xkueg-nwd35-fkj3a-ykwsy-eawp5-3qe")
                .unwrap(),
            50,
        ),
    ];

    assert_eq!(
        dispatch_tokens(token_to_dispatch, balances),
        vec![
            (
                Principal::from_text(
                    "vkp32-xurde-i5td6-chrbx-2b5p2-bogyg-qbckl-74ebs-xwvzo-jrwib-mqe"
                )
                .unwrap(),
                250_000 * E8S
            ),
            (
                Principal::from_text(
                    "wlgcb-f7wlw-yvrlc-vvo7n-j7t4u-zair7-suih4-zvw7m-b7uwv-tapcl-sqe"
                )
                .unwrap(),
                250_000 * E8S
            ),
            (
                Principal::from_text(
                    "aqbuz-ghmx7-hsjcb-hudle-m2olh-xkueg-nwd35-fkj3a-ykwsy-eawp5-3qe"
                )
                .unwrap(),
                500_000 * E8S
            ),
        ]
    )
}
