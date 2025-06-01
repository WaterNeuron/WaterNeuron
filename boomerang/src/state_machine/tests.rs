use crate::state_machine::{
    BoomerangSetup, DEFAULT_PRINCIPAL_ID, ONE_MONTH_SECONDS, USER_PRINCIPAL_ID,
};
use crate::{E8S, TRANSFER_FEE};
use ic_base_types::PrincipalId;
use icp_ledger::{AccountIdentifier, Subaccount};
use icrc_ledger_types::icrc1::account::Account;

#[tokio::test]
async fn check_e2e() {
    let boomerang = BoomerangSetup::new().await;

    let caller = PrincipalId::new_user_test_id(USER_PRINCIPAL_ID);
    let minter = PrincipalId::new_user_test_id(DEFAULT_PRINCIPAL_ID);

    // WaterNeuron initialization
    assert!(boomerang
        .icp_transfer(
            minter.0,
            None,
            3 * E8S,
            AccountIdentifier::new(boomerang.water_neuron_id.into(), None)
        )
        .await
        .is_ok());

    boomerang.advance_time_and_tick(60 * 60).await;

    let staking_account = boomerang.get_staking_account(caller.0).await;

    assert!(boomerang
        .icp_transfer(
            caller.0,
            None,
            1_000 * E8S,
            AccountIdentifier::new(
                staking_account.owner.into(),
                staking_account.subaccount.map(|s| Subaccount(s))
            )
        )
        .await
        .is_ok());

    assert!(boomerang.notify_icp_deposit(caller.0).await.is_ok());

    boomerang.advance_time_and_tick(60 * 60).await;

    assert!(boomerang.notify_nicp_deposit(caller.0).await.is_err());
    assert!(boomerang.retrieve_nicp(caller.0).await.is_ok());

    let balance: u64 = boomerang.nicp_balance(caller.0).await.0.try_into().unwrap();
    assert_eq!(balance, 1_000 * E8S - 3 * TRANSFER_FEE);

    boomerang
        .nicp_transfer(
            boomerang.water_neuron_id.into(),
            None,
            balance,
            Account {
                owner: caller.0,
                subaccount: None,
            },
        )
        .await
        .unwrap();

    let unstaking_account = boomerang.get_unstaking_account(caller.0).await;

    assert_ne!(staking_account, unstaking_account);

    assert!(boomerang
        .nicp_transfer(caller.0, None, balance - TRANSFER_FEE, unstaking_account)
        .await
        .is_ok());
    boomerang.advance_time_and_tick(60 * 60).await;

    assert!(boomerang.notify_nicp_deposit(caller.0).await.is_ok());
    boomerang.advance_time_and_tick(60 * 60).await;

    assert!(boomerang.try_retrieve_icp(caller.0).await.is_err());
    boomerang.advance_time_and_tick(7 * ONE_MONTH_SECONDS).await;

    assert!(boomerang.try_retrieve_icp(caller.0).await.is_ok());

    let balance = boomerang.icp_balance(caller.0).await;
    assert_eq!(balance, 1_000 * E8S - 9 * TRANSFER_FEE);
}
