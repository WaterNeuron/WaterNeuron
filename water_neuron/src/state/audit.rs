pub use super::event::{Event, EventType};
use super::State;
use crate::state::SNS_GOVERNANCE_SUBACCOUNT;
use crate::storage::{record_event, with_event_iter};
use crate::{
    nICP, timestamp_nanos, DEFAULT_LEDGER_FEE, ICP, INITIAL_NEURON_STAKE, SNS_DISTRIBUTION_MEMO,
};

/// Updates the state to reflect the given state transition.
/// public because it's used in tests since process_event
/// requires canister infrastructure to retrieve time
pub fn apply_state_transition(state: &mut State, payload: &EventType, timestamp: u64) {
    match payload {
        EventType::Init(_) => {
            panic!("state re-initialization is not allowed");
        }
        EventType::Upgrade(upgrade_arg) => {
            state.record_upgrade(upgrade_arg.clone());
        }
        EventType::DistributeICPtoSNS { amount, receiver } => {
            state.record_icp_pending_transfer(
                SNS_GOVERNANCE_SUBACCOUNT,
                *receiver,
                *amount,
                Some(SNS_DISTRIBUTION_MEMO),
            );
            state.last_distribution_ts = timestamp;
        }
        EventType::DistributeICPtoSNSv2 => {
            state.last_distribution_ts = timestamp;
        }
        EventType::TransferExecuted {
            transfer_id,
            block_index,
        } => state.record_transfer_executed(*transfer_id, *block_index, timestamp),
        EventType::IcpDeposit {
            receiver,
            amount,
            block_index,
        } => {
            state.record_icp_deposit(*receiver, *amount, *block_index);
        }
        EventType::NIcpWithdrawal {
            receiver,
            nicp_burned,
            nicp_burn_index,
        } => {
            state.record_nicp_withdrawal(*receiver, *nicp_burned, *nicp_burn_index, timestamp);
        }
        EventType::DispatchICPRewards {
            nicp_amount,
            sns_gov_amount,
            from_neuron_type,
        } => {
            state.record_icp_pending_transfer(
                from_neuron_type.to_subaccount(),
                state.get_6m_neuron_account(),
                *nicp_amount,
                None,
            );
            state.tracked_6m_stake += nicp_amount
                .checked_sub(ICP::from_e8s(DEFAULT_LEDGER_FEE))
                .unwrap();
            state.record_icp_pending_transfer(
                from_neuron_type.to_subaccount(),
                state.get_sns_account(),
                *sns_gov_amount,
                None,
            );
        }
        EventType::SplitNeuron {
            withdrawal_id,
            neuron_id,
        } => state.record_neuron_split(*withdrawal_id, *neuron_id),
        EventType::MergeNeuron { neuron_id } => state.record_neuron_merge(*neuron_id),
        EventType::StartedToDissolve { withdrawal_id } => {
            state.record_started_to_dissolve_neuron(*withdrawal_id)
        }
        EventType::DisbursedUserNeuron {
            withdrawal_id,
            transfer_block_height,
        } => state.record_neuron_disbursed(*withdrawal_id, *transfer_block_height),
        EventType::MaturityNeuron {
            neuron_id,
            from_neuron_type,
        } => state.record_maturity_neuron(*neuron_id, *from_neuron_type),
        EventType::DisbursedMaturityNeuron {
            neuron_id,
            transfer_block_height,
        } => state.record_disbursed_maturity_neuron(*neuron_id, *transfer_block_height),
        EventType::NeuronSixMonths(neuron_id) => {
            state.record_6m_neuron_id(*neuron_id);
            state.tracked_6m_stake += ICP::from_e8s(INITIAL_NEURON_STAKE);
            state.total_circulating_nicp += nICP::from_e8s(INITIAL_NEURON_STAKE);
        }
        EventType::NeuronEightYears(neuron_id) => {
            state.record_8y_neuron_id(*neuron_id);
        }
        EventType::ClaimedAirdrop {
            caller,
            block_index: _,
        } => {
            state.record_claimed_airdrop(*caller);
        }
        EventType::MirroredProposal {
            nns_proposal_id,
            sns_proposal_id,
        } => {
            state
                .proposals
                .insert(nns_proposal_id.clone(), sns_proposal_id.clone());

            if *nns_proposal_id > state.last_nns_proposal_processed {
                state.last_nns_proposal_processed = nns_proposal_id.clone();
            }
        }
    }
}

/// Records the given event payload in the event log and updates the state to reflect the change.
pub fn process_event(state: &mut State, payload: EventType) {
    let timestamp = timestamp_nanos();
    apply_state_transition(state, &payload, timestamp);
    record_event(payload, timestamp);
}

/// Recomputes the minter state from the event log.
///
/// # Panics
///
/// This function panics if:
///   * The event log is empty.
///   * The first event in the log is not an Init event.
///   * One of the events in the log invalidates the minter's state invariants.
pub fn replay_events() -> State {
    with_event_iter(|mut iter| {
        let mut state = match iter.next().expect("the event log should not be empty") {
            Event {
                payload: EventType::Init(init_arg),
                timestamp: _,
            } => State::from_init_args(init_arg),
            other => panic!("the first event must be an Init event, got: {other:?}"),
        };
        for event in iter {
            apply_state_transition(&mut state, &event.payload, event.timestamp);
        }
        state
    })
}
