import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface Account {
	owner: Principal;
	subaccount: [] | [Uint8Array | number[]];
}
export interface BallotInfo {
	vote: number;
	proposal_id: [] | [NeuronId];
}
export type CancelWithdrawalError =
	| {
			GenericError: { code: number; message: string };
	  }
	| { TooLate: null }
	| { BadCommand: { message: string } }
	| { UnknownTimeLeft: null }
	| { BadCaller: { message: string } }
	| { MergeNeuronError: { message: string } }
	| { StopDissolvementError: { message: string } }
	| { RequestNotFound: null }
	| { GovernanceError: GovernanceError }
	| { GuardError: { guard_error: GuardError } }
	| { GetFullNeuronError: { message: string } };
export interface CanisterInfo {
	neuron_6m_account: Account;
	latest_distribution_icp_per_vp: [] | [number];
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
export interface ConsentInfo {
	metadata: ConsentMessageMetadata;
	consent_message: ConsentMessage;
}
export type ConsentMessage =
	| {
			LineDisplayMessage: { pages: Array<LineDisplayPage> };
	  }
	| { GenericDisplayMessage: string };
export interface ConsentMessageMetadata {
	utc_offset_minutes: [] | [number];
	language: string;
}
export interface ConsentMessageRequest {
	arg: Uint8Array | number[];
	method: string;
	user_preferences: ConsentMessageSpec;
}
export interface ConsentMessageSpec {
	metadata: ConsentMessageMetadata;
	device_spec: [] | [DisplayMessageType];
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
	nicp_amount: [] | [bigint];
	block_index: bigint;
	transfer_id: bigint;
}
export type DisplayMessageType =
	| { GenericDisplay: null }
	| {
			LineDisplay: {
				characters_per_line: number;
				lines_per_page: number;
			};
	  };
export type DissolveState =
	| { DissolveDelaySeconds: bigint }
	| { WhenDissolvedTimestampSeconds: bigint };
export interface ErrorInfo {
	description: string;
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
	| { MergeNeuron: { neuron_id: NeuronId } }
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
	| { DistributeICPtoSNSv2: null }
	| { SplitNeuron: { withdrawal_id: bigint; neuron_id: NeuronId } };
export interface ExecutedTransfer {
	block_index: [] | [bigint];
	timestamp: bigint;
	transfer: PendingTransfer;
}
export interface Followees {
	followees: Array<NeuronId>;
}
export interface GetEventsArg {
	start: bigint;
	length: bigint;
}
export interface GetEventsResult {
	total_event_count: bigint;
	events: Array<Event>;
}
export interface GovernanceError {
	error_message: string;
	error_type: number;
}
export type GuardError = { AlreadyProcessing: null } | { TooManyConcurrentRequests: null };
export type Icrc21Error =
	| {
			GenericError: { description: string; error_code: bigint };
	  }
	| { InsufficientPayment: ErrorInfo }
	| { UnsupportedCanisterCall: ErrorInfo }
	| { ConsentMessageUnavailable: ErrorInfo };
export interface InitArg {
	wtn_ledger_id: Principal;
	wtn_governance_id: Principal;
	nicp_ledger_id: Principal;
}
export interface KnownNeuronData {
	name: string;
	description: [] | [string];
}
export interface LineDisplayPage {
	lines: Array<string>;
}
export type LiquidArg = { Upgrade: [] | [UpgradeArg] } | { Init: InitArg };
export interface MergeResponse {
	target_neuron: [] | [Neuron];
	source_neuron: [] | [Neuron];
	target_neuron_info: [] | [NeuronInfo];
	source_neuron_info: [] | [NeuronInfo];
}
export interface Neuron {
	id: [] | [NeuronId];
	staked_maturity_e8s_equivalent: [] | [bigint];
	controller: [] | [Principal];
	recent_ballots: Array<BallotInfo>;
	kyc_verified: boolean;
	neuron_type: [] | [number];
	not_for_profit: boolean;
	maturity_e8s_equivalent: bigint;
	cached_neuron_stake_e8s: bigint;
	created_timestamp_seconds: bigint;
	auto_stake_maturity: [] | [boolean];
	aging_since_timestamp_seconds: bigint;
	hot_keys: Array<Principal>;
	account: Uint8Array | number[];
	joined_community_fund_timestamp_seconds: [] | [bigint];
	dissolve_state: [] | [DissolveState];
	followees: Array<[number, Followees]>;
	neuron_fees_e8s: bigint;
	transfer: [] | [NeuronStakeTransfer];
	known_neuron_data: [] | [KnownNeuronData];
	spawn_at_timestamp_seconds: [] | [bigint];
}
export interface NeuronId {
	id: bigint;
}
export interface NeuronInfo {
	dissolve_delay_seconds: bigint;
	recent_ballots: Array<BallotInfo>;
	neuron_type: [] | [number];
	created_timestamp_seconds: bigint;
	state: number;
	stake_e8s: bigint;
	joined_community_fund_timestamp_seconds: [] | [bigint];
	retrieved_at_timestamp_seconds: bigint;
	known_neuron_data: [] | [KnownNeuronData];
	voting_power: bigint;
	age_seconds: bigint;
}
export type NeuronOrigin = { NICPSixMonths: null } | { SnsGovernanceEightYears: null };
export interface NeuronStakeTransfer {
	to_subaccount: Uint8Array | number[];
	neuron_stake_e8s: bigint;
	from: [] | [Principal];
	memo: bigint;
	from_subaccount: Uint8Array | number[];
	transfer_timestamp: bigint;
	block_height: bigint;
}
export interface PendingTransfer {
	memo: [] | [bigint];
	unit: Unit;
	from_subaccount: [] | [Uint8Array | number[]];
	transfer_id: bigint;
	amount: bigint;
	receiver: Account;
}
export type Result = { Ok: MergeResponse } | { Err: CancelWithdrawalError };
export type Result_1 = { Ok: bigint } | { Err: ConversionError };
export type Result_2 = { Ok: NeuronId } | { Err: NeuronId };
export type Result_3 = { Ok: DepositSuccess } | { Err: ConversionError };
export type Result_4 = { Ok: WithdrawalSuccess } | { Err: ConversionError };
export type Result_5 = { Ok: ConsentInfo } | { Err: Icrc21Error };
export interface StandardRecord {
	url: string;
	name: string;
}
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
	| { Cancelled: null }
	| { WaitingToSplitNeuron: null }
	| { WaitingDissolvement: { neuron_id: NeuronId } }
	| { WaitingToStartDissolving: { neuron_id: NeuronId } };
export interface WithdrawalSuccess {
	block_index: bigint;
	withdrawal_id: bigint;
	icp_amount: [] | [bigint];
}
export interface _SERVICE {
	cancel_withdrawal: ActorMethod<[NeuronId], Result>;
	claim_airdrop: ActorMethod<[], Result_1>;
	get_airdrop_allocation: ActorMethod<[[] | [Principal]], bigint>;
	get_events: ActorMethod<[GetEventsArg], GetEventsResult>;
	get_info: ActorMethod<[], CanisterInfo>;
	get_pending_rewards: ActorMethod<[[] | [Principal]], bigint>;
	get_transfer_statuses: ActorMethod<[BigUint64Array | bigint[]], Array<TransferStatus>>;
	get_withdrawal_requests: ActorMethod<[[] | [Account]], Array<WithdrawalDetails>>;
	get_wtn_proposal_id: ActorMethod<[bigint], Result_2>;
	icp_to_nicp: ActorMethod<[ConversionArg], Result_3>;
	icrc10_supported_standards: ActorMethod<[], Array<StandardRecord>>;
	icrc21_canister_call_consent_message: ActorMethod<[ConsentMessageRequest], Result_5>;
	nicp_to_icp: ActorMethod<[ConversionArg], Result_4>;
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: ({ IDL }: { IDL: IDL }) => IDL.Type[];
