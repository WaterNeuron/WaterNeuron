use crate::numeric::ICP;
use crate::{
    mutate_state, process_event, read_state, Account, CdkRuntime, EventType, ICRC1Client, DEBUG,
    DEFAULT_LEDGER_FEE, MINIMUM_ICP_DISTRIBUTION,
};
use async_trait::async_trait;
use candid::Principal;
use ic_canister_log::log;
use ic_sns_governance::pb::v1::{
    ListNeurons, ListNeuronsResponse, Neuron, NeuronId, NeuronPermissionType,
};
use std::collections::{BTreeMap, BTreeSet};

#[async_trait]
pub trait CanisterRuntime {
    async fn list_neurons(&self, args: ListNeurons) -> Result<ListNeuronsResponse, String>;

    async fn balance_of(
        &self,
        target: Account,
        ledger_canister_id: Principal,
    ) -> Result<u64, String>;
}

pub struct IcCanisterRuntime {}

#[async_trait]
impl CanisterRuntime for IcCanisterRuntime {
    async fn list_neurons(&self, args: ListNeurons) -> Result<ListNeuronsResponse, String> {
        let sns_governance_id = read_state(|s| {
            s.sns_governance_id
                .expect("bug: sns_governance_id should be set at this point")
        });

        let res_gov: Result<(ListNeuronsResponse,), (i32, String)> =
            ic_cdk::api::call::call(sns_governance_id, "list_neurons", (args,))
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
}

pub async fn maybe_fetch_neurons_and_distribute<R: CanisterRuntime>(
    runtime: &R,
    icp_amount_to_distribute: u64,
) -> Result<usize, String> {
    let mut stakers_count: usize = 0;

    if icp_amount_to_distribute > MINIMUM_ICP_DISTRIBUTION {
        let sns_neurons = fetch_sns_neurons(runtime).await?;
        let total_staked: u64 = sns_neurons.values().sum();

        for (owner, stake) in sns_neurons {
            let share = stake as f64 / total_staked as f64;
            let share_amount = icp_amount_to_distribute as f64 * share;
            if share_amount as u64 > DEFAULT_LEDGER_FEE {
                mutate_state(|s| {
                    process_event(
                        s,
                        EventType::DistributeICPtoSNS {
                            amount: ICP::from_e8s(share_amount as u64),
                            receiver: owner,
                        },
                    );
                });
                stakers_count += 1;
            }
        }
    }

    Ok(stakers_count)
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

    fn get_neuron_stake(neuron: &Neuron) -> u64 {
        neuron
            .cached_neuron_stake_e8s
            .saturating_sub(neuron.neuron_fees_e8s)
            + neuron.maturity_e8s_equivalent
            + neuron.staked_maturity_e8s_equivalent.unwrap_or(0)
    }

    if read_state(|s| s.sns_governance_id.is_none()) {
        return Err("SNS governace ID not set".to_string());
    }

    let mut list_neurons_arg = ListNeurons {
        limit: 1,
        start_page_at: None,
        of_principal: None,
    };
    let mut result: BTreeMap<Principal, u64> = Default::default();
    let mut seen_neurons: BTreeSet<Option<NeuronId>> = Default::default();
    const MAX_RETRY: u64 = 5;
    let mut error_count: u64 = 0;
    let mut neuron_count = 0;

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
                        let stake = get_neuron_stake(&neuron);
                        result
                            .entry(owner)
                            .and_modify(|e| *e += stake)
                            .or_insert(stake);
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
            crate::INFO,
            "[fetch_sns_neurons] Found {neuron_count} neurons and {} unique neurons with some duplicate.",
            seen_neurons.len()
        );
    } else {
        log!(
            crate::INFO,
            "[fetch_sns_neurons] Found {neuron_count} unique neurons.",
        );
    }

    result.remove(&crate::self_canister_id());

    Ok(result)
}