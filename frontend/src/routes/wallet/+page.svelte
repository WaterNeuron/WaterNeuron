<script lang="ts">
	import { goto } from '$app/navigation';
	import { user, sendAsset } from '$lib/stores';

	if (!$user || !$sendAsset) goto('/stake');

	import Withdrawals from './Withdrawals.svelte';
	import { Asset } from '$lib';
	import SendButton from './SendButton.svelte';
	import { scale } from 'svelte/transition';
	import CopyIcon from '$lib/icons/CopyIcon.svelte';
	import { fade } from 'svelte/transition';

	let isAnimating = false;
	let circleVisible = false;
	let accountId = false;

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
</script>

<div class="wallet-menu-container" in:fade={{ duration: 500 }}>
	<h1>Wallet</h1>
	<div class="address-container">
		<h2>ICP Account Id</h2>
		<div class="principal-container">
			<p>{$user?.accountId}</p>
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
		<SendButton asset={Asset.fromText('ICP')} />
	</div>
	<div class="address-container">
		<h2>Principal Address</h2>
		<div class="principal-container">
			<p>{$user?.principal}</p>
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
		<SendButton asset={Asset.fromText('nICP')} />
		<SendButton asset={Asset.fromText('WTN')} />
	</div>
</div>
<Withdrawals />

<style>
	/* === Base Styles === */
	h1 {
		text-align: center;
		margin: 0;
		font-family: var(--font-type2);
	}

	h2 {
		margin: 0;
		margin-top: 1em;
		font-family: var(--font-type2);
	}

	p {
		font-family: var(--font-type2);
		overflow-wrap: anywhere;
	}

	/* === Layout === */
	.wallet-menu-container {
		background-color: #0c2c4c;
		border: 2px solid #66adff;
		border-radius: 10px;
		color: white;
		padding: 2em;
		display: flex;
		flex-direction: column;
		width: 44em;
		max-width: 80vw;
	}

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
		border-radius: 0.3em;
		transition: all 0.3s ease;
		color: white;
		font-weight: bold;
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
