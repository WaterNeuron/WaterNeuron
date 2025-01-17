use crate::dashboard::DisplayAmount;
use crate::guards::{GuardError, TaskGuard};
use crate::logs::{DEBUG, INFO};
use crate::management::{
    balance_of, disburse, follow_neuron, get_full_neuron, get_full_neuron_by_nonce,
    increase_dissolve_delay, list_neurons, manage_neuron_sns, refresh_neuron, register_vote,
    spawn_all_maturity, split_neuron, start_dissolving, transfer,
};
use crate::nns_types::{
    neuron::DissolveState, CommandResponse, GovernanceError, ListNeurons, NeuronId, ProposalId,
    TOPIC_GOVERNANCE, TOPIC_SNS_AND_COMMUNITY_FUND, TOPIC_UNSPECIFIED,
};
use crate::numeric::{nICP, ICP};
use crate::proposal::{mirror_proposals, vote_on_nns_proposals};
use crate::sns_governance::{CanisterRuntime, IcCanisterRuntime, WTN_MAX_DISSOLVE_DELAY_SECONDS};
use crate::state::audit::process_event;
use crate::state::event::EventType;
use crate::state::{
    mutate_state, read_state, NeuronOrigin, TransferId, EIGHT_YEARS_NEURON_NONCE, ICP_LEDGER_ID,
    NNS_GOVERNANCE_ID, SIX_MONTHS_NEURON_NONCE, SNS_GOVERNANCE_SUBACCOUNT,
};
use crate::storage::{get_rewards_ready_to_be_distributed, stable_sub_rewards};
use crate::tasks::{schedule_after, schedule_now, TaskType};
use candid::{CandidType, Deserialize, Nat, Principal};
use ic_canister_log::log;
use icrc_ledger_client_cdk::{CdkRuntime, ICRC1Client};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::TransferError;
use icrc_ledger_types::icrc2::transfer_from::TransferFromError;
use minicbor::{Decode, Encode};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::fmt;
use std::time::Duration;
use strum::IntoEnumIterator;

pub mod cbor;
pub mod conversion;
pub mod dashboard;
pub mod guards;
pub mod icrc21;
pub mod logs;
pub mod management;
pub mod nns_types;
pub mod numeric;
pub mod proposal;
pub mod sns_distribution;
pub mod sns_governance;
pub mod state;
pub mod storage;
pub mod tasks;

#[cfg(test)]
pub mod state_machine;

const RETRY_DELAY: Duration = Duration::from_secs(10);
const RETRY_DELAY_VOTING: Duration = Duration::from_secs(60);

#[cfg(feature = "self_check")]
const LOGIC_DELAY: Duration = RETRY_DELAY;

#[cfg(not(feature = "self_check"))]
const LOGIC_DELAY: Duration = ONE_HOUR;

const ONE_MINUTE: Duration = Duration::from_secs(60);
const ONE_HOUR: Duration = Duration::from_secs(60 * 60);
pub const ONE_DAY: Duration = Duration::from_secs(24 * 60 * 60);

pub const SEC_NANOS: u64 = 1_000_000_000;
pub const ONE_WEEK_NANOS: u64 = 7 * 24 * 60 * 60 * SEC_NANOS;

const ONE_HOUR_SECONDS: u64 = 60 * 60;
pub const ONE_DAY_SECONDS: u64 = 24 * 60 * 60;
pub const ONE_YEAR_SECONDS: u64 = (4 * 365 + 1) * ONE_DAY_SECONDS / 4;
pub const ONE_MONTH_SECONDS: u64 = ONE_YEAR_SECONDS / 12;
pub const MAX_DISSOLVE_DELAY_SECONDS: u64 = 8 * ONE_YEAR_SECONDS;
pub const MIN_DISSOLVE_DELAY_FOR_REWARDS: u64 = 6 * ONE_MONTH_SECONDS + ONE_DAY_SECONDS;

pub const DEFAULT_LEDGER_FEE: u64 = 10_000;
pub const NEURON_LEDGER_FEE: u64 = 1_000_000;
const E8S: u64 = 100_000_000;
const MINIMUM_ICP_DISTRIBUTION: u64 = 100 * E8S;
pub const INITIAL_NEURON_STAKE: u64 = E8S + 42;

pub const SNS_DISTRIBUTION_MEMO: u64 = 83_78_83;

