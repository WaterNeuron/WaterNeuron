import { Principal } from '@dfinity/principal';
import BigNumber from 'bignumber.js';
import type { NeuronId, WithdrawalStatus } from '../declarations/water_neuron/water_neuron.did';
import { DEV } from './authentification';
import { inputValue } from './stores';

export const E8S = BigNumber(10).pow(BigNumber(8));

export function displayPrincipal(principal: Principal) {
	const a = principal.toString().split('-');
	return a[0] + '...' + a[a.length - 1];
}

export function displayUsFormat(value: BigNumber, decimals = 2): string {
	const formatter = new Intl.NumberFormat('en-US', {
		minimumFractionDigits: 0,
		maximumFractionDigits: decimals
	});

	return formatter.format(value.toNumber()).replace(/,/g, "'");
}

export function numberWithPrecision(x: BigNumber, decimals: BigNumber): BigNumber {
	const scaleFactor = BigNumber(10).pow(decimals);
	const xScaled = BigNumber(x).multipliedBy(scaleFactor).integerValue(BigNumber.ROUND_FLOOR);
	return BigNumber(xScaled ? xScaled : 0).dividedBy(scaleFactor);
}

export function numberToBigintE8s(x: BigNumber): bigint {
	const xScaled = numberWithPrecision(x, BigNumber(8)).multipliedBy(E8S);
	return BigInt(xScaled.toNumber());
}

export function bigintE8sToNumber(x: bigint): BigNumber {
	return BigNumber(Number(x)).dividedBy(E8S);
}

export enum AssetType {
	ICP,
	nICP,
	WTN
}

export class Asset {
	public type: AssetType;

	constructor(asset: AssetType) {
		this.type = asset;
	}

	static fromText(symbol: 'WTN' | 'nICP' | 'ICP'): Asset {
		switch (symbol) {
			case 'WTN':
				return new Asset(AssetType.WTN);
			case 'nICP':
				return new Asset(AssetType.nICP);
			case 'ICP':
				return new Asset(AssetType.ICP);
		}
	}

	intoStr(): string {
		switch (this.type) {
			case AssetType.ICP:
				return 'ICP';
			case AssetType.nICP:
				return 'nICP';
			case AssetType.WTN:
				return 'WTN';
			default:
				throw new Error('Unknown asset');
		}
	}

	getIconPath(): string {
		switch (this.type) {
			case AssetType.ICP:
				return '/tokens/icp.webp';
			case AssetType.nICP:
				return '/tokens/nicp.png';
			case AssetType.WTN:
				return '/tokens/WTN.png';
		}
	}

	getDashboardUrl(): string {
		switch (this.type) {
			case AssetType.ICP:
				return 'https://dashboard.internetcomputer.org/transaction/';
			case AssetType.nICP:
				return '/wallet';
			case AssetType.WTN:
				return 'https://dashboard.internetcomputer.org/sns/jmod6-4iaaa-aaaaq-aadkq-cai/transaction/';
		}
	}

	getTransferFee(): BigNumber {
		switch (this.type) {
			case AssetType.ICP:
				return BigNumber(0.0001);
			case AssetType.nICP:
				return BigNumber(0.0001);
			case AssetType.WTN:
				return BigNumber(0.01);
		}
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
			return 'Withdrawal status not found.';
		case 'WaitingToSplitNeuron':
			return 'Waiting to Split Neuron';
		case 'WaitingDissolvement':
			return 'Waiting dissolvement';

		// if (status[key]['neuron_id']['id']) {
		// 	return displayStatus(status[key]['neuron_id']);
		// } else {
		// 	return 'Waiting dissolvement';
		// }
		case 'WaitingToStartDissolving':
			return `Waiting to Start Dissolving (Neuron ID: ${status[key]['neuron_id']['id']})`;
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

	if (isMobile && daysLeft > 0) {
		return `${daysLeft} days left`;
	} else if (isMobile && hoursLeft > 0) {
		return `${hoursLeft} hours left`;
	} else if (daysLeft > 0 && hoursLeft > 0) {
		return `${daysLeft} days and ${hoursLeft} hours left`;
	} else if (daysLeft > 0) {
		return `${daysLeft} days left`;
	} else if (hoursLeft > 0) {
		return `${hoursLeft} hours left`;
	}
	return `Less than an hour left`;
}

export function handleInput(event: Event): void {
	const target = event.target as HTMLInputElement;
	const value = target.value;
	const regex = /^[0-9]*([\.][0-9]*)?$/;

	if (regex.test(value)) {
		inputValue.set(value);
	} else {
		inputValue.set(value.substring(0, value.length - 1));
		target.value = value.substring(0, value.length - 1);
	}
}
