use crate::guards::GuardPrincipal;
use crate::logs::{DEBUG, INFO};
use crate::management::{merge_neuron_into_six_months, stop_dissolvement};
use crate::nns_types::{time_left_seconds, NeuronId};
use crate::numeric::{nICP, ICP};
use crate::state::audit::process_event;
use crate::state::event::EventType;
use crate::state::{mutate_state, read_state};
use crate::tasks::{schedule_now, TaskType};
use crate::{
    get_full_neuron, timestamp_nanos, CancelWithdrawalError, ConversionArg, ConversionError,
    DepositSuccess, WithdrawalSuccess, ICP_LEDGER_ID, ONE_DAY_SECONDS,
};
use candid::Nat;
use ic_canister_log::log;
use ic_nns_governance_api::{
    manage_neuron_response::Command as CommandResponse, manage_neuron_response::MergeResponse,
};
use icrc_ledger_client_cdk::{CdkRuntime, ICRC1Client};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc2::transfer_from::TransferFromArgs;

pub const MINIMUM_DEPOSIT_AMOUNT: ICP = ICP::ONE;
pub const MINIMUM_WITHDRAWAL_AMOUNT: ICP = ICP::from_unscaled(10);

pub async fn cancel_withdrawal(
    neuron_id: NeuronId,
) -> Result<MergeResponse, CancelWithdrawalError> {
    let caller = ic_cdk::api::msg_caller();
    let _guard_principal = GuardPrincipal::new(caller)
        .map_err(|guard_error| CancelWithdrawalError::GuardError { guard_error })?;

    match get_full_neuron(neuron_id.id).await {
        Ok(result) => match result {
            Ok(neuron) => match time_left_seconds(&neuron, timestamp_nanos() / crate::SEC_NANOS) {
                Some(time) => {
                    if time < ONE_DAY_SECONDS * 14 {
                        return Err(CancelWithdrawalError::TooLate);
                    }
                }
                None => return Err(CancelWithdrawalError::UnknownTimeLeft),
            },
            Err(gov_err) => return Err(CancelWithdrawalError::GovernanceError(gov_err)),
        },
        Err(error) => return Err(CancelWithdrawalError::GetFullNeuronError { message: error }),
    }

    let icp_due = match read_state(|s| {
        s.neuron_id_to_withdrawal_id
            .get(&neuron_id)
            .and_then(|withdrawal_id| s.withdrawal_id_to_request.get(withdrawal_id).cloned())
    }) {
        Some(withdrawal_request) => {
            if withdrawal_request.receiver != caller.into() {
                return Err(CancelWithdrawalError::BadCaller {
                    message: "Caller is not the owner.".to_string(),
                });
            }
            withdrawal_request.icp_due
        }
        None => return Err(CancelWithdrawalError::RequestNotFound),
    };

    log!(
        DEBUG,
        "[cancel_withdrawal] Cancelling neuron with id {}, ICP due: {icp_due}",
        neuron_id.id
    );

    let stop_dissolvement_result = stop_dissolvement(neuron_id)
        .await
        .map_err(|error_msg| CancelWithdrawalError::StopDissolvementError { message: error_msg });

    if stop_dissolvement_result.is_err() {
        log!(
            INFO,
            "[cancel_withdrawal] Unexpected stop_dissolvement result: {stop_dissolvement_result:?}"
        );
    }

    stop_dissolvement_result?;

    match merge_neuron_into_six_months(neuron_id)
        .await
        .map_err(|error_msg| CancelWithdrawalError::MergeNeuronError { message: error_msg })?
        .command
        .expect("Command should always be set.")
    {
        CommandResponse::Merge(response) => {
            if response
                .source_neuron
                .as_ref()
                .unwrap()
                .cached_neuron_stake_e8s
                != 0
            {
                log!(
                    INFO,
                    "[cancel_withdrawal] Expected cached_neuron_stake_e8s to be 0 got {response:?}"
                );
                return Err(CancelWithdrawalError::MergeNeuronError {
                    message: format!("Expected cached_neuron_stake_e8s to be 0 got {response:?}"),
                });
            }

            mutate_state(|s| {
                process_event(s, EventType::MergeNeuron { neuron_id });
            });
            schedule_now(TaskType::ProcessPendingTransfers);
            schedule_now(TaskType::RefreshShortTerm);
            Ok(response)
        }
        CommandResponse::Error(e) => Err(CancelWithdrawalError::GovernanceError(e)),
        other => Err(CancelWithdrawalError::BadCommand {
            message: format!("Expected merge command got {other:?}"),
        }),
    }
}

