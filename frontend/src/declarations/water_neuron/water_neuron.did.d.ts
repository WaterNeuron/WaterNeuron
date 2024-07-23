import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface Account {
	owner: Principal;
	subaccount: [] | [Uint8Array | number[]];
}
export interface CanisterInfo {
	neuron_6m_account: Account;
	neuron_id_6m: [] | [NeuronId];
	neuron_id_8y: [] | [NeuronId];
	tracked_6m_stake: bigint;
	minimum_withdraw_amount: bigint;
	neuron_8y_stake_e8s: bigint;
	governance_fee_share_percent: bigint;
	neuron_8y_account: Account;
	minimum_deposit_amount: bigint;
	neuron_6m_stake_e8s: bigint;
	exchange_rate: bigint;
	nicp_supply: bigint;
	total_icp_deposited: bigint;
	stakers_count: bigint;
}
export interface ConversionArg {
	maybe_subaccount: [] | [Uint8Array | number[]];
	amount_e8s: bigint;
}
export type ConversionError =
	| {
			GenericError: { code: number; message: string };
	  }
	| { TransferError: TransferError }
	| { AmountTooLow: { minimum_amount_e8s: bigint } }
	| { TransferFromError: TransferFromError }
	| { GuardError: { guard_error: GuardError } };
export interface DepositSuccess {
	block_index: bigint;
	transfer_id: bigint;
}
export interface Event {
	timestamp: bigint;
	payload: EventType;
}
export type EventType =
	| {
			ClaimedAirdrop: { block_index: bigint; caller: Principal };
	  }
	| { StartedToDissolve: { withdrawal_id: bigint } }
	| {
			MaturityNeuron: {
				from_neuron_type: NeuronOrigin;
				neuron_id: NeuronId;
			};
	  }
	| { NeuronSixMonths: NeuronId }
	| { Upgrade: UpgradeArg }
	| { Init: InitArg }
	| {
			MirroredProposal: {
				nns_proposal_id: NeuronId;
				sns_proposal_id: NeuronId;
			};
	  }
	| { NeuronEightYears: NeuronId }
	| { DistributeICPtoSNS: { amount: bigint; receiver: Principal } }
	| {
			NIcpWithdrawal: {
				nicp_burned: bigint;
				nicp_burn_index: bigint;
				receiver: Account;
			};
	  }
	| {
			IcpDeposit: {
				block_index: bigint;
				amount: bigint;
				receiver: Account;
			};
	  }
	| {
			DisbursedUserNeuron: {
				withdrawal_id: bigint;
				transfer_block_height: bigint;
			};
	  }
	| {
			TransferExecuted: {
				block_index: [] | [bigint];
				transfer_id: bigint;
			};
	  }
	| {
			DisbursedMaturityNeuron: {
				transfer_block_height: bigint;
				neuron_id: NeuronId;
			};
	  }
	| {
			DispatchICPRewards: {
				nicp_amount: bigint;
				sns_gov_amount: bigint;
				from_neuron_type: NeuronOrigin;
			};
	  }
	| { SplitNeuron: { withdrawal_id: bigint; neuron_id: NeuronId } };
export interface ExecutedTransfer {
	block_index: [] | [bigint];
	timestamp: bigint;
	transfer: PendingTransfer;
}
export interface GetEventsArg {
	start: bigint;
	length: bigint;
}
export interface GetEventsResult {
	total_event_count: bigint;
	events: Array<Event>;
}
export type GuardError = { AlreadyProcessing: null } | { TooManyConcurrentRequests: null };
export interface InitArg {
	wtn_ledger_id: Principal;
	wtn_governance_id: Principal;
	nicp_ledger_id: Principal;
}
export type LiquidArg = { Upgrade: [] | [UpgradeArg] } | { Init: InitArg };
export interface NeuronId {
	id: bigint;
}
export type NeuronOrigin = { NICPSixMonths: null } | { SnsGovernanceEightYears: null };
export interface PendingTransfer {
	memo: [] | [bigint];
	unit: Unit;
	from_subaccount: [] | [Uint8Array | number[]];
	transfer_id: bigint;
	amount: bigint;
	receiver: Account;
}
export type Result = { Ok: bigint } | { Err: ConversionError };
export type Result_1 = { Ok: DepositSuccess } | { Err: ConversionError };
export type Result_2 = { Ok: WithdrawalSuccess } | { Err: ConversionError };
export type TransferError =
	| {
			GenericError: { message: string; error_code: bigint };
	  }
	| { TemporarilyUnavailable: null }
	| { BadBurn: { min_burn_amount: bigint } }
	| { Duplicate: { duplicate_of: bigint } }
	| { BadFee: { expected_fee: bigint } }
	| { CreatedInFuture: { ledger_time: bigint } }
	| { TooOld: null }
	| { InsufficientFunds: { balance: bigint } };
export type TransferFromError =
	| {
			GenericError: { message: string; error_code: bigint };
	  }
	| { TemporarilyUnavailable: null }
	| { InsufficientAllowance: { allowance: bigint } }
	| { BadBurn: { min_burn_amount: bigint } }
	| { Duplicate: { duplicate_of: bigint } }
	| { BadFee: { expected_fee: bigint } }
	| { CreatedInFuture: { ledger_time: bigint } }
	| { TooOld: null }
	| { InsufficientFunds: { balance: bigint } };
export type TransferStatus =
	| { Executed: ExecutedTransfer }
	| { Unknown: null }
	| { Pending: PendingTransfer };
export type Unit = { ICP: null } | { WTN: null } | { NICP: null };
export interface UpgradeArg {
	governance_fee_share_percent: [] | [bigint];
}
export interface WithdrawalDetails {
	status: WithdrawalStatus;
	request: WithdrawalRequest;
}
export interface WithdrawalRequest {
	nicp_burned: bigint;
	withdrawal_id: bigint;
	icp_due: bigint;
	nicp_burn_index: bigint;
	timestamp: bigint;
	receiver: Account;
	neuron_id: [] | [NeuronId];
}
export type WithdrawalStatus =
	| {
			ConversionDone: { transfer_block_height: bigint };
	  }
	| { NotFound: null }
	| { WaitingToSplitNeuron: null }
	| { WaitingDissolvement: { neuron_id: NeuronId } }
	| { WaitingToStartDissolving: { neuron_id: NeuronId } };
export interface WithdrawalSuccess {
	block_index: bigint;
	withdrawal_id: bigint;
}
export interface _SERVICE {
	claim_airdrop: ActorMethod<[], Result>;
	get_airdrop_allocation: ActorMethod<[], bigint>;
	get_events: ActorMethod<[GetEventsArg], GetEventsResult>;
	get_info: ActorMethod<[], CanisterInfo>;
	get_transfer_statuses: ActorMethod<[BigUint64Array | bigint[]], Array<TransferStatus>>;
	get_withdrawal_requests: ActorMethod<[[] | [Principal]], Array<WithdrawalDetails>>;
	get_wtn_proposal_id: ActorMethod<[bigint], [] | [bigint]>;
	icp_to_nicp: ActorMethod<[ConversionArg], Result_1>;
	nicp_to_icp: ActorMethod<[ConversionArg], Result_2>;
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: ({ IDL }: { IDL: IDL }) => IDL.Type[];
