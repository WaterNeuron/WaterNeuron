use crate::EventType::{DisbursedMaturityNeuron, DisbursedUserNeuron};
use crate::conversion::{MINIMUM_DEPOSIT_AMOUNT, MINIMUM_WITHDRAWAL_AMOUNT};
use crate::state::{SNS_GOVERNANCE_SUBACCOUNT, WithdrawalStatus};
use crate::state_machine::{
    WaterNeuron, init_wtn_withdrawal_setup, nns_claim_or_refresh_neuron,
    nns_governance_make_proposal,
};
use crate::{
    E8S, ICP, INITIAL_NEURON_STAKE, LiquidArg, MIN_DISSOLVE_DELAY_FOR_REWARDS, ONE_DAY_SECONDS, ONE_HOUR_SECONDS, UpgradeArg, WithdrawalSuccess, nICP
};
use assert_matches::assert_matches;
use candid::{Encode, Nat};
use ic_base_types::PrincipalId;
use ic_nns_governance_api::{
    MakeProposalRequest, Motion, ProposalActionRequest,
    manage_neuron_response::Command as CommandResponse,
};
use ic_wasm_utils::water_neuron_wasm;
use icrc_ledger_types::icrc1::account::Account;

#[tokio::test]
async fn e2e_basic() {
    let mut env = WaterNeuron::new().await;
    init_wtn_withdrawal_setup(&mut env).await;

    let caller = PrincipalId::new_user_test_id(212);
    let icp_to_wrap = 100 * E8S;
    let neuron_id = nns_claim_or_refresh_neuron(&mut env, caller, 0).await;

    env.advance_time_and_tick(MIN_DISSOLVE_DELAY_FOR_REWARDS.into())
        .await;

    let caller_icp_balance_before_withdrawal = env.balance_of(env.icp_ledger_id, caller.0).await;

    env.advance_time_and_tick(ONE_DAY_SECONDS + 10).await;

    // ... + TransferExecuted
    assert_eq!(env.get_events().await.total_event_count, 5);

    // Caller swaps nICP -> ICP.
    let nicp_to_unwrap = 10 * E8S;
    match env.nicp_to_icp(caller.0.into(), nicp_to_unwrap).await {
        Ok(WithdrawalSuccess { withdrawal_id, .. }) => {
            assert_eq!(withdrawal_id, 0);
        }
        Err(e) => panic!("Expected WithdrawalSuccess, got {e:?}"),
    }

    // ... + NIcpWithdrawal
    assert_eq!(env.get_events().await.total_event_count, 6);

    assert_eq!(env.get_withdrawal_requests(caller.0).await.len(), 1);
    assert_eq!(
        env.get_withdrawal_requests(caller.0).await[0].status,
        WithdrawalStatus::WaitingToSplitNeuron
    );

    env.advance_time_and_tick(60).await;

    assert_matches!(
        env.get_withdrawal_requests(caller.0).await[0].status,
        WithdrawalStatus::WaitingDissolvement { .. }
    );

    // ... + SplitNeuron + StartedToDissolve
    assert_eq!(env.get_events().await.total_event_count, 8);

    env.advance_time_and_tick(MIN_DISSOLVE_DELAY_FOR_REWARDS.into())
        .await;

    assert_eq!(env.get_withdrawal_requests(caller.0).await.len(), 1);
    assert_eq!(
        env.get_withdrawal_requests(caller.0).await[0].status,
        WithdrawalStatus::ConversionDone {
            transfer_block_height: 10
        },
        "nICP neuron should be dissolved after the min dissolve delay"
    );

    // ... + DisbursedUserNeuron
    assert_eq!(env.get_events().await.total_event_count, 9);

    assert_eq!(
        env.balance_of(env.icp_ledger_id, caller.0).await - caller_icp_balance_before_withdrawal,
        Nat::from(9_99_980_000_u64) // 10 ICP - tx fee (approve + transfer)
    );

    env.advance_time_and_tick(ONE_DAY_SECONDS + 1).await;

    assert_eq!(
        env.balance_of(
            env.icp_ledger_id,
            Account {
                owner: env.water_neuron_id.into(),
                subaccount: Some([1; 32])
            }
        )
        .await,
        Nat::from(0_u8)
    );

    env.advance_time_and_tick(ONE_HOUR_SECONDS).await;

    let info = env.get_info().await;
    assert_eq!(info.exchange_rate, E8S);
    assert_eq!(
        info.neuron_6m_stake_e8s,
        ICP::from_e8s(INITIAL_NEURON_STAKE + 100 * E8S - 10 * E8S)
    );
    assert_eq!(info.neuron_6m_stake_e8s, info.tracked_6m_stake);
    assert_eq!(
        info.neuron_8y_stake_e8s,
        ICP::from_e8s(INITIAL_NEURON_STAKE)
    );
    assert_eq!(info.stakers_count, 1);
    assert_eq!(info.total_icp_deposited, ICP::from_e8s(icp_to_wrap));
    assert_eq!(info.minimum_deposit_amount, MINIMUM_DEPOSIT_AMOUNT);
    assert_eq!(info.minimum_withdraw_amount, MINIMUM_WITHDRAWAL_AMOUNT);
    assert!(info.neuron_id_6m.is_some());
    assert!(info.neuron_id_8y.is_some());

    // Make a proposal to generate some rewards.
    assert_eq!(
        env.balance_of(
            env.icp_ledger_id,
            Account {
                owner: env.water_neuron_id.into(),
                subaccount: Some(SNS_GOVERNANCE_SUBACCOUNT)
            }
        )
        .await,
        Nat::from(0_u8)
    );

    let neuron_6m_stake_e8s_before_proposal = env.get_info().await.neuron_6m_stake_e8s;

    let proposal = MakeProposalRequest {
        title: Some("Yellah".to_string()),
        summary: "Dummy Proposal".to_string(),
        url: "https://forum.dfinity.org/t/reevaluating-neuron-control-restrictions/28597/215"
            .to_string(),
        action: Some(ProposalActionRequest::Motion(Motion {
            motion_text: "".to_string(),
        })),
    };

    for _ in 0..8 {
        let _proposal_id =
            match nns_governance_make_proposal(&mut env, caller, neuron_id, &proposal)
                .await
                .command
                .unwrap()
            {
                CommandResponse::MakeProposal(response) => response.proposal_id.unwrap(),
                _ => panic!("unexpected response"),
            };
        env.advance_time_and_tick(ONE_HOUR_SECONDS / 4).await;
        env.advance_time_and_tick(ONE_HOUR_SECONDS / 4).await;
        env.advance_time_and_tick(4 * ONE_DAY_SECONDS - ONE_HOUR_SECONDS).await;
        dbg!(neuron_6m_stake_e8s_before_proposal, env.get_info().await.neuron_6m_stake_e8s);
    }

    let neuron_6m_stake_e8s_after_proposal = env.get_info().await.neuron_6m_stake_e8s;

    assert!(
        neuron_6m_stake_e8s_before_proposal < neuron_6m_stake_e8s_after_proposal,
        "{neuron_6m_stake_e8s_before_proposal} not less than {neuron_6m_stake_e8s_after_proposal}"
    );

    // + 8x MirroredProposal
    // + 11x MaturityNeuron (6x SnsGovernanceEightYears, 5x NICPSixMonths)
    // + 7x DisbursedMaturityNeuron
    // + 7x DispatchICPRewards
    // + 14x TransferExecuted
    // + 2x DistributeICPtoSNSv2
    assert_eq!(env.get_events().await.total_event_count, 58);

    assert!(
        env.get_events()
            .await
            .events
            .iter()
            .map(|e| &e.payload)
            .any(|payload| payload
                == &DisbursedUserNeuron {
                    withdrawal_id: 0,
                    transfer_block_height: 10,
                }),
    );

    assert_eq!(
        env.get_events()
            .await
            .events
            .iter()
            .map(|e| &e.payload)
            .filter(|payload| matches!(payload, DisbursedMaturityNeuron { .. }))
            .count(),
        7
    );

    let info = env.get_info().await;
    assert_eq!(
        env.balance_of(env.icp_ledger_id, info.neuron_6m_account)
            .await,
        Nat::from(info.tracked_6m_stake.0)
    );
    assert_eq!(info.exchange_rate, 0_00_106_010);

    assert_eq!(info.nicp_share_percent, 10);
    assert_eq!(info.governance_share_percent, 10);

    let info = env.get_info().await;
    dbg!(info.neuron_6m_stake_e8s, info.tracked_6m_stake);

    env.advance_time_and_tick(60).await;
    dbg!(
        env.balance_of(env.icp_ledger_id, info.neuron_6m_account)
            .await
    );

    assert_matches!(
        env.upgrade_canister(
            env.water_neuron_id,
            water_neuron_wasm(),
            Encode!(&LiquidArg::Upgrade(Some(UpgradeArg {
                governance_fee_share_percent: Some(20),
            })))
            .unwrap(),
        )
        .await,
        Ok(_)
    );

    env.advance_time_and_tick(60).await;
    let info = env.get_info().await;
    dbg!(
        env.balance_of(env.icp_ledger_id, info.neuron_6m_account)
            .await
    );
    assert_eq!(
        env.balance_of(env.icp_ledger_id, info.neuron_6m_account)
            .await,
        Nat::from(info.tracked_6m_stake.0)
    );
    assert_eq!(info.neuron_6m_stake_e8s, info.tracked_6m_stake);
    assert_eq!(info.exchange_rate, 0_00_106_010);
    assert_eq!(info.nicp_share_percent, 20);
    assert_eq!(info.governance_share_percent, 10);

    assert_eq!(
        env.icp_to_nicp(caller, E8S)
            .await
            .unwrap()
            .nicp_amount,
        Some(nICP::from_e8s(0_00_106_010))
    );

    assert_eq!(
        env.nicp_to_icp(caller, E8S)
            .await
            .unwrap()
            .icp_amount,
        Some(ICP::from_e8s(943_30_534_079))
    );

    assert_eq!(
        env.get_withdrawal_requests(caller.0)
            .await
            .last()
            .unwrap()
            .status,
        WithdrawalStatus::WaitingToSplitNeuron
    );

    env.advance_time_and_tick(60 * 60).await;
    env.advance_time_and_tick(60 * 60).await;

    let result = &env
        .get_withdrawal_requests(caller.0)
        .await
        .last()
        .unwrap()
        .status
        .clone();

    let neuron_id = match result.clone() {
        WithdrawalStatus::WaitingDissolvement { neuron_id } => neuron_id,
        _ => panic!("{result}"),
    };

    let full_neuron = env.get_full_neuron(neuron_id.id).await.unwrap().unwrap();
    assert_eq!(full_neuron.cached_neuron_stake_e8s, 943_30_524_079);
}
