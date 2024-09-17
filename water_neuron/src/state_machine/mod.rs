use crate::conversion::{MINIMUM_DEPOSIT_AMOUNT, MINIMUM_WITHDRAWAL_AMOUNT};
use crate::nns_types::{
    manage_neuron, manage_neuron::claim_or_refresh,
    manage_neuron::claim_or_refresh::MemoAndController, proposal::Action, ClaimOrRefreshResponse,
    CommandResponse, GovernanceError, ManageNeuron, ManageNeuronResponse, MergeResponse, Neuron,
    Proposal, ProposalInfo,
};
use crate::sns_distribution::EXPECTED_INITIAL_BALANCE;
use crate::state::event::{GetEventsArg, GetEventsResult};
use crate::state::{
    TransferStatus, WithdrawalDetails, WithdrawalStatus, SNS_GOVERNANCE_SUBACCOUNT,
};
use crate::EventType::{DisbursedMaturityNeuron, DisbursedUserNeuron};
use crate::{
    compute_neuron_staking_subaccount_bytes, nICP, CancelWithdrawalError, CanisterInfo,
    ConversionArg, ConversionError, DepositSuccess, InitArg, LiquidArg, NeuronId, PendingTransfer,
    Unit, UpgradeArg, WithdrawalSuccess, DEFAULT_LEDGER_FEE, E8S, ICP,
    MIN_DISSOLVE_DELAY_FOR_REWARDS, NEURON_LEDGER_FEE,
};
use assert_matches::assert_matches;
use candid::{Decode, Encode, Nat, Principal};
use cycles_minting_canister::{CyclesCanisterInitPayload, CYCLES_LEDGER_CANISTER_ID};
use ic_icrc1_ledger::{
    ArchiveOptions, InitArgs as LedgerInitArgs, InitArgsBuilder as LedgerInitArgsBuilder,
    LedgerArgument,
};
use ic_nns_constants::{GOVERNANCE_CANISTER_ID, LEDGER_CANISTER_ID};
use ic_nns_governance::pb::v1::{Governance, NetworkEconomics};
use ic_sns_governance::init::GovernanceCanisterInitPayloadBuilder;
use ic_sns_governance::pb::v1::{
    governance::Version,
    manage_neuron::Command as SnsCommand,
    nervous_system_function::{FunctionType, GenericNervousSystemFunction},
    neuron::DissolveState,
    proposal::Action as SnsAction,
    ListProposals, ListProposalsResponse, ManageNeuron as SnsManageNeuron,
    ManageNeuronResponse as SnsManageNeuronResponse, NervousSystemFunction, Neuron as SnsNeuron,
    NeuronId as SnsNeuronId, NeuronPermission, NeuronPermissionType, Proposal as SnsProposal,
};
use ic_sns_init::SnsCanisterInitPayloads;
use ic_sns_root::pb::v1::SnsRootCanister;
use ic_sns_swap::pb::v1::{Init as SwapInit, NeuronBasketConstructionParameters};
use ic_state_machine_tests::{
    CanisterId, CanisterInstallMode, ErrorCode::CanisterCalledTrap, PrincipalId, StateMachine,
    UserError, WasmResult,
};
use icp_ledger::{AccountIdentifier, LedgerCanisterInitPayload, Tokens};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::{TransferArg, TransferError};
use icrc_ledger_types::icrc2::approve::{ApproveArgs, ApproveError};
use lazy_static::lazy_static;
use prost::Message;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;

const DEFAULT_PRINCIPAL_ID: u64 = 10352385;

lazy_static! {
    static ref CARGO_BUILD_RESULT: Result<(), std::io::Error> = cargo_build();
}

fn get_wasm(env: &str) -> Vec<u8> {
    std::fs::read(std::env::var(env).unwrap()).unwrap()
}

fn water_neuron_wasm() -> Vec<u8> {
    get_wasm("WATER_NEURON_CANISTER_WASM_PATH")
}

fn ledger_wasm() -> Vec<u8> {
    get_wasm("IC_ICRC1_LEDGER_WASM_PATH")
}

fn icp_ledger_wasm() -> Vec<u8> {
    get_wasm("LEDGER_CANISTER_WASM_PATH")
}

fn cmc_wasm() -> Vec<u8> {
    get_wasm("CYCLES_MINTING_CANISTER_WASM_PATH")
}

fn governance_wasm() -> Vec<u8> {
    get_wasm("GOVERNANCE_CANISTER_WASM_PATH")
}

fn sns_root() -> Vec<u8> {
    get_wasm("SNS_ROOT_CANISTER_WASM_PATH")
}

fn sns_governance() -> Vec<u8> {
    get_wasm("SNS_GOVERNANCE_CANISTER_WASM_PATH")
}

fn sns_swap() -> Vec<u8> {
    get_wasm("SNS_SWAP_CANISTER_WASM_PATH")
}

pub fn sha256_hash(data: Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(&data);
    hasher.finalize().to_vec()
}

/// Builder to help create the initial payloads for the SNS canisters in tests.
pub struct SnsTestsInitPayloadBuilder {
    pub governance: GovernanceCanisterInitPayloadBuilder,
    pub ledger: LedgerInitArgs,
    pub root: SnsRootCanister,
    pub swap: SwapInit,
}

pub fn nns_governance_make_proposal(
    state_machine: &mut StateMachine,
    sender: PrincipalId,
    neuron_id: NeuronId,
    proposal: &Proposal,
) -> ManageNeuronResponse {
    let command = manage_neuron::Command::MakeProposal(*Box::new(proposal.clone()));

    manage_neuron(state_machine, sender, neuron_id, command)
}

#[must_use]
fn manage_neuron(
    state_machine: &mut StateMachine,
    sender: PrincipalId,
    neuron_id: NeuronId,
    command: manage_neuron::Command,
) -> ManageNeuronResponse {
    let result = state_machine
        .execute_ingress_as(
            sender,
            GOVERNANCE_CANISTER_ID,
            "manage_neuron",
            Encode!(&ManageNeuron {
                id: Some(neuron_id),
                command: Some(command),
                neuron_id_or_subaccount: None
            })
            .unwrap(),
        )
        .unwrap();

    let result = match result {
        WasmResult::Reply(result) => result,
        WasmResult::Reject(s) => panic!("Call to manage_neuron failed: {:#?}", s),
    };

    Decode!(&result, ManageNeuronResponse).unwrap()
}

#[must_use]
pub fn nns_claim_or_refresh_neuron(
    state_machine: &mut StateMachine,
    controller: PrincipalId,
    memo: u64,
) -> NeuronId {
    // Construct request.
    let command = Some(manage_neuron::Command::ClaimOrRefresh(
        manage_neuron::ClaimOrRefresh {
            by: Some(claim_or_refresh::By::MemoAndController(MemoAndController {
                memo,
                controller: Some(controller.into()),
            })),
        },
    ));
    let manage_neuron = ManageNeuron {
        id: None,
        command,
        neuron_id_or_subaccount: None,
    };
    let manage_neuron = Encode!(&manage_neuron).unwrap();

    // Call governance.
    let result = state_machine
        .execute_ingress_as(
            controller,
            GOVERNANCE_CANISTER_ID,
            "manage_neuron",
            manage_neuron,
        )
        .unwrap();

    // Unpack and return result.
    let result = match result {
        WasmResult::Reply(reply) => Decode!(&reply, ManageNeuronResponse).unwrap(),
        _ => panic!("{:?}", result),
    };
    let neuron_id = match &result.command {
        Some(CommandResponse::ClaimOrRefresh(ClaimOrRefreshResponse {
            refreshed_neuron_id: Some(neuron_id),
        })) => neuron_id,
        _ => panic!("{:?}", result),
    };
    *neuron_id
}

