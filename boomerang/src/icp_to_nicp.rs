use crate::log::INFO;
use crate::{
    get_canister_ids, self_canister_id, BoomerangError, ConversionArg, ConversionError,
    DepositSuccess, E8S, TRANSFER_FEE,
};
use candid::{Nat, Principal};
use ic_canister_log::log;
use icrc_ledger_client_cdk::{CdkRuntime, ICRC1Client};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::TransferArg;
use icrc_ledger_types::icrc2::approve::ApproveArgs;
use sha2::{Digest, Sha256};

pub fn derive_subaccount_staking(principal: Principal) -> [u8; 32] {
    const DOMAIN: &[u8] = b"STAKE-ICP";

    let mut hasher = Sha256::new();
    hasher.update(DOMAIN);
    hasher.update(principal.as_slice());
    hasher.finalize().into()
}

pub async fn retrieve_nicp(target: Principal) -> Result<Nat, BoomerangError> {
    let canister_ids = get_canister_ids();
    let nicp_client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id: canister_ids.nicp_ledger_id,
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
            return Err(BoomerangError::GenericError { code, message });
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
    let canister_ids = get_canister_ids();
    let boomerang_id = self_canister_id();
    let subaccount = derive_subaccount_staking(target);

    let boomerang_account = Account {
        owner: boomerang_id,
        subaccount: Some(subaccount),
    };

    let client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id: canister_ids.icp_ledger_id,
    };

    let balance_e8s: u64 = match client.balance_of(boomerang_account).await {
        Ok(balance) => balance.0.try_into().unwrap(),
        Err((code, message)) => {
            return Err(BoomerangError::GenericError { code, message });
        }
    };

    log!(
        INFO,
        "Fetched balance for {target}: {} ICP",
        balance_e8s / E8S
    );

    let spender = Account {
        owner: canister_ids.water_neuron_id,
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

    let conversion_result: (Result<DepositSuccess, ConversionError>,) = ic_cdk::call(
        canister_ids.water_neuron_id,
        "icp_to_nicp",
        (conversion_arg,),
    )
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

#[test]
fn should_return_different_array() {
    use crate::nicp_to_icp::derive_subaccount_unstaking;

    let p = Principal::anonymous();

    assert_ne!(derive_subaccount_staking(p), derive_subaccount_unstaking(p));

    let p1 =
        Principal::from_text("xwpbi-y7r63-dbg7j-ukl5y-5ncft-j5zsv-6uca6-rj5ly-e5xa7-qjlm3-xqe")
            .unwrap();
    let p2 =
        Principal::from_text("i57ky-ppa5u-2nmqo-ngzn6-3y6pl-4jqv2-b44iu-kdix5-76gp3-vxfjz-kqe")
            .unwrap();

    assert_ne!(derive_subaccount_staking(p1), derive_subaccount_staking(p2));
    assert_ne!(
        derive_subaccount_unstaking(p1),
        derive_subaccount_unstaking(p2)
    );
}
