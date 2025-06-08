import { test, expect } from '@playwright/test';
import {
	LedgerError,
	ResponseSignUpdateCall,
	type ResponseAddress,
	type ResponseSign
} from '@zondax/ledger-icp';
import { decodeSignature, decodeUpdateSignatures } from '$lib/ledger-identity';

test('decodeSignature', () => {
	const call1 = () =>
		decodeSignature({
			signatureRS: undefined,
			returnCode: LedgerError.UnknownError
		} as unknown as ResponseSign);

	expect(call1).rejects.toThrow('Signature not provided (28416): undefined');

	const call2 = () =>
		decodeSignature({
			signatureRS: Uint8Array.from('test', (x) => x.charCodeAt(0)),
			returnCode: LedgerError.TransactionRejected
		} as unknown as ResponseSign);

	expect(call2).rejects.toThrow('User rejected transaction.');

	const call3 = () =>
		decodeSignature({
			signatureRS: Uint8Array.from(
				'0410d34980a51af891211d3331ad5fa80fe30d8868ad87526460b3b3e15596ee58e8',
				(x) => x.charCodeAt(0)
			),
			returnCode: LedgerError.NoErrors
		} as unknown as ResponseSign);

	expect(call3).rejects.toThrow('Signature has length 68 instead of 64.');

	const signature = decodeSignature({
		signatureRS: Uint8Array.from(
			'0410d34980a51af89d3331ad5fa80fe30d8868ad87526460b3b3e15596ee58e8',
			(x) => x.charCodeAt(0)
		),
		returnCode: LedgerError.NoErrors
	} as unknown as ResponseSign);

	expect(signature).not.toBeNull();
});

test('decodeUpdateSignatures', () => {
	const call1 = () =>
		decodeUpdateSignatures({
			StatusReadSignatureRS: undefined,
			RequestSignatureRS: undefined,
			returnCode: LedgerError.UnknownError
		} as unknown as ResponseSignUpdateCall);

	expect(call1).rejects.toThrow('Signature not provided (28416): undefined');

	const call2 = () =>
		decodeUpdateSignatures({
			StatusReadSignatureRS: Uint8Array.from('test', (x) => x.charCodeAt(0)),
			RequestSignatureRS: Uint8Array.from('test', (x) => x.charCodeAt(0)),
			returnCode: LedgerError.TransactionRejected
		} as unknown as ResponseSignUpdateCall);

	expect(call2).rejects.toThrow('User rejected transaction.');

	const call3 = () =>
		decodeUpdateSignatures({
			StatusReadSignatureRS: Uint8Array.from(
				'0410d34980a51af89d3331ad5fa80fe30d8868ad87526460b3b3e15596ee58e8',
				(x) => x.charCodeAt(0)
			),
			RequestSignatureRS: Uint8Array.from(
				'041023dd34980a51af89d3331ad5fa80fe30d8868ad87526460b3b3e15596ee58e8',
				(x) => x.charCodeAt(0)
			),
			returnCode: LedgerError.UnknownError
		} as unknown as ResponseSignUpdateCall);

	expect(call3).rejects.toThrow('Signature has length 67 instead of 64.');

	const signature = decodeUpdateSignatures({
		StatusReadSignatureRS: Uint8Array.from(
			'0410d34980a51af89d3331ad5fa80fe30d8868ad87526460b3b3e15596ee58e8',
			(x) => x.charCodeAt(0)
		),
		RequestSignatureRS: Uint8Array.from(
			'0410d34980a51af89d3331ad5fa80fe30d8868ad87526460b3b3e15596ee58e8',
			(x) => x.charCodeAt(0)
		),
		returnCode: LedgerError.NoErrors
	} as unknown as ResponseSignUpdateCall);
	expect(signature).not.toBeNull();
});
