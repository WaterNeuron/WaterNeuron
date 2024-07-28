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

    let target_account = Account {
        owner: boomerang_id,
        subaccount: Some(subaccount),
    };

    let nicp_balance_e8s: u64 = match nicp_client.balance_of(target_account).await {
        Ok(balance) => balance.0.try_into().unwrap(),
        Err((code, msg)) => {
            return Err(BoomerangError::BalanceOfError(format!(
                "code: {code} - msg: {msg}"
            )));
        }
    };

    let to_transfer_amount = nicp_balance_e8s.checked_sub(TRANSFER_FEE).unwrap();

    match nicp_client
        .transfer(TransferArg {
            memo: None,
            amount: to_transfer_amount.into(),
            fee: Some(TRANSFER_FEE.into()),
            from_subaccount: Some(subaccount),
            created_at_time: None,
            to: target.into(),
        })
        .await
        .unwrap()
    {
        Ok(block_index) => {
            log!(
                INFO,
                "Transfered nICP for {target} at block index: {}",
                block_index
            );
            Ok(block_index)
        }
        Err(e) => Err(BoomerangError::TransferError(e)),
    }
}

pub async fn notify_icp_deposit(client_id: Principal) -> Result<DepositSuccess, BoomerangError> {
    let boomerang_id = self_canister_id();

    let subaccount = derive_subaccount_staking(client_id);

    let boomerang_account = Account {
        owner: boomerang_id,
        subaccount: Some(subaccount),
    };

    let client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id: ICP_LEDGER_ID,
    };

    let balance_e8s = match client.balance_of(boomerang_account).await {
        Ok(balance) => balance,
        Err((code, msg)) => {
            return Err(BoomerangError::BalanceOfError(format!(
                "code: {code} - message: {msg}"
            )));
        }
    };

    log!(
        INFO,
        "Fetched balance for {client_id}: {} ICP",
        balance_e8s.clone() / Nat::from(E8S)
    );

    let spender = Account {
        owner: WATER_NEURON_ID,
        subaccount: None,
    };

    let approve_args = ApproveArgs {
        from_subaccount: Some(subaccount),
        spender,
        amount: balance_e8s.clone(),
        expected_allowance: None,
        expires_at: None,
        fee: None,
        memo: None,
        created_at_time: None,
    };

    match client.approve(approve_args).await.unwrap() {
        Ok(block_index) => {
            log! {INFO, "Approved for {client_id} occured at block index: {}", block_index};
        }
        Err(error) => {
            return Err(BoomerangError::ApproveError(error));
        }
    };

    let amount: u64 = balance_e8s.clone().0.try_into().unwrap();

    let transfer_amount_e8s = amount.checked_sub(2 * TRANSFER_FEE).expect("underflow");

    let conversion_arg = ConversionArg {
        amount_e8s: transfer_amount_e8s,
        maybe_subaccount: Some(subaccount),
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
                balance_e8s.clone() / E8S,
                success.block_index
            );
            Ok(success)
        }
        Err(error) => Err(BoomerangError::ConversionError(error)),
    }
}
