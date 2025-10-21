<script lang="ts">
	import { CanisterActor } from '$lib/actors';
	import type { _SERVICE as icrcLedgerInterface } from '../../declarations/icrc_ledger/icrc_ledger.did';
	import { idlFactory as idlFactoryIcrc } from '../../declarations/icrc_ledger';
	import { agent, user } from '$lib/stores';
	import { Principal } from '@dfinity/principal';
	import { get } from 'svelte/store';

	let loading = false;
	let canisterID: string = '';
	let recoveryPrincipalString: string = '';
	let recoveryPrincipal: Principal | undefined = undefined;
	let ledgerName: string = '';
	let balance: bigint | undefined = undefined;
	let decimals: number = 0;
	let fee: bigint | undefined = undefined;
	let ledger: CanisterActor<icrcLedgerInterface> | undefined = undefined;
	let error: string = '';

	$: {
		try {
			loading = true;
			ledger = new CanisterActor<icrcLedgerInterface>({
				idl: idlFactoryIcrc,
				canisterId: canisterID
			});
			ledger.connectWith(get(agent)!);
			ledger.anonymousActor.icrc1_symbol().then((name) => {
				ledgerName = name;
			});
			ledger.anonymousActor.icrc1_decimals().then((dec) => {
				decimals = dec;
			});
			ledger.anonymousActor.icrc1_fee().then((f) => {
				fee = f;
			});
			ledger.anonymousActor
				.icrc1_balance_of({ owner: $user!.principal, subaccount: [] })
				.then((b) => {
					balance = b;
				});
		} catch (e) {
			ledger = undefined;
			ledgerName = '';
			balance = undefined;
			decimals = 0;
			fee = undefined;
			error = (e as Error).message;
		} finally {
			loading = false;
			error = '';
		}
	}

	$: recoveryPrincipal = recoveryPrincipalString
		? (() => {
				try {
					return Principal.fromText(recoveryPrincipalString);
				} catch {
					return undefined;
				}
			})()
		: undefined;

	async function sendRecovery() {
		try {
			loading = true;
			console.log(ledger?.authActor);
			if (ledger && balance !== undefined && fee !== undefined) {
				const transferResult = await ledger.authActor!.icrc1_transfer({
					to: {
						owner: recoveryPrincipal!,
						subaccount: []
					},
					from_subaccount: [],
					amount: balance - fee,
					memo: [],
					fee: [fee!],
					created_at_time: []
				});
				console.log('Transfer Result:', transferResult);
				if ('Err' in transferResult) {
					error = `Transfer failed: ${JSON.stringify(transferResult.Err)}`;
				}
			} else {
				throw new Error('Ledger not initialized or balance/fee undefined.');
			}
		} catch (error) {
			error = (error as Error).message;
		} finally {
			loading = false;
		}
	}
</script>

{#if !$user}
	<p>You need to log in first.</p>
{:else}
	<div>
		<label for="canisterID">ICRC Ledger Canister ID:</label>
		<input id="canisterID" type="text" placeholder="Ledger Canister ID" bind:value={canisterID} />
		{#if loading}
			<p>Loading...</p>
		{:else}
			{#if balance && decimals !== 0}
				<p>Your Balance: {(Number(balance) / 10 ** decimals).toString()} {ledgerName}</p>
			{:else if decimals === 0}
				<p>Please enter a valid ICRC Ledger Canister ID. Failed to get decimals.</p>
			{:else if fee === undefined}
				<p>Please enter a valid ICRC Ledger Canister ID. Failed to get fee.</p>
			{:else}
				<p>No balance found.</p>
			{/if}
			<br />
			{#if balance && fee}
				<p style="font-weight: bold">
					Make sure to double-check your recovery account before proceeding.
				</p>
				<label for="recoveryPrincipal">Recovery Principal:</label>
				<input
					id="recoveryPrincipal"
					type="text"
					placeholder="Recovery Principal"
					bind:value={recoveryPrincipalString}
				/>
				{#if recoveryPrincipal}
					<p>Sending {Number(balance - fee) / 10 ** decimals} to {recoveryPrincipal.toString()}.</p>
					<button on:click={sendRecovery}>Send</button>
				{/if}
			{/if}
		{/if}
		{#if error}
			<p style="color: red;">Error: {error}</p>
		{/if}
	</div>
{/if}

<style>
	label,
	p {
		padding: 0;
		margin: 0.5em 0;
		color: var(--stake-text-color);
	}

	input {
		width: 100%;
	}
</style>
