use candid::{Nat, Principal};
use ic_base_types::PrincipalId;
use ic_cdk::{init, post_upgrade, query, update};
use icp_ledger::{AccountIdentifier, Subaccount};
use scopeguard::guard;
use sns_module::memory::{
    add_in_flight_wtn, add_wtn_owed, decrease_wtn_owed, deposit_icp, get_in_flight_wtn,
    get_principal_to_icp, get_principal_to_wtn_owed, remove_in_flight_wtn,
};
use sns_module::state::{mutate_state, read_state, replace_state, InitArg, State};
use sns_module::{
    balance_of, derive_staking, dispatch_tokens, is_distribution_available, is_swap_available,
    transfer, Status, E8S, MIN_DEPOSIT_AMOUNT, NANOS,
};

fn main() {}

#[init]
fn init(args: InitArg) {
    sns_module::memory::set_state(State::new(args.clone()));
    let state = sns_module::memory::get_state().unwrap();
    replace_state(state);
}

#[post_upgrade]
fn post_upgrade() {
    let state = sns_module::memory::get_state().unwrap();
    replace_state(state);
}

#[query]
fn get_state() -> State {
    read_state(|s| s.clone())
}

#[query]
fn get_status() -> Status {
    let balances = get_principal_to_icp();
    let now = ic_cdk::api::time() / NANOS;
    read_state(|s| {
        let time_left = now
            .checked_sub(s.start_ts)
            .map(|_| s.end_ts.saturating_sub(now));
        Status {
            participants: balances.len(),
            total_icp_deposited: balances.iter().map(|(_, b)| b).sum(),
            time_left,
            start_at: s.start_ts,
            end_at: s.end_ts,
            minimum_deposit_amount: MIN_DEPOSIT_AMOUNT,
        }
    })
}

#[query]
fn get_principal_to_icp_deposited() -> Vec<(Principal, u64)> {
    sns_module::memory::get_principal_to_icp()
}

#[query]
fn get_icp_deposited(of: Principal) -> u64 {
    sns_module::memory::get_icp_deposited(of)
}

#[query]
fn get_principal_to_wtn_allocation() -> Vec<(Principal, u64)> {
    get_principal_to_wtn_owed()
}

#[query]
fn get_wtn_allocated(of: Principal) -> u64 {
    sns_module::memory::get_wtn_owed(of)
}

#[update]
fn get_icp_deposit_address(target: Principal) -> AccountIdentifier {
    let subaccount = Subaccount(derive_staking(target));
    let self_id: PrincipalId = ic_cdk::id().into();
    AccountIdentifier::new(self_id, Some(subaccount))
}

#[update]
async fn notify_icp_deposit(target: Principal, amount: u64) -> Result<u64, String> {
    if amount < MIN_DEPOSIT_AMOUNT {
        return Err(format!(
            "Amount lower than the minimum deposit: {} ICP",
            MIN_DEPOSIT_AMOUNT / E8S
        ));
    }
    is_swap_available()?;
    let icp_ledger = read_state(|s| s.icp_ledger_id);
    let received_tokens = amount.checked_sub(10_000).unwrap();
    match transfer(
        Some(derive_staking(target)),
        ic_cdk::id(),
        Nat::from(received_tokens),
        None,
        icp_ledger,
    )
    .await
    {
        Ok(block_index) => {
            deposit_icp(target, received_tokens);
            Ok(block_index)
        }
        Err(e) => Err(format!("{e}")),
    }
}

#[update]
async fn distribute_tokens() -> Result<u64, String> {
    assert_eq!(
        ic_cdk::caller(),
        Principal::from_text("bo5bf-eaaaa-aaaam-abtza-cai").unwrap()
    );
    if !is_distribution_available() {
        return Err("Distribution not available".to_string());
    }
    if read_state(|s| s.is_distributing) {
        return Err("Already distributing".to_string());
    }

    mutate_state(|s| s.is_distributing = true);
    let _enqueue_followup_guard = guard((), |_| {
        mutate_state(|s| s.is_distributing = false);
    });

    let wtn_ledger = read_state(|s| s.wtn_ledger_id);
    let wtn_balance = balance_of(ic_cdk::id(), wtn_ledger).await?;

    let total_wtn_allocated = sns_module::memory::total_wtn_allocated();
    let wtn_to_allocate = wtn_balance
        .checked_sub(total_wtn_allocated)
        .unwrap()
        .checked_sub(get_in_flight_wtn())
        .unwrap();
    if wtn_to_allocate < E8S {
        return Err("Nothing to distribute".to_string());
    }

    let balances = sns_module::memory::get_principal_to_icp();

    let wtn_map = dispatch_tokens(wtn_to_allocate, balances);

    for (owner, owed_wtn) in wtn_map {
        add_wtn_owed(owner, owed_wtn);
    }

    Ok(wtn_to_allocate)
}

#[update]
async fn claim_wtn(of: Principal) -> Result<u64, String> {
    let wtn_amount = sns_module::memory::get_wtn_owed(of);
    if wtn_amount < E8S {
        return Err("Minimum claim amount of 1 WTN".to_string());
    }
    if get_principal_to_wtn_owed().is_empty() {
        return Err("WTN not allocated yet".to_string());
    }
    if !is_distribution_available() {
        return Err("swap not over".to_string());
    }

    let ledger_canister_id = read_state(|s| s.wtn_ledger_id);
    decrease_wtn_owed(of, wtn_amount);
    add_in_flight_wtn(wtn_amount);
    let wtn_amount_minus_fee = wtn_amount.checked_sub(1_000_000).unwrap();
    match transfer(
        None,
        of,
        Nat::from(wtn_amount_minus_fee),
        None,
        ledger_canister_id,
    )
    .await
    {
        Ok(block_index) => {
            remove_in_flight_wtn(wtn_amount);
            Ok(block_index)
        }
        Err(e) => {
            add_wtn_owed(of, wtn_amount);
            remove_in_flight_wtn(wtn_amount);
            Err(format!("{e}"))
        }
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
        .join("sns_module.did");

    check_service_equal(
        "actual cycles-manager candid interface",
        candid_parser::utils::CandidSource::Text(&new_interface),
        "declared candid interface in sns_module.did file",
        candid_parser::utils::CandidSource::File(old_interface.as_path()),
    );
}
