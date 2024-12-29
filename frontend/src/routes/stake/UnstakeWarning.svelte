<script lang="ts">
	import { BigNumber } from 'bignumber.js';
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
	} from '$lib/authentification';
	import { Toast } from '$lib/toast';
	import {
		displayUsFormat,
		numberToBigintE8s,
		bigintE8sToNumber,
		computeReceiveAmount
	} from '$lib';

	const DEFAULT_LEDGER_FEE = 10_000n;

	export let isFastUnstake: boolean;
	export let fastUnstakeAmount: BigNumber;
	export let minimumWithdraw: BigNumber;
	export let exchangeRate: BigNumber;
	let isUnstaking = false;
	let dialog: HTMLDialogElement;

	onMount(() => {
		dialog = document.getElementById('unstakeWarningDialog') as HTMLDialogElement;
		dialog.showModal();
	});

	async function nicpToIcp(amount: BigNumber) {
		if (
			!$user ||
			$isBusy ||
			!$canisters?.nicpLedger.authenticatedActor ||
			!$canisters?.waterNeuron.authenticatedActor ||
			amount.isNaN() ||
			amount.isLessThan(minimumWithdraw)
		)
			return;
		isBusy.set(true);
		if ($user.nicpBalance().isGreaterThanOrEqualTo(amount) && amount.isGreaterThan(0)) {
			try {
				let amountE8s = numberToBigintE8s(amount);
				const approval = await nicpTransferApproved(
					amountE8s,
					{
						owner: $user.principal,
						subaccount: []
					} as Account,
					$canisters.nicpLedger
				);
				if (!approval.success) {
					toasts.add(Toast.error(approval.message ?? 'Unknown Error.'));
				} else {
					const conversionResult = await $canisters.waterNeuron.authenticatedActor.nicp_to_icp({
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
				toasts.add(Toast.error('Call was rejected.'));
			}
		} else {
			toasts.add(Toast.error('Sorry, there are not enough funds in this account.'));
		}
		isBusy.set(false);
	}

	const approveInFastUnstake = async (spender: Account, amountE8s: bigint) => {
		if (!$canisters?.nicpLedger.authenticatedActor) return;

		const approveResult: ApproveResult =
			await $canisters.nicpLedger.authenticatedActor.icrc2_approve({
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
		if (!$canisters?.icpswapPool.authenticatedActor) return;

		const depositResult = await $canisters.icpswapPool.authenticatedActor.depositFrom({
			fee: DEFAULT_LEDGER_FEE,
			token: CANISTER_ID_NICP_LEDGER,
			amount: amountE8s
		} as DepositArgs);

		if ('ok' in depositResult) {
			return depositResult.ok;
		} else {
			toasts.add(Toast.error('Failed to deposit nICP on ICPSwap. Please try again.'));
			throw new Error(`${handleIcpSwapError(depositResult.err)}`);
		}
	};

	const swapInFastUnstake = async (amountIn: string, amountOut: string) => {
		if (!$canisters?.icpswapPool.authenticatedActor) return;

		const swapResult = await $canisters.icpswapPool.authenticatedActor.swap({
			amountIn: amountIn.toString(),
			zeroForOne: true,
			amountOutMinimum: amountOut.toString()
		} as SwapArgs);

		if ('ok' in swapResult) {
			return swapResult.ok;
		} else {
			toasts.add(Toast.error('Failed swap. Please try again.'));
			throw new Error(`${handleIcpSwapError(swapResult.err)}`);
		}
	};

	const withdrawInFastUnstake = async (amountToWithdrawE8s: bigint) => {
		if (!$canisters?.icpswapPool.authenticatedActor) return;

		const withdrawResult = await $canisters.icpswapPool.authenticatedActor.withdraw({
			fee: DEFAULT_LEDGER_FEE,
			token: CANISTER_ID_ICP_LEDGER,
			amount: amountToWithdrawE8s
		} as WithdrawArgs);

		if ('ok' in withdrawResult) {
			const swapAmount = displayUsFormat(bigintE8sToNumber(withdrawResult.ok), 4);
			toasts.add(Toast.success(`Successful swap, ${swapAmount} ICP received.`));
		} else {
			toasts.add(Toast.error('Failed to withdraw funds during swap. Please try again.'));
			throw new Error(`${handleIcpSwapError(withdrawResult.err)}`);
		}
	};

	async function fastUnstake(amount: BigNumber) {
		if (!$canisters || !$user || $isBusy || amount.isNaN() || !fastUnstakeAmount) return;
		isBusy.set(true);
		if ($user.nicpBalance().isGreaterThanOrEqualTo(amount) && amount.isGreaterThan(0)) {
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
				if (numberToBigintE8s(amount) > allowance) {
					await approveInFastUnstake(spender, amountE8s);
					amountE8s -= DEFAULT_LEDGER_FEE;
				}

				// 2. Deposit
				const amountIn = await depositInFastUnstake(amountE8s);
				if (!amountIn) {
					isBusy.set(false);
					return;
				}

				// 3. Swap
				const amountOut = numberToBigintE8s(fastUnstakeAmount.multipliedBy(BigNumber(0.98)));
				const amountToWithdrawE8s = await swapInFastUnstake(
					amountIn.toString(),
					amountOut.toString()
				);
				if (!amountToWithdrawE8s) {
					isBusy.set(false);
					return;
				}

				// 4. Withdraw

				await withdrawInFastUnstake(amountToWithdrawE8s);
			} catch (error) {
				console.log('[fastUnstake] error:', error);
			}
		} else {
			toasts.add(Toast.error('Sorry, there are not enough funds in this account.'));
		}
		isBusy.set(false);
	}
</script>

<dialog
	id="unstakeWarningDialog"
	on:close={() => {
		inUnstakeWarningMenu.set(false);
	}}
	in:fade={{ duration: 500 }}
>
	<div class="main-container">
		<h2>Unstake Confirmation</h2>
		{#if isFastUnstake}
			<p>You are currently swapping on the market.</p>
		{:else}
			<p>You are currently unstaking with the protocol.</p>
		{/if}
		<div class="sum-up-container">
			<p>Convert {displayUsFormat(BigNumber($inputAmount), 8)} nICP</p>
			{#if isFastUnstake}
				<p>
					Receive {fastUnstakeAmount.toNumber() === 0
						? '-/-'
						: displayUsFormat(fastUnstakeAmount, 8)} ICP
				</p>
			{:else}
				<p>
					Receive {displayUsFormat(
						computeReceiveAmount(false, BigNumber($inputAmount), exchangeRate),
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
						await fastUnstake(BigNumber($inputAmount));
						isUnstaking = false;
						dialog.close();
					} else {
						isUnstaking = true;
						await nicpToIcp(BigNumber($inputAmount));
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
		border: 2px solid black;
		font-size: 14px;
		box-shadow: 3px 3px 0 0 black;
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
		transform: scale(0.95);
		transition: all 0.3s;
		box-shadow: 6px 6px 0 0 black;
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
		border: var(--input-border);
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
		border: var(--input-border);
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
