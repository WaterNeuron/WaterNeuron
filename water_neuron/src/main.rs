use candid::{Nat, Principal};
use ic_canister_log::log;
use ic_cdk_macros::{init, post_upgrade, query, update};
use ic_http_types::{HttpRequest, HttpResponse, HttpResponseBuilder};
use ic_metrics_encoder::MetricsEncoder;
use ic_nns_governance_api::{
    manage_neuron_response::MergeResponse, GovernanceError, ManageNeuronResponse, Neuron,
};
use icrc_ledger_types::icrc1::account::Account;
use water_neuron::conversion::{MINIMUM_DEPOSIT_AMOUNT, MINIMUM_WITHDRAWAL_AMOUNT};
use water_neuron::dashboard::DisplayAmount;
use water_neuron::guards::GuardPrincipal;
use water_neuron::icrc21::{ConsentInfo, ConsentMessageRequest, Icrc21Error, StandardRecord};
use water_neuron::logs::INFO;
use water_neuron::management::register_vote;
use water_neuron::nns_types::{NeuronId, ProposalId};
use water_neuron::numeric::{ICP, WTN};
use water_neuron::sns_distribution::compute_rewards;
use water_neuron::state::audit::{process_event, replay_events};
use water_neuron::state::event::{EventType, GetEventsArg, GetEventsResult};
use water_neuron::state::{
    mutate_state, read_state, replace_state, State, TransferStatus, WithdrawalDetails,
};
use water_neuron::storage::total_event_count;
use water_neuron::tasks::{schedule_now, TaskType};
use water_neuron::{
    CancelWithdrawalError, CanisterInfo, ConversionArg, ConversionError, DepositSuccess, LiquidArg,
    Unit, UpgradeArg, WithdrawalSuccess,
};

fn reject_anonymous_call() {
    if ic_cdk::caller() == Principal::anonymous() {
        ic_cdk::trap("call rejected: anonymous caller");
    }
}

#[init]
fn init(args: LiquidArg) {
    let ts = water_neuron::timestamp_nanos();
    match args {
        LiquidArg::Init(arg) => {
            water_neuron::storage::record_event(EventType::Init(arg.clone()), ts);
            replace_state(State::from_init_args(arg));
        }
        LiquidArg::Upgrade(_) => ic_cdk::trap("expected init args, got upgrade"),
    }
    setup_timer();
}

#[post_upgrade]
pub fn post_upgrade(args: LiquidArg) {
    match args {
        LiquidArg::Init(_) => ic_cdk::trap(""),
        LiquidArg::Upgrade(upgrade_arg) => {
            let start = ic_cdk::api::instruction_counter();

            fn validate_upgrade_args(args: UpgradeArg) -> Result<(), String> {
                if let Some(governance_fee_share_percent) = args.governance_fee_share_percent {
                    if governance_fee_share_percent > 100 {
                        return Err(
                            "governance_fee_share_percent has to be between 0 and 100".to_string()
                        );
                    }
                }
                Ok(())
            }

            replace_state(replay_events());

            if let Some(args) = upgrade_arg {
                if let Err(e) = validate_upgrade_args(args.clone()) {
                    ic_cdk::trap(&e);
                }
                mutate_state(|s| process_event(s, EventType::Upgrade(args)));
            }

            mutate_state(|s| {
                if let Some(entry) = s.proposals.last_entry() {
                    s.last_nns_proposal_processed = entry.key().clone();
                }
            });

            let end = ic_cdk::api::instruction_counter();

            let event_count = total_event_count();
            let instructions_consumed = end - start;

            log!(
                INFO,
                "[upgrade]: replaying {event_count} events consumed {instructions_consumed} instructions ({} instructions per event on average)",
                instructions_consumed / event_count
            );
            setup_timer();
        }
    }
}

fn setup_timer() {
    schedule_now(TaskType::MaybeInitializeMainNeurons);
    schedule_now(TaskType::ProcessLogic);
    schedule_now(TaskType::SpawnNeurons);
    schedule_now(TaskType::ProcessVoting);
    schedule_now(TaskType::ProcessEarlyVoting);
    schedule_now(TaskType::MaybeDistributeICP);
    schedule_now(TaskType::MaybeDistributeRewards);
    schedule_now(TaskType::ProcessPendingTransfers);
}

