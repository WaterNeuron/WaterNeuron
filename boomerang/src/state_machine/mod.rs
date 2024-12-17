use crate::{BoomerangError, CanisterIds, DepositSuccess, WithdrawalSuccess, E8S, TRANSFER_FEE};
use candid::{Decode, Encode, Nat, Principal};
use ic_icrc1_ledger::{InitArgsBuilder as LedgerInitArgsBuilder, LedgerArgument};
use ic_nns_constants::GOVERNANCE_CANISTER_ID;
use ic_nns_governance::pb::v1::{Governance, NetworkEconomics};
use ic_sns_governance::pb::v1::neuron::DissolveState;
use ic_sns_governance::pb::v1::{Neuron, NeuronId, NeuronPermission, NeuronPermissionType};
use ic_state_machine_tests::StateMachine;
use ic_management_canister_types::CanisterInstallMode;
use ic_base_types::{PrincipalId, CanisterId};
use icp_ledger::{
    AccountIdentifier, LedgerCanisterInitPayload, Memo, Subaccount, Tokens, TransferArgs,
    TransferError,
};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::{TransferArg, TransferError as IcrcTransferError};
use lazy_static::lazy_static;
use prost::Message;
use std::collections::HashMap;
use std::process::Command;
use utils::{
    assert_reply, boomerang_wasm, compute_neuron_staking_subaccount_bytes, governance_wasm,
    icp_ledger_wasm, ledger_wasm, setup_sns_canisters, water_neuron_wasm,
};
use water_neuron::{InitArg, LiquidArg, ONE_MONTH_SECONDS};

pub mod tests;
pub mod utils;

#[derive(Debug)]
pub struct BoomerangSetup {
    pub env: StateMachine,
    pub minter: PrincipalId,
    pub boomerang_id: CanisterId,
    pub water_neuron_id: CanisterId,
    pub wtn_ledger_id: CanisterId,
    pub icp_ledger_id: CanisterId,
    pub nicp_ledger_id: CanisterId,
}

const DEFAULT_PRINCIPAL_ID: u64 = 10352385;
const USER_PRINCIPAL_ID: u64 = 212;

