<script lang="ts">
	import {
		displayNumber,
		numberToBigintE8s,
		bigintE8sToNumber,
		computeReceiveAmount,
		Toast,
		assetToTransferFee
	} from '$lib';
	import ChangeIcon from '$lib/icons/ChangeIcon.svelte';
	import InfoIcon from '$lib/icons/InfoIcon.svelte';
	import ErrorIcon from '$lib/icons/ErrorIcon.svelte';
	import SwapInput from './SwapInput.svelte';
	import {
		inputAmount,
		waterNeuronInfo,
		canisters,
		user,
		toasts,
		isBusy,
		inUnstakeWarningMenu
	} from '$lib/stores';
	import { DEFAULT_ERROR_MESSAGE } from '$lib/resultHandler';
	import { CANISTER_ID_ICP_LEDGER, CANISTER_ID_NICP_LEDGER } from '$lib/env';
	import type {
		SwapArgs,
		WithdrawArgs,
		Result as IcpSwapResult,
		Result_7 as IcpSwapUnusedBalanceResult
	} from '$lib/../declarations/icpswap_pool/icpswap_pool.did';
	import { onMount, afterUpdate } from 'svelte';
	import UnstakeWarning from './UnstakeWarning.svelte';

	let invertExchangeRate = false;
	let isFastUnstake = false;
	let isUnstaking = false;
	let exchangeRate: number;
	let minimumWithdraw: number;
	let fastUnstakeAmount = 0;
	let timeoutId: NodeJS.Timeout | null = null;
	let showFailedHelp = false;
	let showImmediateHelp = false;
	let showDelayedHelp = false;

	const withdrawIcpswapTokens = async () => {
		if (!$canisters?.icpswap.authActor || !$user) return;
		isBusy.set(true);
		try {
			const result = await $canisters.icpswap.anonymousActor.getUserUnusedBalance($user.principal);
			const key = Object.keys(result)[0] as keyof IcpSwapUnusedBalanceResult;

			switch (key) {
				case 'err':
					toasts.add(Toast.temporaryError('Failed to fetch balances on ICPswap. Please retry.'));
					break;
				case 'ok':
					const nicpBalanceE8s = result[key]['balance0'];
					const icpBalanceE8s = result[key]['balance1'];

					if (
						nicpBalanceE8s + icpBalanceE8s <
						assetToTransferFee('ICP') + assetToTransferFee('nICP')
					) {
						toasts.add(Toast.temporaryError('No funds to withdraw detected.'));
					}

					if (nicpBalanceE8s > assetToTransferFee('nICP')) {
						const withdrawNicpResult = await $canisters.icpswap.authActor.withdraw({
							fee: assetToTransferFee('nICP'),
							token: CANISTER_ID_NICP_LEDGER,
							amount: nicpBalanceE8s
						} as WithdrawArgs);

						const key = Object.keys(withdrawNicpResult)[0] as keyof IcpSwapResult;
						switch (key) {
							case 'ok':
								const withdrawNicpAmount = displayNumber(
									bigintE8sToNumber(withdrawNicpResult[key]),
									4
								);
								toasts.add(
									Toast.success(`Successful withdraw of ${withdrawNicpAmount} nICP on ICPswap.`)
								);
								break;
							case 'err':
								toasts.add(
									Toast.temporaryError('Failed to withdraw nICP on ICPSwap. Please try again.')
								);
								break;
						}
					}

					if (icpBalanceE8s > assetToTransferFee('ICP')) {
						const withdrawIcpResult = await $canisters.icpswap.authActor.withdraw({
							fee: assetToTransferFee('ICP'),
							token: CANISTER_ID_ICP_LEDGER,
							amount: icpBalanceE8s
						} as WithdrawArgs);

						const key = Object.keys(withdrawIcpResult)[0] as keyof IcpSwapResult;
						switch (key) {
							case 'ok':
								const withdrawIcpAmount = displayNumber(
									bigintE8sToNumber(withdrawIcpResult[key]),
									4
								);
								toasts.add(
									Toast.success(`Successful withdraw of ${withdrawIcpAmount} ICP on ICPswap.`)
								);
								break;
							case 'err':
								toasts.add(
									Toast.temporaryError('Failed to withdraw ICP on ICPSwap. Please try again.')
								);
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
			const amount = Number($inputAmount);
			if (isNaN(amount)) return 0;

			const amountIn = numberToBigintE8s(amount);
			const amountOut = 0;

			const result = await $canisters.icpswap.anonymousActor.quote({
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
					fastUnstakeAmount = 0;
					break;
			}
		} catch (error) {
			console.log(error);
			fastUnstakeAmount = 0;
		}
	};

	function unstakeAvailable(): boolean {
		const amount = parseFloat($inputAmount);
		const minimumAmount = isFastUnstake ? 0 : minimumWithdraw;
		return !isNaN(amount) && amount >= minimumAmount;
	}

	const fetchData = async () => {
		if ($waterNeuronInfo)
			try {
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
	$: if ($waterNeuronInfo) {
		exchangeRate = $waterNeuronInfo.exchangeRate();
		minimumWithdraw = 10 * exchangeRate;
	}
</script>

{#if $inUnstakeWarningMenu}
	<UnstakeWarning {isFastUnstake} {minimumWithdraw} {fastUnstakeAmount} {exchangeRate} />
{/if}

<div class="swap-container">
	<SwapInput asset={'nICP'} />
	{#if $inputAmount && isNaN(parseFloat($inputAmount))}
		<span class="error">
			<ErrorIcon /> Cannot read amount
		</span>
	{:else if !isFastUnstake && parseFloat($inputAmount) < minimumWithdraw}
		<span class="error">
			<ErrorIcon /> Minimum: {displayNumber(minimumWithdraw, 4)} nICP
		</span>
	{:else if parseFloat($inputAmount) > ($user?.nicpBalance() ?? 0)}
		<span class="error">
			<ErrorIcon /> You don't have enough funds to complete the transaction.
		</span>
	{/if}
	<p style:padding-right="0.4em">
		<button class="change-btn" on:click={() => (invertExchangeRate = !invertExchangeRate)}>
			<ChangeIcon />
		</button>
		{#if exchangeRate !== undefined}
			{#if !invertExchangeRate}
				1 nICP = {displayNumber(1 / exchangeRate, 8)} ICP
			{:else}
				1 ICP = {displayNumber(exchangeRate, 8)} nICP
			{/if}
		{:else}
			-/-
		{/if}
	</p>
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
				{#if fastUnstakeAmount >= 0.0002}
					Receive {displayNumber(fastUnstakeAmount - 0.0002, 8)} ICP
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
					Receive {displayNumber(
						computeReceiveAmount(false, parseFloat($inputAmount), exchangeRate),
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
		border-bottom-left-radius: 10px;
		border-bottom-right-radius: 10px;
		background-color: var(--background-color);
		gap: 1em;
	}

	.unstake-selection-container {
		display: flex;
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
		border: var(--main-container-border);
		border-radius: 8px;
		cursor: pointer;
		display: flex;
		justify-content: center;
		align-items: center;
		color: var(--main-button-text-color);
	}

	.main-btn:hover {
		background: var(--main-color-hover);
		transition: all 0.2s;
	}

	.swap-btn {
		min-width: 80px;
		width: 100%;
		padding: 0 1em 0 1em;
		font-weight: bold;
		font-size: 16px;
		height: 3em;
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
		position: absolute;
		background: var(--background-color);
		color: var(--text-color);
		text-align: left;
		padding: 1em;
		border-radius: 8px;
		width: 200px;
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
		border: var(--select-unstake-speed);
		color: var(--title-color);
	}

	.not-selected {
		border: 1px solid transparent;
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
