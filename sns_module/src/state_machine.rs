use crate::state::InitArg as SnsModuleInitArg;
use crate::{derive_staking, DEV_WALLET, E8S};
use assert_matches::assert_matches;
use candid::{Decode, Encode, Nat, Principal};
use ic_base_types::{CanisterId, PrincipalId};
use ic_icrc1_ledger::{InitArgsBuilder as LedgerInitArgsBuilder, LedgerArgument};
use ic_management_canister_types::CanisterInstallMode;
use ic_state_machine_tests::{StateMachine, WasmResult};
use ic_wasm_utils::{icp_ledger_wasm, ledger_wasm, sns_module_wasm};
use icp_ledger::{
    AccountIdentifier, LedgerCanisterInitPayload, TimeStamp, Tokens,
    TransferArgs as ICPTransferArgs, TransferError as ICPTransferError,
};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::{TransferArg, TransferError};
use std::collections::HashMap;
use std::time::Duration;

const DEFAULT_PRINCIPAL_ID: u64 = 10352385;

fn assert_reply(result: WasmResult) -> Vec<u8> {
    match result {
        WasmResult::Reply(bytes) => bytes,
        WasmResult::Reject(reject) => {
            panic!("Expected a successful reply, got a reject: {}", reject)
        }
    }
}

struct SnsModuleEnv {
    pub env: StateMachine,
    pub minter: PrincipalId,
    pub user: PrincipalId,
    pub sns_module_id: CanisterId,
    pub wtn_ledger_id: CanisterId,
    pub icp_ledger_id: CanisterId,
}

impl SnsModuleEnv {
    fn new() -> Self {
        let minter = PrincipalId::new_user_test_id(DEFAULT_PRINCIPAL_ID);
        let user = PrincipalId::new_user_test_id(42);

        let env = StateMachine::new();

        let mut initial_balances = HashMap::new();
        initial_balances.insert(
            AccountIdentifier::new(minter.into(), None),
            Tokens::from_e8s(22_000_000 * E8S),
        );
        initial_balances.insert(
            AccountIdentifier::new(user.into(), None),
            Tokens::from_e8s(500_000 * E8S),
        );

        let icp_ledger_id = env
            .install_canister(
                icp_ledger_wasm(),
                Encode!(&LedgerCanisterInitPayload::builder()
                    .initial_values(initial_balances)
                    .transfer_fee(Tokens::from_e8s(10_000))
                    .minting_account(Principal::anonymous().into())
                    .token_symbol_and_name("ICP", "Internet Computer")
                    .feature_flags(icp_ledger::FeatureFlags { icrc2: true })
                    .build()
                    .unwrap())
                .unwrap(),
                None,
            )
            .unwrap();

        const SEC_NANOS: u64 = 1_000_000_000;
        let start_ts = env.get_time().as_nanos_since_unix_epoch() / SEC_NANOS;
        const ONE_WEEK: u64 = 7 * 24 * 60 * 60;
        let end_ts = start_ts + ONE_WEEK;

        let sns_module_id = env.create_canister(None);
        let wtn_ledger_id = env.create_canister(None);

        let args = SnsModuleInitArg {
            start_ts,
            end_ts,
            icp_ledger_id: icp_ledger_id.get().into(),
            wtn_ledger_id: wtn_ledger_id.get().into(),
        };
        env.install_wasm_in_mode(
            sns_module_id,
            CanisterInstallMode::Install,
            sns_module_wasm(),
            Encode!(&args).unwrap(),
        )
        .unwrap();

        env.install_wasm_in_mode(
            wtn_ledger_id,
            CanisterInstallMode::Install,
            ledger_wasm(),
            Encode!(&LedgerArgument::Init(
                LedgerInitArgsBuilder::with_symbol_and_name("WTN", "WTN")
                    .with_minting_account(minter.0)
                    .with_transfer_fee(1_000_000_u32)
                    .with_decimals(8)
                    .with_feature_flags(ic_icrc1_ledger::FeatureFlags { icrc2: true })
                    .build(),
            ))
            .unwrap(),
        )
        .unwrap();

        SnsModuleEnv {
            env,
            minter,
            user,
            sns_module_id,
            wtn_ledger_id,
            icp_ledger_id,
        }
    }

    fn icp_transfer(&self, caller: PrincipalId, to: AccountIdentifier, amount: u64) -> u64 {
        Decode!(&assert_reply(self.env.execute_ingress_as(
            caller,
            self.icp_ledger_id,
            "transfer",
            Encode!(&ICPTransferArgs {
                from_subaccount: None,
                to: to.to_address(),
                fee: Tokens::from_e8s(10_000),
                created_at_time: Some(TimeStamp::from_nanos_since_unix_epoch(self.env.get_time().as_nanos_since_unix_epoch() )),
                memo: icp_ledger::Memo(0),
                amount: Tokens::from_e8s(amount),
            }).unwrap()
            ).expect("failed to execute token transfer")),
            Result<u64, ICPTransferError>
        )
        .unwrap()
        .expect("token transfer failed")
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

    fn balance_of(&self, canister_id: CanisterId, from: impl Into<Account>) -> Nat {
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

    fn get_icp_deposited(&self, of: Principal) -> u64 {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress(
                        self.sns_module_id,
                        "get_icp_deposited",
                        Encode!(&of).unwrap()
                    )
                    .expect("failed to execute token transfer")
            ),
            u64
        )
        .unwrap()
    }

