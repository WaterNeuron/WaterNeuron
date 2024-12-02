use candid::{Nat, Principal};
use ic_base_types::PrincipalId;
use ic_cdk::update;
use ic_nervous_system_common::ledger::compute_neuron_staking_subaccount;
use icp_ledger::{AccountIdentifier, Subaccount};
use icrc_ledger_types::icrc1::transfer::TransferError;
use sns_module::{derive_staking, transfer};

fn main() {}

#[update]
fn get_icp_deposit_address(target: Principal) -> AccountIdentifier {
    let subaccount = Subaccount(derive_staking(target));
    let self_id: PrincipalId = ic_cdk::id().into();
    AccountIdentifier::new(self_id, Some(subaccount))
}

#[update]
async fn notify_icp_deposit(target: Principal, amount: u64) -> Result<u64, TransferError> {
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
        Ok(block_index) => Ok(block_index),
        Err(e) => Err(e),
    }
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
    let old_interface =
        std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("sns2.did");

    check_service_equal(
        "actual cycles-manager candid interface",
        candid_parser::utils::CandidSource::Text(&new_interface),
        "declared candid interface in sns2.did file",
        candid_parser::utils::CandidSource::File(old_interface.as_path()),
    );
}
