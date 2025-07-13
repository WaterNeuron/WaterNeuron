<script lang="ts">
	import { sns, inSnsMenu, handleSnsChange } from '$lib/stores';
	import { isMobile } from '$lib';
	import snsMetadata from './sns_metadata.json';
	import ArrowIcon from '$lib/icons/ArrowIcon.svelte';
</script>

<div class="sns-selection-container">
	{#if !isMobile}
		<div class="sns-listing">
			{#each snsMetadata.metadata as data}
				<div class="sns-btn-container">
					<button
						class:selected-sns={$sns.name === data.name}
						class:default-sns={$sns.name !== data.name}
						on:click={() => {
							handleSnsChange(data.name, data.governance_id);
						}}>{data.name}</button
					>
				</div>
			{/each}
			<div class="sns-btn-container">
				<button
					class:selected-sns={$sns.name === 'Custom'}
					class:default-sns={$sns.name !== 'Custom'}
					on:click={() => {
						handleSnsChange();
					}}>{'Custom'}</button
				>
			</div>
		</div>
	{:else}
		<div class="select-container">
			<button on:click={() => inSnsMenu.set(true)}>
				<span>{$sns.name}</span>
				<div class="down-arrow">
					<ArrowIcon isUp={false} />
				</div>
			</button>
		</div>
	{/if}
</div>

<style>
	/* === Base Styles === */
	div::-webkit-scrollbar {
		width: 1em;
		position: absolute;
		top: 0;
		right: 0;
		width: 0.4em;
		height: 100%;
		background: transparent;
	}

	div::-webkit-scrollbar-track {
		background: transparent;
	}

	div::-webkit-scrollbar-thumb {
		background: #0a285063;
		border-radius: 0.5em;
	}

	div::-webkit-scrollbar-corner {
		background: transparent;
	}

	button {
		color: var(--stake-text-color);
		cursor: pointer;
		display: flex;
		width: 90%;
		justify-content: center;
		padding: 1em;
		font-size: 16px;
		border-radius: 8px;
	}

	button:hover {
		border: 2px solid transparent;
		background: var(--main-color-hover);
		transition: all 0.2s;
	}

	/* === Layout === */
	.sns-selection-container {
		display: flex;
		flex-direction: column;
		width: 20%;
		flex-grow: 1;
		align-items: center;
		gap: 1em;
		padding: 1em;
	}

	/* === Component === */
	.sns-listing {
		display: flex;
		overflow-y: scroll;
		width: 100%;
		height: 100%;
		flex-direction: column;
		margin: 1em;
		padding: 0;
		gap: 1em;
		position: relative;
	}

	.sns-btn-container {
		width: 100%;
		display: flex;
		justify-content: center;
		align-items: center;
	}

	.select-container {
		display: flex;
		width: 100%;
		justify-content: center;
		align-items: center;
	}

	/* === Utilities === */
	.selected-sns {
		border: 2px solid var(--main-color);
		background-color: var(--sns-selected-button-color);
	}

	.default-sns {
		border: 2px solid transparent;
		background: none;
	}

	@media (max-width: 767px) {
		.sns-selection-container {
			width: 100%;
			height: 5em;
			flex-direction: row;
			padding: 0;
		}

		.sns-listing {
			overflow-x: scroll;
			overflow-y: hidden;
			flex-direction: row;
			margin-left: 1em;
		}

		button {
			display: flex;
			width: 80%;
			justify-content: center;
			position: relative;
			padding: 1em;
			font-size: 16px;
			border-radius: 8px;
			border: 2px solid var(--main-color);
			background-color: var(--sns-selected-button-color);
		}

		.down-arrow {
			position: absolute;
			right: 1em;
		}
	}
</style>
