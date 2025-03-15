import { writable } from 'svelte/store';
import { type User, Canisters, WaterNeuronInfo, fetchIcpBalance, fetchNicpBalance } from './state';
import { bigintE8sToNumber } from '$lib';
import { Toast } from './toast';
import BigNumber from 'bignumber.js';
import { get } from 'svelte/store';
import { Principal } from '@dfinity/principal';
import { encodeIcrcAccount } from '@dfinity/ledger-icrc';
import type { WithdrawalDetails } from '../declarations/water_neuron/water_neuron.did';
import { Signer } from '@slide-computer/signer';
import { LedgerDevice } from './ledger-identity';

/* === Flags === */
export const isLogging = writable<boolean>(false);
export const isBusy = writable<boolean>(false);
export const inSendingMenu = writable<boolean>(false);
export const inReceivingMenu = writable<boolean>(false);
export const inCancelWarningMenu = writable<boolean>(false);
export const inUnstakeWarningMenu = writable<boolean>(false);
export const inMobileMenu = writable<boolean>(false);
export const inQrDestination = writable<boolean>(false);
export const inSnsMenu = writable<boolean>(false);

/* === Components === */
export const language = writable<'en' | 'es' | 'ja' | 'ru'>('en');
export const availableAccounts = writable<{ owner: Principal; subaccount?: ArrayBuffer }[]>([]);
export const signer = writable<Signer | undefined>(undefined);
export const selectedAsset = writable<'ICP' | 'nICP' | 'WTN'>('ICP');
export const selectedWithdrawal = writable<WithdrawalDetails | undefined>(undefined);
export const user = writable<User | undefined>(undefined);
export const ledgerDevice = writable<LedgerDevice | undefined>(undefined);
export const canisters = writable<Canisters | undefined>(undefined);
export const waterNeuronInfo = writable<WaterNeuronInfo | undefined>(undefined);

/* === Input Amount ==== */
function createInputAmountStore() {
	const { subscribe, set } = writable<string>();

	return {
		subscribe,
		change: (value: number) => {
			const input = value.toString().replace(',', '.');
			set(input);
		},
		set: (value: string) => set(value),
		reset: () => set('')
	};
}

export const inputAmount = createInputAmountStore();

export function handleInputAmount(event: Event): void {
	const target = event.target as HTMLInputElement;
	const number = target.value;
	const regex = /^[0-9]*([\.][0-9]*)?$/;

	if (regex.test(number)) {
		inputAmount.set(number);
	} else {
		const newNumber = number.substring(0, number.length - 1);
		inputAmount.set(newNumber);
		target.value = newNumber;
	}
}

/* === SNS === */
function createBoomerangSnsStore() {
	const { subscribe, set, update } = writable<{
		name: string;
		principal: string;
		encodedBoomerangAccount: string | undefined;
		icpBalance: BigNumber | undefined;
		nicpBalance: BigNumber | undefined;
	}>({
		name: '',
		principal: '',
		encodedBoomerangAccount: undefined,
		icpBalance: undefined,
		nicpBalance: undefined
	});

	return {
		subscribe,
		setPrincipal: (principal: string) => update((sns) => ({ ...sns, principal })),
		setName: (name: string) => update((sns) => ({ ...sns, name })),
		setEncodedBoomerangAccount: (encodedBoomerangAccount: string) =>
			update((sns) => ({ ...sns, encodedBoomerangAccount })),
		setIcpBalance: (icpBalance: BigNumber) => update((sns) => ({ ...sns, icpBalance })),
		setNicpBalance: (nicpBalance: BigNumber) => update((sns) => ({ ...sns, nicpBalance })),
		reset: () =>
			set({
				name: '',
				principal: '',
				encodedBoomerangAccount: undefined,
				icpBalance: undefined,
				nicpBalance: undefined
			})
	};
}
export const sns = createBoomerangSnsStore();

export const handleSnsChange = async (name?: string, principal?: string) => {
	const fetchedCanisters = get(canisters);
	if (!fetchedCanisters) return;

	sns.reset();
	inputAmount.reset();

	try {
		if (name && principal) {
			sns.setName(name);
			const p = Principal.fromText(principal);
			sns.setPrincipal(principal);
			const [account, icpBalanceE8s, nicpBalanceE8s] = await Promise.all([
				fetchedCanisters.boomerang.anonymousActor.get_staking_account(p),
				fetchIcpBalance(p, fetchedCanisters.icpLedger.anonymousActor),
				fetchNicpBalance(p, fetchedCanisters.nicpLedger.anonymousActor)
			]);
			const encodedBoomerangAccount = encodeIcrcAccount({
				owner: account.owner,
				subaccount: account.subaccount[0]
			});
			sns.setEncodedBoomerangAccount(encodedBoomerangAccount);
			sns.setIcpBalance(bigintE8sToNumber(icpBalanceE8s));
			sns.setNicpBalance(bigintE8sToNumber(nicpBalanceE8s));
		} else {
			sns.setName('Custom');
			sns.setPrincipal('');
		}
	} catch (error) {
		console.log(error);
	}
};

/* === Toasts === */
function createToasts() {
	const { subscribe, set, update } = writable<Toast[]>([]);

	return {
		subscribe,
		add: (toast: Toast) => update((toasts: Toast[]) => [...toasts, toast]),
		remove: (id: string) => update((toasts: Toast[]) => toasts.filter((toast) => toast.id !== id)),
		reset: () => set([])
	};
}

export const toasts = createToasts();
