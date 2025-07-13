import { Principal } from '@dfinity/principal';
import type { WithdrawalStatus } from '../declarations/water_neuron/water_neuron.did';
import { AccountIdentifier } from '@dfinity/ledger-icp';
import { decodeIcrcAccount } from '@dfinity/ledger-icrc';
import type { Account } from '../declarations/icp_ledger/icp_ledger.did';
import type {
	NeuronId,
	WithdrawalDetails
} from '$lib/../declarations/water_neuron/water_neuron.did';
import { DEV } from './env';

export const E8S = 100_000_000n;

export function displayPrincipal(principal: Principal | string | undefined) {
	if (principal === undefined) return '-/-';

	const a = principal.toString().split('-');
	return a[0] + '...' + a[a.length - 1];
}

export function displayNumber(value: number, decimals = 2): string {
	const result = Math.floor(value * 10 ** decimals) / 10 ** decimals;
	return new Intl.NumberFormat('en-US', {
		minimumFractionDigits: 0,
		maximumFractionDigits: decimals
	})
		.format(result)
		.replace(/,/g, "'");
}

export function numberToBigintE8s(x: number): bigint {
	return BigInt(Math.round(x * Number(E8S)));
}

export function bigintE8sToNumber(x: bigint): number {
	return Number(x) / Number(E8S);
}

export function assetToIconPath(asset: 'ICP' | 'nICP' | 'WTN'): string {
	switch (asset) {
		case 'ICP':
			return '/tokens/icp.webp';
		case 'nICP':
			return '/tokens/nicp.png';
		case 'WTN':
			return '/tokens/WTN.webp';
	}
}

export function assetToDashboardUrl(asset: 'ICP' | 'nICP' | 'WTN'): string {
	switch (asset) {
		case 'ICP':
			return 'https://dashboard.internetcomputer.org/transaction/';
		case 'nICP':
			return '/wallet';
		case 'WTN':
			return 'https://dashboard.internetcomputer.org/sns/jmod6-4iaaa-aaaaq-aadkq-cai/transaction/';
	}
}

export function assetToTransferFee(asset: 'ICP' | 'nICP' | 'WTN'): bigint {
	switch (asset) {
		case 'ICP':
			return 10_000n;
		case 'nICP':
			return 1_000_000n;
		case 'WTN':
			return 1_000_000n;
	}
}

export const TIERS: [number, number][] = [
	[80_000, 8],
	[160_000, 4],
	[320_000, 2],
	[640_000, 1],
	[1_280_000, 0.5],
	[2_560_000, 0.25],
	[5_120_000, 0.125]
];

export const EXPECTED_INITIAL_BALANCE = 4_480_000;

export function computeRewards(alreadyDistributed: number, converting: number): number {
	let totalRewards = 0;
	let amountToDistribute = converting;
	let cumulativeAmount = alreadyDistributed;

	for (const [threshold, rate] of TIERS) {
		const nicpThreshold = threshold;
		const allocationRate = rate;
		if (cumulativeAmount > nicpThreshold) {
			cumulativeAmount = cumulativeAmount - nicpThreshold;
			continue;
		}
		const tierAvailable = nicpThreshold - cumulativeAmount;
		cumulativeAmount = 0;
		const amountInThisTier = Math.min(amountToDistribute, tierAvailable);

		totalRewards += amountInThisTier * allocationRate;

		amountToDistribute -= amountInThisTier;

		if (amountToDistribute === 0) {
			break;
		}
	}

	return totalRewards;
}

export function renderStatus(status: WithdrawalStatus): string {
	const key = Object.keys(status)[0] as keyof WithdrawalStatus;
	switch (key) {
		case 'ConversionDone':
			return `<p>
	  Conversion done at height
	  <a
		style="color: var(--title-color)"
		target="_blank"
		rel="noreferrer"
		href="https://dashboard.internetcomputer.org/transaction/${status[key]['transfer_block_height']}"
	  >
		 ${status[key]['transfer_block_height']}
	  </a>
	</p>`;
		case 'NotFound':
			return 'Not found';
		case 'WaitingToSplitNeuron':
			return 'Waiting to Split Neuron';
		case 'WaitingDissolvement':
			return 'Waiting Dissolvement';
		case 'WaitingToStartDissolving':
			return `Waiting to Start Dissolving`;
		case 'Cancelled':
			return 'Cancelled';
		default:
			return 'Unknown Status';
	}
}

export function displayTimeLeft(created_at: number, isMobile = false) {
	const currentTimestamp = Math.floor(Date.now() / 1000);
	const sixMonthsInSeconds = 6 * 30.44 * 24 * 60 * 60;
	const timeLeft = created_at + sixMonthsInSeconds - currentTimestamp;
	const daysLeft = Math.floor(timeLeft / 60 / 60 / 24);
	const hoursLeft = Math.floor((timeLeft - daysLeft * 60 * 60 * 24) / 60 / 60);

	if (!isMobile && daysLeft > 0 && hoursLeft > 0) {
		return `${daysLeft} days and ${hoursLeft} hours left`;
	} else if (daysLeft > 0) {
		return `${daysLeft} days left`;
	} else if (hoursLeft > 0) {
		return `${hoursLeft} hours left`;
	}
	return `Less than an hour left`;
}

