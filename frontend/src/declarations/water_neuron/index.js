export const idlFactory = ({ IDL }) => {
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
	const GuardError = IDL.Variant({
		AlreadyProcessing: IDL.Null,
		TooManyConcurrentRequests: IDL.Null
	});
	const ConversionError = IDL.Variant({
		GenericError: IDL.Record({ code: IDL.Int32, message: IDL.Text }),
		TransferError: TransferError,
		AmountTooLow: IDL.Record({ minimum_amount_e8s: IDL.Nat64 }),
		TransferFromError: TransferFromError,
		GuardError: IDL.Record({ guard_error: GuardError })
	});
	const Result = IDL.Variant({ Ok: IDL.Nat64, Err: ConversionError });
	const GetEventsArg = IDL.Record({
		start: IDL.Nat64,
		length: IDL.Nat64
	});
	const NeuronOrigin = IDL.Variant({
		NICPSixMonths: IDL.Null,
		SnsGovernanceEightYears: IDL.Null
	});
	const NeuronId = IDL.Record({ id: IDL.Nat64 });
	const UpgradeArg = IDL.Record({
		governance_fee_share_percent: IDL.Opt(IDL.Nat64)
	});
	const InitArg = IDL.Record({
		wtn_ledger_id: IDL.Principal,
		wtn_governance_id: IDL.Principal,
		nicp_ledger_id: IDL.Principal
	});
	const Account = IDL.Record({
		owner: IDL.Principal,
		subaccount: IDL.Opt(IDL.Vec(IDL.Nat8))
	});
	const EventType = IDL.Variant({
		ClaimedAirdrop: IDL.Record({
			block_index: IDL.Nat64,
			caller: IDL.Principal
		}),
		StartedToDissolve: IDL.Record({ withdrawal_id: IDL.Nat64 }),
		MaturityNeuron: IDL.Record({
			from_neuron_type: NeuronOrigin,
			neuron_id: NeuronId
		}),
		NeuronSixMonths: NeuronId,
		Upgrade: UpgradeArg,
		Init: InitArg,
		MirroredProposal: IDL.Record({
			nns_proposal_id: NeuronId,
			sns_proposal_id: NeuronId
		}),
		NeuronEightYears: NeuronId,
		DistributeICPtoSNS: IDL.Record({
			amount: IDL.Nat64,
			receiver: IDL.Principal
		}),
		NIcpWithdrawal: IDL.Record({
			nicp_burned: IDL.Nat64,
			nicp_burn_index: IDL.Nat64,
			receiver: Account
		}),
		IcpDeposit: IDL.Record({
			block_index: IDL.Nat64,
			amount: IDL.Nat64,
			receiver: Account
		}),
		DisbursedUserNeuron: IDL.Record({
			withdrawal_id: IDL.Nat64,
			transfer_block_height: IDL.Nat64
		}),
		TransferExecuted: IDL.Record({
			block_index: IDL.Opt(IDL.Nat64),
			transfer_id: IDL.Nat64
		}),
		DisbursedMaturityNeuron: IDL.Record({
			transfer_block_height: IDL.Nat64,
			neuron_id: NeuronId
		}),
		DispatchICPRewards: IDL.Record({
			nicp_amount: IDL.Nat64,
			sns_gov_amount: IDL.Nat64,
			from_neuron_type: NeuronOrigin
		}),
		SplitNeuron: IDL.Record({
			withdrawal_id: IDL.Nat64,
			neuron_id: NeuronId
		})
	});
	const Event = IDL.Record({ timestamp: IDL.Nat64, payload: EventType });
	const GetEventsResult = IDL.Record({
		total_event_count: IDL.Nat64,
		events: IDL.Vec(Event)
	});
	const CanisterInfo = IDL.Record({
		neuron_6m_account: Account,
		neuron_id_6m: IDL.Opt(NeuronId),
		neuron_id_8y: IDL.Opt(NeuronId),
		tracked_6m_stake: IDL.Nat64,
		minimum_withdraw_amount: IDL.Nat64,
		neuron_8y_stake_e8s: IDL.Nat64,
		governance_fee_share_percent: IDL.Nat64,
		neuron_8y_account: Account,
		minimum_deposit_amount: IDL.Nat64,
		neuron_6m_stake_e8s: IDL.Nat64,
		exchange_rate: IDL.Nat64,
		nicp_supply: IDL.Nat64,
		total_icp_deposited: IDL.Nat64,
		stakers_count: IDL.Nat64
	});
	const Unit = IDL.Variant({
		ICP: IDL.Null,
		WTN: IDL.Null,
		NICP: IDL.Null
	});
	const PendingTransfer = IDL.Record({
		memo: IDL.Opt(IDL.Nat64),
		unit: Unit,
		from_subaccount: IDL.Opt(IDL.Vec(IDL.Nat8)),
		transfer_id: IDL.Nat64,
		amount: IDL.Nat64,
		receiver: Account
	});
	const ExecutedTransfer = IDL.Record({
		block_index: IDL.Opt(IDL.Nat64),
		timestamp: IDL.Nat64,
		transfer: PendingTransfer
	});
	const TransferStatus = IDL.Variant({
		Executed: ExecutedTransfer,
		Unknown: IDL.Null,
		Pending: PendingTransfer
	});
	const WithdrawalStatus = IDL.Variant({
		ConversionDone: IDL.Record({ transfer_block_height: IDL.Nat64 }),
		NotFound: IDL.Null,
		WaitingToSplitNeuron: IDL.Null,
		WaitingDissolvement: IDL.Record({ neuron_id: NeuronId }),
		WaitingToStartDissolving: IDL.Record({ neuron_id: NeuronId })
	});
	const WithdrawalRequest = IDL.Record({
		nicp_burned: IDL.Nat64,
		withdrawal_id: IDL.Nat64,
		icp_due: IDL.Nat64,
		nicp_burn_index: IDL.Nat64,
		timestamp: IDL.Nat64,
		receiver: Account,
		neuron_id: IDL.Opt(NeuronId)
	});
	const WithdrawalDetails = IDL.Record({
		status: WithdrawalStatus,
		request: WithdrawalRequest
	});
	const ConversionArg = IDL.Record({
		maybe_subaccount: IDL.Opt(IDL.Vec(IDL.Nat8)),
		amount_e8s: IDL.Nat64
	});
	const DepositSuccess = IDL.Record({
		block_index: IDL.Nat,
		transfer_id: IDL.Nat64
	});
	const Result_1 = IDL.Variant({
		Ok: DepositSuccess,
		Err: ConversionError
	});
	const WithdrawalSuccess = IDL.Record({
		block_index: IDL.Nat,
		withdrawal_id: IDL.Nat64
	});
	const Result_2 = IDL.Variant({
		Ok: WithdrawalSuccess,
		Err: ConversionError
	});
	return IDL.Service({
		claim_airdrop: IDL.Func([], [Result], []),
		get_airdrop_allocation: IDL.Func([], [IDL.Nat64], ['query']),
		get_events: IDL.Func([GetEventsArg], [GetEventsResult], ['query']),
		get_info: IDL.Func([], [CanisterInfo], ['query']),
		get_transfer_statuses: IDL.Func([IDL.Vec(IDL.Nat64)], [IDL.Vec(TransferStatus)], ['query']),
		get_withdrawal_requests: IDL.Func(
			[IDL.Opt(IDL.Principal)],
			[IDL.Vec(WithdrawalDetails)],
			['query']
		),
		get_wtn_proposal_id: IDL.Func([IDL.Nat64], [IDL.Opt(IDL.Nat64)], ['query']),
		icp_to_nicp: IDL.Func([ConversionArg], [Result_1], []),
		nicp_to_icp: IDL.Func([ConversionArg], [Result_2], [])
	});
};
export const init = ({ IDL }) => {
	return [];
};
