use crate::nns_types::{NeuronId, ProposalId};
use crate::numeric::{nICP, ICP, WTN};
use crate::sns_distribution::compute_rewards;
use crate::tasks::TaskType;
use crate::{
    compute_neuron_staking_subaccount_bytes, self_canister_id, InitArg, PendingTransfer, Unit,
    UpgradeArg, DEFAULT_LEDGER_FEE, E8S, timestamp_nanos
};
use candid::{CandidType, Principal};
use icrc_ledger_types::icrc1::account::Account;
use minicbor::{Decode, Encode};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub mod audit;
pub mod event;
#[cfg(test)]
pub mod tests;

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
        // Use an offset to avoid colision.
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

#[derive(CandidType, Serialize, Deserialize, Debug)]
pub struct WithdrawalDetails {
    pub status: WithdrawalStatus,
    pub request: WithdrawalRequest,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Serialize, Deserialize)]
pub struct DisburseRequest {
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
    Cancelled,
}

impl fmt::Display for WithdrawalStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WithdrawalStatus::WaitingToSplitNeuron => write!(f, "Waiting to split neuron"),
            WithdrawalStatus::WaitingToStartDissolving { neuron_id } => {
                write!(f, "Waiting to start dissolving of {}", neuron_id.id)
            }
            WithdrawalStatus::WaitingDissolvement { neuron_id } => {
                write!(f, "Waiting dissolvement of {}", neuron_id.id)
            }
            WithdrawalStatus::ConversionDone {
                transfer_block_height,
            } => write!(f, "Neuron Disbursed at index: {transfer_block_height}"),
            WithdrawalStatus::NotFound => write!(f, "Neuron Not Found"),
            WithdrawalStatus::Cancelled => write!(f, "Withdrawal Cancelled"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    pub total_icp_deposited: ICP,
    pub total_circulating_nicp: nICP,
    pub governance_fee_share_percent: u64,

    pub transfer_id: TransferId,
    pub withdrawal_id: WithdrawalId,

    // NNS Proposal Id to SNS Proposals ID
    pub proposals: BTreeMap<ProposalId, ProposalId>,
    pub voted_proposals: BTreeSet<ProposalId>,
    pub last_nns_proposal_seen: ProposalId,

    // Airdrop Map
    pub airdrop: BTreeMap<Principal, WTN>,

    // nICP -> ICP
    pub withdrawal_to_split: BTreeSet<WithdrawalId>,
    pub withdrawal_to_start_dissolving: BTreeSet<WithdrawalId>,
    pub withdrawal_to_disburse: BTreeSet<WithdrawalId>,
    pub withdrawal_finalized: BTreeMap<WithdrawalId, u64>,
    pub withdrawal_id_to_request: BTreeMap<WithdrawalId, WithdrawalRequest>,
    pub neuron_id_to_withdrawal_id: BTreeMap<NeuronId, WithdrawalId>,

    // Cancel Withdrawal
    pub withdrawal_cancelled: BTreeSet<WithdrawalId>,

    // Neurons To Disburse
    pub to_disburse: BTreeMap<NeuronId, DisburseRequest>,

    // History of maturity neurons disbursed
    pub maturity_neuron_to_block_indicies: BTreeMap<NeuronId, u64>,

    // Transfer queues
    pub pending_transfers: BTreeMap<TransferId, PendingTransfer>,
    pub transfer_executed: BTreeMap<TransferId, ExecutedTransfer>,

    // Maps for tracking purposes.
    pub account_to_deposits: BTreeMap<Account, Vec<TransferId>>,
    pub account_to_withdrawals: BTreeMap<Account, Vec<WithdrawalId>>,

    // Neurons
    pub neuron_id_6m: Option<NeuronId>,
    pub main_neuron_6m_staked: ICP,
    pub tracked_6m_stake: ICP,
    pub neuron_id_8y: Option<NeuronId>,
    pub main_neuron_8y_stake: ICP,

    // Some canister ids.
    pub nicp_ledger_id: Principal,
    pub wtn_governance_id: Principal,
    pub wtn_ledger_id: Principal,

    // Guards
    pub principal_guards: BTreeSet<Principal>,
    pub active_tasks: BTreeSet<TaskType>,

    // ICP Distribution
    pub latest_distribution_icp_per_vp: Option<f64>,
    pub last_distribution_ts: u64,
}