    fn get_wtn_allocated(&self, of: Principal) -> u64 {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress(
                        self.sns_module_id,
                        "get_wtn_allocated",
                        Encode!(&of).unwrap()
                    )
                    .expect("failed to execute token transfer")
            ),
            u64
        )
        .unwrap()
    }

    fn get_icp_deposit_address(&self, target: Principal) -> AccountIdentifier {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        PrincipalId::new_user_test_id(0),
                        self.sns_module_id,
                        "get_icp_deposit_address",
                        Encode!(&target).unwrap()
                    )
                    .unwrap()
            ),
            AccountIdentifier
        )
        .unwrap()
    }

    fn notify_icp_deposit(&self, target: Principal, amount: u64) -> Result<u64, String> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        PrincipalId::new_user_test_id(0),
                        self.sns_module_id,
                        "notify_icp_deposit",
                        Encode!(&target, &amount).unwrap()
                    )
                    .unwrap()
            ),
            Result<u64, String>
        )
        .unwrap()
    }

    fn return_uncommited_icp(&self, target: Principal, amount: u64) -> Result<u64, String> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        PrincipalId::new_user_test_id(0),
                        self.sns_module_id,
                        "return_uncommited_icp",
                        Encode!(&target, &amount).unwrap()
                    )
                    .unwrap()
            ),
            Result<u64, String>
        )
        .unwrap()
    }

    fn claim_wtn(&self, target: Principal) -> Result<u64, String> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        PrincipalId::new_user_test_id(0),
                        self.sns_module_id,
                        "claim_wtn",
                        Encode!(&target).unwrap()
                    )
                    .unwrap()
            ),
            Result<u64, String>
        )
        .unwrap()
    }

    fn set_is_wtn_claimable(&self, val: bool) -> Result<(), String> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        PrincipalId(Principal::from_text(DEV_WALLET).unwrap()),
                        self.sns_module_id,
                        "set_is_wtn_claimable",
                        Encode!(&val).unwrap()
                    )
                    .unwrap()
            ),
            Result<(), String>
        )
        .unwrap()
    }

    fn distribute_tokens(&self) -> Result<u64, String> {
        Decode!(
            &assert_reply(
                self.env
                    .execute_ingress_as(
                        PrincipalId(Principal::from_text(DEV_WALLET).unwrap()),
                        self.sns_module_id,
                        "distribute_tokens",
                        Encode!().unwrap()
                    )
                    .unwrap()
            ),
            Result<u64, String>
        )
        .unwrap()
    }
}

#[test]
fn e2e_basic() {
    let env = SnsModuleEnv::new();

    assert_matches!(
        env.env
            .upgrade_canister(env.sns_module_id, sns_module_wasm(), Encode!().unwrap(),),
        Ok(_)
    );

    let nns_principal =
        Principal::from_text("wwyv5-q3sgh-tae7o-v2wq7-zd32d-mv4xa-xuaup-z3r5z-vmfcg-xsm6p-xqe")
            .unwrap();
    let deposit_address = env.get_icp_deposit_address(nns_principal);

    assert_eq!(
        deposit_address,
        AccountIdentifier {
            hash: [
                62, 5, 142, 132, 57, 211, 102, 186, 74, 201, 225, 231, 72, 26, 180, 81, 56, 85, 99,
                133, 3, 205, 15, 180, 118, 32, 83, 110,
            ],
        }
    );

    assert_eq!(env.icp_transfer(env.user, deposit_address, 10_000 * E8S), 2);
    env.notify_icp_deposit(nns_principal, 10_000 * E8S).unwrap();
    assert_eq!(env.get_icp_deposited(nns_principal), 10_000 * E8S - 10_000);

    env.env.advance_time(Duration::from_secs(60));
    assert_matches!(
        env.env
            .upgrade_canister(env.sns_module_id, sns_module_wasm(), Encode!().unwrap(),),
        Ok(_)
    );

    assert_eq!(env.icp_transfer(env.user, deposit_address, 10_000 * E8S), 4);
    env.notify_icp_deposit(nns_principal, 10_000 * E8S).unwrap();
    assert_eq!(env.get_icp_deposited(nns_principal), 20_000 * E8S - 20_000);

    assert_eq!(
        env.balance_of(env.icp_ledger_id, env.sns_module_id.get().0),
        20_000 * E8S - 20_000
    );

    env.transfer(
        env.minter,
        env.sns_module_id.get().0,
        24_000_000 * E8S,
        env.wtn_ledger_id,
    );

    assert!(env.set_is_wtn_claimable(true).is_ok());

    env.env.advance_time(Duration::from_secs(7 * 24 * 60 * 60));

    assert!(env.distribute_tokens().is_ok());
    assert!(env.distribute_tokens().is_err());

    assert_eq!(env.get_wtn_allocated(nns_principal), 24_000_000 * E8S);
    assert!(env.claim_wtn(nns_principal).is_ok());
    assert!(env.claim_wtn(nns_principal).is_err());
    assert_eq!(
        env.balance_of(env.wtn_ledger_id, nns_principal),
        24_000_000 * E8S - 1_000_000
    );
    assert_eq!(env.get_wtn_allocated(nns_principal), 0);

    assert!(env.notify_icp_deposit(nns_principal, 10_000 * E8S).is_err());
}

