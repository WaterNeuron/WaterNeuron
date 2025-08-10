import { Principal } from '@dfinity/principal';
import { Secp256k1PublicKey } from '@dfinity/identity-secp256k1';
import {
	Cbor,
	HttpAgent,
	requestIdOf,
	SignIdentity,
	type CallRequest,
	type HttpAgentRequest,
	type PublicKey,
	type ReadRequest,
	type ReadStateRequest,
	type ReadRequestType,
	type RequestId,
	type Signature
} from '@dfinity/agent';
import { smallerVersion } from '@dfinity/utils';
import type Transport from '@ledgerhq/hw-transport';
import LedgerApp from '@zondax/ledger-icp';
import type { ResponseAddress, ResponseSign, ResponseSignUpdateCall } from '@zondax/ledger-icp';
import { AccountIdentifier, LedgerCanister } from '@dfinity/ledger-icp';
import { bigintE8sToNumber } from '$lib';
import { IcrcLedgerCanister } from '@dfinity/ledger-icrc';
import { LedgerError, type ResponseVersion } from '@zondax/ledger-js';

export const LEDGER_DEFAULT_DERIVE_PATH = `m/44'/223'/0'/0/0`;
const LEDGER_SIGNATURE_LENGTH = 64;
// Version published in October 2023. Includes all transactions supported in Candid
export const ALL_CANDID_TXS_VERSION = '2.4.9';

type RequestSignatures = {
	callSignature: Signature;
	readStateSignature: Signature;
};

interface LedgerHQTransportError {
	name: string;
	message: string;
	id: string;
}

export class LedgerDevice {
	public identity: LedgerIdentity;
	public principal: Principal;
	public accountId: string;
	public icpLedger: LedgerCanister;
	public nicpLedger: IcrcLedgerCanister;
	public wtnLedger: IcrcLedgerCanister;
	public icpBalanceE8s: bigint;
	public nicpBalanceE8s: bigint;
	public wtnBalanceE8s: bigint;

	constructor({
		principal,
		identity,
		icpLedger,
		nicpLedger,
		wtnLedger
	}: {
		principal: Principal;
		identity: LedgerIdentity;
		agent: HttpAgent;
		icpLedger: LedgerCanister;
		nicpLedger: IcrcLedgerCanister;
		wtnLedger: IcrcLedgerCanister;
	}) {
		this.identity = identity;
		this.principal = principal;
		this.icpLedger = icpLedger;
		this.nicpLedger = nicpLedger;
		this.wtnLedger = wtnLedger;
		this.accountId = AccountIdentifier.fromPrincipal({ principal }).toHex();
		this.icpBalanceE8s = 0n;
		this.nicpBalanceE8s = 0n;
		this.wtnBalanceE8s = 0n;
	}

	icpBalance(): number {
		return bigintE8sToNumber(this.icpBalanceE8s);
	}

	nicpBalance(): number {
		return bigintE8sToNumber(this.nicpBalanceE8s);
	}

	wtnBalance(): number {
		return bigintE8sToNumber(this.wtnBalanceE8s);
	}

	getBalance(asset: 'ICP' | 'nICP' | 'WTN'): number {
		switch (asset) {
			case 'ICP':
				return this.icpBalance();
			case 'nICP':
				return this.nicpBalance();
			case 'WTN':
				return this.wtnBalance();
		}
	}
}

export class LedgerIdentity extends SignIdentity {
	private readonly derivePath: string;
	private readonly publicKey: Secp256k1PublicKey;

	private constructor(derivePath: string, publicKey: Secp256k1PublicKey) {
		super();
		this.derivePath = derivePath;
		this.publicKey = publicKey;
	}

	public static async create(): Promise<LedgerIdentity> {
		const { app, transport } = await this.connect();

		try {
			const publicKey = await this.fetchPublicKeyFromDevice({
				app,
				derivePath: LEDGER_DEFAULT_DERIVE_PATH
			});

			return new this(LEDGER_DEFAULT_DERIVE_PATH, publicKey);
		} finally {
			await transport.close();
		}
	}

	// For tests only
	public static createMockIdentity(mockIdentity: Secp256k1PublicKey): LedgerIdentity {
		return new this(LEDGER_DEFAULT_DERIVE_PATH, mockIdentity);
	}