pub fn nns_increase_dissolve_delay(
    state_machine: &mut StateMachine,
    sender: PrincipalId,
    neuron_id: NeuronId,
    additional_dissolve_delay_seconds: u64,
) -> ManageNeuronResponse {
    let additional_dissolve_delay_seconds =
        u32::try_from(additional_dissolve_delay_seconds).unwrap();

    nns_configure_neuron(
        state_machine,
        sender,
        neuron_id,
        manage_neuron::configure::Operation::IncreaseDissolveDelay(
            manage_neuron::IncreaseDissolveDelay {
                additional_dissolve_delay_seconds,
            },
        ),
    )
}

fn nns_configure_neuron(
    state_machine: &mut StateMachine,
    sender: PrincipalId,
    neuron_id: NeuronId,
    operation: manage_neuron::configure::Operation,
) -> ManageNeuronResponse {
    manage_neuron(
        state_machine,
        sender,
        neuron_id,
        manage_neuron::Command::Configure(manage_neuron::Configure {
            operation: Some(operation),
        }),
    )
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

    pub fn with_initial_neurons(&mut self, neurons: Vec<SnsNeuron>) -> &mut Self {
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

struct SNSCanisterIds {
    pub governance: CanisterId,
    pub ledger: CanisterId,
}

fn setup_sns_canisters(env: &StateMachine, neurons: Vec<SnsNeuron>) -> SNSCanisterIds {
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
    SNSCanisterIds {
        governance: governance_canister_id,
        ledger: ledger_canister_id,
    }
}

fn cargo_build() -> Result<(), std::io::Error> {
    std::process::Command::new("cargo")
        .current_dir("../")
        .args(&[
            "build",
            "--target=wasm32-unknown-unknown",
            "--release",
            "--locked",
            "--features=self_check",
        ])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    std::process::Command::new("gzip")
        .args(&["-nf9v", "water-neuron.wasm"])
        .current_dir("../target/wasm32-unknown-unknown/release")
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    Ok(())
}

#[derive(Debug)]
struct WaterNeuron {
    pub env: StateMachine,
    pub minter: PrincipalId,
    pub water_neuron_id: CanisterId,
    pub wtn_ledger_id: CanisterId,
    pub wtn_governance_id: CanisterId,
    pub icp_ledger_id: CanisterId,
    pub nicp_ledger_id: CanisterId,
    pub governance_id: CanisterId,
}

impl WaterNeuron {
    fn new() -> Self {
        let minter = PrincipalId::new_user_test_id(DEFAULT_PRINCIPAL_ID);

        let env = StateMachine::new();
        let nicp_ledger_id = env.create_canister(None);

        let arg = Governance {
            economics: Some(NetworkEconomics::with_default_values()),
            wait_for_quiet_threshold_seconds: 60 * 60 * 24 * 4, // 4 days
            short_voting_period_seconds: 60 * 60 * 12,          // 12 hours
            neuron_management_voting_period_seconds: Some(60 * 60 * 48), // 48 hours
            ..Default::default()
        };
        let governance_id = env
            .install_canister(governance_wasm(), arg.encode_to_vec(), None)
            .unwrap();

        let mut initial_balances = HashMap::new();
        initial_balances.insert(
            AccountIdentifier::new(minter.into(), None),
            Tokens::from_e8s(22_000_000 * E8S),
        );

        let icp_ledger_id = env
            .install_canister(
                icp_ledger_wasm(),
                Encode!(&LedgerCanisterInitPayload::builder()
                    .initial_values(initial_balances)
                    .transfer_fee(Tokens::from_e8s(10_000))
                    .minting_account(GOVERNANCE_CANISTER_ID.get().into())
                    .token_symbol_and_name("ICP", "Internet Computer")
                    .feature_flags(icp_ledger::FeatureFlags { icrc2: true })
                    .build()
                    .unwrap())
                .unwrap(),
                None,
            )
            .unwrap();

        let water_neuron_id = env.create_canister(None);
        let water_neuron_principal = water_neuron_id.get().0;

        let mut neurons = vec![];
        neurons.push(SnsNeuron {
            id: Some(SnsNeuronId {
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
            neurons.push(SnsNeuron {
                id: Some(SnsNeuronId {
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
                    12 * crate::ONE_MONTH_SECONDS as u64,
                )),
                voting_power_percentage_multiplier: 100,
                ..Default::default()
            });
        }

        let cmc_id = env
            .install_canister(
                cmc_wasm(),
                Encode!(&Some(CyclesCanisterInitPayload {
                    ledger_canister_id: Some(LEDGER_CANISTER_ID),
                    governance_canister_id: Some(GOVERNANCE_CANISTER_ID),
                    exchange_rate_canister: None,
                    minting_account_id: Some(GOVERNANCE_CANISTER_ID.get().into()),
                    last_purged_notification: Some(1),
                    cycles_ledger_canister_id: Some(CYCLES_LEDGER_CANISTER_ID.try_into().unwrap()),
                }))
                .unwrap(),
                None,
            )
            .unwrap();
        assert_eq!(
            Principal::from_str("rkp4c-7iaaa-aaaaa-aaaca-cai").unwrap(),
            cmc_id.into()
        );

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
                    .with_transfer_fee(DEFAULT_LEDGER_FEE)
                    .with_decimals(8)
                    .with_feature_flags(ic_icrc1_ledger::FeatureFlags { icrc2: true })
                    .build(),
            ))
            .unwrap(),
        )
        .unwrap();

        WaterNeuron {
            env,
            minter,
            water_neuron_id,
            wtn_ledger_id: sns.ledger,
            wtn_governance_id: sns.governance,
            icp_ledger_id,
            nicp_ledger_id,
            governance_id,
        }
    }

    fn with_voting_topic(&self) -> &WaterNeuron {
        let nervous_system_function = NervousSystemFunction {
            id: 1000,
            name: "a".to_string(),
            description: None,
            function_type: Some(FunctionType::GenericNervousSystemFunction(
                GenericNervousSystemFunction {
                    target_canister_id: Some(self.water_neuron_id.get()),
                    target_method_name: Some("approve_proposal".to_string()),
                    validator_canister_id: Some(self.water_neuron_id.get()),
                    validator_method_name: Some("approve_proposal_validate".to_string()),
                },
            )),
        };

        let proposal_payload = SnsProposal {
            title: "Add new GenericNervousSystemFunction".into(),
            action: Some(SnsAction::AddGenericNervousSystemFunction(
                nervous_system_function.clone(),
            )),
            ..Default::default()
        };

        let res = self.wtn_make_proposal(PrincipalId::new_user_test_id(1234).0, proposal_payload);

        println!("[with_voting_topic] {res:?}");

        self.advance_time_and_tick(60);

        self
    }

    fn wtn_make_proposal(
        &self,
        caller: Principal,
        proposal: SnsProposal,
    ) -> SnsManageNeuronResponse {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        PrincipalId::from(caller),
                        self.wtn_governance_id,
                        "manage_neuron",
                        Encode!(&SnsManageNeuron {
                            subaccount: compute_neuron_staking_subaccount_bytes(
                                PrincipalId::new_user_test_id(1234).0,
                                0
                            )
                            .to_vec(),
                            command: Some(SnsCommand::MakeProposal(proposal)),
                        })
                        .unwrap()
                    )
                    .expect("failed to wtn_make_proposal")
            ),
            SnsManageNeuronResponse
        )
        .expect("wtn_make_proposal")
    }

    fn advance_time_and_tick(&self, seconds: u64) {
        self.env
            .advance_time(std::time::Duration::from_secs(seconds));
        const MAX_TICKS: u8 = 10;
        for _ in 0..MAX_TICKS {
            self.env.tick();
        }
    }

    fn transfer(
        &self,
        caller: PrincipalId,
        to: impl Into<Account>,
        amount: u64,
        ledger_id: CanisterId,
    ) -> Nat {
        Decode!(&assert_reply(self.env.execute_ingress_as(
            caller,
            ledger_id,
            "icrc1_transfer",
            Encode!(&TransferArg {
                from_subaccount: None,
                to: to.into(),
                fee: None,
                created_at_time: None,
                memo: None,
                amount: Nat::from(amount),
            }).unwrap()
            ).expect("failed to execute token transfer")),
            Result<Nat, TransferError>
        )
        .unwrap()
        .expect("token transfer failed")
    }

    fn approve(&self, caller: PrincipalId, ledger: CanisterId, spender: Account) {
        assert_matches!(
            Decode!(
                &assert_reply(self.env.execute_ingress_as(
                    caller,
                    ledger,
                    "icrc2_approve",
                    Encode!(&ApproveArgs {
                        from_subaccount: None,
                        spender,
                        amount: u64::MAX.into(),
                        expected_allowance: None,
                        expires_at: None,
                        fee: None,
                        memo: None,
                        created_at_time: None,
                    }).unwrap()
                )
                .expect("failed to approve protocol canister")),
                Result<Nat, ApproveError>
            )
            .expect("failed to decode open_vault response"),
            Ok(_)
        );
    }

    pub fn balance_of(&self, canister_id: CanisterId, from: impl Into<Account>) -> Nat {
        let from = from.into();
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress(canister_id, "icrc1_balance_of", Encode!(&from).unwrap())
                    .expect("failed to execute token transfer")
            ),
            Nat
        )
        .unwrap()
    }

    fn icp_to_nicp(
        &self,
        caller: PrincipalId,
        amount_e8s: u64,
    ) -> Result<DepositSuccess, ConversionError> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        caller,
                        self.water_neuron_id,
                        "icp_to_nicp",
                        Encode!(&ConversionArg {
                            amount_e8s,
                            maybe_subaccount: None
                        })
                        .unwrap()
                    )
                    .expect("failed to deposit")
            ),
            Result<DepositSuccess, ConversionError>
        )
        .unwrap()
    }

    fn nicp_to_icp(
        &self,
        caller: PrincipalId,
        amount_e8s: u64,
    ) -> Result<WithdrawalSuccess, ConversionError> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        caller,
                        self.water_neuron_id,
                        "nicp_to_icp",
                        Encode!(&ConversionArg {
                            amount_e8s,
                            maybe_subaccount: None
                        })
                        .unwrap()
                    )
                    .expect("failed to withdraw")
            ),
            Result<WithdrawalSuccess, ConversionError>
        )
        .unwrap()
    }

    fn cancel_withdrawal(
        &self,
        caller: PrincipalId,
        neuron_id: NeuronId,
    ) -> Result<MergeResponse, CancelWithdrawalError> {
        Decode!(
            &assert_reply(
                self.env.execute_ingress_as(
                    caller,
                    self.water_neuron_id,
                    "cancel_withdrawal",
                    Encode!(&neuron_id).unwrap()
                ).expect("failed to cancel_withdrawal")
            ),
            Result<MergeResponse, CancelWithdrawalError>
        )
        .unwrap()
    }

    fn get_airdrop_allocation(&self, caller: Principal) -> u64 {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        PrincipalId::from(caller),
                        self.water_neuron_id,
                        "get_airdrop_allocation",
                        Encode!(&caller).unwrap()
                    )
                    .expect("failed to get get_airdrop_allocation")
            ),
            u64
        )
        .unwrap()
    }

    #[allow(dead_code)]
    fn get_full_neuron(&self, neuron_id: u64) -> Result<Result<Neuron, GovernanceError>, String> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        Principal::from_text("bo5bf-eaaaa-aaaam-abtza-cai")
                            .unwrap()
                            .into(),
                        self.water_neuron_id,
                        "get_full_neuron",
                        Encode!(&neuron_id).unwrap()
                    )
                    .expect("failed to get get_airdrop_allocation")
            ),
            Result<Result<Neuron, GovernanceError>, String>
        )
        .unwrap()
    }

    fn claim_airdrop(&self, caller: Principal) -> Result<u64, ConversionError> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        PrincipalId::from(caller),
                        self.water_neuron_id,
                        "claim_airdrop",
                        Encode!(&caller).unwrap()
                    )
                    .expect("failed to get claim_airdrop")
            ),
            Result<u64, ConversionError>
        )
        .unwrap()
    }

    fn get_transfer_statuses(&self, ids: Vec<u64>) -> Vec<TransferStatus> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress(
                        self.water_neuron_id,
                        "get_transfer_statuses",
                        Encode!(&ids).unwrap()
                    )
                    .expect("failed to get get_transfer_statuses")
            ),
            Vec<TransferStatus>
        )
        .unwrap()
    }

    fn approve_proposal(&self, id: u64, caller: Principal) -> Result<ManageNeuronResponse, String> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        PrincipalId::from(caller),
                        self.water_neuron_id,
                        "approve_proposal",
                        Encode!(&id).unwrap()
                    )
                    .expect("failed to get approve_proposal")
            ),
            Result<ManageNeuronResponse, String>
        )
        .unwrap()
    }

    fn get_info(&self) -> CanisterInfo {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress(self.water_neuron_id, "get_info", Encode!().unwrap())
                    .expect("failed to get info")
            ),
            CanisterInfo
        )
        .unwrap()
    }

    fn get_events(&self) -> GetEventsResult {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress(
                        self.water_neuron_id,
                        "get_events",
                        Encode!(&GetEventsArg {
                            start: 0,
                            length: 2000,
                        })
                        .unwrap()
                    )
                    .expect("failed to call")
            ),
            GetEventsResult
        )
        .unwrap()
    }

    fn get_withdrawal_requests(&self, target: impl Into<Account>) -> Vec<WithdrawalDetails> {
        let target_account: Account = target.into();
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress(
                        self.water_neuron_id,
                        "get_withdrawal_requests",
                        Encode!(&Some(target_account)).unwrap()
                    )
                    .expect("failed to execute get_withdrawal_requests")
            ),
            Vec<WithdrawalDetails>
        )
        .unwrap()
    }

    #[allow(dead_code)]
    fn update_neuron(&self, neuron: Neuron) -> Option<GovernanceError> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        self.water_neuron_id.into(),
                        self.governance_id,
                        "update_neuron",
                        Encode!(&neuron).unwrap()
                    )
                    .expect("failed to update_neuron")
            ),
            Option<GovernanceError>
        )
        .unwrap()
    }

    fn get_pending_proposals(&self) -> Vec<ProposalInfo> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress(
                        self.governance_id,
                        "get_pending_proposals",
                        Encode!().unwrap()
                    )
                    .expect("failed to get_pending_proposals")
            ),
            Vec<ProposalInfo>
        )
        .unwrap()
    }

    fn get_proposal_info(&self, id: u64) -> Option<ProposalInfo> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress(
                        self.governance_id,
                        "get_proposal_info",
                        Encode!(&id).unwrap()
                    )
                    .expect("failed to get_proposal_info")
            ),
            Option<ProposalInfo>
        )
        .unwrap()
    }

    fn list_proposals(&self, canister: CanisterId, arg: ListProposals) -> ListProposalsResponse {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress(canister, "list_proposals", Encode!(&arg).unwrap())
                    .expect("failed to list_proposals")
            ),
            ListProposalsResponse
        )
        .unwrap()
    }

    #[allow(dead_code)]
    fn manage_neuron(
        &self,
        caller: PrincipalId,
        manage_neuron: ManageNeuron,
    ) -> ManageNeuronResponse {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        caller,
                        self.governance_id,
                        "manage_neuron",
                        Encode!(&manage_neuron).unwrap()
                    )
                    .expect("failed to manage_neuron")
            ),
            ManageNeuronResponse
        )
        .unwrap()
    }
}

