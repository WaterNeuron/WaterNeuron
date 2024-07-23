import { describe, it, expect } from 'vitest';
import {
	numberWithPrecision,
	numberToBigintE8s,
	bigintE8sToNumber,
	TIERS,
	EXPECTED_INITIAL_BALANCE,
	computeRewards
} from '$lib';
import BigNumber from 'bignumber.js';

describe('numberWithPrecision test', () => {
	it('Round a value with a given number of decimals', () => {
		expect(numberWithPrecision(BigNumber(12.436), BigNumber(2)).eq(BigNumber(12.43))).toBeTruthy();
		expect(
			numberWithPrecision(BigNumber(1312.436), BigNumber(8)).eq(BigNumber(1312.436))
		).toBeTruthy();
	});
});

describe('numberToBigintE8s test', () => {
	it('Scale a number with an e8s format and make it a bigint.', () => {
		expect(numberToBigintE8s(BigNumber(1312.436))).toBe(BigInt(131243600000));
	});
});

describe('bigintE8sToNumber test', () => {
	it('Unscale a number with an e8s format and make it a number.', () => {
		expect(bigintE8sToNumber(131243600000n).eq(BigNumber(1312.436))).toBeTruthy();
	});
});

describe('computeRewards', () => {
	it('should return expected rewards', () => {
		const totalThresholds = TIERS.reduce((acc, [threshold, _]) => acc + threshold.toNumber(), 0);
		expect(computeRewards(BigNumber(20_000), BigNumber(100)).eq(BigNumber(800))).toBeTruthy();
		expect(computeRewards(BigNumber(0), BigNumber(10)).eq(BigNumber(80))).toBeTruthy();
		expect(computeRewards(BigNumber(totalThresholds), BigNumber(10)).isZero()).toBeTruthy();
		expect(totalThresholds).toBe(10_160_000);
	});

	it('tiers should add up to expected', () => {
		const total = TIERS.reduce(
			(acc, [threshold, amount]) => acc + threshold.toNumber() * amount.toNumber(),
			0
		);
		expect(BigNumber(total).eq(EXPECTED_INITIAL_BALANCE)).toBeTruthy();
	});
});