pub async fn nicp_to_icp(arg: ConversionArg) -> Result<WithdrawalSuccess, ConversionError> {
    let caller = ic_cdk::api::msg_caller();
    let _guard_principal = GuardPrincipal::new(caller)
        .map_err(|guard_error| ConversionError::GuardError { guard_error })?;

    let (nicp_amount, maybe_subaccount) = (nICP::from_e8s(arg.amount_e8s), arg.maybe_subaccount);
    let icp_due = read_state(|s| s.convert_nicp_to_icp(nicp_amount));

    if icp_due < MINIMUM_WITHDRAWAL_AMOUNT {
        return Err(ConversionError::AmountTooLow {
            minimum_amount_e8s: MINIMUM_WITHDRAWAL_AMOUNT.0,
        });
    }

    let client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id: read_state(|s| s.nicp_ledger_id),
    };

    let receiver = Account {
        owner: caller,
        subaccount: maybe_subaccount,
    };

    match client
        .transfer_from(TransferFromArgs {
            spender_subaccount: None,
            from: receiver,
            to: Account {
                owner: ic_cdk::api::canister_self(),
                subaccount: None,
            },
            amount: Nat::from(arg.amount_e8s),
            fee: None,
            memo: None,
            created_at_time: None,
        })
        .await
    {
        Ok(result) => match result {
            Ok(block_index) => {
                let icp_due = read_state(|s| s.convert_nicp_to_icp(nicp_amount));
                log!(
                    INFO,
                    "[nicp_to_icp] Converted {nicp_amount} nICP for {icp_due} ICP by {receiver}",
                );
                schedule_now(TaskType::ProcessLogic);
                let withdrawal_id = mutate_state(|s| {
                    let withdrawal_id = s.withdrawal_id;
                    process_event(
                        s,
                        EventType::NIcpWithdrawal {
                            receiver,
                            nicp_burned: nicp_amount,
                            nicp_burn_index: block_index.clone().0.try_into().unwrap(),
                        },
                    );
                    withdrawal_id
                });
                Ok(WithdrawalSuccess {
                    withdrawal_id,
                    block_index,
                    icp_amount: Some(icp_due),
                })
            }
            Err(transfer_from_error) => {
                Err(ConversionError::TransferFromError(transfer_from_error))
            }
        },
        Err((code, message)) => Err(ConversionError::GenericError { code, message }),
    }
}

pub async fn icp_to_nicp(arg: ConversionArg) -> Result<DepositSuccess, ConversionError> {
    let caller = ic_cdk::api::msg_caller();
    let _guard_principal = GuardPrincipal::new(caller)
        .map_err(|guard_error| ConversionError::GuardError { guard_error })?;

    let (amount, maybe_subaccount) = (ICP::from_e8s(arg.amount_e8s), arg.maybe_subaccount);
    if amount < MINIMUM_DEPOSIT_AMOUNT {
        return Err(ConversionError::AmountTooLow {
            minimum_amount_e8s: MINIMUM_DEPOSIT_AMOUNT.0,
        });
    }

    let client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id: ICP_LEDGER_ID,
    };

    let receiver = Account {
        owner: caller,
        subaccount: maybe_subaccount,
    };

    match client
        .transfer_from(TransferFromArgs {
            spender_subaccount: None,
            from: receiver,
            to: read_state(|s| s.get_6m_neuron_account()),
            amount: Nat::from(arg.amount_e8s),
            fee: None,
            memo: None,
            created_at_time: None,
        })
        .await
    {
        Ok(result) => match result {
            Ok(block_index) => {
                let nicp_due = read_state(|s| s.convert_icp_to_nicp(amount));
                log!(
                    INFO,
                    "[icp_to_nicp] Converted {amount} ICP for {nicp_due} nICP by {receiver}",
                );
                schedule_now(TaskType::ProcessPendingTransfers);
                schedule_now(TaskType::RefreshShortTerm);
                let transfer_id = read_state(|s| s.transfer_id);
                mutate_state(|s| {
                    process_event(
                        s,
                        EventType::IcpDeposit {
                            receiver,
                            amount,
                            block_index: block_index.clone().0.try_into().unwrap(),
                        },
                    );
                });
                Ok(DepositSuccess {
                    block_index,
                    transfer_id,
                    nicp_amount: Some(nicp_due),
                })
            }
            Err(transfer_from_error) => {
                Err(ConversionError::TransferFromError(transfer_from_error))
            }
        },
        Err((code, message)) => Err(ConversionError::GenericError { code, message }),
    }
}