#[cfg(target_arch = "wasm32")]
pub fn timestamp_nanos() -> u64 {
    ic_cdk::api::time()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn timestamp_nanos() -> u64 {
    use std::time::SystemTime;

    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

#[cfg(target_arch = "wasm32")]
pub fn self_canister_id() -> Principal {
    ic_cdk::id()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn self_canister_id() -> Principal {
    Principal::anonymous()
}

#[derive(Deserialize, CandidType)]
pub enum LiquidArg {
    Init(InitArg),
    Upgrade(Option<UpgradeArg>),
}

#[derive(Deserialize, CandidType, Encode, Decode, PartialEq, Eq, Clone, Debug)]
pub struct InitArg {
    #[cbor(n(0), with = "crate::cbor::principal")]
    pub nicp_ledger_id: Principal,
    #[cbor(n(1), with = "crate::cbor::principal")]
    pub wtn_governance_id: Principal,
    #[cbor(n(2), with = "crate::cbor::principal")]
    pub wtn_ledger_id: Principal,
}

#[derive(Deserialize, CandidType, Encode, Decode, PartialEq, Eq, Clone, Debug)]
pub struct UpgradeArg {
    #[n(0)]
    pub governance_fee_share_percent: Option<u64>,
}

#[derive(CandidType, Debug, Deserialize, PartialEq, Serialize)]
pub struct CanisterInfo {
    pub latest_distribution_icp_per_vp: Option<f64>,
    pub neuron_id_6m: Option<NeuronId>,
    pub neuron_6m_stake_e8s: ICP,
    pub tracked_6m_stake: ICP,
    pub neuron_6m_account: Account,
    pub neuron_id_8y: Option<NeuronId>,
    pub neuron_8y_stake_e8s: ICP,
    pub neuron_8y_account: Account,
    pub exchange_rate: u64,
    pub stakers_count: usize,
    pub total_icp_deposited: ICP,
    pub nicp_supply: nICP,
    pub minimum_deposit_amount: ICP,
    pub minimum_withdraw_amount: ICP,
    pub governance_fee_share_percent: u64,
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Encode, Decode)]
pub enum Unit {
    #[n(0)]
    ICP = 0,
    #[n(1)]
    NICP = 1,
    #[n(2)]
    WTN = 2,
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Unit::ICP => write!(f, "ICP"),
            Unit::NICP => write!(f, "nICP"),
            Unit::WTN => write!(f, "WTN"),
        }
    }
}

impl Unit {
    pub fn ledger_id(&self) -> Principal {
        match self {
            Unit::ICP => ICP_LEDGER_ID,
            Unit::NICP => read_state(|s| s.nicp_ledger_id),
            Unit::WTN => read_state(|s| s.wtn_ledger_id),
        }
    }

    pub fn fee(&self) -> u64 {
        match self {
            Unit::ICP => DEFAULT_LEDGER_FEE,
            Unit::NICP => 0, // The fee is 0 as this canister is the minting account
            Unit::WTN => NEURON_LEDGER_FEE,
        }
    }
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ConversionArg {
    pub amount_e8s: u64,
    pub maybe_subaccount: Option<[u8; 32]>,
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Encode, Decode)]
pub struct PendingTransfer {
    #[n(0)]
    pub transfer_id: TransferId,
    #[cbor(n(1), with = "minicbor::bytes")]
    pub from_subaccount: Option<[u8; 32]>,
    #[n(2)]
    pub memo: Option<u64>,
    #[n(3)]
    pub amount: u64,
    #[cbor(n(4), with = "crate::cbor::account")]
    pub receiver: Account,
    #[n(5)]
    pub unit: Unit,
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct PendingWithdrawal {
    pub due_amount: Nat,
    pub beneficiary: Account,
}

#[derive(CandidType, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct DepositSuccess {
    pub block_index: Nat,
    pub transfer_id: u64,
    pub nicp_amount: Option<nICP>,
}

#[derive(CandidType, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct WithdrawalSuccess {
    pub block_index: Nat,
    pub withdrawal_id: u64,
    pub icp_amount: Option<ICP>,
}

#[derive(CandidType, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum ConversionError {
    TransferFromError(TransferFromError),
    TransferError(TransferError),
    AmountTooLow { minimum_amount_e8s: u64 },
    GuardError { guard_error: GuardError },
    GenericError { code: i32, message: String },
}

#[derive(CandidType, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum CancelWithdrawalError {
    GuardError { guard_error: GuardError },
    GenericError { code: i32, message: String },
    GovernanceError(GovernanceError),
    BadCommand { message: String },
    BadCaller { message: String },
    RequestNotFound,
    StopDissolvementError { message: String },
    MergeNeuronError { message: String },
    GetFullNeuronError { message: String },
    TooLate,
    UnknownTimeLeft,
}

/// Computes the bytes of the subaccount to which neuron staking transfers are made. This
/// function must be kept in sync with the Nervous System UI equivalent.
/// This code comes from the IC repo:
/// https://github.com/dfinity/ic/blob/master/rs/nervous_system/common/src/ledger.rs#L211
pub fn compute_neuron_staking_subaccount_bytes(controller: Principal, nonce: u64) -> [u8; 32] {
    const DOMAIN: &[u8] = b"neuron-stake";
    const DOMAIN_LENGTH: [u8; 1] = [0x0c];

    let mut hasher = Sha256::new();
    hasher.update(DOMAIN_LENGTH);
    hasher.update(DOMAIN);
    hasher.update(controller.as_slice());
    hasher.update(nonce.to_be_bytes());
    hasher.finalize().into()
}

