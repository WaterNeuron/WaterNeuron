<script lang="ts">
	import { language } from '$lib/stores';
	import StatsWidget from './StatsWidget.svelte';
	import Swap from './Swap.svelte';
	import { fade, slide } from 'svelte/transition';

	export let data;

	function getContent(language: string) {
		switch (language) {
			case 'en':
				return data.en.sections;
			case 'es':
				return data.es.sections;
			case 'ru':
				return data.ru.sections;
			case 'ja':
				return data.ja.sections;
			default:
				return [];
		}
	}

	let toggledMap = getContent($language).map(() => {
		return false;
	});
</script>

<StatsWidget />
<Swap />
{#key $language}
	<div class="faq" in:fade={{ duration: 500 }}>
		<h1>FAQ</h1>
		{#each getContent($language) as section, i}
			<button
				class="faq-btn"
				on:click={() => {
					toggledMap[i] = !toggledMap[i];
				}}
			>
				<h2>{section.title}</h2>
				<img
					width="20em"
					height="20em"
					src="/icon/down-arrow.svg"
					alt="Down arrow."
					class:arrow-down={!toggledMap[i]}
					class:arrow-up={toggledMap[i]}
				/>
			</button>

			{#if toggledMap[i]}
				<p transition:slide>{section.content}</p>
			{/if}
		{/each}
	</div>
{/key}
<div class="lang-selection">
	<button
		on:click={() => language.set('en')}
		class="lang-btn"
		class:language-active={$language === 'en'}
	>
		en
	</button>
	<button
		on:click={() => language.set('es')}
		class="lang-btn"
		class:language-active={$language === 'es'}
	>
		es
	</button>
	<button
		on:click={() => language.set('ru')}
		class="lang-btn"
		class:language-active={$language === 'ru'}
	>
		ru
	</button>
	<button
		on:click={() => language.set('ja')}
		class="lang-btn"
		class:language-active={$language === 'ja'}
	>
		ja
	</button>
</div>

<style>
	/* === Base Styles ===*/
	.faq {
		display: flex;
		align-items: center;
		flex-direction: column;
		width: 90vw;
		max-width: 800px;
		overflow-y: auto;
	}

	h1 {
		color: white;
		font-size: 42px;
		font-family: var(--font-type1);
	}

	h2 {
		color: white;
		font-size: 1.5em;
		text-align: start;
	}

	p {
		color: var(--text-color);
		font-weight: 400;
		font-family: CircularXX, sans-serif;
		-webkit-font-smoothing: antialiased;
		line-height: 24px;
	}

	/* === Components === */

	.faq-btn {
		background: none;
		border: none;
		border-bottom: 2px solid;
		border-color: white;
		margin-top: 2em;
		padding-top: 2em;
		width: 100%;
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin: 0;
	}

	.lang-btn {
		background: none;
		border: none;
		color: white;
		cursor: pointer;
	}
	/* === Layout === */
	.lang-selection {
		display: flex;
	}

	/* Utilities */
	.arrow-up {
		transform: rotate(0);
	}

	.arrow-down {
		transform: rotate(180deg);
	}

	.language-active {
		text-decoration: underline;
	}
</style>
