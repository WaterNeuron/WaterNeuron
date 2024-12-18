use candid::{Encode, Nat, Principal};
use ic_base_types::{CanisterId, PrincipalId};
use ic_icrc1_ledger::{
    ArchiveOptions, InitArgs as LedgerInitArgs, InitArgsBuilder as LedgerInitArgsBuilder,
    LedgerArgument,
};
use ic_nns_constants::{GOVERNANCE_CANISTER_ID, LEDGER_CANISTER_ID};
use ic_sns_governance::init::GovernanceCanisterInitPayloadBuilder;
use ic_sns_governance::pb::v1::governance::Version;
use ic_sns_governance::pb::v1::Neuron;
use ic_sns_init::SnsCanisterInitPayloads;
use ic_sns_root::pb::v1::SnsRootCanister;
use ic_sns_swap::pb::v1::{Init as SwapInit, NeuronBasketConstructionParameters};
use ic_state_machine_tests::{StateMachine, WasmResult};
use ic_wasm_utils::{ledger_wasm, sns_governance_wasm, sns_root_wasm, sns_swap_wasm};
use icp_ledger::Tokens;
use icrc_ledger_types::icrc1::account::Account;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

const NEURON_LEDGER_FEE: u64 = 1_000_000;

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

fn populate_canister_ids(
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

pub struct SnsCanisterIds {
    pub governance: CanisterId,
    pub ledger: CanisterId,
}

pub fn setup_sns_canisters(env: &StateMachine, neurons: Vec<Neuron>) -> SnsCanisterIds {
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
        root_wasm_hash: sha256_hash(sns_root_wasm()),
        governance_wasm_hash: sha256_hash(sns_governance_wasm()),
        ledger_wasm_hash: sha256_hash(ledger_wasm()),
        swap_wasm_hash: sha256_hash(sns_swap_wasm()),
        archive_wasm_hash: vec![], // tests don't need it for now so we don't compile it.
        index_wasm_hash: vec![],
    };

    payloads.governance.deployed_version = Some(deployed_version);

    env.install_existing_canister(
        governance_canister_id,
        sns_governance_wasm(),
        Encode!(&payloads.governance).unwrap(),
    )
    .unwrap();
    env.install_existing_canister(
        root_canister_id,
        sns_root_wasm(),
        Encode!(&payloads.root).unwrap(),
    )
    .unwrap();
    env.install_existing_canister(
        swap_canister_id,
        sns_swap_wasm(),
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
pub fn compute_neuron_staking_subaccount_bytes(controller: Principal, nonce: u64) -> [u8; 32] {
    const DOMAIN: &[u8] = b"neuron-stake";
    const DOMAIN_LENGTH: [u8; 1] = [0x0c];

    let mut hasher = Sha256::new();
    hasher.update(DOMAIN_LENGTH);
    hasher.update(DOMAIN);
    hasher.update(controller.as_slice());
    hasher.update(nonce.to_be_bytes());
    hasher.finalize().into()
}

pub fn assert_reply(result: WasmResult) -> Vec<u8> {
    match result {
        WasmResult::Reply(bytes) => bytes,
        WasmResult::Reject(reject) => {
            panic!("Expected a successful reply, got a reject: {}", reject)
        }
    }
}

pub fn sha256_hash(data: Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(&data);
    hasher.finalize().to_vec()
}
