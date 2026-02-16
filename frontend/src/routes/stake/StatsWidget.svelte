<script lang="ts">
	import { waterNeuronInfo } from '$lib/stores';
	import { displayNumber } from '$lib';
	import { onMount } from 'svelte';
	import { fade } from 'svelte/transition';
	import Skeleton from '$lib/components/Skeleton.svelte';

	let totalStaked: number;
	let apy: number;
	let stakersCount: number;

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

	onMount(() => {
		getLedgerBalanceStoreEntries();
	});

	$: if ($waterNeuronInfo) {
		apy = $waterNeuronInfo.apy();
		totalStaked = $waterNeuronInfo.neuron8yStake() + $waterNeuronInfo.neuron6mStake();
	}
</script>

<div class="stat-widget-container" in:fade={{ duration: 200 }}>
	<div class="stat-item">
		<b>TVL</b>
		<p>
			{#if totalStaked}
				{displayNumber(totalStaked, 0)} ICP
			{:else}
				<Skeleton width="5em" />
			{/if}
		</p>
	</div>
	<div class="stat-item">
		<b>APY</b>
		<p>
			{#if apy}
				{displayNumber(100 * apy, 1)}%
			{:else}
				<Skeleton width="3em" />
			{/if}
		</p>
	</div>
	<div class="stat-item">
		<b>Holders</b>
		<p>
			{#if stakersCount || stakersCount === 0}
				{stakersCount}
			{:else}
				<Skeleton width="3em" />
			{/if}
		</p>
	</div>
</div>

<style>
	b {
		font-weight: 600;
		font-size: 0.8em;
		letter-spacing: 0.02em;
		text-transform: uppercase;
		opacity: 0.6;
	}

	p {
		margin: 0;
		font-size: 1em;
		font-weight: 600;
	}

	/* === Layout === */
	.stat-widget-container {
		background: none;
		color: var(--stake-text-color);
		border: none;
		padding: 0.5em 1em;
		display: flex;
		flex-direction: row;
		gap: 2em;
		justify-content: center;
	}

	/* === Components === */
	.stat-item {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
		font-family: var(--secondary-font);
	}
</style>