#[cfg(feature = "self_check")]
fn ok_or_die(result: Result<(), String>) {
    if let Err(msg) = result {
        ic_cdk::println!("{}", msg);
        ic_cdk::trap(&msg);
    }
}

#[cfg(feature = "self_check")]
fn check_invariants() -> Result<(), String> {
    read_state(|s| {
        let recovered_state = replay_events();

        recovered_state.is_equivalent_to(s)?;

        Ok(())
    })
}

#[cfg(feature = "self_check")]
#[query]
pub fn self_check() {}

fn check_postcondition<T>(t: T) -> T {
    #[cfg(feature = "self_check")]
    ok_or_die(check_invariants());
    t
}

#[export_name = "canister_global_timer"]
fn timer() {
    #[cfg(feature = "self_check")]
    ok_or_die(check_invariants());

    water_neuron::timer();
}

#[query]
fn get_events(args: GetEventsArg) -> GetEventsResult {
    const MAX_EVENTS_PER_QUERY: u64 = 2_000;
    let events = water_neuron::storage::with_event_iter(|it| {
        it.skip(args.start as usize)
            .take(args.length.min(MAX_EVENTS_PER_QUERY) as usize)
            .collect()
    });
    GetEventsResult {
        events,
        total_event_count: water_neuron::storage::total_event_count(),
    }
}

#[query]
fn get_airdrop_allocation(p: Option<Principal>) -> WTN {
    read_state(|s| {
        *s.airdrop
            .get(&p.unwrap_or(ic_cdk::caller()))
            .unwrap_or(&WTN::ZERO)
    })
}

#[query]
fn get_pending_rewards(p: Option<Principal>) -> u64 {
    water_neuron::storage::get_pending_rewards(p.unwrap_or(ic_cdk::caller())).unwrap_or(0)
}

#[query]
fn get_wtn_proposal_id(nns_proposal_id: u64) -> Result<ProposalId, ProposalId> {
    read_state(|s| {
        s.proposals
            .get(&ProposalId {
                id: nns_proposal_id,
            })
            .cloned()
            .ok_or_else(|| s.last_nns_proposal_processed.clone())
    })
}

#[update(hidden = true)]
async fn get_full_neuron(neuron_id: u64) -> Result<Result<Neuron, GovernanceError>, String> {
    assert_eq!(
        ic_cdk::caller(),
        Principal::from_text("bo5bf-eaaaa-aaaam-abtza-cai").unwrap()
    );

    water_neuron::management::get_full_neuron(neuron_id).await
}

#[update(hidden = true)]
async fn schedule_task(task: TaskType) {
    assert_eq!(
        ic_cdk::caller(),
        Principal::from_text("bo5bf-eaaaa-aaaam-abtza-cai").unwrap()
    );

    schedule_now(task);
}

#[update(hidden = true)]
async fn approve_proposal(id: u64) -> Result<ManageNeuronResponse, String> {
    assert_eq!(ic_cdk::caller(), read_state(|s| s.wtn_governance_id));

    let neuron_6m = match read_state(|s| s.neuron_id_6m) {
        Some(neuron_6m_id) => neuron_6m_id,
        None => return Err("6 months neuron not set".to_string()),
    };

    match register_vote(neuron_6m, ProposalId { id }, true).await {
        Ok(response) => {
            log!(
                INFO,
                "[approve_proposal] Successfully approved proposal {id} with response {response:?}"
            );
            Ok(response)
        }

        Err(error) => {
            log!(
                INFO,
                "[approve_proposal] Failed to approve proposal {id} with error: {error}"
            );
            Err(error)
        }
    }
}

#[update(hidden = true)]
async fn approve_proposal_validate(id: u64) -> Result<String, String> {
    assert_eq!(ic_cdk::caller(), read_state(|s| s.wtn_governance_id));

    Ok(format!("{id}"))
}

