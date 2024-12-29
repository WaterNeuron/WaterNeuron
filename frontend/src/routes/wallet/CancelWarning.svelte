<script lang="ts">
	import { BigNumber } from 'bignumber.js';
	import { onMount } from 'svelte';
	import {
		inCancelWarningMenu,
		waterNeuronInfo,
		canisters,
		toasts,
		selectedWithdrawal
	} from '$lib/stores';
	import ChangeIcon from '$lib/icons/ChangeIcon.svelte';

	import { Toast } from '$lib/toast';
	import {
		displayUsFormat,
		bigintE8sToNumber,
		displayNeuronId,
		isMobile,
		getWarningError
	} from '$lib';
	import { handleCancelWithdrawalResult } from '$lib/resultHandler';

	let dialog: HTMLDialogElement;

	let exchangeRate: BigNumber;
	let warningError: string | undefined;
	let isConfirmBusy = false;
	let invertExchangeRate = false;

	const nicpAfterCancel = (icpDue: BigNumber) => {
		const transactionFee = BigNumber(0.0001);
		const mergedIcp = icpDue.minus(transactionFee.multipliedBy(2));
		const nicpWithoutFee = mergedIcp.multipliedBy(exchangeRate);
		return nicpWithoutFee.minus(nicpWithoutFee.dividedBy(200));
	};

	const setWarning = async () => {
		warningError = $selectedWithdrawal ? await getWarningError($selectedWithdrawal) : undefined;
	};

	const handleCancellation = async () => {
		if (!$canisters?.waterNeuron.authenticatedActor || !$selectedWithdrawal?.request.neuron_id[0])
			return;

		isConfirmBusy = true;
		try {
			const result = await $canisters.waterNeuron.authenticatedActor.cancel_withdrawal(
				$selectedWithdrawal.request.neuron_id[0]
			);
			const status = handleCancelWithdrawalResult(result);

			if (status.success) {
				toasts.add(Toast.success(status.message));
			} else {
				toasts.add(Toast.error(status.message));
			}
		} catch (error) {
			console.error(error);
			toasts.add(Toast.error('Call was rejected.'));
		}
		isConfirmBusy = false;
		dialog.close();
		inCancelWarningMenu.set(false);
	};

	onMount(() => {
		if ($waterNeuronInfo) {
			exchangeRate = $waterNeuronInfo.exchangeRate();
		}
		setWarning();

		dialog = document.getElementById('cancelWarningDialog') as HTMLDialogElement;
		dialog.showModal();
	});
</script>

<dialog
	id="cancelWarningDialog"
	on:close={() => {
		selectedWithdrawal.set(undefined);
		inCancelWarningMenu.set(false);
	}}
>
	{#if warningError}
		<div class="warning-container">
			<h2>Oups...</h2>
			<div class="review-container">
				<p>{warningError}</p>
			</div>
			<div class="toggle-container">
				<button
					id="abort-btn"
					title="test-cancel-abort"
					on:click={() => {
						inCancelWarningMenu.set(false);
					}}>Back</button
				>
			</div>
		</div>
	{:else if $selectedWithdrawal}
		<div class="warning-container">
			<h2>Cancel Withdrawal {$selectedWithdrawal.request.withdrawal_id}</h2>
			<p class="main-information">Neuron info:</p>
			<div class="review-container">
				<p>
					Neuron id: <a
						target="_blank"
						rel="noreferrer"
						href={'https://dashboard.internetcomputer.org/neuron/' +
							displayNeuronId($selectedWithdrawal.request.neuron_id, false)}
						>{displayNeuronId($selectedWithdrawal.request.neuron_id, isMobile)}</a
					>
				</p>
				<p>
					Stake: {displayUsFormat(bigintE8sToNumber($selectedWithdrawal.request.icp_due), 8)} ICP
				</p>
			</div>
			<p class="main-information" style:margin-bottom="1em">
				You will receive {exchangeRate
					? displayUsFormat(
							nicpAfterCancel(bigintE8sToNumber($selectedWithdrawal.request.icp_due)),
							8
						)
					: '-/-'} nICP
			</p>
			<p class="secondary-information" id="exchange-rate">
				<button class="change-btn" on:click={() => (invertExchangeRate = !invertExchangeRate)}>
					<ChangeIcon />
				</button>
				{#if exchangeRate}
					{#if invertExchangeRate}
						1 nICP = {displayUsFormat(BigNumber(1).dividedBy(exchangeRate), 8)} ICP
					{:else}
						1 ICP = {displayUsFormat(exchangeRate, 8)} nICP
					{/if}
				{/if}
			</p>
			<p class="secondary-information">Fee: 0.5%</p>

			<div class="toggle-container">
				<button
					id="abort-btn"
					on:click={() => {
						dialog.close();
					}}>Back</button
				>
				{#if isConfirmBusy}
					<button id="confirm-btn">
						<div class="spinner"></div>
					</button>
				{:else}
					<button id="confirm-btn" title="test-cancel-confirmation" on:click={handleCancellation}
						>Confirm</button
					>
				{/if}
			</div>
		</div>
	{/if}
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
	}

	p {
		font-family: var(--secondary-font);
		margin: 0.2em 0;
		color: var(--title-color);
	}

	a {
		text-decoration: underline;
		color: var(--title-color);
	}

	/* === Layout === */
	.warning-container {
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

	.review-container {
		display: flex;
		flex-direction: column;
		border-radius: 8px;
		background: var(--background-color);
		border: var(--input-border);
		padding: 0.5em 1em;
		margin: 0.5em 0;
	}

	/* === Components === */
	#confirm-btn,
	#abort-btn {
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

	#confirm-btn:hover,
	#abort-btn:hover {
		transform: scale(0.95);
		transition: all 0.3s;
		box-shadow: 6px 6px 0 0 black;
	}

	#abort-btn {
		background: var(--main-button-text-color);
		color: var(--main-color);
	}

	#confirm-btn {
		background: var(--main-color);
		color: var(--main-button-text-color);
	}

	.main-information {
		color: var(--title-color);
	}

	.secondary-information {
		color: var(--text-color);
		margin-left: 1em;
	}

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

	.change-btn:hover {
		transform: scale(1.2);
		animation: invert 0.5s ease;
	}

	#exchange-rate {
		display: flex;
		align-items: center;
		gap: 0.5em;
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

	@media (max-width: 767px) {
		.warning-container {
			width: 80%;
		}
	}
</style>
