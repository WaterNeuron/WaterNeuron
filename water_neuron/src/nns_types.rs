use candid::{CandidType, Encode};
use ic_nns_governance_api::{
    Neuron as PbNeuron, NeuronState as PbNeuronState, ProposalInfo as ProposalInfoPb,
    neuron::DissolveState,
};
use ic_sns_governance_api::pb::v1::{
    ExecuteGenericNervousSystemFunction, Proposal as SnsProposal, proposal::Action as ActionSns,
};
use minicbor::{Decode, Encode as CborEncode};
use serde::{Deserialize, Serialize};

pub fn time_left_seconds(neuron: &PbNeuron, now_secs: u64) -> Option<u64> {
    match neuron.dissolve_state {
        Some(DissolveState::DissolveDelaySeconds(d)) => Some(d),
        Some(DissolveState::WhenDissolvedTimestampSeconds(ts)) => {
            if ts > now_secs {
                Some(ts - now_secs)
            } else {
                Some(0)
            }
        }
        None => None,
    }
}

pub fn state(neuron: &PbNeuron, now_seconds: u64) -> PbNeuronState {
    if neuron.spawn_at_timestamp_seconds.is_some() {
        return PbNeuronState::Spawning;
    }
    match neuron.dissolve_state {
        Some(DissolveState::DissolveDelaySeconds(d)) => {
            if d > 0 {
                PbNeuronState::NotDissolving
            } else {
                PbNeuronState::Dissolved
            }
        }
        Some(DissolveState::WhenDissolvedTimestampSeconds(ts)) => {
            if ts > now_seconds {
                PbNeuronState::Dissolving
            } else {
                PbNeuronState::Dissolved
            }
        }
        None => PbNeuronState::Dissolved,
    }
}

pub fn is_dissolved(neuron: &PbNeuron, current_ts: u64) -> bool {
    let now_seconds = current_ts / crate::SEC_NANOS;
    if neuron.state(now_seconds) == PbNeuronState::Dissolved {
        return true;
    }
    false
}

// Custom SNS function to vote on an NNS proposal.
// https://nns.ic0.app/proposal/?u=jmod6-4iaaa-aaaaq-aadkq-cai&proposal=3
pub const VOTE_ON_NNS_PROPOSAL_FUNCTION_ID: u64 = 1_000;

pub fn convert_nns_proposal_to_sns_proposal(proposal_info: &ProposalInfoPb) -> Option<SnsProposal> {
    match &proposal_info.proposal {
        Some(proposal) => {
            let original_title = proposal.title.clone().unwrap_or_default();
            let proposal_id = proposal_info.id.as_ref().unwrap().id;
            let original_proposal = format!(
                "\n\n [Original NNS proposal](https://dashboard.internetcomputer.org/proposal/{proposal_id})"
            );
            Some(SnsProposal {
                title: format!(
                    "{}({proposal_id}): {original_title}",
                    display_topic(proposal_info.topic)
                ),
                summary: format!("{}{}", proposal.summary.clone(), original_proposal),
                url: proposal.url.clone(),
                action: Some(ActionSns::ExecuteGenericNervousSystemFunction(
                    ExecuteGenericNervousSystemFunction {
                        function_id: VOTE_ON_NNS_PROPOSAL_FUNCTION_ID,
                        payload: Encode!(&proposal_id).unwrap(),
                    },
                )),
            })
        }
        None => None,
    }
}

fn display_topic(topic: i32) -> String {
    match topic {
        0 => "Unspecified".to_string(),
        1 => "NeuronManagement".to_string(),
        2 => "ExchangeRate".to_string(),
        3 => "NetworkEconomics".to_string(),
        4 => "Governance".to_string(),
        5 => "NodeAdmin".to_string(),
        6 => "ParticipantManagement".to_string(),
        7 => "SubnetManagement".to_string(),
        8 => "NetworkCanisterManagement".to_string(),
        9 => "Kyc".to_string(),
        10 => "NodeProviderRewards".to_string(),
        12 => "IcOsVersionDeployment".to_string(),
        13 => "IcOsVersionElection".to_string(),
        14 => "SnsAndCommunityFund".to_string(),
        15 => "ApiBoundaryNodeManagement".to_string(),
        16 => "SubnetRental".to_string(),
        17 => "ProtocolCanisterManagement".to_string(),
        18 => "ServiceNervousSystemManagement".to_string(),
        _ => "Unknown".to_string(),
    }
}

#[derive(
    CandidType,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    CborEncode,
    Decode,
)]
pub struct NeuronId {
    #[n(0)]
    pub id: u64,
}

impl NeuronId {
    pub fn to_dashboard_link(&self) -> String {
        format!("https://dashboard.internetcomputer.org/neuron/{}", self.id)
    }

    pub fn to_pb(&self) -> ic_nns_common::pb::v1::NeuronId {
        ic_nns_common::pb::v1::NeuronId { id: self.id }
    }
}

impl From<u64> for NeuronId {
    fn from(value: u64) -> Self {
        NeuronId { id: value }
    }
}

#[derive(
    CandidType,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    CborEncode,
    Decode,
    Default,
)]
pub struct ProposalId {
    #[n(0)]
    pub id: u64,
}

impl ProposalId {
    pub fn to_pb(&self) -> ic_nns_common::pb::v1::ProposalId {
        ic_nns_common::pb::v1::ProposalId { id: self.id }
    }
}

impl From<u64> for ProposalId {
    fn from(value: u64) -> Self {
        ProposalId { id: value }
    }
}