// Timestamp in seconds.
export async function fetchNeuronCreationTimestamp(neuron_id: NeuronId): Promise<number> {
	// October 28th, 2024. 3:12 PM
	const localTestingTimestamp = Date.now();
	if (process.env.DFX_NETWORK !== 'ic') return localTestingTimestamp;
	try {
		const response = await fetch(
			`https://ic-api.internetcomputer.org/api/v3/neurons/${neuron_id.id}`
		);
		if (!response.ok) {
			throw new Error('Network response was not ok');
		}
		const data = await response.json();
		const neuron_created_at = data['created_timestamp_seconds'];
		return Number(neuron_created_at);
	} catch (error) {
		throw new Error('[fetchNeuronCreationTimestamp] Failed to fetch with error: ' + error);
	}
}

export const isMobile = typeof window !== 'undefined' && window.innerWidth <= 767;

export function isPrincipalValid(input: string): boolean {
	try {
		Principal.fromText(input);
		return true;
	} catch (_) {
		return false;
	}
}

export function principalToHex(principalString: string): string {
	try {
		const principal = Principal.fromText(principalString);
		return AccountIdentifier.fromPrincipal({ principal: principal }).toHex();
	} catch (error) {
		console.log('[principalToHex]', error);
		return '';
	}
}

export function displayNeuronId(neuronId: [] | [NeuronId], truncate = true): string {
	if (neuronId.length == 0) {
		return 'Not Set';
	} else if (truncate) {
		const id = neuronId[0].id.toString();
		return id.substring(0, 4) + '...' + id.substring(id.length - 5, id.length - 1);
	} else {
		return neuronId[0].id.toString();
	}
}

export function getMaybeAccount(accountString: string): Account | AccountIdentifier | undefined {
	try {
		if (accountString.length === 64) {
			return AccountIdentifier.fromHex(accountString);
		}
		const icrcAccount = decodeIcrcAccount(accountString);

		if (icrcAccount.subaccount) {
			return { owner: icrcAccount.owner, subaccount: [icrcAccount.subaccount] } as Account;
		} else {
			return { owner: icrcAccount.owner, subaccount: [] } as Account;
		}
	} catch (error) {
		if (DEV) {
			console.log(error);
		}
		return;
	}
}

export function computeReceiveAmount(stake: boolean, value: number, exchangeRate: number): number {
	if (isNaN(value)) return 0;

	if (exchangeRate) {
		if (stake) {
			return value * exchangeRate;
		} else {
			return value / exchangeRate;
		}
	} else {
		return 0;
	}
}

export async function getWarningError(withdrawal: WithdrawalDetails): Promise<string | undefined> {
	const key = Object.keys(withdrawal.status)[0] as keyof WithdrawalStatus;
	switch (key) {
		case 'WaitingDissolvement':
			const value: { neuron_id: NeuronId } = withdrawal.status[key];
			const createdAt = await fetchNeuronCreationTimestamp(value.neuron_id);
			const currentTime = Date.now() / 1000;
			const oneYearSeconds = ((4 * 365 + 1) * 24 * 60 * 60) / 4;
			const twoWeeksSeconds = oneYearSeconds / 24;
			const sixMonthsSeconds = oneYearSeconds / 2;
			if (currentTime + twoWeeksSeconds > createdAt + sixMonthsSeconds) {
				return 'Withdrawal is too close to disbursing.';
			} else {
				return undefined;
			}
		case 'WaitingToStartDissolving':
			return undefined;
		case 'NotFound':
			return 'Withdrawal not found.';
		case 'Cancelled':
			return 'Withdrawal already cancelled.';
		case 'WaitingToSplitNeuron':
			return 'Waiting for the withdrawal to split.';
	}
}

export const TOAST_LIFETIME_MS = 5000;

export class Toast {
	public id: number;
	public message: string;
	public type: 'success' | 'error' | 'warning';
	public isTemporary: boolean;
	public timeLeft: number;

	constructor({
		message,
		type,
		isTemporary
	}: {
		message: string;
		type: 'success' | 'error' | 'warning';
		isTemporary: boolean;
	}) {
		this.id = Date.now();
		this.message = message;
		this.type = type;
		this.isTemporary = isTemporary;
		this.timeLeft = TOAST_LIFETIME_MS;
	}

	static temporaryError(message: string): Toast {
		return new Toast({
			message,
			type: 'error',
			isTemporary: true
		});
	}

	static temporaryWarning(message: string): Toast {
		return new Toast({
			message,
			type: 'warning',
			isTemporary: true
		});
	}

	static success(message: string): Toast {
		return new Toast({
			message,
			type: 'success',
			isTemporary: false
		});
	}

	static error(message: string): Toast {
		return new Toast({
			message,
			type: 'error',
			isTemporary: false
		});
	}

	static warning(message: string): Toast {
		return new Toast({
			message,
			type: 'warning',
			isTemporary: false
		});
	}
}
