use candid::{Nat, Principal};
use ic_base_types::PrincipalId;
use ic_cdk::{query, update};
use ic_nervous_system_common::ledger::compute_neuron_staking_subaccount;
use icp_ledger::{AccountIdentifier, Subaccount};
use icrc_ledger_types::icrc1::transfer::TransferError;
use sns_module::memory::{
    deposit_icp, get_principal_to_icp, get_principal_to_wtn_owed, set_wtn_owed,
};
use sns_module::{
    balance_of, derive_staking, dispatch_tokens, is_distribution_available, is_swap_available,
    transfer, Status, END_SWAP_TS, START_SWAP_TS,
};

fn main() {}

#[query]
fn get_status() -> Status {
    let balances = get_principal_to_icp();
    Status {
        participants: balances.len(),
        total_icp_deposited: balances.iter().map(|(_, b)| b).sum(),
        time_left: END_SWAP_TS.saturating_sub(ic_cdk::api::time()),
        start_at: START_SWAP_TS,
        end_at: END_SWAP_TS,
    }
}

#[update]
fn get_icp_deposit_address(target: Principal) -> AccountIdentifier {
    let subaccount = Subaccount(derive_staking(target));
    let self_id: PrincipalId = ic_cdk::id().into();
    AccountIdentifier::new(self_id, Some(subaccount))
}

#[update]
async fn notify_icp_deposit(target: Principal, amount: u64) -> Result<u64, TransferError> {
    assert!(amount >= 100_000_000);
    assert_ne!(ic_cdk::caller(), Principal::anonymous());
    assert!(is_swap_available());
    let icp_ledger = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();
    match transfer(
        Some(derive_staking(target)),
        ic_cdk::id(),
        Nat::from(amount),
        None,
        icp_ledger,
    )
    .await
    {
        Ok(block_index) => {
            let received_tokens = amount.checked_sub(10_000).unwrap();
            deposit_icp(target, received_tokens);
            Ok(block_index)
        }
        Err(e) => Err(e),
    }
}

#[update]
async fn distribute_tokens() -> Result<(), String> {
    assert_eq!(
        ic_cdk::caller(),
        Principal::from_text("bo5bf-eaaaa-aaaam-abtza-cai").unwrap()
    );
    assert!(get_principal_to_wtn_owed().is_empty());
    assert!(is_distribution_available());

    let icp_ledger = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();
    let icp_balance = balance_of(ic_cdk::id(), icp_ledger).await?;

    let wtn_ledger = Principal::from_text("jcmow-hyaaa-aaaaq-aadlq-cai").unwrap();
    let wtn_balance = balance_of(ic_cdk::id(), wtn_ledger).await?;

    let balances = sns_module::memory::get_principal_to_icp();
    let total_tracked_icp: u64 = balances.iter().map(|(_, b)| b).sum();
    assert!(icp_balance >= total_tracked_icp);

    let wtn_map = dispatch_tokens(wtn_balance, balances);

    for (owner, owed_wtn) in wtn_map {
        set_wtn_owed(owner, owed_wtn);
    }

    Ok(())
}

/// Checks the real candid interface against the one declared in the did file
/// Check that the types used to interact with the NNS governance canister are matching.
#[test]
fn check_candid_interface_compatibility() {
    fn source_to_str(source: &candid_parser::utils::CandidSource) -> String {
        match source {
            candid_parser::utils::CandidSource::File(f) => {
                std::fs::read_to_string(f).unwrap_or_else(|_| "".to_string())
            }
            candid_parser::utils::CandidSource::Text(t) => t.to_string(),
        }
    }

    fn check_service_equal(
        new_name: &str,
        new: candid_parser::utils::CandidSource,
        old_name: &str,
        old: candid_parser::utils::CandidSource,
    ) {
        let new_str = source_to_str(&new);
        let old_str = source_to_str(&old);
        match candid_parser::utils::service_equal(new, old) {
            Ok(_) => {}
            Err(e) => {
                eprintln!(
                    "{} is not compatible with {}!\n\n\
            {}:\n\
            {}\n\n\
            {}:\n\
            {}\n",
                    new_name, old_name, new_name, new_str, old_name, old_str
                );
                panic!("{:?}", e);
            }
        }
    }

    candid::export_service!();

    let new_interface = __export_service();

    // check the public interface against the actual one
    let old_interface = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("sns_module.did");

    check_service_equal(
        "actual cycles-manager candid interface",
        candid_parser::utils::CandidSource::Text(&new_interface),
        "declared candid interface in sns_module.did file",
        candid_parser::utils::CandidSource::File(old_interface.as_path()),
    );
}
