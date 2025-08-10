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
				<button class="login-btn" on:click={() => handleConnection('internetIdentity')}>
					<img src="/icon/astronaut.webp" width="auto" height="40px" alt="Dfinity Astronaut." />
					<h2>Internet Identity</h2>
				</button>
				<button class="login-btn" on:click={() => handleConnection('nfid')}>
					<h2>Google via NFID</h2>
					<img src="/icon/nfid.webp" width="auto" height="30em" alt="NFID Logo." />
				</button>
				<button class="login-btn" on:click={() => handleConnection('plug')}>
					<img src="/icon/plug.png" width="auto" height="40px" alt="Plug Icon." />
					<h2>Plug Wallet</h2>
				</button>
				<button class="login-btn" on:click={() => handleConnection('oisy')}>
					<img src="/icon/oisy.webp" width="auto" height="40px" alt="Oisy Icon." />
					<h2>Oisy Wallet</h2>
				</button>
				{#if DEV || STAGING}
					<button
						class="login-btn"
						style:background-color="red"
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
						<h2>Local Development</h2>
					</button>
				{/if}
				{#if DEV}
					<button
						class="login-btn"
						style:background-color="red"
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
						<h2>Test Development</h2>
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
		font-size: 20px;
		color: var(--title-color);
	}

	p {
		font-family: var(--secondary-font);
		color: var(--main-button-text-color);
	}

	h2 {
		color: var(--main-button-text-color);
		font-family: var(--secondary-font);
		font-size: 1.2em;
		font-weight: 500;
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
	}

	/* === Layout === */
	.wallets-container {
		display: flex;
		flex-direction: column;
		height: fit-content;
		max-width: 20em;
		width: 80vw;
		gap: 1em;
		background: var(--background-color);
		padding: 1em;
		border: var(--main-container-border);
		border-radius: 15px;
	}

	.header-container {
		width: 100%;
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.selection-container {
		display: flex;
		flex-direction: column;
		gap: 1em;
	}

	/* === Components === */
	.login-btn {
		gap: 0.3em;
		border-radius: 8px;
		border: var(--main-container-border);
		width: auto;
		height: 5em;
		align-items: center;
		cursor: pointer;
		display: flex;
		background: var(--main-color);
		position: relative;
		padding: 0 20px;
	}

	.login-btn:hover {
		background: var(--main-color-hover);
		transition: all 0.2s;
	}

	.login-btn img {
		position: absolute;
		right: 20px;
	}

	.close-btn {
		border: none;
		background: none;
		cursor: pointer;
	}
</style>
