<script lang="ts">
	import { waterNeuronInfo } from '$lib/stores';
	import { displayUsFormat } from '$lib';
	import BigNumber from 'bignumber.js';
	import { onMount, afterUpdate } from 'svelte';
	import { fade } from 'svelte/transition';

	let totalStaked: BigNumber;
	let apy: BigNumber;
	let stakersCount: Number;

	async function getLedgerBalanceStoreEntries() {
		try {
			const url = 'https://buwm7-7yaaa-aaaar-qagva-cai.raw.icp0.io/metrics';
			const response = await fetch(url);

			if (!response.ok) {
				throw new Error(`HTTP error! status: ${response.status}`);
			}

			const data = await response.text();
			const match = data.match(/ledger_balance_store_entries\s+(\d+)/);
			if (match) {
				stakersCount = parseInt(match[1], 10);
			}
		} catch (error) {
			console.error('Error fetching or parsing data:', error);
		}
	}

	afterUpdate(() => {
		if ($waterNeuronInfo) {
			try {
				apy = $waterNeuronInfo.apy();
				totalStaked = $waterNeuronInfo.neuron8yStake().plus($waterNeuronInfo.neuron6mStake());
			} catch (error) {
				console.error('Error fetching data:', error);
			}
		}
	});

	onMount(() => {
		getLedgerBalanceStoreEntries();
	});
</script>

<div class="stat-widget-container" in:fade={{ duration: 500 }}>
	<div class="stat-item">
		<b>TVL</b>
		<b>
			{#if totalStaked}
				{displayUsFormat(totalStaked, 0)} ICP
			{:else}
				-/-
			{/if}
		</b>
	</div>
	<div class="stat-item">
		<b>APY</b>
		<b
			>{#if apy}
				{displayUsFormat(BigNumber(100).multipliedBy(apy), 1)}%
			{:else}
				-/-
			{/if}</b
		>
	</div>
	<div class="stat-item">
		<b>Holders</b>
		<b>
			{#if stakersCount || stakersCount === 0}
				{stakersCount}
			{:else}
				-/-
			{/if}
		</b>
	</div>
</div>

<style>
	/* === Layout === */
	.stat-widget-container {
		background: var(--background-color);
		border: var(--border-size) solid var(--border-color);
		color: var(--stake-text-color);
		padding: 1em;
		padding-left: 2em;
		padding-right: 2em;
		border-radius: 15px;
		display: flex;
		flex-direction: row;
		gap: 1em;
	}

	/* === Components === */

	.stat-item {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		font-family: var(--secondary-font);
	}
</style>