impl BoomerangSetup {
    fn new() -> Self {
        let env = StateMachine::new();
        let minter = PrincipalId::new_user_test_id(DEFAULT_PRINCIPAL_ID);
        let caller = PrincipalId::new_user_test_id(USER_PRINCIPAL_ID);

        let mut initial_balances = HashMap::new();
        initial_balances.insert(
            AccountIdentifier::new(minter.into(), None),
            Tokens::from_e8s(22_000_000 * E8S),
        );
        initial_balances.insert(
            AccountIdentifier::new(caller.into(), None),
            Tokens::from_e8s(1_000 * E8S + TRANSFER_FEE),
        );

        let nicp_ledger_id = env.create_canister(None);

        let arg = Governance {
            economics: Some(NetworkEconomics::with_default_values()),
            wait_for_quiet_threshold_seconds: 60 * 60 * 24 * 4, // 4 days
            short_voting_period_seconds: 60 * 60 * 12,          // 12 hours
            neuron_management_voting_period_seconds: Some(60 * 60 * 48), // 48 hours
            ..Default::default()
        };

        let _ = env.create_canister(None);

        let icp_ledger_id = env.create_canister(None);
        env.install_existing_canister(
            icp_ledger_id,
            icp_ledger_wasm(),
            Encode!(&LedgerCanisterInitPayload::builder()
                .initial_values(initial_balances)
                .transfer_fee(Tokens::from_e8s(TRANSFER_FEE))
                .minting_account(GOVERNANCE_CANISTER_ID.get().into())
                .token_symbol_and_name("ICP", "Internet Computer")
                .feature_flags(icp_ledger::FeatureFlags { icrc2: true })
                .build()
                .unwrap())
            .unwrap(),
        )
        .unwrap();

        let water_neuron_id = env.create_canister(None);
        let water_neuron_principal = water_neuron_id.get().0;

        let mut neurons = vec![];
        neurons.push(Neuron {
            id: Some(NeuronId {
                id: compute_neuron_staking_subaccount_bytes(water_neuron_principal, 0).to_vec(),
            }),
            permissions: vec![NeuronPermission {
                principal: Some(PrincipalId(water_neuron_principal)),
                permission_type: NeuronPermissionType::all(),
            }],
            cached_neuron_stake_e8s: 1_000_000_000_000,
            dissolve_state: Some(DissolveState::DissolveDelaySeconds(25778800)),
            voting_power_percentage_multiplier: 100,
            ..Default::default()
        });

        for nonce in 0..1 {
            neurons.push(Neuron {
                id: Some(NeuronId {
                    id: compute_neuron_staking_subaccount_bytes(
                        PrincipalId::new_user_test_id(1234).0,
                        nonce,
                    )
                    .to_vec(),
                }),
                permissions: vec![NeuronPermission {
                    principal: Some(PrincipalId::new_user_test_id(1234)),
                    permission_type: NeuronPermissionType::all(),
                }],
                cached_neuron_stake_e8s: 10_000_000_000_000,
                dissolve_state: Some(DissolveState::DissolveDelaySeconds(
                    12 * ONE_MONTH_SECONDS as u64,
                )),
                voting_power_percentage_multiplier: 100,
                ..Default::default()
            });
        }

        let sns = setup_sns_canisters(&env, neurons);

        env.install_wasm_in_mode(
            water_neuron_id,
            CanisterInstallMode::Install,
            water_neuron_wasm(),
            Encode!(&LiquidArg::Init(InitArg {
                wtn_governance_id: sns.governance.into(),
                wtn_ledger_id: sns.ledger.into(),
                nicp_ledger_id: nicp_ledger_id.get().0,
            }))
            .unwrap(),
        )
        .unwrap();

        env.install_wasm_in_mode(
            nicp_ledger_id,
            CanisterInstallMode::Install,
            ledger_wasm(),
            Encode!(&LedgerArgument::Init(
                LedgerInitArgsBuilder::with_symbol_and_name("nICP", "nICP")
                    .with_minting_account(water_neuron_id.get().0)
                    .with_transfer_fee(TRANSFER_FEE)
                    .with_decimals(8)
                    .with_feature_flags(ic_icrc1_ledger::FeatureFlags { icrc2: true })
                    .build(),
            ))
            .unwrap(),
        )
        .unwrap();

        let boomerang_id = env
            .install_canister(
                boomerang_wasm(),
                Encode!(
                    &(CanisterIds {
                        water_neuron_id: water_neuron_id.into(),
                        icp_ledger_id: icp_ledger_id.into(),
                        nicp_ledger_id: nicp_ledger_id.into(),
                        wtn_ledger_id: sns.ledger.into()
                    })
                )
                .unwrap(),
                None,
            )
            .unwrap();

        Self {
            env,
            minter,
            boomerang_id,
            water_neuron_id,
            wtn_ledger_id: sns.ledger,
            icp_ledger_id,
            nicp_ledger_id,
        }
    }