pub fn timer() {
    if let Some(task) = tasks::pop_if_ready() {
        let task_type = task.task_type;
        match task.task_type {
            TaskType::MaybeInitializeMainNeurons => {
                ic_cdk::spawn(async move {
                    let _guard = match TaskGuard::new(task_type) {
                        Ok(guard) => guard,
                        Err(_) => return,
                    };

                    let _enqueue_followup_guard = scopeguard::guard((), |_| {
                        schedule_after(RETRY_DELAY, TaskType::MaybeInitializeMainNeurons);
                    });

                    configure_sns_voting_neuron().await;

                    if let Err(e) = initialize_main_neurons().await {
                        log!(
                            INFO,
                            "[MaybeInitializeMainNeurons] Failed to initialize main neurons with error: {e}",
                        );
                        schedule_after(RETRY_DELAY, TaskType::MaybeInitializeMainNeurons);
                    }

                    scopeguard::ScopeGuard::into_inner(_enqueue_followup_guard);
                });
            }
            TaskType::MaybeDistributeICP => {
                ic_cdk::spawn(async move {
                    let _guard = match TaskGuard::new(task_type) {
                        Ok(guard) => guard,
                        Err(_) => return,
                    };

                    let runtime = IcCanisterRuntime {};
                    dispatch_icp(&runtime).await;

                    schedule_after(ONE_HOUR, TaskType::MaybeDistributeICP);
                });
            }
            TaskType::ProcessVoting => {
                ic_cdk::spawn(async move {
                    let _guard = match TaskGuard::new(task_type) {
                        Ok(guard) => guard,
                        Err(_) => return,
                    };

                    if let Err(e) = mirror_proposals().await {
                        log!(
                            INFO,
                            "[ProcessVoting] failed to mirror proposals with error: {e}"
                        );
                        schedule_after(RETRY_DELAY, TaskType::ProcessVoting);
                    }
                    vote_on_nns_proposals().await;

                    schedule_after(Duration::from_secs(30 * 60), TaskType::ProcessVoting);
                });
            }
            TaskType::ProcessPendingTransfers => {
                ic_cdk::spawn(async move {
                    let _guard = match TaskGuard::new(task_type) {
                        Ok(guard) => guard,
                        Err(_) => return,
                    };

                    let error_count = process_pending_transfer().await;
                    if error_count > 0 {
                        log!(INFO, "[ProcessPendingTransfers] Failed to process {error_count} transfers, rescheduling task.");
                        schedule_after(RETRY_DELAY, TaskType::ProcessPendingTransfers);
                    }
                });
            }
            TaskType::ProcessLogic => {
                ic_cdk::spawn(async move {
                    let _guard = match TaskGuard::new(task_type) {
                        Ok(guard) => guard,
                        Err(_) => return,
                    };

                    refresh_stakes().await;
                    process_witdhrawals_splitting().await;
                    process_start_dissolving().await;
                    process_disburse().await;

                    schedule_after(LOGIC_DELAY, TaskType::ProcessLogic);
                });
            }
            TaskType::SpawnNeurons => {
                ic_cdk::spawn(async move {
                    let _guard = match TaskGuard::new(task_type) {
                        Ok(guard) => guard,
                        Err(_) => return,
                    };

                    process_spawn().await;
                    schedule_after(ONE_DAY, TaskType::SpawnNeurons);
                });
            }
            TaskType::RefreshShortTerm => {
                ic_cdk::spawn(async move {
                    let _guard = match TaskGuard::new(task_type) {
                        Ok(guard) => guard,
                        Err(_) => return,
                    };

                    let _ = refresh_neuron(SIX_MONTHS_NEURON_NONCE).await;
                    if let Ok(main_neuron_6m_staked) =
                        fetch_neuron_stake(SIX_MONTHS_NEURON_NONCE).await
                    {
                        mutate_state(|s| s.main_neuron_6m_staked = main_neuron_6m_staked);
                    }
                });
            }
            TaskType::MaybeDistributeRewards => {
                ic_cdk::spawn(async move {
                    let _guard = match TaskGuard::new(task_type) {
                        Ok(guard) => guard,
                        Err(_) => return,
                    };

                    distribute_icp_to_sns_neurons().await;

                    let runtime = IcCanisterRuntime {};
                    match crate::sns_governance::process_icp_distribution(&runtime).await {
                        Some(error_count) => {
                            if error_count > 0 {
                                log!(INFO, "[MaybeDistributeRewards] Failed to process {error_count} transfers, rescheduling task.");
                                schedule_after(RETRY_DELAY, TaskType::MaybeDistributeRewards);
                            }
                        }
                        None => {
                            schedule_after(ONE_DAY, TaskType::MaybeDistributeRewards);
                        }
                    }
                });
            }
        }
    }
}

