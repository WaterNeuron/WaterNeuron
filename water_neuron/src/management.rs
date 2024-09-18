use crate::nns_types::manage_neuron::claim_or_refresh::{By, MemoAndController};
use crate::nns_types::manage_neuron::configure::Operation;
use crate::nns_types::manage_neuron::{
    Command, Configure, Disburse, IncreaseDissolveDelay, Merge, NeuronIdOrSubaccount, Spawn, Split,
    StartDissolving, StopDissolving,
};
use crate::nns_types::{
    AccountIdentifier, DisburseResponse, Empty, GovernanceError, ListNeurons, ListNeuronsResponse,
    ListProposalInfo, ListProposalInfoResponse, ManageNeuron, ManageNeuronResponse, Neuron,
    NeuronId, ProposalId,
};
use crate::state::{
    read_state, EIGHT_YEARS_NEURON_ID, NNS_GOVERNANCE_ID, SIX_MONTHS_NEURON_ID,
    SIX_MONTHS_NEURON_NONCE,
};
use crate::{compute_neuron_staking_subaccount_bytes, CommandResponse};
use candid::{Nat, Principal};
use ic_sns_governance::pb::v1::{
    manage_neuron::Command as SnsCommand, GetProposal, GetProposalResponse,
    ManageNeuron as ManageSnsNeuron, ManageNeuronResponse as ManageSnsNeuronResponse,
};
use icrc_ledger_client_cdk::{CdkRuntime, ICRC1Client};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::{TransferArg, TransferError};

pub async fn transfer(
    to: impl Into<Account>,
    amount: Nat,
    fee: Option<Nat>,
    from_subaccount: Option<[u8; 32]>,
    ledger_canister_id: Principal,
    memo: Option<u64>,
) -> Result<u64, TransferError> {
    let client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id,
    };
    let block_index = client
        .transfer(TransferArg {
            from_subaccount,
            to: to.into(),
            fee,
            created_at_time: None,
            memo: memo.map(|m| m.into()),
            amount,
        })
        .await
        .map_err(|e| TransferError::GenericError {
            error_code: (Nat::from(e.0 as u32)),
            message: (e.1),
        })??;
    Ok(block_index.0.try_into().unwrap())
}

pub async fn balance_of(
    target: impl Into<Account>,
    ledger_canister_id: Principal,
) -> Result<u64, String> {
    let client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id,
    };
    Ok(client
        .balance_of(target.into())
        .await
        .map_err(|(_code, error)| error)?
        .0
        .try_into()
        .unwrap())
}

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

