use crate::log::INFO;
use crate::{
    derive_subaccount_unstaking, self_canister_id, BoomerangError, ConversionArg, ConversionError,
    WithdrawalSuccess, E8S, ICP_LEDGER_ID, NICP_LEDGER_ID, TRANSFER_FEE, WATER_NEURON_ID,
};
use candid::{Nat, Principal};
use ic_canister_log::log;
use icrc_ledger_client_cdk::{CdkRuntime, ICRC1Client};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::TransferArg;
use icrc_ledger_types::icrc2::approve::ApproveArgs;

pub async fn notify_nicp_deposit(target: Principal) -> Result<WithdrawalSuccess, BoomerangError> {
    let subaccount = derive_subaccount_unstaking(target);

    let boomerang_account = Account {
        owner: self_canister_id(),
        subaccount: Some(subaccount),
    };

    let client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id: NICP_LEDGER_ID,
    };

    let balance_e8s: u64 = match client.balance_of(boomerang_account).await {
        Ok(balance) => balance.0.try_into().unwrap(),
        Err((code, msg)) => {
            return Err(BoomerangError::BalanceOfError(format!(
                "code: {code} - message: {msg}"
            )));
        }
    };

    log!(
        INFO,
        "Fetched balance for {target}: {} ICP",
        balance_e8s / E8S
    );

    let approve_args = ApproveArgs {
        from_subaccount: boomerang_account.subaccount,
        spender: WATER_NEURON_ID.into(),
        amount: balance_e8s.into(),
        expected_allowance: None,
        expires_at: None,
        fee: None,
        memo: None,
        created_at_time: None,
    };

    match client.approve(approve_args).await.unwrap() {
        Ok(block_index) => {
            log! {INFO, "Approved for {target} occured at block index: {}", block_index};
        }
        Err(error) => {
            return Err(BoomerangError::ApproveError(error));
        }
    };

    let transfer_amount_e8s = balance_e8s
        .checked_sub(2 * TRANSFER_FEE)
        .expect("underflow");

    let conversion_arg = ConversionArg {
        amount_e8s: transfer_amount_e8s,
        maybe_subaccount: boomerang_account.subaccount,
    };

    let conversion_result: (Result<WithdrawalSuccess, ConversionError>,) =
        ic_cdk::call(WATER_NEURON_ID, "nicp_to_icp", (conversion_arg,))
            .await
            .unwrap();

    match conversion_result.0 {
        Ok(success) => {
            log!(
                INFO,
                "Successful conversion ({} nICP) at block index {}. Withdrawal id: {}.",
                balance_e8s / E8S,
                success.block_index,
                success.withdrawal_id,
            );
            Ok(success)
        }
        Err(error) => Err(BoomerangError::ConversionError(error)),
    }
}

pub async fn try_retrieve_icp(target: Principal) -> Result<Nat, BoomerangError> {
    let icp_client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id: ICP_LEDGER_ID,
    };

    let boomerang_id = self_canister_id();
    let subaccount = derive_subaccount_unstaking(target);

    let target_account = Account {
        owner: boomerang_id,
        subaccount: Some(subaccount),
    };

    let icp_balance_e8s: u64 = match icp_client.balance_of(target_account).await {
        Ok(balance) => balance.0.try_into().unwrap(),
        Err((code, msg)) => {
            return Err(BoomerangError::BalanceOfError(format!(
                "code: {code} - msg: {msg}"
            )));
        }
    };

    if icp_balance_e8s == 0 {
        return Err(BoomerangError::IcpNotAvailable);
    }

    let to_transfer_amount = icp_balance_e8s.checked_sub(TRANSFER_FEE).unwrap();

    match icp_client
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