async fn refresh_stakes() {
    let _ = refresh_neuron(EIGHT_YEARS_NEURON_NONCE).await;
    if let Ok(neuron_8y_stake_e8s) = fetch_neuron_stake(EIGHT_YEARS_NEURON_NONCE).await {
        mutate_state(|s| s.main_neuron_8y_stake = neuron_8y_stake_e8s);
    }
    let _ = refresh_neuron(SIX_MONTHS_NEURON_NONCE).await;
    if let Ok(main_neuron_6m_staked) = fetch_neuron_stake(SIX_MONTHS_NEURON_NONCE).await {
        mutate_state(|s| s.main_neuron_6m_staked = main_neuron_6m_staked);
    }
}

pub async fn initialize_main_neurons() -> Result<(), String> {
    if read_state(|s| s.neuron_id_6m.is_none()) {
        let neuron_id_6m =
            initialize_main_neuron(SIX_MONTHS_NEURON_NONCE, MIN_DISSOLVE_DELAY_FOR_REWARDS).await?;
        mutate_state(|s| process_event(s, EventType::NeuronSixMonths(neuron_id_6m)));
        log!(
            INFO,
            "[MaybeInitializeMainNeurons] Initialized 6-month neuron: {neuron_id_6m:?}",
        );
    }

    if read_state(|s| s.neuron_id_8y.is_none()) {
        let neuron_id_8y =
            initialize_main_neuron(EIGHT_YEARS_NEURON_NONCE, MAX_DISSOLVE_DELAY_SECONDS).await?;
        mutate_state(|s| process_event(s, EventType::NeuronEightYears(neuron_id_8y)));
        log!(
            INFO,
            "[MaybeInitializeMainNeurons] Initialized 8-year neuron: {neuron_id_8y:?}",
        );
    }

    if read_state(|s| s.neuron_id_6m.is_some() && s.neuron_id_8y.is_some()) {
        neuron_8y_follows_6m().await?;
    } else {
        return Err("Neuron 6 months or 8 years not set".to_string());
    }

    Ok(())
}

async fn neuron_8y_follows_6m() -> Result<(), String> {
    let neuron_id_6m = read_state(|s| s.neuron_id_6m.expect("neuron id 6-month not set."));
    let neuron_id_8y = read_state(|s| s.neuron_id_8y.expect("neuron id 8-year not set."));

    // The `Unspecified` topic is used as a fallback when
    // following. That is, if no followees are specified for a given
    // topic, the followees for this topic are used instead.
    // https://github.com/dfinity/ic/blob/master/rs/nns/governance/proto/ic_nns_governance/pb/v1/governance.proto#L45
    follow_neuron(neuron_id_8y, TOPIC_UNSPECIFIED, neuron_id_6m).await?;

    // For the GOVERNANCE and the SNS topics, the default following doesn't apply.
    // We need to also follow the 6 months neuron with those topics.
    // https://github.com/dfinity/ic/blob/17df8febdb922c3981475035d830f09d9b990a5a/rs/nns/governance/src/governance.rs#L5408
    follow_neuron(neuron_id_8y, TOPIC_GOVERNANCE, neuron_id_6m).await?;
    follow_neuron(neuron_id_8y, TOPIC_SNS_AND_COMMUNITY_FUND, neuron_id_6m).await?;

    Ok(())
}

async fn configure_sns_voting_neuron() {
    use ic_sns_governance::pb::v1::manage_neuron::configure::Operation as OperationSns;
    use ic_sns_governance::pb::v1::manage_neuron::{
        Command as CommandSns, Configure as ConfigureSns,
        IncreaseDissolveDelay as IncreaseDissolveDelaySns,
    };

    let arg = CommandSns::Configure(ConfigureSns {
        operation: Some(OperationSns::IncreaseDissolveDelay(
            IncreaseDissolveDelaySns {
                additional_dissolve_delay_seconds: WTN_MAX_DISSOLVE_DELAY_SECONDS as u32,
            },
        )),
    });

    let subaccount = compute_neuron_staking_subaccount_bytes(self_canister_id(), 0).to_vec();
    let result = manage_neuron_sns(subaccount, arg).await;
    log!(
        DEBUG,
        "[configure_sns_voting_neuron] manage sns neuron response {result:?}"
    );
}

