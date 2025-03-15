import { IDL } from '@dfinity/candid';
export const idlFactory = () => {
	const CanisterIds = IDL.Record({
		icp_ledger_id: IDL.Principal,
		water_neuron_id: IDL.Principal,
		wtn_ledger_id: IDL.Principal,
		nicp_ledger_id: IDL.Principal
	});
	const Account = IDL.Record({
		owner: IDL.Principal,
		subaccount: IDL.Opt(IDL.Vec(IDL.Nat8))
	});
	const DepositSuccess = IDL.Record({
		nicp_amount: IDL.Opt(IDL.Nat64),
		block_index: IDL.Nat,
		transfer_id: IDL.Nat64
	});
	const TransferError = IDL.Variant({
		GenericError: IDL.Record({
			message: IDL.Text,
			error_code: IDL.Nat
		}),
		TemporarilyUnavailable: IDL.Null,
		BadBurn: IDL.Record({ min_burn_amount: IDL.Nat }),
		Duplicate: IDL.Record({ duplicate_of: IDL.Nat }),
		BadFee: IDL.Record({ expected_fee: IDL.Nat }),
		CreatedInFuture: IDL.Record({ ledger_time: IDL.Nat64 }),
		TooOld: IDL.Null,
		InsufficientFunds: IDL.Record({ balance: IDL.Nat })
	});
	const TransferFromError = IDL.Variant({
		GenericError: IDL.Record({
			message: IDL.Text,
			error_code: IDL.Nat
		}),
		TemporarilyUnavailable: IDL.Null,
		InsufficientAllowance: IDL.Record({ allowance: IDL.Nat }),
		BadBurn: IDL.Record({ min_burn_amount: IDL.Nat }),
		Duplicate: IDL.Record({ duplicate_of: IDL.Nat }),
		BadFee: IDL.Record({ expected_fee: IDL.Nat }),
		CreatedInFuture: IDL.Record({ ledger_time: IDL.Nat64 }),
		TooOld: IDL.Null,
		InsufficientFunds: IDL.Record({ balance: IDL.Nat })
	});
	const ConversionError = IDL.Variant({
		GenericError: IDL.Record({ code: IDL.Int32, message: IDL.Text }),
		TransferError: TransferError,
		AmountTooLow: IDL.Record({ minimum_amount_e8s: IDL.Nat64 }),
		TransferFromError: TransferFromError
	});
	const ApproveError = IDL.Variant({
		GenericError: IDL.Record({
			message: IDL.Text,
			error_code: IDL.Nat
		}),
		TemporarilyUnavailable: IDL.Null,
		Duplicate: IDL.Record({ duplicate_of: IDL.Nat }),
		BadFee: IDL.Record({ expected_fee: IDL.Nat }),
		AllowanceChanged: IDL.Record({ current_allowance: IDL.Nat }),
		CreatedInFuture: IDL.Record({ ledger_time: IDL.Nat64 }),
		TooOld: IDL.Null,
		Expired: IDL.Record({ ledger_time: IDL.Nat64 }),
		InsufficientFunds: IDL.Record({ balance: IDL.Nat })
	});
	const BoomerangError = IDL.Variant({
		GenericError: IDL.Record({ code: IDL.Int32, message: IDL.Text }),
		TransferError: TransferError,
		ConversionError: ConversionError,
		ApproveError: ApproveError,
		NotEnoughICP: IDL.Null
	});
	const Result = IDL.Variant({ Ok: DepositSuccess, Err: BoomerangError });
	const WithdrawalSuccess = IDL.Record({
		block_index: IDL.Nat,
		withdrawal_id: IDL.Nat64
	});
	const Result_1 = IDL.Variant({
		Ok: WithdrawalSuccess,
		Err: BoomerangError
	});
	const Result_2 = IDL.Variant({ Ok: IDL.Nat, Err: BoomerangError });
	return IDL.Service({
		get_staking_account: IDL.Func([IDL.Principal], [Account], []),
		get_unstaking_account: IDL.Func([IDL.Principal], [Account], []),
		notify_icp_deposit: IDL.Func([IDL.Principal], [Result], []),
		notify_nicp_deposit: IDL.Func([IDL.Principal], [Result_1], []),
		retrieve_nicp: IDL.Func([IDL.Principal], [Result_2], []),
		try_retrieve_icp: IDL.Func([IDL.Principal], [Result_2], [])
	});
};
export const init = () => {
	const CanisterIds = IDL.Record({
		icp_ledger_id: IDL.Principal,
		water_neuron_id: IDL.Principal,
		wtn_ledger_id: IDL.Principal,
		nicp_ledger_id: IDL.Principal
	});
	return [CanisterIds];
};
