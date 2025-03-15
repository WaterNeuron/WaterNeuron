<script lang="ts">
	import Navbar from './Navbar.svelte';
	import Footer from './Footer.svelte';
	import Connect from './Connect.svelte';
	import Send from './wallet/Send.svelte';
	import Menu from './Menu.svelte';
	import SnsMenu from './sns/SnsMenu.svelte';
	import Receive from './wallet/Receive.svelte';
	import CancelWarning from './wallet/CancelWarning.svelte';
	import {
		isLogging,
		inMobileMenu,
		inSendingMenu,
		inReceivingMenu,
		inCancelWarningMenu,
		inSnsMenu,
		user,
		canisters,
		waterNeuronInfo,
		handleSnsChange,
		ledgerDevice
	} from '$lib/stores';
	import { onMount } from 'svelte';
	import {
		WaterNeuronInfo,
		fetchIcpBalance,
		fetchNicpBalance,
		fetchWtnBalance,
		fetchWtnAllocation
	} from '$lib/state';
	import { tryConnectOnReload } from '$lib/authentification';
	import Toast from './Toast.svelte';

	async function updateBalances() {
		if ($canisters && $user) {
			$user.icpBalanceE8s = await fetchIcpBalance(
				$user.principal,
				$canisters.icpLedger.anonymousActor
			);
			$user.nicpBalanceE8s = await fetchNicpBalance(
				$user.principal,
				$canisters.nicpLedger.anonymousActor
			);
			$user.wtnBalanceE8s = await fetchWtnBalance(
				$user.principal,
				$canisters.wtnLedger.anonymousActor
			);

			$user.wtnAllocationE8s =
				(await fetchWtnAllocation($user.principal, $canisters.waterNeuron.anonymousActor)) ?? 0n;

			if ($ledgerDevice) {
				$ledgerDevice.icpBalanceE8s = await fetchIcpBalance(
					$ledgerDevice.principal,
					$canisters.icpLedger.anonymousActor
				);
				$ledgerDevice.nicpBalanceE8s = await fetchNicpBalance(
					$ledgerDevice.principal,
					$canisters.nicpLedger.anonymousActor
				);
				$ledgerDevice.wtnBalanceE8s = await fetchWtnBalance(
					$ledgerDevice.principal,
					$canisters.wtnLedger.anonymousActor
				);
			}
		}
	}

	async function updateWaterNeuronInfo() {
		if ($canisters) {
			waterNeuronInfo.set(
				new WaterNeuronInfo(await $canisters.waterNeuron.anonymousActor.get_info())
			);
		}
	}

	onMount(() => {
		tryConnectOnReload().then(() => {
			updateBalances();
			updateWaterNeuronInfo();
			handleSnsChange('BOOM DAO', 'xomae-vyaaa-aaaaq-aabhq-cai');
		});

		const intervalId = setInterval(async () => {
			await updateBalances();
			await updateWaterNeuronInfo();
		}, 5000);

		return () => clearInterval(intervalId);
	});
</script>