#[update]
async fn claim_airdrop() -> Result<u64, ConversionError> {
    reject_anonymous_call();

    let rewards = read_state(|s| compute_rewards(s.total_icp_deposited, ICP::ONE));
    if rewards != WTN::ZERO {
        ic_cdk::trap("all rewards must be allocated before being claimable");
    }

    if read_state(|s| s.tracked_6m_stake) < ICP::from_unscaled(21_000_000) {
        ic_cdk::trap("21M ICP must be staked to unlock the airdrop");
    }

    let caller = ic_cdk::caller();
    let _guard_principal = GuardPrincipal::new(caller)
        .map_err(|guard_error| ConversionError::GuardError { guard_error })?;

    let allocation = read_state(|s| *s.airdrop.get(&caller).unwrap_or(&WTN::ZERO));

    let fee_e8s = WTN::from_e8s(Unit::WTN.fee());

    if allocation < fee_e8s {
        return Err(ConversionError::AmountTooLow {
            minimum_amount_e8s: fee_e8s.0,
        });
    }

    let wtn_ledger = read_state(|s| s.wtn_ledger_id);

    let allocation_minus_fee = allocation.0.checked_sub(Unit::WTN.fee()).unwrap();

    match water_neuron::management::transfer(
        caller,
        Nat::from(allocation_minus_fee),
        Some(Nat::from(Unit::WTN.fee())),
        None,
        wtn_ledger,
        None,
    )
    .await
    {
        Ok(block_index) => {
            log!(
                INFO,
                "[claim_airdrop] {caller} claimed {} WTN at block {block_index}",
                DisplayAmount(allocation_minus_fee)
            );
            mutate_state(|s| {
                process_event(
                    s,
                    EventType::ClaimedAirdrop {
                        caller,
                        block_index,
                    },
                );
            });
            Ok(block_index)
        }
        Err(e) => Err(ConversionError::TransferError(e)),
    }
}

#[query]
fn get_info() -> CanisterInfo {
    read_state(|s| CanisterInfo {
        latest_distribution_icp_per_vp: s.latest_distribution_icp_per_vp,
        neuron_id_6m: s.neuron_id_6m,
        neuron_6m_stake_e8s: s.main_neuron_6m_staked,
        tracked_6m_stake: s.tracked_6m_stake,
        neuron_6m_account: s.get_6m_neuron_account(),
        neuron_id_8y: s.neuron_id_8y,
        neuron_8y_stake_e8s: s.main_neuron_8y_stake,
        neuron_8y_account: s.get_8y_neuron_account(),
        exchange_rate: s.get_icp_to_ncip_exchange_rate_e8s(),
        stakers_count: s.account_to_deposits.keys().len(),
        total_icp_deposited: s.total_icp_deposited,
        nicp_supply: s.total_circulating_nicp,
        minimum_deposit_amount: MINIMUM_DEPOSIT_AMOUNT,
        minimum_withdraw_amount: MINIMUM_WITHDRAWAL_AMOUNT,
        governance_fee_share_percent: s.governance_fee_share_percent,
    })
}

#[query]
fn get_withdrawal_requests(maybe_account: Option<Account>) -> Vec<WithdrawalDetails> {
    let account = maybe_account.unwrap_or(ic_cdk::caller().into());
    read_state(|s| {
        s.account_to_withdrawals
            .get(&account)
            .cloned()
            .unwrap_or(vec![])
            .iter()
            .map(|id| WithdrawalDetails {
                status: s.get_withdrawal_status(*id),
                request: s.withdrawal_id_to_request.get(id).unwrap().clone(),
            })
            .collect::<Vec<WithdrawalDetails>>()
    })
}

#[query]
fn get_transfer_statuses(ids: Vec<u64>) -> Vec<TransferStatus> {
    read_state(|s| ids.iter().map(|id| s.get_transfer_status(*id)).collect())
}

#[update]
async fn nicp_to_icp(arg: ConversionArg) -> Result<WithdrawalSuccess, ConversionError> {
    reject_anonymous_call();
    check_postcondition(water_neuron::conversion::nicp_to_icp(arg).await)
}

#[update]
async fn icp_to_nicp(arg: ConversionArg) -> Result<DepositSuccess, ConversionError> {
    reject_anonymous_call();
    check_postcondition(water_neuron::conversion::icp_to_nicp(arg).await)
}

#[update]
async fn cancel_withdrawal(neuron_id: NeuronId) -> Result<MergeResponse, CancelWithdrawalError> {
    reject_anonymous_call();
    check_postcondition(water_neuron::conversion::cancel_withdrawal(neuron_id).await)
}

#[query]
fn icrc10_supported_standards() -> Vec<StandardRecord> {
    water_neuron::icrc21::icrc10_supported_standards()
}

