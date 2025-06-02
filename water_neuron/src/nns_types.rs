use candid::Encode;
use candid::{CandidType, Principal};
use ic_nns_governance_api::{
    Neuron as PbNeuron, NeuronState as PbNeuronState, ProposalInfo as ProposalInfoPb,
};
use ic_sns_governance_api::pb::v1::{
    proposal::Action as ActionSns, ExecuteGenericNervousSystemFunction, Proposal as SnsProposal,
};
use minicbor::{Decode, Encode as CborEncode};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;
use strum_macros::EnumIter;

pub fn time_left_seconds(neuron: &PbNeuron, now_secs: u64) -> Option<u64> {
    match neuron.dissolve_state {
        Some(ic_nns_governance_api::neuron::DissolveState::DissolveDelaySeconds(d)) => Some(d),
        Some(ic_nns_governance_api::neuron::DissolveState::WhenDissolvedTimestampSeconds(ts)) => {
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
        Some(ic_nns_governance_api::neuron::DissolveState::DissolveDelaySeconds(d)) => {
            if d > 0 {
                PbNeuronState::NotDissolving
            } else {
                PbNeuronState::Dissolved
            }
        }
        Some(ic_nns_governance_api::neuron::DissolveState::WhenDissolvedTimestampSeconds(ts)) => {
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

pub const TOPIC_UNSPECIFIED: i32 = 0;
pub const TOPIC_GOVERNANCE: i32 = 4;
pub const TOPIC_SNS_AND_COMMUNITY_FUND: i32 = 14;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Empty {}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct BallotInfo {
    pub proposal_id: Option<ProposalId>,
    pub vote: i32,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct NeuronStakeTransfer {
    to_subaccount: Vec<u8>,
    neuron_stake_e8s: u64,
    from: Option<Principal>,
    memo: u64,
    from_subaccount: Vec<u8>,
    transfer_timestamp: u64,
    block_height: u64,
}

#[derive(Default, PartialEq, Eq, CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Neuron {
    pub id: Option<NeuronId>,
    pub account: Vec<u8>,
    pub controller: Option<Principal>,
    pub hot_keys: Vec<Principal>,
    pub cached_neuron_stake_e8s: u64,
    pub neuron_fees_e8s: u64,
    pub neuron_type: Option<i32>,
    pub transfer: Option<NeuronStakeTransfer>,
    pub created_timestamp_seconds: u64,
    pub aging_since_timestamp_seconds: u64,
    pub spawn_at_timestamp_seconds: Option<u64>,
    pub followees: Vec<(i32, neuron::Followees)>,
    pub recent_ballots: Vec<BallotInfo>,
    pub kyc_verified: bool,
    pub maturity_e8s_equivalent: u64,
    pub staked_maturity_e8s_equivalent: Option<u64>,
    pub auto_stake_maturity: Option<bool>,
    pub not_for_profit: bool,
    pub joined_community_fund_timestamp_seconds: Option<u64>,
    pub known_neuron_data: Option<KnownNeuronData>,
    pub dissolve_state: Option<neuron::DissolveState>,
}

#[derive(PartialEq, Eq, CandidType, Serialize, Deserialize, Clone, Debug)]
// This type is imported from:
// https://github.com/dfinity/ic/blob/master/rs/nns/governance/src/gen/ic_nns_governance.pb.v1.rs#L3280
pub enum NeuronState {
    /// Not a valid state. Required by Protobufs.
    Unspecified = 0,
    /// In this state, the neuron is not dissolving and has a specific
    /// `dissolve_delay`.
    NotDissolving = 1,
    /// In this state, the neuron's `dissolve_delay` decreases with the
    /// passage of time.
    Dissolving = 2,
    /// In the dissolved state, the neuron's stake can be disbursed using
    /// the disburse method.
    Dissolved = 3,
    /// The neuron is in spawning state, meaning it's maturity will be
    /// converted to ICP according to <https://wiki.internetcomputer.org/wiki/Maturity_modulation.>
    Spawning = 4,
}

impl Neuron {
    pub fn new_neuron(id: u64) -> Self {
        Neuron {
            id: Some(NeuronId { id }),
            ..Default::default()
        }
    }

    pub fn is_dissolved(&self, current_ts: u64) -> bool {
        let now_seconds = current_ts / crate::SEC_NANOS;
        if self.state(now_seconds) == NeuronState::Dissolved {
            return true;
        }
        false
    }

    pub fn is_dissolving(&self, current_ts: u64) -> bool {
        let now_seconds = current_ts / crate::SEC_NANOS;
        if self.state(now_seconds) == NeuronState::Dissolving {
            return true;
        }
        false
    }

    pub fn state(&self, now_seconds: u64) -> NeuronState {
        if self.spawn_at_timestamp_seconds.is_some() {
            return NeuronState::Spawning;
        }
        match self.dissolve_state {
            Some(crate::nns_types::neuron::DissolveState::DissolveDelaySeconds(d)) => {
                if d > 0 {
                    NeuronState::NotDissolving
                } else {
                    NeuronState::Dissolved
                }
            }
            Some(crate::nns_types::neuron::DissolveState::WhenDissolvedTimestampSeconds(ts)) => {
                if ts > now_seconds {
                    NeuronState::Dissolving
                } else {
                    NeuronState::Dissolved
                }
            }
            None => NeuronState::Dissolved,
        }
    }

    pub fn time_left_seconds(&self, now_secs: u64) -> Option<u64> {
        match self.dissolve_state {
            Some(crate::nns_types::neuron::DissolveState::DissolveDelaySeconds(d)) => Some(d),
            Some(crate::nns_types::neuron::DissolveState::WhenDissolvedTimestampSeconds(ts)) => {
                if ts > now_secs {
                    Some(ts - now_secs)
                } else {
                    Some(0)
                }
            }
            None => None,
        }
    }
}

pub mod neuron {
    use super::*;

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub struct Followees {
        pub followees: Vec<NeuronId>,
    }

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub enum DissolveState {
        WhenDissolvedTimestampSeconds(u64),
        DissolveDelaySeconds(u64),
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ManageNeuron {
    pub id: Option<NeuronId>,
    pub neuron_id_or_subaccount: Option<manage_neuron::NeuronIdOrSubaccount>,
    pub command: Option<manage_neuron::Command>,
}

impl ManageNeuron {
    pub fn new(neuron_id: u64, command: manage_neuron::Command) -> ManageNeuron {
        ManageNeuron {
            id: Some(NeuronId { id: neuron_id }),
            neuron_id_or_subaccount: None,
            command: Some(command),
        }
    }
}

pub mod manage_neuron {
    use super::*;

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub struct IncreaseDissolveDelay {
        pub additional_dissolve_delay_seconds: u32,
    }

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub struct StartDissolving {}

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub struct StopDissolving {}

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub struct AddHotKey {
        pub new_hot_key: Option<Principal>,
    }

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub struct RemoveHotKey {
        pub hot_key_to_remove: Option<Principal>,
    }

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub struct SetDissolveTimestamp {
        pub dissolve_timestamp_seconds: u64,
    }

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub struct JoinCommunityFund {}

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub struct LeaveCommunityFund {}

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub struct ChangeAutoStakeMaturity {
        pub requested_setting_for_auto_stake_maturity: bool,
    }

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub struct Configure {
        pub operation: Option<configure::Operation>,
    }

    pub mod configure {
        use super::*;

        #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
        pub enum Operation {
            IncreaseDissolveDelay(IncreaseDissolveDelay),
            StartDissolving(StartDissolving),
            StopDissolving(StopDissolving),
            AddHotKey(AddHotKey),
            RemoveHotKey(RemoveHotKey),
            SetDissolveTimestamp(SetDissolveTimestamp),
            JoinCommunityFund(JoinCommunityFund),
            LeaveCommunityFund(LeaveCommunityFund),
            ChangeAutoStakeMaturity(ChangeAutoStakeMaturity),
        }
    }

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub struct Disburse {
        pub amount: Option<disburse::Amount>,
        pub to_account: Option<AccountIdentifier>,
    }

    pub mod disburse {
        use super::*;

        #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
        pub struct Amount {
            pub e8s: u64,
        }
    }

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub struct Split {
        pub amount_e8s: u64,
    }

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub struct Merge {
        pub source_neuron_id: Option<NeuronId>,
    }

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Default, Eq, PartialEq)]
    pub struct Spawn {
        pub new_controller: Option<Principal>,
        pub nonce: Option<u64>,
        pub percentage_to_spawn: Option<u32>,
    }

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub struct MergeMaturity {
        pub percentage_to_merge: u32,
    }

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub struct StakeMaturity {
        pub percentage_to_stake: Option<u32>,
    }

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub struct DisburseToNeuron {
        pub new_controller: Option<Principal>,
        pub amount_e8s: u64,
        pub dissolve_delay_seconds: u64,
        pub kyc_verified: bool,
        pub nonce: u64,
    }

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub struct Follow {
        pub topic: i32,
        pub followees: Vec<NeuronId>,
    }

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub struct RegisterVote {
        pub proposal: Option<ProposalId>,
        pub vote: i32,
    }

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub struct ClaimOrRefresh {
        pub by: Option<claim_or_refresh::By>,
    }

    pub mod claim_or_refresh {
        use super::*;

        #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
        pub struct MemoAndController {
            pub memo: u64,
            pub controller: Option<Principal>,
        }

        #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
        pub enum By {
            Memo(u64),
            MemoAndController(MemoAndController),
            NeuronIdOrSubaccount(Empty),
        }
    }

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub enum NeuronIdOrSubaccount {
        Subaccount(Vec<u8>),
        NeuronId(NeuronId),
    }

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub enum Command {
        Configure(Configure),
        Disburse(Disburse),
        Spawn(Spawn),
        Follow(Follow),
        RegisterVote(RegisterVote),
        Split(Split),
        DisburseToNeuron(DisburseToNeuron),
        ClaimOrRefresh(ClaimOrRefresh),
        MergeMaturity(MergeMaturity),
        Merge(Merge),
        MakeProposal(Proposal),
        StakeMaturity(StakeMaturity),
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ManageNeuronResponse {
    pub command: Option<CommandResponse>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ClaimOrRefreshResponse {
    pub refreshed_neuron_id: Option<NeuronId>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct SpawnResponse {
    pub created_neuron_id: Option<NeuronId>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct MergeResponse {
    pub target_neuron: Option<Neuron>,
    pub source_neuron: Option<Neuron>,
    pub target_neuron_info: Option<NeuronInfo>,
    pub source_neuron_info: Option<NeuronInfo>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct MakeProposalResponse {
    pub message: Option<String>,
    pub proposal_id: Option<ProposalId>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct StakeMaturityResponse {
    pub maturity_e8s: u64,
    pub staked_maturity_e8s: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct MergeMaturityResponse {
    merged_maturity_e8s: u64,
    new_stake_e8s: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct DisburseResponse {
    pub transfer_block_height: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum CommandResponse {
    Error(GovernanceError),
    Configure(Empty),
    Disburse(DisburseResponse),
    Spawn(SpawnResponse),
    Follow(Empty),
    MakeProposal(MakeProposalResponse),
    RegisterVote(Empty),
    Split(SpawnResponse),
    DisburseToNeuron(SpawnResponse),
    ClaimOrRefresh(ClaimOrRefreshResponse),
    MergeMaturity(MergeMaturityResponse),
    Merge(Box<MergeResponse>),
    StakeMaturity(StakeMaturityResponse),
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct KnownNeuronData {
    pub name: String,
    pub description: Option<String>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ListNeurons {
    pub neuron_ids: Vec<u64>,
    pub include_neurons_readable_by_caller: bool,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct NeuronInfo {
    pub dissolve_delay_seconds: u64,
    pub recent_ballots: Vec<BallotInfo>,
    pub neuron_type: Option<i32>,
    pub created_timestamp_seconds: u64,
    pub state: i32,
    pub stake_e8s: u64,
    pub joined_community_fund_timestamp_seconds: Option<u64>,
    pub retrieved_at_timestamp_seconds: u64,
    pub known_neuron_data: Option<KnownNeuronData>,
    pub voting_power: u64,
    pub age_seconds: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ListNeuronsResponse {
    pub neuron_infos: Vec<(u64, NeuronInfo)>,
    pub full_neurons: Vec<Neuron>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct GovernanceError {
    pub error_type: i32,
    pub error_message: String,
}

pub mod governance_error {
    use super::*;

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub enum ErrorType {
        Unspecified = 0,
        Ok = 1,
        Unavailable = 2,
        NotAuthorized = 3,
        NotFound = 4,
        InvalidCommand = 5,
        RequiresNotDissolving = 6,
        RequiresDissolving = 7,
        RequiresDissolved = 8,
        HotKey = 9,
        ResourceExhausted = 10,
        PreconditionFailed = 11,
        External = 12,
        LedgerUpdateOngoing = 13,
        InsufficientFunds = 14,
        InvalidPrincipal = 15,
        InvalidProposal = 16,
        AlreadyJoinedCommunityFund = 17,
        NotInTheCommunityFund = 18,
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Default)]
pub struct ListProposalInfo {
    pub limit: u32,
    pub before_proposal: Option<ProposalId>,
    pub exclude_topic: Vec<i32>,
    pub include_reward_status: Vec<i32>,
    pub include_status: Vec<i32>,
    pub include_all_manage_neuron_proposals: Option<bool>,
    pub omit_large_fields: Option<bool>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ListProposalInfoResponse {
    pub proposal_info: Vec<ProposalInfo>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ProposalInfo {
    pub id: Option<ProposalId>,
    pub status: i32,
    pub topic: i32,
    pub failure_reason: Option<GovernanceError>,
    pub ballots: Vec<(u64, Ballot)>,
    pub proposal_timestamp_seconds: u64,
    pub reward_event_round: u64,
    pub deadline_timestamp_seconds: Option<u64>,
    pub failed_timestamp_seconds: u64,
    pub reject_cost_e8s: u64,
    pub latest_tally: Option<Tally>,
    pub reward_status: i32,
    pub decided_timestamp_seconds: u64,
    pub proposal: Option<Proposal>,
    pub proposer: Option<NeuronId>,
    pub executed_timestamp_seconds: u64,
}

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
                    Topic::from(proposal_info.topic)
                ),
                // S3: Check how special characters get rendered.
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

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq, EnumIter)]
#[repr(i32)]
pub enum Topic {
    Unspecified = 0,
    NeuronManagement = 1,
    ExchangeRate = 2,
    NetworkEconomics = 3,
    Governance = 4,
    NodeAdmin = 5,
    ParticipantManagement = 6,
    SubnetManagement = 7,
    NetworkCanisterManagement = 8,
    Kyc = 9,
    NodeProviderRewards = 10,
    SnsDecentralizationSale = 11,
    IcOsVersionDeployment = 12,
    IcOsVersionElection = 13,
    SnsAndCommunityFund = 14,
    ApiBoundaryNodeManagement = 15,
    SubnetRental = 16,
    ProtocolCanisterManagement = 17,
    ServiceNervousSystemManagement = 18,
    Unknown,
}

impl From<i32> for Topic {
    fn from(topic: i32) -> Self {
        match topic {
            0 => Self::Unspecified,
            1 => Self::NeuronManagement,
            2 => Self::ExchangeRate,
            3 => Self::NetworkEconomics,
            4 => Self::Governance,
            5 => Self::NodeAdmin,
            6 => Self::ParticipantManagement,
            7 => Self::SubnetManagement,
            8 => Self::NetworkCanisterManagement,
            9 => Self::Kyc,
            10 => Self::NodeProviderRewards,
            11 => Self::SnsDecentralizationSale,
            12 => Self::IcOsVersionDeployment,
            13 => Self::IcOsVersionElection,
            14 => Self::SnsAndCommunityFund,
            15 => Self::ApiBoundaryNodeManagement,
            16 => Self::SubnetRental,
            17 => Self::ProtocolCanisterManagement,
            18 => Self::ServiceNervousSystemManagement,
            _ => Self::Unknown,
        }
    }
}

impl fmt::Display for Topic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Topic::Unspecified => "Unspecified",
            Topic::NeuronManagement => "NeuronManagement",
            Topic::ExchangeRate => "ExchangeRate",
            Topic::NetworkEconomics => "NetworkEconomics",
            Topic::Governance => "Governance",
            Topic::NodeAdmin => "NodeAdmin",
            Topic::ParticipantManagement => "ParticipantManagement",
            Topic::SubnetManagement => "SubnetManagement",
            Topic::NetworkCanisterManagement => "NetworkCanisterManagement",
            Topic::Kyc => "Kyc",
            Topic::NodeProviderRewards => "NodeProviderRewards",
            Topic::SnsDecentralizationSale => "SnsDecentralizationSale",
            Topic::IcOsVersionDeployment => "IcOsVersionDeployment",
            Topic::IcOsVersionElection => "IcOsVersionElection",
            Topic::SnsAndCommunityFund => "SnsAndCommunityFund",
            Topic::ApiBoundaryNodeManagement => "ApiBoundaryNodeManagement",
            Topic::SubnetRental => "SubnetRental",
            Topic::ProtocolCanisterManagement => "ProtocolCanisterManagement",
            Topic::ServiceNervousSystemManagement => "ServiceNervousSystemManagement",
            Topic::Unknown => "Unknown",
        };
        write!(f, "{}", s)
    }
}

#[test]
fn should_cover_all_the_topics() {
    use strum::IntoEnumIterator;

    for topic in Topic::iter() {
        assert_eq!(Topic::from(topic.clone() as i32), topic);
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub title: Option<String>,
    pub summary: String,
    pub url: String,
    #[serde(deserialize_with = "ok_or_default")]
    pub action: Option<proposal::Action>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Ballot {
    pub vote: i32,
    pub voting_power: u64,
}

fn ok_or_default<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + Default,
    D: Deserializer<'de>,
{
    Ok(T::deserialize(deserializer).unwrap_or_default())
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Tally {
    pub timestamp_seconds: u64,
    pub yes: u64,
    pub no: u64,
    pub total: u64,
}

pub mod proposal {
    use super::*;

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub enum Action {
        ManageNeuron(Box<ManageNeuron>),
        ManageNetworkEconomics(NetworkEconomics),
        Motion(Motion),
        ExecuteNnsFunction(ExecuteNnsFunction),
        ApproveGenesisKyc(ApproveGenesisKyc),
        AddOrRemoveNodeProvider(AddOrRemoveNodeProvider),
        RewardNodeProvider(RewardNodeProvider),
        SetDefaultFollowees(SetDefaultFollowees),
        RewardNodeProviders(RewardNodeProviders),
        RegisterKnownNeuron(KnownNeuron),
        CreateServiceNervousSystem(Box<CreateServiceNervousSystem>),
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct NetworkEconomics {
    pub reject_cost_e8s: u64,
    pub neuron_minimum_stake_e8s: u64,
    pub neuron_management_fee_per_proposal_e8s: u64,
    pub minimum_icp_xdr_rate: u64,
    pub neuron_spawn_dissolve_delay_seconds: u64,
    pub maximum_node_provider_rewards_e8s: u64,
    pub transaction_fee_e8s: u64,
    pub max_proposals_to_keep_per_topic: u32,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Motion {
    pub motion_text: String,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ExecuteNnsFunction {
    pub nns_function: i32,
    pub payload: Vec<u8>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct AddOrRemoveNodeProvider {
    pub change: Option<add_or_remove_node_provider::Change>,
}

pub mod add_or_remove_node_provider {
    use super::*;

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub enum Change {
        ToAdd(NodeProvider),
        ToRemove(NodeProvider),
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct NodeProvider {
    pub id: Option<Principal>,
    pub reward_account: Option<AccountIdentifier>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct RewardNodeProvider {
    pub node_provider: Option<NodeProvider>,
    pub amount_e8s: u64,
    pub reward_mode: Option<reward_node_provider::RewardMode>,
}

pub mod reward_node_provider {
    use super::*;

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub struct RewardToNeuron {
        pub dissolve_delay_seconds: u64,
    }

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub struct RewardToAccount {
        pub to_account: Option<AccountIdentifier>,
    }

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub enum RewardMode {
        RewardToNeuron(RewardToNeuron),
        RewardToAccount(RewardToAccount),
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct RewardNodeProviders {
    pub rewards: Vec<RewardNodeProvider>,
    pub use_registry_derived_rewards: Option<bool>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct SetDefaultFollowees {
    pub default_followees: Vec<(i32, neuron::Followees)>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct KnownNeuron {
    pub id: Option<NeuronId>,
    pub known_neuron_data: Option<KnownNeuronData>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ApproveGenesisKyc {
    pub principals: Vec<Principal>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct CreateServiceNervousSystem {
    pub name: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub logo: Option<Image>,
    pub fallback_controller_principal_ids: Vec<Principal>,
    pub dapp_canisters: Vec<Canister>,
    pub initial_token_distribution: Option<create_service_nervous_system::InitialTokenDistribution>,
    pub swap_parameters: Option<create_service_nervous_system::SwapParameters>,
    pub ledger_parameters: Option<create_service_nervous_system::LedgerParameters>,
    pub governance_parameters: Option<create_service_nervous_system::GovernanceParameters>,
}

pub mod create_service_nervous_system {
    use super::*;

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub struct InitialTokenDistribution {
        pub developer_distribution: Option<initial_token_distribution::DeveloperDistribution>,
        pub treasury_distribution: Option<initial_token_distribution::TreasuryDistribution>,
        pub swap_distribution: Option<initial_token_distribution::SwapDistribution>,
    }

    pub mod initial_token_distribution {
        use super::*;

        #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
        pub struct DeveloperDistribution {
            pub developer_neurons: Vec<developer_distribution::NeuronDistribution>,
        }

        pub mod developer_distribution {
            use super::*;

            #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
            pub struct NeuronDistribution {
                pub controller: Option<Principal>,
                pub dissolve_delay: Option<Duration>,
                pub memo: Option<u64>,
                pub stake: Option<Tokens>,
                pub vesting_period: Option<Duration>,
            }
        }

        #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
        pub struct TreasuryDistribution {
            pub total: Option<Tokens>,
        }

        #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
        pub struct SwapDistribution {
            pub total: Option<Tokens>,
        }
    }

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub struct SwapParameters {
        pub minimum_participants: Option<u64>,
        pub minimum_icp: Option<Tokens>,
        pub maximum_icp: Option<Tokens>,
        pub minimum_direct_participation_icp: Option<Tokens>,
        pub maximum_direct_participation_icp: Option<Tokens>,
        pub minimum_participant_icp: Option<Tokens>,
        pub maximum_participant_icp: Option<Tokens>,
        pub neuron_basket_construction_parameters:
            Option<swap_parameters::NeuronBasketConstructionParameters>,
        pub confirmation_text: Option<String>,
        pub restricted_countries: Option<Countries>,
        pub start_time: Option<GlobalTimeOfDay>,
        pub duration: Option<Duration>,
        pub neurons_fund_investment_icp: Option<Tokens>,
        pub neurons_fund_participation: Option<bool>,
    }

    pub mod swap_parameters {
        use super::*;

        #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
        pub struct NeuronBasketConstructionParameters {
            pub count: Option<u64>,
            pub dissolve_delay_interval: Option<Duration>,
        }
    }

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub struct LedgerParameters {
        pub transaction_fee: Option<Tokens>,
        pub token_name: Option<String>,
        pub token_symbol: Option<String>,
        pub token_logo: Option<Image>,
    }

    #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    pub struct GovernanceParameters {
        pub proposal_rejection_fee: Option<Tokens>,
        pub proposal_initial_voting_period: Option<Duration>,
        pub proposal_wait_for_quiet_deadline_increase: Option<Duration>,
        pub neuron_minimum_stake: Option<Tokens>,
        pub neuron_minimum_dissolve_delay_to_vote: Option<Duration>,
        pub neuron_maximum_dissolve_delay: Option<Duration>,
        pub neuron_maximum_dissolve_delay_bonus: Option<Percentage>,
        pub neuron_maximum_age_for_age_bonus: Option<Duration>,
        pub neuron_maximum_age_bonus: Option<Percentage>,
        pub voting_reward_parameters: Option<governance_parameters::VotingRewardParameters>,
    }

    pub mod governance_parameters {
        use super::*;

        #[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
        pub struct VotingRewardParameters {
            pub initial_reward_rate: Option<Percentage>,
            pub final_reward_rate: Option<Percentage>,
            pub reward_rate_transition_duration: Option<Duration>,
        }
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

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Tokens {
    pub e8s: Option<u64>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Duration {
    pub seconds: Option<u64>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Percentage {
    pub basis_points: Option<u64>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Canister {
    pub id: Option<Principal>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Image {
    pub base64_encoding: Option<String>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct GlobalTimeOfDay {
    pub seconds_after_utc_midnight: Option<u64>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Countries {
    pub iso_codes: Vec<String>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct AccountIdentifier {
    pub hash: Vec<u8>,
}
