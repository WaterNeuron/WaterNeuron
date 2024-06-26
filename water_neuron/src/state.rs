use crate::nns_types::{NeuronId, ProposalId};
use crate::numeric::{nICP, ICP, WTN};
use crate::sns_distribution::compute_rewards;
use crate::tasks::TaskType;
use crate::{
    compute_neuron_staking_subaccount_bytes, self_canister_id, timestamp_nanos, InitArg,
    PendingTransfer, Unit, UpgradeArg, E8S, ONE_WEEK_NANOS,
};
use candid::{CandidType, Principal};
use icrc_ledger_types::icrc1::account::Account;
use minicbor::{Decode, Encode};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use strum_macros::EnumIter;

pub mod audit;
pub mod event;

thread_local! {
    static __STATE: RefCell<Option<State>> = RefCell::default();
}

pub const SIX_MONTHS_NEURON_NONCE: u64 = 0;
pub const EIGHT_YEARS_NEURON_NONCE: u64 = 1;

// "ryjl3-tyaaa-aaaaa-aaaba-cai"
pub const ICP_LEDGER_ID: Principal = Principal::from_slice(&[0, 0, 0, 0, 0, 0, 0, 2, 1, 1]);
// "rrkah-fqaaa-aaaaa-aaaaq-cai"
pub const NNS_GOVERNANCE_ID: Principal = Principal::from_slice(&[0, 0, 0, 0, 0, 0, 0, 1, 1, 1]);

pub const SNS_GOVERNANCE_SUBACCOUNT: [u8; 32] = [9; 32];

pub type TransferId = u64;
pub type WithdrawalId = u64;

#[derive(
    CandidType, Copy, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Encode, Decode, EnumIter,
)]
// A tag that tracks the origin of the neuron ICP are minted from.
pub enum NeuronOrigin {
    #[n(0)]
    SnsGovernanceEightYears = 0,
    #[n(1)]
    NICPSixMonths = 1,
}

fn u64_to_u8_array32(x: u64) -> [u8; 32] {
    let bytes = x.to_le_bytes();
    let mut result = [0u8; 32];
    result[..8].copy_from_slice(&bytes);
    result
}

impl NeuronOrigin {
    pub fn to_subaccount(self) -> [u8; 32] {
        // Use and offset to avoid colision.
        const OFFSET: u64 = 1234;
        u64_to_u8_array32(OFFSET + self as u64)
    }
}

impl fmt::Display for NeuronOrigin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NeuronOrigin::SnsGovernanceEightYears => write!(f, "SNS Governance 8 years neuron"),
            NeuronOrigin::NICPSixMonths => {
                write!(f, "Six-month nICP neuron")
            }
        }
    }
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum TransferStatus {
    Pending(PendingTransfer),
    Executed(ExecutedTransfer),
    Unknown,
}

#[derive(CandidType, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ExecutedTransfer {
    pub transfer: PendingTransfer,
    pub timestamp: u64,
    pub block_index: Option<u64>,
}

#[derive(CandidType, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct WithdrawalRequest {
    pub withdrawal_id: WithdrawalId,
    pub receiver: Account,
    pub nicp_burned: nICP,
    pub nicp_burn_index: u64,
    pub icp_due: ICP,
    pub neuron_id: Option<NeuronId>,
    pub timestamp: u64,
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct WithdrawalDetails {
    pub status: WithdrawalStatus,
    pub request: WithdrawalRequest,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Serialize, Deserialize)]
pub struct DisburseRequest {
    pub disburse_at: u64,
    pub receiver: Account,
    pub neuron_id: NeuronId,
}

#[derive(CandidType, Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct ExchangeRate {
    pub short_term_neuron_stake: ICP,
    pub nicp_supply: nICP,
    pub timestamp: u64,
}

#[derive(CandidType, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WithdrawalStatus {
    WaitingToSplitNeuron,
    WaitingToStartDissolving { neuron_id: NeuronId },
    WaitingDissolvement { neuron_id: NeuronId },
    ConversionDone { transfer_block_height: u64 },
    NotFound,
}

