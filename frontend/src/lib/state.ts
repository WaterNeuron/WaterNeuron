import { bigintE8sToNumber } from '$lib';
import { AccountIdentifier, type Account, SubAccount } from '@dfinity/ledger-icp';
import { Principal } from '@dfinity/principal';
import BigNumber from 'bignumber.js';
import type { _SERVICE as icrcLedgerInterface } from '../declarations/icrc_ledger/icrc_ledger.did';
import type { _SERVICE as icpLedgerInterface } from '../declarations/icp_ledger/icp_ledger.did';
import type { _SERVICE as icpswapPoolInterface } from '../declarations/icpswap_pool/icpswap_pool.did';
import type { _SERVICE as boomerangInterface } from '../declarations/boomerang/boomerang.did';
import type {
	CanisterInfo,
	_SERVICE as waterNeuronInterface
} from '../declarations/water_neuron/water_neuron.did';
import { SignerAgent } from '@slide-computer/signer-agent';
import { Signer } from '@slide-computer/signer';
import { Actor, HttpAgent } from '@dfinity/agent';
import { IDL } from '@dfinity/candid';

export class User {
	public principal: Principal;
	public accountId: string;
	public identityProvider: 'Plug' | 'II' | 'Nfid';
	public account: 'main' | 'ledger';
	public icpBalanceE8s: bigint;
	public nicpBalanceE8s: bigint;
	public wtnBalanceE8s: bigint;
	public wtnAllocationE8s: bigint;

