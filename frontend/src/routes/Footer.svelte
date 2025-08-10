<script>
	import { isLogging } from '$lib/stores';
	import OpenChatIcon from '$lib/icons/OpenChatIcon.svelte';
	import TelegramIcon from '$lib/icons/TelegramIcon.svelte';
	import TwitterIcon from '$lib/icons/TwitterIcon.svelte';
	import GitBookIcon from '$lib/icons/GitBookIcon.svelte';
	import { isMobile } from '$lib';
	import GithubIcon from '$lib/icons/GithubIcon.svelte';
</script>

<footer class:filter={$isLogging}>
	{#if !isMobile}
		<div class="left-container">
			<a href="https://docs.waterneuron.fi/" target="_blank" rel="noreferrer"> Documentation </a>
			<a href="https://dashboard.waterneuron.fi/" target="_blank" rel="noreferrer"> WTN Calculator </a>
			<a
				href="https://docs.google.com/spreadsheets/d/1Y8JXo8xOB786usk8BaWPPcBJB7oFR0HWvA5y8kcfRqU/edit?gid=746571094#gid=746571094"
				target="_blank"
				rel="noreferrer"
			>
				Spreadsheet
			</a>
		</div>
	{/if}
	<div class="right-container">
		<a href="https://docs.waterneuron.fi/" rel="noreferrer" target="_blank">
			<GitBookIcon />
		</a>
		<a
			href="https://oc.app/community/3fudd-diaaa-aaaar-bgmca-cai/channel/154489341794150690442253978518193726806"
			rel="noreferrer"
			target="_blank"
		>
			<OpenChatIcon />
		</a>
		<a
			href="https://twitter.com/WaterNeuron"
			target="_blank"
			rel="noreferrer"
			style="width: 20px; height: 20px"
		>
			<TwitterIcon />
		</a>
		<a href="https://github.com/WaterNeuron" target="_blank" rel="noreferrer">
			<GithubIcon />
		</a>
		<a href="https://t.me/waterneuron" rel="noreferrer" target="_blank">
			<TelegramIcon />
		</a>
	</div>
</footer>

<style>
	/* === Base Styles === */
	footer {
		display: flex;
		justify-content: space-between;
		color: var(--text-color);
		height: fit-content;
		width: 100%;
		box-sizing: border-box;
		padding: 1em;
	}

	a {
		align-content: end;
		text-decoration: none;
		color: var(--text-color);
		font-family: CircularXX, sans-serif;
		font-size: 0.85em;
	}

	/* === Layout === */
	.right-container {
		display: flex;
		gap: 1em;
		margin-left: auto;
		display: flex;
		align-items: center;
	}

	.right-container a {
		width: 25px;
		height: 25px;
	}

	.left-container {
		display: flex;
		gap: 1em;
		align-items: end;
	}

	.left-container a:hover {
		text-decoration: underline;
	}

	/* === Utillities === */
	.filter {
		filter: blur(5px);
	}
</style>
