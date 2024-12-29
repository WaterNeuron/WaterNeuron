<script lang="ts">
	import { isLogging, isBusy, availableAccounts, signer, toasts } from '$lib/stores';
	import {
		DEV,
		STAGING,
		connectWithInternetIdentity,
		connectWithTransport,
		connectWithPlug,
		testSignIn,
		localSignIn,
		NFID_RPC,
		//OISY_RPC,
		finalizePlugConnection
	} from '$lib/authentification';
	import { fade } from 'svelte/transition';
	import { displayPrincipal } from '$lib';
	import { onMount } from 'svelte';
	import CloseIcon from '$lib/icons/CloseIcon.svelte';
	import { Signer } from '@slide-computer/signer';
	import { Principal } from '@dfinity/principal';
	import { Toast } from '$lib/toast';

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
					await connectWithPlug();
					break;
				// case 'oisy':
				// 	await connectWithTransport(OISY_RPC);
				// 	break;
				case 'nfid':
					await connectWithTransport(NFID_RPC);
					break;
			}
		} catch (e) {
			toasts.add(Toast.error('Connection failed. Please try again.'));
			console.error(e);
			dialog.close();
			return;
		}

		if ($availableAccounts.length === 0) {
			dialog.close();
		}
	}

	async function finalizeConnection(newSigner: Signer | undefined, userPrincipal: Principal) {
		if (!newSigner) {
			toasts.add(Toast.error('Connection with wallet failed.'));
		} else {
			try {
				await finalizePlugConnection(newSigner, userPrincipal);
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
	in:fade={{ duration: 500 }}
	on:close={() => {
		isLogging.set(false);
		availableAccounts.set([]);
		signer.set(undefined);
		isBusy.set(false);
	}}
>
	<div class="wallets-container" in:fade={{ duration: 500 }}>
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
					<img src="/icon/astronaut.webp" width="40em" height="40em" alt="Dfinity Astronaut." />
					<h2>Internet Identity</h2>
				</button>
				<button class="login-btn" on:click={() => handleConnection('nfid')}>
					<img src="/icon/google.svg" width="auto" height="40em" alt="Google Logo." />
					<h2>Google</h2>
					<span>|</span>
					<img src="/icon/nfid.webp" width="auto" height="30em" alt="NFID Logo." />
				</button>
				<button class="login-btn" on:click={() => handleConnection('plug')}>
					<img src="/icon/plug.png" width="40em" height="40em" alt="Plug Icon." />
					<h2>Plug Wallet</h2>
				</button>
				{#if DEV || STAGING}
					<button
						class="login-btn"
						style:background-color="red"
						on:click={async () => {
							if ($isBusy) return;
							await localSignIn();
							dialog.close();
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
							await testSignIn();
							dialog.close();
						}}
						title="ii-connect-btn"
					>
						<h2>Test Development</h2>
					</button>
				{/if}
			</div>
		{/if}
	</div>
</dialog>

<style>
	/* === Base Styles === */
	h1 {
		font-family: var(--secondary-font);
		font-weight: 600;
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
	}

	span {
		color: var(--main-button-text-color);
		font-family: var(--secondary-font);
	}

	::backdrop {
		backdrop-filter: blur(5px);
	}

	dialog {
		height: fit-content;
		display: flex;
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
		max-width: 35em;
		width: 80vw;
		gap: 1em;
		background: var(--background-color);
		padding: 2em;
		border-radius: 15px;
		border: var(--input-border);
	}

	.header-container {
		width: 100%;
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.selection-container {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 1em;
		padding: 1em;
	}

	/* === Components === */
	.login-btn {
		gap: 0.3em;
		border-radius: 8px;
		border: 2px solid black;
		box-shadow: 3px 3px 0 0 black;
		width: auto;
		height: 5em;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		display: flex;
		background: var(--main-color);
	}

	.login-btn:hover {
		transform: scale(0.95);
		transition: all 0.3s;
		box-shadow: 6px 6px 0 0 black;
	}

	.close-btn {
		border: none;
		background: none;
		cursor: pointer;
	}

	@media (max-width: 767px) {
		.selection-container {
			display: flex;
			flex-direction: column;
			gap: 1em;
		}
	}
</style>
