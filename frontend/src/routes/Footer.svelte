<script>
	import { isLogging } from '$lib/stores';
	import OpenChatIcon from '$lib/icons/OpenChatIcon.svelte';
	import TwitterIcon from '$lib/icons/TwitterIcon.svelte';
	import GitBookIcon from '$lib/icons/GitBookIcon.svelte';
	import GithubIcon from '$lib/icons/GithubIcon.svelte';
	import DiscordIcon from '$lib/icons/DiscordIcon.svelte';
	import { ThemeToggle } from '@dfinity/gix-components';
</script>

<footer class:filter={$isLogging}>
	<div class="main-row">
		<a href="https://dashboard.waterneuron.fi/" target="_blank" rel="noreferrer">Calculator</a>
		<a
			href="https://docs.google.com/spreadsheets/d/1Y8JXo8xOB786usk8BaWPPcBJB7oFR0HWvA5y8kcfRqU/edit?gid=746571094#gid=746571094"
			target="_blank"
			rel="noreferrer"
		>
			Spreadsheet
		</a>
		<a href="/recovery">Recovery</a>

		<span class="separator" />

		<div class="socials">
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
			<a href="https://twitter.com/WaterNeuron" target="_blank" rel="noreferrer">
				<TwitterIcon />
			</a>
			<a href="https://github.com/WaterNeuron" target="_blank" rel="noreferrer">
				<GithubIcon />
			</a>
			<a href="https://discord.com/invite/UMvQq7yukd" target="_blank" rel="noreferrer">
				<DiscordIcon />
			</a>
		</div>
	</div>
	<div class="bottom-row">
		<span class="copyright">&copy; 2026 WaterNeuron</span>
		<div class="theme-toggle">
			<ThemeToggle />
		</div>
	</div>
</footer>

<style>
	footer {
		display: flex;
		flex-direction: column;
		align-items: center;
		color: var(--text-color);
		width: 100%;
		box-sizing: border-box;
		padding: 2em 1em 1em;
		gap: 1.25em;
	}

	.main-row {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 1em;
		flex-wrap: wrap;
	}

	a {
		text-decoration: none;
		color: var(--text-color);
		font-family: CircularXX, sans-serif;
		font-size: 0.8em;
	}

	a:hover {
		text-decoration: underline;
	}

	.separator {
		width: 1px;
		height: 1em;
		background: var(--text-color);
		opacity: 0.3;
	}

	.socials {
		display: flex;
		gap: 0.75em;
		align-items: center;
	}

	.socials a {
		width: 20px;
		height: 20px;
	}

	.bottom-row {
		display: flex;
		align-items: center;
		gap: 0.75em;
	}

	.copyright {
		font-family: CircularXX, sans-serif;
		font-size: 0.7em;
		opacity: 0.4;
	}

	.theme-toggle {
		color: var(--text-color);
		opacity: 0.5;
	}

	@media (max-width: 767px) {
		.main-row {
			gap: 0.75em;
		}

		.separator {
			display: none;
		}
	}

	.filter {
		filter: blur(5px);
	}
</style>
