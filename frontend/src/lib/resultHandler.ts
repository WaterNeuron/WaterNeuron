import { Principal } from '@dfinity/principal';
import type {
	Allowance,
	AllowanceArgs,
	Account,
	ApproveArgs,
	Result_2 as ApproveResult,
	ApproveError
} from '../declarations/icrc_ledger/icrc_ledger.did';
import type {
	ConversionError,
	Result as CancelResult,
	Result_3 as IcpToNicpResult,
	Result_4 as NicpToIcpResult,
	TransferError,
	TransferFromError,
	MergeResponse,
	NeuronInfo,
	CancelWithdrawalError
} from '../declarations/water_neuron/water_neuron.did';
import type {
	BoomerangError,
	Result as SnsIcpDepositResult,
	Result_2 as SnsRetrieveNicpResult
} from '../declarations/boomerang/boomerang.did';
import type { Error as IcpswapError } from '$lib/../declarations/icpswap_pool/icpswap_pool.did';
import { assetToDashboardUrl, bigintE8sToNumber, displayUsFormat } from '$lib';
import type {
	TransferResult,
	Icrc1TransferResult,
	_SERVICE as icpLedgerInterface
} from '../declarations/icp_ledger/icp_ledger.did';
import type { _SERVICE as icrcLedgerInterface } from '../declarations/icrc_ledger/icrc_ledger.did';
import { CANISTER_ID_WATER_NEURON } from './authentification';
import { CanisterActor } from './state';

export const DEFAULT_ERROR_MESSAGE: string = 'Unknown result, please refresh the page.';

export interface ToastResult {
	success: boolean;
	message: string;
}

function handleApproveError(error: ApproveError): ToastResult {
	const key = Object.keys(error)[0] as keyof ApproveError;

	switch (key) {
		case 'GenericError':
			return {
				success: false,
				message: `Generic Error: ${error[key]['message']}`
			};

		case 'TemporarilyUnavailable':
			return { success: false, message: 'Ledger is temporarily unavailable.' };

		case 'AllowanceChanged':
			return {
				success: false,
				message: `Insufficient allowance: ${displayUsFormat(bigintE8sToNumber(error[key]['current_allowance']))}`
			};

		case 'Expired':
			return {
				success: false,
				message: `Approval expired: ${error[key]['ledger_time']}`
			};

		case 'Duplicate':
			return {
				success: false,
				message: `Duplicate transfer of: ${error[key]['duplicate_of']}`
			};

		case 'BadFee':
			return {
				success: false,
				message: `Bad fee, expected: ${displayUsFormat(bigintE8sToNumber(error[key]['expected_fee']))}`
			};

		case 'CreatedInFuture':
			return {
				success: false,
				message: `Created in future: ${error[key]['ledger_time']}`
			};

		case 'TooOld':
			return { success: false, message: `The transfer is too old.` };

		case 'InsufficientFunds':
			return {
				success: false,
				message: `Insufficient funds, balance: ${displayUsFormat(bigintE8sToNumber(error[key]['balance']))}`
			};

		default:
			return { success: false, message: DEFAULT_ERROR_MESSAGE };
	}
}

export function handleApproveResult(result: ApproveResult): ToastResult {
	const key = Object.keys(result)[0] as keyof ApproveResult;
	switch (key) {
		case 'Ok':
			return { success: true, message: '' };
		case 'Err': {
			return handleApproveError(result[key]);
		}
		default:
			return { success: false, message: DEFAULT_ERROR_MESSAGE };
	}
}

export async function nicpTransferApproved(
	amount: bigint,
	account: Account,
	nicpLedger: CanisterActor<icrcLedgerInterface>
): Promise<ToastResult> {
	if (!nicpLedger.authenticatedActor) return { success: false, message: 'User not authenticated.' };

	const spender = {
		owner: Principal.fromText(CANISTER_ID_WATER_NEURON),
		subaccount: []
	} as Account;
	const allowanceResult: Allowance = await nicpLedger.anonymousActor.icrc2_allowance({
		account,
		spender
	} as AllowanceArgs);
	const allowance = allowanceResult['allowance'];
	if (amount > allowance) {
		try {
			// Two weeks later, in nanoseconds
			const expiryDate = BigInt(Date.now() * 1_000_000 + 1_209_600_000_000_000);
			const approveResult: ApproveResult = await nicpLedger.authenticatedActor.icrc2_approve({
				spender,
				fee: [],
				memo: [],
				from_subaccount: [],
				created_at_time: [],
				expires_at: [expiryDate],
				expected_allowance: [],
				amount: amount * BigInt(3)
			} as ApproveArgs);
			return handleApproveResult(approveResult);
		} catch (error) {
			return { success: false, message: `${error}.` };
		}
	}
	return { success: true, message: '' };
}

