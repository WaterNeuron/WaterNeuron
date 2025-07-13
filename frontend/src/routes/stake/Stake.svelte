<script lang="ts">
	import {
		computeRewards,
		displayNumber,
		numberToBigintE8s,
		computeReceiveAmount,
		Toast,
		isMobile,
		assetToTransferFee
	} from '$lib';
	import SwapInput from './SwapInput.svelte';
	import ChangeIcon from '$lib/icons/ChangeIcon.svelte';
	import ErrorIcon from '$lib/icons/ErrorIcon.svelte';
	import {
		inputAmount,
		waterNeuronInfo,
		canisters,
		user,
		toasts,
		isBusy,
		inChart
	} from '$lib/stores';
	import {
		icpTransferApproved,
		handleStakeResult,
		DEFAULT_ERROR_MESSAGE
	} from '$lib/resultHandler';
	import type { ConversionArg } from '$lib/../declarations/water_neuron/water_neuron.did';
	import type { Account } from '../../declarations/icrc_ledger/icrc_ledger.did';
	import { onMount, afterUpdate } from 'svelte';
	import { fade } from 'svelte/transition';
	import Chart from './Chart.svelte';
	import FullScreenIcon from '$lib/icons/FullScreenIcon.svelte';

	let invertExchangeRate = false;
	let exchangeRate: number;
	let totalIcpDeposited: number;
	let isStaking = false;

	async function icpToNicp(amount: number) {
		if (
			!$user ||
			!$canisters?.waterNeuron.authActor ||
			!$canisters?.icpLedger.authActor ||
			isNaN(amount) ||
			amount < 1 ||
			$isBusy
		)
			return;
		isBusy.set(true);

		if ($user.icpBalance() > amount) {
			try {
				let amountE8s = numberToBigintE8s(amount);
				const approval = await icpTransferApproved(
					amountE8s + assetToTransferFee('ICP'),
					{
						owner: $user.principal,
						subaccount: []
					} as Account,
					$canisters.icpLedger
				);
				if (!approval.success) {
					toasts.add(Toast.error(approval.message ?? DEFAULT_ERROR_MESSAGE));
				} else {
					const conversionResult = await $canisters.waterNeuron.authActor.icp_to_nicp({
						maybe_subaccount: [],
						amount_e8s: amountE8s
					} as ConversionArg);
					const status = handleStakeResult(conversionResult);
					if (status.success) {
						toasts.add(Toast.success(status.message));
					} else {
						toasts.add(Toast.error(status.message));
					}
				}
			} catch (error) {
				console.log('[icpToNicp] error:', error);
				toasts.add(Toast.temporaryError('Call was rejected.'));
			}
		} else {
			toasts.add(Toast.temporaryWarning('Sorry, there are not enough funds in this account.'));
		}
		inputAmount.reset();
		isBusy.set(false);
	}

	const fetchData = async () => {
		if ($waterNeuronInfo)
			try {
				exchangeRate = $waterNeuronInfo.exchangeRate();
				totalIcpDeposited = $waterNeuronInfo.totalIcpDeposited();
			} catch (error) {
				console.error('Error fetching data:', error);
			}
	};

	afterUpdate(() => {
		if ($waterNeuronInfo) {
			fetchData();
		}
	});

	onMount(() => {
		const intervalId = setInterval(fetchData, 5000);
		return () => clearInterval(intervalId);
	});
</script>