async fn process_pending_transfer() -> u64 {
    let mut error_count = 0;

    let pending_transfers: Vec<PendingTransfer> = read_state(|s| {
        s.pending_transfers
            .values()
            .cloned()
            .collect::<Vec<PendingTransfer>>()
    });

    for transfer in pending_transfers {
        let (ledger_id, fee) = (transfer.unit.ledger_id(), transfer.unit.fee());
        if transfer.amount <= fee || transfer.receiver == NNS_GOVERNANCE_ID.into() {
            log!(
                INFO,
                "[process_pending_transfer] Impossible transfer with id {}, skipping.",
                transfer.transfer_id
            );
            mutate_state(|s| {
                process_event(
                    s,
                    EventType::TransferExecuted {
                        transfer_id: transfer.transfer_id,
                        block_index: None,
                    },
                );
            });
        }
        match crate::management::transfer(
            transfer.receiver,
            transfer
                .amount
                .checked_sub(fee)
                .expect("bug: all transfers should be greater than the fee")
                .into(),
            Some(Nat::from(fee)),
            transfer.from_subaccount,
            ledger_id,
            transfer.memo,
        )
        .await
        {
            Ok(block_index) => {
                log!(
                    INFO,
                    "[process_pending_transfer] successfully transfered: {} {} to {}, transfer id: {}",
                    DisplayAmount(transfer.amount),
                    transfer.unit,
                    transfer.receiver,
                    transfer.transfer_id
                );
                mutate_state(|s| {
                    process_event(
                        s,
                        EventType::TransferExecuted {
                            transfer_id: transfer.transfer_id,
                            block_index: Some(block_index),
                        },
                    );
                });
            }
            Err(error) => {
                log!(
                    DEBUG,
                    "[process_pending_transfer] failed to transfer margin: {}, with error: {}",
                    transfer.amount,
                    error
                );
                error_count += 1;
            }
        }
    }
    error_count
}

async fn initialize_main_neuron(
    neuron_nonce: u64,
    dissolve_delay_seconds: u64,
) -> Result<NeuronId, String> {
    let target = Account {
        owner: NNS_GOVERNANCE_ID,
        subaccount: Some(compute_neuron_staking_subaccount_bytes(
            ic_cdk::id(),
            neuron_nonce,
        )),
    };

    if balance_of(target, ICP_LEDGER_ID).await? < E8S + 1 {
        let block_index = transfer(
            target,
            Nat::from(INITIAL_NEURON_STAKE),
            Some(Nat::from(DEFAULT_LEDGER_FEE)),
            None,
            ICP_LEDGER_ID,
            None,
        )
        .await
        .map_err(|e| format!("{}", e))?;
        log!(
            DEBUG,
            "[initialize_main_neuron] Successfully transfered at index {}",
            block_index
        );
    }

    let refresh_neuron_result = refresh_neuron(neuron_nonce).await?.command.unwrap();
    let neuron_id = if let CommandResponse::ClaimOrRefresh(refresh_response) = refresh_neuron_result
    {
        refresh_response.refreshed_neuron_id.unwrap()
    } else {
        return Err(format!(
            "Failed to refresh neuron with error: {refresh_neuron_result:?}"
        ));
    };

    match get_full_neuron(neuron_id.id).await? {
        Ok(neuron) => {
            match neuron
                .dissolve_state
                .expect("BUG: this field should be set at any time")
            {
                DissolveState::DissolveDelaySeconds(found_dissolve_delay_seconds) => {
                    match (found_dissolve_delay_seconds).cmp(&dissolve_delay_seconds) {
                        Ordering::Greater => {
                            // TODO: decrease dissolve delay
                            // S5: you could remove this TODO and change the contract of this function that it guarantees at least the given dissolve duration.
                            // If you go this path, then you could greatly simplify this `match` by using a simple `if`.
                        }
                        Ordering::Less => {
                            let dissolve_delay_diff =
                                dissolve_delay_seconds - found_dissolve_delay_seconds;
                            let result =
                                increase_dissolve_delay(neuron_nonce, dissolve_delay_diff as u32)
                                    .await?;
                            log!(
                                INFO,
                                "[initialize_main_neuron] increase disolve delay result: {:?}",
                                result
                            );
                        }
                        Ordering::Equal => {
                            // Don't do anything, the dissolve delay is set as expected.
                        }
                    }
                }
                DissolveState::WhenDissolvedTimestampSeconds(_) => {
                    let result =
                        increase_dissolve_delay(neuron_nonce, dissolve_delay_seconds as u32)
                            .await?;
                    log!(
                        INFO,
                        "[initialize_main_neuron] increase disolve delay result: {:?}",
                        result
                    );
                }
            }
            Ok(neuron_id)
        }
        Err(error) => Err(format!("failed to get full neuron with error: {error:?}")),
    }
}

async fn fetch_neuron_stake(neuron_nonce: u64) -> Result<ICP, String> {
    let res = get_full_neuron_by_nonce(neuron_nonce).await?;
    match res {
        Ok(neuron) => Ok(ICP::from_e8s(neuron.cached_neuron_stake_e8s)),
        Err(gov_err) => Err(format!("governance error: {gov_err:?}")),
    }
}

