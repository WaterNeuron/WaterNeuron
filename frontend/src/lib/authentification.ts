import { AuthClient } from '@dfinity/auth-client';
import { Principal } from '@dfinity/principal';
import { HttpAgent, Actor, type Identity } from '@dfinity/agent';
import type { _SERVICE as icrcLedgerInterface } from '../declarations/icrc_ledger/icrc_ledger.did';
import type { _SERVICE as icpLedgerInterface } from '../declarations/nns-ledger/nns-ledger.did';
import type {
	CanisterInfo,
	_SERVICE as waterNeuronInterface
} from '../declarations/water_neuron/water_neuron.did';
import { idlFactory as idlFactoryIcrc } from '../declarations/icrc_ledger';
import { idlFactory as idlFactoryWaterNeuron } from '../declarations/water_neuron';
import { idlFactory as idlFactoryIcp } from '../declarations/nns-ledger';

// 1 hour in nanoseconds
const AUTH_MAX_TIME_TO_LIVE = BigInt(60 * 60 * 1000 * 1000 * 1000);

export const DEV = import.meta.env.DEV;
const INTERNET_IDENTITY_CANISTER_ID = DEV
	? 'qhbym-qaaaa-aaaaa-aaafq-cai'
	: 'rdmx6-jaaaa-aaaaa-aaadq-cai';

export const HOST = DEV ? 'http://127.0.1:8080' : 'https://ic0.app';

const CANISTER_ID_WTN_LEDGER = 'jcmow-hyaaa-aaaaq-aadlq-cai';
const CANISTER_ID_ICP_LEDGER = 'ryjl3-tyaaa-aaaaa-aaaba-cai';
const CANISTER_ID_NICP_LEDGER = DEV ? 'ny7ez-6aaaa-aaaam-acc5q-cai' : 'buwm7-7yaaa-aaaar-qagva-cai';
export const CANISTER_ID_WATER_NEURON = DEV
	? 'n76cn-tyaaa-aaaam-acc5a-cai'
	: 'tsbvt-pyaaa-aaaar-qafva-cai';

export interface AuthResult {
	actors: Actors;
	principal: Principal;
}

export interface Actors {
	icpLedger: icpLedgerInterface;
	nicpLedger: icrcLedgerInterface;
	wtnLedger: icrcLedgerInterface;
	waterNeuron: waterNeuronInterface;
	wtnCanisterInfo: CanisterInfo;
}
export async function internetIdentitySignIn(): Promise<AuthResult> {
	return new Promise<AuthResult>(async (resolve, reject) => {
		try {
			const authClient = await AuthClient.create();
			if (!(await authClient.isAuthenticated())) {
				const identityProvider = import.meta.env.DEV
					? `http://127.0.1:8080/?canisterId=${INTERNET_IDENTITY_CANISTER_ID}`
					: `https://identity.${'ic0.app'}`;

				const authClient = await AuthClient.create();

				const derivation = DEV ? undefined : 'https://n3i53-gyaaa-aaaam-acfaq-cai.icp0.io';
				await authClient.login({
					maxTimeToLive: AUTH_MAX_TIME_TO_LIVE,
					allowPinAuthentication: true,
					derivationOrigin: derivation,
					onSuccess: async () => {
						const identity: Identity = authClient?.getIdentity();
						const agent = new HttpAgent({
							identity,
							host: HOST
						});
						const actors = await fetchActors(agent, true);
						resolve({
							actors,
							principal: identity.getPrincipal()
						});
					},
					onError: (error) => {
						console.log('invalid');
						reject(error);
					},
					identityProvider
				});
			} else {
				const identity: Identity = authClient?.getIdentity();
				const agent = new HttpAgent({
					identity,
					host: HOST
				});

				const actors = await fetchActors(agent, true);

				resolve({
					actors,
					principal: identity.getPrincipal()
				});
			}
		} catch (error) {
			reject(error);
		}
	});
}

interface LoginWindow {
	ic: any;
}

declare global {
	interface Window extends LoginWindow {}
}

export async function plugSignIn(): Promise<AuthResult> {
	return new Promise<AuthResult>(async (resolve, reject) => {
		try {
			if (!(await window.ic.plug.isConnected())) {
				let whitelist: string[] = [
					CANISTER_ID_ICP_LEDGER,
					CANISTER_ID_NICP_LEDGER,
					CANISTER_ID_WTN_LEDGER,
					CANISTER_ID_WATER_NEURON
				];
				const onConnectionUpdate = () => {
					console.log(window.ic.plug.sessionManager.sessionData);
				};

				await window.ic.plug.requestConnect({
					whitelist,
					host: HOST,
					onConnectionUpdate,
					timeout: 50000
				});
			}

			const principal: Principal = await window.ic.plug.getPrincipal();
			const agent = window.ic.plug.agent;

			const actors = await fetchActors(agent);

			resolve({ actors, principal });
		} catch (error) {
			reject(error);
		}
	});
}

export async function internetIdentityLogout() {
	const autClient = await AuthClient.create();
	await autClient.logout();
}

export function fetchActors(agent?: HttpAgent, isInternetIdentity?: boolean): Promise<Actors> {
	return new Promise<Actors>(async (resolve, reject) => {
		try {
			if (!agent) {
				agent = new HttpAgent({
					host: HOST
				});
			}
			if (process.env.DFX_NETWORK !== 'ic' && isInternetIdentity) {
				console.log('fetching root key');
				agent.fetchRootKey().catch((err) => {
					console.warn(
						'Unable to fetch root key. Check to ensure that your local replica is running'
					);
					console.error(err);
				});
			}

			const icpLedger: icpLedgerInterface = Actor.createActor(idlFactoryIcp, {
				agent,
				canisterId: CANISTER_ID_ICP_LEDGER
			});

			const nicpLedger: icrcLedgerInterface = Actor.createActor(idlFactoryIcrc, {
				agent,
				canisterId: CANISTER_ID_NICP_LEDGER
			});
			const wtnLedger: icrcLedgerInterface = Actor.createActor(idlFactoryIcrc, {
				agent,
				canisterId: CANISTER_ID_WTN_LEDGER
			});
			const waterNeuron: waterNeuronInterface = Actor.createActor(idlFactoryWaterNeuron, {
				agent,
				canisterId: CANISTER_ID_WATER_NEURON
			});

			const wtnCanisterInfo = await waterNeuron.get_info();

			resolve({ icpLedger, wtnLedger, nicpLedger, waterNeuron, wtnCanisterInfo });
		} catch (error) {
			reject(error);
		}
	});
}
