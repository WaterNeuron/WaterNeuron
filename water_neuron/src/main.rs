use candid::Nat;
use candid::Principal;
use ic_canister_log::log;
use ic_canisters_http_types::{HttpRequest, HttpResponse, HttpResponseBuilder};
use ic_cdk_macros::{init, post_upgrade, query, update};
use ic_metrics_encoder::MetricsEncoder;
use water_neuron::conversion::{MINIMUM_DEPOSIT_AMOUNT, MINIMUM_WITHDRAWAL_AMOUNT};
use water_neuron::guards::GuardPrincipal;
use water_neuron::logs::INFO;
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
    CanisterInfo, ConversionArg, ConversionError, DepositSuccess, LiquidArg, Unit,
    WithdrawalSuccess,
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
            replace_state(State::from_init_args(arg, ts));
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

            replace_state(replay_events());

            if let Some(args) = upgrade_arg {
                mutate_state(|s| process_event(s, EventType::Upgrade(args)));
            }

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
    schedule_now(TaskType::ScheduleVoting);
    schedule_now(TaskType::MaybeDistributeICP);
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

fn check_postcondition<T>(t: T) -> T {
    #[cfg(feature = "self_check")]
    ok_or_die(check_invariants());
    t
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
fn get_airdrop_allocation() -> WTN {
    reject_anonymous_call();

    read_state(|s| *s.airdrop.get(&ic_cdk::caller()).unwrap_or(&WTN::ZERO))
}

#[update]
async fn claim_airdrop() -> Result<u64, ConversionError> {
    reject_anonymous_call();

    let rewards = read_state(|s| compute_rewards(s.total_icp_deposited, ICP::ONE));
    if rewards != WTN::ZERO {
        ic_cdk::trap("all rewards must be allocated before being claimable");
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

    let wtn_ledger = read_state(|s| s.wtn_ledger_id.expect("WTN ledger not set"));

    match water_neuron::management::transfer(
        caller,
        Nat::from(allocation.0.checked_sub(Unit::WTN.fee()).unwrap()),
        Some(Nat::from(Unit::WTN.fee())),
        None,
        wtn_ledger,
        None,
    )
    .await
    {
        Ok(block_index) => {
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
        neuron_id_6m: s.neuron_id_6m,
        neuron_6m_stake_e8s: s.main_neuron_6m_staked,
        tracked_6m_stake: s.tracked_6m_stake,
        neuron_6m_account: s.get_6m_neuron_account(),
        neuron_id_8y: s.neuron_id_8y,
        neuron_8y_stake_e8s: s.main_neuron_8y_stake,
        neuron_8y_account: s.get_8y_neuron_account(),
        exchange_rate: s.get_icp_to_ncip_exchange_rate_e8s(),
        stakers_count: s.principal_to_deposit.keys().len(),
        total_icp_deposited: s.total_icp_deposited,
        nicp_supply: s.total_circulating_nicp,
        minimum_deposit_amount: MINIMUM_DEPOSIT_AMOUNT,
        minimum_withdraw_amount: MINIMUM_WITHDRAWAL_AMOUNT,
    })
}

#[query]
fn get_withdrawal_requests(maybe_principal: Option<Principal>) -> Vec<WithdrawalDetails> {
    let principal = maybe_principal.unwrap_or(ic_cdk::caller());
    read_state(|s| {
        s.principal_to_withdrawal
            .get(&principal)
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

#[query(hidden = true)]
fn http_request(req: HttpRequest) -> HttpResponse {
    if ic_cdk::api::data_certificate().is_none() {
        ic_cdk::trap("update call rejected");
    }

    if req.path() == "/dashboard" {
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
                    s.principal_to_deposit.keys().len() as f64,
                    "Stakers count",
                )?;
                w.encode_gauge(
                    "neuron_6m_stake",
                    s.main_neuron_6m_staked.0 as f64,
                    "6 months neuron fetched stake",
                )?;
                w.encode_gauge(
                    "neuron_6m_stake",
                    s.tracked_6m_stake.0 as f64,
                    "6 months neuron tracked stake",
                )?;
                w.encode_gauge(
                    "nicp_supply",
                    s.total_circulating_nicp.0 as f64,
                    "nICP total supply.",
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
                w.encode_gauge("transfer_id", s.transfer_id as f64, "Transfer Id.")?;
                w.encode_gauge("withdrawal_id", s.withdrawal_id as f64, "Withdrawal Id.")?;
                w.encode_gauge("guards", s.principal_guards.len() as f64, "Guard.")?;
                w.encode_gauge(
                    "exchange_rate",
                    s.get_icp_to_ncip_exchange_rate_e8s() as f64,
                    "Exchange Rate.",
                )?;

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
