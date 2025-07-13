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
	import { WaterNeuronInfo, fetchBalance, fetchWtnAllocation } from '$lib/actors';
	import { tryConnectOnReload } from '$lib/authentification';
	import Toast from './Toast.svelte';

	async function updateBalances() {
		if ($canisters && $user) {
			$user.icpBalanceE8s = await fetchBalance(
				$canisters.icpLedger.anonymousActor,
				$user.principal
			);
			$user.nicpBalanceE8s = await fetchBalance(
				$canisters.nicpLedger.anonymousActor,
				$user.principal
			);
			$user.wtnBalanceE8s = await fetchBalance(
				$canisters.wtnLedger.anonymousActor,
				$user.principal
			);

			$user.wtnAllocationE8s =
				(await fetchWtnAllocation($user.principal, $canisters.waterNeuron.anonymousActor)) ?? 0n;

			if ($ledgerDevice) {
				$ledgerDevice.icpBalanceE8s = await fetchBalance(
					$canisters.icpLedger.anonymousActor,
					$ledgerDevice.principal
				);
				$ledgerDevice.nicpBalanceE8s = await fetchBalance(
					$canisters.nicpLedger.anonymousActor,
					$ledgerDevice.principal
				);
				$ledgerDevice.wtnBalanceE8s = await fetchBalance(
					$canisters.wtnLedger.anonymousActor,
					$ledgerDevice.principal
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
		<div class="content-container" class:filter={$inReceivingMenu || $inSendingMenu || $isLogging}>
			<slot />
		</div>
		<Footer />
		{#if !$inSendingMenu && !$isLogging}
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

		--main-color: #286e5f;
		--main-color-hover: #308171;
		--main-color-light: #286e5f5e;
		--main-color-disabled: #286e5f;
		--main-button-text-color: white;
		--main-button-text-color-disabled: #fefefeb5;
	}

	:root[theme='light'] {
		--important-text-color: black;
		--stake-text-color: black;
		--box-shadow: #7c7c7c63 0px 8px 16px;

		--title-color: black;
		--border-color: #454545;
		--page-background: rgb(241, 241, 243);
		--background-color: #fcfffd;
		--background-color-transparent: #f0f0f0de;
		--switch-background-color: rgba(220, 220, 220, 0.49);

		--qr-code-background: #283e95;

		--input-color: #fcfffd;

		--text-color: rgb(127 127 127);
		--faq-color: black;

		--unstake-selection-color: #283e9521;

		--svg-fill-color: #a7a7a7;
		--svg-opposite-color: #b3b3b3;
		--sns-selected-button-color: rgb(107 180 249 / 50%);

		--theme-background-asset-logo: none;
		--theme-border-asset-logo: none;

		--main-container-border: 1px solid rgb(193 193 193);
		--select-unstake-speed: 1px solid black;
	}

	:root[theme='dark'] {
		--svg-fill-color: #a7a7a7;
		--svg-opposite-color: #7f7f7f;
		--stake-text-color: white;
		--qr-code-background: none;
		--box-shadow: none;

		--main-button-text-color: #fcfffd;
		--title-color: white;

		--border-color: rgb(158 163 178);
		--background-color: rgb(43, 51, 67);
		--background-color-transparent: #252c38;
		--switch-background-color: rgba(34, 38, 47, 0.6);

		--input-color: rgb(39, 46, 60);
		--text-color: rgb(181 181 181);

		--unstake-selection-color: #a9bbff54;

		--faq-color: white;

		--page-background: radial-gradient(farthest-corner circle at 0% 100%, #090a0d, #272f3d);
		--sns-selected-button-color: #404f9987;

		--theme-background-asset-logo: #dadef2;
		--theme-border-asset-logo: 2px solid #a3a5b0;

		--main-container-border: 1px solid rgba(235, 235, 239, 0.08);
		--select-unstake-speed: 1px solid white;
	}

	@font-face {
		font-family: 'Akrobat-black';
		src: url('/FliegeMonoVF.ttf') format('truetype');
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

	.content-container {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: start;
		height: fit-content;
		min-height: 45vh;
		flex-grow: 1;
		width: 100%;
		gap: 2em;
		padding-top: 2em;
		margin-bottom: 5dvh;
		color: white;
	}

	/* === Utilities ===*/
	.filter {
		filter: blur(5px);
	}
</style>
