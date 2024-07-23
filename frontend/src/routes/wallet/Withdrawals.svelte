<script lang="ts">
	import { bigintE8sToNumber, displayUsFormat, renderStatus, displayTimeLeft } from '$lib';
	import { state, user } from '$lib/stores';
	import { onMount } from 'svelte';
	import type {
		WithdrawalDetails,
		NeuronId
	} from '../../declarations/water_neuron/water_neuron.did';
	import { fade } from 'svelte/transition';

	let withdrawalRequests: WithdrawalDetails[];
	let withdrawalStatuses: {
		[key: string]: string;
	} = {};

	function displayNeuronId(neuronId: [] | [NeuronId], truncate = true): string {
		if (neuronId.length == 0) {
			return 'Not Set';
		} else if (truncate) {
			const id = neuronId[0].id.toString();
			return id.substring(0, 4) + '...' + id.substring(id.length - 5, id.length - 1);
		} else {
			return neuronId[0].id.toString();
		}
	}

	const fetchWithdrawals = async () => {
		if ($user && $state) {
			withdrawalRequests = await $state.waterNeuron.get_withdrawal_requests([$user.principal]);
			await fetchStatuses();
		}
	};

	const fetchStatuses = async () => {
		if ($user && $state) {
			for (const detail of withdrawalRequests) {
				const neuronId = detail.request.neuron_id;
				if (neuronId.length !== 0) {
					try {
						const status = await fetchTimeLeft(neuronId[0]);
						withdrawalStatuses[neuronId[0].id.toString()] = status;
					} catch (e) {
						console.error(e);
					}
				}
			}
		}
	};

	async function fetchTimeLeft(neuron_id: NeuronId): Promise<string> {
		if (process.env.DFX_NETWORK !== 'ic') return 'Waiting dissolvement';
		try {
			const response = await fetch(
				`https://ic-api.internetcomputer.org/api/v3/neurons/${neuron_id.id}`
			);
			if (!response.ok) {
				throw new Error('Network response was not ok');
			}
			const data = await response.json();
			const neuron_created_at = data['created_timestamp_seconds'];
			return displayTimeLeft(Number(neuron_created_at), isMobile);
		} catch (error) {
			throw new Error('Failed to fetch with error: ' + error);
		}
	}

	onMount(() => {
		fetchWithdrawals();
	});

	$: $user, () => fetchWithdrawals();

	const userAgent = navigator.userAgent;
	const isMobile = /Mobi|Android|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(userAgent);
</script>

{#if withdrawalRequests && withdrawalRequests.length >= 1}
	<div class="withdrawals-container" in:fade={{ duration: 500 }}>
		<h1>Withdrawal Requests</h1>
		<table>
			<thead>
				{#if isMobile}
					<tr>
						<th>ICP Due</th>
						<th>Neuron Id</th>
						<th>Status</th>
					</tr>
				{:else}
					<tr>
						<th>nICP Burned</th>
						<th>ICP Due</th>
						<th>Neuron Id</th>
						<th>Status</th>
						<th>Id</th>
					</tr>
				{/if}
			</thead>
			<tbody>
				{#each withdrawalRequests as details}
					{#if isMobile}
						<tr>
							<td>{displayUsFormat(bigintE8sToNumber(details.request.icp_due))}</td>
							<td>
								<a
									target="_blank"
									rel="noreferrer"
									href={'https://dashboard.internetcomputer.org/neuron/' +
										displayNeuronId(details.request.neuron_id, false)}
								>
									{displayNeuronId(details.request.neuron_id)}
								</a>
							</td>
							<td>
								{details.request.neuron_id[0]
									? withdrawalStatuses[details.request.neuron_id[0].id.toString()]
									: 'Waiting Dissolvement'}
							</td>
						</tr>
					{:else}
						<tr>
							<td>{displayUsFormat(bigintE8sToNumber(details.request.nicp_burned))}</td>
							<td>{displayUsFormat(bigintE8sToNumber(details.request.icp_due))}</td>
							<td>
								<a
									target="_blank"
									rel="noreferrer"
									href={'https://dashboard.internetcomputer.org/neuron/' +
										displayNeuronId(details.request.neuron_id, false)}
								>
									{displayNeuronId(details.request.neuron_id, false)}
								</a>
							</td>
							<td>
								{details.request.neuron_id[0]
									? withdrawalStatuses[details.request.neuron_id[0].id.toString()]
									: 'Waiting Dissolvement'}
							</td>
							<!-- <td
								>{#if renderStatus(details.status)}
									...
								{:then status}
									{@html status}
								{/await}
							</td> -->
							<td>{details.request.withdrawal_id}</td>
						</tr>
					{/if}
				{/each}
			</tbody>
		</table>
	</div>
{/if}

<style>
	/* === Layout === */
	.withdrawals-container {
		background-color: #0c2c4c;
		border: 2px solid #66adff;
		border-radius: 10px;
		color: white;
		padding: 2em;
		display: flex;
		flex-direction: column;
		width: 44em;
		max-width: 97vw;
	}

	/* === Base Styles ==== */
	h1 {
		text-align: center;
		font-family: var(--font-type1);
	}

	table {
		display: table;
		border-collapse: collapse;
		margin: 1.5em;
		font-family: sans-serif;
	}

	th {
		text-align: center;
		padding: 0.5em;
	}

	td {
		border-top: 2px solid;
		text-align: center;
		padding: 1em;
	}

	a {
		text-decoration: underline;
		color: var(--text-color);
	}

	/* === Responsive === */
	@media (max-width: 767px) {
		.withdrawals-container {
			padding: 0em;
		}
	}
</style>
