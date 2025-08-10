import { HttpAgent } from '@dfinity/agent';
import { Ed25519KeyIdentity } from '@dfinity/identity';
import { User } from '$lib/actors';
import type {
	TransferArgs,
	Tokens,
	TransferArg
} from '../src/declarations/icp_ledger/icp_ledger.did';
import { AccountIdentifier } from '@dfinity/ledger-icp';
import { expect, Page } from '@playwright/test';
import { assetToTransferFee, getMaybeAccount } from '$lib';
import { registerActors } from '$lib/authentification';

const key = [
	'302a300506032b657003210093d488f46b485c07e09b554d9451574bfc669912b99d453722c474e6a7f90fcc',
	'90252a6913658dbb4b36b276410216d47a1891280493cd485328279a12a53e2c'
];

const parsedKey = JSON.stringify(key);

// The key is used to generate an intermediary account dispatching ICP/nICP tokens to testing accounts.
// AccountId = 90526bdfd692793cba1f96bde9079994ce4d40033746f04c12064ea599e2c274
// Principal = syna7-6ipnd-myx4g-ia46u-nxwok-u5nrr-yxgpi-iang7-lvru2-i7n23-tqe

export const mockSetup = async () => {
	const dummyIdentity = Ed25519KeyIdentity.fromJSON(parsedKey);

	const agent = HttpAgent.createSync({ host: 'http://127.0.1:8080', identity: dummyIdentity });
	agent.fetchRootKey().catch((err) => {
		console.warn('Unable to fetch root key. Check to ensure that your local replica is running');
		console.error(err);
	});

	return {
		mockCanisters: registerActors(agent),
		mockMintingAccount: new User(dummyIdentity.getPrincipal())
	};
};

export async function transferICP(accountString: string) {
	const { mockCanisters, mockMintingAccount } = await mockSetup();

	if (!(mockCanisters && mockMintingAccount))
		throw new Error('Mock user or mock canisters are undefined.');

	const maybeAccount = getMaybeAccount(accountString);

	if (!maybeAccount) throw new Error('Failed to decode account.');

	if (maybeAccount instanceof AccountIdentifier) {
		const result = await mockCanisters.icpLedger.authActor?.transfer({
			to: maybeAccount.toUint8Array(),
			fee: { e8s: assetToTransferFee('ICP') } as Tokens,
			memo: 0n,
			from_subaccount: [],
			created_at_time: [],
			amount: { e8s: 15n * 100_000_000n } as Tokens
		} as TransferArgs);

		if (!result || Object.keys(result)[0] === 'Err') throw new Error('Failed to transfer balance');
	} else {
		const result = await mockCanisters.icpLedger.authActor?.icrc1_transfer({
			to: maybeAccount,
			fee: [],
			memo: [],
			from_subaccount: [],
			created_at_time: [],
			amount: 15n * 100_000_000n
		} as TransferArg);

		if (!result || Object.keys(result)[0] === 'Err') throw new Error('Failed to transfer balance');
	}
}

export async function transferNICP(accountString: string) {
	const { mockCanisters, mockMintingAccount } = await mockSetup();

	if (!(mockCanisters && mockMintingAccount))
		throw new Error('Mock user or mock canisters are undefined.');

	const maybeAccount = getMaybeAccount(accountString);

	if (!maybeAccount) throw new Error('Failed to decode account.');
	if (maybeAccount instanceof AccountIdentifier)
		throw new Error('Acount ID provided. You should use principal.');
	const result = await mockCanisters.nicpLedger.authActor?.icrc1_transfer({
		to: maybeAccount,
		fee: [],
		memo: [],
		from_subaccount: [],
		created_at_time: [],
		amount: 15n * 100_000_000n
	} as TransferArg);

	if (!result || Object.keys(result)[0] === 'Err') throw new Error('Failed to transfer balance');
}

export async function swap(page: Page, amount: number, isUnstake?: boolean) {
	await page.locator('[title="swap-input"]').fill(amount.toString());
	await page.locator('[title="stake-unstake-btn"]').click();

	if (isUnstake) {
		await page.locator('[title="confirm-unstake-btn"]').click();
	}
}

export async function isToastSuccess(page: Page) {
	await page.waitForTimeout(2000);
	const message = await page.locator('p[title="toast-message"]').evaluate((msg) => msg.textContent);
	console.log(message);
	await page.locator('.toast-close').click();
	await expect(page.locator('p[title="toast-message"]')).not.toBeVisible();
	return message?.split(' ')[0].slice(0, 7) === 'Success';
}

export async function send(page: Page, destination: string, amount: string) {
	await page.locator('[title="send-destination"]').fill(destination);
	await page.locator('[title="send-amount"]').fill(amount);

	await page.locator('[title="continue-btn"]').click();
}
