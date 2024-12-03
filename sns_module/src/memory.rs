use crate::state::State;
use candid::Principal;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager as MM, VirtualMemory};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{DefaultMemoryImpl, RestrictedMemory, StableCell};
use ic_stable_structures::{DefaultMemoryImpl as DefMem, StableBTreeMap, Storable};
use std::borrow::Cow;
use std::cell::RefCell;

/// A helper type implementing Storable for all
/// serde-serializable types using the CBOR encoding.
#[derive(Default, Ord, PartialOrd, Clone, Eq, PartialEq)]
struct Cbor<T>(pub T)
where
    T: serde::Serialize + serde::de::DeserializeOwned;

impl<T> Storable for Cbor<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut buf = vec![];
        ciborium::ser::into_writer(&self.0, &mut buf).unwrap();
        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Self(ciborium::de::from_reader(bytes.as_ref()).unwrap())
    }

    const BOUND: Bound = Bound::Unbounded;
}

// NOTE: ensure that all memory ids are unique and
// do not change across upgrades!
const PRINCIPAL_TO_ICP_ID: MemoryId = MemoryId::new(0);
const PRINCIPAL_TO_WTN_ID: MemoryId = MemoryId::new(1);

const METADATA_PAGES: u64 = 16;

type VM = VirtualMemory<DefMem>;
type RM = RestrictedMemory<DefMem>;

thread_local! {
    static METADATA: RefCell<StableCell<Cbor<Option<State>>, RM>> =
        RefCell::new(StableCell::init(
            RM::new(DefMem::default(), 0..METADATA_PAGES),
            Cbor::default(),
        ).expect("failed to initialize the metadata cell")
    );

    static MEMORY_MANAGER: RefCell<MM<DefaultMemoryImpl>> = RefCell::new(
        MM::init(DefaultMemoryImpl::default())
    );

    static PRINCIPAL_TO_ICP: RefCell<StableBTreeMap<Principal, u64, VM>> =
        MEMORY_MANAGER.with(|mm| {
        RefCell::new(StableBTreeMap::init(mm.borrow().get(PRINCIPAL_TO_ICP_ID)))
    });

    static PRINCIPAL_TO_WTN: RefCell<StableBTreeMap<Principal, u64, VM>> =
        MEMORY_MANAGER.with(|mm| {
        RefCell::new(StableBTreeMap::init(mm.borrow().get(PRINCIPAL_TO_WTN_ID)))
    });
}

pub fn deposit_icp(to: Principal, tokens: u64) {
    PRINCIPAL_TO_ICP.with(|m| {
        let current_balance = m.borrow().get(&to).unwrap_or(0);
        let new_balance = current_balance.checked_add(tokens).unwrap();
        m.borrow_mut().insert(to, new_balance);
    });
}

pub fn get_icp_deposited(of: Principal) -> u64 {
    PRINCIPAL_TO_ICP.with(|m| m.borrow().get(&of).unwrap_or(0))
}

pub fn get_principal_to_icp() -> Vec<(Principal, u64)> {
    PRINCIPAL_TO_ICP.with(|m| m.borrow().iter().collect())
}

pub fn set_wtn_owed(to: Principal, tokens: u64) {
    PRINCIPAL_TO_WTN.with(|m| {
        m.borrow_mut().insert(to, tokens);
    });
}

pub fn get_wtn_owed(of: Principal) -> u64 {
    PRINCIPAL_TO_WTN.with(|m| m.borrow().get(&of).unwrap_or(0))
}

pub fn get_principal_to_wtn_owed() -> Vec<(Principal, u64)> {
    PRINCIPAL_TO_WTN.with(|m| m.borrow().iter().collect())
}

pub fn get_state() -> State {
    METADATA
        .with(|m| m.borrow().get().0.clone())
        .expect("state not init")
}

pub fn set_state(state: State) {
    METADATA.with(|m| {
        m.borrow_mut()
            .set(Cbor(Some(state)))
            .expect("failed to set metadata")
    });
}

#[test]
fn should_add_tokens() {
    let p1 = Principal::anonymous();
    deposit_icp(p1, 100);
    assert_eq!(get_icp_deposited(p1), 100);
    deposit_icp(p1, 200);
    assert_eq!(get_icp_deposited(p1), 300);
}

#[test]
fn should_set_wtn() {
    let p1 = Principal::anonymous();
    set_wtn_owed(p1, 100);
    assert_eq!(get_wtn_owed(p1), 100);
    set_wtn_owed(p1, 200);
    assert_eq!(get_wtn_owed(p1), 200);
}
