use crate::numeric::ICP;
use crate::storage::{stable_add_rewards, total_pending_rewards};
use crate::{
    get_rewards_ready_to_be_distributed, mutate_state, process_event, read_state, self_canister_id,
    stable_sub_rewards, timestamp_nanos, Account, CdkRuntime, DisplayAmount, EventType,
    ICRC1Client, DEBUG, DEFAULT_LEDGER_FEE, E8S, ICP_LEDGER_ID, INFO, MINIMUM_ICP_DISTRIBUTION,
    SEC_NANOS, SNS_DISTRIBUTION_MEMO, SNS_GOVERNANCE_SUBACCOUNT,
};
use async_trait::async_trait;
use candid::{Nat, Principal};
use ic_canister_log::log;
use ic_sns_governance::pb::v1::{
    ListNeurons, ListNeuronsResponse, Neuron, NeuronId, NeuronPermissionType,
};
use icrc_ledger_types::icrc1::transfer::TransferError;
use std::collections::{BTreeMap, BTreeSet};

pub const WTN_MAX_DISSOLVE_DELAY_SECONDS: u64 = 94_672_800;
const WTN_MAX_NEURON_AGE_FOR_AGE_BONUS: u64 = 94_672_800;
const WTN_MAX_DISSOLVE_DELAY_BONUS_PERCENTAGE: u64 = 100;
const WTN_MAX_AGE_BONUS_PERCENTAGE: u64 = 100;

#[async_trait]
pub trait CanisterRuntime {
    async fn list_neurons(&self, args: ListNeurons) -> Result<ListNeuronsResponse, String>;

    async fn balance_of(
        &self,
        target: Account,
        ledger_canister_id: Principal,
    ) -> Result<u64, String>;

    async fn transfer_icp(&self, to: Principal, amount: u64) -> Result<u64, TransferError>;
}

pub struct IcCanisterRuntime {}

#[async_trait]
impl CanisterRuntime for IcCanisterRuntime {
    async fn list_neurons(&self, args: ListNeurons) -> Result<ListNeuronsResponse, String> {
        let wtn_governance_id = read_state(|s| s.wtn_governance_id);

        let res_gov: Result<(ListNeuronsResponse,), (i32, String)> =
            ic_cdk::api::call::call(wtn_governance_id, "list_neurons", (args,))
                .await
                .map_err(|(code, msg)| (code as i32, msg));
        match res_gov {
            Ok((res,)) => Ok(res),
            Err((code, msg)) => Err(format!(
                "Error while calling Governance canister ({}): {:?}",
                code, msg
            )),
        }
    }

    async fn balance_of(
        &self,
        target: Account,
        ledger_canister_id: Principal,
    ) -> Result<u64, String> {
        let client = ICRC1Client {
            runtime: CdkRuntime,
            ledger_canister_id,
        };
        Ok(client
            .balance_of(target)
            .await
            .map_err(|(_code, error)| error)?
            .0
            .try_into()
            .unwrap())
    }

    async fn transfer_icp(&self, to: Principal, amount: u64) -> Result<u64, TransferError> {
        let amount = amount.checked_sub(DEFAULT_LEDGER_FEE).unwrap();
        crate::transfer(
            to,
            Nat::from(amount),
            Some(Nat::from(DEFAULT_LEDGER_FEE)),
            Some(SNS_GOVERNANCE_SUBACCOUNT),
            ICP_LEDGER_ID,
            Some(SNS_DISTRIBUTION_MEMO.into()),
        )
        .await
    }
}

pub async fn process_icp_distribution<R: CanisterRuntime>(runtime: &R) -> Option<u64> {
    let mut error_count = 0;
    let rewards = get_rewards_ready_to_be_distributed();
    if rewards.is_empty() {
        return None;
    }

    for (to, reward) in rewards {
        stable_sub_rewards(to, reward);
        match runtime.transfer_icp(to, reward).await {
            Ok(block_index) => {
                log!(
                    INFO,
                    "[process_icp_distribution] successfully transferred {} ICP to {to} at {block_index}",
                    DisplayAmount(reward),
                );
            }
            Err(e) => {
                log!(
                    DEBUG,
                    "[process_icp_distribution] failed to transfer for {to} with error: {e}",
                );
                error_count += 1;
                stable_add_rewards(to, reward);
            }
        }
    }

    Some(error_count)
}

