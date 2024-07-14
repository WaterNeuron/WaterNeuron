use crate::numeric::{nICP, ICP};
use crate::state::{NeuronOrigin, WithdrawalId};
use crate::{InitArg, NeuronId, Principal, ProposalId, TransferId, UpgradeArg};
use candid::CandidType;
use icrc_ledger_types::icrc1::account::Account;
use minicbor_derive::{Decode, Encode};
use serde::Deserialize;

/// The event describing a state transition.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, CandidType, Deserialize)]
pub enum EventType {
    #[n(0)]
    Init(#[n(0)] InitArg),

    #[n(1)]
    Upgrade(#[n(0)] UpgradeArg),

    #[n(2)]
    DistributeICPtoSNS {
        #[n(0)]
        amount: ICP,
        #[cbor(n(1), with = "crate::cbor::principal")]
        receiver: Principal,
    },

    #[n(3)]
    TransferExecuted {
        #[n(0)]
        transfer_id: TransferId,
        #[n(1)]
        block_index: Option<u64>,
    },

    #[n(4)]
    IcpDeposit {
        #[cbor(n(0), with = "crate::cbor::account")]
        receiver: Account,
        #[n(1)]
        amount: ICP,
        #[n(2)]
        block_index: u64,
    },

    #[n(5)]
    NIcpWithdrawal {
        #[cbor(n(0), with = "crate::cbor::account")]
        receiver: Account,
        #[n(1)]
        nicp_burned: nICP,
        #[n(2)]
        nicp_burn_index: u64,
    },

    #[n(6)]
    DispatchICPRewards {
        #[n(0)]
        nicp_amount: ICP,
        #[n(1)]
        sns_gov_amount: ICP,
        #[n(2)]
        from_neuron_type: NeuronOrigin,
    },

    #[n(7)]
    SplitNeuron {
        #[n(0)]
        withdrawal_id: u64,
        #[n(1)]
        neuron_id: NeuronId,
    },

    #[n(8)]
    StartedToDissolve {
        #[n(0)]
        withdrawal_id: WithdrawalId,
    },

    #[n(9)]
    DisbursedUserNeuron {
        #[n(0)]
        withdrawal_id: WithdrawalId,
        #[n(1)]
        transfer_block_height: u64,
    },

    #[n(10)]
    MaturityNeuron {
        #[n(0)]
        neuron_id: NeuronId,
        #[n(1)]
        from_neuron_type: NeuronOrigin,
    },

    #[n(11)]
    DisbursedMaturityNeuron {
        #[n(0)]
        neuron_id: NeuronId,
        #[n(1)]
        transfer_block_height: u64,
    },

    #[n(12)]
    NeuronSixMonths(#[n(0)] NeuronId),

    #[n(13)]
    NeuronEightYears(#[n(0)] NeuronId),

    #[n(14)]
    ClaimedAirdrop {
        #[cbor(n(0), with = "crate::cbor::principal")]
        caller: Principal,
        #[n(1)]
        block_index: u64,
    },

    #[n(15)]
    MirroredProposal {
        #[n(0)]
        nns_proposal_id: ProposalId,
        #[n(1)]
        sns_proposal_id: ProposalId,
    },
}

#[derive(CandidType, Encode, Decode, Debug, PartialEq, Eq, Clone, Deserialize)]
pub struct Event {
    /// The canister time at which this event was generated.
    #[n(0)]
    pub timestamp: u64,
    /// The event type.
    #[n(1)]
    pub payload: EventType,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct GetEventsResult {
    pub events: Vec<Event>,
    pub total_event_count: u64,
}

#[derive(candid::CandidType, Deserialize)]
pub struct GetEventsArg {
    pub start: u64,
    pub length: u64,
}
