use crate::state::{mutate_state, read_state};
use candid::Principal;
use ic_canister_log::{declare_log_buffer, export as export_logs, log, GlobalBuffer, Sink};
use ic_nns_common::pb::v1::NeuronId;
use ic_nns_governance::pb::v1::manage_neuron::{Command, Disburse, NeuronIdOrSubaccount};
use ic_nns_governance::pb::v1::manage_neuron_response::DisburseToNeuronResponse;
use ic_nns_governance::pb::v1::{
    ListNeurons, ListNeuronsResponse, ManageNeuron, ManageNeuronResponse,
};
use icp_ledger::protobuf::AccountIdentifier;
use icrc_ledger_types::icrc1::account::Account;
use serde::Deserialize;

pub mod state;

declare_log_buffer!(name = INFO_BUF, capacity = 1000);
pub const INFO: PrintProxySink = PrintProxySink("INFO", &INFO_BUF);

pub struct PrintProxySink(&'static str, &'static GlobalBuffer);

impl Sink for PrintProxySink {
    fn append(&self, entry: ic_canister_log::LogEntry) {
        ic_cdk::println!("{} {}:{} {}", self.0, entry.file, entry.line, entry.message);
        self.1.append(entry)
    }
}

#[derive(Clone, serde::Serialize, Deserialize, Debug)]
pub struct LogEntry {
    pub timestamp: u64,
    pub file: String,
    pub line: u32,
    pub message: String,
    pub counter: u64,
}

#[derive(Clone, Default, serde::Serialize, Deserialize, Debug)]
pub struct Log {
    pub entries: Vec<LogEntry>,
}

impl Log {
    pub fn new() -> Self {
        let logs = export_logs(&INFO_BUF);
        let mut log: Log = Default::default();
        for entry in logs {
            log.entries.push(LogEntry {
                timestamp: entry.timestamp,
                counter: entry.counter,
                file: entry.file.to_string(),
                line: entry.line,
                message: entry.message,
            });
        }
        log
    }

    pub fn serialize_logs(&self, max_body_size: usize) -> String {
        let mut entries_json: String = serde_json::to_string(&self).unwrap_or_default();

        if entries_json.len() > max_body_size {
            let mut left = 0;
            let mut right = self.entries.len();

            while left < right {
                let mid = left + (right - left) / 2;
                let mut temp_log = self.clone();
                temp_log.entries.truncate(mid);
                let temp_entries_json = serde_json::to_string(&temp_log).unwrap_or_default();

                if temp_entries_json.len() <= max_body_size {
                    entries_json = temp_entries_json;
                    left = mid + 1;
                } else {
                    right = mid;
                }
            }
        }
        entries_json
    }
}

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

async fn manage_neuron(
    command: Command,
    neuron_id: NeuronId,
) -> Result<ManageNeuronResponse, String> {
    let arg = ManageNeuron {
        id: None,
        neuron_id_or_subaccount: Some(NeuronIdOrSubaccount::NeuronId(neuron_id)),
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
) -> Result<DisburseToNeuronResponse, DisburseError> {
    let account_id: icp_ledger::AccountIdentifier = to_account.into();
    let manage_neuron_response = manage_neuron(
        Command::Disburse(Disburse {
            amount: None,
            to_account: Some(AccountIdentifier {
                hash: account_id.to_vec(),
            }),
        }),
        neuron_id,
    )
    .await?;
    if let Some(ic_nns_governance::pb::v1::manage_neuron_response::Command::DisburseToNeuron(
        disburse_response,
    )) = manage_neuron_response.command
    {
        return Ok(disburse_response);
    }
    Err(DisburseError::UnexpectedAnswer(manage_neuron_response))
}

pub async fn update_neurons() {
    mutate_state(|s| s.reset_neuron_map());

    match list_neurons(ListNeurons {
        neuron_ids: vec![],
        include_neurons_readable_by_caller: true,
    })
    .await
    {
        Ok(neurons_response) => {
            for neuron in neurons_response.full_neurons {
                let neuron_id = neuron.id.unwrap().id;
                mutate_state(|s| s.insert_or_update_neuron(neuron_id, neuron));
            }
        }
        Err(e) => {
            log!(
                INFO,
                "[try_fetch_new_neurons] failed to fetch with error {e:?}"
            );
        }
    }
}

pub async fn maybe_disburse() {
    let neurons = read_state(|s| s.neurons.clone());
    let target_account = read_state(|s| s.disburse_to);

    for (neuron_id, _neuron) in neurons {
        let disburse_result = disburse(NeuronId { id: neuron_id }, target_account).await;
        log!(
            INFO,
            "[maybe_disburse] Disburse {neuron_id} with result {disburse_result:?}"
        );
    }
}