pub async fn maybe_fetch_neurons_and_distribute<R: CanisterRuntime>(
    runtime: &R,
    balance: u64,
) -> Result<usize, String> {
    let mut stakers_count: usize = 0;

    let icp_amount_to_distribute = balance.checked_sub(total_pending_rewards()).unwrap();

    if icp_amount_to_distribute >= MINIMUM_ICP_DISTRIBUTION {
        let sns_neurons = fetch_sns_neurons(runtime).await?;
        let total_voting_power: u64 = sns_neurons.values().sum();

        log!(INFO, "[maybe_fetch_neurons_and_distribute] fetched {} neurons for a total voting power of {total_voting_power}", sns_neurons.len());

        if total_voting_power == 0 {
            return Err("total_voting_power cannot be 0".to_string());
        }

        mutate_state(|s| {
            s.latest_distribution_icp_per_vp =
                Some((icp_amount_to_distribute / E8S) as f64 / total_voting_power as f64);
        });

        for (owner, voting_power) in sns_neurons {
            let share = voting_power as f64 / total_voting_power as f64;
            let share_amount = icp_amount_to_distribute as f64 * share;
            let share_amount_icp = ICP::from_e8s(share_amount as u64);
            stable_add_rewards(owner, share_amount_icp.0);
            stakers_count += 1;
            log!(
                INFO,
                "[maybe_fetch_neurons_and_distribute] distribute {share_amount_icp} ICP to {owner}",
            );
        }

        mutate_state(|s| {
            process_event(s, EventType::DistributeICPtoSNSv2);
        });
    }

    Ok(stakers_count)
}

fn get_rounded_voting_power(neuron: &Neuron, now_seconds: u64) -> u64 {
    neuron.voting_power(
        now_seconds,
        WTN_MAX_DISSOLVE_DELAY_SECONDS,
        WTN_MAX_NEURON_AGE_FOR_AGE_BONUS,
        WTN_MAX_DISSOLVE_DELAY_BONUS_PERCENTAGE,
        WTN_MAX_AGE_BONUS_PERCENTAGE,
    ) / E8S
}

async fn fetch_sns_neurons<R: CanisterRuntime>(
    runtime: &R,
) -> Result<BTreeMap<Principal, u64>, String> {
    fn get_neuron_owner(neuron: &Neuron) -> Option<Principal> {
        for permission in &neuron.permissions {
            if permission.permission_type.len() >= NeuronPermissionType::all().len() {
                return permission.principal.map(|p| p.0);
            }
        }
        None
    }

    let mut list_neurons_arg = ListNeurons {
        limit: 0,
        start_page_at: None,
        of_principal: None,
    };
    let mut result: BTreeMap<Principal, u64> = Default::default();
    let mut seen_neurons: BTreeSet<Option<NeuronId>> = Default::default();
    const MAX_RETRY: u64 = 5;
    let mut error_count: u64 = 0;
    let mut neuron_count = 0;

    let now_seconds = timestamp_nanos() / SEC_NANOS;

    loop {
        match runtime.list_neurons(list_neurons_arg.clone()).await {
            Ok(response) => {
                match response.neurons.last() {
                    Some(neuron) => list_neurons_arg.start_page_at = neuron.id.clone(),
                    None => break,
                }
                neuron_count += response.neurons.len();
                for neuron in response.neurons {
                    if !seen_neurons.insert(neuron.id.clone()) {
                        continue;
                    }
                    if let Some(owner) = get_neuron_owner(&neuron) {
                        if owner == self_canister_id() || owner == crate::NNS_GOVERNANCE_ID {
                            continue;
                        }
                        let vp = get_rounded_voting_power(&neuron, now_seconds);
                        result.entry(owner).and_modify(|e| *e += vp).or_insert(vp);
                    } else {
                        log!(
                            INFO,
                            "[fetch_sns_neurons] failed to get neuron owner of neuron with id: {:?}",
                            neuron.id
                        );
                    }
                }
                error_count = 0;
            }
            Err(error) => {
                error_count += 1;
                if error_count == MAX_RETRY {
                    return Err(
                        "Failed to fetch all neurons, reached the maximum retry count.".to_string(),
                    );
                }
                log!(
                    DEBUG,
                    "[fetch_sns_neurons] failed to fetch with error {error}."
                );
            }
        }
    }
    if neuron_count != seen_neurons.len() {
        log!(
            INFO,
            "[fetch_sns_neurons] Found {neuron_count} neurons and {} unique neurons with some duplicate.",
            seen_neurons.len()
        );
    } else {
        log!(
            INFO,
            "[fetch_sns_neurons] Found {neuron_count} unique neurons.",
        );
    }

    Ok(result)
}