export async function icpTransferApproved(
	amount: bigint,
	account: Account,
	icpLedger: CanisterActor<icpLedgerInterface>
): Promise<ToastResult> {
	if (!icpLedger.authenticatedActor) return { success: false, message: 'User not authenticated.' };
	const spender = {
		owner: Principal.fromText(CANISTER_ID_WATER_NEURON),
		subaccount: []
	} as Account;
	const allowanceResult: Allowance = await icpLedger.anonymousActor.icrc2_allowance({
		account,
		spender
	} as AllowanceArgs);
	const allowance = allowanceResult['allowance'];
	if (amount > allowance) {
		try {
			// Two weeks later, in nanoseconds
			const expiryDate = BigInt(Date.now() * 1_000_000 + 1_209_600_000_000_000);
			const approveResult: ApproveResult = await icpLedger.authenticatedActor.icrc2_approve({
				spender,
				fee: [],
				memo: [],
				from_subaccount: [],
				created_at_time: [],
				expires_at: [expiryDate],
				expected_allowance: [],
				amount: amount * BigInt(3)
			} as ApproveArgs);
			return handleApproveResult(approveResult);
		} catch (error) {
			return { success: false, message: `${error}` };
		}
	}
	return { success: true, message: '' };
}

function handleTransferError(error: TransferError) {
	const key = Object.keys(error)[0] as keyof TransferError;

	switch (key) {
		case 'GenericError':
			return {
				success: false,
				message: `Generic Error: ${error[key]['message']}`
			};

		case 'TemporarilyUnavailable':
			return { success: false, message: 'Ledger is temporarily unavailable.' };

		case 'BadBurn':
			return {
				success: false,
				message: `Bad burn. Minimum burn amount: ${displayUsFormat(bigintE8sToNumber(error[key]['min_burn_amount']))}`
			};

		case 'Duplicate':
			return {
				success: false,
				message: `Duplicate transfer of: ${error[key]['duplicate_of']}`
			};

		case 'BadFee':
			return {
				success: false,
				message: `Bad fee, expected ${displayUsFormat(bigintE8sToNumber(error[key]['expected_fee']))}`
			};

		case 'CreatedInFuture':
			return {
				success: false,
				message: `Created in future: ${error[key]['ledger_time']}`
			};

		case 'TooOld':
			return { success: false, message: `The transfer is too old.` };

		case 'InsufficientFunds':
			return {
				success: false,
				message: `Insufficient funds, current balance: ${displayUsFormat(bigintE8sToNumber(error[key]['balance']))}`
			};

		default:
			return { success: false, message: DEFAULT_ERROR_MESSAGE };
	}
}

function handleTransferFromError(error: TransferFromError) {
	const key = Object.keys(error)[0] as keyof TransferFromError;

	switch (key) {
		case 'GenericError':
			return {
				success: false,
				message: `Generic Error: ${error[key]['message']}`
			};

		case 'TemporarilyUnavailable':
			return { success: false, message: 'Ledger is temporarily unavailable.' };

		case 'InsufficientAllowance':
			return {
				success: false,
				message: `Insufficient allowance: ${displayUsFormat(bigintE8sToNumber(error[key]['allowance']))}`
			};

		case 'BadBurn':
			return {
				success: false,
				message: `Bad burn, minimum burn amount: ${displayUsFormat(bigintE8sToNumber(error[key]['min_burn_amount']))}`
			};

		case 'Duplicate':
			return {
				success: false,
				message: `Duplicate transfer of: ${error[key]['duplicate_of']}`
			};

		case 'BadFee':
			return {
				success: false,
				message: `Bad fee, expected: ${displayUsFormat(bigintE8sToNumber(error[key]['expected_fee']))}`
			};

		case 'CreatedInFuture':
			return {
				success: false,
				message: `Created in future: ${error[key]['ledger_time']}`
			};

		case 'TooOld':
			return { success: false, message: `The transfer is too old.` };

		case 'InsufficientFunds':
			return {
				success: false,
				message: `Insufficient funds, balance: ${displayUsFormat(bigintE8sToNumber(error[key]['balance']))}`
			};

		default:
			return { success: false, message: 'Unknown transferfrom error.' };
	}
}