#[update]
fn icrc21_canister_call_consent_message(
    request: ConsentMessageRequest,
) -> Result<ConsentInfo, Icrc21Error> {
    water_neuron::icrc21::icrc21_canister_call_consent_message(request)
}

#[query(hidden = true)]
fn http_request(req: HttpRequest) -> HttpResponse {
    if req.path() == "/dashboard" {
        if ic_cdk::api::data_certificate().is_none() {
            ic_cdk::trap("update call rejected");
        };
        use water_neuron::dashboard::build_dashboard;

        let dashboard = build_dashboard();
        return HttpResponseBuilder::ok()
            .header("Content-Type", "text/html; charset=utf-8")
            .with_body_and_content_length(dashboard)
            .build();
    } else if req.path() == "/metrics" {
        let mut writer = MetricsEncoder::new(vec![], ic_cdk::api::time() as i64 / 1_000_000);

        fn encode_metrics(w: &mut MetricsEncoder<Vec<u8>>) -> std::io::Result<()> {
            const WASM_PAGE_SIZE_IN_BYTES: f64 = 65536.0;

            read_state(|s| {
                w.encode_gauge(
                    "cycle_balance",
                    ic_cdk::api::canister_balance128() as f64,
                    "Cycle balance.",
                )?;
                w.encode_gauge(
                    "stable_memory_bytes",
                    ic_cdk::api::stable::stable_size() as f64 * WASM_PAGE_SIZE_IN_BYTES,
                    "Size of the stable memory allocated by this canister.",
                )?;
                w.encode_gauge(
                    "stakers",
                    s.account_to_deposits.keys().len() as f64,
                    "Stakers count",
                )?;
                w.encode_gauge(
                    "neuron_6m_fetched_stake",
                    s.main_neuron_6m_staked.0 as f64,
                    "6 months neuron fetched stake",
                )?;
                w.encode_gauge(
                    "neuron_6m_tracked_stake",
                    s.tracked_6m_stake.0 as f64,
                    "6 months neuron tracked stake",
                )?;
                w.encode_gauge(
                    "neuron_8y_stake",
                    s.main_neuron_8y_stake.0 as f64,
                    "8 years neuron stake",
                )?;
                w.encode_gauge(
                    "total_circulating_nicp",
                    s.total_circulating_nicp.0 as f64,
                    "Total nICP minted.",
                )?;
                w.encode_gauge(
                    "total_icp_deposited",
                    s.total_icp_deposited.0 as f64,
                    "Total ICP deposited.",
                )?;
                w.encode_gauge(
                    "revenue",
                    s.compute_daily_revenue() as f64,
                    "The maturity redistributed to the DAO.",
                )?;
                w.encode_gauge(
                    "fees",
                    s.compute_daily_fees() as f64,
                    "The maturity generated by both neurons.",
                )?;
                w.encode_gauge("apy", s.compute_nicp_apy(), "The APY of the protocol.")?;
                w.encode_gauge(
                    "next_transfer_id",
                    s.transfer_id as f64,
                    "Next transfer Id.",
                )?;
                w.encode_gauge(
                    "next_withdrawal_id",
                    s.withdrawal_id as f64,
                    "Next withdrawal Id.",
                )?;
                w.encode_gauge("guards", s.principal_guards.len() as f64, "Guard.")?;
                w.encode_gauge(
                    "exchange_rate",
                    s.get_icp_to_ncip_exchange_rate_e8s() as f64,
                    "Exchange Rate.",
                )?;
                w.encode_gauge(
                    "nns_proposals_mirrored_count",
                    s.proposals.len() as f64,
                    "Count of NNS proposals mirrored.",
                )?;
                w.encode_gauge(
                    "airdrop_participants_count",
                    s.airdrop.len() as f64,
                    "Count of airdrop participants.",
                )?;
                w.encode_gauge(
                    "airdrop_wtn_allocated",
                    s.airdrop.values().map(|v| v.0).sum::<u64>() as f64,
                    "Amount of WTN allocated.",
                )?;
                w.encode_gauge(
                    "in_flight_neurons",
                    (s.withdrawal_to_start_dissolving.len() + s.withdrawal_to_disburse.len())
                        as f64,
                    "Count of in-flight neurons.",
                )?;
                w.encode_gauge(
                    "finalized_withdrawals",
                    s.withdrawal_finalized.len() as f64,
                    "Count of finalized withdrawals requests.",
                )?;
                if let Some(latest_distribution_icp_per_vp) = s.latest_distribution_icp_per_vp {
                    w.encode_gauge(
                        "latest_distribution_icp_per_vp",
                        latest_distribution_icp_per_vp,
                        "ICP per voting power in the latest distribution to SNS neurons.",
                    )?;
                }

                Ok(())
            })
        }

        match encode_metrics(&mut writer) {
            Ok(()) => {
                return HttpResponseBuilder::ok()
                    .header("Content-Type", "text/plain; version=0.0.4")
                    .with_body_and_content_length(writer.into_inner())
                    .build()
            }
            Err(err) => {
                return HttpResponseBuilder::server_error(format!(
                    "Failed to encode metrics: {}",
                    err
                ))
                .build();
            }
        }
    } else if req.path() == "/api/metadata" {
        use serde_json;

        let bytes: Vec<u8> = serde_json::to_string(&read_state(|s| CanisterInfo {
            latest_distribution_icp_per_vp: s.latest_distribution_icp_per_vp,
            neuron_id_6m: s.neuron_id_6m,
            neuron_6m_stake_e8s: s.main_neuron_6m_staked,
            tracked_6m_stake: s.tracked_6m_stake,
            neuron_6m_account: s.get_6m_neuron_account(),
            neuron_id_8y: s.neuron_id_8y,
            neuron_8y_stake_e8s: s.main_neuron_8y_stake,
            neuron_8y_account: s.get_8y_neuron_account(),
            exchange_rate: s.get_icp_to_ncip_exchange_rate_e8s(),
            stakers_count: s.account_to_deposits.keys().len(),
            total_icp_deposited: s.total_icp_deposited,
            nicp_supply: s.total_circulating_nicp,
            minimum_deposit_amount: MINIMUM_DEPOSIT_AMOUNT,
            minimum_withdraw_amount: MINIMUM_WITHDRAWAL_AMOUNT,
            governance_fee_share_percent: s.governance_fee_share_percent,
        }))
        .unwrap_or_default()
        .into_bytes();
        return HttpResponseBuilder::ok()
            .header("Content-Type", "application/json; charset=utf-8")
            .with_body_and_content_length(bytes)
            .build();
    }

    use std::str::FromStr;
    use water_neuron::logs::{Log, Priority, Sort};

    let max_skip_timestamp = match req.raw_query_param("time") {
        Some(arg) => match u64::from_str(arg) {
            Ok(value) => value,
            Err(_) => {
                return HttpResponseBuilder::bad_request()
                    .with_body_and_content_length("failed to parse the 'time' parameter")
                    .build();
            }
        },
        None => 0,
    };

    let mut log: Log = Default::default();

    match req.raw_query_param("priority") {
        Some(priority_str) => match Priority::from_str(priority_str) {
            Ok(priority) => match priority {
                Priority::Info => log.push_logs(Priority::Info),
                Priority::Debug => log.push_logs(Priority::Debug),
            },
            Err(_) => log.push_all(),
        },
        None => log.push_all(),
    }

    log.entries
        .retain(|entry| entry.timestamp >= max_skip_timestamp);

    fn ordering_from_query_params(sort: Option<&str>, max_skip_timestamp: u64) -> Sort {
        match sort {
            Some(ord_str) => match Sort::from_str(ord_str) {
                Ok(order) => order,
                Err(_) => {
                    if max_skip_timestamp == 0 {
                        Sort::Ascending
                    } else {
                        Sort::Descending
                    }
                }
            },
            None => {
                if max_skip_timestamp == 0 {
                    Sort::Ascending
                } else {
                    Sort::Descending
                }
            }
        }
    }

    log.sort_logs(ordering_from_query_params(
        req.raw_query_param("sort"),
        max_skip_timestamp,
    ));

    const MAX_BODY_SIZE: usize = 3_000_000;
    HttpResponseBuilder::ok()
        .header("Content-Type", "application/json; charset=utf-8")
        .with_body_and_content_length(log.serialize_logs(MAX_BODY_SIZE))
        .build()
}

fn main() {}

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
        .join("water_neuron.did");

    check_service_equal(
        "actual ledger candid interface",
        candid_parser::utils::CandidSource::Text(&new_interface),
        "declared candid interface in water_neuron.did file",
        candid_parser::utils::CandidSource::File(old_interface.as_path()),
    );
}
