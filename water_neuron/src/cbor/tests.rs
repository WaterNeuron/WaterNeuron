use candid::Principal;
use icrc_ledger_types::icrc1::account::Account;
use minicbor::{Decode, Encode};
use proptest::array::uniform32;
use proptest::collection::vec as pvec;
use proptest::prelude::*;

pub fn check_roundtrip<T>(v: &T) -> Result<(), TestCaseError>
where
    for<'a> T: PartialEq + std::fmt::Debug + Encode<()> + Decode<'a, ()>,
{
    let mut buf = vec![];
    minicbor::encode(v, &mut buf).expect("encoding should succeed");
    let decoded = minicbor::decode(&buf).expect("decoding should succeed");
    prop_assert_eq!(v, &decoded);
    Ok(())
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
struct PrincipalContainer {
    #[cbor(n(0), with = "crate::cbor::principal")]
    pub value: Principal,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
struct AccountContainer {
    #[cbor(n(0), with = "crate::cbor::account")]
    pub value: Account,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
struct OptPrincipalContainer {
    #[cbor(n(0), with = "crate::cbor::principal::option")]
    pub value: Option<Principal>,
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

proptest! {
    #[test]
    fn principal_encoding_roundtrip(p in pvec(any::<u8>(), 0..30)) {
        check_roundtrip(&PrincipalContainer {
            value: Principal::from_slice(&p),
        })?;
    }

    #[test]
    fn opt_principal_encoding_roundtrip(p in proptest::option::of(pvec(any::<u8>(), 0..30))) {
        check_roundtrip(&OptPrincipalContainer {
            value: p.map(|principal| Principal::from_slice(&principal)),
        })?;
    }

    #[test]
    fn account_encoding_roundtrip(
        account in arb_account()) {
        check_roundtrip(&AccountContainer {
            value: account
        })?;
    }
}