#[test]
fn should_return_uncommited_icp() {
    let env = SnsModuleEnv::new();

    let nns_principal =
        Principal::from_text("wwyv5-q3sgh-tae7o-v2wq7-zd32d-mv4xa-xuaup-z3r5z-vmfcg-xsm6p-xqe")
            .unwrap();
    let deposit_address = env.get_icp_deposit_address(nns_principal);
    let deposit_account = Account {
        owner: env.sns_module_id.into(),
        subaccount: Some(derive_staking(nns_principal)),
    };

    let amount = 10_000 * E8S;

    assert_eq!(env.icp_transfer(env.user, deposit_address, amount), 2);
    assert_eq!(env.balance_of(env.icp_ledger_id, deposit_account), amount);

    assert_eq!(env.return_uncommited_icp(nns_principal, amount), Ok(3));
    assert_eq!(
        env.balance_of(env.icp_ledger_id, nns_principal),
        amount - 10_000
    );
    assert_eq!(env.balance_of(env.icp_ledger_id, deposit_account), 0u64);
}

#[test]
fn should_dispatch_tokens_accordingly() {
    let env = SnsModuleEnv::new();

    let nns_principal =
        Principal::from_text("wwyv5-q3sgh-tae7o-v2wq7-zd32d-mv4xa-xuaup-z3r5z-vmfcg-xsm6p-xqe")
            .unwrap();
    let deposit_address = env.get_icp_deposit_address(nns_principal);

    assert_eq!(
        deposit_address,
        AccountIdentifier {
            hash: [
                62, 5, 142, 132, 57, 211, 102, 186, 74, 201, 225, 231, 72, 26, 180, 81, 56, 85, 99,
                133, 3, 205, 15, 180, 118, 32, 83, 110,
            ],
        }
    );

    env.env.advance_time(Duration::from_secs(60));

    assert_eq!(env.icp_transfer(env.user, deposit_address, 10_000 * E8S), 2);
    env.notify_icp_deposit(nns_principal, 10_000 * E8S).unwrap();
    assert_eq!(env.get_icp_deposited(nns_principal), 10_000 * E8S - 10_000);

    env.transfer(
        env.minter,
        env.sns_module_id.get().0,
        2_600_000 * E8S,
        env.wtn_ledger_id,
    );
    assert_eq!(env.get_wtn_allocated(nns_principal), 0);

    assert!(env.set_is_wtn_claimable(true).is_ok());

    env.env.advance_time(Duration::from_secs(7 * 24 * 60 * 60));
    assert_eq!(env.distribute_tokens(), Ok(2_600_000 * E8S));
    assert_eq!(env.get_wtn_allocated(nns_principal), 2_600_000 * E8S);

    env.transfer(
        env.minter,
        env.sns_module_id.get().0,
        2_600_000 * E8S,
        env.wtn_ledger_id,
    );
    assert_eq!(env.get_wtn_allocated(nns_principal), 2_600_000 * E8S);
    assert_eq!(env.distribute_tokens(), Ok(2_600_000 * E8S));
    assert_eq!(env.get_wtn_allocated(nns_principal), 5_200_000 * E8S);

    assert_eq!(
        env.distribute_tokens(),
        Err("Nothing to distribute".to_string())
    );

    assert!(env.env.stop_canister(env.wtn_ledger_id).is_ok());
    assert_eq!(env.get_wtn_allocated(nns_principal), 5_200_000 * E8S);
    assert!(env.claim_wtn(nns_principal).is_err());
    assert_eq!(env.get_wtn_allocated(nns_principal), 5_200_000 * E8S);
    assert!(env.env.start_canister(env.wtn_ledger_id).is_ok());

    assert!(env.claim_wtn(nns_principal).is_ok());
    assert!(env.claim_wtn(nns_principal).is_err());
    assert_eq!(
        env.balance_of(env.wtn_ledger_id, nns_principal),
        5_200_000 * E8S - 1_000_000
    );
}
