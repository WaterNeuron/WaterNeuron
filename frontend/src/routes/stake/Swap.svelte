<script lang="ts">
	import { Asset, AssetType, computeRewards, displayUsFormat, numberToBigintE8s } from '$lib';
	import SwapInput from './SwapInput.svelte';
	import { Toast } from '$lib/toast';
	import { inputValue, state, user, isLogging, isConverting, toasts } from '$lib/stores';
	import BigNumber from 'bignumber.js';
	import {
		icpTransferApproved,
		nicpTransferApproved,
		handleStakeResult,
		handleRetrieveResult
	} from '$lib/ledger';
	import type { ConversionArg } from '../../declarations/water_neuron/water_neuron.did';
	import type { Account } from '@dfinity/ledger-icp';
	import { onMount, afterUpdate } from 'svelte';
	import { fade } from 'svelte/transition';

	let stake = true;
	let exchangeRate: BigNumber;
	let totalIcpDeposited: BigNumber;
	let minimumWithdraw: BigNumber;

	function computeReceiveAmount(
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

	export async function icpToNicp(amount: BigNumber) {
		if (!($user && !$isConverting) || !$state) return;
		isConverting.set(true);

		if ($user.icpBalance().isGreaterThanOrEqualTo(amount) && amount.isGreaterThan(0)) {
			try {
				let amountE8s = numberToBigintE8s(amount);
				const approval = await icpTransferApproved(
					amountE8s,
					{
						owner: $user.principal,
						subaccount: []
					} as Account,
					$state.icpLedger
				);
				if (!approval.granted) {
					toasts.add(Toast.error(approval.message ?? 'Unknown Error.'));
				} else {
					const conversionResult = await $state.waterNeuron.icp_to_nicp({
						maybe_subaccount: [],
						amount_e8s: amountE8s
					} as ConversionArg);
					let status = handleStakeResult(conversionResult);
					if (status.success) {
						toasts.add(Toast.success(status.message));
					} else {
						toasts.add(Toast.error(status.message));
					}
				}
			} catch (error) {
				toasts.add(Toast.error('Call was rejected.'));
			}
		} else {
			toasts.add(Toast.error('Conversion failed due to ICP balance.'));
		}
		isConverting.set(false);
	}

	export async function nicpToIcp(amount: BigNumber) {
		if (!($user && !$isConverting) || !$state) return;
		isConverting.set(true);

		if ($user.nicpBalance().isGreaterThanOrEqualTo(amount) && amount.isGreaterThan(0)) {
			try {
				let amountE8s = numberToBigintE8s(amount);
				const approval = await nicpTransferApproved(
					amountE8s,
					{
						owner: $user.principal,
						subaccount: []
					} as Account,
					$state.nicpLedger
				);
				if (!approval.granted) {
					toasts.add(Toast.error(approval.message ?? 'Unknown Error.'));
				} else {
					const conversionResult = await $state.waterNeuron.nicp_to_icp({
						maybe_subaccount: [],
						amount_e8s: amountE8s
					} as ConversionArg);
					let status = handleRetrieveResult(conversionResult);
					if (status.success) {
						toasts.add(Toast.success(status.message));
					} else {
						toasts.add(Toast.error(status.message));
					}
				}
			} catch (error) {
				toasts.add(Toast.error('Call was rejected.'));
			}
		} else {
			toasts.add(Toast.error('Conversion failed due to nICP balance.'));
		}
		isConverting.set(false);
	}

	const fetchData = async () => {
		if ($state)
			try {
				exchangeRate = $state.exchangeRate();
				totalIcpDeposited = $state.totalIcpDeposited();
				minimumWithdraw = BigNumber(10).multipliedBy(exchangeRate);
			} catch (error) {
				console.error('Error fetching data:', error);
			}
	};

	afterUpdate(() => {
		if ($state) {
			fetchData();
		}
	});

	onMount(() => {
		const intervalId = setInterval(fetchData, 5000);
		return () => clearInterval(intervalId);
	});
</script>

<div class="main-container" in:fade={{ duration: 500 }}>
	{#key stake}
		<div class="header-container">
			<button
				class="header-btn"
				style:text-align="start"
				on:click={() => {
					stake = true;
					inputValue.set('');
				}}
				class:selected={stake}
				class:not-selected={!stake}>Stake ICP</button
			>
			<button
				class="header-btn"
				style:text-align="end"
				on:click={() => {
					stake = false;
					inputValue.set('');
				}}
				class:selected={!stake}
				class:not-selected={stake}>Unstake nICP</button
			>
		</div>
		<div class="swap-container">
			<SwapInput asset={stake ? Asset.fromText('ICP') : Asset.fromText('nICP')} />
			<div class="paragraphs" in:fade={{ duration: 500 }}>
				{#if stake}
					<p style:color="#fa796e">
						{#if exchangeRate}
							You will receive {displayUsFormat(
								computeReceiveAmount(stake, BigNumber($inputValue), exchangeRate),
								8
							)} nICP
						{:else}
							...
						{/if}
					</p>
					<p>
						{#if exchangeRate}
							1 ICP = {displayUsFormat(exchangeRate)} nICP
						{:else}
							...
						{/if}
					</p>
					<div class="reward">
						<p style:margin-right={'2.5em'}>
							Future WTN Airdrop:
							{#if exchangeRate && totalIcpDeposited}
								{displayUsFormat(
									computeRewards(
										totalIcpDeposited,
										computeReceiveAmount(stake, BigNumber($inputValue), exchangeRate)
									),
									8
								)}
							{:else}
								...
							{/if}
						</p>
						<img src="/tokens/WTN.png" width="30em" height="30em" alt="WTN logo" />
					</div>
				{:else}
					<p style:color="#fa796e">
						{#if exchangeRate}
							You will receive {displayUsFormat(
								computeReceiveAmount(stake, BigNumber($inputValue), exchangeRate),
								8
							)} ICP
						{:else}
							...
						{/if}
					</p>
					<p>
						{#if exchangeRate}
							1 nICP = {displayUsFormat(BigNumber(1).dividedBy(exchangeRate))} ICP
						{:else}
							...
						{/if}
					</p>
					<p>Waiting Time: 6 months</p>
					<p>
						{#if minimumWithdraw}
							Minimum Withdrawal: {minimumWithdraw} nICP
						{:else}
							...
						{/if}
					</p>
				{/if}
			</div>
			{#if !$user}
				<button
					class="swap-btn"
					on:click={() => {
						isLogging.update(() => true);
					}}
				>
					<span>Connect your wallet</span>
				</button>
			{:else}
				<button
					class="swap-btn"
					on:click={() =>
						stake ? icpToNicp(BigNumber($inputValue)) : nicpToIcp(BigNumber($inputValue))}
				>
					{#if $isConverting}
						<div class="spinner"></div>
					{:else if stake}
						<span>Stake</span>
					{:else}
						<span>Unstake</span>
					{/if}
				</button>
			{/if}
		</div>
	{/key}
</div>

<style>
	/* === Base Styles === */
	p {
		color: var(--text-color);
		font-family: var(--font-type2);
		font-weight: bold;
		text-align: end;
		margin: 0;
	}

	img {
		padding: 0.3em;
		position: absolute;
	}

	span {
		color: black;
	}

	/* === Layout === */
	.main-container {
		display: flex;
		place-content: center;
		flex-direction: column;
		box-shadow: rgba(41, 49, 71, 0.1) 0px 8px 16px;
		width: 30em;
		max-width: 97vw;
		height: auto;
	}

	.header-container {
		display: flex;
		justify-content: space-between;
	}

	.swap-container {
		display: flex;
		flex-direction: column;
		padding: 1em;
		border-left: 2px solid var(--border-color);
		border-right: 2px solid var(--border-color);
		border-bottom: 2px solid var(--border-color);
		border-bottom-left-radius: 10px;
		border-bottom-right-radius: 10px;
		background-color: var(--background-color);
		gap: 1em;
	}

	.paragraphs {
		display: flex;
		justify-content: space-around;
		flex-direction: column;
		height: 8em;
	}

	/* === Components === */
	.header-btn {
		font-family: var(--font-type2);
		font-size: 1.2em;
		font-weight: bold;
		color: white;
		border: none;
		color: white;
		padding: 1em;
		width: 100%;
	}

	.reward {
		display: inline-flex;
		align-items: center;
		justify-content: flex-end;
		position: relative;
	}

	.swap-btn {
		background: var(--main-color);
		min-width: 80px;
		max-width: fit-content;
		position: relative;
		border: 2px solid black;
		font-size: 16px;
		font-weight: bold;
		box-shadow: 3px 3px 0 0 black;
		padding: 0 1em 0 1em;
		max-width: none;
		height: 4em;
		cursor: pointer;
		display: flex;
		justify-content: center;
		align-items: center;
	}

	.swap-btn:hover {
		box-shadow: 6px 6px 0 0 black;
	}

	/* === Utilities === */
	.selected {
		border-left: 2px solid var(--border-color);
		border-top: 2px solid var(--border-color);
		border-right: 2px solid var(--border-color);
		background-color: var(--background-color);
	}

	.not-selected {
		border-bottom: 2px solid var(--border-color);
		background-color: #5d6b77;
		color: #c7c7c7;
		cursor: pointer;
	}

	/* === Animation === */

	.spinner {
		width: 2em;
		height: 2em;
		border: 3px solid black;
		border-top-color: transparent;
		border-radius: 50%;
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}
</style>
