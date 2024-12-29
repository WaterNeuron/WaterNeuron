<script lang="ts">
	import StatsWidget from '../stake/StatsWidget.svelte';
	import SnsListing from './SnsListing.svelte';
	import CopyIcon from '$lib/icons/CopyIcon.svelte';
	import { fade, scale } from 'svelte/transition';
	import {
		sns,
		canisters,
		isBusy,
		toasts,
		handleSnsChange,
		inputAmount,
		handleInputAmount
	} from '$lib/stores';
	import { Toast } from '$lib/toast';
	import { handleSnsIcpDepositResult, handleSnsRetrieveNicpResult } from '$lib/resultHandler';
	import { Principal } from '@dfinity/principal';
	import BigNumber from 'bignumber.js';
	import { displayUsFormat, isPrincipalValid, numberToBigintE8s, principalToHex } from '$lib';

	let isConfirmBusy: boolean;
	let isRetrieveBusy: boolean;
	let principalInput: string;

	const handlePrincipalInputChange = async () => {
		if ($sns.name !== 'Custom') return;

		const filteredInput = principalInput.replace(/\s+/g, '');
		const shouldChange = isPrincipalValid(filteredInput);
		if (shouldChange) {
			await handleSnsChange('Custom', filteredInput);
		}
	};

	const notifyIcpDeposit = async () => {
		if ($isBusy || !$canisters || !isPrincipalValid($sns.principal)) return;
		try {
			isBusy.set(true);
			isConfirmBusy = true;
			const boomerangResult = await $canisters.boomerang.anonymousActor.notify_icp_deposit(
				Principal.fromText($sns.principal)
			);
			const result = handleSnsIcpDepositResult(boomerangResult);
			if (result.success) {
				toasts.add(Toast.success(result.message));
			} else {
				toasts.add(Toast.error(result.message));
			}
		} catch (error) {
			console.log(error);
			toasts.add(Toast.error('Notify ICP deposit call failed, please retry.'));
		}
		isBusy.set(false);
		isConfirmBusy = false;
	};

	async function retrieveNicp() {
		if ($isBusy || !$canisters || !isPrincipalValid($sns.principal)) return;
		try {
			isBusy.set(true);
			isRetrieveBusy = true;
			const retrieveResult = await $canisters.boomerang.anonymousActor.retrieve_nicp(
				Principal.fromText($sns.principal)
			);
			const result = handleSnsRetrieveNicpResult(retrieveResult);
			if (result.success) {
				toasts.add(Toast.success(result.message));
			} else {
				toasts.add(Toast.error(result.message));
			}
		} catch (error) {
			console.log(error);
			toasts.add(Toast.error('Retrieve nICP call failed, please retry.'));
		}
		isBusy.set(false);
		isRetrieveBusy = false;
	}

	let isAnimating = false;
	let isCircleVisible = false;

	const handleAnimation = () => {
		if (!isAnimating) {
			isAnimating = true;
			isCircleVisible = true;
			setTimeout(() => {
				isCircleVisible = false;
				setTimeout(() => {
					isAnimating = false;
				}, 500);
			}, 500);
		}
	};
</script>

