import { AuthClient } from '@dfinity/auth-client';
import { HttpAgent, type Identity } from '@dfinity/agent';
import { idlFactory as idlFactoryIcrc } from '../declarations/icrc_ledger';
import type { _SERVICE as icrcLedgerInterface } from '../declarations/icrc_ledger/icrc_ledger.did';
import { idlFactory as idlFactoryIcp } from '../declarations/icp_ledger';
import type { _SERVICE as icpLedgerInterface } from '../declarations/icp_ledger/icp_ledger.did';
import { idlFactory as idlFactoryWaterNeuron } from '../declarations/water_neuron';
import type { _SERVICE as waterNeuronInterface } from '../declarations/water_neuron/water_neuron.did';
import { idlFactory as idlFactoryBoomerang } from '../declarations/boomerang';
import type { _SERVICE as boomerangInterface } from '../declarations/boomerang/boomerang.did';
import type { _SERVICE as icpswapPoolInterface } from '../declarations/icpswap_pool/icpswap_pool.did';
import { idlFactory as idlFactoryIcpswapPool } from '../declarations/icpswap_pool';
import { Signer } from '@slide-computer/signer';
import { PostMessageTransport } from '@slide-computer/signer-web';
import { user, canisters, availableAccounts, signer, ledgerDevice } from './stores';
import { CanisterActor, Canisters, User } from './state';
import { SignerAgent } from '@slide-computer/signer-agent';
import { PlugTransport } from '@slide-computer/signer-transport-plug';
import { Principal } from '@dfinity/principal';
import { LedgerDevice, LedgerIdentity } from './ledger-identity';
import { IcrcLedgerCanister } from '@dfinity/ledger-icrc';
import { LedgerCanister } from '@dfinity/ledger-icp';
import { Secp256k1PublicKey } from '@dfinity/identity-secp256k1';

// 1 hour in nanoseconds
const AUTH_MAX_TIME_TO_LIVE = BigInt(60 * 60 * 1000 * 1000 * 1000);

export const DEV = import.meta.env ? import.meta.env.DEV : true;
export const STAGING = process.env.CANISTER_ID === '47pxu-byaaa-aaaap-ahpsa-cai';

export const HOST = DEV ? 'http://127.0.1:8080' : 'https://ic0.app';
const DAPP_DERIVATION_ORIGIN = 'https://n3i53-gyaaa-aaaam-acfaq-cai.icp0.io';
const IDENTITY_PROVIDER = 'https://identity.ic0.app';

// export const OISY_RPC = 'https://oisy.com/sign' as const;
export const NFID_RPC = 'https://nfid.one/rpc' as const;

const CANISTER_ID_II = DEV ? 'iidmm-fiaaa-aaaaq-aadmq-cai' : 'rdmx6-jaaaa-aaaaa-aaadq-cai';
export const CANISTER_ID_WTN_LEDGER = 'jcmow-hyaaa-aaaaq-aadlq-cai';
export const CANISTER_ID_ICP_LEDGER = 'ryjl3-tyaaa-aaaaa-aaaba-cai';
export const CANISTER_ID_NICP_LEDGER = 'buwm7-7yaaa-aaaar-qagva-cai';
export const CANISTER_ID_BOOMERANG = 'daijl-2yaaa-aaaar-qag3a-cai';
export const CANISTER_ID_WATER_NEURON = 'tsbvt-pyaaa-aaaar-qafva-cai';
export const CANISTER_ID_ICPSWAP_POOL = 'e5a7x-pqaaa-aaaag-qkcga-cai';

export async function connectWithInternetIdentity() {
	try {
		const authClient = await AuthClient.create();

		if (await authClient.isAuthenticated()) {
			const identity = authClient.getIdentity();
			const agent = HttpAgent.createSync({
				identity,
				host: HOST
			});
			canisters.set(await fetchActors(agent));
			user.set(new User(identity.getPrincipal()));
		} else {
			await authClient.login({
				maxTimeToLive: AUTH_MAX_TIME_TO_LIVE,
				allowPinAuthentication: true,
				derivationOrigin: DAPP_DERIVATION_ORIGIN,
				identityProvider: IDENTITY_PROVIDER,
				onSuccess: async () => {
					const identity = authClient.getIdentity();
					const agent = HttpAgent.createSync({
						identity,
						host: HOST
					});
					canisters.set(await fetchActors(agent));
					user.set(new User(identity.getPrincipal()));
				},
				onError: (error) => {
					throw Error(error);
				}
			});
		}
	} catch (error) {
		console.error(error);
	}
}

