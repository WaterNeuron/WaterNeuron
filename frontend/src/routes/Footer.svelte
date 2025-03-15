<script>
	import { isLogging } from '$lib/stores';
	import OpenChatIcon from '$lib/icons/OpenChatIcon.svelte';
	import TelegramIcon from '$lib/icons/TelegramIcon.svelte';
	import TwitterIcon from '$lib/icons/TwitterIcon.svelte';
	import GitBookIcon from '$lib/icons/GitBookIcon.svelte';
	import GithubIcon from '$lib/icons/GithubIcon.svelte';
</script>

<footer class:filter={$isLogging}>
	<div class="central-container">
		<img alt="WTN icon" src="/tokens/WTN.webp" id="wtn-icon" />
		<a
			class="item--1"
			href="https://oc.app/community/3fudd-diaaa-aaaar-bgmca-cai/channel/154489341794150690442253978518193726806"
			rel="noreferrer"
			target="_blank"
		>
			<OpenChatIcon />
		</a>
		<a class="item--5" href="https://github.com/WaterNeuron" target="_blank" rel="noreferrer">
			<GithubIcon />
		</a>
		<a class="item--3" href="https://twitter.com/WaterNeuron" target="_blank" rel="noreferrer">
			<TwitterIcon />
		</a>
		<a class="item--4" href="https://docs.waterneuron.fi/" rel="noreferrer" target="_blank">
			<GitBookIcon />
		</a>
		<a class="item--2" href="https://t.me/waterneuron" rel="noreferrer" target="_blank">
			<TelegramIcon />
		</a>
		<p>WaterNeuron</p>
	</div>
</footer>

<style>
	/* === Base Styles === */
	footer {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		color: var(--text-color);
		height: fit-content;
		width: 100%;
		margin-bottom: 2vh;
	}

	img {
		height: 30px;
		width: 30px;
		align-content: center;
		justify-content: center;
	}

	p {
		width: fit-content;
		position: absolute;
		bottom: -1em;
		font-family: var(--main-font);
	}

	a {
		position: absolute;
		top: 100%;
		left: 50;
		transform: translate(-50%, -50%);
	}

	/* === Layout === */
	.central-container {
		display: flex;
		justify-content: center;
		align-items: center;
		position: relative;
		height: 7em;
		width: 10em;
	}

	/* === Component === */
	#wtn-icon {
		height: 4em;
		width: 4em;
		position: absolute;
		bottom: 20px;
	}

	/* === Utillities === */
	.filter {
		filter: blur(5px);
	}

	.item--1 {
		top: 35%;
		left: 8%;
		transform: translate(-50%, -50%);
	}

	.item--2 {
		top: 10%;
		left: 23%;
		transform: translate(-50%, -50%);
	}

	.item--3 {
		top: 0%;
		left: 50%;
		transform: translate(-50%, -50%);
	}

	.item--4 {
		top: 10%;
		right: 23%;
		transform: translate(50%, -50%);
	}

	.item--5 {
		top: 35%;
		right: 8%;
		transform: translate(50%, -50%);
	}
</style>