impl fmt::Display for WithdrawalStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WithdrawalStatus::WaitingToSplitNeuron => write!(f, "Waiting to split neuron"),
            WithdrawalStatus::WaitingToStartDissolving { neuron_id } => {
                write!(f, "Waiting to start dissolving {:?}", neuron_id)
            }
            WithdrawalStatus::WaitingDissolvement { neuron_id } => {
                write!(f, "Waiting dissolvement {:?}", neuron_id)
            }
            WithdrawalStatus::ConversionDone {
                transfer_block_height,
            } => write!(f, "Neuron Disbursed at index: {transfer_block_height}"),
            WithdrawalStatus::NotFound => write!(f, "Neuron Not Found"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    pub inception_ts: u64,
    pub total_icp_deposited: ICP,
    pub total_circulating_nicp: nICP,
    pub governance_fee_share: Decimal,

    pub transfer_id: TransferId,
    pub withdrawal_id: WithdrawalId,

    // NNS Proposal Id to SNS Proposals ID
    // and deadline_timestamp_seconds of the NNS proposal
    pub proposals: BTreeMap<ProposalId, ProposalId>,

    // Airdrop Map
    pub airdrop: BTreeMap<Principal, WTN>,

    // nICP -> ICP
    pub withdrawal_to_split: BTreeSet<WithdrawalId>,
    pub withdrawal_to_start_dissolving: BTreeSet<WithdrawalId>,
    pub withdrawal_to_disburse: BTreeSet<WithdrawalId>,
    pub withdrawal_finalized: BTreeMap<WithdrawalId, u64>,
    pub withdrawal_id_to_request: BTreeMap<WithdrawalId, WithdrawalRequest>,

    // Neurons To Disburse
    pub to_disburse: BTreeMap<NeuronId, DisburseRequest>,
    // History of maturity neurons disbursed.
    pub maturity_neuron_to_block_indicies: BTreeMap<NeuronId, u64>,

    // Transfer queues
    pub pending_transfers: BTreeMap<TransferId, PendingTransfer>,
    pub transfer_executed: BTreeMap<TransferId, ExecutedTransfer>,

    // Maps for tracking purposes.
    pub principal_to_deposit: BTreeMap<Principal, Vec<TransferId>>,
    pub principal_to_withdrawal: BTreeMap<Principal, Vec<WithdrawalId>>,

    // Neurons
    pub neuron_id_6m: Option<NeuronId>,
    pub main_neuron_6m_staked: ICP,
    pub tracked_6m_stake: ICP,
    pub neuron_id_8y: Option<NeuronId>,
    pub main_neuron_8y_stake: ICP,

    // Some canister ids.
    pub nicp_ledger_id: Principal,
    pub sns_governance_id: Option<Principal>,
    pub wtn_ledger_id: Option<Principal>,

    // Guards
    pub principal_guards: BTreeSet<Principal>,
    pub active_tasks: BTreeSet<TaskType>,
}

impl State {
    pub fn from_init_args(init_arg: InitArg, inception_ts: u64) -> Self {
        Self {
            inception_ts,
            airdrop: BTreeMap::default(),
            proposals: BTreeMap::default(),
            governance_fee_share: dec!(0.1),
            total_circulating_nicp: nICP::ZERO,
            total_icp_deposited: ICP::ZERO,
            tracked_6m_stake: ICP::ZERO,
            to_disburse: BTreeMap::default(),
            withdrawal_to_split: BTreeSet::default(),
            withdrawal_to_start_dissolving: Default::default(),
            withdrawal_to_disburse: Default::default(),
            maturity_neuron_to_block_indicies: Default::default(),
            withdrawal_finalized: Default::default(),
            withdrawal_id_to_request: BTreeMap::default(),
            principal_to_deposit: BTreeMap::default(),
            principal_to_withdrawal: BTreeMap::default(),
            transfer_id: 0,
            withdrawal_id: 0,
            pending_transfers: BTreeMap::default(),
            transfer_executed: BTreeMap::default(),
            neuron_id_6m: None,
            neuron_id_8y: None,
            main_neuron_6m_staked: ICP::ZERO,
            main_neuron_8y_stake: ICP::ZERO,
            nicp_ledger_id: init_arg.nicp_ledger_id,
            sns_governance_id: None,
            wtn_ledger_id: None,
            principal_guards: BTreeSet::default(),
            active_tasks: BTreeSet::default(),
        }
    }

