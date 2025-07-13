<script lang="ts">
	import {
		displayNumber,
		numberToBigintE8s,
		getMaybeAccount,
		assetToIconPath,
		assetToTransferFee,
		assetToDashboardUrl,
		Toast as ToastMessage,
		bigintE8sToNumber
	} from '$lib';
	import {
		inSendingMenu,
		selectedAsset,
		ledgerDevice,
		user,
		toasts,
		canisters,
		inputAmount,
		handleInputAmount
	} from '$lib/stores';
	import { onMount } from 'svelte';
	import { AccountIdentifier, LedgerCanister } from '@dfinity/ledger-icp';
	import {
		handleIcrcTransferResult,
		handleTransferResult,
		type ToastResult
	} from '$lib/resultHandler';
	import type {
		Tokens,
		TransferArgs,
		TransferArg
	} from '$lib/../declarations/icp_ledger/icp_ledger.did';
	import type {
		Account,
		_SERVICE as icrcLedgerInterface
	} from '$lib/../declarations/icrc_ledger/icrc_ledger.did';
	import type { _SERVICE as icpLedgerInterface } from '$lib/../declarations/icp_ledger/icp_ledger.did';
	import { fade } from 'svelte/transition';
	import { IcrcLedgerCanister } from '@dfinity/ledger-icrc';
	import Toast from '../Toast.svelte';
	import ErrorIcon from '$lib/icons/ErrorIcon.svelte';

	let principal: string;
	let isSending = false;
	let dialog: HTMLDialogElement;
	let balance = 0;

	const fetchBalance = () => {
		if ($user?.account === 'ledger') {
			balance = $ledgerDevice?.getBalance($selectedAsset) ?? 0;
		} else {
			balance = $user?.getBalance($selectedAsset) ?? 0;
		}
	};

	async function handleTransferRequest(amount: number, accountString: string) {
		if (isSending || isNaN(amount) || balance <= amount || !principal || !$canisters || !$user)
			return;
		isSending = true;
		const amount_e8s = numberToBigintE8s(amount);
		const maybeAccount = getMaybeAccount(accountString);
		if (!maybeAccount) {
			isSending = false;
			return;
		}

		try {
			let status: ToastResult;
			switch ($selectedAsset) {
				case 'ICP':
					if (maybeAccount instanceof AccountIdentifier) {
						status = await icpTransfer(maybeAccount, amount_e8s);
					} else {
						if ($user?.account === 'main') {
							status = await icrcTransfer(
								maybeAccount,
								amount_e8s,
								$canisters.icpLedger.authActor,
								'ICP'
							);
						} else {
							status = await icrcLedgerWalletTransfer(
								maybeAccount,
								amount_e8s,
								$ledgerDevice?.icpLedger,
								'ICP'
							);
						}
					}
					break;
				case 'nICP':
					if (maybeAccount instanceof AccountIdentifier) {
						status = {
							success: false,
							message:
								'Transfer failed: nICP transfers require a principal. Please provide a valid principal.'
						};
					} else {
						if ($user.account === 'main') {
							status = await icrcTransfer(
								maybeAccount,
								amount_e8s,
								$canisters.nicpLedger.authActor,
								'nICP'
							);
						} else {
							status = await icrcLedgerWalletTransfer(
								maybeAccount,
								amount_e8s,
								$ledgerDevice?.nicpLedger,
								'ICP'
							);
						}
					}
					break;
				case 'WTN':
					if (maybeAccount instanceof AccountIdentifier) {
						status = {
							success: false,
							message:
								'Transfer failed: WTN transfers require a principal. Please provide a valid principal.'
						};
					} else {
						if ($user?.account === 'main') {
							status = await icrcTransfer(
								maybeAccount,
								amount_e8s,
								$canisters.wtnLedger.authActor,
								'WTN'
							);
						} else {
							status = await icrcLedgerWalletTransfer(
								maybeAccount,
								amount_e8s,
								$ledgerDevice?.wtnLedger,
								'ICP'
							);
						}
					}
					break;
			}
			if (status.success) {
				toasts.add(ToastMessage.success(status.message));
				dialog.close();
			} else {
				toasts.add(ToastMessage.error(status.message));
			}
		} catch (error) {
			console.error(error);
			toasts.add(ToastMessage.temporaryError('Transfer failed. Try again.'));
		}
		isSending = false;
		inputAmount.reset();
	}

	async function icpTransfer(
		to_account: AccountIdentifier,
		amount_e8s: bigint
	): Promise<ToastResult> {
		if (!$user) return { success: false, message: 'User is not authenticated.' };
		try {
			if ($user?.account === 'main') {
				if (!$canisters?.icpLedger.authActor)
					return { success: false, message: 'User is not authenticated.' };

				const args = {
					to: to_account.toUint8Array(),
					fee: { e8s: assetToTransferFee('ICP') } as Tokens,
					memo: 0n,
					from_subaccount: [],
					created_at_time: [],
					amount: { e8s: amount_e8s } as Tokens
				} as TransferArgs;
				const result = await $canisters?.icpLedger.authActor.transfer(args);
				return handleTransferResult(result);
			} else {
				if (!$ledgerDevice) return { success: false, message: 'Device is not connected.' };
				const blockHeight = await $ledgerDevice.icpLedger.transfer({
					to: to_account,
					amount: amount_e8s
				});
				return {
					success: true,
					message: `Successful transfer at <a target='_blank' style="text-decoration: underline; color: var(--toast-text-color);" href=https://dashboard.internetcomputer.org/transaction/${blockHeight}>block index ${blockHeight}</a>.`
				};
			}
		} catch (error) {
			console.error('[icpTransfer] ', error);
			return { success: false, message: 'Transfer failed. Please, try again.' };
		}
	}

	async function icrcLedgerWalletTransfer(
		to_account: Account,
		amount_e8s: bigint,
		ledger: IcrcLedgerCanister | LedgerCanister | undefined,
		asset: 'nICP' | 'ICP' | 'WTN'
	): Promise<ToastResult> {
		try {
			if (!ledger) return { success: false, message: 'Device is not connected.' };

			if (ledger instanceof LedgerCanister) {
				const blockHeight = await ledger.icrc1Transfer({
					to: to_account,
					amount: amount_e8s,
					fee: assetToTransferFee(asset),
					createdAt: BigInt(Date.now()) * BigInt(1e6)
				});

				return {
					success: true,
					message: `Successful transfer at <a target='_blank' style="text-decoration: underline; color: var(--toast-text-color);" href=${assetToDashboardUrl('ICP')}${blockHeight}>block index ${blockHeight}</a>.`
				};
			} else {
				const blockHeight = await ledger.transfer({
					to: to_account,
					amount: amount_e8s,
					fee: assetToTransferFee(asset),
					created_at_time: BigInt(Date.now()) * BigInt(1e6)
				});

				return {
					success: true,
					message: `Successful transfer at <a target='_blank' style="text-decoration: underline; color: var(--toast-text-color);" href=${assetToDashboardUrl(asset)}${blockHeight}>block index ${blockHeight}</a>.`
				};
			}
		} catch (error) {
			console.error('[icrcLedgerWalletTransfer] ', error);
			return { success: false, message: 'Transfer failed. Please, try again.' };
		}
	}

	async function icrcTransfer(
		to_account: Account,
		amount_e8s: bigint,
		ledger: icrcLedgerInterface | icpLedgerInterface | undefined,
		asset: 'nICP' | 'ICP' | 'WTN'
	): Promise<ToastResult> {
		try {
			if (!ledger) return { success: false, message: 'User is not authenticated.' };

			const icrcArgs = {
				to: to_account,
				fee: [],
				memo: [],
				from_subaccount: [],
				created_at_time: [],
				amount: amount_e8s
			} as TransferArg;

			const transferResult = await ledger.icrc1_transfer(icrcArgs);
			return handleIcrcTransferResult(transferResult, asset);
		} catch (error) {
			console.error('[icrcTransfer] ', error);
			return { success: false, message: 'Transfer failed. Please, try again.' };
		}
	}

	onMount(() => {
		dialog = document.getElementById('senderDialog') as HTMLDialogElement;
		dialog.showModal();

		fetchBalance();

		const intervalId = setInterval(async () => {
			fetchBalance();
		}, 5000);

		return () => clearInterval(intervalId);
	});
