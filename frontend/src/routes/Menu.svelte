<script>
	import { inMobileMenu, ledgerDevice, user } from '$lib/stores';
	import { internetIdentityLogout } from '$lib/authentification';
</script>

<div class="background-menu">
	<div class="menu-container">
		<div class="menu-panel">
			<a
				href="/"
				on:click={() => {
					inMobileMenu.set(false);
				}}>Stake</a
			>
			<a
				href="/wallet"
				on:click={() => {
					inMobileMenu.set(false);
				}}>Wallet</a
			>
			<a
				href="/"
				on:click={async () => {
					await internetIdentityLogout();

					user.set(undefined);
					ledgerDevice.set(undefined);
					inMobileMenu.set(false);
				}}>Disconnect</a
			>
		</div>

		<button
			on:click={() => {
				inMobileMenu.set(false);
			}}
		>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				width="32px"
				height="32px"
				viewBox="0 0 24 24"
				fill="none"
				stroke="var(--stake-text-color)"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<line x1="18" y1="6" x2="6" y2="18"></line>
				<line x1="6" y1="6" x2="18" y2="18"></line>
			</svg>
		</button>
	</div>
</div>

<style>
	/* === Base Styles === */
	div {
		display: flex;
	}

	button {
		background-color: transparent;
		border: none;
		height: 10%;
	}

	a {
		color: var(--stake-text-color);
		font-weight: 200;
		text-align: start;
		text-decoration: none;
		font-family: var(--secondary-font);
		font-weight: 200;
	}

	/* === Layout === */

	.background-menu {
		height: 100vh;
		width: 100vw;
		background: var(--background-color);
		z-index: 1;
	}

	.menu-container {
		height: fit-content;
		width: 100%;
		justify-content: space-between;
		padding: 1em;
	}

	.menu-panel {
		font-size: 16px;
		color: white;
		display: flex;
		gap: 2em;
		flex-direction: column;
		margin: 2em;
		height: 100%;
	}
</style>
