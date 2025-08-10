import { test, expect } from '@playwright/test';
import { testWithII } from '@dfinity/internet-identity-playwright';
import { mockSetup, transferICP, swap, isToastSuccess, transferNICP, send } from './utils';

const VALID_PRINCIPAL = 'l72el-pt5ry-lmj66-3opyw-tl5xx-3wzfl-n3mja-dqirc-oxmqs-uxqe6-6qe';
const ACCOUNT_ID = 'e73a99617af2a8dbfe9b75e463e83a905e30aa50250972ad19c21922c22b2a2a';
const VALID_ACCOUNT =
	'daijl-2yaaa-aaaar-qag3a-cai-clltauq.5f0e93000f4cbd9db8c36d27cad8b8a97706c0710154172029e54541e18fd180';

test('Intermediary account should have balance', async () => {
	const { mockCanisters, mockMintingAccount } = await mockSetup();

	if (!(mockCanisters && mockMintingAccount))
		throw new Error('Mock user or mock canisters are undefined.');
	const icpBalance = await mockCanisters.icpLedger.authActor?.icrc1_balance_of({
		owner: mockMintingAccount.principal,
		subaccount: []
	});

	const nicpBalance = await mockCanisters.nicpLedger.authActor?.icrc1_balance_of({
		owner: mockMintingAccount.principal,
		subaccount: []
	});

	console.log('ICP balance of mock minting account', mockMintingAccount.accountId, ':', icpBalance);
	console.log(
		'nICP balance of mock minting account',
		mockMintingAccount.principal.toString(),
		':',
		nicpBalance
	);
	expect(icpBalance && nicpBalance && icpBalance > 0n && nicpBalance > 0n).toBeTruthy();
});

test('has title', async ({ page }) => {
	await page.goto('/');

	await expect(page).toHaveTitle('WaterNeuron | ICP Liquid Staking');
});

testWithII('test page navigation', async ({ page, iiPage }) => {
	await page.goto('/');
	await expect(page).toHaveURL('/');

	await page.goto('/any');
	await expect(page).toHaveURL('/');

	await page.goto('/sns/');
	await expect(page).toHaveURL('/sns/');

	await page.goto('/wallet/');
	await expect(page).toHaveURL('/');

	await page.locator('[title="connect-btn"]').click();

	await iiPage.signInWithNewIdentity({ selector: '[title="ii-connect-btn"]' });

	await page.locator('#disconnect-btn').click();
});

testWithII('e2e test stake', async ({ page, iiPage }) => {
	await page.goto('/');

	await page.locator('[title="connect-btn"]').click();

	await iiPage.signInWithNewIdentity({ selector: '[title="ii-connect-btn"]' });

	const walletInfo = page.locator('#wallet-info');
	await expect(walletInfo).toBeVisible();

	await walletInfo.click();

	const accountId = await page.locator('p[title="accountIdentifier-hex"]').textContent();

	if (!accountId) throw new Error('No account id found.');

	await transferICP(accountId);
	await expect(walletInfo.locator('[title="icp-balance-nav"]')).toHaveText('15 ICP');

	await page.locator('[title="home-btn"]').click();

	await swap(page, 0.001);
	await expect(page.locator('span.error')).toBeVisible();

	await swap(page, 15);
	expect(await isToastSuccess(page)).toBeFalsy();

	await page.locator('.max-btn').click();
	const maxAmountStake = parseFloat(
		(await page
			.locator('[title="swap-input"]')
			.evaluate((input) => (input as HTMLInputElement).value)) ?? '0'
	);
	await swap(page, maxAmountStake);
	expect(await isToastSuccess(page)).toBeTruthy();
});

testWithII('e2e test unstake', async ({ page, iiPage }) => {
	await page.goto('/');

	await page.locator('[title="connect-btn"]').click();

	await iiPage.signInWithNewIdentity({ selector: '[title="ii-connect-btn"]' });

	const walletInfo = page.locator('#wallet-info');
	await expect(walletInfo).toBeVisible();

	await walletInfo.click();
	await expect(page.locator('.withdrawals-container')).not.toBeVisible();

	const principal = await page.locator('p[title="principal-user"]').textContent();

	if (!principal) throw new Error('No account id found.');

	await transferNICP(principal);
	await expect(walletInfo.locator('[title="nicp-balance-nav"]')).toHaveText('15 nICP');

	await page.locator('[title="home-btn"]').click();
	await page.locator('[title="unstake-header"]').click();
	await page.locator('[title="delayed-btn"]').click();

	await page.locator('[title="swap-input"]').fill('9');
	await expect(page.locator('span.error')).toBeVisible();

	await swap(page, 15, true);
	expect(await isToastSuccess(page)).toBeFalsy();

	await page.locator('.max-btn').click();
	const maxAmountUnstake = parseFloat(
		(await page
			.locator('[title="swap-input"]')
			.evaluate((input) => (input as HTMLInputElement).value)) ?? '0'
	);
	await swap(page, maxAmountUnstake, true);
	expect(await isToastSuccess(page)).toBeTruthy();

	await walletInfo.click();
	await expect(page.locator('.withdrawals-container')).toBeVisible();
});

