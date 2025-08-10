import { type Account, AccountIdentifier, SubAccount } from '@dfinity/ledger-icp';
import { IDL } from '@dfinity/candid';
import { Actor, HttpAgent, type Agent } from '@dfinity/agent';
import type { _SERVICE as icrcLedgerInterface } from '../declarations/icrc_ledger/icrc_ledger.did';
import type { _SERVICE as icpLedgerInterface } from '../declarations/icp_ledger/icp_ledger.did';
import type { _SERVICE as icpswapInterface } from '../declarations/icpswap_pool/icpswap_pool.did';
import type { _SERVICE as boomerangInterface } from '../declarations/boomerang/boomerang.did';
import type {
	CanisterInfo,
	_SERVICE as waterNeuronInterface
} from '../declarations/water_neuron/water_neuron.did';
import { idlFactory as idlFactoryIcrc } from '../declarations/icrc_ledger';
import { idlFactory as idlFactoryIcp } from '../declarations/icp_ledger';
import { idlFactory as idlFactoryicpswap } from '../declarations/icpswap_pool';
import { idlFactory as idlFactoryBoomerang } from '../declarations/boomerang';
import { idlFactory as idlFactoryWaterNeuron } from '../declarations/water_neuron';
import { Principal } from '@dfinity/principal';
import { bigintE8sToNumber } from '$lib';
import {
	CANISTER_ID_BOOMERANG,
	CANISTER_ID_ICP_LEDGER,
	CANISTER_ID_ICPSWAP_POOL,
	CANISTER_ID_NICP_LEDGER,
	CANISTER_ID_WATER_NEURON,
	CANISTER_ID_WTN_LEDGER,
	DEV,
	HOST
} from './env';

export class User {
	public principal: Principal;
	public accountId: string;
	public account: 'main' | 'ledger';
	public icpBalanceE8s: bigint;
	public nicpBalanceE8s: bigint;
	public wtnBalanceE8s: bigint;
	public wtnAllocationE8s: bigint;

	constructor(principal: Principal) {
		this.principal = principal;
		this.account = 'main';
		this.accountId = AccountIdentifier.fromPrincipal({ principal }).toHex();
		this.icpBalanceE8s = 0n;
		this.nicpBalanceE8s = 0n;
		this.wtnBalanceE8s = 0n;
		this.wtnAllocationE8s = 0n;
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

	wtnAllocation(): number {
		return bigintE8sToNumber(this.wtnAllocationE8s);
	}

	getBalance(asset: 'ICP' | 'WTN' | 'nICP'): number {
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

export class CanisterActor<T> {
	public authActor?: T;
	public anonymousActor: T;
	public _canisterId: string;
	private _idl: IDL.InterfaceFactory;

	constructor(arg: { idl: IDL.InterfaceFactory; canisterId: string }) {
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

		this.anonymousActor = Actor.createActor(arg.idl, {
			agent,
			canisterId: arg.canisterId
		});
		this._canisterId = arg.canisterId;
		this._idl = arg.idl;
	}

	connectWith(agent: Agent) {
		if (DEV) {
			agent.fetchRootKey().catch((err) => {
				console.warn('Unable to fetch root key. Check to ensure that the local replica is running');
				console.error(err);
			});
		}
		this.authActor = Actor.createActor(this._idl, {
			agent,
			canisterId: this._canisterId
		});
	}
}

export class Canisters {
	public icpLedger = new CanisterActor<icpLedgerInterface>({
		idl: idlFactoryIcp,
		canisterId: CANISTER_ID_ICP_LEDGER
	});
	public nicpLedger = new CanisterActor<icrcLedgerInterface>({
		idl: idlFactoryIcrc,
		canisterId: CANISTER_ID_NICP_LEDGER
	});
	public wtnLedger = new CanisterActor<icrcLedgerInterface>({
		idl: idlFactoryIcrc,
		canisterId: CANISTER_ID_WTN_LEDGER
	});
	public icpswap = new CanisterActor<icpswapInterface>({
		idl: idlFactoryicpswap,
		canisterId: CANISTER_ID_ICPSWAP_POOL
	});
	public boomerang = new CanisterActor<boomerangInterface>({
		idl: idlFactoryBoomerang,
		canisterId: CANISTER_ID_BOOMERANG
	});
	public waterNeuron = new CanisterActor<waterNeuronInterface>({
		idl: idlFactoryWaterNeuron,
		canisterId: CANISTER_ID_WATER_NEURON
	});

	constructor() {}
}

export async function fetchBalance(
	ledger: icrcLedgerInterface | icpLedgerInterface,
	principal: Principal,
	maybe_subaccount?: SubAccount
): Promise<bigint> {
	const user_account: Account = {
		owner: principal,
		subaccount: maybe_subaccount ? [maybe_subaccount.toUint8Array()] : []
	};
	return await ledger.icrc1_balance_of(user_account);
}

export async function fetchWtnAllocation(
	principal: Principal,
	waterNeuron: waterNeuronInterface
): Promise<bigint | undefined> {
	try {
		return await waterNeuron.get_airdrop_allocation([principal]);
	} catch (e) {
		console.log('[fetchWtnAllocation] error:', e);
	}
}

const DAO_SHARE = 0.1;
const APY_6M = 0.073;
const APY_8Y = 0.137;

export class WaterNeuronInfo {
	public info: CanisterInfo;

	constructor(wtnCanisterInfo: CanisterInfo) {
		this.info = wtnCanisterInfo;
	}

	totalIcpDeposited(): number {
		return bigintE8sToNumber(this.info.total_icp_deposited);
	}

	neuron8yStake(): number {
		return bigintE8sToNumber(this.info.neuron_8y_stake_e8s);
	}

	neuron6mStake(): number {
		return bigintE8sToNumber(this.info.neuron_6m_stake_e8s);
	}

	exchangeRate(): number {
		return bigintE8sToNumber(this.info.exchange_rate);
	}

	apy(): number {
		const neuron6mStake = this.neuron6mStake();
		const neuron8yStake = this.neuron8yStake();

		if (neuron6mStake + neuron8yStake === 0) return 0;

		const reward6m = APY_6M * neuron6mStake;
		const reward8y = APY_8Y * neuron8yStake;
		const rewardTotal = reward6m + reward8y;
		const share = 1 - DAO_SHARE;

		return (share * rewardTotal) / neuron6mStake;
	}

	stakersCount(): Number {
		return Number(this.info.stakers_count);
	}
}
