<script lang="ts">
	import StatsWidget from './stake/StatsWidget.svelte';
	import Stake from './stake/Stake.svelte';
	import Unstake from './stake/Unstake.svelte';
	import Faq from './Faq.svelte';
	import { inputAmount } from '$lib/stores';
	import { fade } from 'svelte/transition';

	let stake = true;
</script>

<StatsWidget />
<div class="main-container" in:fade={{ duration: 200 }}>
	<div class="header-container">
		<button
			class="header-btn"
			title="stake-header"
			on:click={() => {
				inputAmount.reset();
				stake = true;
			}}
			class:selected={stake}
			class:not-selected={!stake}
		>
			<h2>Stake ICP</h2>
		</button>
		<button
			class="header-btn"
			title="unstake-header"
			on:click={() => {
				inputAmount.reset();
				stake = false;
			}}
			class:selected={!stake}
			class:not-selected={stake}
		>
			<h2>Unstake nICP</h2>
		</button>
	</div>
	{#if stake}
		<Stake />
	{:else}
		<Unstake />
	{/if}
</div>
<Faq />

<style>
	h2 {
		font-weight: 800;
		font-family: var(--main-font);
		letter-spacing: -0.08em;
		font-size: 1.25em;
		margin: 0;
	}
	/* === Layout === */
	.main-container {
		display: flex;
		justify-content: start;
		flex-direction: column;
		width: 30em;
		max-width: 95vw;
		border: var(--main-container-border);
		border-radius: 12px;
		overflow: hidden;
	}

	.header-container {
		display: flex;
	}
	/* === Components === */
	.header-btn {
		border: none;
		color: var(--stake-text-color);
		width: 100%;
		height: 3.5em;
		padding: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		transition:
			background-color 0.15s,
			color 0.15s;
	}

	@media (max-width: 767px) {
		.header-btn {
			padding: 0 1em;
		}
	}

	/* === Utilities === */
	.selected {
		background-color: var(--background-color);
	}

	.not-selected {
		background-color: var(--switch-background-color);
		color: var(--text-color);
		cursor: pointer;
	}

	.not-selected:hover {
		background-color: var(--background-color-transparent);
	}
</style>