</script>

<dialog
	id="senderDialog"
	on:close={() => {
		inSendingMenu.set(false);
		inputAmount.reset();
	}}
>
	<div class="send-container" transition:fade={{ duration: 100 }}>
		<div class="header-container">
			<h2>Send {$selectedAsset}</h2>
			<img alt="Asset logo" src={assetToIconPath($selectedAsset)} width="50px" height="50px" />
		</div>
		{#if $user}
			<div>
				<p>Balance</p>
				<div style:display={'flex'}>
					<div class="balances">
						<span style:margin-left={'1em'}
							>{balance ? displayNumber(balance, 8) : '-/-'}
							{$selectedAsset}</span
						>
						<img
							alt="{$selectedAsset} logo"
							src={assetToIconPath($selectedAsset)}
							width="20px"
							height="20px"
						/>
					</div>
				</div>
			</div>
		{/if}
		<div>
			<p>Destination</p>
			<div class="wallet-input">
				<input type="text" placeholder="Address" title="send-destination" bind:value={principal} />
				{#if $ledgerDevice}
					<button
						class="placeholder-btn"
						title="destination-placeholder"
						on:click={() => {
							if ($user?.account === 'ledger') {
								principal = $user.principal.toString();
							} else {
								principal = $ledgerDevice.principal.toString();
							}
						}}
					>
						{$user?.account === 'ledger' ? 'Main' : 'Ledger Nano'}
					</button>
				{/if}
			</div>
			{#if principal && !getMaybeAccount(principal)}
				<span class="error" title="destination-error">
					<ErrorIcon /> Please enter a valid address.</span
				>
			{/if}
		</div>
		<div>
			<p>Amount</p>
			<div class="wallet-input">
				<input
					title="send-amount"
					type="text"
					maxlength="20"
					bind:value={$inputAmount}
					placeholder="Amount"
					on:input={handleInputAmount}
				/>
				<button
					class="placeholder-btn"
					title="max-placeholder"
					on:click={() => {
						const fee = assetToTransferFee($selectedAsset);
						const amount = Math.max(balance - bigintE8sToNumber(fee), 0);
						inputAmount.change(amount);
					}}
				>
					MAX
				</button>
			</div>
			<span class="error" title="amount-error">
				{#if !isNaN(parseFloat($inputAmount)) && parseFloat($inputAmount) > balance}
					<ErrorIcon /> You don't have enough funds to complete the transaction.
				{:else if !isNaN(parseFloat($inputAmount)) && parseFloat($inputAmount) <= bigintE8sToNumber(assetToTransferFee($selectedAsset))}
					<ErrorIcon /> Minimum amount: bigintE8sToNumber(assetToTransferFee($selectedAsset))
				{/if}
			</span>
		</div>
		<div>
			<p>Transfer Fee</p>
			<p style:padding-left="1em">
				{bigintE8sToNumber(assetToTransferFee($selectedAsset))}
				{$selectedAsset}
			</p>
		</div>
		<div class="button-container">
			{#if isSending}
				<button class="toggle-btn">
					<div class="spinner"></div>
				</button>
			{:else}
				<button
					id="abort-btn"
					class="toggle-btn"
					on:click={() => {
						dialog.close();
					}}>Back</button
				>
				<button
					id="continue-btn"
					class="toggle-btn"
					title="continue-btn"
					on:click={() => {
						handleTransferRequest(parseFloat($inputAmount), principal);
					}}
				>
					<span>Continue</span>
				</button>
			{/if}
		</div>
	</div>
	<Toast />
</dialog>

<style>
	/* === Base Styles === */

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

	input {
		color: var(--stake-text-color);
		height: 3em;
		border: none;
		font-size: 16px;
		background: none;
		outline: none;
		margin: 0 1em;
		flex-grow: 1;
	}

	p {
		font-family: var(--secondary-font);
	}

	span {
		font-family: var(--secondary-font);
	}

	button {
		color: var(--main-button-text-color);
	}

	/* === Layout === */
	.send-container {
		display: flex;
		flex-direction: column;
		height: fit-content;
		max-width: 35em;
		width: 80vw;
		background: var(--background-color);
		color: var(--stake-text-color);
		padding: 2em;
		border-radius: 15px;
		border: var(--main-container-border);
	}

	.header-container {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0 2%;
		font-family: var(--secondary-font);
	}

	.button-container {
		display: flex;
		justify-content: end;
		gap: 1em;
	}

	/* === Componennts === */
	.error {
		display: flex;
		align-items: center;
		color: var(--text-color);
		gap: 0.2em;
		margin-left: 1em;
		margin-top: 4px;
		font-size: 16px;
		font-family: var(--secondary-font);
		flex-wrap: wrap;
		font-size: 14px;
	}

	.placeholder-btn {
		text-align: right;
		padding-right: 1em;
		background: none;
		color: var(--stake-text-color);
		border: none;
		cursor: pointer;
	}

	.wallet-input {
		display: flex;
		align-items: center;
		justify-content: space-between;
		border: var(--main-container-border);
		background: var(--input-color);
		border-radius: 0.4em;
	}

	.balances {
		display: flex;
		align-items: center;
		gap: 0.5em;
	}

	.toggle-btn {
		background: var(--main-color);
		min-width: 80px;
		border-radius: 8px;
		position: relative;
		border: var(--main-container-border);
		font-size: 14px;
		padding: 0 1em 0 1em;
		max-width: none;
		height: 3em;
		font-weight: bold;
		display: flex;
		justify-content: center;
		align-items: center;
		cursor: pointer;
	}

	.toggle-btn:hover {
		background: var(--main-color-hover);
		transition: all 0.2s;
	}

	#abort-btn {
		background: var(--main-button-text-color);
		color: black;
	}

	/* #continue-btn {
		background: var(--main-color);
		color: var(--main-button-text-color);
	} */

	input::-webkit-outer-spin-button,
	input::-webkit-inner-spin-button {
		-webkit-appearance: none;
		margin: 0;
	}

	/* === Animation === */

	.spinner {
		width: 1em;
		height: 1em;
		border: 3px solid var(--main-button-text-color);
		border-top-color: transparent;
		border-radius: 50%;
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}
</style>
