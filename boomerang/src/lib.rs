use candid::{CandidType, Deserialize, Nat, Principal};
use icrc_ledger_types::icrc1::transfer::TransferError;
use icrc_ledger_types::icrc2::approve::ApproveError;
use icrc_ledger_types::icrc2::transfer_from::TransferFromError;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::cell::RefCell;

pub mod icp_to_nicp;
pub mod log;
pub mod nicp_to_icp;

#[cfg(test)]
pub mod state_machine;

#[derive(Deserialize, CandidType, Clone)]
pub struct CanisterIds {
    pub icp_ledger_id: Principal,
    pub nicp_ledger_id: Principal,
    pub wtn_ledger_id: Principal,
    pub water_neuron_id: Principal,
}

thread_local! {
    static __CANISTER_IDS: RefCell<Option<CanisterIds>> = RefCell::default();
}

/// Clones the canister ids.
///
/// Panics if there is no state.
pub fn get_canister_ids() -> CanisterIds {
    __CANISTER_IDS.with(|c| {
        let canister_ids = c.borrow();
        canister_ids.clone().expect("Canister Ids not initialized!")
    })
}

/// Replaces the current state.
pub fn set_canister_ids(canister_ids: CanisterIds) {
    __CANISTER_IDS.with(|c| {
        *c.borrow_mut() = Some(canister_ids);
    });
}

pub const E8S: u64 = 100_000_000;
pub const TRANSFER_FEE: u64 = 10_000;

pub fn derive_subaccount_staking(principal: Principal) -> [u8; 32] {
    const DOMAIN: &[u8] = b"STAKE-ICP";

    let mut hasher = Sha256::new();
    hasher.update(DOMAIN);
    hasher.update(principal.as_slice());
    hasher.finalize().into()
}

pub fn derive_subaccount_unstaking(principal: Principal) -> [u8; 32] {
    const DOMAIN: &[u8] = b"UNSTAKE-nICP";

    let mut hasher = Sha256::new();
    hasher.update(DOMAIN);
    hasher.update(principal.as_slice());
    hasher.finalize().into()
}

#[test]
fn should_return_different_array() {
    let p = Principal::anonymous();

    assert_ne!(derive_subaccount_staking(p), derive_subaccount_unstaking(p));

    let p1 =
        Principal::from_text("xwpbi-y7r63-dbg7j-ukl5y-5ncft-j5zsv-6uca6-rj5ly-e5xa7-qjlm3-xqe")
            .unwrap();
    let p2 =
        Principal::from_text("i57ky-ppa5u-2nmqo-ngzn6-3y6pl-4jqv2-b44iu-kdix5-76gp3-vxfjz-kqe")
            .unwrap();

    assert_ne!(derive_subaccount_staking(p1), derive_subaccount_staking(p2));
    assert_ne!(
        derive_subaccount_unstaking(p1),
        derive_subaccount_unstaking(p2)
    );
}

#[cfg(target_arch = "wasm32")]
pub fn self_canister_id() -> Principal {
    ic_cdk::id()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn self_canister_id() -> Principal {
    Principal::anonymous()
}

pub struct Icrc1TransferArg {
    pub amount_e8s: Nat,
    pub fee_e8s: u64,
    pub ledger_id: Principal,
    pub to: Principal,
}

#[derive(Debug, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum GuardError {
    AlreadyProcessing,
    TooManyConcurrentRequests,
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ConversionArg {
    pub amount_e8s: u64,
    pub maybe_subaccount: Option<[u8; 32]>,
}

#[derive(CandidType, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum ConversionError {
    TransferFromError(TransferFromError),
    TransferError(TransferError),
    AmountTooLow { minimum_amount_e8s: u64 },
    GuardError { guard_error: GuardError },
    GenericError { code: i32, message: String },
}

#[derive(CandidType, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct DepositSuccess {
    pub block_index: Nat,
    pub transfer_id: u64,
    pub nicp_amount: Option<u64>,
}

#[derive(CandidType, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct WithdrawalSuccess {
    block_index: Nat,
    withdrawal_id: u64,
}

#[derive(CandidType, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum BoomerangError {
    ApproveError(ApproveError),
    ConversionError(ConversionError),
    TransferError(TransferError),
    GenericError { code: i32, message: String },
    NotEnoughICP,
}