    pub fn get_withdrawal_request(&self, withdrawal_id: WithdrawalId) -> Option<WithdrawalRequest> {
        self.withdrawal_id_to_request.get(&withdrawal_id).cloned()
    }

    pub fn get_withdrawal_requests_to_dissolve(&self) -> Vec<WithdrawalRequest> {
        let mut res: Vec<WithdrawalRequest> = vec![];
        for withdrawal_id in self.withdrawal_to_start_dissolving.iter() {
            if let Some(req) = self.get_withdrawal_request(*withdrawal_id) {
                res.push(req.clone());
            }
        }
        res
    }

    pub fn neuron_id_to_withdrawal_id(&self, neuron_id: NeuronId) -> Option<WithdrawalId> {
        for req in self.withdrawal_id_to_request.values() {
            if let Some(req_neuron_id) = req.neuron_id {
                if neuron_id == req_neuron_id {
                    return Some(req.withdrawal_id);
                }
            }
        }
        None
    }

    pub fn get_withdrawal_status(&self, withdrawal_id: WithdrawalId) -> WithdrawalStatus {
        if self.withdrawal_to_split.get(&withdrawal_id).is_some() {
            return WithdrawalStatus::WaitingToSplitNeuron;
        }

        if self
            .withdrawal_to_start_dissolving
            .iter()
            .any(|&id| id == withdrawal_id)
        {
            return WithdrawalStatus::WaitingToStartDissolving {
                neuron_id: self
                    .get_withdrawal_request(withdrawal_id)
                    .expect("bug: we should have the request associated to this id")
                    .neuron_id
                    .expect("bug: at this point the neuron_id field should be set"),
            };
        }

        if self.withdrawal_to_disburse.get(&withdrawal_id).is_some() {
            return WithdrawalStatus::WaitingDissolvement {
                neuron_id: self
                    .get_withdrawal_request(withdrawal_id)
                    .expect("bug: we should have the request associated to this id")
                    .neuron_id
                    .expect("bug: at this point the neuron_id field should be set"),
            };
        }

        if let Some(block_index) = self.withdrawal_finalized.get(&withdrawal_id) {
            return WithdrawalStatus::ConversionDone {
                transfer_block_height: *block_index,
            };
        }
        WithdrawalStatus::NotFound
    }

    pub fn get_transfer_status(&self, id: u64) -> TransferStatus {
        if let Some(transfer) = self.pending_transfers.get(&id) {
            return TransferStatus::Pending(transfer.clone());
        }
        if let Some(transfer) = self.transfer_executed.get(&id) {
            return TransferStatus::Executed(transfer.clone());
        }
        TransferStatus::Unknown
    }

    pub fn is_processing_icp_transfer(&self) -> bool {
        self.pending_transfers
            .values()
            .any(|transfer| transfer.unit == Unit::ICP)
    }

    /// We should never the 6 months and 8 years main neurons.
    pub fn is_neuron_allowed_to_dissolve(&self, neuron_id: NeuronId) -> bool {
        self.neuron_id_6m != Some(neuron_id) && self.neuron_id_8y != Some(neuron_id)
    }

    pub fn get_icp_to_ncip_exchange_rate(&self) -> Decimal {
        // If we didn't mint any nicp yet the exchange rate is 1.
        if self.total_circulating_nicp == nICP::ZERO || self.tracked_6m_stake == ICP::ZERO {
            return Decimal::ONE;
        }
        // For the first week the exchange rate is to 1.
        if timestamp_nanos() < self.inception_ts + ONE_WEEK_NANOS {
            return Decimal::ONE;
        }

        Decimal::from(self.total_circulating_nicp.0) / Decimal::from(self.tracked_6m_stake.0)
    }

    pub fn get_icp_to_ncip_exchange_rate_e8s(&self) -> u64 {
        let mut exchange_rate = self.get_icp_to_ncip_exchange_rate();
        exchange_rate.rescale(8_u32);
        exchange_rate.mantissa() as u64
    }

