<script lang="ts">
	import { onMount } from 'svelte';
	import { inUnstakeWarningMenu, isBusy, inputAmount, user, canisters, toasts } from '$lib/stores';
	import type { ConversionArg } from '$lib/../declarations/water_neuron/water_neuron.did';
	import type {
		ApproveArgs,
		Result_2 as ApproveResult,
		Allowance,
		AllowanceArgs,
		Account
	} from '$lib/../declarations/icrc_ledger/icrc_ledger.did';
	import { fade } from 'svelte/transition';
	import type {
		DepositArgs,
		SwapArgs,
		WithdrawArgs
	} from '$lib/../declarations/icpswap_pool/icpswap_pool.did';
	import { Principal } from '@dfinity/principal';
	import {
		nicpTransferApproved,
		handleUnstakeResult,
		handleApproveResult,
		handleIcpSwapError
	} from '$lib/resultHandler';
	import {
		CANISTER_ID_ICP_LEDGER,
		CANISTER_ID_ICPSWAP_POOL,
		CANISTER_ID_NICP_LEDGER
	} from '$lib/env';
	import {
		displayNumber,
		numberToBigintE8s,
		bigintE8sToNumber,
		computeReceiveAmount,
		Toast,
		assetToTransferFee
	} from '$lib';

	export let isFastUnstake: boolean;
	export let fastUnstakeAmount: number;
	export let minimumWithdraw: number;
	export let exchangeRate: number;
	let isUnstaking = false;
	let dialog: HTMLDialogElement;

	onMount(() => {
		dialog = document.getElementById('unstakeWarningDialog') as HTMLDialogElement;
		dialog.showModal();
	});

	async function nicpToIcp(amount: number) {
		if (
			!$user ||
			$isBusy ||
			!$canisters?.nicpLedger.authActor ||
			!$canisters?.waterNeuron.authActor ||
			isNaN(amount) ||
			amount < minimumWithdraw
		)
			return;
		isBusy.set(true);
		if ($user.nicpBalance() > amount) {
			try {
				let amountE8s = numberToBigintE8s(amount);
				const approval = await nicpTransferApproved(
					amountE8s + assetToTransferFee('nICP'),
					{
						owner: $user.principal,
						subaccount: []
					} as Account,
					$canisters.nicpLedger
				);
				if (!approval.success) {
					toasts.add(Toast.error(approval.message ?? 'Unknown Error.'));
				} else {
					const conversionResult = await $canisters.waterNeuron.authActor.nicp_to_icp({
						maybe_subaccount: [],
						amount_e8s: amountE8s
					} as ConversionArg);
					const status = handleUnstakeResult(conversionResult);
					if (status.success) {
						toasts.add(Toast.success(status.message));
					} else {
						toasts.add(Toast.error(status.message));
					}
				}
			} catch (error) {
				console.log('[nicpToIcp] error:', error);
				toasts.add(Toast.temporaryError('Call was rejected.'));
			}
		} else {
			toasts.add(Toast.temporaryWarning('Sorry, there are not enough funds in this account.'));
		}
		inputAmount.reset();
		isBusy.set(false);
	}

	const approveInFastUnstake = async (spender: Account, amountE8s: bigint) => {
		if (!$canisters?.nicpLedger.authActor) return;

		const approveResult: ApproveResult = await $canisters.nicpLedger.authActor.icrc2_approve({
			spender,
			fee: [],
			memo: [],
			from_subaccount: [],
			created_at_time: [],
			expires_at: [],
			expected_allowance: [],
			amount: amountE8s
		} as ApproveArgs);

		const status = handleApproveResult(approveResult);
		if (!status.success) {
			toasts.add(Toast.error(status.message));
			throw new Error(`${status.message}`);
		}
	};

	const depositInFastUnstake = async (amountE8s: bigint) => {
		if (!$canisters?.icpswap.authActor) return;

		const depositResult = await $canisters.icpswap.authActor.depositFrom({
			fee: assetToTransferFee('nICP'),
			token: CANISTER_ID_NICP_LEDGER,
			amount: amountE8s
		} as DepositArgs);

		if ('ok' in depositResult) {
			return depositResult.ok;
		} else {
			toasts.add(Toast.temporaryError('Failed to deposit nICP on ICPSwap. Please try again.'));
			throw new Error(`${handleIcpSwapError(depositResult.err)}`);
		}
	};

	const swapInFastUnstake = async (amountIn: string, amountOut: string) => {
		if (!$canisters?.icpswap.authActor) return;

		const swapResult = await $canisters.icpswap.authActor.swap({
			amountIn: amountIn.toString(),
			zeroForOne: true,
			amountOutMinimum: amountOut.toString()
		} as SwapArgs);

		if ('ok' in swapResult) {
			return swapResult.ok;
		} else {
			toasts.add(Toast.temporaryError('Failed swap. Please try again.'));
			throw new Error(`${handleIcpSwapError(swapResult.err)}`);
		}
	};

	const withdrawInFastUnstake = async (amountToWithdrawE8s: bigint) => {
		if (!$canisters?.icpswap.authActor) return;

		const withdrawResult = await $canisters.icpswap.authActor.withdraw({
			fee: assetToTransferFee('ICP'),
			token: CANISTER_ID_ICP_LEDGER,
			amount: amountToWithdrawE8s
		} as WithdrawArgs);

		if ('ok' in withdrawResult) {
			const swapAmount = displayNumber(bigintE8sToNumber(withdrawResult.ok), 4);
			toasts.add(Toast.success(`Successful swap, ${swapAmount} ICP received.`));
		} else {
			toasts.add(Toast.temporaryError('Failed to withdraw funds during swap. Please try again.'));
			throw new Error(`${handleIcpSwapError(withdrawResult.err)}`);
		}
	};

	// Fees -> Approve: 0.01 nICP; Deposit: 0.01 nICP; Withdraw: 0.0001 ICP
	async function fastUnstake(amount: number) {
		if (!$canisters || !$user || $isBusy || isNaN(amount) || !fastUnstakeAmount) return;
		isBusy.set(true);
		if ($user.nicpBalance() > amount && amount > 0) {
			try {
				let amountE8s = numberToBigintE8s(amount);
				// 1. Approve
				const spender = {
					owner: Principal.fromText(CANISTER_ID_ICPSWAP_POOL),
					subaccount: []
				} as Account;

				const allowanceResult: Allowance =
					await $canisters.nicpLedger.anonymousActor.icrc2_allowance({
						account: { owner: $user.principal, subaccount: [] } as Account,
						spender
					} as AllowanceArgs);
				const allowance = allowanceResult['allowance'];
				if (numberToBigintE8s(amount) + assetToTransferFee('nICP') > allowance) {
					// Approve an additional 0.01 nICP to cover the deposit fee.
					await approveInFastUnstake(spender, amountE8s + assetToTransferFee('nICP'));
				}

				// 2. Deposit
				const amountIn = await depositInFastUnstake(amountE8s);
				if (!amountIn) {
					inputAmount.reset();
					isBusy.set(false);
					return;
				}

				// 3. Swap
				// Trade with 2 % max-slippage.
				const amountOut = numberToBigintE8s(fastUnstakeAmount * 0.98);
				const amountToWithdrawE8s = await swapInFastUnstake(
					amountIn.toString(),
					amountOut.toString()
				);
				if (!amountToWithdrawE8s) {
					inputAmount.reset();
					isBusy.set(false);
					return;
				}

				// 4. Withdraw

				await withdrawInFastUnstake(amountToWithdrawE8s);
			} catch (error) {
				console.log('[fastUnstake] error:', error);
			}
		} else {
			toasts.add(Toast.temporaryWarning('Sorry, there are not enough funds in this account.'));
		}
		inputAmount.reset();
		isBusy.set(false);
	}