impl State {
    pub fn from_init_args(init_arg: InitArg) -> Self {
        Self {
            airdrop: BTreeMap::default(),
            neuron_id_to_withdrawal_id: BTreeMap::default(),
            proposals: BTreeMap::default(),
            governance_fee_share_percent: 10,
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
            withdrawal_cancelled: BTreeSet::default(),
            account_to_deposits: BTreeMap::default(),
            account_to_withdrawals: BTreeMap::default(),
            transfer_id: 0,
            withdrawal_id: 0,
            voted_proposals: BTreeSet::default(),
            pending_transfers: BTreeMap::default(),
            transfer_executed: BTreeMap::default(),
            neuron_id_6m: None,
            neuron_id_8y: None,
            main_neuron_6m_staked: ICP::ZERO,
            main_neuron_8y_stake: ICP::ZERO,
            nicp_ledger_id: init_arg.nicp_ledger_id,
            wtn_governance_id: init_arg.wtn_governance_id,
            wtn_ledger_id: init_arg.wtn_ledger_id,
            principal_guards: BTreeSet::default(),
            active_tasks: BTreeSet::default(),
            latest_distribution_icp_per_vp: None,
            last_nns_proposal_seen: Default::default(),
            last_distribution_ts: timestamp_nanos(),
        }
    }

    pub fn get_withdrawal_request(&self, withdrawal_id: WithdrawalId) -> Option<WithdrawalRequest> {
        self.withdrawal_id_to_request.get(&withdrawal_id).cloned()
    }

    pub fn get_withdrawal_request_ids_to_dissolve(&self) -> Vec<NeuronId> {
        let mut res: Vec<NeuronId> = vec![];
        for withdrawal_id in self.withdrawal_to_start_dissolving.iter() {
            if let Some(req) = self.get_withdrawal_request(*withdrawal_id) {
                res.push(req.neuron_id.expect("bug: neuron_id should be set"));
            }
        }
        res
    }

