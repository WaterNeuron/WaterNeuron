<script lang="ts">
	import Navbar from './Navbar.svelte';
	import Footer from './Footer.svelte';
	import Connect from './Connect.svelte';
	import Send from './wallet/Send.svelte';
	import Menu from './Menu.svelte';
	import { isLogging, menu, isSelecting, user, state } from '$lib/stores';
	import { onMount } from 'svelte';
	import { fetchBalances, User } from '$lib/state';
	import Toast from './Toast.svelte';
	import { signIn } from '$lib/state';
	import { initializeState } from '$lib/stores';

	async function updateBalances() {
		if ($state && $user) {
			const { icp, nicp, wtn } = await fetchBalances(
				$user.principal,
				$state.nicpLedger,
				$state.wtnLedger,
				$state.icpLedger
			);

			$user.icpBalanceE8s = icp;
			$user.nicpBalanceE8s = nicp;
			$user.wtnBalanceE8s = wtn;
		}
	}

	onMount(() => {
		initializeState();
		signIn('reload').then(() => {
			updateBalances();
		});

		const intervalId = setInterval(async () => {
			if ($user && $state) {
				await updateBalances();
			}
		}, 5000);

		return () => clearInterval(intervalId);
	});
</script>

{#if $isLogging}
	<div class="background-filter">
		<Connect />
	</div>
{:else if $isSelecting}
	<div class="background-filter">
		<Send />
	</div>
{/if}
{#if $menu}
	<Menu />
{:else}
	<div class="page-container">
		<Navbar />
		<div class="content-container" class:filter={$isSelecting || $isLogging}>
			<slot />
		</div>
		<Footer />
		<Toast />
	</div>
{/if}

<style>
	/* === Variables === */
	:root {
		--main-color: oklab(0.88 -0.18 0.03);
		--border-color: rgb(102, 173, 255);
		--background-color: rgb(12, 44, 76);
		--text-color: rgb(176, 163, 217);
		--font-type1: 'Akrobat-black';
		--font-type2: Arial;
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
		background-attachment: fixed;
		background-size: cover;
		flex-direction: column;
		height: fit-content;
		min-height: 100%;
		width: 100vw;
		background: radial-gradient(farthest-corner circle at 0% 0%, rgb(18 69 89), #0f0f4d);
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
		justify-content: center;
		height: fit-content;
		min-height: 45vh;
		width: 100%;
		gap: 1.5em;
		padding-top: 2em;
		margin-bottom: 4em;
		color: white;
	}

	/* === Components === */
	.background-filter {
		position: fixed;
		width: 100vw;
		height: 100vh;
		z-index: 1;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	/* === Utilities ===*/
	.filter {
		filter: blur(5px);
	}
</style>