async fn process_start_dissolving() {
    if read_state(|s| s.withdrawal_to_start_dissolving.is_empty()) {
        return;
    }

    let dissolve_request_ids = read_state(|s| s.get_withdrawal_request_ids_to_dissolve());

    for neuron_id in &dissolve_request_ids {
        let result = start_dissolving(*neuron_id).await;
        if result.is_ok() {
            mutate_state(|s| {
                if let Some(withdrawal_id) = s.neuron_id_to_withdrawal_id(*neuron_id) {
                    process_event(s, EventType::StartedToDissolve { withdrawal_id });
                } else {
                    panic!("bug: neuron_id_to_withdrawal_id doesn't contain withdrawal id associated with neuron {neuron_id:?}");
                }
            });
            log!(
                DEBUG,
                "[process_start_dissolving] Dissolving neuron with id: {}",
                neuron_id.id
            );
        } else {
            log!(
                DEBUG,
                "[process_start_dissolving] Failed with error: {result:?}"
            );
        }
    }
}

async fn process_disburse() {
    if read_state(|s| s.to_disburse.is_empty()) {
        return;
    }

    let neuron_ids = read_state(|s| {
        s.to_disburse
            .values()
            .map(|req| req.neuron_id.id)
            .collect::<Vec<u64>>()
    });

    // Helper function to chunk the neuron_ids
    fn chunk_ids(ids: Vec<u64>, chunk_size: usize) -> Vec<Vec<u64>> {
        ids.chunks(chunk_size).map(|chunk| chunk.to_vec()).collect()
    }

    let chunks = chunk_ids(neuron_ids, 100);

    for chunk in chunks {
        match list_neurons(ListNeurons {
            neuron_ids: chunk,
            include_neurons_readable_by_caller: false,
        })
        .await
        {
            Ok(response) => {
                for neuron in response.full_neurons {
                    if !neuron.is_dissolved(timestamp_nanos()) {
                        continue;
                    }
                    if let Some(neuron_id) = neuron.id {
                        if let Some(req) = read_state(|s| s.to_disburse.get(&neuron_id).cloned()) {
                            log!(
                                INFO,
                                "[process_disburse] Disbursing neuron id: {}",
                                neuron_id.id
                            );
                            match disburse(neuron_id, req.receiver).await {
                                Ok(disburse_response) => {
                                    schedule_now(TaskType::ProcessPendingTransfers);
                                    log!(
                                        INFO,
                                        "[process_disburse] Sucessfully disbursed at height: {}",
                                        disburse_response.transfer_block_height
                                    );
                                    mutate_state(|s| {
                                        match s.neuron_id_to_withdrawal_id(neuron_id) {
                                            Some(withdrawal_id) => {
                                                process_event(
                                                    s,
                                                    EventType::DisbursedUserNeuron {
                                                        withdrawal_id,
                                                        transfer_block_height: disburse_response
                                                            .transfer_block_height,
                                                    },
                                                );
                                            }
                                            None => {
                                                process_event(
                                                    s,
                                                    EventType::DisbursedMaturityNeuron {
                                                        neuron_id,
                                                        transfer_block_height: disburse_response
                                                            .transfer_block_height,
                                                    },
                                                );
                                                schedule_after(
                                                    ONE_MINUTE,
                                                    TaskType::MaybeDistributeICP,
                                                );
                                            }
                                        }
                                    });
                                }
                                Err(e) => {
                                    log!(
                                        INFO,
                                        "[process_disburse] failed to disburse neurons: {e:?}"
                                    )
                                }
                            }
                        }
                    }
                }
            }
            Err(error) => log!(
                INFO,
                "[process_disburse] failed to fetch list_neurons with error: {error}"
            ),
        }
    }
}

/// Distribute ICP to the SNS neurons. This will fetch all the SNS neurons
/// and distribute rewards proportionally.
async fn distribute_icp_to_sns_neurons() {
    if read_state(|s| s.is_processing_icp_transfer_from_sns_subaccount()) {
        schedule_now(TaskType::ProcessPendingTransfers);
        log!(
            DEBUG,
            "[distribute_icp_to_sns_neurons] Still processing ICP transfers."
        );
        return;
    }

    let last_distribution_ts = read_state(|s| s.last_distribution_ts);
    if last_distribution_ts + ONE_WEEK_NANOS > timestamp_nanos() {
        log!(
            INFO,
            "[distribute_icp_to_sns_neurons] Distributed less than a week ago, last distribution ts: {}",
            last_distribution_ts / SEC_NANOS
        );
        return;
    }

    let sns_account = read_state(|s| s.get_sns_account());

    match balance_of(sns_account, ICP_LEDGER_ID).await {
        Ok(balance) => {
            if balance >= MINIMUM_ICP_DISTRIBUTION {
                let runtime = IcCanisterRuntime {};
                match crate::sns_governance::maybe_fetch_neurons_and_distribute(&runtime, balance)
                    .await
                {
                    Ok(stakers_count) => {
                        schedule_now(TaskType::ProcessPendingTransfers);
                        log!(
                            INFO,
                            "[distribute_icp_to_sns_neurons] Distributed {} ICP to {} stakers.",
                            DisplayAmount(balance),
                            stakers_count
                        );
                    }
                        Err(error) => log!(INFO, "[distribute_icp_to_sns_neurons] Failed to distribute ICP to SNS neurons {error}"),
                }
            } else {
                log!(DEBUG, "[distribute_icp_to_sns_neurons] Not enough ICP to distribute, balance {balance} ICP min {} ICP", MINIMUM_ICP_DISTRIBUTION / E8S);
            }
        }
        Err(error) => {
            log!(
                DEBUG,
                "[distribute_icp_to_sns_neurons] failed to fetch balance with error: {error}"
            );
            schedule_after(ONE_MINUTE, TaskType::MaybeDistributeICP);
        }
    }
}