export async function tryConnectOnReload() {
	const authClient = await AuthClient.create();

	if (await authClient.isAuthenticated()) {
		const identity = authClient.getIdentity();
		const agent = HttpAgent.createSync({
			identity,
			host: HOST
		});
		canisters.set(await fetchActors(agent));
		user.set(new User(identity.getPrincipal()));
	} else {
		canisters.set(await fetchActors());
	}
}

interface LoginWindow {
	ic: any;
}

declare global {
	interface Window extends LoginWindow {}
}

export async function connectWithPlug() {
	const transport = new PlugTransport();
	const newSigner = new Signer({ transport });

	await newSigner.requestPermissions([
		{
			method: 'icrc27_accounts'
		},
		{ method: 'icrc49_call_canister' }
	]);

	console.log('The wallet set the following permission scope:', await newSigner.permissions());

	const accounts = await newSigner.accounts();

	if (accounts.length > 1) {
		availableAccounts.set(accounts);
		signer.set(newSigner);
	} else {
		await finalizePlugConnection(newSigner, accounts[0].owner);
	}
}

export async function finalizePlugConnection(newSigner: Signer, userPrincipal: Principal) {
	const signerAgent = SignerAgent.createSync({
		signer: newSigner,
		account: userPrincipal
	});

	canisters.set(await fetchActors(signerAgent));
	user.set(new User(userPrincipal));
}

export async function connectWithTransport(rpc: typeof NFID_RPC) {
	try {
		const transport = new PostMessageTransport({
			url: rpc
		});

		const newSigner = new Signer({ transport });

		console.log('The wallet set the following permission scope:', await newSigner.permissions());

		const userPrincipal = (await newSigner.accounts())[0].owner;

		const signerAgent = SignerAgent.createSync({
			signer: newSigner,
			account: userPrincipal
		});

		canisters.set(await fetchActors(signerAgent));
		user.set(new User(userPrincipal));
	} catch (error) {
		console.log(error);
	}
}

export async function connectWithHardwareWallet() {
	const ledgerIdentity = await LedgerIdentity.create();
	const agent = HttpAgent.createSync({
		host: HOST
	});

	const authenticatedAgent = HttpAgent.createSync({
		identity: ledgerIdentity,
		host: HOST
	});

	const icpLedger = LedgerCanister.create({
		agent: authenticatedAgent,
		canisterId: Principal.fromText(CANISTER_ID_ICP_LEDGER)
	});

	const nicpLedger = IcrcLedgerCanister.create({
		agent: authenticatedAgent,
		canisterId: Principal.fromText(CANISTER_ID_NICP_LEDGER)
	});

	const wtnLedger = IcrcLedgerCanister.create({
		agent: authenticatedAgent,
		canisterId: Principal.fromText(CANISTER_ID_WTN_LEDGER)
	});

	ledgerDevice.set(
		new LedgerDevice({
			principal: await authenticatedAgent.getPrincipal(),
			identity: ledgerIdentity,
			agent,
			icpLedger,
			nicpLedger,
			wtnLedger
		})
	);
}

export async function localSignIn() {
	try {
		const authClient = await AuthClient.create();

		const identityProvider = DEV
			? `http://localhost:8080/?canisterId=${CANISTER_ID_II}`
			: IDENTITY_PROVIDER;
		await authClient.login({
			maxTimeToLive: AUTH_MAX_TIME_TO_LIVE,
			allowPinAuthentication: true,
			derivationOrigin: undefined,
			identityProvider,
			onSuccess: async () => {
				const identity: Identity = authClient.getIdentity();
				const agent = HttpAgent.createSync({
					identity,
					host: HOST
				});

				canisters.set(await fetchActors(agent));
				user.set(new User(identity.getPrincipal()));
			},
			onError: (error) => {
				throw new Error(error);
			}
		});
	} catch (error) {
		console.error(error);
	}
}

