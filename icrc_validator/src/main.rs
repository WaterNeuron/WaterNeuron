use candid::Principal;
use ic_cdk::update;
use icrc_ledger_client_cdk::{CdkRuntime, ICRC1Client};
use icrc_ledger_types::icrc1::transfer::TransferArg;

fn main() {}

#[update]
async fn validate_nicp_icrc1_transfer(target: TransferArg) -> Result<String, String> {
    let nicp_ledger = Principal::from_text("buwm7-7yaaa-aaaar-qagva-cai").unwrap();
    let nicp_client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id: nicp_ledger,
    };
    match nicp_client.balance_of(ic_cdk::caller().into()).await {
        Ok(balance) => {
            if balance >= target.amount {
                return Ok("".to_string());
            }
            Err(format!(
                "cannot transfer, balance {balance} less than required amount {}",
                target.amount
            ))
        }
        Err((i, s)) => Err(format!("{i} - {s}")),
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
    let old_interface = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("icrc_validator.did");

    check_service_equal(
        "actual cycles-manager candid interface",
        candid_parser::utils::CandidSource::Text(&new_interface),
        "declared candid interface in icrc_validator.did file",
        candid_parser::utils::CandidSource::File(old_interface.as_path()),
    );
}
