<script lang="ts">
	import en from './lang/en.json';
	import es from './lang/es.json';
	import ru from './lang/ru.json';
	import ja from './lang/ja.json';
	import cn from './lang/cn.json';
	import { language } from '$lib/stores';
	import { fade, slide } from 'svelte/transition';
	import ArrowIcon from '$lib/icons/ArrowIcon.svelte';
	import { onMount } from 'svelte';

	const data = {
		en: en,
		es: es,
		ru: ru,
		ja: ja,
		cn: cn
	};

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
			case 'cn':
				return data.cn.sections;
			default:
				return [];
		}
	}

	let toggledMap = getContent($language).map(() => {
		return false;
	});

	onMount(() => {
		toggledMap[0] = !toggledMap[0];
	});
</script>

{#key $language}
	<div class="faq" in:fade={{ duration: 200 }}>
		<div class="faq-header">
			<h1>FAQ</h1>
			<div class="lang-selection">
				{#each ['en', 'es', 'ru', 'cn', 'ja'] as lang}
					<button
						on:click={() => language.set(lang)}
						class="lang-btn"
						class:language-active={$language === lang}
					>
						{lang.toUpperCase()}
					</button>
				{/each}
			</div>
		</div>
		{#each getContent($language) as section, i}
			<button
				class="faq-container"
				on:click={() => {
					toggledMap[i] = !toggledMap[i];
				}}
			>
				<div class="faq-container-title">
					<h2>{section.title}</h2>
					<ArrowIcon isUp={toggledMap[i]} />
				</div>
				{#if toggledMap[i]}
					<p transition:slide={{ duration: 250 }}>{section.content}</p>
				{/if}
			</button>
		{/each}
	</div>
{/key}

<style>
	/* === Base Styles ===*/
	.faq {
		display: flex;
		align-items: center;
		flex-direction: column;
		width: 30em;
		max-width: 95vw;
		overflow-y: auto;
		gap: 0.5em;
	}

	.faq-container {
		width: 100%;
		border: var(--main-container-border);
		border-radius: 15px;
		background: var(--background-color);
		display: flex;
		flex-direction: column;
		text-align: left;
		justify-content: space-between;
	}

	.faq-container p {
		width: 90%;
		align-self: center;
		margin: 0 0 1em;
	}

	.faq-container-title {
		display: flex;
		width: 100%;
		padding: 0 5%;
		box-sizing: border-box;
		flex-direction: row;
		justify-content: space-between;
		align-items: center;
	}

	.faq-container-title :global(svg) {
		flex-shrink: 0;
		width: 16px;
		height: 16px;
	}

	.faq-header {
		display: flex;
		width: 100%;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.25em;
	}

	h1 {
		color: var(--faq-color);
		font-size: 1.2em;
		margin: 0;
		font-family: var(--main-font);
	}

	.lang-selection {
		display: flex;
		gap: 0.1em;
	}

	.lang-btn {
		background: none;
		border: none;
		color: var(--text-color);
		cursor: pointer;
		font-family: CircularXX, sans-serif;
		font-size: 0.8em;
		padding: 0.2em 0.4em;
		border-radius: 4px;
		opacity: 0.5;
		transition: opacity 0.15s;
	}

	.lang-btn:hover {
		opacity: 1;
	}

	.language-active {
		opacity: 1;
		font-weight: 600;
	}

	h2 {
		color: var(--faq-color);
		font-weight: 500;
		font-size: 1em;
		text-align: start;
		margin: 0.85em 0;
		padding-right: 1em;
	}

	p {
		color: var(--text-color);
		font-weight: 400;
		font-family: CircularXX, sans-serif;
		-webkit-font-smoothing: antialiased;
		line-height: 24px;
		text-align: left;
	}
</style>
