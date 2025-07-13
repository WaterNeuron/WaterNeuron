<script lang="ts">
	import { inQrDestination } from '$lib/stores';
	import CopyIcon from '$lib/icons/CopyIcon.svelte';
	import { fade, scale } from 'svelte/transition';
	import { onMount } from 'svelte';
	import QrCreator from 'qr-creator';

	export let destination: string;
	let dialog: HTMLDialogElement;

	let isAnimating = false;
	let circleVisible = false;

	function handleAnimation() {
		if (!isAnimating) {
			isAnimating = true;
			circleVisible = true;
			setTimeout(() => {
				circleVisible = false;
				setTimeout(() => {
					isAnimating = false;
				}, 500);
			}, 500);
		}
	}

	onMount(() => {
		dialog = document.getElementById('receiverDialog') as HTMLDialogElement;
		dialog.showModal();

		QrCreator.render(
			{
				text: `${destination ?? ''}`,
				radius: 0.0, // 0.0 to 0.5
				ecLevel: 'H', // L, M, Q, H
				fill: 'white',
				background: null,
				size: 1000 // in pixels
			},
			document.querySelector('#qr-code') as HTMLElement
		);
	});
</script>

<dialog
	id="receiverDialog"
	on:close={() => {
		inQrDestination.set(false);
	}}
>
	<div class="receive-container" transition:fade={{ duration: 100 }}>
		<div class="header-container">
			<h3>Send ICP to the following destination</h3>
		</div>
		<div class="qr-code-container">
			<canvas id="qr-code" />
		</div>
		<div class="principal-container">
			<p>{destination}</p>
			<button
				class="copy-btn"
				on:click={() => {
					handleAnimation();
					navigator.clipboard.writeText(destination ?? '');
				}}
			>
				<CopyIcon />
				{#if circleVisible}
					<div class="circle" transition:scale={{ duration: 200 }}></div>
				{/if}
			</button>
		</div>
		<div class="finish-container">
			<button
				class="finish-btn"
				on:click={() => {
					dialog.close();
				}}
			>
				<span>Finish</span>
			</button>
		</div>
	</div>
</dialog>

<style>
	/* === Base Styles === */
	::backdrop {
		backdrop-filter: blur(10px);
	}

	dialog {
		display: flex;
		justify-content: center;
		align-items: center;
		background: none;
		height: fit-content;
		min-height: 100%;
		min-width: 100dvw;
		border: none;
		padding: 0;
		margin: 0;
		border: none;
	}

	span {
		font-family: var(--secondary-font);
		display: flex;
		align-items: center;
	}

	button {
		color: var(--main-button-text-color);
	}

	p {
		font-family: var(--secondary-font);
		overflow-wrap: anywhere;
		font-size: 14px;
	}

	h3 {
		font-family: var(--secondary-font);
		font-size: 16px;
		color: var(--title-color);
	}

	canvas {
		background: var(--qr-code-background);
		padding: 1em;
		border-radius: 8px;
	}
	/* === Layout === */
	.receive-container {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		max-width: 90dvw;
		width: 35em;
		background: var(--background-color);
		color: var(--title-color);
		padding: 2em;
		margin: 0.3em;
		border-radius: 15px;
		border: var(--input-border);
		gap: 1em;
		height: fit-content;
		overflow-x: hidden;
	}

	.header-container {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0 2%;
		width: 100%;
		font-family: var(--secondary-font);
	}

	.principal-container {
		display: flex;
		width: 100%;
		align-items: center;
	}

	.qr-code-container {
		position: relative;
		width: 100%;
		display: flex;
		justify-content: center;
	}

	.finish-container {
		display: flex;
		justify-content: end;
		width: 100%;
		padding-right: 1em;
	}

	/* === Components === */

	.finish-btn {
		background: var(--main-color);
		min-width: 80px;
		position: relative;
		border: 2px solid black;
		border-radius: 8px;
		font-size: 14px;
		padding: 0 1em 0 1em;
		width: 10em;
		height: 3em;
		font-weight: bold;
		display: flex;
		justify-content: center;
		align-items: center;
	}

	.finish-btn:hover {
		background: var(--main-color-hover);
		transition: all 0.2s;
	}

	#qr-code {
		height: 268px;
		width: 268px;
	}

	.copy-btn {
		background-color: transparent;
		border: none;
		cursor: pointer;
		transition: all 0.3s ease;
		display: flex;
		position: relative;
		width: 40px;
		height: fit-content;
	}

	/* === Animation === */

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
</style>
