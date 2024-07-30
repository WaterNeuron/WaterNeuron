use boomerang::{
    derive_subaccount_staking, derive_subaccount_unstaking, self_canister_id, BoomerangError,
    DepositSuccess, WithdrawalSuccess,
};
use candid::{Nat, Principal};
use ic_base_types::PrincipalId;
use ic_cdk::{query, update};
use icp_ledger::AccountIdentifier;
use icrc_ledger_types::icrc1::account::Account;

fn main() {}

#[query]
fn get_staking_account_id(principal: Principal) -> AccountIdentifier {
    let boomerang_id = self_canister_id();
    let subaccount = derive_subaccount_staking(principal);
    AccountIdentifier::new(
        PrincipalId::from(boomerang_id),
        Some(icp_ledger::Subaccount(subaccount)),
    )
}

#[update]
async fn retrieve_nicp(target: Principal) -> Result<Nat, BoomerangError> {
    boomerang::icp_to_nicp::retrieve_nicp(target).await
}

#[update]
async fn notify_icp_deposit(client_id: Principal) -> Result<DepositSuccess, BoomerangError> {
    boomerang::icp_to_nicp::notify_icp_deposit(client_id).await
}

#[query]
fn get_unstaking_account(target: Principal) -> Account {
    Account {
        owner: self_canister_id(),
        subaccount: Some(derive_subaccount_unstaking(target)),
    }
}

#[update]
async fn notify_nicp_deposit(target: Principal) -> Result<WithdrawalSuccess, BoomerangError> {
    boomerang::nicp_to_icp::notify_nicp_deposit(target).await
}

#[update]
async fn try_retrieve_icp(target: Principal) -> Result<Nat, BoomerangError> {
    boomerang::nicp_to_icp::try_retrieve_icp(target).await
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
        .join("boomerang.did");

    check_service_equal(
        "actual cycles-manager candid interface",
        candid_parser::utils::CandidSource::Text(&new_interface),
        "declared candid interface in boomerang.did file",
        candid_parser::utils::CandidSource::File(old_interface.as_path()),
    );
}