    pub fn compute_governance_share_e8s(&self, balance: u64) -> u64 {
        let mut governance_share =
            Decimal::from(balance) / Decimal::from(E8S) * self.governance_fee_share;
        governance_share.rescale(8);
        governance_share.mantissa() as u64
    }

    pub fn convert_icp_to_nicp(&self, amount: ICP) -> nICP {
        let mut result =
            Decimal::from(amount.0) / Decimal::from(E8S) * self.get_icp_to_ncip_exchange_rate();
        result.rescale(8_u32);
        nICP::from_e8s(result.mantissa() as u64)
    }

    pub fn convert_nicp_to_icp(&self, amount: nICP) -> ICP {
        let mut result =
            Decimal::from(amount.0) / Decimal::from(E8S) / self.get_icp_to_ncip_exchange_rate();
        result.rescale(8_u32);
        ICP::from_e8s(result.mantissa() as u64)
    }

    pub fn increment_transfer_id(&mut self) -> u64 {
        let transfer_id = self.transfer_id;
        self.transfer_id += 1;
        transfer_id
    }

    pub fn increment_withdrawal_id(&mut self) -> u64 {
        let withdrawal_id = self.withdrawal_id;
        self.withdrawal_id += 1;
        withdrawal_id
    }

    pub fn record_upgrade(&mut self, upgrade_arg: UpgradeArg) {
        if let Some(sns_governance_id) = upgrade_arg.sns_governance_id {
            self.sns_governance_id = Some(sns_governance_id);
        }
        if let Some(wtn_ledger_id) = upgrade_arg.wtn_ledger_id {
            self.wtn_ledger_id = Some(wtn_ledger_id);
        }
    }

    pub fn record_transfer_executed(
        &mut self,
        transfer_id: TransferId,
        block_index: Option<u64>,
        timestamp: u64,
    ) {
        match self.pending_transfers.remove(&transfer_id) {
            Some(removed_tansfer) => {
                self.transfer_executed.insert(
                    transfer_id,
                    ExecutedTransfer {
                        transfer: removed_tansfer,
                        timestamp,
                        block_index,
                    },
                );
            }
            None => ic_cdk::trap(&format!("transfer with id {transfer_id} not found")),
        }
    }

    pub fn record_icp_deposit(&mut self, receiver: Account, amount: ICP, block_index: u64) {
        let nicp_to_mint = self.convert_icp_to_nicp(amount);
        self.total_circulating_nicp += nicp_to_mint;
        let rewards = compute_rewards(self.total_icp_deposited, amount);
        if rewards > WTN::ZERO {
            self.airdrop
                .entry(receiver.owner)
                .and_modify(|e| *e += rewards)
                .or_insert(rewards);
        }
        self.total_icp_deposited += amount;
        self.tracked_6m_stake += amount;
        let transfer_id = self.increment_transfer_id();
        assert_eq!(
            self.pending_transfers.insert(
                transfer_id,
                PendingTransfer {
                    transfer_id,
                    from_subaccount: None,
                    amount: nicp_to_mint.0,
                    receiver,
                    unit: Unit::NICP,
                    memo: Some(block_index)
                }
            ),
            None
        );
        self.principal_to_deposit
            .entry(receiver.owner)
            .and_modify(|deposits| deposits.push(transfer_id))
            .or_insert(vec![transfer_id]);
    }

    pub fn record_claimed_airdrop(&mut self, caller: Principal) {
        assert!(self.airdrop.remove(&caller).is_some());
    }

    pub fn record_pending_neuron_transfer(&mut self, amount: u64, receiver: Account) {
        let transfer_id = self.increment_transfer_id();
        let unit = Unit::WTN;
        assert!(amount >= unit.fee());

        assert_eq!(
            self.pending_transfers.insert(
                transfer_id,
                PendingTransfer {
                    transfer_id,
                    from_subaccount: None,
                    amount,
                    receiver,
                    unit,
                    memo: None
                }
            ),
            None
        );
    }

    pub fn record_icp_pending_transfer(
        &mut self,
        from_subaccount: [u8; 32],
        receiver: impl Into<Account>,
        amount: ICP,
    ) {
        let transfer_id = self.increment_transfer_id();
        let unit = Unit::ICP;
        assert!(amount.0 >= unit.fee());

        assert_eq!(
            self.pending_transfers.insert(
                transfer_id,
                PendingTransfer {
                    transfer_id,
                    from_subaccount: Some(from_subaccount),
                    amount: amount.0,
                    receiver: receiver.into(),
                    unit,
                    memo: None
                }
            ),
            None
        );
    }