function handleConversionError(error: ConversionError): ToastResult {
	const errorKey = Object.keys(error)[0] as keyof ConversionError;

	switch (errorKey) {
		case 'GenericError':
			return { success: false, message: `Generic Error: ${error[errorKey]['message']}` };

		case 'TransferError':
			return handleTransferError(error[errorKey]);

		case 'AmountTooLow':
			return {
				success: false,
				message: `Amount too low. Should be greater than ${displayUsFormat(bigintE8sToNumber(error[errorKey]['minimum_amount_e8s']))}`
			};

		case 'TransferFromError':
			return handleTransferFromError(error[errorKey]);
		case 'GuardError':
			const guardErrorKey = Object.keys(error[errorKey])[0];

			switch (guardErrorKey) {
				case 'AlreadyProcessing':
					return { success: false, message: `Conversion already processing.` };
				case 'TooManyConcurrentRequests':
					return { success: false, message: `Too many concurrent requests.` };
			}

		default:
			return { success: false, message: DEFAULT_ERROR_MESSAGE };
	}
}

export function handleStakeResult(result: IcpToNicpResult): ToastResult {
	const key = Object.keys(result)[0] as keyof IcpToNicpResult;
	switch (key) {
		case 'Ok':
			return {
				success: true,
				message: `Successful conversion at <a target='_blank' style="text-decoration: underline; color: var(--toast-text-color);" href=https://dashboard.internetcomputer.org/transaction/${result[key]['block_index']}>block index ${result[key]['block_index']}</a>.`
			};
		case 'Err':
			return handleConversionError(result[key]);
		default:
			return { success: false, message: DEFAULT_ERROR_MESSAGE };
	}
}

export function handleUnstakeResult(result: NicpToIcpResult): ToastResult {
	const key = Object.keys(result)[0] as keyof NicpToIcpResult;

	switch (key) {
		case 'Ok':
			return {
				success: true,
				message: `Successful conversion at block index ${result[key]['block_index']}. Follow your <a style="text-decoration: underline; color: var(--toast-text-color);" href='/wallet'>withdrawal status</a>.`
			};
		case 'Err':
			return handleConversionError(result[key]);
		default:
			return { success: false, message: DEFAULT_ERROR_MESSAGE };
	}
}

function handleBoomerangError(error: BoomerangError): ToastResult {
	const key = Object.keys(error)[0] as keyof BoomerangError;

	switch (key) {
		case 'GenericError':
			return { success: false, message: `Generic Error: ${error[key]['message']}` };

		case 'TransferError':
			return handleTransferError(error[key]);

		case 'ConversionError':
			return handleConversionError(error[key]);

		case 'ApproveError':
			return handleApproveError(error[key]);

		case 'NotEnoughICP':
			return { success: false, message: `Sorry, there are not enough funds in this account.` };

		default:
			return { success: false, message: DEFAULT_ERROR_MESSAGE };
	}
}

export function handleSnsIcpDepositResult(result: SnsIcpDepositResult): ToastResult {
	const key = Object.keys(result)[0] as keyof SnsIcpDepositResult;

	switch (key) {
		case 'Ok':
			return {
				success: true,
				message: `Successful conversion at <a target='_blank' style="text-decoration: underline; color: var(--toast-text-color);" href=https://dashboard.internetcomputer.org/transaction/${result[key]['block_index']}>block index ${result[key]['block_index']}</a>.`
			};
		case 'Err':
			return handleBoomerangError(result[key]);

		default:
			return { success: false, message: DEFAULT_ERROR_MESSAGE };
	}
}

export function handleSnsRetrieveNicpResult(result: SnsRetrieveNicpResult): ToastResult {
	const key = Object.keys(result)[0] as keyof SnsRetrieveNicpResult;

	switch (key) {
		case 'Ok':
			return {
				success: true,
				message: `Successful conversion at <a target='_blank' style="text-decoration: underline; color: var(--toast-text-color);" href="https://221bravo.app/blocks/NICP">block index ${result[key]}</a>.`
			};
		case 'Err':
			return handleBoomerangError(result[key]);

		default:
			return { success: false, message: DEFAULT_ERROR_MESSAGE };
	}
}

