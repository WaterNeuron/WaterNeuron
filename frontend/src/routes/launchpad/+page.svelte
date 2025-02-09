<script lang="ts">
	import { HttpAgent, Actor } from '@dfinity/agent';
	import { idlFactory as idlFactorySns } from './sns_module';
	import { Principal } from '@dfinity/principal';
	import type { _SERVICE as snsModuleInterface, Status } from './sns_module.did';
	import { AccountIdentifier } from '@dfinity/ledger-icp';
	import SuccessIcon from '$lib/icons/SuccessIcon.svelte';
	import CopyIcon from '$lib/icons/CopyIcon.svelte';
	import { afterUpdate, onMount } from 'svelte';
	import { Toast } from '$lib/toast';
	import { toasts, canisters, inQrDestination } from '$lib/stores';
	import BigNumber from 'bignumber.js';
	import { isMobile, displayUsFormat, bigintE8sToNumber } from '$lib';
	import QrCodeScannerIcon from '$lib/icons/QRCodeScannerIcon.svelte';
	import { DEV, HOST } from '$lib/authentification';
	import { fade } from 'svelte/transition';
	import QrDestination from './QrDestination.svelte';

	let status: Status = {
		participants: 0n,
		time_left: [],
		start_at: 0n,
		minimum_deposit_amount: 0n,
		total_icp_deposited: 0n,
		end_at: 0n
	};
	let destination: string;
	let participant: string;
	let balance: bigint;
	let snsCanister: snsModuleInterface;
	let isNotAvailable = false;
	let snsRatio = 0;
	let icpDepositedSns: bigint;
	let wtnClaimable: bigint;
	let minCommitment: HTMLDivElement;
	let selector: HTMLDivElement;
	let currentBar: HTMLDivElement;
	let totalBar: HTMLDivElement;

	export function displaySnsTimeLeft(timeLeft: number): string {
		if (timeLeft === 0) return 'The SNS has ended.';

		const days = Math.floor(timeLeft / (3600 * 24));
		const hours = Math.floor((timeLeft % (3600 * 24)) / 3600);
		const minutes = Math.floor((timeLeft % (60 * 60)) / 60);
		const seconds = timeLeft % 60;

		if (days > 0) {
			return `${days} day${days > 1 ? 's' : ''} and ${hours} hour${hours > 1 ? 's' : ''} left.`;
		}
		if (hours > 0) {
			return `${hours} hour${hours > 1 ? 's' : ''} and ${minutes} minute${minutes > 1 ? 's' : ''} left.`;
		}
		if (minutes > 0) {
			return `${minutes} minute${minutes > 1 ? 's' : ''} and ${seconds} second${seconds > 1 ? 's' : ''} left.`;
		}
		return `${seconds} second${seconds > 1 ? 's' : ''} left.`;
	}

	function displayAccountId(accountId: string) {
		if (accountId === undefined) return '';
		if (isMobile) {
			return (
				accountId.slice(0, 4) + '...' + accountId.slice(accountId.length - 4, accountId.length)
			);
		}
		return accountId;
	}

	function displayDate(timestamp: bigint): string {
		const date = new Date(Number(timestamp * 1_000n));

		const formattedDate = date.toLocaleString('en-US', {
			year: 'numeric',
			month: 'long',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit',
			second: '2-digit'
		});

		return formattedDate;
	}

	const updateIcpDeposited = async () => {
		if (!participant) return;
		icpDepositedSns = await snsCanister.get_icp_deposited(Principal.fromText(participant));
	};

	const updateWTNClaimable = async () => {
		if (!participant) return;
		wtnClaimable = await snsCanister.get_wtn_allocated(Principal.fromText(participant));
	};

	const setPositions = () => {
		if (!selector || !currentBar || !minCommitment) return;

		const barWidth = totalBar.offsetWidth;
		const ratio = bigintE8sToNumber(status.total_icp_deposited).toNumber() / 23_295_621;
		const previousSnsCheckpoint = (3 * barWidth) / 4;
		const previousSnsCheckRatio = 0.01526;
		snsRatio = ratio;
		selector.style.left = `${ratio >= (4 * previousSnsCheckRatio) / 3 ? barWidth : (ratio * previousSnsCheckpoint) / previousSnsCheckRatio}px`;
		currentBar.style.width = `${ratio >= (4 * previousSnsCheckRatio) / 3 ? barWidth : (ratio * previousSnsCheckpoint) / previousSnsCheckRatio}px`;
		minCommitment.style.left = `${(3 * barWidth) / 4}px`;
	};

	const setupLaunchpad = async () => {
		const agent = HttpAgent.createSync({
			host: HOST
		});

		if (DEV) {
			agent.fetchRootKey();
		}

		const CANISTER_ID = 'zftzm-qqaaa-aaaam-adxfa-cai';

		snsCanister = Actor.createActor(idlFactorySns, {
			agent,
			canisterId: CANISTER_ID
		});
	};

	const claimWtn = async () => {
		isNotAvailable = true;
		try {
			const result = await snsCanister.claim_wtn(Principal.fromText(participant));
			const key = Object.keys(result)[0];
			switch (key) {
				case 'Err':
					toasts.add(Toast.error((result as { Err: string }).Err));
					break;
				case 'Ok':
					toasts.add(
						Toast.success(`Successful transfer at block ${(result as { Ok: bigint }).Ok}`)
					);
			}
		} catch (e) {
			console.error(e);
		}
		isNotAvailable = false;
	};

	const updateBalance = async () => {
		if (!destination || !$canisters) return;
		try {
			const { e8s } = await $canisters.icpLedger.anonymousActor.account_balance({
				account: AccountIdentifier.fromHex(destination).toUint8Array()
			});
			balance = e8s;
		} catch (e) {
			console.log(e);
		}
	};

	const fetchStatus = async () => {
		try {
			status = await snsCanister.get_status();
		} catch (e) {
			console.log(e);
		}
	};

	const setDestination = async () => {
		isNotAvailable = true;
		try {
			destination = await snsCanister.get_icp_deposit_address(Principal.fromText(participant));
			await updateBalance();
			await updateIcpDeposited();
			await updateWTNClaimable();
		} catch (e) {
			console.log(e);
		}
		isNotAvailable = false;
	};

	const notifyDeposit = async () => {
		isNotAvailable = true;
		try {
			const result = await snsCanister.notify_icp_deposit(Principal.fromText(participant), balance);
			const key = Object.keys(result)[0];
			switch (key) {
				case 'Err':
					toasts.add(Toast.error((result as { Err: string }).Err));
					break;
				case 'Ok':
					toasts.add(
						Toast.success(`Successful transfer at block ${(result as { Ok: bigint }).Ok}`)
					);
			}
		} catch (e) {
			console.log(e);
		}
		isNotAvailable = false;
	};

	onMount(() => {
		setupLaunchpad().then(async () => {
			await fetchStatus();
		});

		const intervalId = setInterval(async () => {
			await fetchStatus();
			await updateBalance();
			await updateIcpDeposited();
			await updateWTNClaimable();
			setPositions();
		}, 5000);

		return () => clearInterval(intervalId);
	});

	afterUpdate(() => {
		setPositions();
	});
