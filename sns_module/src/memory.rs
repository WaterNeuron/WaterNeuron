use crate::state::State;
use candid::Principal;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager as MM, VirtualMemory};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{
    DefaultMemoryImpl as DefMem, RestrictedMemory, StableBTreeMap, StableCell, Storable,
};
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
const IN_FLIGHT_WTN_ID: MemoryId = MemoryId::new(2);

const METADATA_PAGES: u64 = 16;

type RM = RestrictedMemory<DefMem>;
type VM = VirtualMemory<RM>;

const WASM_PAGE_SIZE: u64 = 65536;

/// The maximum number of stable memory pages a canister can address.
const MAX_PAGES: u64 = u64::MAX / WASM_PAGE_SIZE;

thread_local! {
    static METADATA: RefCell<StableCell<Cbor<Option<State>>, RM>> =
        RefCell::new(StableCell::init(
            RM::new(DefMem::default(), 0..METADATA_PAGES),
            Cbor::default(),
        ).expect("failed to initialize the metadata cell")
    );

    static MEMORY_MANAGER: RefCell<MM<RM>> = RefCell::new(
        MM::init(RM::new(DefMem::default(), METADATA_PAGES..MAX_PAGES))
      );

    static PRINCIPAL_TO_ICP: RefCell<StableBTreeMap<Principal, u64, VM>> =
        MEMORY_MANAGER.with(|mm| {
        RefCell::new(StableBTreeMap::init(mm.borrow().get(PRINCIPAL_TO_ICP_ID)))
    });

    static PRINCIPAL_TO_WTN: RefCell<StableBTreeMap<Principal, u64, VM>> =
        MEMORY_MANAGER.with(|mm| {
        RefCell::new(StableBTreeMap::init(mm.borrow().get(PRINCIPAL_TO_WTN_ID)))
    });

    static IN_FLIGHT_WTN: RefCell<StableCell<u64, VM>> =
        MEMORY_MANAGER.with(|mm| {
        RefCell::new(StableCell::init(mm.borrow().get(IN_FLIGHT_WTN_ID), 0_u64).expect("failed to initialize the cell"))
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

pub fn add_wtn_owed(to: Principal, tokens: u64) {
    PRINCIPAL_TO_WTN.with(|m| {
        let current_balance = m.borrow().get(&to).unwrap_or(0);
        let new_balance = current_balance.checked_add(tokens).unwrap();
        m.borrow_mut().insert(to, new_balance);
    });
}

pub fn decrease_wtn_owed(to: Principal, tokens: u64) {
    PRINCIPAL_TO_WTN.with(|m| {
        let current_balance = m.borrow().get(&to).unwrap_or(0);
        let new_balance = current_balance.checked_sub(tokens).unwrap();
        m.borrow_mut().insert(to, new_balance);
    });
}

pub fn get_wtn_owed(of: Principal) -> u64 {
    PRINCIPAL_TO_WTN.with(|m| m.borrow().get(&of).unwrap_or(0))
}

pub fn get_principal_to_wtn_owed() -> Vec<(Principal, u64)> {
    PRINCIPAL_TO_WTN.with(|m| m.borrow().iter().collect())
}

pub fn total_wtn_allocated() -> u64 {
    PRINCIPAL_TO_WTN.with(|m| m.borrow().iter().map(|(_, v)| v).sum())
}

pub fn get_state() -> Option<State> {
    METADATA.with(|m| m.borrow().get().0.clone())
}

pub fn set_state(state: State) {
    METADATA.with(|m| {
        m.borrow_mut()
            .set(Cbor(Some(state)))
            .expect("failed to set metadata")
    });
}

pub fn add_in_flight_wtn(amount: u64) {
    IN_FLIGHT_WTN.with(|b| {
        let balance = *b.borrow().get();
        let new_balance = balance.checked_add(amount).unwrap();
        b.borrow_mut().set(new_balance).unwrap();
    });
}

pub fn remove_in_flight_wtn(amount: u64) {
    IN_FLIGHT_WTN.with(|b| {
        let balance = *b.borrow().get();
        let new_balance = balance.checked_sub(amount).unwrap();
        b.borrow_mut().set(new_balance).unwrap();
    });
}

pub fn get_in_flight_wtn() -> u64 {
    IN_FLIGHT_WTN.with(|m| *m.borrow().get())
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
fn should_add_wtn() {
    let p1 = Principal::anonymous();
    add_wtn_owed(p1, 100);
    assert_eq!(get_wtn_owed(p1), 100);
    add_wtn_owed(p1, 200);
    assert_eq!(get_wtn_owed(p1), 300);
    decrease_wtn_owed(p1, 300);
    assert_eq!(get_wtn_owed(p1), 0);
}

#[test]
fn should_apply_algebra() {
    assert_eq!(get_in_flight_wtn(), 0);
    add_in_flight_wtn(100);
    assert_eq!(get_in_flight_wtn(), 100);
    add_in_flight_wtn(200);
    assert_eq!(get_in_flight_wtn(), 300);
    remove_in_flight_wtn(100);
    assert_eq!(get_in_flight_wtn(), 200);
    remove_in_flight_wtn(200);
    assert_eq!(get_in_flight_wtn(), 00);
}
