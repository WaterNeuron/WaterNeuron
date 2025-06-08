import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface Account {
	owner: Principal;
	subaccount: [] | [Uint8Array | number[]];
}
export type ApproveError =
	| {
			GenericError: { message: string; error_code: bigint };
	  }
	| { TemporarilyUnavailable: null }
	| { Duplicate: { duplicate_of: bigint } }
	| { BadFee: { expected_fee: bigint } }
	| { AllowanceChanged: { current_allowance: bigint } }
	| { CreatedInFuture: { ledger_time: bigint } }
	| { TooOld: null }
	| { Expired: { ledger_time: bigint } }
	| { InsufficientFunds: { balance: bigint } };
export type BoomerangError =
	| {
			GenericError: { code: number; message: string };
	  }
	| { TransferError: TransferError }
	| { ConversionError: ConversionError }
	| { ApproveError: ApproveError }
	| { NotEnoughICP: null };
export interface CanisterIds {
	icp_ledger_id: Principal;
	water_neuron_id: Principal;
	wtn_ledger_id: Principal;
	nicp_ledger_id: Principal;
}
export type ConversionError =
	| {
			GenericError: { code: number; message: string };
	  }
	| { TransferError: TransferError }
	| { AmountTooLow: { minimum_amount_e8s: bigint } }
	| { TransferFromError: TransferFromError };
export interface DepositSuccess {
	nicp_amount: [] | [bigint];
	block_index: bigint;
	transfer_id: bigint;
}
export type Result = { Ok: DepositSuccess } | { Err: BoomerangError };
export type Result_1 = { Ok: WithdrawalSuccess } | { Err: BoomerangError };
export type Result_2 = { Ok: bigint } | { Err: BoomerangError };
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
export interface WithdrawalSuccess {
	block_index: bigint;
	withdrawal_id: bigint;
}
export interface _SERVICE {
	get_staking_account: ActorMethod<[Principal], Account>;
	get_unstaking_account: ActorMethod<[Principal], Account>;
	notify_icp_deposit: ActorMethod<[Principal], Result>;
	notify_nicp_deposit: ActorMethod<[Principal], Result_1>;
	retrieve_nicp: ActorMethod<[Principal], Result_2>;
	try_retrieve_icp: ActorMethod<[Principal], Result_2>;
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: ({ IDL }: { IDL: IDL }) => IDL.Type[];
