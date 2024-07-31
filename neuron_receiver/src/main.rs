use candid::Principal;
use ic_canisters_http_types::{HttpRequest, HttpResponse, HttpResponseBuilder};
use ic_cdk::{init, query};
use icrc_ledger_types::icrc1::account::Account;
use neuron_receiver::state::{replace_state, State, read_state};
use neuron_receiver::{maybe_disburse, update_neurons, Log};
use std::time::Duration;

const SUBACCOUNT: [u8; 32] = [
    210, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0,
];

const PROCESS_DELAY: Duration = Duration::from_secs(60 * 60);

fn main() {}

#[init]
fn init() {
    let water_neuron_id = Principal::from_text("tsbvt-pyaaa-aaaar-qafva-cai").unwrap();

    let state = State {
        neurons: Default::default(),
        disburse_to: Account {
            owner: water_neuron_id,
            subaccount: Some(SUBACCOUNT),
        },
    };
    replace_state(state);

    setup_timer();
}

#[query]
fn get_state() -> State {
    read_state(|s| s.clone())
}

fn setup_timer() {
    ic_cdk_timers::set_timer_interval(PROCESS_DELAY, || {
        ic_cdk::spawn(async move {
            update_neurons().await;
            maybe_disburse().await;
        })
    });
}

#[query(hidden = true)]
fn http_request(_req: HttpRequest) -> HttpResponse {
    let mut log: Log = Log::new();

    log.entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    const MAX_BODY_SIZE: usize = 3_000_000;
    HttpResponseBuilder::ok()
        .header("Content-Type", "application/json; charset=utf-8")
        .with_body_and_content_length(log.serialize_logs(MAX_BODY_SIZE))
        .build()
}

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
        std::path::PathBuf::from(std::env::var("NEURON_RECEIVER_CANDID_PATH").unwrap());

    check_service_equal(
        "actual ledger candid interface",
        candid_parser::utils::CandidSource::Text(&new_interface),
        "declared candid interface in neuron_receiver.did file",
        candid_parser::utils::CandidSource::File(old_interface.as_path()),
    );
}
