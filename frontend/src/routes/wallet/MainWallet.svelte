<script lang="ts">
	import CopyIcon from '$lib/icons/CopyIcon.svelte';
	import { scale } from 'svelte/transition';
	import { user } from '$lib/stores';
	import SendButton from './SendButton.svelte';

	let isAnimating = false;
	let circleVisible = false;
	let accountId = false;

	const handleAnimation = () => {
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
	};
</script>

<div class="address-container">
	<h2>ICP Account Id</h2>
	<div class="principal-container">
		<p title="accountIdentifier-hex" style:max-width="82%">{$user?.accountId ?? '-/-'}</p>
		<button
			class="copy-btn"
			on:click={() => {
				accountId = true;
				handleAnimation();
				navigator.clipboard.writeText($user ? $user.accountId : '');
			}}
		>
			<CopyIcon />
			{#if circleVisible && accountId}
				<div class="circle" transition:scale={{ duration: 500 }}></div>
			{/if}
		</button>
	</div>
	<SendButton asset="ICP" />
</div>
<div class="address-container">
	<h2>Principal Address</h2>
	<div class="principal-container">
		<p title="principal-user" style:max-width="80%">{$user?.principal ?? '-/-'}</p>
		<button
			class="copy-btn"
			on:click={() => {
				accountId = false;
				handleAnimation();
				navigator.clipboard.writeText($user ? $user.principal.toString() : '');
			}}
		>
			<CopyIcon />
			{#if circleVisible && !accountId}
				<div class="circle" transition:scale={{ duration: 500 }}></div>
			{/if}
		</button>
	</div>
	<SendButton asset={'nICP'} />
	<SendButton asset={'WTN'} />
</div>

<style>
	/* === Base Styles === */
	h2 {
		margin: 0;
		margin-top: 1em;
		font-family: var(--secondary-font);
	}

	p {
		font-family: var(--secondary-font);
		overflow-wrap: anywhere;
	}

	/* === Layout === */
	.principal-container {
		margin-left: 1em;
		display: flex;
		align-items: center;
	}

	.address-container {
		gap: 1em;
		display: flex;
		flex-direction: column;
	}

	/* === Components ==== */
	.copy-btn {
		background-color: transparent;
		border: none;
		cursor: pointer;
		transition: all 0.3s ease;
		display: flex;
		position: relative;
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