</script>

<dialog
	id="unstakeWarningDialog"
	on:close={() => {
		inUnstakeWarningMenu.set(false);
	}}
	in:fade={{ duration: 200 }}
>
	<div class="main-container">
		<h2>Unstake Confirmation</h2>
		{#if isFastUnstake}
			<p>You are currently swapping on the market.</p>
		{:else}
			<p>You are currently unstaking with the protocol.</p>
		{/if}
		<div class="sum-up-container">
			<p>Convert {displayNumber(Number($inputAmount), 8)} nICP</p>
			{#if isFastUnstake}
				<p>
					Receive {fastUnstakeAmount === 0 ? '-/-' : displayNumber(fastUnstakeAmount, 8)} ICP
				</p>
			{:else}
				<p>
					Receive {displayNumber(
						computeReceiveAmount(false, Number($inputAmount), exchangeRate),
						8
					)} ICP
				</p>
			{/if}
			<p>Effective {isFastUnstake ? 'immediately' : 'in 6 months'}</p>
		</div>
		<div class="toggle-container">
			<button
				id="abort-btn"
				on:click={() => {
					dialog.close();
				}}>Back</button
			>
			<button
				id="confirm-btn"
				title="confirm-unstake-btn"
				on:click={async () => {
					if (isFastUnstake) {
						isUnstaking = true;
						await fastUnstake(Number($inputAmount));
						isUnstaking = false;
						dialog.close();
					} else {
						isUnstaking = true;
						await nicpToIcp(Number($inputAmount));
						isUnstaking = false;
						dialog.close();
					}
				}}
				disabled={$isBusy}
			>
				{#if isUnstaking}
					<div class="spinner"></div>
				{:else}
					Unstake
				{/if}
			</button>
		</div>
	</div>
</dialog>

<style>
	/* === Base styles === */
	::backdrop {
		backdrop-filter: blur(5px);
	}

	dialog {
		display: flex;
		background: transparent;
		justify-content: center;
		align-items: center;
		height: fit-content;
		min-height: 100%;
		min-width: 100dvw;
		padding: 0;
		margin: 0;
		border: none;
	}

	h2 {
		font-family: var(--main-font);
		align-self: center;
		margin: 0.2em;
		margin-bottom: 1em;
	}

	p {
		font-family: var(--secondary-font);
		margin: 0.3em;
	}

	button {
		min-width: 80px;
		border-radius: 8px;
		position: relative;
		border: var(--main-container-border);
		font-size: 14px;
		padding: 0 1em 0 1em;
		max-width: none;
		height: 3em;
		font-weight: bold;
		display: flex;
		justify-content: center;
		align-items: center;
		cursor: pointer;
	}

	button:hover {
		background: var(--main-color-hover);
		transition: all 0.2s;
	}

	/* === Containers === */
	.main-container {
		display: flex;
		flex-direction: column;
		height: fit-content;
		max-width: 35em;
		width: 50vw;
		background-color: var(--background-color);
		color: var(--stake-text-color);
		padding: 1.5em;
		border-radius: 15px;
	}

	.toggle-container {
		display: flex;
		width: 100%;
		justify-content: end;
		gap: 1em;
		margin-top: 1em;
	}

	.sum-up-container {
		display: flex;
		flex-direction: column;
		border-radius: 8 px;
		background: var(--background-color);
		border: var(--main-container-border);
		padding: 0.5em 1em;
		margin: 0.5em 0;
		border-radius: 8px;
	}

	/* === Components === */
	#abort-btn {
		background: var(--main-button-text-color);
		color: var(--main-color);
	}

	#confirm-btn {
		background: var(--main-color);
		color: var(--main-button-text-color);
	}

	/* === Animation === */
	.spinner {
		width: 1em;
		height: 1em;
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
		.main-container {
			width: 85%;
		}
	}
</style>