pub async fn list_proposals(args: ListProposalInfo) -> Result<ListProposalInfoResponse, String> {
    let res_gov: Result<(ListProposalInfoResponse,), (i32, String)> =
        ic_cdk::api::call::call(NNS_GOVERNANCE_ID, "list_proposals", (args,))
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

pub async fn get_sns_proposal(
    governace_id: Principal,
    proposal_id: u64,
) -> Result<GetProposalResponse, String> {
    let arg = GetProposal {
        proposal_id: Some(ic_sns_governance::pb::v1::ProposalId { id: proposal_id }),
    };

    let res_gov: Result<(GetProposalResponse,), (i32, String)> =
        ic_cdk::api::call::call(governace_id, "get_proposal", (arg,))
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

pub async fn register_vote(
    neuron_id: NeuronId,
    proposal_id: ProposalId,
    vote_bool: bool,
) -> Result<ManageNeuronResponse, String> {
    let vote = if vote_bool { 1 } else { 2 };

    let arg = ManageNeuron {
        id: None,
        neuron_id_or_subaccount: Some(NeuronIdOrSubaccount::NeuronId(neuron_id)),
        command: Some(Command::RegisterVote(
            crate::nns_types::manage_neuron::RegisterVote {
                proposal: Some(proposal_id),
                vote,
            },
        )),
    };
    let res_gov: Result<(ManageNeuronResponse,), (i32, String)> =
        ic_cdk::api::call::call(NNS_GOVERNANCE_ID, "manage_neuron", (arg,))
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

pub async fn manage_neuron_sns(
    subaccount: Vec<u8>,
    command: SnsCommand,
) -> Result<ManageSnsNeuronResponse, String> {
    let wtn_governance_id = read_state(|s| s.wtn_governance_id);

    let arg = ManageSnsNeuron {
        subaccount,
        command: Some(command),
    };
    let res_gov: Result<(ManageSnsNeuronResponse,), (i32, String)> =
        ic_cdk::api::call::call(wtn_governance_id, "manage_neuron", (arg,))
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

pub async fn follow_neuron(
    neuron_id: NeuronId,
    topic: i32,
    neuron_to_follow: NeuronId,
) -> Result<ManageNeuronResponse, String> {
    let arg = ManageNeuron {
        id: None,
        neuron_id_or_subaccount: Some(NeuronIdOrSubaccount::NeuronId(neuron_id)),
        command: Some(Command::Follow(crate::nns_types::manage_neuron::Follow {
            topic,
            followees: vec![neuron_to_follow],
        })),
    };
    let res_gov: Result<(ManageNeuronResponse,), (i32, String)> =
        ic_cdk::api::call::call(NNS_GOVERNANCE_ID, "manage_neuron", (arg,))
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

pub async fn get_full_neuron(neuron_id: u64) -> Result<Result<Neuron, GovernanceError>, String> {
    let res_gov: Result<(Result<Neuron, GovernanceError>,), (i32, String)> =
        ic_cdk::api::call::call(NNS_GOVERNANCE_ID, "get_full_neuron", (neuron_id,))
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

pub async fn get_full_neuron_by_nonce(
    neuron_nonce: u64,
) -> Result<Result<Neuron, GovernanceError>, String> {
    let subaccount = compute_neuron_staking_subaccount_bytes(ic_cdk::id(), neuron_nonce);
    let args = NeuronIdOrSubaccount::Subaccount(subaccount.to_vec());

    let res_gov: Result<(Result<Neuron, GovernanceError>,), (i32, String)> =
        ic_cdk::api::call::call(
            NNS_GOVERNANCE_ID,
            "get_full_neuron_by_id_or_subaccount",
            (args,),
        )
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

pub async fn get_neuron_ids() -> Result<Vec<u64>, String> {
    let res_gov: Result<(Vec<u64>,), (i32, String)> =
        ic_cdk::api::call::call(NNS_GOVERNANCE_ID, "get_neuron_ids", ())
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

enum NeuronNonceOrId {
    Id(NeuronId),
    Nonce(u64),
}

async fn manage_neuron(
    command: Command,
    neuron_nonce_or_id: NeuronNonceOrId,
) -> Result<ManageNeuronResponse, String> {
    let neuron_id_or_subaccount = match neuron_nonce_or_id {
        NeuronNonceOrId::Id(neuron_id) => NeuronIdOrSubaccount::NeuronId(neuron_id),
        NeuronNonceOrId::Nonce(neuron_nonce) => {
            let subaccount = compute_neuron_staking_subaccount_bytes(ic_cdk::id(), neuron_nonce);
            NeuronIdOrSubaccount::Subaccount(subaccount.to_vec())
        }
    };

    let arg = ManageNeuron {
        id: None,
        neuron_id_or_subaccount: Some(neuron_id_or_subaccount),
        command: Some(command),
    };

    let res_gov: Result<(ManageNeuronResponse,), (i32, String)> =
        ic_cdk::api::call::call(NNS_GOVERNANCE_ID, "manage_neuron", (arg,))
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

pub async fn increase_dissolve_delay(
    neuron_nonce: u64,
    additional_dissolve_delay_seconds: u32,
) -> Result<ManageNeuronResponse, String> {
    manage_neuron(
        Command::Configure(Configure {
            operation: Some(Operation::IncreaseDissolveDelay(IncreaseDissolveDelay {
                additional_dissolve_delay_seconds,
            })),
        }),
        NeuronNonceOrId::Nonce(neuron_nonce),
    )
    .await
}

pub async fn split_neuron(
    neuron_nonce: u64,
    amount_e8s: u64,
) -> Result<ManageNeuronResponse, String> {
    manage_neuron(
        Command::Split(Split { amount_e8s }),
        NeuronNonceOrId::Nonce(neuron_nonce),
    )
    .await
}

pub async fn stop_dissolvement(neuron_id: NeuronId) -> Result<ManageNeuronResponse, String> {
    manage_neuron(
        Command::Configure(Configure {
            operation: Some(Operation::StopDissolving(StopDissolving {})),
        }),
        NeuronNonceOrId::Id(neuron_id),
    )
    .await
}

pub async fn merge_neuron_into_six_months(
    neuron_id: NeuronId,
) -> Result<ManageNeuronResponse, String> {
    assert!(neuron_id.id != SIX_MONTHS_NEURON_ID && neuron_id.id != EIGHT_YEARS_NEURON_ID);
    manage_neuron(
        Command::Merge(Merge {
            source_neuron_id: Some(neuron_id),
        }),
        NeuronNonceOrId::Nonce(SIX_MONTHS_NEURON_NONCE),
    )
    .await
}

#[derive(Debug)]
pub enum SpawnMaturityError {
    FailedToCall(String),
    UnexpectedAnswer(ManageNeuronResponse),
}

impl From<String> for SpawnMaturityError {
    fn from(e: std::string::String) -> Self {
        Self::FailedToCall(e)
    }
}

pub async fn spawn_all_maturity(neuron_id: NeuronId) -> Result<NeuronId, SpawnMaturityError> {
    let manage_neuron_response = manage_neuron(
        Command::Spawn(Spawn {
            new_controller: None,
            nonce: None,
            percentage_to_spawn: Some(100_u32),
        }),
        NeuronNonceOrId::Id(neuron_id),
    )
    .await?;
    if let Some(CommandResponse::Spawn(spawn_response)) = manage_neuron_response.command.clone() {
        if let Some(neuron_id) = spawn_response.created_neuron_id {
            return Ok(neuron_id);
        }
    }

    Err(SpawnMaturityError::UnexpectedAnswer(manage_neuron_response))
}

#[derive(Debug)]
pub enum StartDissolvingError {
    FailedToCall(String),
    UnexpectedAnswer(ManageNeuronResponse),
    NotAllowedToDissolve(String),
}

impl From<String> for StartDissolvingError {
    fn from(e: std::string::String) -> Self {
        Self::FailedToCall(e)
    }
}

pub async fn start_dissolving(neuron_id: NeuronId) -> Result<(), StartDissolvingError> {
    if read_state(|s| !s.is_neuron_allowed_to_dissolve(neuron_id)) {
        return Err(StartDissolvingError::NotAllowedToDissolve(format!(
            "Trying to dissolve main neuron with id: {:?}",
            neuron_id
        )));
    }
    let manage_neuron_response = manage_neuron(
        Command::Configure(Configure {
            operation: Some(Operation::StartDissolving(StartDissolving {})),
        }),
        NeuronNonceOrId::Id(neuron_id),
    )
    .await?;
    if Some(CommandResponse::Configure(Empty {})) == manage_neuron_response.command {
        return Ok(());
    }
    Err(StartDissolvingError::UnexpectedAnswer(
        manage_neuron_response,
    ))
}

#[derive(Debug)]
pub enum DisburseError {
    FailedToCall(String),
    UnexpectedAnswer(ManageNeuronResponse),
}

impl From<String> for DisburseError {
    fn from(e: std::string::String) -> Self {
        Self::FailedToCall(e)
    }
}

pub async fn disburse(
    neuron_id: NeuronId,
    to_account: Account,
) -> Result<DisburseResponse, DisburseError> {
    let account_id: icp_ledger::AccountIdentifier = to_account.into();
    let manage_neuron_response = manage_neuron(
        Command::Disburse(Disburse {
            amount: None,
            to_account: Some(AccountIdentifier {
                hash: account_id.to_vec(),
            }),
        }),
        NeuronNonceOrId::Id(neuron_id),
    )
    .await?;
    if let Some(CommandResponse::Disburse(disburse_response)) = manage_neuron_response.command {
        return Ok(disburse_response);
    }
    Err(DisburseError::UnexpectedAnswer(manage_neuron_response))
}

pub async fn refresh_neuron(neuron_nonce: u64) -> Result<ManageNeuronResponse, String> {
    let arg = ManageNeuron {
        id: None,
        neuron_id_or_subaccount: None,
        command: Some(Command::ClaimOrRefresh(
            crate::nns_types::manage_neuron::ClaimOrRefresh {
                by: Some(By::MemoAndController(MemoAndController {
                    controller: Some(ic_cdk::id()),
                    memo: neuron_nonce,
                })),
            },
        )),
    };
    let res_gov: Result<(ManageNeuronResponse,), (i32, String)> =
        ic_cdk::api::call::call(NNS_GOVERNANCE_ID, "manage_neuron", (arg,))
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