#[cfg(test)]
mod test {
    use crate::sns_governance::{
        fetch_sns_neurons, maybe_fetch_neurons_and_distribute, process_icp_distribution,
        CanisterRuntime, ListNeurons, ListNeuronsResponse, Neuron,
    };
    use crate::state::replace_state;
    use crate::state::test::default_state;
    use crate::storage::get_pending_rewards;
    use crate::{compute_neuron_staking_subaccount_bytes, Account, E8S};
    use async_trait::async_trait;
    use candid::{Nat, Principal};
    use ic_sns_governance::pb::v1::{NeuronId, NeuronPermission, NeuronPermissionType};
    use icrc_ledger_types::icrc1::transfer::TransferError;
    use mockall::mock;
    use std::str::FromStr;

    mock! {
        pub CanisterRuntime{}

         #[async_trait]
         impl CanisterRuntime for CanisterRuntime {
            async fn list_neurons(&self, args: ListNeurons) -> Result<ListNeuronsResponse, String>;

            async fn balance_of(
                &self,
                target: Account,
                ledger_canister_id: Principal,
            ) -> Result<u64, String>;

            async fn transfer_icp(&self, to: Principal, amount: u64) -> Result<u64, TransferError>;
         }
    }

    #[tokio::test]
    async fn should_retry_and_fail() {
        replace_state(default_state());
        let mut runtime = MockCanisterRuntime::new();

        runtime
            .expect_list_neurons()
            .withf(move |_| true)
            .times(5)
            .return_const(Err("".to_string()));

        assert_eq!(
            fetch_sns_neurons(&runtime).await,
            Err("Failed to fetch all neurons, reached the maximum retry count.".to_string())
        );
    }

