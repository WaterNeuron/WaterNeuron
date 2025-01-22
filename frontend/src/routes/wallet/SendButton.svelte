<script lang="ts">
	import { assetToIconPath, displayUsFormat, isMobile } from '$lib';
	import { user, selectedAsset, inSendingMenu, inReceivingMenu, ledgerDevice } from '$lib/stores';
	import { BigNumber } from 'bignumber.js';
	import QRCodeScannerIcon from '$lib/icons/QRCodeScannerIcon.svelte';
	import UpIcon from '$lib/icons/UpIcon.svelte';
	import { onMount } from 'svelte';

	export let asset: 'ICP' | 'nICP' | 'WTN';
	let balance: BigNumber | undefined;

	const fetchBalance = () => {
		if ($user?.account === 'ledger') {
			balance = $ledgerDevice?.getBalance(asset);
		} else {
			balance = $user?.getBalance(asset);
		}
	};

	onMount(() => {
		fetchBalance();

		const intervalId = setInterval(async () => {
			fetchBalance();
		}, 5000);

		return () => clearInterval(intervalId);
	});
</script>

<div class="token-balance-container">
	<div class="balance">
		<p>
			{balance ? displayUsFormat(balance, 8) : '-/-'}
			{asset}
		</p>
		<img alt="{asset} logo" src={assetToIconPath(asset)} width="30px" height="30px" />
	</div>
	<div class="btns-container">
		{#if isMobile}
			<button
				class="mobile-action-btn"
				on:click={() => {
					inReceivingMenu.set(true);
					selectedAsset.set(asset);
				}}
			>
				<QRCodeScannerIcon color="--main-color" />
			</button>
			<button
				class="mobile-action-btn"
				on:click={() => {
					inSendingMenu.set(true);
					selectedAsset.set(asset);
				}}
			>
				<UpIcon />
			</button>
		{:else}
			<button
				class="action-btn"
				on:click={() => {
					inReceivingMenu.set(true);
					selectedAsset.set(asset);
				}}
			>
				Receive
			</button>
			<button
				class="action-btn"
				title="send-btn-{asset}"
				on:click={() => {
					inSendingMenu.set(true);
					selectedAsset.set(asset);
				}}
			>
				Send
			</button>
		{/if}
	</div>
	{#if asset === 'WTN' && $user?.account !== 'ledger'}
		<p class="airdrop-allocation">
			{#if isMobile}
				Airdrop:
			{:else}
				Airdrop Allocation:
			{/if}
			{#if $user}
				{displayUsFormat($user.wtnAllocation())}
			{:else}
				-/-
			{/if} WTN
		</p>
	{/if}
</div>

<style>
	/* === Base Styles === */
	p {
		font-family: var(--secondary-font);
	}

	/* === Layout === */
	.token-balance-container {
		display: flex;
		justify-content: space-between;
		position: relative;
		margin-left: 1em;
		align-items: center;
	}

	.btns-container {
		display: flex;
		align-items: center;
	}

	/* === Components ==== */
	.balance {
		display: flex;
		align-items: center;
		gap: 5px;
	}

	.airdrop-allocation {
		position: absolute;
		color: var(--text-color);
		top: 50%;
		width: 60%;
		margin-top: 1em;
		font-family: var(--secondary-font);
	}

	.mobile-action-btn {
		border: none;
		background: transparent;
		display: flex;
		cursor: pointer;
		color: var(--main-color);
	}

	.action-btn {
		background: var(--main-color);
		color: var(--main-button-text-color);
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
		margin-right: 1em;
	}

	.action-btn:hover {
		transform: scale(0.95);
		transition: all 0.3s;
		box-shadow: 6px 6px 0 0 black;
	}
</style>
