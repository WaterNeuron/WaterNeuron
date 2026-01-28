use crate::state::event::{GetEventsArg, GetEventsResult};
use crate::state::{TransferStatus, WithdrawalDetails};
use crate::{
    CancelWithdrawalError, CanisterInfo, ConversionArg, ConversionError, DEFAULT_LEDGER_FEE,
    DepositSuccess, E8S, INITIAL_NEURON_STAKE, InitArg, LiquidArg, NEURON_LEDGER_FEE, NeuronId,
    WithdrawalSuccess, compute_neuron_staking_subaccount_bytes, nICP,
};
use assert_matches::assert_matches;
use candid::{CandidType, Deserialize, Encode, Nat, Principal};
use cycles_minting_canister::CyclesCanisterInitPayload;
use ic_base_types::PrincipalId;
use ic_icrc1_ledger::{
    ArchiveOptions, InitArgs as LedgerInitArgs, InitArgsBuilder as LedgerInitArgsBuilder,
    LedgerArgument,
};
use ic_nns_constants::{CYCLES_LEDGER_CANISTER_ID, GOVERNANCE_CANISTER_ID, LEDGER_CANISTER_ID};
use ic_nns_governance_api::{
    Governance, GovernanceError, MakeProposalRequest, ManageNeuronCommandRequest,
    ManageNeuronRequest, ManageNeuronResponse, NetworkEconomics, Neuron, ProposalInfo,
    manage_neuron::{
        self,
        claim_or_refresh::{self, MemoAndController},
    },
    manage_neuron_response::{ClaimOrRefreshResponse, Command as CommandResponse, MergeResponse},
};
use ic_sns_governance::init::GovernanceCanisterInitPayloadBuilder;
use ic_sns_governance::pb::v1::{
    Neuron as SnsNeuron, NeuronId as SnsNeuronId, NeuronPermission, NeuronPermissionType,
    governance::Version, neuron::DissolveState,
};
use ic_sns_governance_api::pb::v1::{
    ListProposals, ListProposalsResponse, ManageNeuron as SnsManageNeuron,
    ManageNeuronResponse as SnsManageNeuronResponse, NervousSystemFunction,
    Proposal as SnsProposal,
    manage_neuron::Command as SnsCommand,
    nervous_system_function::{FunctionType, GenericNervousSystemFunction},
    proposal::Action as SnsAction,
    topics::Topic,
};
use ic_sns_init::SnsCanisterInitPayloads;
use ic_sns_root::pb::v1::SnsRootCanister;
use ic_sns_swap::pb::v1::{Init as SwapInit, NeuronBasketConstructionParameters};
use ic_wasm_utils::{
    cmc_wasm, governance_wasm, icp_ledger_wasm, ledger_wasm, sns_governance_wasm, sns_root_wasm,
    sns_swap_wasm, water_neuron_wasm,
};
use icp_ledger::{AccountIdentifier, LedgerCanisterInitPayload, Tokens};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::{TransferArg, TransferError};
use icrc_ledger_types::icrc2::approve::{ApproveArgs, ApproveError};
use pocket_ic::{
    PocketIcBuilder, RejectResponse,
    nonblocking::{PocketIc, query_candid_as, update_candid_as},
};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

mod e2e;
mod tests;

type CanisterId = Principal;

const DEFAULT_PRINCIPAL_ID: u64 = 10352385;

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

pub async fn upgrade_canister(
    pic: &PocketIc,
    canister_id: CanisterId,
    wasm_module: Vec<u8>,
    arg: Vec<u8>,
) -> Result<(), String> {
    pic.upgrade_canister(canister_id, wasm_module, arg, None)
        .await
        .map_err(|e| format!("{e:?}"))
}

