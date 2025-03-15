<script lang="ts">
	import { goto } from '$app/navigation';
	import { user, ledgerDevice, toasts } from '$lib/stores';
	import { isMobile } from '$lib';

	if (!$user) goto('/');

	import Withdrawals from './Withdrawals.svelte';
	import LedgerWallet from './LedgerWallet.svelte';
	import MainWallet from './MainWallet.svelte';
	import { fade } from 'svelte/transition';
	import { connectWithHardwareWallet } from '$lib/authentification';
	import { Toast } from '$lib/toast';
	import SuccessIcon from '$lib/icons/SuccessIcon.svelte';

	let inMainWallet = true;

	async function handleWalletSelection(isMain: boolean) {
		if (!$user) return;

		try {
			if (!$ledgerDevice && !isMain) {
				await connectWithHardwareWallet();
			}
			inMainWallet = isMain;
			$user.account = inMainWallet ? 'main' : 'ledger';
		} catch (e) {
			console.error(e);
			toasts.add(Toast.error('Ledger device not found.'));
		}
	}
</script>

<div class="wallet-menu-container" in:fade={{ duration: 500 }}>
	{#key inMainWallet}
		<div class="header-container">
			<h1 style:align-self="center">Wallet</h1>
			{#if !isMobile}
				<div class="third-column-container">
					<div class="switch-container">
						<div class="btn-active-container">
							<button on:click={() => handleWalletSelection(true)} title="switch-main-btn">
								<p>Main</p>
							</button>
							{#if inMainWallet}
								<SuccessIcon color="--title-color" />
							{/if}
						</div>
						<div class="btn-active-container">
							<button on:click={() => handleWalletSelection(false)} title="switch-ledger-btn">
								<p>Ledger</p>
							</button>
							{#if !inMainWallet}
								<SuccessIcon color="--title-color" />
							{/if}
						</div>
					</div>
				</div>
			{/if}
		</div>
		{#if inMainWallet}
			<MainWallet />
		{:else}
			<LedgerWallet />
		{/if}
	{/key}
</div>
<Withdrawals />

<style>
	/* === Base Styles === */
	h1 {
		margin: 0;
		font-family: var(--secondary-font);
		grid-column: 2;
		text-align: center;
		gap: 0.5em;
	}

	p {
		display: flex;
		align-items: center;
		gap: 0.5em;
		margin: 0.4em;
	}

	/* === Layout === */
	.wallet-menu-container {
		background-color: var(--background-color);
		border: var(--input-border);
		border-radius: 10px;
		color: var(--stake-text-color);
		padding: 2em;
		display: flex;
		flex-direction: column;
		width: 44em;
		max-width: 80vw;
	}

	.header-container {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
	}

	.switch-container {
		background: var(--switch-background-color);
		border-radius: 8px;
		padding: 0.2em 0.4em;
		width: 5em;
	}

	.btn-active-container {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.third-column-container {
		display: flex;
		justify-content: end;
		grid-column: 3;
	}

	/* === Components ==== */
	.header-container button {
		display: flex;
		justify-content: center;
		align-items: center;
		border: none;
		background: none;
		color: var(--title-color);
		font-weight: bold;
		cursor: pointer;
		padding: 0;
	}
</style>
