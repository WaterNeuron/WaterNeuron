<script lang="ts">
	import { canisters, inChart, chartData } from '$lib/stores';
	import type { Event } from '$lib/../declarations/water_neuron/water_neuron.did';
	import { Chart } from 'flowbite-svelte';
	import { onMount } from 'svelte';
	import ChangeIcon from '$lib/icons/ChangeIcon.svelte';
	import CloseIcon from '$lib/icons/CloseIcon.svelte';
	import { isMobile } from '$lib';

	type Scale = '1m' | '3m' | '6m' | '1y' | 'All';

	const NANOS_PER_SEC = 1_000_000_000n;
	const ONE_MONTH_MILLIS = 30 * 24 * 60 * 60 * 1_000;
	const BATCH_SIZE = 2_000n;

	let dialog: HTMLDialogElement;
	let chart: ApexCharts | undefined;

	export let isInverted: boolean;
	let exchangeRates: number[] = [];
	let timestamps: number[] = [];
	let isFirstLoad = true;
	let width = 600;
	let height = 300;
	const scales: Scale[] = ['1m', '3m', '6m', '1y', 'All'];
	let options: ApexCharts.ApexOptions;

	const getApexOptions = () => {
		return {
			chart: {
				width,
				height,
				type: 'area' as 'area',
				fontFamily: 'var(--main-font)',
				foreColor: 'var(--title-color)',
				toolbar: { show: false },
				dropShadow: { enabled: false },
				animations: {
					enabled: !isFirstLoad && chart !== undefined,
					dynamicAnimation: { enabled: !isFirstLoad && chart !== undefined, speed: 300 }
				}
			},
			series: [
				{
					name: !isInverted ? 'nICP/ICP' : 'ICP/nICP',
					data: isInverted
						? exchangeRates.map((xr) => {
								return Number((1 / xr).toFixed(4));
							})
						: exchangeRates,
					color: '#286e5f'
				}
			] as ApexAxisChartSeries,
			fill: {
				type: 'gradient',
				gradient: {
					opacityFrom: 0.55,
					opacityTo: 0,
					shade: '#286e5f',
					gradientToColors: ['#286e5f']
				}
			},
			stroke: {
				width: 2,
				colors: ['#286e5f']
			},
			xaxis: {
				categories: timestamps,
				type: 'datetime' as 'datetime'
			},
			tooltip: { enabled: true },
			dataLabels: {
				enabled: false
			}
		};
	};

	async function updateCache() {
		const [ts, xr] = await fetchEvent();
		chartData.set({ timestamps: [1718748000000].concat(ts), exchangeRates: [1].concat(xr) });
		timestamps = [1718748000000].concat(ts);
		exchangeRates = [1].concat(xr);
	}

	function checkCache() {
		if ($chartData) {
			timestamps = $chartData.timestamps;
			exchangeRates = $chartData.exchangeRates;
		} else {
			updateCache();
		}
	}

	async function fetchEvent(): Promise<[number[], number[]]> {
		const response = await fetch(
			'https://wtn-exchange-rate.s3.eu-north-1.amazonaws.com/exchange-rates.json'
		);
		const { timestamps: ts, exchangeRates: xrs } = await response.json();
		return [ts, xrs];
	}

	onMount(() => {
		dialog = document.getElementById('chartDialog') as HTMLDialogElement;
		dialog.showModal();
		checkCache();
		handleResize();
		window.addEventListener('resize', handleResize);

		return () => window.removeEventListener('resize', handleResize);
	});

	function handleResize() {
		width = Math.min(600, 0.9 * document.getElementsByClassName('chart-container')[0].clientWidth);
		height = Math.min(300, width / 2);
	}
	const setDateRange = (scale: Scale) => {
		if (!$chartData) return;

		const { timestamps: fullRangeTs, exchangeRates: fullRangeXrs } = $chartData;
		const now = Date.now();
		const range =
			{
				'1m': ONE_MONTH_MILLIS,
				'3m': 3 * ONE_MONTH_MILLIS,
				'6m': 6 * ONE_MONTH_MILLIS,
				'1y': 12 * ONE_MONTH_MILLIS,
				All: Infinity
			}[scale] ?? Infinity;

		const filtered = fullRangeTs
			.map((timestamp, index) => ({ timestamp, rate: fullRangeXrs[index] }))
			.filter(({ timestamp }) => timestamp + range >= now);

		timestamps = filtered.map(({ timestamp }) => timestamp);
		exchangeRates = filtered.map(({ rate }) => rate);
	};

	$: timestamps,
		exchangeRates,
		isInverted,
		width,
		height,
		isFirstLoad,
		chart,
		(options = getApexOptions());