export async function testSignIn() {
	try {
		const authClient = await AuthClient.create();

		const identityProvider = `http://localhost:8080/?canisterId=${CANISTER_ID_II}`;
		await authClient.login({
			maxTimeToLive: AUTH_MAX_TIME_TO_LIVE,
			allowPinAuthentication: true,
			derivationOrigin: undefined,
			identityProvider,
			onSuccess: async () => {
				const identity: Identity = authClient.getIdentity();
				const agent = HttpAgent.createSync({
					identity,
					host: HOST
				});

				canisters.set(await fetchActors(agent));
				user.set(new User(identity.getPrincipal()));
			},
			onError: (error) => {
				throw new Error(error);
			}
		});
		const rawLedgerIdentity = new ArrayBuffer(65);
		const view = new Uint8Array(rawLedgerIdentity);
		view.set(Uint8Array.from('Test', (x) => x.charCodeAt(0)));
		const key = Secp256k1PublicKey.fromRaw(rawLedgerIdentity);
		const ledgerIdentity = LedgerIdentity.createMockIdentity(key);

		const ledgerAgent = HttpAgent.createSync({
			identity: ledgerIdentity,
			host: HOST
		});

		ledgerAgent.fetchRootKey().catch((err) => {
			console.warn('Unable to fetch root key. Check to ensure that your local replica is running');
			console.error(err);
		});

		const icpLedger = LedgerCanister.create({
			agent: ledgerAgent,
			canisterId: Principal.fromText(CANISTER_ID_ICP_LEDGER)
		});

		const nicpLedger = IcrcLedgerCanister.create({
			agent: ledgerAgent,
			canisterId: Principal.fromText(CANISTER_ID_NICP_LEDGER)
		});

		const wtnLedger = IcrcLedgerCanister.create({
			agent: ledgerAgent,
			canisterId: Principal.fromText(CANISTER_ID_WTN_LEDGER)
		});

		ledgerDevice.set(
			new LedgerDevice({
				principal: await ledgerAgent.getPrincipal(),
				identity: ledgerIdentity,
				agent: ledgerAgent,
				icpLedger,
				nicpLedger,
				wtnLedger
			})
		);
	} catch (error) {
		console.error(error);
	}
}

export async function internetIdentityLogout() {
	const autClient = await AuthClient.create();
	await autClient.logout();
}

export function fetchActors<T extends Pick<Signer, 'callCanister' | 'openChannel'>>(
	authenticatedAgent?: HttpAgent | SignerAgent<T>,
	isPlug = false
): Promise<Canisters> {
	return new Promise<Canisters>(async (resolve, reject) => {
		try {
			const agent = HttpAgent.createSync({
				host: HOST
			});

			if (DEV) {
				agent.fetchRootKey().catch((err) => {
					console.warn(
						'Unable to fetch root key. Check to ensure that your local replica is running'
					);
					console.error(err);
				});
			}

			const icpLedger = new CanisterActor<icpLedgerInterface>(
				agent,
				idlFactoryIcp,
				CANISTER_ID_ICP_LEDGER
			);
			const nicpLedger = new CanisterActor<icrcLedgerInterface>(
				agent,
				idlFactoryIcrc,
				CANISTER_ID_NICP_LEDGER
			);
			const wtnLedger = new CanisterActor<icrcLedgerInterface>(
				agent,
				idlFactoryIcrc,
				CANISTER_ID_WTN_LEDGER
			);
			const waterNeuron = new CanisterActor<waterNeuronInterface>(
				agent,
				idlFactoryWaterNeuron,
				CANISTER_ID_WATER_NEURON
			);
			const boomerang = new CanisterActor<boomerangInterface>(
				agent,
				idlFactoryBoomerang,
				CANISTER_ID_BOOMERANG
			);
			const icpswapPool = new CanisterActor<icpswapPoolInterface>(
				agent,
				idlFactoryIcpswapPool,
				CANISTER_ID_ICPSWAP_POOL
			);

			if (authenticatedAgent) {
				if (DEV && !isPlug) {
					authenticatedAgent.fetchRootKey().catch((err) => {
						console.warn(
							'Unable to fetch root key. Check to ensure that your local replica is running'
						);
						console.error(err);
					});
				}

				icpLedger.setAuthenticatedActor(authenticatedAgent);
				nicpLedger.setAuthenticatedActor(authenticatedAgent);
				wtnLedger.setAuthenticatedActor(authenticatedAgent);
				waterNeuron.setAuthenticatedActor(authenticatedAgent);
				boomerang.setAuthenticatedActor(authenticatedAgent);
				icpswapPool.setAuthenticatedActor(authenticatedAgent);
			}

			resolve(
				new Canisters({ icpLedger, nicpLedger, wtnLedger, waterNeuron, boomerang, icpswapPool })
			);
		} catch (error) {
			reject(error);
		}
	});
}
