use crate::log::INFO;
use crate::{
    derive_subaccount_staking, self_canister_id, BoomerangError, ConversionArg, ConversionError,
    DepositSuccess, E8S, ICP_LEDGER_ID, NICP_LEDGER_ID, TRANSFER_FEE, WATER_NEURON_ID,
};
use candid::{Nat, Principal};
use ic_canister_log::log;
use icrc_ledger_client_cdk::{CdkRuntime, ICRC1Client};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::TransferArg;
use icrc_ledger_types::icrc2::approve::ApproveArgs;

pub async fn retrieve_nicp(target: Principal) -> Result<Nat, BoomerangError> {
    let nicp_client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id: NICP_LEDGER_ID,
    };

    let boomerang_id = self_canister_id();
    let subaccount = derive_subaccount_staking(target);

    let boomerang_account = Account {
        owner: boomerang_id,
        subaccount: Some(subaccount),
    };

    let nicp_balance_e8s: u64 = match nicp_client.balance_of(boomerang_account).await {
        Ok(balance) => balance.0.try_into().unwrap(),
        Err((code, message)) => {
            return Err(BoomerangError::BalanceOfError(format!(
                "code: {code} - message: {message}"
            )));
        }
    };

    let to_transfer_amount = nicp_balance_e8s.checked_sub(TRANSFER_FEE).unwrap();

    match nicp_client
        .transfer(TransferArg {
            memo: None,
            amount: to_transfer_amount.into(),
            fee: Some(TRANSFER_FEE.into()),
            from_subaccount: boomerang_account.subaccount,
            created_at_time: None,
            to: target.into(),
        })
        .await
    {
        Ok(result) => match result {
            Ok(block_index) => {
                log!(
                    INFO,
                    "Transfered nICP for {target} at block index: {}",
                    block_index
                );
                Ok(block_index)
            }
            Err(e) => Err(BoomerangError::TransferError(e)),
        },
        Err((code, message)) => Err(BoomerangError::GenericError { code, message }),
    }
}

pub async fn notify_icp_deposit(target: Principal) -> Result<DepositSuccess, BoomerangError> {
    let boomerang_id = self_canister_id();
    let subaccount = derive_subaccount_staking(target);

    let boomerang_account = Account {
        owner: boomerang_id,
        subaccount: Some(subaccount),
    };

    let client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id: ICP_LEDGER_ID,
    };

    let balance_e8s: u64 = match client.balance_of(boomerang_account).await {
        Ok(balance) => balance.0.try_into().unwrap(),
        Err((code, message)) => {
            return Err(BoomerangError::BalanceOfError(format!(
                "code: {code} - message: {message}"
            )));
        }
    };

    log!(
        INFO,
        "Fetched balance for {target}: {} ICP",
        balance_e8s / E8S
    );

    let spender = Account {
        owner: WATER_NEURON_ID,
        subaccount: None,
    };

    let approve_args = ApproveArgs {
        from_subaccount: boomerang_account.subaccount,
        spender,
        amount: balance_e8s.into(),
        expected_allowance: None,
        expires_at: None,
        fee: None,
        memo: None,
        created_at_time: None,
    };

    match client.approve(approve_args).await {
        Ok(result) => match result {
            Ok(block_index) => {
                log! {INFO, "Approved for {target} occured at block index: {}", block_index};
            }
            Err(error) => {
                return Err(BoomerangError::ApproveError(error));
            }
        },
        Err((code, message)) => {
            return Err(BoomerangError::GenericError { code, message });
        }
    };

    let transfer_amount_e8s = balance_e8s
        .checked_sub(2 * TRANSFER_FEE)
        .expect("underflow");

    let conversion_arg = ConversionArg {
        amount_e8s: transfer_amount_e8s,
        maybe_subaccount: boomerang_account.subaccount,
    };

    let conversion_result: (Result<DepositSuccess, ConversionError>,) =
        ic_cdk::call(WATER_NEURON_ID, "icp_to_nicp", (conversion_arg,))
            .await
            .unwrap();

    match conversion_result.0 {
        Ok(success) => {
            log!(
                INFO,
                "Transfered {} ICP at block index: {}",
                balance_e8s / E8S,
                success.block_index
            );
            Ok(success)
        }
        Err(error) => Err(BoomerangError::ConversionError(error)),
    }
}
