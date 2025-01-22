import { IDL } from '@dfinity/candid';

export const idlFactory = () => {
	const InitArg = IDL.Record({
		icp_ledger_id: IDL.Principal,
		start_ts: IDL.Nat64,
		wtn_ledger_id: IDL.Principal,
		end_ts: IDL.Nat64
	});
	const Result = IDL.Variant({ Ok: IDL.Nat64, Err: IDL.Text });
	const Result_1 = IDL.Variant({ Ok: IDL.Nat64, Err: IDL.Text });
	const Status = IDL.Record({
		participants: IDL.Nat64,
		time_left: IDL.Opt(IDL.Nat64),
		start_at: IDL.Nat64,
		minimum_deposit_amount: IDL.Nat64,
		total_icp_deposited: IDL.Nat64,
		end_at: IDL.Nat64
	});
	return IDL.Service({
		claim_wtn: IDL.Func([IDL.Principal], [Result], []),
		distribute_tokens: IDL.Func([], [Result_1], []),
		get_icp_deposit_address: IDL.Func([IDL.Principal], [IDL.Text], []),
		get_icp_deposited: IDL.Func([IDL.Principal], [IDL.Nat64], ['query']),
		get_principal_to_icp_deposited: IDL.Func(
			[],
			[IDL.Vec(IDL.Tuple(IDL.Principal, IDL.Nat64))],
			['query']
		),
		get_principal_to_wtn_allocation: IDL.Func(
			[],
			[IDL.Vec(IDL.Tuple(IDL.Principal, IDL.Nat64))],
			['query']
		),
		get_status: IDL.Func([], [Status], ['query']),
		get_wtn_allocated: IDL.Func([IDL.Principal], [IDL.Nat64], ['query']),
		notify_icp_deposit: IDL.Func([IDL.Principal, IDL.Nat64], [Result], [])
	});
};
export const init = () => {
	const InitArg = IDL.Record({
		icp_ledger_id: IDL.Principal,
		start_ts: IDL.Nat64,
		wtn_ledger_id: IDL.Principal,
		end_ts: IDL.Nat64
	});
	return [InitArg];
};
