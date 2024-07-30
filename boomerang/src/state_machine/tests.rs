use crate::state_machine::{
    BoomerangSetup, DEFAULT_PRINCIPAL_ID, ONE_MONTH_SECONDS, USER_PRINCIPAL_ID,
};
use crate::{E8S, TRANSFER_FEE};
use ic_state_machine_tests::PrincipalId;
use icp_ledger::AccountIdentifier;
use icrc_ledger_types::icrc1::account::Account;

#[test]
fn check_e2e() {
    let boomerang = BoomerangSetup::new();

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
        .is_ok());

        boomerang.advance_time_and_tick(60*60);

    let account_id = boomerang.get_staking_account_id(caller.0);

    assert!(boomerang
        .icp_transfer(caller.0, None, 1_000 * E8S, account_id)
        .is_ok());

    assert!(boomerang.notify_icp_deposit(caller.0).is_ok());

    boomerang.advance_time_and_tick(60*60);

    assert!(boomerang.retrieve_nicp(caller.0).is_ok());

    let balance: u64 = boomerang.nicp_balance(caller.0).0.try_into().unwrap();
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
        .unwrap();

    let account = boomerang.get_unstaking_account(caller.0);

    assert!(boomerang
        .nicp_transfer(caller.0, None, balance - TRANSFER_FEE, account)
        .is_ok());

        boomerang.advance_time_and_tick(60*60);

    assert!(boomerang.notify_nicp_deposit(caller.0).is_ok());

    assert!(boomerang.try_retrieve_icp(caller.0).is_err());

    boomerang.advance_time_and_tick(7 * ONE_MONTH_SECONDS);
    

    assert!(boomerang.try_retrieve_icp(caller.0).is_ok());


    let balance = boomerang.icp_balance(caller.0);
    assert_eq!(balance, 1_000 * E8S - 9 * TRANSFER_FEE);
}
