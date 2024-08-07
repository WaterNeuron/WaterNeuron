use candid::{CandidType, Deserialize, Nat, Principal};
use icrc_ledger_types::icrc1::transfer::TransferError;
use icrc_ledger_types::icrc2::approve::ApproveError;
use icrc_ledger_types::icrc2::transfer_from::TransferFromError;
use serde::Serialize;
use std::cell::RefCell;

pub mod icp_to_nicp;
pub mod log;
pub mod nicp_to_icp;

#[cfg(test)]
pub mod state_machine;

pub const E8S: u64 = 100_000_000;
pub const TRANSFER_FEE: u64 = 10_000;

thread_local! {
    static __CANISTER_IDS: RefCell<Option<CanisterIds>> = RefCell::default();
}

#[cfg(target_arch = "wasm32")]
pub fn self_canister_id() -> Principal {
    ic_cdk::id()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn self_canister_id() -> Principal {
    Principal::anonymous()
}

#[derive(Deserialize, CandidType, Clone)]
pub struct CanisterIds {
    pub icp_ledger_id: Principal,
    pub nicp_ledger_id: Principal,
    pub wtn_ledger_id: Principal,
    pub water_neuron_id: Principal,
}

pub struct Icrc1TransferArg {
    pub amount_e8s: Nat,
    pub fee_e8s: u64,
    pub ledger_id: Principal,
    pub to: Principal,
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