testWithII('e2e test send', async ({ page, iiPage }) => {
	await page.goto('/');

	await page.locator('[title="connect-btn"]').click();

	await iiPage.signInWithNewIdentity({ selector: '[title="ii-connect-btn"]' });

	const walletInfo = page.locator('#wallet-info');
	await expect(walletInfo).toBeVisible();

	await walletInfo.click();

	const accountId = await page.locator('p[title="accountIdentifier-hex"]').textContent();

	if (!accountId) throw new Error('No account id found.');

	const principal = await page.locator('p[title="principal-user"]').textContent();

	if (!principal) throw new Error('No principal found.');

	await transferICP(accountId);
	await transferNICP(principal);

	const icpBalance = walletInfo.locator('[title="icp-balance-nav"]');
	const nicpBalance = walletInfo.locator('[title="nicp-balance-nav"]');

	await expect(icpBalance).toHaveText('15 ICP');
	await expect(nicpBalance).toHaveText('15 nICP');
	await page.locator('[title="send-btn-ICP"]').click();

	await send(page, 'aaa-aa', '10');
	await expect(page.locator('[title="destination-error"]')).toBeVisible();
	await send(page, ACCOUNT_ID, '16');
	await expect(page.locator('[title="amount-error"]')).toBeVisible();

	await send(page, ACCOUNT_ID, '1');
	expect(await isToastSuccess(page)).toBeTruthy();
	await expect(icpBalance).toHaveText('13.99 ICP');

	await page.locator('[title="send-btn-ICP"]').click();
	await send(page, VALID_ACCOUNT, '1');
	expect(await isToastSuccess(page)).toBeTruthy();
	await expect(icpBalance).toHaveText('12.99 ICP');

	await page.locator('[title="send-btn-ICP"]').click();
	await page.locator('[title="max-placeholder"]').click();
	const maxAmountSendIcp = parseFloat(
		(await page
			.locator('[title="send-amount"]')
			.evaluate((input) => (input as HTMLInputElement).value)) ?? '0'
	);
	await send(page, VALID_PRINCIPAL, maxAmountSendIcp.toString());
	expect(await isToastSuccess(page)).toBeTruthy();
	await expect(icpBalance).toHaveText('0 ICP');

	await page.locator('[title="send-btn-nICP"]').click();
	await send(page, ACCOUNT_ID, '10');
	expect(await isToastSuccess(page)).toBeFalsy();

	await send(page, VALID_ACCOUNT, '1');
	expect(await isToastSuccess(page)).toBeTruthy();
	await expect(nicpBalance).toHaveText('13.99 nICP');

	await page.locator('[title="send-btn-nICP"]').click();
	await page.locator('[title="max-placeholder"]').click();
	const maxAmountSendNicp = parseFloat(
		(await page
			.locator('[title="send-amount"]')
			.evaluate((input) => (input as HTMLInputElement).value)) ?? '0'
	);
	await send(page, VALID_PRINCIPAL, maxAmountSendNicp.toString());
	expect(await isToastSuccess(page)).toBeTruthy();
});

