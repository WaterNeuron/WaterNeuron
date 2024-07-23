import { Principal } from '@dfinity/principal';
import type {
	Allowance,
	AllowanceArgs,
	Account,
	ApproveArgs,
	ApproveResult,
	ApproveError
} from '../declarations/icrc_ledger/icrc_ledger.did';
import type { Result_1, Result_2 } from '../declarations/water_neuron/water_neuron.did';
import { Asset, AssetType, bigintE8sToNumber, displayUsFormat } from '$lib';
import type {
	TransferResult,
	Icrc1TransferResult
} from '../declarations/nns-ledger/nns-ledger.did';
import type { _SERVICE as icpLedgerInterface } from '../declarations/nns-ledger/nns-ledger.did';
import type { _SERVICE as nicpLedgerInterface } from '../declarations/icrc_ledger/icrc_ledger.did';
import { CANISTER_ID_WATER_NEURON } from './authentification';

const DEFAULT_ERROR_MESSAGE: string = 'Unknown result, please refresh the page.';

export interface ApprovalResult {
	granted: boolean;
	message?: string;
}

export function handleApproveResult(result: ApproveResult): ApprovalResult {
	const key = Object.keys(result)[0] as keyof ApproveResult;
	switch (key) {
		case 'Ok':
			return { granted: true };
		case 'Err': {
			const error = result[key];
			const errorKey = Object.keys(result[key])[0];

			switch (errorKey) {
				case 'GenericError':
					return {
						granted: false,
						message: `Generic Error: ${error[errorKey]['message']}`
					};

				case 'TemporarilyUnavailable':
					return { granted: false, message: 'Ledger is temporarily unavailable.' };

				case 'AllowanceChanged':
					return {
						granted: false,
						message: `Insufficient allowance: ${displayUsFormat(bigintE8sToNumber(error[errorKey]['current_allowance']))}`
					};

				case 'Expired':
					return {
						granted: false,
						message: `Approval expired: ${error[errorKey]['ledger_time']}`
					};

				case 'Duplicate':
					return {
						granted: false,
						message: `Duplicate transfer of: ${error[errorKey]['duplicate_of']}`
					};

				case 'BadFee':
					return {
						granted: false,
						message: `Bad fee, expected: ${displayUsFormat(bigintE8sToNumber(error[errorKey]['expected_fee']))}`
					};

				case 'CreatedInFuture':
					return {
						granted: false,
						message: `Created in future: ${error[errorKey]['ledger_time']}`
					};

				case 'TooOld':
					return { granted: false, message: `The transfer is too old.` };

				case 'InsufficientFunds':
					return {
						granted: false,
						message: `Insufficient funds, balance: ${displayUsFormat(bigintE8sToNumber(error[errorKey]['balance']))}`
					};

				default:
					return { granted: false, message: DEFAULT_ERROR_MESSAGE };
			}
		}
		default:
			return { granted: false, message: DEFAULT_ERROR_MESSAGE };
	}
}

export async function nicpTransferApproved(
	amount: bigint,
	account: Account,
	nicpLedger: nicpLedgerInterface
): Promise<ApprovalResult> {
	const spender = {
		owner: Principal.fromText(CANISTER_ID_WATER_NEURON),
		subaccount: []
	} as Account;
	const allowanceResult: Allowance = await nicpLedger.icrc2_allowance({
		account,
		spender
	} as AllowanceArgs);
	const allowance = allowanceResult['allowance'];
	if (amount > allowance) {
		try {
			const approveResult: ApproveResult = await nicpLedger.icrc2_approve({
				spender,
				fee: [],
				memo: [],
				from_subaccount: [],
				created_at_time: [],
				expires_at: [],
				expected_allowance: [],
				amount: amount * BigInt(3)
			} as ApproveArgs);
			return handleApproveResult(approveResult);
		} catch (error) {
			return { granted: false, message: `${error}.` };
		}
	}
	return { granted: true };
}

export async function icpTransferApproved(
	amount: bigint,
	account: Account,
	icpLedger: icpLedgerInterface
): Promise<ApprovalResult> {
	const spender = {
		owner: Principal.fromText(CANISTER_ID_WATER_NEURON),
		subaccount: []
	} as Account;
	const allowanceResult: Allowance = await icpLedger.icrc2_allowance({
		account,
		spender
	} as AllowanceArgs);
	const allowance = allowanceResult['allowance'];
	if (amount > allowance) {
		try {
			const approveResult: ApproveResult = await icpLedger.icrc2_approve({
				spender,
				fee: [],
				memo: [],
				from_subaccount: [],
				created_at_time: [],
				expires_at: [],
				expected_allowance: [],
				amount: amount * BigInt(3)
			} as ApproveArgs);
			return handleApproveResult(approveResult);
		} catch (error) {
			return { granted: false, message: `${error}` };
		}
	}
	return { granted: true };
}