    pub fn record_nicp_withdrawal(
        &mut self,
        receiver: Account,
        nicp_burned: nICP,
        nicp_burn_index: u64,
        timestamp: u64,
    ) -> u64 {
        let icp_due = self.convert_nicp_to_icp(nicp_burned);
        self.tracked_6m_stake = self.tracked_6m_stake.checked_sub(icp_due).unwrap();
        self.total_circulating_nicp = self
            .total_circulating_nicp
            .checked_sub(nicp_burned)
            .unwrap_or_else(|| {
                panic!(
                    "bug: trying to burn more nicp {nicp_burned} than nicp tracked {}",
                    self.total_circulating_nicp
                )
            });
        let withdrawal_id = self.increment_withdrawal_id();
        self.principal_to_withdrawal
            .entry(receiver.owner)
            .and_modify(|ids| ids.push(withdrawal_id))
            .or_insert(vec![withdrawal_id]);
        assert_eq!(
            self.withdrawal_id_to_request.insert(
                withdrawal_id,
                WithdrawalRequest {
                    withdrawal_id,
                    receiver,
                    nicp_burned,
                    nicp_burn_index,
                    icp_due,
                    neuron_id: None,
                    timestamp
                }
            ),
            None
        );
        assert!(self.withdrawal_to_split.insert(withdrawal_id));
        withdrawal_id
    }

    pub fn record_neuron_split(&mut self, withdrawal_id: u64, neuron_id: NeuronId) {
        assert!(self.withdrawal_to_split.remove(&withdrawal_id));
        self.withdrawal_id_to_request
            .entry(withdrawal_id)
            .and_modify(|n| n.neuron_id = Some(neuron_id));
        assert!(self.withdrawal_to_start_dissolving.insert(withdrawal_id),);
    }

    pub fn record_started_to_dissolve_neuron(
        &mut self,
        withdrawal_id: WithdrawalId,
        disburse_at: u64,
    ) {
        let request = self
            .withdrawal_id_to_request
            .get(&withdrawal_id)
            .expect("bug: withdrawal id to request should be set");
        let neuron_id = request
            .neuron_id
            .expect("bug: neuron id should be set at this point");
        assert_eq!(
            self.to_disburse.insert(
                neuron_id,
                DisburseRequest {
                    disburse_at,
                    receiver: request.receiver,
                    neuron_id,
                },
            ),
            None
        );
        assert!(self.withdrawal_to_start_dissolving.remove(&withdrawal_id),);
        assert!(self.withdrawal_to_disburse.insert(withdrawal_id));
    }

    pub fn record_neuron_disbursed(&mut self, withdrawal_id: WithdrawalId, block_index: u64) {
        assert!(self.withdrawal_to_disburse.remove(&withdrawal_id));
        let request = self
            .withdrawal_id_to_request
            .get(&withdrawal_id)
            .expect("bug: withdrawal id to request should be set");
        let neuron_id = request
            .neuron_id
            .expect("bug: neuron id should be set at this point");
        assert!(self
            .withdrawal_finalized
            .insert(withdrawal_id, block_index)
            .is_none());
        assert!(self.to_disburse.remove(&neuron_id).is_some());
    }

    pub fn record_disbursed_maturity_neuron(&mut self, neuron_id: NeuronId, block_index: u64) {
        assert!(self.to_disburse.remove(&neuron_id).is_some());
        self.maturity_neuron_to_block_indicies
            .insert(neuron_id, block_index);
    }

    pub fn record_maturity_neuron(
        &mut self,
        neuron_id: NeuronId,
        neuron_kind: NeuronOrigin,
        disburse_at: u64,
    ) {
        assert_eq!(
            self.to_disburse.insert(
                neuron_id,
                DisburseRequest {
                    disburse_at,
                    receiver: Account {
                        owner: ic_cdk::id(),
                        subaccount: Some(neuron_kind.to_subaccount()),
                    },
                    neuron_id,
                }
            ),
            None
        );
    }

