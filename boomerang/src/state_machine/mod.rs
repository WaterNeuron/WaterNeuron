use crate::{BoomerangError, DepositSuccess, WithdrawalSuccess, E8S, TRANSFER_FEE};
use candid::{CandidType, Decode, Deserialize, Encode, Nat, Principal};
use ic_icrc1_ledger::{
    ArchiveOptions, InitArgs as LedgerInitArgs, InitArgsBuilder as LedgerInitArgsBuilder,
    LedgerArgument,
};
use ic_nns_constants::{GOVERNANCE_CANISTER_ID, LEDGER_CANISTER_ID};

use ic_nns_governance::pb::v1::{Governance, NetworkEconomics};
use ic_sns_governance::init::GovernanceCanisterInitPayloadBuilder;
use ic_sns_governance::pb::v1::governance::Version;
use ic_sns_governance::pb::v1::neuron::DissolveState;
use ic_sns_governance::pb::v1::{Neuron, NeuronId, NeuronPermission, NeuronPermissionType};
use ic_sns_init::SnsCanisterInitPayloads;
use ic_sns_root::pb::v1::SnsRootCanister;
use ic_sns_swap::pb::v1::{Init as SwapInit, NeuronBasketConstructionParameters};
use ic_state_machine_tests::{
    CanisterId, CanisterInstallMode, PrincipalId, StateMachine, WasmResult,
};
use icp_ledger::{
    AccountIdentifier, LedgerCanisterInitPayload, Memo, Subaccount, Tokens, TransferArgs,
    TransferError,
};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::{TransferArg, TransferError as IcrcTransferError};
use lazy_static::lazy_static;
use prost::Message;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::process::Command;

pub mod tests;

const ONE_DAY_SECONDS: u64 = 24 * 60 * 60;
const ONE_YEAR_SECONDS: u64 = (4 * 365 + 1) * ONE_DAY_SECONDS / 4;
const ONE_MONTH_SECONDS: u64 = ONE_YEAR_SECONDS / 12;

const NEURON_LEDGER_FEE: u64 = 1_000_000;

lazy_static! {
    static ref CARGO_BUILD_RESULT: Result<(), std::io::Error> = cargo_build();
}

fn get_wasm(dir: &str) -> Vec<u8> {
    let _ = *CARGO_BUILD_RESULT;
    let current_dir = std::env::current_dir().unwrap();
    let file_path = current_dir.join(dir);
    std::fs::read(file_path).unwrap()
}

fn boomerang_wasm() -> Vec<u8> {
    get_wasm("./target/wasm32-unknown-unknown/debug/boomerang.wasm")
}

fn icp_ledger_wasm() -> Vec<u8> {
    get_wasm("LEDGER_CANISTER_WASM_PATH")
}

fn governance_wasm() -> Vec<u8> {
    get_wasm("GOVERNANCE_CANISTER_WASM_PATH")
}

fn water_neuron_wasm() -> Vec<u8> {
    get_wasm("WATER_NEURON_CANISTER_WASM_PATH")
}

fn ledger_wasm() -> Vec<u8> {
    get_wasm("IC_ICRC1_LEDGER_WASM_PATH")
}

fn sns_governance() -> Vec<u8> {
    get_wasm("SNS_GOVERNANCE_CANISTER_WASM_PATH")
}

fn sns_root() -> Vec<u8> {
    get_wasm("SNS_ROOT_CANISTER_WASM_PATH")
}

fn sns_swap() -> Vec<u8> {
    get_wasm("SNS_SWAP_CANISTER_WASM_PATH")
}

fn cargo_build() -> Result<(), std::io::Error> {
    Command::new("cargo")
        .args(&[
            "build",
            "--target",
            "wasm32-unknown-unknown",
            "--release",
            "-p",
            "boomerang",
            "--locked",
        ])
        .spawn()?
        .wait()?;
    Ok(())
}

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

#[derive(Deserialize, CandidType)]
enum LiquidArg {
    Init(InitArg),
    Upgrade(Option<UpgradeArg>),
}