	constructor(principal: Principal, identityProvider: 'Plug' | 'II' | 'Nfid') {
		this.principal = principal;
		this.identityProvider = identityProvider;
		this.account = 'main';
		this.accountId = AccountIdentifier.fromPrincipal({ principal }).toHex();
		this.icpBalanceE8s = 0n;
		this.nicpBalanceE8s = 0n;
		this.wtnBalanceE8s = 0n;
		this.wtnAllocationE8s = 0n;
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

	wtnAllocation(): BigNumber {
		return bigintE8sToNumber(this.wtnAllocationE8s);
	}

	getBalance(asset: 'ICP' | 'WTN' | 'nICP'): BigNumber {
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

const DAO_SHARE = BigNumber(0.1);
const APY_6M = BigNumber(0.08);
const APY_8Y = BigNumber(0.15);

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

export class CanisterActor<canisterInterface> {
	public authenticatedActor?: canisterInterface;
	public anonymousActor: canisterInterface;
	private _idl: IDL.InterfaceFactory;
	private _canisterId: string;

	constructor(httpAgent: HttpAgent, idl: IDL.InterfaceFactory, canisterId: string) {
		this.anonymousActor = Actor.createActor(idl, { agent: httpAgent, canisterId });
		this._idl = idl;
		this._canisterId = canisterId;
	}

	setAuthenticatedActor<T extends Pick<Signer, 'callCanister' | 'openChannel'>>(
		authenticatedAgent: SignerAgent<T> | HttpAgent
	) {
		this.authenticatedActor = Actor.createActor(this._idl, {
			agent: authenticatedAgent,
			canisterId: this._canisterId
		});
	}
}

export class Canisters {
	public icpLedger: CanisterActor<icpLedgerInterface>;
	public nicpLedger: CanisterActor<icrcLedgerInterface>;
	public wtnLedger: CanisterActor<icrcLedgerInterface>;
	public waterNeuron: CanisterActor<waterNeuronInterface>;
	public boomerang: CanisterActor<boomerangInterface>;
	public icpswapPool: CanisterActor<icpswapPoolInterface>;

	constructor({
		icpLedger,
		nicpLedger,
		wtnLedger,
		waterNeuron,
		boomerang,
		icpswapPool
	}: {
		icpLedger: CanisterActor<icpLedgerInterface>;
		nicpLedger: CanisterActor<icrcLedgerInterface>;
		wtnLedger: CanisterActor<icrcLedgerInterface>;
		waterNeuron: CanisterActor<waterNeuronInterface>;
		boomerang: CanisterActor<boomerangInterface>;
		icpswapPool: CanisterActor<icpswapPoolInterface>;
	}) {
		this.icpLedger = icpLedger;
		this.nicpLedger = nicpLedger;
		this.wtnLedger = wtnLedger;
		this.icpLedger = icpLedger;
		this.waterNeuron = waterNeuron;
		this.boomerang = boomerang;
		this.icpswapPool = icpswapPool;
	}
}

export async function fetchIcpBalance(
	principal: Principal,
	icpLedger: icpLedgerInterface,
	maybe_subaccount?: SubAccount
): Promise<bigint> {
	const user_account: Account = {
		owner: principal,
		subaccount: maybe_subaccount ? [maybe_subaccount.toUint8Array()] : []
	};
	return await icpLedger.icrc1_balance_of(user_account);
}

export async function fetchNicpBalance(
	principal: Principal,
	nicpLedger: icrcLedgerInterface,
	maybe_subaccount?: SubAccount
): Promise<bigint> {
	const user_account: Account = {
		owner: principal,
		subaccount: maybe_subaccount ? [maybe_subaccount.toUint8Array()] : []
	};
	return await nicpLedger.icrc1_balance_of(user_account);
}

export async function fetchWtnBalance(
	principal: Principal,
	wtnLedger: icrcLedgerInterface
): Promise<bigint> {
	const user_account: Account = {
		owner: principal,
		subaccount: []
	};
	return await wtnLedger.icrc1_balance_of(user_account);
}

export class WaterNeuronInfo {
	public info: CanisterInfo;

	constructor(wtnCanisterInfo: CanisterInfo) {
		this.info = wtnCanisterInfo;
	}

	totalIcpDeposited(): BigNumber {
		return bigintE8sToNumber(this.info.total_icp_deposited);
	}

	neuron8yStake(): BigNumber {
		return bigintE8sToNumber(this.info.neuron_8y_stake_e8s);
	}

	neuron6mStake(): BigNumber {
		return bigintE8sToNumber(this.info.neuron_6m_stake_e8s);
	}

	exchangeRate(): BigNumber {
		return bigintE8sToNumber(this.info.exchange_rate);
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
		return Number(this.info.stakers_count);
	}
}

// export const fetchGovernanceData = async () => {
//     try {
//       const urls = [
//         "https://ic-api.internetcomputer.org/api/v3/governance-metrics",
//         "https://ic-api.internetcomputer.org/api/v3/neurons/13680855657433416220",
//         "https://ic-api.internetcomputer.org/api/v3/neurons/433047053926084807",
//       ];

//       const [metricsResponse, neuronResponse6m, neuronResponse8y] =
//         await Promise.all(
//           urls.map((url) =>
//             fetch(url, {
//               method: "GET",
//               headers: {
//                 accept: "application/json",
//               },
//             }).then((response) => {
//               if (!response.ok) {
//                 throw new Error(`Error! status: ${response.status}`);
//               }
//               return response.json();
//             }),
//           ),
//         );

//       const metrics = metricsResponse.metrics;
//       let lastRewardsRoundTotalAvailableIcp = null;
//       let totalVotingPower = null;

//       metrics.forEach((metric: any) => {
//         if (
//           metric.name === "governance_latest_reward_round_total_available_e8s"
//         ) {
//           lastRewardsRoundTotalAvailableIcp = metric.subsets[0].value[1];
//         }
//         if (metric.name === "governance_voting_power_total") {
//           totalVotingPower = metric.subsets[0].value[1];
//         }
//       });

//       const voting_power_6m = neuronResponse6m["voting_power"] / 100_000_000;
//       const voting_power_8y = neuronResponse8y["voting_power"] / 100_000_000;
//       const stake_e8s =
//         (neuronResponse6m["stake_e8s"] + neuronResponse8y["stake_e8s"]) /
//         100_000_000;

//       if (
//         lastRewardsRoundTotalAvailableIcp !== null &&
//         totalVotingPower !== null &&
//         stake_e8s !== 0 &&
//         totalVotingPower !== 0
//       ) {
//         const dissolveDelayBonus6m = 1 + 0.5 / 8;
//         const dissolveDelayBonus8y = 1 + 8 / 8;
//         const dailyRewardsIcpPerVotingPowerUnit =
//           lastRewardsRoundTotalAvailableIcp / totalVotingPower;
//         const estimatedNeuronDailyRewards =
//           (voting_power_6m * dissolveDelayBonus6m +
//             voting_power_8y * dissolveDelayBonus8y) *
//           dailyRewardsIcpPerVotingPowerUnit;

//         const neuronAnnualRewards =
//           ((estimatedNeuronDailyRewards * 365.25) / stake_e8s) * 100;

//         console.log(neuronAnnualRewards);
//       }
//     } catch (error) {
//       console.error("An error occurred:", error);
//     }
//   };
