<script lang="ts">
	import { isLogging, inMobileMenu, user, ledgerDevice } from '$lib/stores';
	import { displayNumber } from '$lib';
	import { internetIdentityLogout } from '$lib/authentification';
	import { ThemeToggle } from '@dfinity/gix-components';
	import PowerOffIcon from '$lib/icons/PowerOffIcon.svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { Principal } from '@dfinity/principal';

	export function displayUserPrincipal(principal: Principal | undefined) {
		if (principal === undefined) return '-/-';
		const a = principal.toString().split('-');
		return a[0] + '...' + a[a.length - 1];
	}
</script>

<nav class:filter={$isLogging}>
	<a href="/" class="menu-selection-container" title="home-btn">
		<img src="/tokens/WTN.webp" width="50em" height="50em" alt="WTN logo" />
		<div class="wave-title">
			<h1>WaterNeuron</h1>
		</div>
	</a>

	<div class="right-container">
		<div class="theme-toggle">
			<ThemeToggle />
		</div>
		{#if !($page.url.pathname === '/launchpad/')}
			{#if !$user}
				<button
					title="connect-btn"
					class="smart"
					on:click={() => {
						isLogging.set(true);
					}}
				>
					Connect
				</button>
			{:else}
				<a href="/wallet" class="wallet-btn" id="wallet-info">
					<h2 style:font-weight={'bold'}>{displayUserPrincipal($user.principal)}</h2>
					<p title="icp-balance-nav">{displayNumber($user.icpBalance(), 2)} ICP</p>
					<p title="nicp-balance-nav">
						{displayNumber($user.nicpBalance(), 2)} nICP
					</p>
					<p title="wtn-balance-nav">{displayNumber($user.wtnBalance(), 2)} WTN</p>
				</a>
				<button
					id="disconnect-btn"
					class="wallet-action-btn"
					on:click={async () => {
						await internetIdentityLogout();
						user.set(undefined);
						ledgerDevice.set(undefined);
						goto('/');
					}}
				>
					<PowerOffIcon />
				</button>
				<button
					id="menu-btn"
					class="wallet-action-btn"
					on:click={() => {
						inMobileMenu.set(true);
					}}
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						height="24"
						viewBox="0 -960 960 960"
						width="24"
						fill="var(--title-color)"
					>
						<path d="M120-240v-80h720v80H120Zm0-200v-80h720v80H120Zm0-200v-80h720v80H120Z"></path>
					</svg>
				</button>
			{/if}
		{/if}
	</div>
</nav>

<style>
	.theme-toggle {
		color: var(--title-color);
	}

	/* === Base Styles === */
	nav {
		display: flex;
		align-items: center;
		justify-content: space-between;
		color: white;
		height: fit-content;
		padding: 0 2.5vw;
		margin-top: 2vh;
	}

	button,
	a {
		background-color: transparent;
		border: none;
		cursor: pointer;
		border-radius: 0.3em;
		transition: all 0.3s ease;
		color: white;
		font-weight: bold;
	}

	button:hover {
		background-color: var(--input-color);
	}

	button {
		display: flex;
		align-items: center;
		justify-content: center;
	}

	div {
		display: flex;
		align-items: center;
	}

	a {
		display: flex;
		align-items: center;
		text-decoration: none;
		color: inherit;
		font-family: var(--secondary-font);
	}

	p,
	h2 {
		margin: 0;
		padding: 0;
		font-size: 13px;
	}

	h2 {
		margin: 0 0 4px 0;
	}

	h1 {
		color: var(--title-color);
		font-weight: 800;
		font-family: var(--main-font);
		letter-spacing: -0.11em;
	}

	p {
		text-align: end;
		font-weight: normal;
	}

	/* ===Layout=== */
	.menu-selection-container {
		gap: 1em;
	}

	.menu-selection-container:hover {
		cursor: pointer;
	}

	.menu-selection-container h1 {
		font-size: 1.5em;
		cursor: pointer;
		font-family: var(--main-font);
		position: absolute;
	}

	.wave-title {
		position: relative;
	}

	.right-container {
		display: flex;
		align-items: center;
		gap: 1em;
		color: var(--title-color);
	}

	/* === Components === */
	.smart {
		display: flex;
		align-items: center;
		height: fit-content;
		font-size: 16px;
		color: var(--stake-text-color);
		padding: 0.5em;
		margin: 0;
	}

	.smart:hover {
		background-color: var(--main-color);
		color: var(--main-button-text-color);
		cursor: pointer;
	}

	.wallet-action-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 2em;
		width: 3em;
		padding: 1.3em 0.5em;
		margin: 0;
	}

	.wallet-btn {
		display: flex;
		flex-direction: column;
		text-align: right;
		font-size: 16px;
		padding: 0.5em;
		margin: 0;
	}

	.wallet-btn:hover {
		cursor: pointer;
	}

	#menu-btn {
		display: none;
	}

	#wallet-info {
		align-items: end;
	}
	#wallet-info:hover {
		background-color: var(--input-color);
		cursor: pointer;
	}

	/* === Utillities === */
	.filter {
		filter: blur(5px);
	}

	/* === Responsive Design === */
	@media (max-width: 767px) {
		nav {
			background: none;
			box-shadow: none;
			height: 100px;
		}

		.menu-selection-container h1 {
			display: none;
		}

		#disconnect-btn {
			display: none;
		}

		#menu-btn {
			display: flex;
		}

		img {
			height: 50px;
			width: 50px;
		}

		.right-container {
			gap: 0;
		}
	}
</style>