</script>

{#if $inQrDestination}
	<QrDestination {destination} />
{/if}
<main>
	<div class="header-container">
		<p>
			WaterNeuron is a liquid staking protocol designed for the Internet Computer network. Staking
			ICP becomes straightforward and efficient.
		</p>
		<span
			><a target="_blank" href="https://docs.waterneuron.fi">https://docs.waterneuron.fi</a> |
			<a target="_blank" href="https://x.com/waterneuron">https://x.com/waterneuron</a>
			| <a target="_blank" href="https://t.me/waterneuron">https://t.me/waterneuron</a>
		</span>
	</div>
	<div class="core-container">
		<div class="parameters-container">
			<div class="parameter">
				<span>Token Name</span> <span>WaterNeuron</span>
			</div>

			<div class="parameter">
				<span>Token Symbol</span> <span>WTN</span>
			</div>

			<div class="parameter">
				<span>Total Supply</span> <span>116'479'684.72 WTN</span>
			</div>

			<div class="parameter">
				<span>Minimum Participant Commitment</span> <span>10 ICP</span>
			</div>

			<div class="parameter">
				<span>Swap Start</span> <span>{displayDate(status.start_at)}</span>
			</div>

			<div class="parameter">
				<span>Swap End</span> <span>{displayDate(status.end_at)}</span>
			</div>
		</div>
		<div class="sns-status-container">
			<h2>Status</h2>
			<div class="parameter">
				<span>Total Participants</span>
				<span>{status.participants.toString()}</span>
			</div>
			<div class="parameter">
				<span>Tokens Distributed</span>
				<span>23'295'621 WTN</span>
			</div>
			<div class="commitment-container">
				<div class="overall-commitment">
					<div class="parameter">
						<span>Overall Commitment</span>
						<span>{displayUsFormat(bigintE8sToNumber(status.total_icp_deposited))} ICP</span>
					</div>
					<div class="progress-bar">
						<div bind:this={selector} class="triangle-down"></div>
						<div bind:this={minCommitment} class="checkpoint"></div>
						<div bind:this={currentBar} class="bar bar--1"></div>
						<div bind:this={totalBar} class="bar bar--2"></div>
					</div>
				</div>
			</div>

			<div class="parameter">
				<span style="color: #4d79ff">Current SNS</span>
				<span style="color: #4d79ff"
					>{snsRatio === 0 ? 0 : displayUsFormat(BigNumber(1 / snsRatio))} WTN/ICP</span
				>
			</div>
			<div class="parameter">
				<span style="color: #faa123">Previous SNS</span>
				<span style="color: #faa123">{displayUsFormat(BigNumber(65.5))} WTN/ICP</span>
			</div>
			<div class="parameter">
				<span>{displaySnsTimeLeft(Number(status.time_left))}</span>
			</div>
		</div>
		<div class="participate-container">
			<div
				class:blur={destination === undefined}
				class:visible={destination !== undefined}
				class="submit-container"
				style:align-items={status.time_left[0] !== 0n ? 'start' : 'center'}
			>
				{#if status.time_left[0] !== 0n}
					<h2>Participate</h2>
					<div class="destination-container">
						<span> Send ICP to the following destination: </span>
						<span style="margin-left: 10px;" id="destination-accountId">
							{displayAccountId(destination)}
						</span>
						<button
							class="raw icon-btn"
							on:click={() => navigator.clipboard.writeText(destination ?? '')}
						>
							<CopyIcon />
						</button>
						<button
							class="raw icon-btn"
							on:click={() => {
								inQrDestination.set(true);
							}}
						>
							<QrCodeScannerIcon color="--title-color" />
						</button>
					</div>
					<div class="destination-container">
						<span>ICP available for commit: </span>
						<span style="margin-left: 10px;" id="destination-icp-balance">
							{#if balance !== undefined}
								{displayUsFormat(bigintE8sToNumber(balance))} ICP
							{:else}
								-/- ICP
							{/if}
						</span>
					</div>
					<div class="destination-container">
						<span>ICP deposited in the SNS: </span>
						<span style="margin-left: 10px;" id="destination-icp-balance">
							{#if icpDepositedSns !== undefined}
								{displayUsFormat(bigintE8sToNumber(icpDepositedSns))} ICP
							{:else}
								-/- ICP
							{/if}
						</span>
					</div>
					<button class="commit-btn" on:click={notifyDeposit}>
						{#if isNotAvailable}
							<div class="spinner spinner-type-1"></div>
						{:else}
							Commit
						{/if}
					</button>
				{:else}
					<h2>🎉 The swap is successful.</h2>
					<div class="destination-container">
						<span>You have successfully deposited</span>
						<span style="margin-left: 0.5em;" id="destination-icp-balance">
							{#if icpDepositedSns !== undefined}
								{displayUsFormat(bigintE8sToNumber(icpDepositedSns))} ICP.
							{:else}
								-/- ICP.
							{/if}
						</span>
					</div>
					<div class="destination-container">
						<span>Claimable:</span>
						<span style="margin-left: 0.5em;" id="destination-icp-balance">
							{#if wtnClaimable !== undefined}
								{displayUsFormat(bigintE8sToNumber(wtnClaimable))} WTN
							{:else}
								-/- WTN.
							{/if}
						</span>
					</div>
					<button class="commit-btn" on:click={claimWtn}>
						{#if isNotAvailable}
							<div class="spinner spinner-type-1"></div>
						{:else}
							Claim WTN
						{/if}
					</button>
				{/if}
			</div>
			{#if destination === undefined}
				<div class="register-container" out:fade={{ duration: 500 }}>
					<span>Register your NNS principal to participate in the SNS.</span>
					<div class="derive-container">
						<input class="raw derive-input" placeholder="Principal" bind:value={participant} />
						<button class="raw derive-btn" on:click={setDestination}>
							{#if isNotAvailable}
								<div class="spinner spinner-type-2"></div>
							{:else}
								<SuccessIcon color="--title-color" />
							{/if}
						</button>
					</div>
				</div>
			{/if}
		</div>
	</div>
</main>

<style>
	@font-face {
		font-family: 'Circular';
		src: url('/CircularXXWeb-Book.woff2') format('truetype');
		font-weight: normal;
		font-style: normal;
	}

	.parameter {
		display: flex;
		justify-content: space-between;
		align-items: center;
		width: 100%;
		padding: 0.4em 0;
	}

	.core-container {
		width: 100%;
		display: grid;
		grid-template-columns: 1fr 1fr 1fr 1fr;
		gap: 1em;
		justify-content: space-evenly;
	}

	.parameters-container,
	.sns-status-container,
	.participate-container {
		display: flex;
		justify-content: center;
		flex-direction: column;
		background-color: var(--background-color);
		border-radius: 8px;
		padding: 1em;
		box-sizing: border-box;
		border: var(--input-border);
	}

	h2 {
		margin: 0;
	}

	h2,
	p,
	span,
	a {
		color: var(--title-color);
	}

	.spinner {
		width: 1em;
		height: 1em;
		border-radius: 50%;
		animation: spin 1s linear infinite;
	}

	.spinner-type-1 {
		border: 2px solid var(--main-button-text-color);
		border-top-color: transparent;
	}

	.spinner-type-2 {
		border: 2px solid var(--title-color);
		border-top-color: transparent;
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

	.parameters-container {
		grid-column: 1 / span 2;
	}

	.sns-status-container {
		grid-column: 3 / span 2;
	}

	.participate-container {
		grid-column: 1 / span 4;
		position: relative;
		width: 100%;
		height: 250px;
	}

	.register-container,
	.submit-container {
		position: absolute;
		width: 100%;
		height: 100%;
		top: 0;
		left: 0;
		box-sizing: border-box;
		padding: 1em;
		border-radius: 8px;
	}

	.register-container {
		z-index: 1;
		display: flex;
		gap: 1em;
		align-items: center;
		justify-content: center;
		flex-direction: column;
		width: 100%;
		height: 100%;
		background-color: var(--background-color-transparent);
	}

	.submit-container {
		flex-direction: column;
		gap: 1em;
		justify-content: space-around;
		align-items: center;
	}

	.commit-btn {
		background: rgb(77, 121, 255);
		color: var(--main-button-text-color);
		border-radius: 8px;
		position: relative;
		border: 2px solid black;
		font-size: 14px;
		box-shadow: 3px 3px 0 0 black;
		padding: 0 1em 0 1em;
		max-width: none;
		align-self: center;
		height: 3em;
		font-weight: bold;
		display: flex;
		justify-content: center;
		align-items: center;
		cursor: pointer;
	}

	.commit-btn:hover {
		transform: scale(0.95);
		transition: all 0.3s;
		box-shadow: 6px 6px 0 0 black;
	}

	.destination-container {
		display: flex;
		align-items: center;
	}

	span {
		text-wrap: wrap;
		word-break: break-word;
	}

	.visible {
		display: flex;
	}

	.blur {
		filter: blur(5px);
		display: flex;
	}

	.raw,
	.derive-input:focus {
		background: none;
		border: none;
		text-decoration: none;
		display: flex;
		align-items: center;
		outline: none;
		appearance: none;
	}

	input {
		color: var(--title-color);
	}

	.derive-btn:hover,
	.icon-btn:hover {
		transform: scale(0.9);
		animation: invert 0.5s ease;
	}

	.derive-container {
		display: flex;
		align-items: center;
		justify-content: space-between;
		border-radius: 8px;
		border: var(--input-border);
		padding: 0.5em;
	}

	main {
		font-family: 'Circular', sans-serif;
		margin: 0;
		background-size: cover;
		background-attachment: fixed;
		overflow-x: hidden;
		flex-grow: 1;
	}

	.bar {
		border-radius: 8px;
		height: 8px;
		position: absolute;
	}

	.bar--1 {
		background-color: #4d79ff;
		z-index: 1;
	}

	.bar--2 {
		background-color: #dadef2;
		width: 100%;
	}

	.triangle-down {
		position: absolute;
		transform: translate(-50%, -100%);
		width: 0;
		height: 0;
		border-left: 5px solid transparent;
		border-right: 5px solid transparent;
		border-top: 10px solid #4d79ff;
	}

	.checkpoint {
		background-color: #faa123;
		height: 8px;
		width: 2px;
		position: absolute;
		transform: translate(-50%, -50%);
		translate: 0 4px;
		z-index: 2;
	}

	.progress-bar {
		position: relative;
	}

	.overall-commitment {
		display: flex;
		flex-direction: column;
		gap: 0.5em;
		margin-bottom: 1em;
	}

	main {
		display: flex;
		flex-direction: column;
		padding: 1em;
		gap: 1em;
		box-sizing: border-box;
		width: 100vw;
	}

	@media (max-width: 767px) {
		.core-container {
			display: grid;
			grid-template-columns: 1fr;
		}

		.parameters-container {
			grid-column: 1;
			width: 100%;
		}

		.sns-status-container {
			grid-column: 1;
			width: 100%;
		}

		.participate-container {
			grid-column: 1;
		}
	}
</style>