export interface ConversionResult {
	success: boolean;
	message: string;
}

export function handleStakeResult(result: Result_1): ConversionResult {
	const key = Object.keys(result)[0] as keyof Result_1;
	switch (key) {
		case 'Ok':
			return {
				success: true,
				message: `Successful conversion at <a target='_blank' style="text-decoration: underline; color: var(--text-color);" href=https://dashboard.internetcomputer.org/transaction/${result[key]['block_index']}>block index ${result[key]['block_index']}</a>.`
			};
		case 'Err':
			const error = result[key];
			const errorKey = Object.keys(result[key])[0];

			switch (errorKey) {
				case 'GenericError':
					return { success: false, message: `Generic Error: ${error[errorKey]['message']}` };

				case 'TransferError':
					const transferError = error[errorKey];
					const transferErrorKey = Object.keys(error[errorKey])[0];

					switch (transferErrorKey) {
						case 'GenericError':
							return {
								success: false,
								message: `Generic Error: ${transferError[transferErrorKey]['message']}`
							};

						case 'TemporarilyUnavailable':
							return { success: false, message: 'Ledger is temporarily unavailable.' };

						case 'BadBurn':
							return {
								success: false,
								message: `Bad burn. Minimum burn amount: ${displayUsFormat(bigintE8sToNumber(transferError[transferErrorKey]['min_burn_amount']))}`
							};

						case 'Duplicate':
							return {
								success: false,
								message: `Duplicate transfer of: ${transferError[transferErrorKey]['duplicate_of']}`
							};

						case 'BadFee':
							return {
								success: false,
								message: `Bad fee, expected ${displayUsFormat(bigintE8sToNumber(transferError[transferErrorKey]['expected_fee']))}`
							};

						case 'CreatedInFuture':
							return {
								success: false,
								message: `Created in future: ${transferError[transferErrorKey]['ledger_time']}`
							};

						case 'TooOld':
							return { success: false, message: `The transfer is too old.` };

						case 'InsufficientFunds':
							return {
								success: false,
								message: `Insufficient funds, current balance: ${displayUsFormat(bigintE8sToNumber(transferError[transferErrorKey]['balance']))}`
							};

						default:
							return { success: false, message: 'Unknown transferfrom error.' };
					}

				case 'AmountTooLow':
					return {
						success: false,
						message: `Amount too low. Should be greater than ${displayUsFormat(bigintE8sToNumber(error[errorKey]['minimum_amount_e8s']))}`
					};

				case 'TransferFromError':
					const transferFromError = error[errorKey];
					const transferFromErrorKey = Object.keys(error[errorKey])[0];

					switch (transferFromErrorKey) {
						case 'GenericError':
							return {
								success: false,
								message: `Generic Error: ${transferFromError[transferFromErrorKey]['message']}`
							};

						case 'TemporarilyUnavailable':
							return { success: false, message: 'Ledger is temporarily unavailable.' };

						case 'InsufficientAllowance':
							return {
								success: false,
								message: `Insufficient allowance: ${displayUsFormat(bigintE8sToNumber(transferFromError[transferFromErrorKey]['allowance']))}`
							};

						case 'BadBurn':
							return {
								success: false,
								message: `Bad burn, minimum burn amount: ${displayUsFormat(bigintE8sToNumber(transferFromError[transferFromErrorKey]['min_burn_amount']))}`
							};

						case 'Duplicate':
							return {
								success: false,
								message: `Duplicate transfer of: ${transferFromError[transferFromErrorKey]['duplicate_of']}`
							};

						case 'BadFee':
							return {
								success: false,
								message: `Bad fee, expected: ${displayUsFormat(bigintE8sToNumber(transferFromError[transferFromErrorKey]['expected_fee']))}`
							};

						case 'CreatedInFuture':
							return {
								success: false,
								message: `Created in future: ${transferFromError[transferFromErrorKey]['ledger_time']}`
							};

						case 'TooOld':
							return { success: false, message: `The transfer is too old.` };

						case 'InsufficientFunds':
							return {
								success: false,
								message: `Insufficient funds, balance: ${displayUsFormat(bigintE8sToNumber(transferFromError[transferFromErrorKey]['balance']))}`
							};

						default:
							return { success: false, message: 'Unknown transferfrom error.' };
					}

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
		default:
			return { success: false, message: 'Unknown Error.' };
	}
}

export function handleRetrieveResult(result: Result_2): ConversionResult {
	const key = Object.keys(result)[0] as keyof Result_2;

	switch (key) {
		case 'Ok':
			return {
				success: true,
				message: `Successful conversion at block index ${result[key]['block_index']}. Follow your <a style="text-decoration: underline; color: var(--text-color);" href='/wallet'>withdrawal status</a>.`
			};
		case 'Err':
			const error = result[key];
			const errorKey = Object.keys(result[key])[0];
			switch (errorKey) {
				case 'GenericError':
					return { success: false, message: `Generic Error: ${error[errorKey]['message']}` };

				case 'TransferError':
					const transferError = error[errorKey];
					const transferErrorKey = Object.keys(error[errorKey])[0];

					switch (transferErrorKey) {
						case 'GenericError':
							return {
								success: false,
								message: `Generic Error: ${transferError[transferErrorKey]['message']}`
							};

						case 'TemporarilyUnavailable':
							return { success: false, message: 'Ledger is temporarily unavailable.' };

						case 'BadBurn':
							return {
								success: false,
								message: `Bad burn, minimum burn amount: ${displayUsFormat(bigintE8sToNumber(transferError[transferErrorKey]['min_burn_amount']))}`
							};

						case 'Duplicate':
							return {
								success: false,
								message: `Duplicate, already occurring transfer: ${transferError[transferErrorKey]['duplicate_of']}`
							};

						case 'BadFee':
							return {
								success: false,
								message: `Bad fee, expected: ${displayUsFormat(bigintE8sToNumber(transferError[transferErrorKey]['expected_fee']))}`
							};

						case 'CreatedInFuture':
							return {
								success: false,
								message: `Created in future: ${transferError[transferErrorKey]['ledger_time']}`
							};

						case 'TooOld':
							return { success: false, message: `Transfer is too old.` };

						case 'InsufficientFunds':
							return {
								success: false,
								message: `Insufficient funds, balance: ${displayUsFormat(bigintE8sToNumber(transferError[transferErrorKey]['balance']))}`
							};

						default:
							return { success: false, message: 'Unknown transferfrom error.' };
					}

				case 'AmountTooLow':
					return {
						success: false,
						message: `Amount too low, minimum amount: ${displayUsFormat(bigintE8sToNumber(error[errorKey]['minimum_amount_e8s']))}`
					};

				case 'TransferFromError':
					const transferFromError = error[errorKey][0];
					const transferFromErrorKey = Object.keys(error[errorKey][0])[0];

					switch (transferFromErrorKey) {
						case 'GenericError':
							return {
								success: false,
								message: `Generic Error: ${transferFromError[transferFromErrorKey]['message']}`
							};

						case 'TemporarilyUnavailable':
							return { success: false, message: 'Ledger is temporarily unavailable.' };

						case 'InsufficientAllowance':
							return {
								success: false,
								message: `Insufficient allowance, current allowance: ${displayUsFormat(bigintE8sToNumber(transferFromError[transferFromErrorKey]['allowance']))}`
							};

						case 'BadBurn':
							return {
								success: false,
								message: `Bad burn, minimum burn amount: ${displayUsFormat(bigintE8sToNumber(transferFromError[transferFromErrorKey]['min_burn_amount']))}`
							};

						case 'Duplicate':
							return {
								success: false,
								message: `Duplicate. Already occurring transfer: ${transferFromError[transferFromErrorKey]['duplicate_of']}`
							};

						case 'BadFee':
							return {
								success: false,
								message: `Bad fee. Expected fee: ${displayUsFormat(bigintE8sToNumber(transferFromError[transferFromErrorKey]['expected_fee']))}`
							};

						case 'CreatedInFuture':
							return {
								success: false,
								message: `Created in future: ${transferFromError[transferFromErrorKey]['ledger_time']}`
							};

						case 'TooOld':
							return { success: false, message: `The transfer is too old.` };

						case 'InsufficientFunds':
							return {
								success: false,
								message: `Insufficient funds. Balance: ${displayUsFormat(bigintE8sToNumber(transferFromError[transferFromErrorKey]['balance']))}`
							};

						default:
							return { success: false, message: 'Unknown transferfrom error.' };
					}

				case 'GuardError':
					const guardErrorKey = Object.keys(error[errorKey])[0];

					switch (guardErrorKey) {
						case 'AlreadyProcessing':
							return { success: false, message: `Guard Error. Conversion already processing.` };
						case 'TooManyConcurrentRequests':
							return { success: false, message: `Guard Error. Too many concurrent requests.` };
					}

				default:
					return { success: false, message: DEFAULT_ERROR_MESSAGE };
			}
		default:
			return { success: false, message: 'Unknown Error.' };
	}
}

export function handleTransferResult(result: TransferResult): ConversionResult {
	const key = Object.keys(result)[0] as keyof TransferResult;

	switch (key) {
		case 'Ok':
			return {
				success: true,
				message: `Successful transfer at <a target='_blank' style="text-decoration: underline; color: var(--text-color);" href=https://dashboard.internetcomputer.org/transaction/${result[key]}>block index ${result[key]}</a>.`
			};
		case 'Err': {
			const error = result[key];
			const errorKey = Object(error[key])[0];

			switch (errorKey) {
				case 'TxTooOld':
					return { success: false, message: `The transfer is too old.` };
				case 'BadFee':
					return {
						success: false,
						message: `Bad fee, expected: ${displayUsFormat(bigintE8sToNumber(error[errorKey]['expected_fee']))}`
					};
				case 'TxDuplicate':
					return {
						success: false,
						message: `Duplicate, already occurring transfer: ${error[errorKey]['duplicate_of']}`
					};
				case 'TxCreatedInFuture':
					return {
						success: false,
						message: `The transfer is created in future.`
					};

				case 'InsufficientFunds':
					return {
						success: false,
						message: `Insufficient funds, balance: ${displayUsFormat(bigintE8sToNumber(error[errorKey]['balance']))}`
					};
				default:
					return {
						success: false,
						message: `Unknown Error. Try again and verify the Account Id.`
					};
			}
		}

		default:
			return {
				success: false,
				message: `Unknown Error. Try again.`
			};
	}
}

export function handleIcrcTransferResult(
	result: Icrc1TransferResult,
	asset: Asset
): ConversionResult {
	const key = Object.keys(result)[0] as keyof TransferResult;

	switch (key) {
		case 'Ok':
			switch (asset.type) {
				case AssetType.nICP:
					return {
						success: true,
						message: `Successful transfer at block index ${result[key]}.`
					};
				default:
					return {
						success: true,
						message: `Successful transfer at <a target='_blank' style="text-decoration: underline; color: var(--text-color);" href=${asset.getDashboardUrl()}${result[key]}>block index ${result[key]}</a>.`
					};
			}

		case 'Err': {
			const error = result[key];
			const errorKey = Object(error[key])[0];

			switch (errorKey) {
				case 'TooOld':
					return { success: false, message: `The transfer is too old.` };
				case 'BadFee':
					return {
						success: false,
						message: `Bad fee, expected: ${displayUsFormat(bigintE8sToNumber(error[errorKey]['min_burn_amount']))}`
					};
				case 'Duplicate':
					return {
						success: false,
						message: `Duplicate transfer of: ${error[errorKey]['duplicate_of']}`
					};
				case 'CreatedInFuture':
					return {
						success: false,
						message: `Created in future: ${error[errorKey]['ledger_time']}`
					};

				case 'InsufficientFunds':
					return {
						success: false,
						message: `Insufficient funds, balance: ${displayUsFormat(bigintE8sToNumber(error[errorKey]['balance']))}`
					};

				case 'GenericError':
					return { success: false, message: `Generic Error: ${error[errorKey]['message']}` };

				case 'TemporarilyUnavailable':
					return { success: false, message: 'Ledger is temporarily unavailable.' };

				case 'BadBurn':
					return {
						success: false,
						message: `Bad burn, minimum burn amount: ${displayUsFormat(bigintE8sToNumber(error[errorKey]['min_burn_amount']))}`
					};

				default:
					return {
						success: false,
						message: `Unknown Error. Try again and verify the Account Id.`
					};
			}
		}

		default:
			return {
				success: false,
				message: DEFAULT_ERROR_MESSAGE
			};
	}
}