	public override getPublicKey(): Required<PublicKey> {
		return this.publicKey;
	}

	private async executeWithApp<T>(callback: (app: LedgerApp) => Promise<T>): Promise<T> {
		const { app, transport } = await LedgerIdentity.connect();

		try {
			const devicePublicKey: Secp256k1PublicKey = await LedgerIdentity.fetchPublicKeyFromDevice({
				app,
				derivePath: this.derivePath
			});

			if (JSON.stringify(devicePublicKey) !== JSON.stringify(this.publicKey)) {
				throw new Error('The identity in use does not match the device identity.');
			}

			return await callback(app);
		} finally {
			await transport.close();
		}
	}

	// Required by Ledger.com that the user should be able to press a Button in UI
	// and verify the address/pubkey are the same as on the device screen.
	public async showAddressAndPubKeyOnDevice(): Promise<ResponseAddress> {
		const callback = (app: LedgerApp): Promise<ResponseAddress> =>
			app.showAddressAndPubKey(this.derivePath);

		return this.executeWithApp<ResponseAddress>(callback);
	}

	// Check device target ids: https://github.com/LedgerHQ/blue-loader-python?tab=readme-ov-file#target-id
	public async getVersion(): Promise<ResponseVersion> {
		const callback = async (app: LedgerApp): Promise<ResponseVersion> => app.getVersion();
		return this.executeWithApp<ResponseVersion>(callback);
	}

	private raiseIfVersionIsDeprecated = async () => {
		const { major, minor, patch } = await this.getVersion();
		const currentVersion = `${major}.${minor}.${patch}`;
		if (smallerVersion({ minVersion: ALL_CANDID_TXS_VERSION, currentVersion })) {
			throw new Error('The app version is deprecated.');
		}
	};

	public static async getTransport(): Promise<Transport> {
		const { default: TransportWebHID } = await import('@ledgerhq/hw-transport-webhid');
		return TransportWebHID.create();
	}

	private static async connect(): Promise<{
		app: LedgerApp;
		transport: Transport;
	}> {
		try {
			const transport = await this.getTransport();

			const { default: LedgerAppConstructor } = await import('@zondax/ledger-icp');
			const app = new LedgerAppConstructor(transport);

			return { app, transport };
		} catch (err: unknown) {
			if ((err as LedgerHQTransportError)?.name === 'TransportOpenUserCancelled') {
				if ((err as LedgerHQTransportError)?.message.includes('not supported')) {
					throw new Error('The browser is not supported.');
				}
				throw new Error('Connection failed. Access denied to the ledger.');
			}

			if ((err as LedgerHQTransportError)?.id === 'NoDeviceFound') {
				throw new Error('Connection failed. No device found.');
			}

			if ((err as LedgerHQTransportError).message?.includes('cannot open device with path')) {
				throw new Error('Connection failed. Several devices detected.');
			}

			throw new Error(`Unknown error: ${err}`);
		}
	}

	public async transformRequest(request: HttpAgentRequest): Promise<unknown> {
		if (request.endpoint === 'call') {
			return this.transformCallRequest(request);
		}

		const { body, ...fields } = request;
		return {
			...fields,
			body: {
				content: body,
				sender_pubkey: this.publicKey.toDer(),
				sender_sig: await this.sign(prepareCborForLedger(body))
			}
		};
	}

	private async transformCallRequest(request: HttpAgentRequest) {
		const { body, ...fields } = request;
		const callBody = body as CallRequest;
		const readStateBody = createReadStateRequest(callBody);
		const signatures = await this.signCallRequest(
			prepareCborForLedger(body),
			prepareCborForLedger(readStateBody)
		);

		return {
			...fields,
			body: {
				content: body,
				sender_pubkey: this.publicKey.toDer(),
				sender_sig: signatures.callSignature
			}
		};
	}

	private async signCallRequest(
		callBlob: ArrayBuffer,
		readStateBlob: ArrayBuffer
	): Promise<RequestSignatures> {
		await this.raiseIfVersionIsDeprecated();

		const callback = async (app: LedgerApp): Promise<RequestSignatures> => {
			const responseSign: ResponseSignUpdateCall = await app.signUpdateCall(
				this.derivePath,
				Buffer.from(callBlob),
				Buffer.from(readStateBlob),
				0 // isNeuronStaking ? 1 : 0
			);

			return decodeUpdateSignatures(responseSign);
		};

		return this.executeWithApp<RequestSignatures>(callback);
	}