fn assert_reply(result: WasmResult) -> Vec<u8> {
    match result {
        WasmResult::Reply(bytes) => bytes,
        WasmResult::Reject(reject) => {
            panic!("Expected a successful reply, got a reject: {}", reject)
        }
    }
}

#[test]
fn e2e_basic() {
    let mut water_neuron = WaterNeuron::new();
    water_neuron.with_voting_topic();

    let caller = PrincipalId::new_user_test_id(212);

    let water_neuron_principal: Principal = water_neuron.water_neuron_id.get().into();

    assert_eq!(
        water_neuron.transfer(
            water_neuron.minter,
            water_neuron_principal,
            10 * E8S,
            water_neuron.icp_ledger_id
        ),
        Nat::from(1_u8)
    );
    assert_eq!(
        water_neuron.transfer(
            water_neuron.minter,
            caller.0,
            110 * E8S,
            water_neuron.icp_ledger_id
        ),
        Nat::from(2_u8)
    );

    assert_eq!(
        water_neuron.transfer(
            water_neuron.minter,
            Account {
                owner: GOVERNANCE_CANISTER_ID.into(),
                subaccount: Some(compute_neuron_staking_subaccount_bytes(caller.into(), 0))
            },
            11 * E8S,
            water_neuron.icp_ledger_id
        ),
        Nat::from(3_u8)
    );

    let neuron_id = nns_claim_or_refresh_neuron(&mut water_neuron.env, caller, 0);

    let _increase_dissolve_delay_result =
        nns_increase_dissolve_delay(&mut water_neuron.env, caller, neuron_id, 200 * 24 * 60 * 60);

    water_neuron.advance_time_and_tick(70);

    water_neuron.approve(
        caller,
        water_neuron.icp_ledger_id,
        water_neuron.water_neuron_id.get().0.into(),
    );

    assert_eq!(
        water_neuron.balance_of(water_neuron.icp_ledger_id, caller.0),
        Nat::from(10_999_990_000_u64)
    );

    let icp_to_wrap = 100 * E8S;

    water_neuron.advance_time_and_tick(60);

    let mut info = water_neuron.get_info();
    assert_eq!(
        water_neuron.balance_of(water_neuron.icp_ledger_id, info.neuron_6m_account),
        Nat::from(E8S + 42)
    );

    assert_eq!(
        water_neuron.icp_to_nicp(caller.0.into(), icp_to_wrap),
        Ok(DepositSuccess {
            block_index: Nat::from(7_u8),
            transfer_id: 0,
            nicp_amount: Some(nICP::from_e8s(icp_to_wrap)),
        })
    );

    assert_eq!(
        water_neuron.balance_of(water_neuron.icp_ledger_id, info.neuron_6m_account),
        Nat::from(E8S + 42 + icp_to_wrap)
    );
    assert_eq!(
        water_neuron.balance_of(water_neuron.nicp_ledger_id, caller.0),
        Nat::from(icp_to_wrap)
    );
    assert_eq!(
        water_neuron.balance_of(water_neuron.icp_ledger_id, caller.0),
        Nat::from(999_980_000_u64)
    );

    water_neuron.advance_time_and_tick(MIN_DISSOLVE_DELAY_FOR_REWARDS.into());

    water_neuron.approve(
        caller,
        water_neuron.nicp_ledger_id,
        water_neuron.water_neuron_id.get().0.into(),
    );
    assert_eq!(
        water_neuron.balance_of(water_neuron.nicp_ledger_id, caller.0),
        Nat::from(9_999_990_000_u64)
    );

    water_neuron.advance_time_and_tick(24 * 60 * 60 + 10);

    let nicp_to_unwrap = 10 * E8S;
    match water_neuron.nicp_to_icp(caller.0.into(), nicp_to_unwrap) {
        Ok(WithdrawalSuccess { withdrawal_id, .. }) => {
            assert_eq!(withdrawal_id, 0);
        }
        Err(e) => panic!("Expected WithdrawalSuccess, got {e:?}"),
    }

    assert_eq!(water_neuron.get_withdrawal_requests(caller.0).len(), 1);
    assert_eq!(
        water_neuron.get_withdrawal_requests(caller.0)[0].status,
        WithdrawalStatus::WaitingToSplitNeuron
    );

    assert_eq!(
        water_neuron.balance_of(water_neuron.nicp_ledger_id, caller.0),
        Nat::from(8_999_990_000_u64)
    );
    assert_eq!(
        water_neuron.balance_of(water_neuron.icp_ledger_id, info.neuron_6m_account),
        Nat::from(E8S + 42 + icp_to_wrap - nicp_to_unwrap)
    );

    assert_matches!(
        water_neuron.get_withdrawal_requests(caller.0)[0].status,
        WithdrawalStatus::WaitingDissolvement { .. }
    );

    water_neuron.advance_time_and_tick(MIN_DISSOLVE_DELAY_FOR_REWARDS.into());

    assert_eq!(water_neuron.get_withdrawal_requests(caller.0).len(), 1);
    assert_eq!(
        water_neuron.get_withdrawal_requests(caller.0)[0].status,
        WithdrawalStatus::ConversionDone {
            transfer_block_height: 9
        }
    );

    assert_eq!(
        water_neuron.balance_of(water_neuron.icp_ledger_id, caller.0),
        Nat::from(1_999_960_000_u64)
    );

    assert_eq!(
        water_neuron.balance_of(water_neuron.icp_ledger_id, info.neuron_6m_account),
        Nat::from(9_100_000_042_u64)
    );

    water_neuron.advance_time_and_tick(60 * 60 * 24 + 1);

    match water_neuron.cancel_withdrawal(
        caller.0.into(),
        water_neuron.get_withdrawal_requests(caller.0)[0]
            .request
            .neuron_id
            .unwrap(),
    ) {
        Ok(response) => {
            panic!("Expected CancelWithdrawalError, got response: {response:?}");
        }
        Err(e) => match e {
            CancelWithdrawalError::RequestNotFound => {}
            _ => {
                panic!("Expected RequestNotFound, got {e:?}")
            }
        },
    }

    assert_eq!(
        water_neuron.balance_of(
            water_neuron.icp_ledger_id,
            Account {
                owner: water_neuron.water_neuron_id.get().0,
                subaccount: Some([1; 32])
            }
        ),
        Nat::from(0_u8)
    );

    water_neuron.advance_time_and_tick(60 * 60);

    info = water_neuron.get_info();
    assert_eq!(info.exchange_rate, E8S);
    assert_eq!(info.stakers_count, 1);
    assert_eq!(info.total_icp_deposited, ICP::from_e8s(icp_to_wrap));
    assert_eq!(info.minimum_deposit_amount, MINIMUM_DEPOSIT_AMOUNT);
    assert_eq!(info.minimum_withdraw_amount, MINIMUM_WITHDRAWAL_AMOUNT);
    assert!(info.neuron_id_6m.is_some());
    assert!(info.neuron_id_8y.is_some());
    assert_eq!(info.neuron_8y_stake_e8s, ICP::from_e8s(100_000_042));
    assert_eq!(info.neuron_6m_stake_e8s, info.tracked_6m_stake);

    match water_neuron.nicp_to_icp(caller.0.into(), nicp_to_unwrap) {
        Ok(WithdrawalSuccess { withdrawal_id, .. }) => {
            assert_eq!(withdrawal_id, 1);
        }
        Err(e) => panic!("Expected WithdrawalSuccess, got {e:?}"),
    }

    assert_eq!(
        water_neuron.balance_of(water_neuron.nicp_ledger_id, caller.0),
        Nat::from(7_999_990_000_u64)
    );

    water_neuron.advance_time_and_tick(60);

    let neuron_ids: Vec<Option<NeuronId>> = water_neuron
        .get_withdrawal_requests(caller.0)
        .iter()
        .map(|detail| detail.request.neuron_id)
        .collect();
    assert_eq!(neuron_ids.len(), 2);

    water_neuron.advance_time_and_tick(60 * 60);

    info = water_neuron.get_info();

    match water_neuron.cancel_withdrawal(caller.0.into(), neuron_ids[1].unwrap()) {
        Ok(response) => {
            let target_neuron_info = response.target_neuron_info.unwrap().clone();
            let source_neuron_info = response.source_neuron_info.unwrap().clone();
            let source_neuron = response.source_neuron.unwrap().clone();
            let target_neuron = response.target_neuron.unwrap().clone();
            assert_eq!(source_neuron.id.unwrap().id, 12440400712491049369);
            assert_eq!(source_neuron.neuron_fees_e8s, 0);
            assert_eq!(target_neuron.id.unwrap().id, 12420353447771927594);
            assert_eq!(target_neuron.neuron_fees_e8s, 0);
            assert_eq!(
                target_neuron_info.dissolve_delay_seconds,
                15_865_200 // 6 months
            );
            assert_eq!(target_neuron_info.stake_e8s, 9_099_980_042);
            assert_eq!(source_neuron_info.age_seconds, 0);
            assert_eq!(source_neuron_info.stake_e8s, 0);
        }
        Err(e) => {
            panic!("Expected MergeResponse, got error: {e:?}");
        }
    }

    water_neuron.advance_time_and_tick(60 * 60);

    assert_eq!(
        water_neuron
            .get_withdrawal_requests(caller.0)
            .last()
            .unwrap()
            .status,
        WithdrawalStatus::Cancelled {
            transfer_block_height: 1
        },
    );

    match water_neuron.cancel_withdrawal(caller.0.into(), neuron_ids[1].unwrap()) {
        Ok(response) => {
            panic!("Expected CancelWithdrawalError, got response: {response:?}");
        }
        Err(e) => match e {
            CancelWithdrawalError::RequestNotFound => {}
            _ => {
                panic!("Expected RequestNotFound, got {e:?}")
            }
        },
    }

    assert_eq!(
        water_neuron.balance_of(water_neuron.icp_ledger_id, info.neuron_6m_account),
        Nat::from(9_099_980_042_u64)
    );

    info = water_neuron.get_info();
    assert_eq!(info.neuron_6m_stake_e8s, info.tracked_6m_stake);
    assert_eq!(info.total_icp_deposited, ICP::from_e8s(icp_to_wrap));

    // Make a proposal to generate some rewards.

    assert_eq!(
        water_neuron.balance_of(
            water_neuron.icp_ledger_id,
            Account {
                owner: water_neuron.water_neuron_id.into(),
                subaccount: Some(SNS_GOVERNANCE_SUBACCOUNT)
            }
        ),
        Nat::from(0_u8)
    );
    let neuron_6m_stake_e8s_before_proposal = water_neuron.get_info().neuron_6m_stake_e8s;

    let proposal = Proposal {
        title: Some("Yellah".to_string()),
        summary: "Dummy Proposal".to_string(),
        url: "https://forum.dfinity.org/t/reevaluating-neuron-control-restrictions/28597/215"
            .to_string(),
        action: Some(Action::Motion(crate::nns_types::Motion {
            motion_text: "".to_string(),
        })),
    };

    let _proposal_id =
        match nns_governance_make_proposal(&mut water_neuron.env, caller, neuron_id, &proposal)
            .command
            .unwrap()
        {
            CommandResponse::MakeProposal(response) => response.proposal_id.unwrap(),
            _ => panic!("unexpected response"),
        };

    water_neuron.advance_time_and_tick(15 * 60);
    water_neuron.advance_time_and_tick(15 * 60);
    water_neuron.advance_time_and_tick(4 * 60 * 60 * 24 - 60 * 60);
    water_neuron.advance_time_and_tick(7 * 24 * 60 * 60 + 10);
    water_neuron.advance_time_and_tick(7 * 24 * 60 * 60 + 10);
    water_neuron.advance_time_and_tick(24 * 60 * 60 + 10);
    water_neuron.advance_time_and_tick(24 * 60 * 60 + 10);

    assert!(neuron_6m_stake_e8s_before_proposal < water_neuron.get_info().neuron_6m_stake_e8s);

    dbg!(water_neuron.get_events());
    assert_eq!(water_neuron.get_events().total_event_count, 27);

    assert!(water_neuron
        .get_events()
        .events
        .iter()
        .map(|e| &e.payload)
        .any(|payload| payload
            == &DisbursedUserNeuron {
                withdrawal_id: 0,
                transfer_block_height: 9,
            }),);

    let count = water_neuron
        .get_events()
        .events
        .iter()
        .map(|e| &e.payload)
        .filter(|payload| matches!(payload, DisbursedMaturityNeuron { .. }))
        .count();

    assert_eq!(count, 2);
    info = water_neuron.get_info();

    assert_eq!(
        water_neuron.balance_of(water_neuron.icp_ledger_id, info.neuron_6m_account),
        Nat::from(info.tracked_6m_stake.0)
    );
    assert_eq!(info.exchange_rate, 5274);

    assert_eq!(info.governance_fee_share_percent, 10);

    assert_matches!(
        water_neuron.env.upgrade_canister(
            water_neuron.water_neuron_id,
            water_neuron_wasm(),
            Encode!(&LiquidArg::Upgrade(Some(UpgradeArg {
                governance_fee_share_percent: Some(20),
            })))
            .unwrap(),
        ),
        Ok(_)
    );
    water_neuron.advance_time_and_tick(60);

    info = water_neuron.get_info();
    assert_eq!(info.neuron_6m_stake_e8s, info.tracked_6m_stake);
    assert_eq!(info.exchange_rate, 5274);
    assert_eq!(info.governance_fee_share_percent, 20);
    assert_eq!(
        water_neuron
            .icp_to_nicp(caller.0.into(), E8S)
            .unwrap()
            .nicp_amount,
        Some(nICP::from_e8s(5274))
    );
    assert_eq!(
        water_neuron
            .nicp_to_icp(caller.0.into(), nicp_to_unwrap)
            .unwrap()
            .icp_amount,
        Some(ICP::from_e8s(18961559429865))
    );

    assert_eq!(
        water_neuron
            .get_withdrawal_requests(caller.0)
            .last()
            .unwrap()
            .status,
        WithdrawalStatus::WaitingToSplitNeuron
    );

    water_neuron.advance_time_and_tick(60 * 60);
    let neuron_id = match water_neuron
        .get_withdrawal_requests(caller.0)
        .last()
        .unwrap()
        .status
    {
        WithdrawalStatus::WaitingDissolvement { neuron_id } => neuron_id,
        _ => panic!(""),
    };

    let full_neuron = water_neuron.get_full_neuron(neuron_id.id).unwrap().unwrap();
    assert_eq!(full_neuron.cached_neuron_stake_e8s, 18961559419865);
}