</script>

<dialog
	id="chartDialog"
	on:close={() => {
		inChart.set(false);
	}}
>
	<div class="chart-container">
		<div class="header-container">
			<h2>{isMobile ? '' : 'Exchange rate'} {isInverted ? 'ICP/nICP' : 'nICP/ICP'}</h2>
			<button
				class="change-btn"
				on:click={() => {
					isFirstLoad = false;
					isInverted = !isInverted;
				}}
			>
				<ChangeIcon />
			</button>
			<button
				class="close-btn"
				on:click={() => {
					dialog.close();
				}}
			>
				<CloseIcon color="--title-color" />
			</button>
		</div>
		<div class="chart-content-container" style:width="{width}px" style:height="{height}px">
			{#if timestamps.length === 0 || exchangeRates.length === 0}
				<div class="spinner"></div>
			{/if}
			<Chart {options} bind:chart />
		</div>
		<div class="scales">
			{#each scales as scale}
				<button
					class="scale-btn"
					on:click={() => {
						isFirstLoad = false;
						setDateRange(scale);
					}}>{scale}</button
				>
			{/each}
		</div>
	</div>
</dialog>

<style>
	::backdrop {
		backdrop-filter: blur(5px);
	}

	dialog {
		display: flex;
		background: transparent;
		justify-content: center;
		align-items: center;
		height: fit-content;
		min-height: 100%;
		min-width: 100dvw;
		padding: 0;
		margin: 0;
		border: none;
	}

	.chart-container {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		background: var(--background-color);
		width: 90%;
		max-width: 600px;
		position: relative;
		padding: 1em;
		border-radius: 10px;
		border: var(--main-container-border);
	}

	.chart-content-container {
		display: flex;
		justify-content: center;
		align-items: center;
	}

	.close-btn {
		background: none;
		top: 1em;
		right: 1em;
		z-index: 10;
		cursor: pointer;
		border: none;
		display: flex;
		flex-grow: 1;
		justify-content: end;
		align-items: center;
	}

	.scales {
		display: flex;
		width: fit-content;
		gap: 1px;
	}

	.scales button:first-child {
		border-top-left-radius: 8px;
		border-bottom-left-radius: 8px;
	}

	.scales button:last-child {
		border-top-right-radius: 8px;
		border-bottom-right-radius: 8px;
	}

	h2 {
		color: var(--title-color);
		font-family: var(--main-font);
		margin: 0;
	}

	.change-btn {
		border: none;
		display: flex;
		width: fit-content;
		height: fit-content;
		background: transparent;
		padding: 0;
		margin: 0;
		cursor: pointer;
	}

	.change-btn:hover {
		transform: scale(1.2);
		animation: invert 0.5s ease;
	}

	@keyframes invert {
		from {
			transform: scale(1);
		}
		to {
			transform: scale(1.2);
		}
	}

	.header-container {
		display: flex;
		align-items: center;
		width: 100%;
		gap: 1em;
	}

	.scale-btn {
		border: none;
		background-color: #286e5f5e;
		height: 40px;
		width: 60px;
		color: var(--title-color);
	}

	.scale-btn:hover {
		background-color: #286e5fd1;
	}

	.spinner {
		width: 1em;
		height: 1em;
		border: 3px solid var(--main-color);
		border-top-color: transparent;
		border-radius: 50%;
		animation: spin 1s linear infinite;
		position: absolute;
	}

	@keyframes spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}

	@media (max-width: 767px) {
		h2 {
			font-size: 20px;
		}
	}
</style>