    #[tokio::test]
    async fn should_fetch_all_neurons() {
        replace_state(default_state());
        let mut runtime = MockCanisterRuntime::new();
        let caller = Principal::from_str("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();

        let mut neurons: Vec<Neuron> = vec![];
        for id in 0..30 {
            neurons.push(Neuron {
                id: Some(NeuronId {
                    id: compute_neuron_staking_subaccount_bytes(caller, id).to_vec(),
                }),
                permissions: vec![NeuronPermission {
                    principal: Some(caller.into()),
                    permission_type: NeuronPermissionType::all(),
                }],
                cached_neuron_stake_e8s: E8S,
                maturity_e8s_equivalent: E8S,
                staked_maturity_e8s_equivalent: Some(E8S),
                dissolve_state: Some(DissolveState::DissolveDelaySeconds(94_672_799)),
                created_timestamp_seconds: 1_718_691_769,
                aging_since_timestamp_seconds: 1_718_691_769,
                voting_power_percentage_multiplier: 100,
                ..Default::default()
            });
        }

        runtime
            .expect_list_neurons()
            .withf(move |arg| arg.start_page_at == None)
            .times(1)
            .return_const(Ok(ListNeuronsResponse {
                neurons: neurons.iter().take(10).cloned().collect(),
            }));

        runtime
            .expect_list_neurons()
            .withf(move |arg| {
                arg.start_page_at
                    == Some(NeuronId {
                        id: compute_neuron_staking_subaccount_bytes(caller, 9).to_vec(),
                    })
            })
            .times(1)
            .return_const(Ok(ListNeuronsResponse {
                neurons: neurons.iter().skip(10).take(10).cloned().collect(),
            }));

        runtime
            .expect_list_neurons()
            .withf(move |arg| {
                arg.start_page_at
                    == Some(NeuronId {
                        id: compute_neuron_staking_subaccount_bytes(caller, 19).to_vec(),
                    })
            })
            .times(1)
            .return_const(Ok(ListNeuronsResponse {
                neurons: neurons.iter().skip(20).take(10).cloned().collect(),
            }));

        runtime
            .expect_list_neurons()
            .withf(move |arg| {
                arg.start_page_at
                    == Some(NeuronId {
                        id: compute_neuron_staking_subaccount_bytes(caller, 29).to_vec(),
                    })
            })
            .times(1)
            .return_const(Ok(ListNeuronsResponse { neurons: vec![] }));

        for (k, v) in fetch_sns_neurons(&runtime).await.unwrap() {
            assert!(v >= 90, "{v}");
            assert_eq!(k, caller);
        }
    }

    #[tokio::test]
    async fn should_distribute_icp() {
        replace_state(default_state());
        let mut runtime = MockCanisterRuntime::new();
        let caller = Principal::from_str("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();
        let caller_2 =
            Principal::from_str("44bpz-wpk6f-zydao-ahpkm-dxl3b-kcx2w-b5qd5-tlhg4-3jh7i-ccf33-dae")
                .unwrap();

        let mut neurons: Vec<Neuron> = vec![];
        neurons.push(Neuron {
            id: Some(NeuronId {
                id: compute_neuron_staking_subaccount_bytes(caller, 0).to_vec(),
            }),
            permissions: vec![NeuronPermission {
                principal: Some(caller.into()),
                permission_type: NeuronPermissionType::all(),
            }],
            cached_neuron_stake_e8s: E8S,
            maturity_e8s_equivalent: E8S,
            staked_maturity_e8s_equivalent: Some(E8S),
            dissolve_state: Some(DissolveState::DissolveDelaySeconds(94_672_799)),
            created_timestamp_seconds: 1_718_691_769,
            aging_since_timestamp_seconds: 1_718_691_769,
            voting_power_percentage_multiplier: 100,
            ..Default::default()
        });
        neurons.push(Neuron {
            id: Some(NeuronId {
                id: compute_neuron_staking_subaccount_bytes(caller_2, 0).to_vec(),
            }),
            permissions: vec![NeuronPermission {
                principal: Some(caller_2.into()),
                permission_type: NeuronPermissionType::all(),
            }],
            cached_neuron_stake_e8s: E8S,
            maturity_e8s_equivalent: E8S,
            staked_maturity_e8s_equivalent: Some(E8S),
            dissolve_state: Some(DissolveState::DissolveDelaySeconds(94_672_799)),
            created_timestamp_seconds: 1_718_691_769,
            aging_since_timestamp_seconds: 1_718_691_769,
            voting_power_percentage_multiplier: 100,
            ..Default::default()
        });

        runtime
            .expect_list_neurons()
            .withf(move |arg| arg.start_page_at == None)
            .times(1)
            .return_const(Ok(ListNeuronsResponse { neurons }));
        runtime
            .expect_list_neurons()
            .times(1)
            .return_const(Ok(ListNeuronsResponse { neurons: vec![] }));

        let icp_to_distribute: u64 = 100 * E8S;
        let res = maybe_fetch_neurons_and_distribute(&runtime, icp_to_distribute).await;
        assert_eq!(res, Ok(2));
        assert_eq!(
            get_pending_rewards(caller_2).unwrap(),
            Nat::from(icp_to_distribute / 2)
        );
        assert_eq!(
            get_pending_rewards(caller).unwrap(),
            Nat::from(icp_to_distribute / 2)
        );

        let res = maybe_fetch_neurons_and_distribute(&runtime, icp_to_distribute).await;
        assert_eq!(res, Ok(0));
        assert_eq!(
            get_pending_rewards(caller_2).unwrap(),
            Nat::from(icp_to_distribute / 2)
        );
        assert_eq!(
            get_pending_rewards(caller).unwrap(),
            Nat::from(icp_to_distribute / 2)
        );

        runtime
            .expect_transfer_icp()
            .withf(move |_to, balance| *balance == icp_to_distribute / 2)
            .times(2)
            .return_const(Ok(1));

        process_icp_distribution(&runtime).await;

        assert_eq!(get_pending_rewards(caller), None);
        assert_eq!(get_pending_rewards(caller_2), None);
    }

    use crate::sns_governance::get_rounded_voting_power;
    use ic_sns_governance::pb::v1::neuron::DissolveState;

    #[test]
    fn check_voting_power() {
        let neuron = Neuron {
            id: Some(NeuronId { id: vec![] }),
            permissions: vec![],
            cached_neuron_stake_e8s: 1_400_000 * E8S,
            maturity_e8s_equivalent: 0,
            staked_maturity_e8s_equivalent: None,
            neuron_fees_e8s: 0,
            created_timestamp_seconds: 1_718_691_769,
            aging_since_timestamp_seconds: 1_718_691_769,
            voting_power_percentage_multiplier: 100,
            dissolve_state: Some(DissolveState::DissolveDelaySeconds(94_672_799)),
            ..Default::default()
        };

        let vp = get_rounded_voting_power(&neuron, 1_720_683_746);

        assert_eq!(vp, 2_858_913);
    }
}