#[test]
fn should_mirror_proposal() {
    let mut water_neuron = WaterNeuron::new();
    water_neuron.with_voting_topic();

    let water_neuron_principal: Principal = water_neuron.water_neuron_id.get().into();
    let caller = PrincipalId::new_user_test_id(212);

    assert_eq!(
        water_neuron.transfer(
            water_neuron.minter,
            water_neuron_principal,
            10 * E8S,
            water_neuron.icp_ledger_id
        ),
        Nat::from(1_u8)
    );
    assert_eq!(
        water_neuron.transfer(
            water_neuron.minter,
            caller.0,
            100 * E8S,
            water_neuron.icp_ledger_id
        ),
        Nat::from(2_u8)
    );

    water_neuron.advance_time_and_tick(60);

    assert_eq!(
        water_neuron.transfer(
            water_neuron.minter,
            Account {
                owner: GOVERNANCE_CANISTER_ID.into(),
                subaccount: Some(compute_neuron_staking_subaccount_bytes(caller.into(), 0))
            },
            11 * E8S,
            water_neuron.icp_ledger_id
        ),
        Nat::from(5_u8)
    );

    water_neuron.approve(
        water_neuron.minter,
        water_neuron.icp_ledger_id,
        water_neuron.water_neuron_id.get().0.into(),
    );

    assert_eq!(
        water_neuron.icp_to_nicp(water_neuron.minter, 1_000 * E8S),
        Ok(DepositSuccess {
            block_index: Nat::from(7_u8),
            transfer_id: 0,
            nicp_amount: Some(nICP::from_unscaled(1_000)),
        })
    );

    water_neuron.advance_time_and_tick(70);

    let neuron_id = nns_claim_or_refresh_neuron(&mut water_neuron.env, caller, 0);

    let _increase_dissolve_delay_result =
        nns_increase_dissolve_delay(&mut water_neuron.env, caller, neuron_id, 200 * 24 * 60 * 60);

    water_neuron.advance_time_and_tick(70);

    let proposal = Proposal {
        title: Some("Yellah".to_string()),
        summary: "Dummy Proposal".to_string(),
        url: "https://forum.dfinity.org/t/reevaluating-neuron-control-restrictions/28597/215"
            .to_string(),
        action: Some(Action::Motion(crate::nns_types::Motion {
            motion_text: "".to_string(),
        })),
    };

    let proposal_id =
        match nns_governance_make_proposal(&mut water_neuron.env, caller, neuron_id, &proposal)
            .command
            .unwrap()
        {
            CommandResponse::MakeProposal(response) => response.proposal_id.unwrap(),
            _ => panic!("unexpected response"),
        };

    let yes_before_water_neuron = water_neuron.get_pending_proposals()[0]
        .latest_tally
        .clone()
        .unwrap()
        .yes;

    water_neuron.advance_time_and_tick(30 * 60);

    let proposals = water_neuron.list_proposals(
        water_neuron.wtn_governance_id,
        ListProposals {
            include_reward_status: vec![],
            before_proposal: None,
            limit: 10,
            exclude_type: vec![],
            include_status: vec![],
        },
    );
    assert_eq!(proposals.proposals.len(), 2);

    assert!(water_neuron
        .env
        .execute_ingress_as(
            PrincipalId::from(Principal::anonymous()),
            water_neuron.water_neuron_id,
            "approve_proposal",
            Encode!(&proposal_id.id).unwrap()
        )
        .is_err());

    use crate::nns_types::Empty;
    use crate::CommandResponse::RegisterVote;

    assert_eq!(
        water_neuron.approve_proposal(proposal_id.id, water_neuron.wtn_governance_id.get().0),
        Ok(ManageNeuronResponse {
            command: Some(RegisterVote(Empty {}))
        })
    );

    water_neuron.advance_time_and_tick(4 * 60 * 60 * 24 - 60 * 60);

    let yes_after_water_neuron = water_neuron
        .get_proposal_info(proposal_id.id)
        .unwrap()
        .latest_tally
        .clone()
        .unwrap()
        .yes;

    assert!(yes_after_water_neuron > yes_before_water_neuron, "Yes after proposal {yes_after_water_neuron} not greater than before {yes_before_water_neuron}");

    assert_matches!(
        water_neuron.env.upgrade_canister(
            water_neuron.water_neuron_id,
            water_neuron_wasm(),
            Encode!(&LiquidArg::Upgrade(Some(UpgradeArg {
                governance_fee_share_percent: None,
            })))
            .unwrap(),
        ),
        Ok(_)
    );

    water_neuron.advance_time_and_tick(60);
    let info = water_neuron.get_info();
    assert_eq!(info.neuron_6m_stake_e8s, info.tracked_6m_stake);
}

