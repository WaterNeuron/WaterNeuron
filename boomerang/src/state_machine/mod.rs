use crate::{BoomerangError, CanisterIds, DepositSuccess, E8S, TRANSFER_FEE, WithdrawalSuccess};
use candid::{CandidType, Deserialize, Encode, Nat, Principal};
use ic_base_types::PrincipalId;
use ic_icrc1_ledger::{InitArgsBuilder as LedgerInitArgsBuilder, LedgerArgument};
use ic_nns_constants::GOVERNANCE_CANISTER_ID;
use ic_nns_governance_api::{Governance, NetworkEconomics};
use ic_sns_governance::pb::v1::{
    Neuron, NeuronId, NeuronPermission, NeuronPermissionType, neuron::DissolveState,
};
use ic_wasm_utils::{
    boomerang_wasm, governance_wasm, icp_ledger_wasm, ledger_wasm, water_neuron_wasm,
};
use icp_ledger::{
    AccountIdentifier, LedgerCanisterInitPayload, Memo, Subaccount, Tokens, TransferArgs,
    TransferError,
};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::{TransferArg, TransferError as IcrcTransferError};
use pocket_ic::{
    PocketIcBuilder, RejectResponse,
    nonblocking::{PocketIc, query_candid_as, update_candid_as},
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use utils::{compute_neuron_staking_subaccount_bytes, setup_sns_canisters};
use water_neuron::state::ICP_LEDGER_ID;
use water_neuron::{InitArg, LiquidArg, ONE_MONTH_SECONDS};

pub mod tests;
pub mod utils;

pub async fn update<T>(
    pic: &PocketIc,
    canister: Principal,
    caller: Principal,
    method: &str,
    arg: impl CandidType,
) -> Result<T, String>
where
    T: for<'a> Deserialize<'a> + CandidType,
{
    let r: Result<(T,), RejectResponse> =
        update_candid_as(pic, canister, caller, method, (arg,)).await;
    match r {
        Ok((r,)) => Ok(r),
        Err(e) => Err(format!(
            "Failed to call {method} of {canister} with error: {e}"
        )),
    }
}

pub async fn query<T>(
    pic: &PocketIc,
    canister: Principal,
    caller: Principal,
    method: &str,
    arg: impl CandidType,
) -> Result<T, String>
where
    T: for<'a> Deserialize<'a> + CandidType,
{
    let r: Result<(T,), RejectResponse> =
        query_candid_as(pic, canister, caller, method, (arg,)).await;
    match r {
        Ok((r,)) => Ok(r),
        Err(e) => Err(format!(
            "Failed to call {method} of {canister} with error: {e}"
        )),
    }
}

pub struct BoomerangSetup {
    pub env: Arc<Mutex<PocketIc>>,
    pub minter: PrincipalId,
    pub boomerang_id: Principal,
    pub water_neuron_id: Principal,
    pub wtn_ledger_id: Principal,
    pub icp_ledger_id: Principal,
    pub nicp_ledger_id: Principal,
}

const DEFAULT_PRINCIPAL_ID: u64 = 10352385;
const USER_PRINCIPAL_ID: u64 = 212;

impl BoomerangSetup {
    async fn new() -> Self {
        let env = PocketIcBuilder::new()
            .with_nns_subnet()
            .with_sns_subnet()
            .with_ii_subnet()
            .build_async()
            .await;
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

        let nicp_ledger_id = env.create_canister().await;

        env.add_cycles(nicp_ledger_id, u64::MAX.into()).await;

        let governance_canister_init = Governance {
            economics: Some(NetworkEconomics::with_default_values()),
            wait_for_quiet_threshold_seconds: 60 * 60 * 24 * 4, // 4 days
            short_voting_period_seconds: 60 * 60 * 12,          // 12 hours
            neuron_management_voting_period_seconds: Some(60 * 60 * 48), // 48 hours
            ..Default::default()
        };

        let encoded = Encode!(&governance_canister_init).unwrap();

        let governance_id = env
            .create_canister_with_id(None, None, GOVERNANCE_CANISTER_ID.into())
            .await
            .unwrap();

        env.add_cycles(governance_id, u64::MAX.into()).await;

        env.install_canister(governance_id, governance_wasm().await, encoded, None)
            .await;

        let icp_ledger_id = env
            .create_canister_with_id(None, None, ICP_LEDGER_ID.into())
            .await
            .unwrap();
        env.add_cycles(icp_ledger_id, u64::MAX.into()).await;
        env.install_canister(
            icp_ledger_id,
            icp_ledger_wasm().await,
            Encode!(
                &LedgerCanisterInitPayload::builder()
                    .initial_values(initial_balances)
                    .transfer_fee(Tokens::from_e8s(TRANSFER_FEE))
                    .minting_account(GOVERNANCE_CANISTER_ID.get().into())
                    .token_symbol_and_name("ICP", "Internet Computer")
                    .feature_flags(icp_ledger::FeatureFlags { icrc2: true })
                    .build()
                    .unwrap()
            )
            .unwrap(),
            None,
        )
        .await;

        let water_neuron_id = env.create_canister().await;
        env.add_cycles(water_neuron_id, u64::MAX.into()).await;
        let water_neuron_principal = water_neuron_id;

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

        let sns = setup_sns_canisters(&env, neurons).await;

        env.install_canister(
            water_neuron_id,
            water_neuron_wasm(),
            Encode!(&LiquidArg::Init(InitArg {
                wtn_governance_id: sns.governance.into(),
                wtn_ledger_id: sns.ledger.into(),
                nicp_ledger_id: nicp_ledger_id,
            }))
            .unwrap(),
            None,
        )
        .await;

        env.install_canister(
            nicp_ledger_id,
            ledger_wasm().await,
            Encode!(&LedgerArgument::Init(
                LedgerInitArgsBuilder::with_symbol_and_name("nICP", "nICP")
                    .with_minting_account(water_neuron_id)
                    .with_transfer_fee(TRANSFER_FEE)
                    .with_decimals(8)
                    .with_feature_flags(ic_icrc1_ledger::FeatureFlags { icrc2: true })
                    .build(),
            ))
            .unwrap(),
            None,
        )
        .await;

        let boomerang_id = env.create_canister().await;
        env.add_cycles(boomerang_id, u64::MAX.into()).await;

        env.install_canister(
            boomerang_id,
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
        .await;

        Self {
            env: Arc::new(Mutex::new(env)),
            minter,
            boomerang_id,
            water_neuron_id,
            wtn_ledger_id: sns.ledger,
            icp_ledger_id,
            nicp_ledger_id,
        }
    }

    pub async fn icp_transfer(
        &self,
        caller: Principal,
        from_subaccount: Option<Subaccount>,
        transfer_amount: u64,
        target: AccountIdentifier,
    ) -> Result<u64, TransferError> {
        let pic = self.env.lock().await;
        update::<Result<u64, TransferError>>(
            &pic,
            self.icp_ledger_id,
            caller.into(),
            "transfer",
            TransferArgs {
                memo: Memo(0),
                amount: Tokens::from_e8s(transfer_amount),
                fee: Tokens::from_e8s(TRANSFER_FEE),
                from_subaccount,
                created_at_time: None,
                to: target.to_address(),
            },
        )
        .await
        .expect("failed canister call in icp_transfer")
    }

    pub async fn advance_time_and_tick(&self, seconds: u64) {
        let pic = self.env.lock().await;
        pic.advance_time(std::time::Duration::from_secs(seconds))
            .await;
        const MAX_TICKS: u8 = 10;
        for _ in 0..MAX_TICKS {
            pic.tick().await;
        }
    }

    pub async fn nicp_transfer(
        &self,
        caller: Principal,
        from_subaccount: Option<[u8; 32]>,
        transfer_amount: u64,
        target: Account,
    ) -> Result<Nat, IcrcTransferError> {
        let pic = self.env.lock().await;
        update::<Result<Nat, IcrcTransferError>>(
            &pic,
            self.nicp_ledger_id,
            caller.into(),
            "icrc1_transfer",
            TransferArg {
                memo: None,
                amount: transfer_amount.into(),
                fee: None,
                from_subaccount,
                created_at_time: None,
                to: target,
            },
        )
        .await
        .expect("failed canister call in nicp_transfer")
    }

    pub async fn notify_icp_deposit(
        &self,
        caller: Principal,
    ) -> Result<DepositSuccess, BoomerangError> {
        let pic = self.env.lock().await;
        update::<Result<DepositSuccess, BoomerangError>>(
            &pic,
            self.boomerang_id,
            caller.into(),
            "notify_icp_deposit",
            caller,
        )
        .await
        .expect("failed to decode result in notify_icp_deposit")
    }

    pub async fn notify_nicp_deposit(
        &self,
        caller: Principal,
    ) -> Result<WithdrawalSuccess, BoomerangError> {
        let pic = self.env.lock().await;
        update::<Result<WithdrawalSuccess, BoomerangError>>(
            &pic,
            self.boomerang_id,
            caller.into(),
            "notify_nicp_deposit",
            caller,
        )
        .await
        .expect("failed to decode result in notify_icp_deposit")
    }

    async fn get_staking_account(&self, caller: Principal) -> Account {
        let pic = self.env.lock().await;
        update::<Account>(
            &pic,
            self.boomerang_id,
            caller.into(),
            "get_staking_account",
            caller,
        )
        .await
        .expect("failed to decode result in notify_icp_deposit")
    }

    async fn get_unstaking_account(&self, caller: Principal) -> Account {
        let pic = self.env.lock().await;
        update::<Account>(
            &pic,
            self.boomerang_id,
            caller.into(),
            "get_unstaking_account",
            caller,
        )
        .await
        .expect("failed to decode result in notify_icp_deposit")
    }

    pub async fn icp_balance(&self, caller: Principal) -> Nat {
        let pic = self.env.lock().await;
        update::<Nat>(
            &pic,
            self.icp_ledger_id,
            caller.into(),
            "icrc1_balance_of",
            &(Account {
                owner: caller,
                subaccount: None,
            }),
        )
        .await
        .expect("failed to decode result in notify_icp_deposit")
    }

    pub async fn nicp_balance(&self, caller: Principal) -> Nat {
        let pic = self.env.lock().await;
        update::<Nat>(
            &pic,
            self.nicp_ledger_id,
            caller.into(),
            "icrc1_balance_of",
            &(Account {
                owner: caller,
                subaccount: None,
            }),
        )
        .await
        .expect("failed to decode result in nicp_balance")
    }

    pub async fn retrieve_nicp(&self, caller: Principal) -> Result<Nat, BoomerangError> {
        let pic = self.env.lock().await;
        update::<Result<Nat, BoomerangError>>(
            &pic,
            self.boomerang_id,
            caller.into(),
            "retrieve_nicp",
            caller,
        )
        .await
        .expect("failed to decode result in retrieve_nicp")
    }

    pub async fn try_retrieve_icp(&self, caller: Principal) -> Result<Nat, BoomerangError> {
        let pic = self.env.lock().await;
        update::<Result<Nat, BoomerangError>>(
            &pic,
            self.boomerang_id,
            caller.into(),
            "try_retrieve_icp",
            caller,
        )
        .await
        .expect("failed to decode result in retrieve_nicp")
    }
}
