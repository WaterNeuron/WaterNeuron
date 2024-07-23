<script>
	import { ToastType } from '$lib/toast';
	import { toasts } from '$lib/stores';
	import { fade } from 'svelte/transition';
	import { flip } from 'svelte/animate';
	import ErrorIcon from '$lib/icons/ErrorIcon.svelte';
	import CloseIcon from '$lib/icons/CloseIcon.svelte';
	import SuccessIcon from '$lib/icons/SuccessIcon.svelte';
</script>

<div class="toasts-container">
	{#each $toasts as toast (toast.id)}
		<div class="toast-container" animate:flip transition:fade>
			<div class="info-container">
				<div class="info-icon">
					{#if toast.type == ToastType.Success}
						<SuccessIcon />
					{:else}
						<ErrorIcon />
					{/if}
				</div>
				<p>{@html toast.message}</p>
			</div>
			<button
				class="toast-close"
				on:click={() => {
					toasts.remove(toast.id);
				}}
			>
				<CloseIcon />
			</button>
		</div>
	{/each}
</div>

<style>
	/* === Base Styles === */
	p {
		font-weight: lighter;
		font-family: sans-serif;
	}

	/* === Layout === */
	.toasts-container {
		position: fixed;
		display: flex;
		flex-direction: column-reverse;
		align-items: center;
		justify-content: center;
		bottom: 0;
		width: 100%;
		gap: 0.5em;
		margin-bottom: 1em;
	}

	.toast-container {
		background-color: var(--main-color);
		box-shadow: 8px 8px 16px 0 rgba(0, 0, 0, 0.25);
		display: flex;
		border-radius: 8px;
		align-items: center;
		justify-content: space-between;
		width: 30em;
		max-width: 90vw;
		padding: 0 1%;
	}

	.info-container {
		display: flex;
		align-items: center;
		gap: 1em;
		margin-right: 1em;
	}

	.toast-close {
		border: 0;
		outline: 0;
		background: transparent;
		margin: 0;
		cursor: pointer;
		padding: 0 var(--padding-0_5x);
	}

	/* === Components === */
	.info-icon {
		display: flex;
		align-items: center;
	}
</style>