    pub fn neuron_id_to_withdrawal_id(&self, neuron_id: NeuronId) -> Option<WithdrawalId> {
        self.neuron_id_to_withdrawal_id.get(&neuron_id).copied()
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

        if self.withdrawal_cancelled.get(&withdrawal_id).is_some() {
            return WithdrawalStatus::Cancelled;
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

    pub fn is_processing_icp_transfer_from_neuron(&self) -> bool {
        self.pending_transfers.values().any(|transfer| {
            NeuronOrigin::iter()
                .any(|origin| transfer.from_subaccount == Some(origin.to_subaccount()))
        })
    }

    pub fn is_processing_icp_transfer_from_sns_subaccount(&self) -> bool {
        self.pending_transfers
            .values()
            .any(|transfer| transfer.from_subaccount == Some(SNS_GOVERNANCE_SUBACCOUNT))
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

        Decimal::from(self.total_circulating_nicp.0) / Decimal::from(self.tracked_6m_stake.0)
    }

    pub fn get_icp_to_ncip_exchange_rate_e8s(&self) -> u64 {
        let mut exchange_rate = self.get_icp_to_ncip_exchange_rate();
        exchange_rate.rescale(8_u32);
        exchange_rate.mantissa() as u64
    }

    pub fn compute_governance_share_e8s(&self, balance: u64) -> u64 {
        balance
            .checked_mul(self.governance_fee_share_percent)
            .unwrap()
            / 100
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
        if let Some(governance_fee_share_percent) = upgrade_arg.governance_fee_share_percent {
            self.governance_fee_share_percent = governance_fee_share_percent;
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
        self.account_to_deposits
            .entry(receiver)
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
        memo: Option<u64>,
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
                    memo
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
        self.account_to_withdrawals
            .entry(receiver)
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
        assert!(self.withdrawal_to_start_dissolving.insert(withdrawal_id));
        assert!(self
            .neuron_id_to_withdrawal_id
            .insert(neuron_id, withdrawal_id)
            .is_none());
    }

    pub fn record_neuron_merge(&mut self, neuron_id: NeuronId) {
        let withdrawal_id: &u64 = self.neuron_id_to_withdrawal_id.get(&neuron_id).unwrap();
        assert!(
            self.withdrawal_to_start_dissolving.remove(withdrawal_id)
                || (self.withdrawal_to_disburse.remove(withdrawal_id)
                    && self.to_disburse.remove(&neuron_id).is_some())
        );

        let withdrawal_request = self
            .withdrawal_id_to_request
            .get(withdrawal_id)
            .unwrap()
            .clone();

        self.withdrawal_cancelled.insert(*withdrawal_id);
        assert!(self.neuron_id_to_withdrawal_id.remove(&neuron_id).is_some());

        // Merging the neurons costs two times the ICP ledger transaction fee.
        // Once to calculate the effects of merging two neurons (step 1).
        // Once to operate the transaction of the source neuron stake to the target neuron (step 5).
        // Here is the link to the according merge_neurons function used:
        // https://github.com/dfinity/ic/blob/714c85c6a4245fb5b39e76f5c8003e6d90e49c4d/rs/nns/governance/src/governance.rs#L2780
        let icp_stake_e8s = withdrawal_request
            .icp_due
            .checked_sub(ICP::from_e8s(2 * DEFAULT_LEDGER_FEE))
            .expect("ICP due should be greater than 10.");
        let nicp_stake_value_e8s = self.convert_icp_to_nicp(icp_stake_e8s);

        // 0.5% fee when a withdrawal is cancelled.
        let nicp_fee = nICP::from_e8s(nicp_stake_value_e8s.0.checked_div(200).unwrap());
        let nicp_to_mint = nicp_stake_value_e8s.checked_sub(nicp_fee).unwrap();
        self.total_circulating_nicp += nicp_to_mint;

        self.tracked_6m_stake += icp_stake_e8s;

        let transfer_id = self.increment_transfer_id();
        assert_eq!(
            self.pending_transfers.insert(
                transfer_id,
                PendingTransfer {
                    transfer_id,
                    from_subaccount: None,
                    amount: nicp_to_mint.0,
                    receiver: withdrawal_request.receiver,
                    unit: Unit::NICP,
                    memo: None
                }
            ),
            None
        );
        self.account_to_deposits
            .entry(withdrawal_request.receiver)
            .and_modify(|deposits| deposits.push(transfer_id))
            .or_insert(vec![transfer_id]);
    }

    pub fn record_started_to_dissolve_neuron(&mut self, withdrawal_id: WithdrawalId) {
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
        assert!(self.neuron_id_to_withdrawal_id.remove(&neuron_id).is_some());
    }

    pub fn record_disbursed_maturity_neuron(&mut self, neuron_id: NeuronId, block_index: u64) {
        assert!(self.to_disburse.remove(&neuron_id).is_some());
        self.maturity_neuron_to_block_indicies
            .insert(neuron_id, block_index);
    }

    pub fn record_maturity_neuron(&mut self, neuron_id: NeuronId, neuron_kind: NeuronOrigin) {
        assert_eq!(
            self.to_disburse.insert(
                neuron_id,
                DisburseRequest {
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
            self.account_to_withdrawals,
            other.account_to_withdrawals,
            "account_to_withdrawals do not match"
        );

        ensure_eq!(
            self.account_to_deposits,
            other.account_to_deposits,
            "account_to_deposits do not match"
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
            self.wtn_governance_id,
            other.wtn_governance_id,
            "wtn_governance_id do not match"
        );
        ensure_eq!(
            self.wtn_ledger_id,
            other.wtn_ledger_id,
            "wtn_ledger_id do not match"
        );
        ensure_eq!(self.airdrop, other.airdrop, "airdrop do not match");
        ensure_eq!(self.proposals, other.proposals, "proposals do not match");
        ensure_eq!(
            self.get_icp_to_ncip_exchange_rate_e8s(),
            other.get_icp_to_ncip_exchange_rate_e8s(),
            "exchange rate are the same"
        );
        ensure_eq!(
            self.get_icp_to_ncip_exchange_rate(),
            other.get_icp_to_ncip_exchange_rate(),
            "exchange rate are the same"
        );

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

#[cfg(test)]
pub mod test {
    use crate::state::{State, WithdrawalStatus, ICP_LEDGER_ID, NNS_GOVERNANCE_ID, WTN};
    use crate::{nICP, InitArg, NeuronId, NeuronOrigin, PendingTransfer, Unit, E8S, ICP};
    use candid::Principal;
    use std::str::FromStr;

    pub fn default_state() -> State {
        let mut state = State::from_init_args(InitArg {
            wtn_ledger_id: Principal::anonymous(),
            wtn_governance_id: Principal::anonymous(),
            nicp_ledger_id: Principal::anonymous(),
        });
        state.governance_fee_share_percent = 10;
        state
    }

    fn get_neuron_id(id: u64) -> NeuronId {
        NeuronId { id }
    }

    #[test]
    fn should_have_expected_principal() {
        const NNS_GOVERNANCE_ID_STR: &str = "rrkah-fqaaa-aaaaa-aaaaq-cai";
        const ICP_LEDGER_ID_STR: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";

        assert_eq!(
            ICP_LEDGER_ID,
            Principal::from_text(ICP_LEDGER_ID_STR).unwrap()
        );
        assert_eq!(
            NNS_GOVERNANCE_ID,
            Principal::from_text(NNS_GOVERNANCE_ID_STR).unwrap()
        );
    }

    #[test]
    fn should_have_expected_subaccount() {
        assert_eq!(
            NeuronOrigin::SnsGovernanceEightYears.to_subaccount(),
            [
                210, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0
            ]
        );
        assert_eq!(
            NeuronOrigin::NICPSixMonths.to_subaccount(),
            [
                211, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0
            ]
        );
        assert_ne!(
            NeuronOrigin::NICPSixMonths.to_subaccount(),
            NeuronOrigin::SnsGovernanceEightYears.to_subaccount()
        );
    }

    #[test]
    fn should_never_dissolve_6m_and_8y_main_neurons() {
        let mut state = default_state();

        let neuron_6m = get_neuron_id(0_u64);
        let neuron_8y = get_neuron_id(1_u64);

        state.record_6m_neuron_id(neuron_6m);
        state.record_8y_neuron_id(neuron_8y);

        let neurond_id = get_neuron_id(3_u64);
        assert!(state.is_neuron_allowed_to_dissolve(neurond_id));
        assert!(!state.is_neuron_allowed_to_dissolve(neuron_6m));
        assert!(!state.is_neuron_allowed_to_dissolve(neuron_8y));
    }

    #[test]
    fn icp_to_nicp_conversion() {
        let mut state = default_state();
        state.tracked_6m_stake = ICP::from_e8s(200_020_001);
        state.total_circulating_nicp = nICP::from_e8s(99_990_000);
        assert_eq!(
            state.convert_icp_to_nicp(ICP::from_e8s(99_990_000)),
            nICP::from_e8s(49_985_002)
        );
    }

    #[test]
    fn nicp_to_icp_conversion() {
        let mut state = default_state();
        state.tracked_6m_stake = ICP::from_unscaled(10);
        state.total_circulating_nicp = nICP::from_unscaled(5);
        assert_eq!(state.convert_nicp_to_icp(nICP::ONE), ICP::TWO);
    }

    #[test]
    fn should_increment() {
        let mut state = default_state();

        assert_eq!(state.increment_transfer_id(), 0);
        let res = state.increment_transfer_id();
        assert_eq!(state.increment_transfer_id() - res, 1);

        assert_eq!(state.increment_withdrawal_id(), 0);
        let res = state.increment_withdrawal_id();
        assert_eq!(state.increment_withdrawal_id() - res, 1);
    }

    #[test]
    fn rewards_should_be_as_expected() {
        let mut state = default_state();

        let caller = Principal::from_str("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();
        state.record_icp_deposit(caller.into(), ICP::from_unscaled(80_001), 0);
        assert_eq!(
            state.pending_transfers.get(&0).unwrap(),
            &PendingTransfer {
                transfer_id: 0,
                from_subaccount: None,
                memo: Some(0),
                amount: 80_001 * E8S,
                receiver: caller.into(),
                unit: Unit::NICP,
            }
        );
        assert_eq!(
            state.airdrop.get(&caller).unwrap(),
            &WTN::from_unscaled(8 * 80_000 + 4)
        );
        state.record_claimed_airdrop(caller);
        assert_eq!(state.airdrop.get(&caller), None);
    }

    #[test]
    fn withdrawal_flow() {
        let mut state = default_state();
        let caller = Principal::from_str("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();
        let withdrawal_id = 0_u64;
        let neuron_id = NeuronId { id: 0 };

        state.record_icp_deposit(caller.into(), ICP::from_unscaled(10), 0_64);
        assert_eq!(
            state.get_withdrawal_status(withdrawal_id),
            WithdrawalStatus::NotFound
        );
        state.record_nicp_withdrawal(caller.into(), nICP::from_unscaled(5), 1, 0);
        assert_eq!(
            state.get_withdrawal_status(withdrawal_id),
            WithdrawalStatus::WaitingToSplitNeuron
        );
        state.record_neuron_split(withdrawal_id, neuron_id);
        assert_eq!(
            state.get_withdrawal_status(withdrawal_id),
            WithdrawalStatus::WaitingToStartDissolving { neuron_id }
        );
        state.record_started_to_dissolve_neuron(withdrawal_id);
        assert_eq!(
            state.get_withdrawal_status(withdrawal_id),
            WithdrawalStatus::WaitingDissolvement { neuron_id }
        );
        state.record_neuron_disbursed(withdrawal_id, 0);
        assert_eq!(
            state.get_withdrawal_status(withdrawal_id),
            WithdrawalStatus::ConversionDone {
                transfer_block_height: 0
            }
        );
    }

    #[test]
    fn should_compute_governance_share() {
        let state = default_state();

        let res_1 = state.compute_governance_share_e8s(100 * E8S);
        assert_eq!(res_1, 10 * E8S);

        let res_2 = state.compute_governance_share_e8s(123 * E8S);
        assert_eq!(res_2, 1_230_000_000);

        let res_3 = state.compute_governance_share_e8s(880_123_000);
        assert_eq!(res_3, 88_012_300);
    }
}
