use crate::guards::GuardPrincipal;
use crate::logs::INFO;
use crate::management::{merge_neuron, stop_dissolvement};
use crate::nns_types::{CommandResponse, MergeResponse, NeuronId};
use crate::numeric::{nICP, ICP};
use crate::state::audit::process_event;
use crate::state::event::EventType;
use crate::state::SIX_MONTHS_NEURON_NONCE;
use crate::state::{mutate_state, read_state};
use crate::tasks::{schedule_now, TaskType};
use crate::DEFAULT_LEDGER_FEE;
use crate::{
    CancelWithdrawalError, ConversionArg, ConversionError, DepositSuccess, WithdrawalSuccess,
    ICP_LEDGER_ID,
};
use candid::Nat;
use ic_canister_log::log;
use icrc_ledger_client_cdk::{CdkRuntime, ICRC1Client};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc2::transfer_from::TransferFromArgs;

pub const MINIMUM_DEPOSIT_AMOUNT: ICP = ICP::ONE;
pub const MINIMUM_WITHDRAWAL_AMOUNT: ICP = ICP::from_unscaled(10);

pub async fn cancel_withdrawal(
    neuron_id: NeuronId,
    icp_due: ICP,
) -> Result<MergeResponse, CancelWithdrawalError> {
    let caller = ic_cdk::caller();
    let _guard_principal = GuardPrincipal::new(caller)
        .map_err(|guard_error| CancelWithdrawalError::GuardError { guard_error })?;

    stop_dissolvement(neuron_id).await.unwrap();
    match merge_neuron(SIX_MONTHS_NEURON_NONCE, neuron_id)
        .await
        .unwrap()
        .command
        .expect("Command should always be set.")
    {
        CommandResponse::Merge(response) => {
            mutate_state(|s| {
                if s.neuron_id_to_withdrawal_id.get(&neuron_id).is_some() {
                    process_event(
                        s,
                        EventType::MergeNeuron {
                            icp_stake_e8s: icp_due
                                .checked_sub(ICP::from_e8s(2 * DEFAULT_LEDGER_FEE))
                                .expect("ICP due should always be more than 10."),
                            receiver: Account {
                                owner: caller,
                                subaccount: None,
                            },
                            neuron_id,
                        },
                    );
                    schedule_now(TaskType::ProcessPendingTransfers);
                    schedule_now(TaskType::ProcessLogic);
                }
            });
            Ok(*response)
        }
        CommandResponse::Error(e) => Err(CancelWithdrawalError::GovernanceError(e)),
        _ => Err(CancelWithdrawalError::BadCommand {
            message: format!("Expected merge commande got other."),
        }),
    }
}

pub async fn nicp_to_icp(arg: ConversionArg) -> Result<WithdrawalSuccess, ConversionError> {
    let caller = ic_cdk::caller();
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
                owner: ic_cdk::id(),
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
    let caller = ic_cdk::caller();
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
