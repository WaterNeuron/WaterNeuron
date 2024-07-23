import { AssetType, bigintE8sToNumber } from '$lib';
import { AccountIdentifier, type Account } from '@dfinity/ledger-icp';
import { Principal } from '@dfinity/principal';
import BigNumber from 'bignumber.js';
import type { _SERVICE as icrcLedgerInterface } from '../declarations/icrc_ledger/icrc_ledger.did';
import type { _SERVICE as icpLedgerInterface } from '../declarations/nns-ledger/nns-ledger.did';
import type {
	CanisterInfo,
	_SERVICE as waterNeuronInterface
} from '../declarations/water_neuron/water_neuron.did';
import { type AuthResult, fetchActors, type Actors } from './authentification';
import { state, user } from './stores';
import { internetIdentitySignIn, plugSignIn } from '$lib/authentification';
import { AuthClient } from '@dfinity/auth-client';

interface UserProps {
	principal: Principal;
	icpBalanceE8s: bigint;
	nicpBalanceE8s: bigint;
	wtnBalanceE8s: bigint;
}

export class User {
	public principal: Principal;
	public accountId: string;
	public icpBalanceE8s: bigint;
	public nicpBalanceE8s: bigint;
	public wtnBalanceE8s: bigint;

	constructor(props: UserProps) {
		this.principal = props.principal;
		this.accountId = AccountIdentifier.fromPrincipal({ principal: props.principal }).toHex();
		this.icpBalanceE8s = props.icpBalanceE8s;
		this.nicpBalanceE8s = props.nicpBalanceE8s;
		this.wtnBalanceE8s = props.wtnBalanceE8s;
	}

	icpBalance(): BigNumber {
		return bigintE8sToNumber(this.icpBalanceE8s);
	}

	nicpBalance(): BigNumber {
		return bigintE8sToNumber(this.nicpBalanceE8s);
	}

	wtnBalance(): BigNumber {
		return bigintE8sToNumber(this.wtnBalanceE8s);
	}

	getBalance(asset: AssetType): BigNumber {
		switch (asset) {
			case AssetType.ICP:
				return this.icpBalance();
			case AssetType.nICP:
				return this.nicpBalance();
			case AssetType.WTN:
				return this.wtnBalance();
		}
	}
}

const DAO_SHARE = BigNumber(0.1);
const APY_6M = BigNumber(0.08);
const APY_8Y = BigNumber(0.15);

export async function signIn(walletOrigin: 'internetIdentity' | 'plug' | 'reload') {
	try {
		let authResult: AuthResult;

		switch (walletOrigin) {
			case 'internetIdentity':
				authResult = await internetIdentitySignIn();
				break;
			case 'plug':
				authResult = await plugSignIn();
				break;
			case 'reload':
				const authClient = await AuthClient.create();
				if (!(await authClient.isAuthenticated())) return;
				authResult = await internetIdentitySignIn();
				break;
		}

		state.set(new State(authResult.actors));

		user.set(
			new User({
				principal: authResult.principal,
				icpBalanceE8s: 0n,
				nicpBalanceE8s: 0n,
				wtnBalanceE8s: 0n
			})
		);
	} catch (error) {
		console.error('Login failed:', error);
	}
}

export async function fetchBalances(
	principal: Principal,
	nicpLedger: icrcLedgerInterface,
	wtnLedger: icrcLedgerInterface,
	icpLedger: icpLedgerInterface
): Promise<{ icp: bigint; nicp: bigint; wtn: bigint }> {
	const user_account: Account = {
		owner: principal,
		subaccount: []
	};
	const nicpBalanceE8s = await nicpLedger.icrc1_balance_of(user_account);
	const wtnBalanceE8s = await wtnLedger.icrc1_balance_of(user_account);
	const icpBalanceE8s = await icpLedger.icrc1_balance_of(user_account);

	return { icp: icpBalanceE8s, nicp: nicpBalanceE8s, wtn: wtnBalanceE8s };
}

export class State {
	public neuron8yStakeE8s: bigint;
	public neuron6mStakeE8s: bigint;
	public icpLedger: icpLedgerInterface;
	public wtnLedger: icrcLedgerInterface;
	public nicpLedger: icrcLedgerInterface;
	public waterNeuron: waterNeuronInterface;
	public wtnCanisterInfo: CanisterInfo;

	constructor(actors: Actors) {
		this.neuron8yStakeE8s = BigInt(0);
		this.neuron6mStakeE8s = BigInt(0);
		this.nicpLedger = actors.nicpLedger;
		this.wtnLedger = actors.wtnLedger;
		this.icpLedger = actors.icpLedger;
		this.waterNeuron = actors.waterNeuron;
		this.wtnCanisterInfo = actors.wtnCanisterInfo;
	}

	totalIcpDeposited(): BigNumber {
		const neuron6mStake = this.neuron6mStake();
		const neuron8yStake = this.neuron8yStake();
		return neuron6mStake.plus(neuron8yStake);
	}

	neuron8yStake(): BigNumber {
		return bigintE8sToNumber(this.wtnCanisterInfo.neuron_8y_stake_e8s);
	}

	neuron6mStake(): BigNumber {
		return bigintE8sToNumber(this.wtnCanisterInfo.neuron_6m_stake_e8s);
	}

	exchangeRate(): BigNumber {
		return bigintE8sToNumber(this.wtnCanisterInfo.exchange_rate);
	}

	async wtnAllocation(): Promise<BigNumber> {
		try {
			const allocation = await this.waterNeuron.get_airdrop_allocation();
			return bigintE8sToNumber(allocation);
		} catch (e) {
			return BigNumber(0);
		}
	}

	apy(): BigNumber {
		const neuron6mStake = this.neuron6mStake();
		const neuron8yStake = this.neuron8yStake();

		if (neuron6mStake.plus(neuron8yStake).isZero()) return BigNumber(0);

		const amount6m = APY_6M.multipliedBy(neuron6mStake);
		const amount8y = APY_8Y.multipliedBy(neuron8yStake);
		const amountTotal = amount6m.plus(amount8y);
		const share = BigNumber(1).minus(DAO_SHARE);

		return share.multipliedBy(amountTotal).multipliedBy(BigNumber(1)).dividedBy(neuron6mStake);
	}

	stakersCount(): Number {
		return Number(this.wtnCanisterInfo.stakers_count);
	}
}

export async function provideState(): Promise<State> {
	let actors = await fetchActors(undefined, true);
	return new State(actors);
}