    pub fn record_6m_neuron_id(&mut self, neuron_id: NeuronId) {
        self.neuron_id_6m = Some(neuron_id);
    }

    pub fn record_8y_neuron_id(&mut self, neuron_id: NeuronId) {
        self.neuron_id_8y = Some(neuron_id);
    }

    pub fn get_6m_neuron_account(&self) -> Account {
        Account {
            owner: NNS_GOVERNANCE_ID,
            subaccount: Some(compute_neuron_staking_subaccount_bytes(
                self_canister_id(),
                SIX_MONTHS_NEURON_NONCE,
            )),
        }
    }

    pub fn get_8y_neuron_account(&self) -> Account {
        Account {
            owner: NNS_GOVERNANCE_ID,
            subaccount: Some(compute_neuron_staking_subaccount_bytes(
                self_canister_id(),
                EIGHT_YEARS_NEURON_NONCE,
            )),
        }
    }

    // Account where we send the SNS rewards.
    pub fn get_sns_account(&self) -> Account {
        Account {
            owner: self_canister_id(),
            subaccount: Some(SNS_GOVERNANCE_SUBACCOUNT),
        }
    }

    pub fn is_equivalent_to(&self, other: &Self) -> Result<(), String> {
        use ic_utils_ensure::ensure_eq;

        ensure_eq!(
            self.inception_ts,
            other.inception_ts,
            "inception_ts do not match"
        );
        ensure_eq!(
            self.tracked_6m_stake,
            other.tracked_6m_stake,
            "tracked_6m_stake do not match"
        );
        ensure_eq!(
            self.total_circulating_nicp,
            other.total_circulating_nicp,
            "total_circulating_nicp do not match"
        );
        ensure_eq!(
            self.transfer_id,
            other.transfer_id,
            "transfer_id do not match"
        );
        ensure_eq!(
            self.withdrawal_id,
            other.withdrawal_id,
            "withdrawal_id do not match"
        );

        ensure_eq!(
            self.withdrawal_to_split,
            other.withdrawal_to_split,
            "withdrawal_to_split do not match"
        );
        ensure_eq!(
            self.withdrawal_to_start_dissolving,
            other.withdrawal_to_start_dissolving,
            "withdrawal_to_start_dissolving do not match"
        );
        ensure_eq!(
            self.withdrawal_to_disburse,
            other.withdrawal_to_disburse,
            "withdrawal_to_disburse do not match"
        );
        ensure_eq!(
            self.withdrawal_finalized,
            other.withdrawal_finalized,
            "withdrawal_finalized do not match"
        );
        ensure_eq!(
            self.withdrawal_id_to_request,
            other.withdrawal_id_to_request,
            "withdrawal_id_to_request do not match"
        );
        ensure_eq!(
            self.to_disburse,
            other.to_disburse,
            "to_disburse do not match"
        );
        ensure_eq!(
            self.pending_transfers,
            other.pending_transfers,
            "pending_transfers do not match"
        );
        ensure_eq!(
            self.principal_to_withdrawal,
            other.principal_to_withdrawal,
            "principal_to_withdrawal do not match"
        );

        ensure_eq!(
            self.principal_to_deposit,
            other.principal_to_deposit,
            "principal_to_deposit do not match"
        );
        ensure_eq!(
            self.neuron_id_6m,
            other.neuron_id_6m,
            "neuron_id_6m do not match"
        );
        ensure_eq!(
            self.neuron_id_8y,
            other.neuron_id_8y,
            "neuron_id_8y do not match"
        );
        ensure_eq!(
            self.nicp_ledger_id,
            other.nicp_ledger_id,
            "nicp_ledger_id do not match"
        );
        ensure_eq!(
            self.sns_governance_id,
            other.sns_governance_id,
            "sns_governance_id do not match"
        );
        ensure_eq!(
            self.wtn_ledger_id,
            other.wtn_ledger_id,
            "wtn_ledger_id do not match"
        );
        ensure_eq!(self.airdrop, other.airdrop, "airdrop do not match");
        ensure_eq!(self.proposals, other.proposals, "proposals do not match");

        Ok(())
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