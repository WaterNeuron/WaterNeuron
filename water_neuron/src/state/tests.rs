use crate::state::event::Event;
use crate::{nICP, Account, EventType, InitArg, NeuronOrigin, UpgradeArg, ICP};
use candid::Principal;
use proptest::array::uniform32;
use proptest::collection::vec as pvec;
use proptest::prelude::*;

fn arb_event() -> impl Strategy<Value = Event> {
    (any::<u64>(), arb_event_type()).prop_map(|(timestamp, payload)| Event { timestamp, payload })
}

fn arb_principal() -> impl Strategy<Value = Principal> {
    pvec(any::<u8>(), 0..=29).prop_map(|bytes| Principal::from_slice(&bytes))
}

fn arb_subaccount() -> impl Strategy<Value = Option<[u8; 32]>> {
    proptest::option::of(uniform32(any::<u8>()))
}

prop_compose! {
    fn arb_account()(
        owner in arb_principal(),
        subaccount in arb_subaccount(),
    ) -> Account {
        Account {
            owner,
            subaccount: subaccount,
        }
    }
}

prop_compose! {
    fn arb_init_arg()(
        nicp_ledger_id in arb_principal(),
        wtn_ledger_id in arb_principal(),
        wtn_governance_id in arb_principal(),
    ) -> InitArg {
        InitArg {
            nicp_ledger_id,
            wtn_ledger_id,
            wtn_governance_id
        }
    }
}

prop_compose! {
    fn arb_upgrade_arg()(
        governance_fee_share_percent in proptest::option::of(any::<u64>()),
    ) -> UpgradeArg {
        UpgradeArg {
            governance_fee_share_percent
        }
    }
}

fn arb_event_type() -> impl Strategy<Value = EventType> {
    prop_oneof![
        arb_init_arg().prop_map(EventType::Init),
        arb_upgrade_arg().prop_map(EventType::Upgrade),
        (any::<u64>(), arb_principal()).prop_map(|(amount, receiver)| {
            EventType::DistributeICPtoSNS {
                receiver,
                amount: ICP::from_e8s(amount),
            }
        }),
        (any::<u64>(), any::<u64>()).prop_map(|(transfer_id, block_index)| {
            EventType::TransferExecuted {
                transfer_id,
                block_index: Some(block_index),
            }
        }),
        (arb_account(), any::<u64>(), any::<u64>()).prop_map(|(receiver, amount, block_index)| {
            EventType::IcpDeposit {
                receiver,
                amount: ICP::from_e8s(amount),
                block_index,
            }
        }),
        (arb_account(), any::<u64>(), any::<u64>()).prop_map(
            |(receiver, nicp_burned, nicp_burn_index)| {
                EventType::NIcpWithdrawal {
                    receiver,
                    nicp_burned: nICP::from_e8s(nicp_burned),
                    nicp_burn_index,
                }
            }
        ),
        (any::<u64>(), any::<u64>()).prop_map(|(nicp_amount, sns_gov_amount)| {
            EventType::DispatchICPRewards {
                nicp_amount: ICP::from_e8s(nicp_amount),
                sns_gov_amount: ICP::from_e8s(sns_gov_amount),
                from_neuron_type: NeuronOrigin::SnsGovernanceEightYears,
            }
        }),
    ]
}

proptest! {
    #[test]
    fn event_encoding_roundtrip(event in arb_event()) {
        use ic_stable_structures::storable::Storable;
        let bytes = event.to_bytes();
        prop_assert_eq!(&event, &Event::from_bytes(bytes.clone()), "failed to decode bytes {}", hex::encode(bytes));
    }
}