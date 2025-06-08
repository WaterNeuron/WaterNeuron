<script lang="ts">
	import {
		bigintE8sToNumber,
		displayUsFormat,
		renderStatus,
		displayTimeLeft,
		fetchNeuronCreationTimestamp,
		isMobile,
		displayNeuronId
	} from '$lib';
	import { user, canisters, selectedWithdrawal, inCancelWarningMenu } from '$lib/stores';
	import { onMount } from 'svelte';
	import type { WithdrawalDetails } from '$lib/../declarations/water_neuron/water_neuron.did';
	import { fade } from 'svelte/transition';
	import RevertIcon from '$lib/icons/RevertIcon.svelte';

	let activeWithdrawalRequests: { [key: number]: WithdrawalDetails } = {};
	let cancelledWithdrawalRequests: { [key: number]: WithdrawalDetails } = {};
	let timesLeft: {
		[key: number]: string;
	} = {};

	const fetchWithdrawals = async () => {
		if ($user && $canisters) {
			const withdrawalRequests =
				await $canisters.waterNeuron.anonymousActor.get_withdrawal_requests([
					{ owner: $user.principal, subaccount: [] }
				]);

			const [newCancelled, newActive]: [
				{ [key: number]: WithdrawalDetails },
				{ [key: number]: WithdrawalDetails }
			] = [{}, {}];
			for (const withdrawal of withdrawalRequests) {
				if ('Cancelled' in withdrawal.status) {
					newCancelled[Number(withdrawal.request.withdrawal_id)] = withdrawal;
				} else {
					newActive[Number(withdrawal.request.withdrawal_id)] = withdrawal;
				}
			}

			cancelledWithdrawalRequests = newCancelled;
			activeWithdrawalRequests = newActive;
			await fetchTimesLeft();
		}
	};

	const fetchTimesLeft = async () => {
		if ($user) {
			const newTimesLeft: { [key: number]: string } = {};
			for (const detail of Object.values(activeWithdrawalRequests)) {
				const neuronId = detail.request.neuron_id;
				if (neuronId.length !== 0) {
					try {
						const createdAt = await fetchNeuronCreationTimestamp(neuronId[0]);
						newTimesLeft[Number(neuronId[0].id)] = displayTimeLeft(createdAt, isMobile);
					} catch (e) {
						console.log(e);
					}
				}
			}
			timesLeft = newTimesLeft;
		}
	};

	onMount(() => {
		fetchWithdrawals();
		const intervalId = setInterval(async () => {
			await fetchWithdrawals();
		}, 5000);

		return () => clearInterval(intervalId);
	});
</script>

{#if Object.keys(activeWithdrawalRequests).length + Object.keys(cancelledWithdrawalRequests).length >= 1}
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
				{#each Object.values(activeWithdrawalRequests).concat(Object.values(cancelledWithdrawalRequests)) as details, index}
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
								{#if renderStatus(details.status) === 'Waiting Dissolvement'}
									{details.request.neuron_id[0]
										? timesLeft[Number(details.request.neuron_id[0].id)]
										: '-/-'}
								{:else}
									{renderStatus(details.status)}
								{/if}
							</td>
							<td>
								{#if renderStatus(details.status) === 'Waiting Dissolvement' || renderStatus(details.status) === 'Waiting to Start Dissolving'}
									<button
										id="cancel-btn-mobile"
										title="test-withdrawal-{index}"
										on:click={() => {
											selectedWithdrawal.set(details);
											inCancelWarningMenu.set(true);
										}}
									>
										<RevertIcon />
									</button>
								{/if}
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
								{@html renderStatus(details.status)}
							</td>
							<td>{details.request.withdrawal_id}</td>
							<td>
								{#if renderStatus(details.status) === 'Waiting Dissolvement' || renderStatus(details.status) === 'Waiting to Start Dissolving'}
									<button
										id="cancel-btn"
										title="test-withdrawal-{index}"
										on:click={() => {
											selectedWithdrawal.set(details);
											inCancelWarningMenu.set(true);
										}}
									>
										Cancel
									</button>
								{/if}
							</td>
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
		background-color: var(--background-color);
		border: var(--input-border);
		border-radius: 10px;
		color: var(--stake-text-color);
		padding: 2em;
		display: flex;
		flex-direction: column;
		width: 44em;
		max-width: 97vw;
	}

	/* === Base Styles ==== */
	h1 {
		text-align: center;
		font-family: var(--main-font);
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
		padding: 1em 0.5em;
	}

	a {
		text-decoration: underline;
		color: var(--stake-text-color);
	}

	/* === Components === */
	#cancel-btn {
		background: var(--main-color);
		color: var(--main-button-text-color);
		border-radius: 8px;
		border: 2px solid black;
		font-size: 14px;
		box-shadow: 3px 3px 0 0 black;
		padding: 0 0.6em;
		height: 3em;
		font-weight: bold;
		display: flex;
		justify-content: center;
		align-items: center;
		cursor: pointer;
	}

	#cancel-btn:hover {
		transform: scale(0.95);
		transition: all 0.3s;
		box-shadow: 6px 6px 0 0 black;
	}

	#cancel-btn-mobile {
		border: none;
		background: none;
		display: flex;
		justify-content: center;
		align-items: center;
	}

	/* === Responsive === */
	@media (max-width: 767px) {
		.withdrawals-container {
			padding: 0em;
		}
	}
</style>
