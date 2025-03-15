<script lang="ts">
	import {
		displayUsFormat,
		numberToBigintE8s,
		bigintE8sToNumber,
		computeReceiveAmount
	} from '$lib';
	import ChangeIcon from '$lib/icons/ChangeIcon.svelte';
	import InfoIcon from '$lib/icons/InfoIcon.svelte';
	import ErrorIcon from '$lib/icons/ErrorIcon.svelte';
	import SwapInput from './SwapInput.svelte';
	import { Toast } from '$lib/toast';
	import {
		inputAmount,
		waterNeuronInfo,
		canisters,
		user,
		toasts,
		isBusy,
		inUnstakeWarningMenu
	} from '$lib/stores';
	import BigNumber from 'bignumber.js';
	import { DEFAULT_ERROR_MESSAGE } from '$lib/resultHandler';
	import { CANISTER_ID_ICP_LEDGER, CANISTER_ID_NICP_LEDGER } from '$lib/authentification';
	import type {
		SwapArgs,
		WithdrawArgs,
		Result as IcpSwapResult,
		Result_7 as IcpSwapUnusedBalanceResult
	} from '$lib/../declarations/icpswap_pool/icpswap_pool.did';
	import { onMount, afterUpdate } from 'svelte';
	import { fade } from 'svelte/transition';
	import UnstakeWarning from './UnstakeWarning.svelte';

	let invertExchangeRate = false;
	let isFastUnstake = false;
	let isUnstaking = false;
	let exchangeRate: BigNumber;
	let minimumWithdraw: BigNumber;
	let fastUnstakeAmount = BigNumber(0);
	let timeoutId: NodeJS.Timeout | null = null;
	let showFailedHelp = false;
	let showImmediateHelp = false;
	let showDelayedHelp = false;
	const DEFAULT_LEDGER_FEE = 10_000n;

	const withdrawIcpswapTokens = async () => {
		if (!$canisters?.icpswapPool.authenticatedActor || !$user) return;
		isBusy.set(true);
		try {
			const result = await $canisters.icpswapPool.anonymousActor.getUserUnusedBalance(
				$user.principal
			);
			const key = Object.keys(result)[0] as keyof IcpSwapUnusedBalanceResult;

			switch (key) {
				case 'err':
					toasts.add(Toast.error('Failed to fetch balances on ICPswap. Please retry.'));
					break;
				case 'ok':
					const nicpBalanceE8s = result[key]['balance0'];
					const icpBalanceE8s = result[key]['balance1'];

					if (nicpBalanceE8s + icpBalanceE8s < 2n * DEFAULT_LEDGER_FEE) {
						toasts.add(Toast.error('No funds to withdraw detected.'));
					}

					if (nicpBalanceE8s > DEFAULT_LEDGER_FEE) {
						const withdrawNicpResult = await $canisters.icpswapPool.authenticatedActor.withdraw({
							fee: DEFAULT_LEDGER_FEE,
							token: CANISTER_ID_NICP_LEDGER,
							amount: nicpBalanceE8s
						} as WithdrawArgs);

						const key = Object.keys(withdrawNicpResult)[0] as keyof IcpSwapResult;
						switch (key) {
							case 'ok':
								const withdrawNicpAmount = displayUsFormat(
									bigintE8sToNumber(withdrawNicpResult[key]),
									4
								);
								toasts.add(
									Toast.success(`Successful withdraw of ${withdrawNicpAmount} nICP on ICPswap.`)
								);
								break;
							case 'err':
								toasts.add(Toast.error('Failed to withdraw nICP on ICPSwap. Please try again.'));
								break;
						}
					}

					if (icpBalanceE8s > DEFAULT_LEDGER_FEE) {
						const withdrawIcpResult = await $canisters.icpswapPool.authenticatedActor.withdraw({
							fee: DEFAULT_LEDGER_FEE,
							token: CANISTER_ID_ICP_LEDGER,
							amount: icpBalanceE8s
						} as WithdrawArgs);

						const key = Object.keys(withdrawIcpResult)[0] as keyof IcpSwapResult;
						switch (key) {
							case 'ok':
								const withdrawIcpAmount = displayUsFormat(
									bigintE8sToNumber(withdrawIcpResult[key]),
									4
								);
								toasts.add(
									Toast.success(`Successful withdraw of ${withdrawIcpAmount} ICP on ICPswap.`)
								);
								break;
							case 'err':
								toasts.add(Toast.error('Failed to withdraw ICP on ICPSwap. Please try again.'));
								break;
						}
					}
			}
		} catch (error) {
			console.log('[withdrawIcpswapTokens] error:', error);
			toasts.add(Toast.success(DEFAULT_ERROR_MESSAGE));
		}
		isBusy.set(false);
	};

	const computeReceiveAmountFastUnstake = async () => {
		if (!$canisters) return;

		try {
			const amount = BigNumber($inputAmount);
			if (amount.isNaN()) return BigNumber(0);

			const amountIn = numberToBigintE8s(amount);
			const amountOut = amountIn - numberToBigintE8s(amount.multipliedBy(BigNumber(0.02)));

			const result = await $canisters.icpswapPool.anonymousActor.quote({
				amountIn: amountIn.toString(),
				zeroForOne: true,
				amountOutMinimum: amountOut.toString()
			} as SwapArgs);

			const key = Object.keys(result)[0] as keyof IcpSwapResult;
			switch (key) {
				case 'ok':
					fastUnstakeAmount = bigintE8sToNumber((result as { ok: bigint }).ok);
					break;
				case 'err':
					fastUnstakeAmount = BigNumber(0);
					break;
			}
		} catch (error) {
			console.log(error);
			fastUnstakeAmount = BigNumber(0);
		}
	};

	function unstakeAvailable(): boolean {
		return (
			$inputAmount !== '' &&
			((isFastUnstake && fastUnstakeAmount.toNumber() > 0) ||
				(!isFastUnstake &&
					minimumWithdraw &&
					parseFloat($inputAmount) >= minimumWithdraw.toNumber()))
		);
	}

	const fetchData = async () => {
		if ($waterNeuronInfo)
			try {
				exchangeRate = $waterNeuronInfo.exchangeRate();
				minimumWithdraw = BigNumber(10).multipliedBy(exchangeRate);
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
		const intervalIdFetchData = setInterval(fetchData, 5000);

		return () => {
			clearInterval(intervalIdFetchData);
			if (timeoutId) {
				clearTimeout(timeoutId);
			}
		};
	});

	const triggerTimeout = async () => {
		if (timeoutId) {
			clearTimeout(timeoutId);
		}
		timeoutId = setTimeout(async () => {
			await computeReceiveAmountFastUnstake();
		}, 400);
	};

	$: $inputAmount, triggerTimeout();