async fn dispatch_icp<R: CanisterRuntime>(runtime: &R) {
    if read_state(|s| s.is_processing_icp_transfer_from_neuron()) {
        schedule_now(TaskType::ProcessPendingTransfers);
        log!(DEBUG, "[dispatch_icp] Some ICP withdrawal are still queued");
        return;
    }

    for neuron_type in NeuronOrigin::iter() {
        match runtime
            .balance_of(
                Account {
                    owner: self_canister_id(),
                    subaccount: Some(neuron_type.to_subaccount()),
                },
                ICP_LEDGER_ID,
            )
            .await
        {
            Ok(balance) => {
                if balance > MINIMUM_ICP_DISTRIBUTION {
                    let governance_share_e8s =
                        read_state(|s| s.compute_governance_share_e8s(balance));
                    let nicp_share_e8s = balance.checked_sub(governance_share_e8s).expect(
                        "bug: the governance share should always be strictly less than the balance",
                    );
                    log!(INFO, "[dispatch_icp] {neuron_type} generated {} ICP for nICP and {} for SNS governance.", DisplayAmount(nicp_share_e8s), DisplayAmount(governance_share_e8s));
                    mutate_state(|s| {
                        process_event(
                            s,
                            EventType::DispatchICPRewards {
                                nicp_amount: ICP::from_e8s(nicp_share_e8s),
                                sns_gov_amount: ICP::from_e8s(governance_share_e8s),
                                from_neuron_type: neuron_type,
                            },
                        );
                    });
                    schedule_now(TaskType::ProcessPendingTransfers);
                    schedule_now(TaskType::MaybeDistributeRewards);
                } else {
                    log!(
                        DEBUG,
                        "[dispatch_icp] Canister balance: {} is not big enough to distribute.",
                        DisplayAmount(balance)
                    );
                }
            }
            Err(error) => {
                log!(
                    INFO,
                    "[dispatch_icp] failed to fetch balance with error: {error}"
                );
                schedule_after(ONE_MINUTE, TaskType::MaybeDistributeICP);
            }
        }
    }
}

async fn process_spawn() {
    async fn process_neuron_spawn(neuron_id: Option<NeuronId>, from_neuron_type: NeuronOrigin) {
        if let Some(id) = neuron_id {
            match spawn_all_maturity(id).await {
                Ok(neuron_id) => {
                    log!(
                        INFO,
                        "[process_spawn] Spawned neuron {neuron_id:?} with type {from_neuron_type}",
                    );
                    mutate_state(|s| {
                        process_event(
                            s,
                            EventType::MaturityNeuron {
                                neuron_id,
                                from_neuron_type,
                            },
                        );
                    });
                }
                Err(e) => {
                    log!(
                        INFO,
                        "[process_spawn] failed to spawn maturity for {from_neuron_type} neuron: {e:?}",
                    );
                }
            }
        }
    }

    let neuron_id_6m = read_state(|s| s.neuron_id_6m);
    let neuron_id_8y = read_state(|s| s.neuron_id_8y);

    process_neuron_spawn(neuron_id_6m, NeuronOrigin::NICPSixMonths).await;
    process_neuron_spawn(neuron_id_8y, NeuronOrigin::SnsGovernanceEightYears).await;
}

