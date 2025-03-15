<script lang="ts">
	import { inSnsMenu, sns, handleSnsChange } from '$lib/stores';
	import snsMetadata from './sns_metadata.json';
</script>

<div class="background-menu">
	<div class="menu-container">
		<div class="header-container">
			<h1>Select SNS</h1>
			<button
				class="close-btn"
				on:click={() => {
					inSnsMenu.set(false);
				}}
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					width="32px"
					height="32px"
					viewBox="0 0 24 24"
					fill="none"
					stroke="var(--svg-fill-color)"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<line x1="18" y1="6" x2="6" y2="18"></line>
					<line x1="6" y1="6" x2="18" y2="18"></line>
				</svg>
			</button>
		</div>
		<div class="sns-listing">
			{#each snsMetadata.metadata as data}
				<div class="sns-btn-container">
					<button
						class="sns-btn-selection"
						class:selected-sns={$sns.name === data.name}
						on:click={() => {
							handleSnsChange(data.name, data.governance_id);
							inSnsMenu.set(false);
						}}>{data.name}</button
					>
				</div>
			{/each}
			<div class="sns-btn-container">
				<button
					class="sns-btn-selection"
					class:selected-sns={$sns.name === 'Custom'}
					on:click={() => {
						handleSnsChange();
						inSnsMenu.set(false);
					}}>{'Custom'}</button
				>
			</div>
		</div>
	</div>
</div>

<style>
	/* === Base Styles === */
	h1 {
		font-family: var(--main-font);
		color: var(--title-color);
	}
	/* === Layout === */
	.background-menu {
		height: fit-content;
		width: 100vw;
		background: var(--page-background);
		z-index: 1;
	}

	.menu-container {
		display: flex;
		height: fit-content;
		width: 100%;
		align-items: center;
		flex-direction: column;
	}

	.header-container {
		display: flex;
		justify-content: space-between;
		width: 90%;
	}

	.sns-btn-container {
		width: 100%;
		display: flex;
		justify-content: center;
		align-items: center;
	}

	/* === Component === */
	.close-btn {
		background: none;
		border: none;
	}
	.sns-listing {
		display: flex;
		width: 100%;
		height: fit-content;
		flex-direction: column;
		margin: 0;
		padding: 0;
		gap: 1em;
	}

	.sns-btn-selection {
		color: var(--title-color);
		cursor: pointer;
		display: flex;
		width: 80%;
		justify-content: center;
		padding: 1em;
		font-size: 16px;
		border-radius: 8px;
		border: 2px solid transparent;
		background: none;
	}

	/* === Utilities === */
	.selected-sns {
		border: 2px solid var(--main-color);
		background-color: var(--sns-selected-button-color);
	}
</style>
