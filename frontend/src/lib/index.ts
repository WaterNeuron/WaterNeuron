import { Principal } from '@dfinity/principal';
import BigNumber from 'bignumber.js';
import type { WithdrawalStatus } from '../declarations/water_neuron/water_neuron.did';
import { AccountIdentifier } from '@dfinity/ledger-icp';
import { decodeIcrcAccount } from '@dfinity/ledger-icrc';
import type { Account } from '../declarations/icp_ledger/icp_ledger.did';
import type {
	NeuronId,
	WithdrawalDetails
} from '$lib/../declarations/water_neuron/water_neuron.did';
import { DEV } from './authentification';

export const E8S = BigNumber(10).pow(BigNumber(8));

export function displayPrincipal(principal: Principal) {
	const a = principal.toString().split('-');
	return a[0] + '...' + a[a.length - 1];
}

export function displayUsFormat(value: BigNumber, decimals = 2): string {
	const factor = new BigNumber(10).pow(decimals);
	const truncatedValue = value
		.multipliedBy(factor)
		.integerValue(BigNumber.ROUND_DOWN)
		.dividedBy(factor);

	const converted = truncatedValue.toFixed(decimals);

	return new Intl.NumberFormat('en-US', {
		minimumFractionDigits: 0,
		maximumFractionDigits: decimals
	})
		.format(Number(converted))
		.replace(/,/g, "'");
}

export function numberWithPrecision(x: BigNumber, decimals: BigNumber): BigNumber {
	const scaleFactor = BigNumber(10).pow(decimals);
	const xScaled = BigNumber(x).multipliedBy(scaleFactor).integerValue(BigNumber.ROUND_FLOOR);
	return BigNumber(xScaled ? xScaled : 0).dividedBy(scaleFactor);
}

export function numberToBigintE8s(x: BigNumber): bigint {
	const xScaled = numberWithPrecision(x, BigNumber(8)).multipliedBy(E8S);
	return BigInt(xScaled.toFixed(0));
}

export function bigintE8sToNumber(x: bigint): BigNumber {
	return BigNumber(Number(x)).dividedBy(E8S);
}

export function assetToIconPath(asset: 'ICP' | 'nICP' | 'WTN'): string {
	switch (asset) {
		case 'ICP':
			return '/tokens/icp.webp';
		case 'nICP':
			return '/tokens/nicp.webp';
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

export function assetToTransferFee(asset: 'ICP' | 'nICP' | 'WTN'): BigNumber {
	switch (asset) {
		case 'ICP':
			return BigNumber(0.0001);
		case 'nICP':
			return BigNumber(0.0001);
		case 'WTN':
			return BigNumber(0.01);
	}
}

export const TIERS: [BigNumber, BigNumber][] = [
	[BigNumber(80_000), BigNumber(8)],
	[BigNumber(160_000), BigNumber(4)],
	[BigNumber(320_000), BigNumber(2)],
	[BigNumber(640_000), BigNumber(1)],
	[BigNumber(1_280_000), BigNumber(0.5)],
	[BigNumber(2_560_000), BigNumber(0.25)],
	[BigNumber(5_120_000), BigNumber(0.125)]
];

export const EXPECTED_INITIAL_BALANCE: BigNumber = BigNumber(4_480_000);

export function computeRewards(alreadyDistributed: BigNumber, converting: BigNumber): BigNumber {
	let totalRewards = BigNumber(0);
	let amountToDistribute = BigNumber(converting);
	let cumulativeAmount = BigNumber(alreadyDistributed);

	for (const [threshold, rate] of TIERS) {
		const nicpThreshold = BigNumber(threshold);
		const allocationRate = BigNumber(rate);
		if (cumulativeAmount.comparedTo(nicpThreshold) === 1) {
			cumulativeAmount = cumulativeAmount.minus(nicpThreshold);
			continue;
		}
		const tierAvailable = nicpThreshold.minus(cumulativeAmount);
		cumulativeAmount = BigNumber(0);
		const amountInThisTier = BigNumber(
			Math.min(amountToDistribute.toNumber(), tierAvailable.toNumber())
		);

		totalRewards = totalRewards.plus(amountInThisTier.multipliedBy(allocationRate));

		amountToDistribute = amountToDistribute.minus(amountInThisTier);

		if (amountToDistribute.comparedTo(BigNumber(0)) === 0) {
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
	  Conversion done at{" "}
	  <a
		target="_blank"
		rel="noreferrer"
		href={https://dashboard.internetcomputer.org/transaction/${status[key]['transfer_block_height']}}
	  >
		Height ${status[key]['transfer_block_height']}
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
	const localTestingTimestamp = 1730124683;
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

export function computeReceiveAmount(
	stake: boolean,
	value: BigNumber,
	exchangeRate: BigNumber
): BigNumber {
	if (value.isNaN()) return BigNumber(0);

	if (exchangeRate) {
		if (stake) {
			return value.multipliedBy(exchangeRate);
		} else {
			return value.dividedBy(exchangeRate);
		}
	} else {
		return BigNumber(0);
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
			if (currentTime - createdAt > sixMonthsSeconds - twoWeeksSeconds) {
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