#[test]
fn should_distribute_icp_to_sns_neurons() {
    let water_neuron = WaterNeuron::new();

    let caller = PrincipalId::new_user_test_id(212);

    let water_neuron_principal: Principal = water_neuron.water_neuron_id.get().into();

    assert_eq!(
        water_neuron.transfer(
            water_neuron.minter,
            water_neuron_principal,
            10 * E8S,
            water_neuron.icp_ledger_id
        ),
        Nat::from(1_u8)
    );
    assert_eq!(
        water_neuron.transfer(
            water_neuron.minter,
            caller.0,
            100 * E8S,
            water_neuron.icp_ledger_id
        ),
        Nat::from(2_u8)
    );

    water_neuron.advance_time_and_tick(60);

    water_neuron.approve(
        caller,
        water_neuron.icp_ledger_id,
        water_neuron.water_neuron_id.get().0.into(),
    );

    let icp_to_wrap = 10 * E8S;

    assert_eq!(
        water_neuron.icp_to_nicp(caller.0.into(), icp_to_wrap),
        Ok(DepositSuccess {
            block_index: Nat::from(6_u8),
            transfer_id: 0,
            nicp_amount: Some(nICP::from_e8s(icp_to_wrap)),
        })
    );

    water_neuron.advance_time_and_tick(70);

    assert_eq!(
        water_neuron.transfer(
            Principal::anonymous().into(),
            water_neuron.water_neuron_id.get().0,
            EXPECTED_INITIAL_BALANCE,
            water_neuron.wtn_ledger_id
        ),
        Nat::from(0_u8)
    );

    water_neuron.advance_time_and_tick(0);

    assert_eq!(
        water_neuron.balance_of(water_neuron.wtn_ledger_id, caller.0),
        Nat::from(0_u8)
    );

    assert_eq!(water_neuron.get_airdrop_allocation(caller.0), 8_000_000_000);

    assert_eq!(
        water_neuron.icp_to_nicp(caller.0.into(), icp_to_wrap),
        Ok(DepositSuccess {
            block_index: Nat::from(7_u8),
            transfer_id: 1,
            nicp_amount: Some(nICP::from_e8s(icp_to_wrap)),
        })
    );

    water_neuron.advance_time_and_tick(0);

    assert_eq!(
        water_neuron.get_airdrop_allocation(caller.0),
        16_000_000_000
    );

    assert_eq!(
        water_neuron.transfer(
            water_neuron.minter,
            Account {
                owner: water_neuron.water_neuron_id.get().0,
                subaccount: Some(SNS_GOVERNANCE_SUBACCOUNT)
            },
            100 * E8S,
            water_neuron.icp_ledger_id
        ),
        Nat::from(8_u8)
    );

    assert_eq!(
        water_neuron.balance_of(
            water_neuron.icp_ledger_id,
            Account {
                owner: water_neuron.water_neuron_id.into(),
                subaccount: Some(SNS_GOVERNANCE_SUBACCOUNT),
            }
        ),
        Nat::from(100 * E8S)
    );

    water_neuron.advance_time_and_tick(60 * 60 * 24);

    assert_eq!(
        water_neuron.balance_of(
            water_neuron.icp_ledger_id,
            Account {
                owner: PrincipalId::new_user_test_id(1234).into(),
                subaccount: None,
            }
        ),
        Nat::from(100 * E8S) - DEFAULT_LEDGER_FEE
    );

    assert_eq!(water_neuron.get_events().total_event_count, 9);

    assert_eq!(water_neuron
        .env
        .execute_ingress_as(
            PrincipalId::from(caller),
            water_neuron.water_neuron_id,
            "claim_airdrop",
            Encode!(&caller).unwrap()
        ),Err(
            UserError::new(CanisterCalledTrap, "Canister r7inp-6aaaa-aaaaa-aaabq-cai trapped explicitly: all rewards must be allocated before being claimable".to_string())
        ));

    water_neuron.approve(
        water_neuron.minter.into(),
        water_neuron.icp_ledger_id,
        water_neuron.water_neuron_id.get().0.into(),
    );

    assert!(water_neuron
        .icp_to_nicp(water_neuron.minter.into(), 21_000_000 * E8S)
        .is_ok());

    assert_eq!(water_neuron.claim_airdrop(caller.0), Ok(1));

    assert_eq!(
        water_neuron.balance_of(water_neuron.wtn_ledger_id, caller.0),
        Nat::from(15_999_000_000_u64)
    );

    assert_matches!(
        water_neuron.env.upgrade_canister(
            water_neuron.water_neuron_id,
            water_neuron_wasm(),
            Encode!(&LiquidArg::Upgrade(Some(UpgradeArg {
                governance_fee_share_percent: None
            })))
            .unwrap(),
        ),
        Ok(_)
    );

    water_neuron.advance_time_and_tick(60);
    let info = water_neuron.get_info();
    assert_eq!(info.neuron_6m_stake_e8s, info.tracked_6m_stake);
}