<StatsWidget />
<div class="sns-stake-container" in:fade>
	<SnsListing />
	{#key $sns.name}
		<div class="boomerang-container" in:fade={{ duration: 500 }}>
			<div class="top-container">
				<h1>Stake <span style:color="var(--main-color)">{$sns.name}</span> Treasury</h1>
				<div class="sns-info-container">
					<span class="governance-id">
						{#if $sns.name === 'Custom'}
							Principal: <input
								type="text"
								placeholder="Address"
								bind:value={principalInput}
								on:input={handlePrincipalInputChange}
							/>
						{:else}
							Goverance id: <a
								target="blank"
								href="https://dashboard.internetcomputer.org/canister/{$sns.principal}"
								class="dashboard">{$sns.principal}</a
							>
						{/if}
					</span>
					<div class="balances-container">
						{#if $sns.icpBalance}
							<a
								target="blank"
								href="https://dashboard.internetcomputer.org/account/{principalToHex(
									$sns.principal
								)}"
								class="balance dashboard">{displayUsFormat($sns.icpBalance)} ICP</a
							>
						{:else}
							<span class="balance">-/- ICP</span>
						{/if}
						{#if $sns.nicpBalance}
							<span class="balance">{displayUsFormat($sns.nicpBalance)} nICP</span>
						{:else}
							<span class="balance">-/- nICP</span>
						{/if}
					</div>
				</div>
			</div>
			<div class="step-container" in:fade={{ duration: 500 }}>
				<div class="instruction-container">
					<div class="number-step-container">
						<span class="round">1</span>
					</div>
					<span>
						Submit a proposal to transfer ICP from the SNS Treasury to the following destination.
					</span>
				</div>
				<div class="account-container">
					<div class="principal-container">
						{#if $sns.encodedBoomerangAccount}
							<p>{$sns.encodedBoomerangAccount}</p>
						{:else}
							<p>-/-</p>
						{/if}
						<button
							class="copy-btn"
							on:click={() => {
								handleAnimation();
								navigator.clipboard.writeText($sns.encodedBoomerangAccount ?? '');
							}}
						>
							<CopyIcon />
							{#if isCircleVisible}
								<div class="circle" transition:scale={{ duration: 500 }}></div>
							{/if}
						</button>
					</div>
					<span class="sns-amount">
						Choose the amount of ICP to transfer:
						<input
							title="sns-amount-input"
							type="text"
							maxlength="20"
							bind:value={$inputAmount}
							placeholder="Amount"
							on:input={handleInputAmount}
						/>
					</span>
				</div>
				{#if BigNumber($inputAmount).isNaN()}
					<a
						class="action-btn"
						href="https://proposals.network/submit?g={$sns.principal}&action=TransferSnsTreasuryFunds&destination={$sns.encodedBoomerangAccount}"
						target="blank"
					>
						Make a proposal
					</a>
				{:else}
					<a
						class="action-btn"
						href="https://proposals.network/submit?g={$sns.principal}&action=TransferSnsTreasuryFunds&destination={$sns.encodedBoomerangAccount}&amount={numberToBigintE8s(
							BigNumber($inputAmount)
						)}"
						target="blank"
					>
						Make a proposal
					</a>
				{/if}
			</div>
			<div class="step-container" in:fade={{ duration: 500 }}>
				<div class="instruction-container">
					<div class="number-step-container">
						<span class="round">2</span>
					</div>
					<span>Once the proposal is executed, notify the protocol of the transfer.</span>
				</div>
				{#if isConfirmBusy}
					<button class="action-btn">
						<div class="spinner"></div>
					</button>
				{:else}
					<button class="action-btn" title="notifyIcpDeposit-btn" on:click={notifyIcpDeposit}
						>Confirm SNS deposit</button
					>
				{/if}
			</div>
			<div class="step-container" in:fade={{ duration: 500 }}>
				<div class="instruction-container">
					<div class="number-step-container">
						<span class="round">3</span>
					</div>
					<span>Collect the minted nICP tokens to the governance canister of the SNS.</span>
				</div>
				{#if isRetrieveBusy}
					<button class="action-btn">
						<div class="spinner"></div>
					</button>
				{:else}
					<button class="action-btn" title="retrieveNicp-btn" on:click={retrieveNicp}
						>Retrieve nICP</button
					>
				{/if}
			</div>
		</div>
	{/key}
</div>

<style>
	/* === Base Styles === */
	span {
		color: var(--text-color);
		overflow-wrap: anywhere;
		font-family: var(--secondary-font);
	}

	p {
		font-family: var(--secondary-font);
		font-weight: bold;
		overflow-wrap: anywhere;
		margin: 0;
	}

	.dashboard {
		color: var(--stake-text-color);
		padding: 0.5em;
		font-family: var(--secondary-font);
	}

	h1 {
		color: var(--title-color);
		font-size: 26px;
		font-family: var(--main-font);
		align-self: center;
		margin: 0;
	}

	input {
		border: var(--input-border);
		padding-left: 0.4em;
		height: 2em;
		font-size: 15px;
		color: var(--stake-text-color);
		background: var(--input-color);
		outline: none;
		margin-left: 1em;
		border-radius: 0.4em;
	}

	/* === Layout === */
	.sns-stake-container {
		background-color: var(--background-color);
		border: var(--input-border);
		border-radius: 10px;
		display: flex;
		height: 44em;
		width: 60em;
		max-width: 95dvw;
	}

	.boomerang-container {
		display: flex;
		flex-direction: column;
		width: 80%;
		align-items: start;
		justify-content: start;
		gap: 2em;
		padding: 3em;
	}

	.step-container {
		display: flex;
		flex-direction: column;
		background: none;
		align-items: center;
		height: fit-content;
		width: 100%;
		border: none;
		gap: 1em;
	}

	.principal-container {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 80%;
		height: 4em;
		color: var(--stake-text-color);
	}

	.account-container {
		display: flex;
		flex-direction: column;
		gap: 1em;
		width: 100%;
		justify-content: center;
		align-items: center;
	}

	.number-step-container {
		display: flex;
		align-items: center;
		gap: 1em;
	}

	.top-container {
		display: flex;
		flex-direction: column;
		width: 100%;
		align-items: center;
		gap: 1em;
	}

	.sns-info-container {
		display: flex;
		width: 90%;
		justify-content: space-between;
		align-items: center;
		height: 4em;
	}

	.instruction-container {
		display: flex;
		align-items: center;
		gap: 1em;
		width: 90%;
	}

	.balances-container {
		display: flex;
		flex-direction: column;
		align-items: end;
		width: fit-content;
	}

	/* === Component === */
	.action-btn {
		background: var(--main-color);
		border: 2px solid black;
		border-radius: 8px;
		box-shadow: 3px 3px 0 0 black;
		font-size: 16px;
		font-weight: bold;
		font-family: var(--secondary-font);
		display: flex;
		justify-content: center;
		align-items: center;
		width: 15em;
		height: 3em;
		text-decoration: none;
		color: var(--main-button-text-color);
	}

	.action-btn:hover {
		transform: scale(0.95);
		transition: all 0.3s;
		box-shadow: 6px 6px 0 0 black;
	}

	.copy-btn {
		background-color: transparent;
		border: none;
		cursor: pointer;
		border-radius: 0.3em;
		transition: all 0.3s ease;
		color: white;
		font-weight: bold;
		display: flex;
		position: relative;
	}

	.balance {
		display: flex;
		text-align: end;
		width: fit-content;
		padding: 0;
		color: var(--stake-text-color);
		font-family: var(--main-font);
		font-size: 18px;
	}

	.governance-id {
		color: var(--stake-text-color);
		width: 60%;
		display: flex;
		align-items: center;
	}

	.sns-amount {
		color: var(--stake-text-color);
		width: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	/* === Utilities === */
	.round {
		border-radius: 50%;
		color: var(--text-color);
		border: 2px solid;
		width: 1em;
		height: fit-content;
		padding: 0.2em;
		font-weight: bold;
		text-align: center;
		font-family: var(--font-type2);
	}

	.circle {
		position: absolute;
		border-radius: 50%;
		background-color: rgb(37, 139, 255, 0.5);
		width: 25px;
		height: 25px;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
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
		.sns-stake-container {
			flex-direction: column;
			justify-content: start;
			align-items: center;
			height: fit-content;
		}

		.boomerang-container {
			width: 95%;
			padding: 1em 0 1em 0;
			gap: 2em;
			align-items: center;
		}

		.step-container {
			width: 98%;
			justify-content: center;
		}

		.sns-info-container {
			flex-direction: column;
			gap: 1em;
			align-items: center;
		}

		.instruction-container {
			gap: 1em;
		}

		.balances-container {
			display: none;
		}

		.dashboard {
			font-size: 15px;
		}

		.governance-id {
			width: 100%;
			justify-content: center;
		}
	}
</style>