#[derive(Deserialize, CandidType, PartialEq, Eq, Clone, Debug)]
pub struct InitArg {
    nicp_ledger_id: Principal,
    pub wtn_governance_id: Principal,
    pub wtn_ledger_id: Principal,
}

#[derive(Deserialize, CandidType, PartialEq, Eq, Clone, Debug)]
pub struct UpgradeArg {
    pub governance_fee_share_percent: Option<u64>,
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
        let _governance_id = env
            .install_canister(governance_wasm(), arg.encode_to_vec(), None)
            .unwrap();

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
            .install_canister(boomerang_wasm(), Encode!(&()).unwrap(), None)
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

    fn get_staking_account_id(&self, caller: Principal) -> AccountIdentifier {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        caller.into(),
                        self.boomerang_id,
                        "get_staking_account_id",
                        Encode!(&(caller)).unwrap()
                    )
                    .expect("failed canister call in get_staking_account_id")
            ),
            AccountIdentifier
        )
        .expect("failed to decode result in get_staking_account_id")
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

pub fn sha256_hash(data: Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(&data);
    hasher.finalize().to_vec()
}

struct SnsCanisterIds {
    governance: CanisterId,
    ledger: CanisterId,
}

/// Builder to help create the initial payloads for the SNS canisters in tests.
pub struct SnsTestsInitPayloadBuilder {
    pub governance: GovernanceCanisterInitPayloadBuilder,
    pub ledger: LedgerInitArgs,
    pub root: SnsRootCanister,
    pub swap: SwapInit,
}

/// Caveat emptor: Even though sns-wasm creates SNS governance in
/// PreInitializationSwap mode, this uses Normal mode as the default. Use the
/// with_governance_mode method to initialize SNS governance in
/// PreInitializationSwap, like what sns-wasm does.
#[allow(clippy::new_without_default)]
impl SnsTestsInitPayloadBuilder {
    pub fn new() -> SnsTestsInitPayloadBuilder {
        let ledger = LedgerInitArgsBuilder::for_tests()
            .with_minting_account(Principal::anonymous()) // will be set when the Governance canister ID is allocated
            .with_archive_options(ArchiveOptions {
                trigger_threshold: 2000,
                num_blocks_to_archive: 1000,
                // 1 GB, which gives us 3 GB space when upgrading
                node_max_memory_size_bytes: Some(1024 * 1024 * 1024),
                // 128kb
                max_message_size_bytes: Some(128 * 1024),
                // controller_id will be set when the Root canister ID is allocated
                controller_id: CanisterId::from_u64(0).into(),
                more_controller_ids: None,
                cycles_for_archive_creation: Some(0),
                max_transactions_per_response: None,
            })
            .with_transfer_fee(NEURON_LEDGER_FEE)
            .build();

        let swap = SwapInit {
            fallback_controller_principal_ids: vec![PrincipalId::new_user_test_id(6360).to_string()],
            should_auto_finalize: Some(true),
            ..Default::default()
        };

        let mut governance = GovernanceCanisterInitPayloadBuilder::new();
        // Existing tests expect this.
        governance.with_mode(ic_sns_governance::pb::v1::governance::Mode::Normal);

        SnsTestsInitPayloadBuilder {
            root: SnsRootCanister::default(),
            governance,
            swap,
            ledger,
        }
    }

    pub fn with_governance_mode(
        &mut self,
        mode: ic_sns_governance::pb::v1::governance::Mode,
    ) -> &mut Self {
        self.governance.with_mode(mode);
        self
    }

    pub fn with_genesis_timestamp_seconds(&mut self, genesis_timestamp_seconds: u64) -> &mut Self {
        self.governance
            .with_genesis_timestamp_seconds(genesis_timestamp_seconds);
        self
    }

    pub fn with_ledger_init_state(&mut self, state: LedgerInitArgs) -> &mut Self {
        self.ledger = state;
        self
    }

    pub fn with_ledger_account(&mut self, account: Account, icpts: Tokens) -> &mut Self {
        self.ledger.initial_balances.push((account, icpts.into()));
        self
    }

    pub fn with_ledger_accounts(&mut self, accounts: Vec<Account>, tokens: Tokens) -> &mut Self {
        for account in accounts {
            self.ledger.initial_balances.push((account, tokens.into()));
        }
        self
    }