#[test]
fn transfer_ids_are_as_expected() {
    let water_neuron = WaterNeuron::new();
    let caller = PrincipalId::new_user_test_id(212);

    let water_neuron_principal: Principal = water_neuron.water_neuron_id.get().into();

    assert_eq!(
        water_neuron.transfer(
            water_neuron.minter,
            water_neuron_principal,
            10 * E8S,
            water_neuron.icp_ledger_id
        ),
        Nat::from(1_u8)
    );
    assert_eq!(
        water_neuron.transfer(
            water_neuron.minter,
            caller.0,
            110 * E8S,
            water_neuron.icp_ledger_id
        ),
        Nat::from(2_u8)
    );

    water_neuron.approve(
        caller,
        water_neuron.icp_ledger_id,
        water_neuron.water_neuron_id.get().0.into(),
    );

    let icp_to_wrap = 100 * E8S;

    water_neuron.advance_time_and_tick(60);

    let result = water_neuron.icp_to_nicp(caller.0.into(), icp_to_wrap);
    assert_eq!(
        result,
        Ok(DepositSuccess {
            block_index: Nat::from(6_u8),
            transfer_id: 0,
            nicp_amount: Some(nICP::from_e8s(icp_to_wrap)),
        })
    );

    let statuses = water_neuron.get_transfer_statuses(vec![0]);
    assert_eq!(
        statuses,
        vec![TransferStatus::Pending(PendingTransfer {
            transfer_id: 0,
            from_subaccount: None,
            memo: Some(6),
            amount: 100 * E8S,
            receiver: caller.0.into(),
            unit: Unit::NICP,
        }),]
    );

    assert_matches!(
        water_neuron.env.upgrade_canister(
            water_neuron.water_neuron_id,
            water_neuron_wasm(),
            Encode!(&LiquidArg::Upgrade(Some(UpgradeArg {
                governance_fee_share_percent: None
            })))
            .unwrap(),
        ),
        Ok(_)
    );

    water_neuron.advance_time_and_tick(60);
    let info = water_neuron.get_info();
    assert_eq!(info.neuron_6m_stake_e8s, info.tracked_6m_stake);
}

