import { IDL } from '@dfinity/candid';

export const idlFactory = () => {
	const UpgradeArg = IDL.Record({
		governance_fee_share_percent: IDL.Opt(IDL.Nat64)
	});
	const InitArg = IDL.Record({
		wtn_ledger_id: IDL.Principal,
		wtn_governance_id: IDL.Principal,
		nicp_ledger_id: IDL.Principal
	});
	const LiquidArg = IDL.Variant({
		Upgrade: IDL.Opt(UpgradeArg),
		Init: InitArg
	});
	const NeuronId = IDL.Record({ id: IDL.Nat64 });
	const BallotInfo = IDL.Record({
		vote: IDL.Int32,
		proposal_id: IDL.Opt(NeuronId)
	});
	const DissolveState = IDL.Variant({
		DissolveDelaySeconds: IDL.Nat64,
		WhenDissolvedTimestampSeconds: IDL.Nat64
	});
	const Followees = IDL.Record({ followees: IDL.Vec(NeuronId) });
	const NeuronStakeTransfer = IDL.Record({
		to_subaccount: IDL.Vec(IDL.Nat8),
		neuron_stake_e8s: IDL.Nat64,
		from: IDL.Opt(IDL.Principal),
		memo: IDL.Nat64,
		from_subaccount: IDL.Vec(IDL.Nat8),
		transfer_timestamp: IDL.Nat64,
		block_height: IDL.Nat64
	});
	const KnownNeuronData = IDL.Record({
		name: IDL.Text,
		description: IDL.Opt(IDL.Text)
	});
	const Neuron = IDL.Record({
		id: IDL.Opt(NeuronId),
		staked_maturity_e8s_equivalent: IDL.Opt(IDL.Nat64),
		controller: IDL.Opt(IDL.Principal),
		recent_ballots: IDL.Vec(BallotInfo),
		kyc_verified: IDL.Bool,
		neuron_type: IDL.Opt(IDL.Int32),
		not_for_profit: IDL.Bool,
		maturity_e8s_equivalent: IDL.Nat64,
		cached_neuron_stake_e8s: IDL.Nat64,
		created_timestamp_seconds: IDL.Nat64,
		auto_stake_maturity: IDL.Opt(IDL.Bool),
		aging_since_timestamp_seconds: IDL.Nat64,
		hot_keys: IDL.Vec(IDL.Principal),
		account: IDL.Vec(IDL.Nat8),
		joined_community_fund_timestamp_seconds: IDL.Opt(IDL.Nat64),
		dissolve_state: IDL.Opt(DissolveState),
		followees: IDL.Vec(IDL.Tuple(IDL.Int32, Followees)),
		neuron_fees_e8s: IDL.Nat64,
		transfer: IDL.Opt(NeuronStakeTransfer),
		known_neuron_data: IDL.Opt(KnownNeuronData),
		spawn_at_timestamp_seconds: IDL.Opt(IDL.Nat64)
	});
	const NeuronInfo = IDL.Record({
		dissolve_delay_seconds: IDL.Nat64,
		recent_ballots: IDL.Vec(BallotInfo),
		neuron_type: IDL.Opt(IDL.Int32),
		created_timestamp_seconds: IDL.Nat64,
		state: IDL.Int32,
		stake_e8s: IDL.Nat64,
		joined_community_fund_timestamp_seconds: IDL.Opt(IDL.Nat64),
		retrieved_at_timestamp_seconds: IDL.Nat64,
		known_neuron_data: IDL.Opt(KnownNeuronData),
		voting_power: IDL.Nat64,
		age_seconds: IDL.Nat64
	});
	const MergeResponse = IDL.Record({
		target_neuron: IDL.Opt(Neuron),
		source_neuron: IDL.Opt(Neuron),
		target_neuron_info: IDL.Opt(NeuronInfo),
		source_neuron_info: IDL.Opt(NeuronInfo)
	});
	const GovernanceError = IDL.Record({
		error_message: IDL.Text,
		error_type: IDL.Int32
	});
	const GuardError = IDL.Variant({
		AlreadyProcessing: IDL.Null,
		TooManyConcurrentRequests: IDL.Null
	});
	const CancelWithdrawalError = IDL.Variant({
		GenericError: IDL.Record({ code: IDL.Int32, message: IDL.Text }),
		TooLate: IDL.Null,
		BadCommand: IDL.Record({ message: IDL.Text }),
		UnknownTimeLeft: IDL.Null,
		BadCaller: IDL.Record({ message: IDL.Text }),
		MergeNeuronError: IDL.Record({ message: IDL.Text }),
		StopDissolvementError: IDL.Record({ message: IDL.Text }),
		RequestNotFound: IDL.Null,
		GovernanceError: GovernanceError,
		GuardError: IDL.Record({ guard_error: GuardError }),
		GetFullNeuronError: IDL.Record({ message: IDL.Text })
	});
	const Result = IDL.Variant({
		Ok: MergeResponse,
		Err: CancelWithdrawalError
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
		TransferFromError: TransferFromError,
		GuardError: IDL.Record({ guard_error: GuardError })
	});
	const Result_1 = IDL.Variant({ Ok: IDL.Nat64, Err: ConversionError });
	const GetEventsArg = IDL.Record({
		start: IDL.Nat64,
		length: IDL.Nat64
	});
	const NeuronOrigin = IDL.Variant({
		NICPSixMonths: IDL.Null,
		SnsGovernanceEightYears: IDL.Null
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
		MergeNeuron: IDL.Record({ neuron_id: NeuronId }),
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
		DistributeICPtoSNSv2: IDL.Null,
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
		latest_distribution_icp_per_vp: IDL.Opt(IDL.Float64),
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
		Cancelled: IDL.Null,
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
	const Result_2 = IDL.Variant({ Ok: NeuronId, Err: NeuronId });
	const ConversionArg = IDL.Record({
		maybe_subaccount: IDL.Opt(IDL.Vec(IDL.Nat8)),
		amount_e8s: IDL.Nat64
	});
	const DepositSuccess = IDL.Record({
		nicp_amount: IDL.Opt(IDL.Nat64),
		block_index: IDL.Nat,
		transfer_id: IDL.Nat64
	});
	const Result_3 = IDL.Variant({
		Ok: DepositSuccess,
		Err: ConversionError
	});
	const StandardRecord = IDL.Record({ url: IDL.Text, name: IDL.Text });
	const ConsentMessageMetadata = IDL.Record({
		utc_offset_minutes: IDL.Opt(IDL.Nat16),
		language: IDL.Text
	});
	const DisplayMessageType = IDL.Variant({
		GenericDisplay: IDL.Null,
		LineDisplay: IDL.Record({
			characters_per_line: IDL.Nat16,
			lines_per_page: IDL.Nat16
		})
	});
	const ConsentMessageSpec = IDL.Record({
		metadata: ConsentMessageMetadata,
		device_spec: IDL.Opt(DisplayMessageType)
	});
	const ConsentMessageRequest = IDL.Record({
		arg: IDL.Vec(IDL.Nat8),
		method: IDL.Text,
		user_preferences: ConsentMessageSpec
	});
	const LineDisplayPage = IDL.Record({ lines: IDL.Vec(IDL.Text) });
	const ConsentMessage = IDL.Variant({
		LineDisplayMessage: IDL.Record({ pages: IDL.Vec(LineDisplayPage) }),
		GenericDisplayMessage: IDL.Text
	});
	const ConsentInfo = IDL.Record({
		metadata: ConsentMessageMetadata,
		consent_message: ConsentMessage
	});
	const ErrorInfo = IDL.Record({ description: IDL.Text });
	const Icrc21Error = IDL.Variant({
		GenericError: IDL.Record({
			description: IDL.Text,
			error_code: IDL.Nat64
		}),
		InsufficientPayment: ErrorInfo,
		UnsupportedCanisterCall: ErrorInfo,
		ConsentMessageUnavailable: ErrorInfo
	});
	const Result_5 = IDL.Variant({ Ok: ConsentInfo, Err: Icrc21Error });
	const WithdrawalSuccess = IDL.Record({
		block_index: IDL.Nat,
		withdrawal_id: IDL.Nat64,
		icp_amount: IDL.Opt(IDL.Nat64)
	});
	const Result_4 = IDL.Variant({
		Ok: WithdrawalSuccess,
		Err: ConversionError
	});
	return IDL.Service({
		cancel_withdrawal: IDL.Func([NeuronId], [Result], []),
		claim_airdrop: IDL.Func([], [Result_1], []),
		get_airdrop_allocation: IDL.Func([IDL.Opt(IDL.Principal)], [IDL.Nat64], ['query']),
		get_events: IDL.Func([GetEventsArg], [GetEventsResult], ['query']),
		get_info: IDL.Func([], [CanisterInfo], ['query']),
		get_pending_rewards: IDL.Func([IDL.Opt(IDL.Principal)], [IDL.Nat64], ['query']),
		get_transfer_statuses: IDL.Func([IDL.Vec(IDL.Nat64)], [IDL.Vec(TransferStatus)], ['query']),
		get_withdrawal_requests: IDL.Func([IDL.Opt(Account)], [IDL.Vec(WithdrawalDetails)], ['query']),
		get_wtn_proposal_id: IDL.Func([IDL.Nat64], [Result_2], ['query']),
		icp_to_nicp: IDL.Func([ConversionArg], [Result_3], []),
		icrc10_supported_standards: IDL.Func([], [IDL.Vec(StandardRecord)], ['query']),
		icrc21_canister_call_consent_message: IDL.Func([ConsentMessageRequest], [Result_5], []),
		nicp_to_icp: IDL.Func([ConversionArg], [Result_4], [])
	});
};
export const init = () => {
	const UpgradeArg = IDL.Record({
		governance_fee_share_percent: IDL.Opt(IDL.Nat64)
	});
	const InitArg = IDL.Record({
		wtn_ledger_id: IDL.Principal,
		wtn_governance_id: IDL.Principal,
		nicp_ledger_id: IDL.Principal
	});
	const LiquidArg = IDL.Variant({
		Upgrade: IDL.Opt(UpgradeArg),
		Init: InitArg
	});
	return [LiquidArg];
};