    pub fn with_ledger_transfer_fee(&mut self, fee: impl Into<Nat>) -> &mut Self {
        self.ledger.transfer_fee = fee.into();
        self
    }

    pub fn with_governance_init_payload(
        &mut self,
        governance_init_payload_builder: GovernanceCanisterInitPayloadBuilder,
    ) -> &mut Self {
        self.governance = governance_init_payload_builder;
        self
    }

    pub fn with_nervous_system_parameters(
        &mut self,
        params: ic_sns_governance::pb::v1::NervousSystemParameters,
    ) -> &mut Self {
        self.governance.with_parameters(params);
        self
    }

    pub fn with_initial_neurons(&mut self, neurons: Vec<Neuron>) -> &mut Self {
        let mut neuron_map = BTreeMap::new();
        for neuron in neurons {
            neuron_map.insert(neuron.id.as_ref().unwrap().to_string(), neuron);
        }
        self.governance.with_neurons(neuron_map);
        self
    }

    pub fn with_archive_options(&mut self, archive_options: ArchiveOptions) -> &mut Self {
        self.ledger.archive_options = archive_options;
        self
    }

    pub fn build(&mut self) -> SnsCanisterInitPayloads {
        use num_traits::ToPrimitive;

        let governance = self.governance.build();

        let ledger = LedgerArgument::Init(self.ledger.clone());

        let swap = SwapInit {
            fallback_controller_principal_ids: vec![PrincipalId::new_user_test_id(6360).to_string()],
            should_auto_finalize: Some(true),
            transaction_fee_e8s: Some(self.ledger.transfer_fee.0.to_u64().unwrap()),
            neuron_minimum_stake_e8s: Some(
                governance
                    .parameters
                    .as_ref()
                    .unwrap()
                    .neuron_minimum_stake_e8s
                    .unwrap(),
            ),
            min_participants: Some(5),
            min_icp_e8s: None,
            max_icp_e8s: None,
            min_direct_participation_icp_e8s: Some(12_300_000_000),
            max_direct_participation_icp_e8s: Some(65_000_000_000),
            min_participant_icp_e8s: Some(6_500_000_000),
            max_participant_icp_e8s: Some(65_000_000_000),
            swap_start_timestamp_seconds: Some(10_000_000),
            swap_due_timestamp_seconds: Some(10_086_400),
            sns_token_e8s: Some(10_000_000),
            neuron_basket_construction_parameters: Some(NeuronBasketConstructionParameters {
                count: 5,
                dissolve_delay_interval_seconds: 10_001,
            }),
            nns_proposal_id: Some(10),
            neurons_fund_participants: None,
            neurons_fund_participation: Some(false),
            neurons_fund_participation_constraints: None,
            ..Default::default()
        };

        let root = self.root.clone();

        SnsCanisterInitPayloads {
            governance,
            ledger,
            root,
            swap,
            index_ng: None,
        }
    }
}

pub fn populate_canister_ids(
    root_canister_id: PrincipalId,
    governance_canister_id: PrincipalId,
    ledger_canister_id: PrincipalId,
    swap_canister_id: PrincipalId,
    index_canister_id: PrincipalId,
    archive_canister_ids: Vec<PrincipalId>,
    sns_canister_init_payloads: &mut SnsCanisterInitPayloads,
) {
    // Root.
    {
        let root = &mut sns_canister_init_payloads.root;
        if root.governance_canister_id.is_none() {
            root.governance_canister_id = Some(governance_canister_id);
        }
        if root.ledger_canister_id.is_none() {
            root.ledger_canister_id = Some(ledger_canister_id);
        }
        if root.swap_canister_id.is_none() {
            root.swap_canister_id = Some(swap_canister_id);
        }
        if root.index_canister_id.is_none() {
            root.index_canister_id = Some(index_canister_id);
        }
        if root.archive_canister_ids.is_empty() {
            root.archive_canister_ids = archive_canister_ids;
        }
    }
    // Governance canister_init args.
    {
        let governance = &mut sns_canister_init_payloads.governance;
        governance.ledger_canister_id = Some(ledger_canister_id);
        governance.root_canister_id = Some(root_canister_id);
        governance.swap_canister_id = Some(swap_canister_id);
    }
    // Ledger
    {
        if let LedgerArgument::Init(ref mut ledger) = sns_canister_init_payloads.ledger {
            // ledger.minting_account = Account {
            //     owner: governance_canister_id.0,
            //     subaccount: None,
            // };
            ledger.archive_options.controller_id = root_canister_id;
        } else {
            panic!("bug: expected Init got Upgrade");
        }
    }
    // Swap
    {
        let swap = &mut sns_canister_init_payloads.swap;
        swap.sns_root_canister_id = root_canister_id.to_string();
        swap.sns_governance_canister_id = governance_canister_id.to_string();
        swap.sns_ledger_canister_id = ledger_canister_id.to_string();

        swap.nns_governance_canister_id = GOVERNANCE_CANISTER_ID.to_string();
        swap.icp_ledger_canister_id = LEDGER_CANISTER_ID.to_string();
    }
}