#[test]
fn should_compute_exchange_rate() {
    let water_neuron = WaterNeuron::new();
    let caller = PrincipalId::new_user_test_id(212);

    let water_neuron_principal: Principal = water_neuron.water_neuron_id.get().into();

    assert_eq!(
        water_neuron.transfer(
            water_neuron.minter,
            water_neuron_principal,
            10 * E8S,
            water_neuron.icp_ledger_id
        ),
        Nat::from(1_u8)
    );
    assert_eq!(
        water_neuron.transfer(
            water_neuron.minter,
            caller.0,
            110 * E8S,
            water_neuron.icp_ledger_id
        ),
        Nat::from(2_u8)
    );

    water_neuron.advance_time_and_tick(60);

    water_neuron.approve(
        water_neuron.minter,
        water_neuron.icp_ledger_id,
        water_neuron.water_neuron_id.get().0.into(),
    );

    assert_matches!(
        water_neuron.icp_to_nicp(water_neuron.minter, 1_000 * E8S),
        Ok(_)
    );

    water_neuron.advance_time_and_tick(70);

    let water_neuron_principal: Principal = water_neuron.water_neuron_id.get().into();

    assert_eq!(
        water_neuron.transfer(
            water_neuron.minter,
            Account {
                owner: water_neuron_principal,
                subaccount: Some(crate::NeuronOrigin::NICPSixMonths.to_subaccount())
            },
            100 * E8S,
            water_neuron.icp_ledger_id
        ),
        Nat::from(7_u8)
    );

    water_neuron.advance_time_and_tick(60 * 60);

    assert_matches!(
        water_neuron.icp_to_nicp(water_neuron.minter, 1_000 * E8S),
        Ok(_)
    );

    water_neuron.advance_time_and_tick(7 * 24 * 60 * 60);
    water_neuron.advance_time_and_tick(60 * 60);

    let info = water_neuron.get_info();
    let previous_rate = info.tracked_6m_stake;
    assert_eq!(info.neuron_6m_stake_e8s, info.tracked_6m_stake);
    assert_eq!(
        water_neuron.balance_of(water_neuron.icp_ledger_id, info.neuron_6m_account),
        Nat::from(info.tracked_6m_stake.0)
    );
    let prev = info.nicp_supply;

    assert_matches!(
        water_neuron.env.upgrade_canister(
            water_neuron.water_neuron_id,
            water_neuron_wasm(),
            Encode!(&LiquidArg::Upgrade(Some(UpgradeArg {
                governance_fee_share_percent: None
            })))
            .unwrap(),
        ),
        Ok(_)
    );
    let info = water_neuron.get_info();
    assert_eq!(info.nicp_supply, prev);

    water_neuron.advance_time_and_tick(0);

    water_neuron.advance_time_and_tick(60 * 60);
    let info = water_neuron.get_info();
    assert_eq!(
        water_neuron.balance_of(water_neuron.icp_ledger_id, info.neuron_6m_account),
        Nat::from(info.tracked_6m_stake.0)
    );
    assert_eq!(info.neuron_6m_stake_e8s, info.tracked_6m_stake);
    assert_eq!(info.tracked_6m_stake, previous_rate);
}