{#if $isLogging}
	<Connect />
{:else if $inSendingMenu}
	<Send />
{:else if $inReceivingMenu}
	<Receive />
{:else if $inCancelWarningMenu}
	<CancelWarning />
{/if}
{#if $inMobileMenu}
	<Menu />
{:else if $inSnsMenu}
	<SnsMenu />
{:else}
	<div class="page-container">
		<Navbar />
		<div class="redirect-container">
			<p><a href="/launchpad">Checkout Papaya SNS 🥭</a></p>
		</div>
		<div class="content-container" class:filter={$inReceivingMenu || $inSendingMenu || $isLogging}>
			<slot />
		</div>
		<Footer />
		{#if !$inSendingMenu}
			<Toast />
		{/if}
	</div>
{/if}

<style>
	/* === Variables === */
	:root {
		--unselected-header-color: #5d6b77;
		--unselected-header-text-color: #c7c7c7;
		--toast-text-color: var(--main-button-text-color);
		--input-border: 1px solid var(--border-color);

		--border-size: 1px;

		--main-font: 'Akrobat-black';
		--secondary-font: Arial;

		--padding: 8px;
		--padding-0_25x: calc(var(--padding) / 4);
		--padding-0_5x: calc(var(--padding) / 2);
		--padding-1_5x: calc(1.5 * var(--padding));
		--padding-2x: calc(2 * var(--padding));
		--padding-3x: calc(3 * var(--padding));
		--padding-4x: calc(4 * var(--padding));
		--padding-6x: calc(6 * var(--padding));
		--padding-8x: calc(8 * var(--padding));
		--card-background-contrast: var(--main-color);
		--card-background: white;

		--main-color: #4c66dc;
		--main-color-disabled: #4c66dcbd;
		--main-button-text-color: white;
		--main-button-text-color-disabled: #fefefeb5;
	}

	:root[theme='light'] {
		--important-text-color: black;
		--stake-text-color: black;

		--title-color: black;
		--border-color: #454545;
		--page-background: #fcfffd;
		--background-color: #fcfffd;
		--background-color-transparent: #fefefede;
		--switch-background-color: rgba(220, 220, 220, 0.49);

		--qr-code-background: #283e95;

		--input-color: #fcfffd;

		--text-color: rgb(127 127 127);
		--faq-color: black;

		--unstake-selection-color: #283e9521;

		--svg-fill-color: #000000;
		--svg-opposite-color: #b3b3b3;
		--sns-selected-button-color: rgb(107 180 249 / 50%);
	}

	:root[theme='dark'] {
		--svg-fill-color: #ffffff;
		--svg-opposite-color: #7f7f7f;
		--stake-text-color: white;
		--qr-code-background: none;

		--main-button-text-color: #fcfffd;
		--title-color: white;

		--border-color: rgb(158 163 178);
		--background-color: rgb(43, 51, 67);
		--background-color-transparent: rgb(43, 51, 67, 0.9);
		--switch-background-color: rgba(34, 38, 47, 0.6);

		--input-color: rgb(39, 46, 60);
		--text-color: rgb(181 181 181);

		--unstake-selection-color: #a9bbff54;

		--faq-color: white;

		--page-background: radial-gradient(farthest-corner circle at 0% 100%, #090a0d, #272f3d);
		--sns-selected-button-color: #404f9987;
	}

	@font-face {
		font-family: 'Akrobat-black';
		src: url('/Akrobat-Black.ttf') format('truetype');
		font-weight: normal;
		font-style: normal;
	}

	/* === Layout === */
	.page-container {
		display: flex;
		flex-direction: column;
		height: fit-content;
		min-height: 100%;
		width: 100vw;
		background: var(--page-background);
	}

	.page-container::-webkit-scrollbar {
		width: 12px; /* Width of the scrollbar */
		background: radial-gradient(
			farthest-corner circle at 0% 0%,
			rgb(18, 69, 89),
			#0f0f4d
		); /* Match the background gradient */
	}

	.page-container::-webkit-scrollbar-track {
		background: radial-gradient(
			farthest-corner circle at 0% 0%,
			rgb(18, 69, 89),
			#0f0f4d
		); /* Match the background gradient */
	}

	.page-container::-webkit-scrollbar-thumb {
		background-color: rgba(255, 255, 255, 0.5); /* Thumb color with transparency */
		border-radius: 6px; /* Rounded corners for the thumb */
		background-clip: padding-box;
	}

	.page-container::-webkit-scrollbar-corner {
		background: radial-gradient(
			farthest-corner circle at 0% 0%,
			rgb(18, 69, 89),
			#0f0f4d
		); /* Match the background gradient */
	}

	.redirect-container {
		display: flex;
		color: var(--title-color);
		align-items: center;
		font-family: var(--secondary-font);
		background: linear-gradient(135deg, #ffdab9, #ffb347);
		width: fit-content;
		padding: 1em;
		height: fit-content;
		border-radius: 10px;
		display: flex;
		align-self: center;
	}

	.redirect-container p {
		margin: 0;
		text-decoration: underline;
		text-decoration-color: black;
		font-size: 1.2em;
	}

	.redirect-container a {
		color: black;
	}

	.content-container {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: start;
		height: fit-content;
		min-height: 45vh;
		flex-grow: 1;
		width: 100%;
		gap: 3em;
		padding-top: 2em;
		margin-bottom: 4em;
		color: white;
	}

	/* === Utilities ===*/
	.filter {
		filter: blur(5px);
	}
</style>