	public override async sign(blob: ArrayBuffer): Promise<Signature> {
		await this.raiseIfVersionIsDeprecated();

		const callback = async (app: LedgerApp): Promise<Signature> => {
			const responseSign: ResponseSign = await app.sign(
				this.derivePath,
				Buffer.from(blob),
				0 // isNeuronStaking ? 1 : 0
			);

			return decodeSignature(responseSign);
		};

		return this.executeWithApp<Signature>(callback);
	}

	private static async fetchPublicKeyFromDevice({
		app,
		derivePath
	}: {
		app: LedgerApp;
		derivePath: string;
	}): Promise<Secp256k1PublicKey> {
		const { principalText, publicKey } = await app.getAddressAndPubKey(derivePath);

		if (!publicKey) {
			throw new Error('Failed to fetch public key from device.');
		}
		const secp256Key = Secp256k1PublicKey.fromRaw(bufferToArrayBuffer(publicKey));

		if (
			principalText !== Principal.selfAuthenticating(new Uint8Array(secp256Key.toDer())).toText()
		) {
			throw new Error('Principal returned by device does not match public key.');
		}

		return secp256Key;
	}
}

const checkResponseCode = async (returnCode: LedgerError): Promise<void> => {
	const { LedgerError } = await import('@zondax/ledger-js');
	if (returnCode === LedgerError.TransactionRejected) {
		throw new Error('User rejected transaction.');
	}
};

const bufferToArrayBuffer = (buffer: Buffer<ArrayBufferLike> | undefined): ArrayBuffer => {
	if (!buffer) return new ArrayBuffer();

	return buffer.buffer.slice(
		buffer.byteOffset,
		buffer.byteOffset + buffer.byteLength
	) as ArrayBuffer;
};

export const prepareCborForLedger = (request: ReadRequest | CallRequest): ArrayBuffer => {
	return Cbor.encode({ content: request });
};

export function createReadStateRequest(body: CallRequest): ReadStateRequest {
	const requestId = requestIdOf(body);
	const readStateBody: ReadStateRequest = {
		request_type: 'read_state' as ReadRequestType.ReadState,
		paths: createReadStatePaths(requestId),
		ingress_expiry: body.ingress_expiry,
		sender: body.sender
	};
	return readStateBody;
}

export const decodeUpdateSignatures = async ({
	RequestSignatureRS,
	StatusReadSignatureRS,
	returnCode,
	errorMessage
}: ResponseSignUpdateCall): Promise<RequestSignatures> => {
	await checkResponseCode(returnCode);
	checkSignature({ signature: RequestSignatureRS, returnCode, errorMessage });
	checkSignature({
		signature: StatusReadSignatureRS,
		returnCode,
		errorMessage
	});

	return {
		callSignature: bufferToArrayBuffer(RequestSignatureRS) as Signature,
		readStateSignature: bufferToArrayBuffer(StatusReadSignatureRS) as Signature
	};
};

export const decodeSignature = async ({
	signatureRS,
	returnCode,
	errorMessage
}: ResponseSign): Promise<Signature> => {
	await checkResponseCode(returnCode);
	checkSignature({ signature: signatureRS, returnCode, errorMessage });

	return bufferToArrayBuffer(signatureRS) as Signature;
};

const checkSignature = ({
	signature,
	returnCode,
	errorMessage
}: {
	signature?: Buffer;
	returnCode: LedgerError;
	errorMessage?: string;
}) => {
	if (!signature) {
		throw new Error(`Signature not provided (${returnCode}): ${errorMessage}`);
	}

	const { byteLength, length } = signature;

	if (byteLength !== LEDGER_SIGNATURE_LENGTH) {
		throw new Error(`Signature has length ${length} instead of ${LEDGER_SIGNATURE_LENGTH}.`);
	}
};

export const createReadStatePaths = (requestId: RequestId): ArrayBuffer[][] => {
	return [[new TextEncoder().encode('request_status'), requestId]] as ArrayBuffer[][];
};
