use ic_nns_governance::pb::v1::Neuron;
use icrc_ledger_types::icrc1::account::Account;
use std::cell::RefCell;
use std::collections::BTreeMap;

thread_local! {
    static __STATE: RefCell<Option<State>> = RefCell::default();
}

pub struct State {
    pub neurons: BTreeMap<u64, Neuron>,
    pub disburse_to: Account,
}

impl State {
    pub fn insert_or_update_neuron(&mut self, key: u64, neuron: Neuron) {
        self.neurons
            .entry(key)
            .and_modify(|e| *e = neuron.clone())
            .or_insert(neuron);
    }

    pub fn reset_neuron_map(&mut self) {
        self.neurons = Default::default();
    }
}

/// Mutates (part of) the current state using `f`.
///
/// Panics if there is no state.
pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut State) -> R,
{
    __STATE.with(|s| f(s.borrow_mut().as_mut().expect("State not initialized!")))
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