</script>

{#if $inUnstakeWarningMenu}
	<UnstakeWarning {isFastUnstake} {minimumWithdraw} {fastUnstakeAmount} {exchangeRate} />
{/if}

<div class="swap-container">
	<SwapInput asset={'nICP'} />
	<div class="paragraphs-container" in:fade={{ duration: 500 }}>
		<span class="error">
			{#if $inputAmount && isNaN(parseFloat($inputAmount))}
				<ErrorIcon /> Cannot read amount
			{:else if !isFastUnstake && $inputAmount && minimumWithdraw && parseFloat($inputAmount) < minimumWithdraw.toNumber()}
				<ErrorIcon /> Minimum: {displayUsFormat(minimumWithdraw, 4)} nICP
			{/if}
		</span>
		<p style:padding-right="0.4em">
			<button class="change-btn" on:click={() => (invertExchangeRate = !invertExchangeRate)}>
				<ChangeIcon />
			</button>
			{#if exchangeRate}
				{#if !invertExchangeRate}
					1 nICP = {displayUsFormat(BigNumber(1).dividedBy(exchangeRate), 8)} ICP
				{:else}
					1 ICP = {displayUsFormat(exchangeRate, 8)} nICP
				{/if}
			{:else}
				-/-
			{/if}
		</p>
	</div>
	<div class="unstake-selection-container">
		<button
			class="unstake-container"
			class:selected={isFastUnstake}
			class:not-selected={!isFastUnstake}
			on:click={() => (isFastUnstake = true)}
		>
			<div class="delay-header">
				<h2>Immediately</h2>
				<button
					class="help-btn"
					on:mouseover={() => (showImmediateHelp = true)}
					on:focus={() => (showImmediateHelp = true)}
					on:mouseleave={() => (showImmediateHelp = false)}
					on:click={withdrawIcpswapTokens}
				>
					<InfoIcon />
					<p style:display={showImmediateHelp ? 'flex' : 'none'} class="help-content">
						Immediate unstake via ICPSwap, traded at the current price with a 2% max-slippage.
					</p>
				</button>
			</div>
			<p>
				{#if fastUnstakeAmount.isGreaterThanOrEqualTo(0.0002)}
					Receive {displayUsFormat(fastUnstakeAmount.minus(BigNumber(0.0002)), 8)} ICP
				{:else}
					Receive -/- ICP
				{/if}
			</p>
			<button
				class="help-btn"
				on:mouseover={() => (showFailedHelp = true)}
				on:focus={() => (showFailedHelp = true)}
				on:mouseleave={() => (showFailedHelp = false)}
				on:click={withdrawIcpswapTokens}
			>
				Failed swap?
				<p
					style:display={showFailedHelp ? 'flex' : 'none'}
					class="help-content left transform-left"
				>
					If a swap is unsuccessful, click here to retrieve the deposited nICP to your wallet.
				</p>
			</button>
		</button>
		<button
			title="delayed-btn"
			class="unstake-container"
			class:not-selected={isFastUnstake}
			class:selected={!isFastUnstake}
			on:click={() => (isFastUnstake = false)}
		>
			<div class="delay-header">
				<h2>Delayed</h2>
				<button
					class="help-btn"
					on:mouseover={() => (showDelayedHelp = true)}
					on:focus={() => (showDelayedHelp = true)}
					on:mouseleave={() => (showDelayedHelp = false)}
					on:click={withdrawIcpswapTokens}
				>
					<InfoIcon />
					<p style:display={showDelayedHelp ? 'flex' : 'none'} class="help-content transform-right">
						The ICP will be sent to your wallet as soon as the 6 months dissolve delay is elapsed,
						it is the minimum amount of time to get rewards on ICP.
					</p>
				</button>
			</div>
			<p>
				{#if exchangeRate}
					Receive {displayUsFormat(
						computeReceiveAmount(false, BigNumber($inputAmount), exchangeRate),
						8
					)} ICP
				{:else}
					-/-
				{/if}
			</p>
			<h2 class="waiting-time">Waiting time: 6 months</h2>
		</button>
	</div>

	<button
		class="main-btn swap-btn"
		on:click={() => {
			if (unstakeAvailable()) {
				inUnstakeWarningMenu.set(true);
			}
		}}
		title="stake-unstake-btn"
		disabled={$isBusy || !$user}
	>
		{#if isUnstaking}
			<div class="spinner"></div>
		{:else}
			Unstake
		{/if}
	</button>
</div>

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

	h2 {
		font-family: var(--secondary-font);
		font-size: 16px;
		margin: 0;
		color: var(--title-color);
		display: flex;
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
		border-left: var(--input-border);
		border-right: var(--input-border);
		border-bottom: var(--input-border);
		border-bottom-left-radius: 10px;
		border-bottom-right-radius: 10px;
		background-color: var(--background-color);
		gap: 1em;
	}

	.paragraphs-container {
		display: flex;
		justify-content: space-between;
		width: 100%;
	}

	.unstake-selection-container {
		display: flex;
		border: var(--input-border);
		border-radius: 8px;
		padding: 1em;
	}

	.unstake-container {
		display: flex;
		width: 50%;
		flex-direction: column;
		background: transparent;
		color: white;
		border: none;
		border-radius: 6px;
		padding: 1em;
		gap: 1em;
	}

	.unstake-container > p {
		align-self: start;
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

	.main-btn {
		background: var(--main-color);
		position: relative;
		border: 2px solid black;
		border-radius: 8px;
		box-shadow: 3px 3px 0 0 black;
		cursor: pointer;
		display: flex;
		justify-content: center;
		align-items: center;
		color: var(--main-button-text-color);
	}

	.main-btn:hover {
		box-shadow: 6px 6px 0 0 black;
	}

	.swap-btn {
		min-width: 80px;
		width: 100%;
		padding: 0 1em 0 1em;
		font-weight: bold;
		font-size: 16px;
		height: 4em;
	}

	.error {
		display: flex;
		align-items: center;
		color: var(--title-color);
		gap: 0.2em;
		margin-left: 1em;
		font-size: 16px;
		font-family: var(--secondary-font);
		flex-wrap: wrap;
		max-width: 45%;
		font-size: 14px;
	}

	.help-btn {
		display: flex;
		position: relative;
		background: none;
		border: none;
		text-decoration: underline;
		font-size: 12px;
		padding: 0;
		color: var(--text-color);
		cursor: pointer;
	}

	.help-content {
		background: var(--background-color);
		color: var(--text-color);
		text-align: left;
		padding: 1em;
		border-radius: 8px;
		width: 200px;
		position: absolute;
		bottom: 2em;
		left: 50%;
		transform: translate(-50%, 0);
		z-index: 1;
		border: var(--input-border);
		box-shadow: 2px 2px 0 0 #8e8b8b;
	}

	.delay-header {
		display: flex;
		width: 100%;
		justify-content: space-between;
		align-items: center;
	}

	.waiting-time {
		font-size: 13px;
		text-align: start;
		font-weight: lighter;
	}

	/* === Utilities === */
	.selected {
		background-color: var(--unstake-selection-color);
		color: var(--title-color);
	}

	.not-selected {
		background-color: var(--background-color);
		cursor: pointer;
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

	@media (max-width: 767px) {
		.swap-container {
			padding: 1em 0.4em;
		}

		.transform-left {
			transform: translate(0%, 0);
		}

		.transform-right {
			transform: translate(-100%, 0);
		}

		.left {
			left: 0;
		}
	}
</style>