pub async fn process_witdhrawals_splitting() {
    if read_state(|s| s.withdrawal_to_split.is_empty()) {
        return;
    }

    let requests_ids = read_state(|s| s.withdrawal_to_split.clone());

    for withdrawal_id in requests_ids {
        let request = read_state(|s| {
            s.get_withdrawal_request(withdrawal_id)
                .expect("bug: request not found")
        });
        log!(
            INFO,
            "[process_witdhrawals_splitting] Trying to start split neuron for withdrawal id: {}",
            request.withdrawal_id
        );
        match split_neuron(SIX_MONTHS_NEURON_NONCE, request.icp_due.0).await {
            Ok(manage_neuron_response) => {
                if let Some(command_response) = manage_neuron_response.command.clone() {
                    if let CommandResponse::Split(spawn_response) = command_response {
                        if let Some(created_neuron_id) = spawn_response.created_neuron_id {
                            mutate_state(|s| {
                                process_event(
                                    s,
                                    EventType::SplitNeuron {
                                        withdrawal_id: request.withdrawal_id,
                                        neuron_id: created_neuron_id,
                                    },
                                );
                            });
                        } else {
                            log!(
                                INFO,
                                "[process_witdhrawals_splitting] bug: neuron with no id {:?}",
                                manage_neuron_response
                            );
                        }
                        continue;
                    }
                    log!(
                        INFO,
                        "[process_witdhrawals_splitting] failed to split neuron: {:?}",
                        manage_neuron_response
                    );
                } else {
                    log!(
                        INFO,
                        "[process_witdhrawals_splitting] bug: unexppected response {:?}",
                        manage_neuron_response
                    );
                }
            }
            Err(e) => log!(
                INFO,
                "[process_witdhrawals_splitting] failed to split neurons: {e}"
            ),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::sns_governance::CanisterRuntime;
    use crate::state::test::default_state;
    use crate::state::{replace_state, ICP_LEDGER_ID, SNS_GOVERNANCE_SUBACCOUNT};
    use crate::{
        dispatch_icp, read_state, self_canister_id, Account, NeuronOrigin, PendingTransfer, Unit,
        E8S,
    };
    use async_trait::async_trait;
    use candid::Principal;
    use ic_sns_governance::pb::v1::{ListNeurons, ListNeuronsResponse};
    use icrc_ledger_types::icrc1::transfer::TransferError;
    use mockall::mock;

    mock! {
        pub CanisterRuntime{}

         #[async_trait]
         impl CanisterRuntime for CanisterRuntime {
            async fn list_neurons(&self, args: ListNeurons) -> Result<ListNeuronsResponse, String>;

            async fn balance_of(
                &self,
                target: Account,
                ledger_canister_id: Principal,
            ) -> Result<u64, String>;

            async fn transfer_icp(&self, to: Principal, amount: u64) -> Result<u64, TransferError>;
         }
    }

    #[tokio::test]
    async fn should_schedule_icp_distribution() {
        let mut runtime = MockCanisterRuntime::new();
        replace_state(default_state());

        runtime
            .expect_balance_of()
            .withf(move |account, ledger_id| {
                account
                    == &Account {
                        owner: self_canister_id(),
                        subaccount: Some(NeuronOrigin::SnsGovernanceEightYears.to_subaccount()),
                    }
                    || account
                        == &Account {
                            owner: self_canister_id(),
                            subaccount: Some(NeuronOrigin::NICPSixMonths.to_subaccount()),
                        }
                        && ledger_id == &ICP_LEDGER_ID
            })
            .times(2)
            .return_const(Ok(100 * E8S));

        dispatch_icp(&runtime).await;

        runtime
            .expect_balance_of()
            .withf(move |account, ledger_id| {
                account
                    == &Account {
                        owner: self_canister_id(),
                        subaccount: Some(NeuronOrigin::SnsGovernanceEightYears.to_subaccount()),
                    }
                    || account
                        == &Account {
                            owner: self_canister_id(),
                            subaccount: Some(NeuronOrigin::NICPSixMonths.to_subaccount()),
                        }
                        && ledger_id == &ICP_LEDGER_ID
            })
            .times(2)
            .return_const(Ok(200 * E8S));

        dispatch_icp(&runtime).await;

        read_state(|s| {
            assert_eq!(s.pending_transfers.len(), 4);
            assert_eq!(
                s.pending_transfers[&0],
                PendingTransfer {
                    transfer_id: 0,
                    from_subaccount: Some(NeuronOrigin::SnsGovernanceEightYears.to_subaccount()),
                    memo: None,
                    amount: 180 * E8S,
                    receiver: s.get_6m_neuron_account(),
                    unit: Unit::ICP,
                }
            );
            assert_eq!(
                s.pending_transfers[&1],
                PendingTransfer {
                    transfer_id: 1,
                    from_subaccount: Some(NeuronOrigin::SnsGovernanceEightYears.to_subaccount()),
                    memo: None,
                    amount: 20 * E8S,
                    receiver: Account {
                        owner: self_canister_id(),
                        subaccount: Some(SNS_GOVERNANCE_SUBACCOUNT)
                    },
                    unit: Unit::ICP,
                }
            );
            assert_eq!(
                s.pending_transfers[&2],
                PendingTransfer {
                    transfer_id: 2,
                    from_subaccount: Some(NeuronOrigin::NICPSixMonths.to_subaccount()),
                    memo: None,
                    amount: 180 * E8S,
                    receiver: s.get_6m_neuron_account(),
                    unit: Unit::ICP,
                }
            );
            assert_eq!(
                s.pending_transfers[&3],
                PendingTransfer {
                    transfer_id: 3,
                    from_subaccount: Some(NeuronOrigin::NICPSixMonths.to_subaccount()),
                    memo: None,
                    amount: 20 * E8S,
                    receiver: Account {
                        owner: self_canister_id(),
                        subaccount: Some(SNS_GOVERNANCE_SUBACCOUNT)
                    },
                    unit: Unit::ICP,
                }
            );
        });
    }
}
