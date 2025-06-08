import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface InitArg {
	icp_ledger_id: Principal;
	start_ts: bigint;
	wtn_ledger_id: Principal;
	end_ts: bigint;
}
export type Result = { Ok: bigint } | { Err: string };
export type Result_1 = { Ok: bigint } | { Err: string };
export interface Status {
	participants: bigint;
	time_left: [] | [bigint];
	start_at: bigint;
	minimum_deposit_amount: bigint;
	total_icp_deposited: bigint;
	end_at: bigint;
}
export interface _SERVICE {
	claim_wtn: ActorMethod<[Principal], Result>;
	distribute_tokens: ActorMethod<[], Result_1>;
	get_icp_deposit_address: ActorMethod<[Principal], string>;
	get_icp_deposited: ActorMethod<[Principal], bigint>;
	get_principal_to_icp_deposited: ActorMethod<[], Array<[Principal, bigint]>>;
	get_principal_to_wtn_allocation: ActorMethod<[], Array<[Principal, bigint]>>;
	get_status: ActorMethod<[], Status>;
	get_wtn_allocated: ActorMethod<[Principal], bigint>;
	notify_icp_deposit: ActorMethod<[Principal, bigint], Result>;
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: ({ IDL }: { IDL: IDL }) => IDL.Type[];
