use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

// const END_SWAP_TS: u64 = 1735603211;
// const START_SWAP_TS: u64 = 1734739211;

#[derive(Deserialize, CandidType, Clone)]
pub struct InitArg {
    pub start_ts: u64,
    pub end_ts: u64,
    pub icp_ledger_id: Principal,
    pub wtn_ledger_id: Principal,
}

thread_local! {
    static __STATE: RefCell<Option<State>> = RefCell::default();
}

#[derive(Serialize, Deserialize, Clone)]
pub struct State {
    pub start_ts: u64,
    pub end_ts: u64,
    pub icp_ledger_id: Principal,
    pub wtn_ledger_id: Principal,
}

impl State {
    pub fn new(args: InitArg) -> State {
        State {
            start_ts: args.start_ts,
            end_ts: args.end_ts,
            icp_ledger_id: args.icp_ledger_id,
            wtn_ledger_id: args.wtn_ledger_id,
        }
    }
}

/// Read (part of) the current state using `f`.
///
/// Panics if there is no state.
pub fn read_state<F, R>(f: F) -> R
where
    F: FnOnce(&State) -> R,
{
    __STATE.with(|s| f(s.borrow().as_ref().expect("State not initialized!")))
}

/// Replaces the current state.
pub fn replace_state(state: State) {
    __STATE.with(|s| {
        *s.borrow_mut() = Some(state);
    });
}