    pub fn icp_transfer(
        &self,
        caller: Principal,
        from_subaccount: Option<Subaccount>,
        transfer_amount: u64,
        target: AccountIdentifier,
    ) -> Result<u64, TransferError> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        caller.into(),
                        self.icp_ledger_id,
                        "transfer",
                        Encode!(
                            &(TransferArgs {
                                memo: Memo(0),
                                amount: Tokens::from_e8s(transfer_amount),
                                fee: Tokens::from_e8s(TRANSFER_FEE),
                                from_subaccount,
                                created_at_time: None,
                                to: target.to_address(),
                            })
                        )
                        .unwrap()
                    )
                    .expect("failed canister call in icp_transfer")
            ),
            Result<u64, TransferError>
        )
        .expect("failed to decode result in icp_transfer")
    }

    pub fn advance_time_and_tick(&self, seconds: u64) {
        self.env
            .advance_time(std::time::Duration::from_secs(seconds));
        const MAX_TICKS: u8 = 10;
        for _ in 0..MAX_TICKS {
            self.env.tick();
        }
    }

    pub fn nicp_transfer(
        &self,
        caller: Principal,
        from_subaccount: Option<[u8; 32]>,
        transfer_amount: u64,
        target: Account,
    ) -> Result<Nat, IcrcTransferError> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        caller.into(),
                        self.nicp_ledger_id,
                        "icrc1_transfer",
                        Encode!(
                            &(TransferArg {
                                memo: None,
                                amount: transfer_amount.into(),
                                fee: None,
                                from_subaccount,
                                created_at_time: None,
                                to: target,
                            })
                        )
                        .unwrap()
                    )
                    .expect("failed canister call in nicp_transfer")
            ),
            Result<Nat, IcrcTransferError>
        )
        .expect("failed to decode result in nicp_transfer")
    }

    pub fn notify_icp_deposit(&self, caller: Principal) -> Result<DepositSuccess, BoomerangError> {
        Decode!(
            &assert_reply(
                    self.env.execute_ingress_as(
                        caller.into(),
                        self.boomerang_id,
                        "notify_icp_deposit",
                        Encode!(&(caller)).unwrap()
                    )
                    .expect("failed canister call in notify_icp_deposit")
            ),
            Result<DepositSuccess, BoomerangError>
        )
        .expect("failed to decode result in notify_icp_deposit")
    }

    pub fn notify_nicp_deposit(
        &self,
        caller: Principal,
    ) -> Result<WithdrawalSuccess, BoomerangError> {
        Decode!(
            &assert_reply(
                    self.env.execute_ingress_as(
                        caller.into(),
                        self.boomerang_id,
                        "notify_nicp_deposit",
                        Encode!(&(caller)).unwrap()
                    )
                    .expect("failed canister call in notify_nicp_deposit")
            ),
            Result<WithdrawalSuccess, BoomerangError>
        )
        .expect("failed to decode result in notify_nicp_deposit")
    }

    fn get_staking_account(&self, caller: Principal) -> Account {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        caller.into(),
                        self.boomerang_id,
                        "get_staking_account",
                        Encode!(&(caller)).unwrap()
                    )
                    .expect("failed canister call in get_staking_account")
            ),
            Account
        )
        .expect("failed to decode result in get_staking_account")
    }

    fn get_unstaking_account(&self, caller: Principal) -> Account {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        caller.into(),
                        self.boomerang_id,
                        "get_unstaking_account",
                        Encode!(&(caller)).unwrap()
                    )
                    .expect("failed canister call in get_unstaking_account")
            ),
            Account
        )
        .expect("failed to decode result in get_unstaking_account")
    }

    pub fn icp_balance(&self, caller: Principal) -> Nat {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        caller.into(),
                        self.icp_ledger_id,
                        "icrc1_balance_of",
                        Encode!(
                            &(Account {
                                owner: caller,
                                subaccount: None
                            })
                        )
                        .unwrap()
                    )
                    .expect("failed canister call in icp_balance")
            ),
            Nat
        )
        .expect("failed to decode result in icp_balance")
    }

    pub fn nicp_balance(&self, caller: Principal) -> Nat {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        caller.into(),
                        self.nicp_ledger_id,
                        "icrc1_balance_of",
                        Encode!(
                            &(Account {
                                owner: caller,
                                subaccount: None
                            })
                        )
                        .unwrap()
                    )
                    .expect("failed canister call in nicp_balance")
            ),
            Nat
        )
        .expect("failed to decode result in nicp_balance")
    }

    pub fn retrieve_nicp(&self, caller: Principal) -> Result<Nat, BoomerangError> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        caller.into(),
                        self.boomerang_id,
                        "retrieve_nicp",
                        Encode!(&(caller)).unwrap()
                    )
                    .expect("failed canister call in retrieve_nicp")
            ),
            Result<Nat, BoomerangError>
        )
        .expect("failed to decode result in retrieve_nicp")
    }

    pub fn try_retrieve_icp(&self, caller: Principal) -> Result<Nat, BoomerangError> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        caller.into(),
                        self.boomerang_id,
                        "try_retrieve_icp",
                        Encode!(&(caller)).unwrap()
                    )
                    .expect("failed canister call in try_retrieve_icp")
            ),
            Result<Nat, BoomerangError>
        )
        .expect("failed to decode result in try_retrieve_icp")
    }
}
