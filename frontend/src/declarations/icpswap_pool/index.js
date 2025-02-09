import { IDL } from '@dfinity/candid';
export const idlFactory = () => {
	const Token = IDL.Record({ address: IDL.Text, standard: IDL.Text });
	const AccountBalance = IDL.Record({
		balance0: IDL.Nat,
		balance1: IDL.Nat
	});
	const Page_5 = IDL.Record({
		content: IDL.Vec(IDL.Tuple(IDL.Principal, AccountBalance)),
		offset: IDL.Nat,
		limit: IDL.Nat,
		totalElements: IDL.Nat
	});
	const Error = IDL.Variant({
		CommonError: IDL.Null,
		InternalError: IDL.Text,
		UnsupportedToken: IDL.Text,
		InsufficientFunds: IDL.Null
	});
	const Result_24 = IDL.Variant({ ok: Page_5, err: Error });
	const Result_2 = IDL.Variant({ ok: IDL.Bool, err: Error });
	const Result_23 = IDL.Variant({
		ok: IDL.Record({
			tokenIncome: IDL.Vec(
				IDL.Tuple(IDL.Nat, IDL.Record({ tokensOwed0: IDL.Nat, tokensOwed1: IDL.Nat }))
			),
			totalTokensOwed0: IDL.Nat,
			totalTokensOwed1: IDL.Nat
		}),
		err: Error
	});
	const ClaimArgs = IDL.Record({ positionId: IDL.Nat });
	const Result_22 = IDL.Variant({
		ok: IDL.Record({ amount0: IDL.Nat, amount1: IDL.Nat }),
		err: Error
	});
	const DecreaseLiquidityArgs = IDL.Record({
		liquidity: IDL.Text,
		positionId: IDL.Nat
	});
	const DepositArgs = IDL.Record({
		fee: IDL.Nat,
		token: IDL.Text,
		amount: IDL.Nat
	});
	const Result = IDL.Variant({ ok: IDL.Nat, err: Error });
	const DepositAndMintArgs = IDL.Record({
		tickUpper: IDL.Int,
		fee0: IDL.Nat,
		fee1: IDL.Nat,
		amount0: IDL.Nat,
		amount1: IDL.Nat,
		positionOwner: IDL.Principal,
		amount0Desired: IDL.Text,
		amount1Desired: IDL.Text,
		tickLower: IDL.Int
	});
	const CycleInfo = IDL.Record({ balance: IDL.Nat, available: IDL.Nat });
	const Result_21 = IDL.Variant({ ok: CycleInfo, err: Error });
	const GetPositionArgs = IDL.Record({
		tickUpper: IDL.Int,
		tickLower: IDL.Int
	});
	const PositionInfo = IDL.Record({
		tokensOwed0: IDL.Nat,
		tokensOwed1: IDL.Nat,
		feeGrowthInside1LastX128: IDL.Nat,
		liquidity: IDL.Nat,
		feeGrowthInside0LastX128: IDL.Nat
	});
	const Result_20 = IDL.Variant({ ok: PositionInfo, err: Error });
	const PositionInfoWithId = IDL.Record({
		id: IDL.Text,
		tokensOwed0: IDL.Nat,
		tokensOwed1: IDL.Nat,
		feeGrowthInside1LastX128: IDL.Nat,
		liquidity: IDL.Nat,
		feeGrowthInside0LastX128: IDL.Nat
	});
	const Page_4 = IDL.Record({
		content: IDL.Vec(PositionInfoWithId),
		offset: IDL.Nat,
		limit: IDL.Nat,
		totalElements: IDL.Nat
	});
	const Result_19 = IDL.Variant({ ok: Page_4, err: Error });
	const TransactionType = IDL.Variant({
		decreaseLiquidity: IDL.Null,
		claim: IDL.Null,
		swap: IDL.Null,
		addLiquidity: IDL.Null,
		increaseLiquidity: IDL.Null
	});
	const SwapRecordInfo = IDL.Record({
		to: IDL.Text,
		feeAmount: IDL.Int,
		action: TransactionType,
		feeAmountTotal: IDL.Int,
		token0Id: IDL.Text,
		token1Id: IDL.Text,
		token0AmountTotal: IDL.Nat,
		liquidityTotal: IDL.Nat,
		from: IDL.Text,
		tick: IDL.Int,
		feeTire: IDL.Nat,
		recipient: IDL.Text,
		token0ChangeAmount: IDL.Nat,
		token1AmountTotal: IDL.Nat,
		liquidityChange: IDL.Nat,
		token1Standard: IDL.Text,
		TVLToken0: IDL.Int,
		TVLToken1: IDL.Int,
		token0Fee: IDL.Nat,
		token1Fee: IDL.Nat,
		timestamp: IDL.Int,
		token1ChangeAmount: IDL.Nat,
		token0Standard: IDL.Text,
		price: IDL.Nat,
		poolId: IDL.Text
	});
	const PushError = IDL.Record({ time: IDL.Int, message: IDL.Text });
	const Result_18 = IDL.Variant({
		ok: IDL.Record({
			infoCid: IDL.Text,
			records: IDL.Vec(SwapRecordInfo),
			errors: IDL.Vec(PushError),
			retryCount: IDL.Nat
		}),
		err: Error
	});
	const TickLiquidityInfo = IDL.Record({
		tickIndex: IDL.Int,
		price0Decimal: IDL.Nat,
		liquidityNet: IDL.Int,
		price0: IDL.Nat,
		price1: IDL.Nat,
		liquidityGross: IDL.Nat,
		price1Decimal: IDL.Nat
	});
	const Page_3 = IDL.Record({
		content: IDL.Vec(TickLiquidityInfo),
		offset: IDL.Nat,
		limit: IDL.Nat,
		totalElements: IDL.Nat
	});
	const Result_17 = IDL.Variant({ ok: Page_3, err: Error });
	const TickInfoWithId = IDL.Record({
		id: IDL.Text,
		initialized: IDL.Bool,
		feeGrowthOutside1X128: IDL.Nat,
		secondsPerLiquidityOutsideX128: IDL.Nat,
		liquidityNet: IDL.Int,
		secondsOutside: IDL.Nat,
		liquidityGross: IDL.Nat,
		feeGrowthOutside0X128: IDL.Nat,
		tickCumulativeOutside: IDL.Int
	});
	const Page_2 = IDL.Record({
		content: IDL.Vec(TickInfoWithId),
		offset: IDL.Nat,
		limit: IDL.Nat,
		totalElements: IDL.Nat
	});
	const Result_16 = IDL.Variant({ ok: Page_2, err: Error });
	const Result_15 = IDL.Variant({
		ok: IDL.Record({
			swapFee0Repurchase: IDL.Nat,
			token0Amount: IDL.Nat,
			swapFeeReceiver: IDL.Text,
			token1Amount: IDL.Nat,
			swapFee1Repurchase: IDL.Nat
		}),
		err: Error
	});
	const Value = IDL.Variant({
		Int: IDL.Int,
		Nat: IDL.Nat,
		Blob: IDL.Vec(IDL.Nat8),
		Text: IDL.Text
	});
	const TransferLog = IDL.Record({
		to: IDL.Principal,
		fee: IDL.Nat,
		result: IDL.Text,
		token: Token,
		action: IDL.Text,
		daysFrom19700101: IDL.Nat,
		owner: IDL.Principal,
		from: IDL.Principal,
		fromSubaccount: IDL.Opt(IDL.Vec(IDL.Nat8)),
		timestamp: IDL.Nat,
		index: IDL.Nat,
		amount: IDL.Nat,
		errorMsg: IDL.Text
	});
	const Result_14 = IDL.Variant({ ok: IDL.Vec(TransferLog), err: Error });
	const Result_1 = IDL.Variant({ ok: IDL.Text, err: Error });
	const UserPositionInfo = IDL.Record({
		tickUpper: IDL.Int,
		tokensOwed0: IDL.Nat,
		tokensOwed1: IDL.Nat,
		feeGrowthInside1LastX128: IDL.Nat,
		liquidity: IDL.Nat,
		feeGrowthInside0LastX128: IDL.Nat,
		tickLower: IDL.Int
	});
	const Result_13 = IDL.Variant({ ok: UserPositionInfo, err: Error });
	const Result_12 = IDL.Variant({
		ok: IDL.Vec(IDL.Tuple(IDL.Text, IDL.Vec(IDL.Nat))),
		err: Error
	});
	const Result_11 = IDL.Variant({ ok: IDL.Vec(IDL.Nat), err: Error });
	const UserPositionInfoWithTokenAmount = IDL.Record({
		id: IDL.Nat,
		tickUpper: IDL.Int,
		tokensOwed0: IDL.Nat,
		tokensOwed1: IDL.Nat,
		feeGrowthInside1LastX128: IDL.Nat,
		liquidity: IDL.Nat,
		feeGrowthInside0LastX128: IDL.Nat,
		token0Amount: IDL.Nat,
		token1Amount: IDL.Nat,
		tickLower: IDL.Int
	});
	const Page_1 = IDL.Record({
		content: IDL.Vec(UserPositionInfoWithTokenAmount),
		offset: IDL.Nat,
		limit: IDL.Nat,
		totalElements: IDL.Nat
	});
	const Result_10 = IDL.Variant({ ok: Page_1, err: Error });
	const UserPositionInfoWithId = IDL.Record({
		id: IDL.Nat,
		tickUpper: IDL.Int,
		tokensOwed0: IDL.Nat,
		tokensOwed1: IDL.Nat,
		feeGrowthInside1LastX128: IDL.Nat,
		liquidity: IDL.Nat,
		feeGrowthInside0LastX128: IDL.Nat,
		tickLower: IDL.Int
	});
	const Page = IDL.Record({
		content: IDL.Vec(UserPositionInfoWithId),
		offset: IDL.Nat,
		limit: IDL.Nat,
		totalElements: IDL.Nat
	});
	const Result_9 = IDL.Variant({ ok: Page, err: Error });
	const Result_8 = IDL.Variant({
		ok: IDL.Vec(UserPositionInfoWithId),
		err: Error
	});
	const Result_7 = IDL.Variant({
		ok: IDL.Record({ balance0: IDL.Nat, balance1: IDL.Nat }),
		err: Error
	});
	const WithdrawErrorLog = IDL.Record({
		token: Token,
		time: IDL.Int,
		user: IDL.Principal,
		amount: IDL.Nat
	});
	const Result_6 = IDL.Variant({
		ok: IDL.Vec(IDL.Tuple(IDL.Nat, WithdrawErrorLog)),
		err: Error
	});
	const IncreaseLiquidityArgs = IDL.Record({
		positionId: IDL.Nat,
		amount0Desired: IDL.Text,
		amount1Desired: IDL.Text
	});
	const PoolMetadata = IDL.Record({
		fee: IDL.Nat,
		key: IDL.Text,
		sqrtPriceX96: IDL.Nat,
		tick: IDL.Int,
		liquidity: IDL.Nat,
		token0: Token,
		token1: Token,
		maxLiquidityPerTick: IDL.Nat,
		nextPositionId: IDL.Nat
	});
	const Result_5 = IDL.Variant({ ok: PoolMetadata, err: Error });
	const MintArgs = IDL.Record({
		fee: IDL.Nat,
		tickUpper: IDL.Int,
		token0: IDL.Text,
		token1: IDL.Text,
		amount0Desired: IDL.Text,
		amount1Desired: IDL.Text,
		tickLower: IDL.Int
	});
	const SwapArgs = IDL.Record({
		amountIn: IDL.Text,
		zeroForOne: IDL.Bool,
		amountOutMinimum: IDL.Text
	});
	const Result_4 = IDL.Variant({
		ok: IDL.Record({ tokensOwed0: IDL.Nat, tokensOwed1: IDL.Nat }),
		err: Error
	});
	const Result_3 = IDL.Variant({ ok: IDL.Int, err: Error });
	const WithdrawArgs = IDL.Record({
		fee: IDL.Nat,
		token: IDL.Text,
		amount: IDL.Nat
	});
	const SwapPool = IDL.Service({
		allTokenBalance: IDL.Func([IDL.Nat, IDL.Nat], [Result_24], ['query']),
		approvePosition: IDL.Func([IDL.Principal, IDL.Nat], [Result_2], []),
		batchRefreshIncome: IDL.Func([IDL.Vec(IDL.Nat)], [Result_23], ['query']),
		checkOwnerOfUserPosition: IDL.Func([IDL.Principal, IDL.Nat], [Result_2], ['query']),
		claim: IDL.Func([ClaimArgs], [Result_22], []),
		decreaseLiquidity: IDL.Func([DecreaseLiquidityArgs], [Result_22], []),
		deposit: IDL.Func([DepositArgs], [Result], []),
		depositAllAndMint: IDL.Func([DepositAndMintArgs], [Result], []),
		depositFrom: IDL.Func([DepositArgs], [Result], []),
		getAdmins: IDL.Func([], [IDL.Vec(IDL.Principal)], ['query']),
		getAvailabilityState: IDL.Func(
			[],
			[
				IDL.Record({
					whiteList: IDL.Vec(IDL.Principal),
					available: IDL.Bool
				})
			],
			['query']
		),
		getClaimLog: IDL.Func([], [IDL.Vec(IDL.Text)], ['query']),
		getCycleInfo: IDL.Func([], [Result_21], []),
		getMistransferBalance: IDL.Func([Token], [Result], []),
		getPosition: IDL.Func([GetPositionArgs], [Result_20], ['query']),
		getPositions: IDL.Func([IDL.Nat, IDL.Nat], [Result_19], ['query']),
		getSwapRecordState: IDL.Func([], [Result_18], ['query']),
		getTickInfos: IDL.Func([IDL.Nat, IDL.Nat], [Result_17], ['query']),
		getTicks: IDL.Func([IDL.Nat, IDL.Nat], [Result_16], ['query']),
		getTokenAmountState: IDL.Func([], [Result_15], ['query']),
		getTokenBalance: IDL.Func([], [IDL.Record({ token0: IDL.Nat, token1: IDL.Nat })], []),
		getTokenMeta: IDL.Func(
			[],
			[
				IDL.Record({
					token0: IDL.Vec(IDL.Tuple(IDL.Text, Value)),
					token1: IDL.Vec(IDL.Tuple(IDL.Text, Value))
				})
			],
			[]
		),
		getTransferLogs: IDL.Func([], [Result_14], ['query']),
		getUserByPositionId: IDL.Func([IDL.Nat], [Result_1], ['query']),
		getUserPosition: IDL.Func([IDL.Nat], [Result_13], ['query']),
		getUserPositionIds: IDL.Func([], [Result_12], ['query']),
		getUserPositionIdsByPrincipal: IDL.Func([IDL.Principal], [Result_11], ['query']),
		getUserPositionWithTokenAmount: IDL.Func([IDL.Nat, IDL.Nat], [Result_10], ['query']),
		getUserPositions: IDL.Func([IDL.Nat, IDL.Nat], [Result_9], ['query']),
		getUserPositionsByPrincipal: IDL.Func([IDL.Principal], [Result_8], ['query']),
		getUserUnusedBalance: IDL.Func([IDL.Principal], [Result_7], ['query']),
		getVersion: IDL.Func([], [IDL.Text], ['query']),
		getWithdrawErrorLog: IDL.Func([], [Result_6], ['query']),
		increaseLiquidity: IDL.Func([IncreaseLiquidityArgs], [Result], []),
		init: IDL.Func([IDL.Nat, IDL.Int, IDL.Nat], [], []),
		metadata: IDL.Func([], [Result_5], ['query']),
		mint: IDL.Func([MintArgs], [Result], []),
		quote: IDL.Func([SwapArgs], [Result], ['query']),
		quoteForAll: IDL.Func([SwapArgs], [Result], ['query']),
		refreshIncome: IDL.Func([IDL.Nat], [Result_4], ['query']),
		removeErrorTransferLog: IDL.Func([IDL.Nat, IDL.Bool], [], []),
		removeWithdrawErrorLog: IDL.Func([IDL.Nat, IDL.Bool], [], []),
		resetTokenAmountState: IDL.Func([IDL.Nat, IDL.Nat, IDL.Nat, IDL.Nat], [], []),
		setAdmins: IDL.Func([IDL.Vec(IDL.Principal)], [], []),
		setAvailable: IDL.Func([IDL.Bool], [], []),
		setWhiteList: IDL.Func([IDL.Vec(IDL.Principal)], [], []),
		sumTick: IDL.Func([], [Result_3], ['query']),
		swap: IDL.Func([SwapArgs], [Result], []),
		transferPosition: IDL.Func([IDL.Principal, IDL.Principal, IDL.Nat], [Result_2], []),
		upgradeTokenStandard: IDL.Func([IDL.Principal], [Result_1], []),
		withdraw: IDL.Func([WithdrawArgs], [Result], []),
		withdrawMistransferBalance: IDL.Func([Token], [Result], [])
	});
	return SwapPool;
};
export const init = () => {
	const Token = IDL.Record({ address: IDL.Text, standard: IDL.Text });
	return [Token, Token, IDL.Principal, IDL.Principal, IDL.Principal];
};
