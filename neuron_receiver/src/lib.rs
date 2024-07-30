use candid::Principal;
use ic_nns_governance::pb::v1::{ListNeurons, ListNeuronsResponse};

pub mod state;

// "ryjl3-tyaaa-aaaaa-aaaba-cai"
pub const ICP_LEDGER_ID: Principal = Principal::from_slice(&[0, 0, 0, 0, 0, 0, 0, 2, 1, 1]);
// "rrkah-fqaaa-aaaaa-aaaaq-cai"
pub const NNS_GOVERNANCE_ID: Principal = Principal::from_slice(&[0, 0, 0, 0, 0, 0, 0, 1, 1, 1]);

/// Call the list_neurons from the management canister.
/// This canister needs to be a hot key of the neurons in order to have full neuron access.
/// At most this endpoint can return 9_400 neurons following estimations.
/// As the response size grows following s(n) = 212 * n + 359.
pub async fn list_neurons(args: ListNeurons) -> Result<ListNeuronsResponse, String> {
    let res_gov: Result<(ListNeuronsResponse,), (i32, String)> =
        ic_cdk::api::call::call(NNS_GOVERNANCE_ID, "list_neurons", (args,))
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