export function handleTransferResult(result: TransferResult): ToastResult {
	const key = Object.keys(result)[0] as keyof TransferResult;

	switch (key) {
		case 'Ok':
			return {
				success: true,
				message: `Successful transfer at <a target='_blank' style="text-decoration: underline; color: var(--toast-text-color);" href=https://dashboard.internetcomputer.org/transaction/${result[key]}>block index ${result[key]}</a>.`
			};
		case 'Err':
			return handleTransferError(result[key]);

		default:
			return {
				success: false,
				message: DEFAULT_ERROR_MESSAGE
			};
	}
}

export function handleIcrcTransferResult(
	result: Icrc1TransferResult,
	asset: 'ICP' | 'nICP' | 'WTN'
): ToastResult {
	const key = Object.keys(result)[0] as keyof Icrc1TransferResult;

	switch (key) {
		case 'Ok':
			switch (asset) {
				case 'nICP':
					return {
						success: true,
						message: `Successful transfer at block index ${result[key]}.`
					};
				default:
					return {
						success: true,
						message: `Successful transfer at <a target='_blank' style="text-decoration: underline; color: var(--toast-text-color);" href=${assetToDashboardUrl(asset)}${result[key]}>block index ${result[key]}</a>.`
					};
			}

		case 'Err':
			return handleTransferFromError(result[key]);

		default:
			return {
				success: false,
				message: DEFAULT_ERROR_MESSAGE
			};
	}
}

function handleCancelError(error: CancelWithdrawalError): ToastResult {
	const key = Object.keys(error)[0] as keyof CancelWithdrawalError;

	switch (key) {
		case 'GenericError':
			return {
				success: false,
				message: `Generic Error: ${error[key]['message']}`
			};
		case 'TooLate':
			return {
				success: false,
				message: 'The neuron is too close to disbursement.'
			};
		case 'BadCommand':
			return {
				success: false,
				message: `The protocol did not trigger the neuron merge. Try again.`
			};
		case 'UnknownTimeLeft':
			return {
				success: false,
				message: `Unable to fetch the neuron information. Try again.`
			};
		case 'BadCaller':
			return {
				success: false,
				message: 'You are not the owner of this neuron. Check neuron ownership.'
			};
		case 'MergeNeuronError':
			return {
				success: false,
				message: `The merge failed with error: ${error[key]}.`
			};
		case 'StopDissolvementError':
			return {
				success: false,
				message: `Failed to stop the neuron dissolvement. Try again later.`
			};
		case 'RequestNotFound':
			return {
				success: false,
				message: `Unable to find the withdrawal request. The withdrawal might have been already cancelled.`
			};
		case 'GovernanceError':
			return {
				success: false,
				message: 'Failed to merge neuron. Please, try again.'
			};
		case 'GuardError':
			const guardErrorKey = Object.keys(error[key])[0];

			switch (guardErrorKey) {
				case 'AlreadyProcessing':
					return { success: false, message: `Conversion already processing.` };
				case 'TooManyConcurrentRequests':
					return { success: false, message: `Too many concurrent requests.` };
			}
		case 'GetFullNeuronError':
			return { success: false, message: `Call failed with error: ${error[key]}` };
		default:
			return {
				success: false,
				message: DEFAULT_ERROR_MESSAGE
			};
	}
}

export function handleCancelWithdrawalResult(result: CancelResult): ToastResult {
	const key = Object.keys(result)[0] as keyof CancelResult;

	switch (key) {
		case 'Ok':
			const response: MergeResponse = result[key];
			const info: [] | [NeuronInfo] = response.source_neuron_info;
			if (info.length === 1) {
				return { success: true, message: `Successfully cancelled withdrawal.` };
			} else {
				return { success: false, message: DEFAULT_ERROR_MESSAGE };
			}
		case 'Err':
			return handleCancelError(result[key]);
		default:
			return {
				success: false,
				message: DEFAULT_ERROR_MESSAGE
			};
	}
}

export function handleIcpSwapError(error: IcpswapError): string {
	const key = Object.keys(error)[0];
	const message = Object.values(error)[0];

	switch (key) {
		case 'CommonError':
			return 'CommonError.';
		case 'InternalError':
			return `InternalError: ${message}`;
		case 'UnsupportedToken':
			return `UnsupportedToken: ${message}`;
		case 'InsufficientFunds':
			return 'Insufficient Funds.';
		default:
			return 'Unknown Error.';
	}
}