fn setup_sns_canisters(env: &StateMachine, neurons: Vec<Neuron>) -> SnsCanisterIds {
    let root_canister_id = env.create_canister(None);
    let governance_canister_id = env.create_canister(None);
    let ledger_canister_id = env.create_canister(None);
    let swap_canister_id = env.create_canister(None);
    let index_canister_id = env.create_canister(None);

    let mut payloads = SnsTestsInitPayloadBuilder::new()
        .with_initial_neurons(neurons)
        .build();

    populate_canister_ids(
        root_canister_id.get(),
        governance_canister_id.get(),
        ledger_canister_id.get(),
        swap_canister_id.get(),
        index_canister_id.get(),
        vec![],
        &mut payloads,
    );

    let deployed_version = Version {
        root_wasm_hash: sha256_hash(sns_root()),
        governance_wasm_hash: sha256_hash(sns_governance()),
        ledger_wasm_hash: sha256_hash(ledger_wasm()),
        swap_wasm_hash: sha256_hash(sns_swap()),
        archive_wasm_hash: vec![], // tests don't need it for now so we don't compile it.
        index_wasm_hash: vec![],
    };

    payloads.governance.deployed_version = Some(deployed_version);

    env.install_existing_canister(
        governance_canister_id,
        sns_governance(),
        Encode!(&payloads.governance).unwrap(),
    )
    .unwrap();
    env.install_existing_canister(
        root_canister_id,
        sns_root(),
        Encode!(&payloads.root).unwrap(),
    )
    .unwrap();
    env.install_existing_canister(
        swap_canister_id,
        sns_swap(),
        Encode!(&payloads.swap).unwrap(),
    )
    .unwrap();
    env.install_existing_canister(
        ledger_canister_id,
        ledger_wasm(),
        Encode!(&payloads.ledger).unwrap(),
    )
    .unwrap();
    SnsCanisterIds {
        governance: governance_canister_id,
        ledger: ledger_canister_id,
    }
}

/// Computes the bytes of the subaccount to which neuron staking transfers are made. This
/// function must be kept in sync with the Nervous System UI equivalent.
/// This code comes from the IC repo:
/// https://github.com/dfinity/ic/blob/master/rs/nervous_system/common/src/ledger.rs#L211
fn compute_neuron_staking_subaccount_bytes(controller: Principal, nonce: u64) -> [u8; 32] {
    const DOMAIN: &[u8] = b"neuron-stake";
    const DOMAIN_LENGTH: [u8; 1] = [0x0c];

    let mut hasher = Sha256::new();
    hasher.update(DOMAIN_LENGTH);
    hasher.update(DOMAIN);
    hasher.update(controller.as_slice());
    hasher.update(nonce.to_be_bytes());
    hasher.finalize().into()
}

fn assert_reply(result: WasmResult) -> Vec<u8> {
    match result {
        WasmResult::Reply(bytes) => bytes,
        WasmResult::Reject(reject) => {
            panic!("Expected a successful reply, got a reject: {}", reject)
        }
    }
}
