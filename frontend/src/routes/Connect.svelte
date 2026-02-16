<script lang="ts">
	import { isBusy, availableAccounts, signer, toasts, isLogging } from '$lib/stores';
	import {
		connectWithInternetIdentity,
		connectWithTransport,
		testSignIn,
		localSignIn,
		finalizePlugConnection,
		connectWithExtension
	} from '$lib/authentification';
	import { fade } from 'svelte/transition';
	import { displayPrincipal, Toast as ToastMessage } from '$lib';
	import { onMount } from 'svelte';
	import CloseIcon from '$lib/icons/CloseIcon.svelte';
	import { Signer } from '@slide-computer/signer';
	import { Principal } from '@dfinity/principal';
	import Toast from './Toast.svelte';
	import { user } from '$lib/stores';
	import { DEV, NFID_RPC, OISY_RPC, STAGING } from '$lib/env';

	let dialog: HTMLDialogElement;

	async function handleConnection(wallet: 'internetIdentity' | 'plug' | 'oisy' | 'nfid') {
		if ($isBusy) return;
		isBusy.set(true);

		try {
			switch (wallet) {
				case 'internetIdentity':
					await connectWithInternetIdentity();
					break;
				case 'plug':
					await connectWithExtension();
					break;
				case 'oisy':
					await connectWithTransport(OISY_RPC);
					break;
				case 'nfid':
					await connectWithTransport(NFID_RPC);
					break;
			}
		} catch (e) {
			if (!$user) {
				toasts.add(ToastMessage.temporaryWarning('Connection failed. Please try again.'));
			}
			console.log(e);
			availableAccounts.set([]);
			signer.set(undefined);
			isBusy.set(false);
			return;
		}
		if ($availableAccounts.length === 0) {
			dialog.close();
		}
	}

	async function finalizeConnection(newSigner: Signer | undefined, userPrincipal: Principal) {
		if (!newSigner) {
			toasts.add(ToastMessage.temporaryWarning('Connection with wallet failed.'));
		} else {
			try {
				finalizePlugConnection(newSigner, userPrincipal);
			} catch (error) {
				console.log(error);
			}
		}
		dialog.close();
	}

	onMount(() => {
		dialog = document.getElementById('connectDialog') as HTMLDialogElement;
		dialog.showModal();
	});
</script>

<dialog
	id="connectDialog"
	in:fade={{ duration: 100 }}
	on:close={() => {
		isLogging.set(false);
		availableAccounts.set([]);
		signer.set(undefined);
		isBusy.set(false);
	}}
	on:click={(e) => {
		if (e.target === dialog) dialog.close();
	}}
>
	<div class="wallets-container">
		{#if $availableAccounts.length > 0}
			<div class="header-container">
				<h1>Select your account</h1>
				<button
					on:click={() => {
						dialog.close();
					}}
					class="close-btn"
				>
					<CloseIcon color="--title-color" />
				</button>
			</div>
			<div class="selection-container">
				{#each $availableAccounts as account}
					<button class="login-btn" on:click={() => finalizeConnection($signer, account.owner)}>
						<p>
							{displayPrincipal(account.owner)}
						</p>
					</button>
				{/each}
			</div>
		{:else}
			<div class="header-container">
				<h1>Connect Wallet</h1>
				<button
					on:click={() => {
						dialog.close();
					}}
					class="close-btn"
				>
					<CloseIcon color="--title-color" />
				</button>
			</div>
			<div class="selection-container">
				<button class="wallet-btn" on:click={() => handleConnection('internetIdentity')}>
					<img src="/icon/astronaut.webp" width="auto" height="28px" alt="Dfinity Astronaut." />
					<span>Internet Identity</span>
				</button>
				<button class="wallet-btn" on:click={() => handleConnection('nfid')}>
					<img src="/icon/nfid.webp" width="auto" height="22px" alt="NFID Logo." />
					<span>Google via NFID</span>
				</button>
				<button class="wallet-btn" on:click={() => handleConnection('plug')}>
					<img src="/icon/plug.png" width="auto" height="28px" alt="Plug Icon." />
					<span>Plug Wallet</span>
				</button>
				<button class="wallet-btn" on:click={() => handleConnection('oisy')}>
					<img src="/icon/oisy.webp" width="auto" height="28px" alt="Oisy Icon." />
					<span>Oisy Wallet</span>
				</button>
				{#if DEV || STAGING}
					<button
						class="wallet-btn dev-btn"
						on:click={async () => {
							if ($isBusy) return;
							try {
								await localSignIn();
								dialog.close();
							} catch (e) {
								toasts.add(ToastMessage.temporaryWarning('Connection failed. Please try again.'));
								console.error(e);
								availableAccounts.set([]);
								signer.set(undefined);
								isBusy.set(false);
							}
						}}
					>
						<span>Local Development</span>
					</button>
				{/if}
				{#if DEV}
					<button
						class="wallet-btn dev-btn"
						on:click={async () => {
							if ($isBusy) return;
							try {
								await testSignIn();
								dialog.close();
							} catch (e) {
								toasts.add(ToastMessage.temporaryWarning('Connection failed. Please try again.'));
								console.error(e);
								availableAccounts.set([]);
								signer.set(undefined);
								isBusy.set(false);
							}
						}}
						title="ii-connect-btn"
					>
						<span>Test Development</span>
					</button>
				{/if}
			</div>
		{/if}
	</div>
	{#if $isLogging}
		<Toast />
	{/if}
</dialog>

<style>
	/* === Base Styles === */
	h1 {
		font-family: var(--main-font);
		font-weight: 500;
		font-size: 16px;
		color: var(--title-color);
	}

	p {
		font-family: var(--secondary-font);
		color: var(--title-color);
	}

	::backdrop {
		backdrop-filter: blur(5px);
	}

	dialog {
		height: fit-content;
		display: flex;
		justify-content: center;
		flex-wrap: wrap;
		gap: 1em;
		border: none;
		background: none;
		margin: auto;
	}

	/* === Layout === */
	.wallets-container {
		display: flex;
		flex-direction: column;
		height: fit-content;
		max-width: 22em;
		width: 80vw;
		gap: 1.25em;
		background: var(--background-color);
		padding: 1.5em;
		border: var(--main-container-border);
		border-radius: 15px;
	}

	.header-container {
		width: 100%;
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0 0.25em;
	}

	.selection-container {
		display: flex;
		flex-direction: column;
		gap: 0.6em;
	}

	/* === Components === */
	.wallet-btn {
		border-radius: 10px;
		border: var(--main-container-border);
		width: 100%;
		align-items: center;
		cursor: pointer;
		display: flex;
		gap: 0.75em;
		background: var(--background-color);
		padding: 0.85em 1em;
		transition: background 0.15s;
	}

	.wallet-btn:hover {
		background: var(--main-color);
		transition: background 0.15s;
	}

	.wallet-btn span {
		font-family: var(--secondary-font);
		font-size: 0.95em;
		font-weight: 500;
		color: var(--title-color);
	}

	.wallet-btn:hover span {
		color: var(--main-button-text-color);
	}

	.wallet-btn img {
		width: 28px;
		height: 28px;
		object-fit: contain;
	}

	.dev-btn {
		border-color: #e04040;
		opacity: 0.7;
	}

	.dev-btn:hover {
		background: #e04040;
	}

	.close-btn {
		border: none;
		background: none;
		cursor: pointer;
		width: 20px;
		height: 20px;
	}
</style>