#[test]
fn should_mirror_all_proposals() {
    let mut water_neuron = WaterNeuron::new();
    water_neuron.with_voting_topic();

    let water_neuron_principal: Principal = water_neuron.water_neuron_id.get().into();
    let caller = PrincipalId::new_user_test_id(212);

    assert_eq!(
        water_neuron.transfer(
            water_neuron.minter,
            water_neuron_principal,
            10 * E8S,
            water_neuron.icp_ledger_id
        ),
        Nat::from(1_u8)
    );
    assert_eq!(
        water_neuron.transfer(
            water_neuron.minter,
            caller.0,
            100 * E8S,
            water_neuron.icp_ledger_id
        ),
        Nat::from(2_u8)
    );

    water_neuron.advance_time_and_tick(60);

    assert_eq!(
        water_neuron.transfer(
            water_neuron.minter,
            Account {
                owner: GOVERNANCE_CANISTER_ID.into(),
                subaccount: Some(compute_neuron_staking_subaccount_bytes(caller.into(), 0))
            },
            11 * E8S,
            water_neuron.icp_ledger_id
        ),
        Nat::from(5_u8)
    );

    water_neuron.approve(
        water_neuron.minter,
        water_neuron.icp_ledger_id,
        water_neuron.water_neuron_id.get().0.into(),
    );

    assert_eq!(
        water_neuron.icp_to_nicp(water_neuron.minter, 1_000 * E8S),
        Ok(DepositSuccess {
            block_index: Nat::from(7_u8),
            transfer_id: 0,
            nicp_amount: Some(nICP::from_unscaled(1_000)),
        })
    );

    water_neuron.advance_time_and_tick(70);

    let neuron_id = nns_claim_or_refresh_neuron(&mut water_neuron.env, caller, 0);

    let _increase_dissolve_delay_result =
        nns_increase_dissolve_delay(&mut water_neuron.env, caller, neuron_id, 200 * 24 * 60 * 60);

    water_neuron.advance_time_and_tick(70);

    assert_matches!(
        water_neuron.env.upgrade_canister(
            water_neuron.water_neuron_id,
            water_neuron_wasm(),
            Encode!(&LiquidArg::Upgrade(Some(UpgradeArg {
                governance_fee_share_percent: None,
            })))
            .unwrap(),
        ),
        Ok(_)
    );

    let mut proposals = vec![];

    proposals.push(Proposal {
        title: Some("Dummy Motion".to_string()),
        summary: "Dummy Proposal".to_string(),
        url: "https://forum.dfinity.org/t/reevaluating-neuron-control-restrictions/28597/215"
            .to_string(),
        action: Some(Action::Motion(crate::nns_types::Motion {
            motion_text: "Some text".to_string(),
        })),
    });

    proposals.push(Proposal {
        title: Some("Dummy ManageNetworkEconomics".to_string()),
        summary: "Dummy Proposal".to_string(),
        url: "https://forum.dfinity.org/t/reevaluating-neuron-control-restrictions/28597/215"
            .to_string(),
        action: Some(Action::ManageNetworkEconomics(
            crate::nns_types::NetworkEconomics {
                reject_cost_e8s: 0,
                neuron_minimum_stake_e8s: 0,
                neuron_management_fee_per_proposal_e8s: 0,
                minimum_icp_xdr_rate: 0,
                neuron_spawn_dissolve_delay_seconds: 0,
                maximum_node_provider_rewards_e8s: 0,
                transaction_fee_e8s: 0,
                max_proposals_to_keep_per_topic: 0,
            },
        )),
    });

    proposals.push(Proposal {
        title: Some("Upgrade NNS Canister: qoctq-giaaa-aaaaa-aaaea-cai to wasm with hash: 7e355e66705fa6772c6a4cfdec1bd3853429bcee9553b44ff440a1bbb9abd748".to_string()),
        summary: "Dummy Proposal".to_string(),
        url: "https://forum.dfinity.org/t/reevaluating-neuron-control-restrictions/28597/215"
            .to_string(),
        action: Some(Action::ExecuteNnsFunction(crate::nns_types::ExecuteNnsFunction {
                nns_function: 4,
                payload: vec![],
        })),
    });

    for proposal in proposals {
        let res = nns_governance_make_proposal(&mut water_neuron.env, caller, neuron_id, &proposal)
            .command
            .unwrap();
        dbg!(res.clone());
        match res {
            CommandResponse::MakeProposal(_rees) => {}
            _ => panic!("unexpected response"),
        }
    }

    assert_eq!(water_neuron.get_pending_proposals().len(), 3);
    dbg!(water_neuron.get_pending_proposals());

    water_neuron.advance_time_and_tick(30 * 60);

    let proposals = water_neuron.list_proposals(
        water_neuron.wtn_governance_id,
        ListProposals {
            include_reward_status: vec![],
            before_proposal: None,
            limit: 10,
            exclude_type: vec![],
            include_status: vec![],
        },
    );
    assert_eq!(proposals.proposals.len(), 4);
}