pub async fn update<T>(
    pic: &PocketIc,
    canister: CanisterId,
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
    canister: CanisterId,
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

async fn nns_governance_make_proposal(
    state_machine: &mut WaterNeuron,
    sender: PrincipalId,
    neuron_id: NeuronId,
    proposal: &MakeProposalRequest,
) -> ManageNeuronResponse {
    let command = ManageNeuronCommandRequest::MakeProposal(Box::new(proposal.clone()));

    manage_neuron(state_machine, sender, neuron_id, command).await
}

async fn manage_neuron(
    env: &mut WaterNeuron,
    sender: PrincipalId,
    neuron_id: NeuronId,
    command: ManageNeuronCommandRequest,
) -> ManageNeuronResponse {
    env.update::<ManageNeuronResponse>(
        sender,
        GOVERNANCE_CANISTER_ID.into(),
        "manage_neuron",
        ManageNeuronRequest {
            id: Some(neuron_id.to_pb()),
            command: Some(command),
            neuron_id_or_subaccount: None,
        },
    )
    .await
    .unwrap()
}

async fn nns_claim_or_refresh_neuron(
    env: &mut WaterNeuron,
    controller: PrincipalId,
    memo: u64,
) -> NeuronId {
    // Construct request.
    let command = Some(ManageNeuronCommandRequest::ClaimOrRefresh(
        manage_neuron::ClaimOrRefresh {
            by: Some(claim_or_refresh::By::MemoAndController(MemoAndController {
                memo,
                controller: Some(controller.into()),
            })),
        },
    ));
    let manage_neuron = ManageNeuronRequest {
        id: None,
        command,
        neuron_id_or_subaccount: None,
    };

    // Call governance.
    let result = env
        .update::<ManageNeuronResponse>(
            controller,
            GOVERNANCE_CANISTER_ID.into(),
            "manage_neuron",
            manage_neuron,
        )
        .await
        .unwrap();
    let neuron_id = match &result.command {
        Some(CommandResponse::ClaimOrRefresh(ClaimOrRefreshResponse {
            refreshed_neuron_id: Some(neuron_id),
        })) => neuron_id,
        _ => panic!("{:?}", result),
    };
    NeuronId { id: neuron_id.id }
}

async fn nns_increase_dissolve_delay(
    state_machine: &mut WaterNeuron,
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
    .await
}

async fn nns_configure_neuron(
    state_machine: &mut WaterNeuron,
    sender: PrincipalId,
    neuron_id: NeuronId,
    operation: manage_neuron::configure::Operation,
) -> ManageNeuronResponse {
    manage_neuron(
        state_machine,
        sender,
        neuron_id,
        ManageNeuronCommandRequest::Configure(manage_neuron::Configure {
            operation: Some(operation),
        }),
    )
    .await
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
                controller_id: CanisterId::from(
                    Principal::from_text("r7inp-6aaaa-aaaaa-aaabq-cai").unwrap(),
                )
                .into(),
                more_controller_ids: None,
                cycles_for_archive_creation: Some(0),
                max_transactions_per_response: None,
            })
            .with_transfer_fee(NEURON_LEDGER_FEE)
            .build();

        let swap = SwapInit {
            fallback_controller_principal_ids: vec![
                PrincipalId::new_user_test_id(6360).to_string(),
            ],
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
            neuron_map.insert(
                format!("{}", hex::encode(neuron.id.as_ref().unwrap().id.clone())),
                neuron,
            );
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
            fallback_controller_principal_ids: vec![
                PrincipalId::new_user_test_id(6360).to_string(),
            ],
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
    root_canister_id: Principal,
    governance_canister_id: Principal,
    ledger_canister_id: Principal,
    swap_canister_id: Principal,
    index_canister_id: Principal,
    sns_canister_init_payloads: &mut SnsCanisterInitPayloads,
) {
    // Root.
    {
        let root = &mut sns_canister_init_payloads.root;
        if root.governance_canister_id.is_none() {
            root.governance_canister_id = Some(governance_canister_id.into());
        }
        if root.ledger_canister_id.is_none() {
            root.ledger_canister_id = Some(ledger_canister_id.into());
        }
        if root.swap_canister_id.is_none() {
            root.swap_canister_id = Some(swap_canister_id.into());
        }
        if root.index_canister_id.is_none() {
            root.index_canister_id = Some(index_canister_id.into());
        }
        if root.archive_canister_ids.is_empty() {
            root.archive_canister_ids = vec![];
        }
    }
    // Governance canister_init args.
    {
        let governance = &mut sns_canister_init_payloads.governance;
        governance.ledger_canister_id = Some(ledger_canister_id.into());
        governance.root_canister_id = Some(root_canister_id.into());
        governance.swap_canister_id = Some(swap_canister_id.into());
    }
    // Ledger
    {
        if let LedgerArgument::Init(ref mut ledger) = sns_canister_init_payloads.ledger {
            // ledger.minting_account = Account {
            //     owner: governance_canister_id.0,
            //     subaccount: None,
            // };
            ledger.archive_options.controller_id = root_canister_id.into();
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

async fn setup_sns_canisters(pic: &PocketIc, neurons: Vec<SnsNeuron>) -> SNSCanisterIds {
    let root_canister_id = pic.create_canister().await;
    let governance_canister_id = pic.create_canister().await;
    let ledger_canister_id = pic.create_canister().await;
    let swap_canister_id = pic.create_canister().await;
    let index_canister_id = pic.create_canister().await;

    pic.add_cycles(root_canister_id, u64::MAX.into()).await;
    pic.add_cycles(governance_canister_id, u64::MAX.into())
        .await;
    pic.add_cycles(ledger_canister_id, u64::MAX.into()).await;
    pic.add_cycles(swap_canister_id, u64::MAX.into()).await;
    pic.add_cycles(index_canister_id, u64::MAX.into()).await;

    let mut payloads = SnsTestsInitPayloadBuilder::new()
        .with_initial_neurons(neurons)
        .build();

    populate_canister_ids(
        root_canister_id,
        governance_canister_id,
        ledger_canister_id,
        swap_canister_id,
        index_canister_id,
        &mut payloads,
    );

    let deployed_version = Version {
        root_wasm_hash: sha256_hash(sns_root_wasm().await),
        governance_wasm_hash: sha256_hash(sns_governance_wasm().await),
        ledger_wasm_hash: sha256_hash(ledger_wasm().await),
        swap_wasm_hash: sha256_hash(sns_swap_wasm().await),
        archive_wasm_hash: vec![], // tests don't need it for now so we don't compile it.
        index_wasm_hash: vec![],
    };

    payloads.governance.deployed_version = Some(deployed_version);

    pic.install_canister(
        governance_canister_id,
        sns_governance_wasm().await,
        Encode!(&payloads.governance).unwrap(),
        None,
    )
    .await;
    pic.install_canister(
        root_canister_id,
        sns_root_wasm().await,
        Encode!(&payloads.root).unwrap(),
        None,
    )
    .await;
    pic.install_canister(
        swap_canister_id,
        sns_swap_wasm().await,
        Encode!(&payloads.swap).unwrap(),
        None,
    )
    .await;
    pic.install_canister(
        ledger_canister_id,
        ledger_wasm().await,
        Encode!(&payloads.ledger).unwrap(),
        None,
    )
    .await;
    SNSCanisterIds {
        governance: governance_canister_id,
        ledger: ledger_canister_id,
    }
}

struct WaterNeuron {
    pub env: Arc<Mutex<PocketIc>>,
    pub minter: PrincipalId,
    pub water_neuron_id: CanisterId,
    pub wtn_ledger_id: CanisterId,
    pub wtn_governance_id: CanisterId,
    pub icp_ledger_id: CanisterId,
    pub nicp_ledger_id: CanisterId,
    pub governance_id: CanisterId,
}

impl WaterNeuron {
    async fn new() -> Self {
        let env = PocketIcBuilder::new()
            .with_nns_subnet()
            .with_sns_subnet()
            .with_ii_subnet()
            .build_async()
            .await;

        let water_neuron_canister_id = env.create_canister().await;

        let minter = PrincipalId::new_user_test_id(DEFAULT_PRINCIPAL_ID);

        let mut initial_balances = HashMap::new();
        initial_balances.insert(
            AccountIdentifier::new(minter.into(), None),
            Tokens::from_e8s(22_000_000 * E8S),
        );
        initial_balances.insert(
            AccountIdentifier::new(PrincipalId(water_neuron_canister_id), None),
            Tokens::from_e8s(5 * E8S),
        );

        env.create_canister_with_id(None, None, LEDGER_CANISTER_ID.into())
            .await
            .unwrap();

        env.install_canister(
            LEDGER_CANISTER_ID.into(),
            icp_ledger_wasm().await,
            Encode!(
                &LedgerCanisterInitPayload::builder()
                    .initial_values(initial_balances)
                    .transfer_fee(Tokens::from_e8s(10_000))
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

        env.create_canister_with_id(None, None, GOVERNANCE_CANISTER_ID.into())
            .await
            .unwrap();

        env.install_canister(
            GOVERNANCE_CANISTER_ID.into(),
            governance_wasm().await,
            Encode!(&Governance {
                economics: Some(NetworkEconomics::with_default_values()),
                wait_for_quiet_threshold_seconds: 60 * 60 * 24 * 4, // 4 days
                short_voting_period_seconds: 60 * 60 * 12,          // 12 hours
                neuron_management_voting_period_seconds: Some(60 * 60 * 48), // 48 hours
                ..Default::default()
            })
            .unwrap(),
            None,
        )
        .await;

        let mut neurons = vec![];
        neurons.push(SnsNeuron {
            id: Some(SnsNeuronId {
                id: compute_neuron_staking_subaccount_bytes(water_neuron_canister_id, 0).to_vec(),
            }),
            permissions: vec![NeuronPermission {
                principal: Some(PrincipalId(water_neuron_canister_id)),
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

        let cycles_minting_canister_id =
            CanisterId::from(Principal::from_text("rkp4c-7iaaa-aaaaa-aaaca-cai").unwrap());

        env.create_canister_with_id(None, None, cycles_minting_canister_id)
            .await
            .unwrap();

        let _cmc_id = env
            .install_canister(
                cycles_minting_canister_id,
                cmc_wasm().await,
                Encode!(&Some(CyclesCanisterInitPayload {
                    ledger_canister_id: Some(LEDGER_CANISTER_ID),
                    governance_canister_id: Some(GOVERNANCE_CANISTER_ID.into()),
                    exchange_rate_canister: None,
                    minting_account_id: Some(GOVERNANCE_CANISTER_ID.into()),
                    last_purged_notification: Some(1),
                    cycles_ledger_canister_id: Some(CYCLES_LEDGER_CANISTER_ID),
                }))
                .unwrap(),
                None,
            )
            .await;

        let sns = setup_sns_canisters(&env, neurons).await;

        let nicp_ledger_id = env.create_canister().await;
        env.install_canister(
            water_neuron_canister_id,
            water_neuron_wasm(),
            Encode!(&LiquidArg::Init(InitArg {
                wtn_governance_id: sns.governance,
                wtn_ledger_id: sns.ledger,
                nicp_ledger_id,
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
                    .with_minting_account(water_neuron_canister_id)
                    .with_transfer_fee(DEFAULT_LEDGER_FEE)
                    .with_decimals(8)
                    .with_feature_flags(ic_icrc1_ledger::FeatureFlags { icrc2: true })
                    .build(),
            ))
            .unwrap(),
            None,
        )
        .await;

        env.add_cycles(LEDGER_CANISTER_ID.into(), u64::MAX.into())
            .await;
        env.add_cycles(GOVERNANCE_CANISTER_ID.into(), u64::MAX.into())
            .await;
        env.add_cycles(nicp_ledger_id, u64::MAX.into()).await;
        env.add_cycles(sns.ledger, u64::MAX.into()).await;
        env.add_cycles(sns.governance, u64::MAX.into()).await;
        env.add_cycles(CanisterId::from(water_neuron_canister_id), u64::MAX.into())
            .await;

        WaterNeuron {
            env: Arc::new(Mutex::new(env)),
            minter,
            water_neuron_id: CanisterId::from(water_neuron_canister_id),
            wtn_ledger_id: sns.ledger,
            wtn_governance_id: sns.governance,
            icp_ledger_id: LEDGER_CANISTER_ID.into(),
            nicp_ledger_id,
            governance_id: GOVERNANCE_CANISTER_ID.into(),
        }
    }

    pub async fn tick(&self) {
        let pic = self.env.lock().await;
        pic.advance_time(Duration::from_secs(1)).await;
        pic.tick().await;
    }

    pub async fn advance_time_and_tick(&self, duration_secs: u64) {
        let pic = self.env.lock().await;
        pic.advance_time(Duration::from_secs(duration_secs)).await;
        const MAX_TICKS: u8 = 10;
        for _ in 0..MAX_TICKS {
            pic.tick().await;
        }
    }

    async fn update<T>(
        &self,
        caller: PrincipalId,
        canister: CanisterId,
        method: &str,
        arg: impl CandidType,
    ) -> Result<T, String>
    where
        T: for<'a> Deserialize<'a> + CandidType,
    {
        let pic = self.env.lock().await;
        update(&pic, canister, caller.into(), method, arg).await
    }

    async fn upgrade_canister(
        &self,
        canister_id: CanisterId,
        wasm_module: Vec<u8>,
        arg: Vec<u8>,
    ) -> Result<(), String> {
        let pic = self.env.lock().await;
        upgrade_canister(&pic, canister_id, wasm_module, arg).await
    }

    async fn query<T>(
        &self,
        canister: CanisterId,
        method: &str,
        arg: impl CandidType,
    ) -> Result<T, String>
    where
        T: for<'a> Deserialize<'a> + CandidType,
    {
        let pic = self.env.lock().await;
        query(&pic, canister, Principal::anonymous(), method, arg).await
    }

    async fn with_voting_topic(&self) -> &WaterNeuron {
        let nervous_system_function = NervousSystemFunction {
            id: 1000,
            name: "a".to_string(),
            description: None,
            function_type: Some(FunctionType::GenericNervousSystemFunction(
                GenericNervousSystemFunction {
                    topic: Some(Topic::ApplicationBusinessLogic),
                    target_canister_id: Some(ic_base_types::PrincipalId(self.water_neuron_id)),
                    target_method_name: Some("approve_proposal".to_string()),
                    validator_canister_id: Some(ic_base_types::PrincipalId(self.water_neuron_id)),
                    validator_method_name: Some("approve_proposal_validate".to_string()),
                },
            )),
        };

        let proposal_payload = SnsProposal {
            title: "Add new GenericNervousSystemFunction".into(),
            action: Some(SnsAction::AddGenericNervousSystemFunction(
                nervous_system_function.clone(),
            )),
            summary: String::new(),
            url: String::new(),
        };

        let res = self
            .wtn_make_proposal(PrincipalId::new_user_test_id(1234).0, proposal_payload)
            .await;

        println!("[with_voting_topic] {res:?}");

        self.advance_time_and_tick(60).await;

        self
    }

    async fn wtn_make_proposal(
        &self,
        caller: Principal,
        proposal: SnsProposal,
    ) -> SnsManageNeuronResponse {
        self.update::<SnsManageNeuronResponse>(
            PrincipalId::from(caller),
            self.wtn_governance_id,
            "manage_neuron",
            SnsManageNeuron {
                subaccount: compute_neuron_staking_subaccount_bytes(
                    PrincipalId::new_user_test_id(1234).0,
                    0,
                )
                .to_vec(),
                command: Some(SnsCommand::MakeProposal(proposal)),
            },
        )
        .await
        .expect("failed to wtn_make_proposal")
    }

    async fn transfer(
        &self,
        caller: PrincipalId,
        to: impl Into<Account>,
        amount: u64,
        ledger_id: CanisterId,
    ) -> Nat {
        self.update::<Result<Nat, TransferError>>(
            caller,
            ledger_id,
            "icrc1_transfer",
            TransferArg {
                from_subaccount: None,
                to: to.into(),
                fee: None,
                created_at_time: None,
                memo: None,
                amount: Nat::from(amount),
            },
        )
        .await
        .expect("failed to execute token transfer")
        .unwrap()
    }

    async fn approve(&self, caller: PrincipalId, ledger: CanisterId, spender: Account) {
        assert_matches!(
            self.update::<Result<Nat, ApproveError>>(
                caller,
                ledger,
                "icrc2_approve",
                ApproveArgs {
                    from_subaccount: None,
                    spender,
                    amount: u64::MAX.into(),
                    expected_allowance: None,
                    expires_at: None,
                    fee: None,
                    memo: None,
                    created_at_time: None,
                }
            )
            .await
            .expect("failed to approve protocol canister"),
            Ok(_)
        );
    }

    pub async fn balance_of(&self, canister_id: CanisterId, from: impl Into<Account>) -> Nat {
        let from = from.into();
        self.query::<Nat>(canister_id, "icrc1_balance_of", from)
            .await
            .expect("failed to execute token transfer")
    }

    async fn icp_to_nicp(
        &self,
        caller: PrincipalId,
        amount_e8s: u64,
    ) -> Result<DepositSuccess, ConversionError> {
        self.update::<Result<DepositSuccess, ConversionError>>(
            caller,
            self.water_neuron_id,
            "icp_to_nicp",
            ConversionArg {
                amount_e8s,
                maybe_subaccount: None,
            },
        )
        .await
        .expect("failed to deposit")
    }

    async fn nicp_to_icp(
        &self,
        caller: PrincipalId,
        amount_e8s: u64,
    ) -> Result<WithdrawalSuccess, ConversionError> {
        self.update::<Result<WithdrawalSuccess, ConversionError>>(
            caller,
            self.water_neuron_id,
            "nicp_to_icp",
            ConversionArg {
                amount_e8s,
                maybe_subaccount: None,
            },
        )
        .await
        .expect("failed to withdraw")
    }

    async fn cancel_withdrawal(
        &self,
        caller: PrincipalId,
        neuron_id: NeuronId,
    ) -> Result<MergeResponse, CancelWithdrawalError> {
        self.update::<Result<MergeResponse, CancelWithdrawalError>>(
            caller,
            self.water_neuron_id,
            "cancel_withdrawal",
            neuron_id,
        )
        .await
        .expect("failed to cancel_withdrawal")
    }

    async fn get_airdrop_allocation(&self, caller: Principal) -> u64 {
        self.update::<u64>(
            PrincipalId::from(caller),
            self.water_neuron_id,
            "get_airdrop_allocation",
            caller,
        )
        .await
        .expect("failed to get get_airdrop_allocation")
    }

    async fn get_full_neuron(
        &self,
        neuron_id: u64,
    ) -> Result<Result<Neuron, GovernanceError>, String> {
        self.update::<Result<Result<Neuron, GovernanceError>, String>>(
            Principal::from_text("bo5bf-eaaaa-aaaam-abtza-cai")
                .unwrap()
                .into(),
            self.water_neuron_id,
            "get_full_neuron",
            neuron_id,
        )
        .await
        .expect("failed to get get_airdrop_allocation")
    }

    async fn claim_airdrop(&self, caller: Principal) -> Result<u64, ConversionError> {
        self.update::<Result<u64, ConversionError>>(
            PrincipalId::from(caller),
            self.water_neuron_id,
            "claim_airdrop",
            caller,
        )
        .await
        .expect("failed to get claim_airdrop")
    }

    async fn get_transfer_statuses(&self, ids: Vec<u64>) -> Vec<TransferStatus> {
        self.query::<Vec<TransferStatus>>(self.water_neuron_id, "get_transfer_statuses", ids)
            .await
            .expect("failed to get get_transfer_statuses")
    }

    async fn approve_proposal(&self, id: u64) -> Result<ManageNeuronResponse, String> {
        self.update::<Result<ManageNeuronResponse, String>>(
            PrincipalId::from(self.wtn_governance_id),
            self.water_neuron_id,
            "approve_proposal",
            id,
        )
        .await
        .expect("failed to get approve_proposal")
    }

    async fn get_info(&self) -> CanisterInfo {
        self.query::<CanisterInfo>(self.water_neuron_id, "get_info", Encode!().unwrap())
            .await
            .expect("failed to get info")
    }

    async fn get_events(&self) -> GetEventsResult {
        self.query::<GetEventsResult>(
            self.water_neuron_id,
            "get_events",
            GetEventsArg {
                start: 0,
                length: 2000,
            },
        )
        .await
        .expect("failed to call")
    }

    async fn get_withdrawal_requests(&self, target: impl Into<Account>) -> Vec<WithdrawalDetails> {
        let target_account: Account = target.into();
        self.query::<Vec<WithdrawalDetails>>(
            self.water_neuron_id,
            "get_withdrawal_requests",
            Some(target_account),
        )
        .await
        .expect("failed to execute get_withdrawal_requests")
    }

    async fn get_pending_proposals(&self) -> Vec<ProposalInfo> {
        self.query::<Vec<ProposalInfo>>(
            self.governance_id,
            "get_pending_proposals",
            Encode!().unwrap(),
        )
        .await
        .expect("failed to get_pending_proposals")
    }

    async fn get_proposal_info(&self, id: u64) -> Option<ProposalInfo> {
        self.query::<Option<ProposalInfo>>(self.governance_id, "get_proposal_info", &id)
            .await
            .expect("failed to get_proposal_info")
    }

    async fn list_proposals(
        &self,
        canister: CanisterId,
        arg: ListProposals,
    ) -> ListProposalsResponse {
        self.query::<ListProposalsResponse>(canister, "list_proposals", arg)
            .await
            .expect("failed to list_proposals")
    }
}

async fn init_wtn_withdrawal_setup(wtn: &mut WaterNeuron) {
    wtn.with_voting_topic().await;

    let caller = PrincipalId::new_user_test_id(212);

    // Transfer
    //  10 ICP -> WTN
    // 110 ICP -> caller
    //  11 ICP -> caller deposit account
    assert_eq!(
        wtn.transfer(wtn.minter, wtn.water_neuron_id, 10 * E8S, wtn.icp_ledger_id)
            .await,
        Nat::from(4_u8)
    );
    assert_eq!(
        wtn.transfer(wtn.minter, caller.0, 110 * E8S, wtn.icp_ledger_id)
            .await,
        Nat::from(5_u8)
    );
    assert_eq!(
        wtn.transfer(
            wtn.minter,
            Account {
                owner: GOVERNANCE_CANISTER_ID.into(),
                subaccount: Some(compute_neuron_staking_subaccount_bytes(caller.into(), 0))
            },
            11 * E8S,
            wtn.icp_ledger_id
        )
        .await,
        Nat::from(6_u8)
    );

    // Notify NNS to update neuron.
    let neuron_id = nns_claim_or_refresh_neuron(wtn, caller, 0).await;
    nns_increase_dissolve_delay(wtn, caller, neuron_id, 200 * 24 * 60 * 60).await;

    wtn.advance_time_and_tick(70).await;

    let caller_icp_balance_before = Nat::from(110 * E8S);
    assert_eq!(
        wtn.balance_of(wtn.icp_ledger_id, caller.0).await,
        caller_icp_balance_before,
    );

    let info = wtn.get_info().await;
    let wtn_6m_start_icp_balance = Nat::from(INITIAL_NEURON_STAKE);
    assert_eq!(
        wtn.balance_of(wtn.icp_ledger_id, info.neuron_6m_account)
            .await,
        wtn_6m_start_icp_balance
    );

    wtn.approve(caller, wtn.icp_ledger_id, wtn.water_neuron_id.into())
        .await; // tx_id: 7
    let icp_to_wrap = 100 * E8S;
    assert_eq!(
        wtn.icp_to_nicp(caller.0.into(), icp_to_wrap).await,
        Ok(DepositSuccess {
            block_index: Nat::from(8_u8),
            transfer_id: 0,
            nicp_amount: Some(nICP::from_e8s(icp_to_wrap)),
        })
    );

    // Init, NeuronSixMonths, NeuronEightYears
    // + IcpDeposit
    assert_eq!(wtn.get_events().await.total_event_count, 4);

    wtn.tick().await;

    assert_eq!(
        wtn.balance_of(wtn.icp_ledger_id, info.neuron_6m_account)
            .await,
        Nat::from(wtn_6m_start_icp_balance + icp_to_wrap)
    );
    assert_eq!(
        wtn.balance_of(wtn.nicp_ledger_id, caller.0).await,
        Nat::from(icp_to_wrap)
    );
    assert_eq!(
        wtn.balance_of(wtn.icp_ledger_id, caller.0).await,
        caller_icp_balance_before - icp_to_wrap - 2 * 10_000u64 // fee: approve + transfer_from
    );

    wtn.approve(caller, wtn.nicp_ledger_id, wtn.water_neuron_id.into())
        .await;
    assert_eq!(
        wtn.balance_of(wtn.nicp_ledger_id, caller.0).await,
        Nat::from(99_99_990_000_u64) // 100 ICP - tx fee (approve)
    );
}