// This test is expected to work only with one worker (otherwise one test impact the others).
// Use npx playwright test --prokect=chromium (you can use firefox or webkit too).
test('e2e test sns', async ({ page }) => {
	await page.goto('/sns');
	await expect(page.locator('.sns-listing')).toBeVisible();

	await page.waitForTimeout(3000);

	const encodedAccountLocator = page.locator('.principal-container').locator('p');
	expect(await encodedAccountLocator.evaluate((p) => p.textContent)).not.toBe('-/-');

	const encodedAccount = await encodedAccountLocator.textContent();
	if (!encodedAccount) throw new Error('Invalid encoded account');

	await page.locator('[title="notifyIcpDeposit-btn"]').click();
	expect(await isToastSuccess(page)).toBeFalsy();

	await page.locator('[title="retrieveNicp-btn"]').click();
	expect(await isToastSuccess(page)).toBeFalsy();

	transferICP(encodedAccount);
	await page.waitForTimeout(5000);

	await page.locator('[title="notifyIcpDeposit-btn"]').click();
	expect(await isToastSuccess(page)).toBeTruthy();

	await page.locator('[title="retrieveNicp-btn"]').click();
	expect(await isToastSuccess(page)).toBeTruthy();
});

testWithII('e2e test cancel withdrawal', async ({ page, iiPage }) => {
	await page.goto('/');

	await page.locator('[title="connect-btn"]').click();

	await iiPage.signInWithNewIdentity({ selector: '[title="ii-connect-btn"]' });

	const walletInfo = page.locator('#wallet-info');
	await expect(walletInfo).toBeVisible();

	await walletInfo.click();
	await expect(page.locator('.withdrawals-container')).not.toBeVisible();

	const principal = await page.locator('p[title="principal-user"]').textContent();

	if (!principal) throw new Error('No account id found.');

	await transferNICP(principal);
	await transferNICP(principal);
	await expect(walletInfo.locator('[title="nicp-balance-nav"]')).toHaveText('30 nICP');

	await page.locator('[title="home-btn"]').click();

	await page.locator('[title="unstake-header"]').click();
	await page.locator('[title="delayed-btn"]').click();

	await page.waitForTimeout(5000);
	await swap(page, 10, true);
	expect(await isToastSuccess(page)).toBeTruthy();

	await walletInfo.click();
	await expect(page.locator('.withdrawals-container')).toBeVisible();
	await page.waitForTimeout(5000);
	await page.locator('[title="test-withdrawal-0"]').click();

	await page.locator('[title="test-cancel-confirmation"]').click();
	expect(await isToastSuccess(page)).toBeTruthy();

	await page.locator('[title="home-btn"]').click();
	await page.locator('[title="unstake-header"]').click();
	await page.locator('[title="delayed-btn"]').click();
	await swap(page, 10, true);
	expect(await isToastSuccess(page)).toBeTruthy();

	await walletInfo.click();
	await expect(page.locator('.withdrawals-container')).toBeVisible();

	await page.locator('[title="test-withdrawal-0"]').click();
	await page.locator('[title="test-cancel-confirmation"]').click();
	expect(await isToastSuccess(page)).toBeTruthy();
});

testWithII('test ledger hardware wallet interaction', async ({ page, iiPage }) => {
	await page.goto('/');

	await page.locator('[title="connect-btn"]').click();

	await iiPage.signInWithNewIdentity({ selector: '[title="ii-connect-btn"]' });

	const walletInfo = page.locator('#wallet-info');
	await expect(walletInfo).toBeVisible();

	await walletInfo.click();

	const userPrincipal = await page.locator('p[title="principal-user"]').textContent();

	await page.locator('[title="send-btn-ICP"]').click();

	const placeholder = page.locator('[title="destination-placeholder"]');
	expect(await placeholder.textContent()).toBe('Ledger Nano');
	await placeholder.click();
	const ledgerDestination =
		(await page
			.locator('[title="send-destination"]')
			.evaluate((input) => (input as HTMLInputElement).value)) ?? '';

	await page.locator('#abort-btn').click();
	await page.locator('[title="switch-ledger-btn"]').click();

	const principal = await page.locator('p[title="principal-user"]').textContent();
	const accountId = await page.locator('p[title="accountIdentifier-hex"]').textContent();

	expect(principal).toBe('bqfvl-vwbmq-wjhyu-l24zd-mqeaw-or6ji-avorb-ozacl-wuj5j-tviiu-oae');
	expect(ledgerDestination).toBe(principal);
	expect(accountId).toBe('bc5e6a28697c5766dbd322f40105d3cb5a044a79070af91dabb675c643ce4722');

	await page.waitForTimeout(100);
	await page.locator('[title="send-btn-ICP"]').click();

	expect(await placeholder.textContent()).toBe('Main');
	await placeholder.click();
	const userDestination =
		(await page
			.locator('[title="send-destination"]')
			.evaluate((input) => (input as HTMLInputElement).value)) ?? '';

	expect(userPrincipal).toBe(userDestination);
});
