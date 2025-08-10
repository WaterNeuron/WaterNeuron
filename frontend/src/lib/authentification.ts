import { AuthClient } from '@dfinity/auth-client';
import { type Agent, HttpAgent, type Identity } from '@dfinity/agent';
import { Signer } from '@slide-computer/signer';
import { PostMessageTransport } from '@slide-computer/signer-web';
import { user, canisters, availableAccounts, signer, ledgerDevice } from './stores';
import { Canisters, User } from './actors';
import { SignerAgent } from '@slide-computer/signer-agent';
import { Principal } from '@dfinity/principal';
import { LedgerDevice, LedgerIdentity } from './ledger-identity';
import { IcrcLedgerCanister } from '@dfinity/ledger-icrc';
import { LedgerCanister } from '@dfinity/ledger-icp';
import { Secp256k1PublicKey } from '@dfinity/identity-secp256k1';
import { BrowserExtensionTransport } from '@slide-computer/signer-extension';
import {
	CANISTER_ID_ICP_LEDGER,
	CANISTER_ID_II,
	CANISTER_ID_NICP_LEDGER,
	CANISTER_ID_WTN_LEDGER,
	DAPP_DERIVATION_ORIGIN,
	DEV,
	HOST,
	IDENTITY_PROVIDER,
	NFID_RPC,
	OISY_RPC
} from './env';
import { get } from 'svelte/store';

export function registerActors(agent: Agent): Canisters {
	let store = get(canisters);
	store.waterNeuron.connectWith(agent);
	store.icpLedger.connectWith(agent);
	store.nicpLedger.connectWith(agent);
	store.wtnLedger.connectWith(agent);
	store.icpswap.connectWith(agent);
	store.boomerang.connectWith(agent);
	return store;
}

// 1 hour in nanoseconds
const AUTH_MAX_TIME_TO_LIVE = BigInt(60 * 60 * 1000 * 1000 * 1000);

export async function connectWithInternetIdentity() {
	const authClient = await AuthClient.create();

	if (await authClient.isAuthenticated()) {
		const identity = authClient.getIdentity();
		const agent = HttpAgent.createSync({
			identity,
			host: HOST
		});
		canisters.set(registerActors(agent));
		user.set(new User(identity.getPrincipal()));
	} else {
		return new Promise((resolve, reject) => {
			authClient.login({
				maxTimeToLive: AUTH_MAX_TIME_TO_LIVE,
				allowPinAuthentication: true,
				derivationOrigin: DAPP_DERIVATION_ORIGIN,
				identityProvider: IDENTITY_PROVIDER,
				onSuccess: () => {
					const identity = authClient.getIdentity();
					const agent = HttpAgent.createSync({
						identity,
						host: HOST
					});
					canisters.set(registerActors(agent));
					user.set(new User(identity.getPrincipal()));
					resolve(null);
				},
				onError: (error) => {
					reject(error);
				}
			});
		});
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
		canisters.set(registerActors(agent));
		user.set(new User(identity.getPrincipal()));
	}
}

interface LoginWindow {
	ic: any;
}

declare global {
	interface Window extends LoginWindow {}
}

// export async function connectWithPlug() {
// 	const transport = new PlugTransport();
// 	const newSigner = new Signer({ transport });

// 	await newSigner.requestPermissions([
// 		{
// 			method: 'icrc27_accounts'
// 		},
// 		{ method: 'icrc49_call_canister' }
// 	]);

// 	console.log('The wallet set the following permission scope:', await newSigner.permissions());

// 	const accounts = await newSigner.accounts();

// 	if (accounts.length > 1) {
// 		availableAccounts.set(accounts);
// 		signer.set(newSigner);
// 	} else {
// 		await finalizePlugConnection(newSigner, accounts[0].owner);
// 	}
// }

export function finalizePlugConnection(newSigner: Signer, userPrincipal: Principal) {
	const signerAgent = SignerAgent.createSync({
		signer: newSigner,
		account: userPrincipal
	});

	canisters.set(registerActors(signerAgent));
	user.set(new User(userPrincipal));
}

export async function connectWithExtension() {
	try {
		const transport = await BrowserExtensionTransport.findTransport({
			uuid: '71edc834-bab2-4d59-8860-c36a01fee7b8'
		});

		const newSigner = new Signer({ transport });
		const accounts = await newSigner.accounts();

		if (accounts.length > 1) {
			availableAccounts.set(accounts);
			signer.set(newSigner);
		} else {
			finalizePlugConnection(newSigner, accounts[0].owner);
		}
	} catch (e) {
		console.error(e);
	}
}

export async function connectWithTransport(rpc: typeof NFID_RPC | typeof OISY_RPC) {
	const transport = new PostMessageTransport({
		url: rpc,
		detectNonClickEstablishment: false,
		closeOnEstablishTimeout: true,
		establishTimeout: 120000,
		disconnectTimeout: 120000
	});

	const newSigner = new Signer({ transport });

	const userPrincipal = (await newSigner.accounts())[0].owner;

	const signerAgent = SignerAgent.createSync({
		signer: newSigner,
		account: userPrincipal
	});

	canisters.set(registerActors(signerAgent));
	user.set(new User(userPrincipal));
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
	const authClient = await AuthClient.create();

	const identityProvider = DEV
		? `http://localhost:8080/?canisterId=${CANISTER_ID_II}`
		: IDENTITY_PROVIDER;
	return new Promise((resolve, reject) => {
		authClient.login({
			maxTimeToLive: AUTH_MAX_TIME_TO_LIVE,
			allowPinAuthentication: true,
			derivationOrigin: undefined,
			identityProvider,
			onSuccess: () => {
				const identity: Identity = authClient.getIdentity();
				const agent = HttpAgent.createSync({
					identity,
					host: HOST
				});

				canisters.set(registerActors(agent));
				user.set(new User(identity.getPrincipal()));
				resolve(null);
			},
			onError: (error) => {
				reject(error);
			}
		});
	});
}

export async function testSignIn() {
	const authClient = await AuthClient.create();

	const identityProvider = `http://localhost:8080/?canisterId=${CANISTER_ID_II}`;
	return new Promise((resolve, reject) => {
		authClient.login({
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

				canisters.set(registerActors(agent));
				user.set(new User(identity.getPrincipal()));

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
					console.warn(
						'Unable to fetch root key. Check to ensure that your local replica is running'
					);
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

				resolve(null);
			},
			onError: (error) => {
				reject(error);
			}
		});
	});
}

export async function internetIdentityLogout() {
	const autClient = await AuthClient.create();
	await autClient.logout();
}