<div class="swap-container">
	<SwapInput asset={'ICP'} />
	<div class="paragraphs" in:fade={{ duration: 200 }}>
		{#if $inputAmount && isNaN(parseFloat($inputAmount))}
			<span class="error">
				<ErrorIcon /> Cannot read amount
			</span>
		{:else if parseFloat($inputAmount) < 1}
			<span class="error">
				<ErrorIcon /> You should have at least 1 ICP to stake.
			</span>
		{:else if !isNaN(parseFloat($inputAmount)) && parseFloat($inputAmount) > ($user?.icpBalance() ?? 0)}
			<span class="error">
				<ErrorIcon /> You don't have enough funds to complete the transaction.
			</span>
		{/if}
		<p style:color="var(--important-text-color)">
			{#if exchangeRate}
				You will receive {displayNumber(
					computeReceiveAmount(true, parseFloat($inputAmount), exchangeRate),
					8
				)} nICP
			{:else}
				-/-
			{/if}
		</p>
		<p style:display="flex">
			<button class="change-btn" on:click={() => (invertExchangeRate = !invertExchangeRate)}>
				<ChangeIcon />
			</button>
			{#if exchangeRate}
				{invertExchangeRate
					? `1 nICP = ${displayNumber(1 / exchangeRate, 8)} ICP`
					: `1 ICP = ${displayNumber(exchangeRate, 8)} nICP`}
				<button class="chart-btn" on:click={() => inChart.set(true)}>
					<svg width="30" height="20">
						<rect
							width="100%"
							height="100%"
							fill="var(--background-color-transparent)"
							rx="8"
							ry="8"
						/>
						<path
							d={invertExchangeRate ? 'M0,15 L10,15 L30,5' : 'M0,5 L10,5 L30,15'}
							fill="none"
							stroke="var(--main-color)"
							stroke-width="2"
						/>
					</svg>
				</button>
				<button class="full-screen" on:click={() => inChart.set(true)}>
					<FullScreenIcon />
				</button>
			{:else}
				-/-
			{/if}
		</p>

		<a class="reward" href="https://docs.waterneuron.fi/wtn/airdrop" target="_blank">
			<div class="reward">
				<p style:margin-right={'2.5em'}>
					Future WTN Airdrop:
					{#if totalIcpDeposited && !isNaN(parseFloat($inputAmount))}
						{displayNumber(computeRewards(totalIcpDeposited, parseFloat($inputAmount)), 4)}
					{:else}
						-/-
					{/if}
				</p>
				<img src="/tokens/WTN.webp" width="30em" height="30em" alt="WTN logo" class="wtn-logo" />
			</div>
		</a>
	</div>
	<button
		class="swap-btn"
		on:click={async () => {
			isStaking = true;
			await icpToNicp(parseFloat($inputAmount));
			isStaking = false;
		}}
		title="stake-unstake-btn"
		disabled={$isBusy || !$user}
	>
		{#if isStaking}
			<div class="spinner"></div>
		{:else}
			<span>Stake</span>
		{/if}
	</button>
</div>
{#if $inChart}
	<Chart isInverted={invertExchangeRate} />
{/if}

<style>
	/* === Base Styles === */
	p {
		color: var(--text-color);
		font-family: var(--secondary-font);
		text-align: end;
		margin: 0;
		display: flex;
		justify-content: end;
		align-items: center;
		gap: 0.2em;
	}

	button:disabled {
		background-color: var(--main-color-disabled);
		color: var(--main-button-text-color-disabled);
		cursor: default;
	}

	/* === Layout === */
	.swap-container {
		display: flex;
		flex-direction: column;
		padding: 1em;
		border-bottom-left-radius: 10px;
		border-bottom-right-radius: 10px;
		background-color: var(--background-color);
		gap: 1em;
	}

	.paragraphs {
		display: flex;
		flex-direction: column;
		position: relative;
		gap: 1em;
	}

	.error {
		display: flex;
		align-items: center;
		color: var(--text-color);
		gap: 0.2em;
		margin-left: 1em;
		font-size: 16px;
		font-family: var(--secondary-font);
		flex-wrap: wrap;
		font-size: 14px;
	}

	/* === Components === */
	.change-btn {
		border: none;
		display: flex;
		width: fit-content;
		height: fit-content;
		background: transparent;
		padding: 0;
		margin: 0;
		cursor: pointer;
	}

	.full-screen {
		border: none;
		background: none;
		border-radius: 8px;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 0.2em;
		cursor: pointer;
	}

	.chart-btn {
		border: none;
		background: none;
		cursor: pointer;
		display: flex;
		justify-content: center;
	}

	.full-screen:hover {
		background-color: var(--main-color-light);
	}

	.wtn-logo {
		padding: 0.3em;
		position: absolute;
	}

	.reward {
		display: inline-flex;
		align-items: center;
		justify-content: flex-end;
		position: relative;
		text-decoration: underline;
		text-decoration-color: var(--text-color);
	}

	.swap-btn {
		background: var(--main-color);
		min-width: 80px;
		max-width: fit-content;
		position: relative;
		border: var(--main-container-border);
		border-radius: 8px;
		font-size: 16px;
		font-weight: bold;
		padding: 0 1em 0 1em;
		max-width: none;
		height: 3em;
		cursor: pointer;
		display: flex;
		justify-content: center;
		align-items: center;
		color: var(--main-button-text-color);
	}

	.swap-btn:hover {
		background: var(--main-color-hover);
	}

	/* === Animation === */
	.change-btn:hover {
		transform: scale(1.2);
		animation: invert 0.5s ease;
	}

	.spinner {
		width: 2em;
		height: 2em;
		border: 3px solid var(--main-button-text-color);
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

	@keyframes invert {
		from {
			transform: scale(1);
		}
		to {
			transform: scale(1.2);
		}
	}
</style>
