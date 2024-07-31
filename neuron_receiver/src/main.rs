use candid::Principal;
use ic_canisters_http_types::{HttpRequest, HttpResponse, HttpResponseBuilder};
use ic_cdk::{init, query};
use icrc_ledger_types::icrc1::account::Account;
use neuron_receiver::state::{replace_state, State};
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
