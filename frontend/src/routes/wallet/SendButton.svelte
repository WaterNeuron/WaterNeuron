<script lang="ts">
	import { AssetType, displayUsFormat } from '$lib';
	import { user, sendAsset, isSelecting, state } from '$lib/stores';
	import BigNumber from 'bignumber.js';
	import { onMount } from 'svelte';
	import { fade } from 'svelte/transition';

	export let asset;
	let wtnAllocation: BigNumber;

	const fetchAllocation = async () => {
		if ($state) wtnAllocation = await $state.wtnAllocation();
	};
	onMount(() => {
		fetchAllocation();
		const intervalId = setInterval(fetchAllocation, 5000);

		return () => clearInterval(intervalId);
	});
</script>

<div class="token-balance-container" in:fade={{ duration: 500 }}>
	<div class="balance">
		<p>
			{displayUsFormat($user ? $user.getBalance(asset.type) : BigNumber(0), 8)}
			{asset.intoStr()}
		</p>
		<img alt="{asset.intoStr()} logo" src={asset.getIconPath()} width="30px" height="30px" />
	</div>
	<button
		class="swap-btn"
		on:click={() => {
			isSelecting.set(true);
			sendAsset.set(asset);
		}}>Send</button
	>
	{#if asset.type === AssetType.WTN}
		<p class="airdrop-allocation">
			Airdrop Allocation:
			{#if wtnAllocation}
				{displayUsFormat(wtnAllocation)}
			{:else}
				...
			{/if} WTN
		</p>
	{/if}
</div>

<style>
	/* === Base Styles === */
	p {
		font-family: var(--font-type2);
	}

	button {
		color: black;
	}

	/* === Layout === */
	.token-balance-container {
		display: flex;
		justify-content: space-between;
		position: relative;
		margin-left: 1em;
	}

	/* === Components ==== */
	.balance {
		display: flex;
		align-items: center;
		gap: 5px;
	}

	.airdrop-allocation {
		position: absolute;
		color: lightgrey;
		top: 50%;
		width: 60%;
		margin-top: 1em;
		font-family: var(--font-type2);
	}

	.swap-btn {
		border: 2px solid black;
		box-shadow: 3px 3px 0 0 black;
		background-color: var(--main-color);
		align-items: center;
		padding: 0 2em;
		font-weight: bold;
		justify-content: center;
		cursor: pointer;
		display: flex;
	}

	.swap-btn:hover {
		transform: scale(0.95);
		transition: all 0.3s;
		box-shadow: 6px 6px 0 0 black;
	}
</style>
