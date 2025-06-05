use crate::state::event::{Event, EventType};
use candid::Principal;
use ic_stable_structures::{
    log::Log as StableLog,
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    storable::{Bound, Storable},
    DefaultMemoryImpl, StableBTreeMap,
};
use std::borrow::Cow;
use std::cell::RefCell;

const LOG_INDEX_MEMORY_ID: MemoryId = MemoryId::new(0);
const LOG_DATA_MEMORY_ID: MemoryId = MemoryId::new(1);
const PRINCIPAL_TO_ICP_REWARDS_ID: MemoryId = MemoryId::new(2);

type VMem = VirtualMemory<DefaultMemoryImpl>;
type EventLog = StableLog<Event, VMem, VMem>;

impl Storable for Event {
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut buf = vec![];
        minicbor::encode(self, &mut buf).expect("event encoding should always succeed");
        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        minicbor::decode(bytes.as_ref())
            .unwrap_or_else(|e| panic!("failed to decode event bytes {}: {e}", hex::encode(bytes)))
    }

    const BOUND: Bound = Bound::Unbounded;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static EVENTS: RefCell<EventLog> = MEMORY_MANAGER
        .with(|m|
              RefCell::new(
                  StableLog::init(
                      m.borrow().get(LOG_INDEX_MEMORY_ID),
                      m.borrow().get(LOG_DATA_MEMORY_ID)
                  ).expect("failed to initialize stable log")
              )
        );

    static PRINCIPAL_TO_ICP_REWARDS: RefCell<StableBTreeMap<Principal, u64, VMem>> =
        MEMORY_MANAGER.with(|mm| {
        RefCell::new(StableBTreeMap::init(mm.borrow().get(PRINCIPAL_TO_ICP_REWARDS_ID)))
    });
}

/// Appends the event to the event log.
pub fn record_event(payload: EventType, timestamp: u64) {
    EVENTS
        .with(|events| events.borrow().append(&Event { timestamp, payload }))
        .expect("recording an event should succeed");
}

/// Returns the total number of events in the audit log.
pub fn total_event_count() -> u64 {
    EVENTS.with(|events| events.borrow().len())
}

pub fn with_event_iter<F, R>(f: F) -> R
where
    F: for<'a> FnOnce(Box<dyn Iterator<Item = Event> + 'a>) -> R,
{
    EVENTS.with(|events| f(Box::new(events.borrow().iter())))
}

pub fn stable_add_rewards(to: Principal, amount_e8s: u64) {
    PRINCIPAL_TO_ICP_REWARDS.with(|p| {
        let balance = p.borrow().get(&to).unwrap_or(0);
        let new_balance = balance.checked_add(amount_e8s).unwrap();
        p.borrow_mut().insert(to, new_balance);
    });
}

pub fn stable_sub_rewards(to: Principal, amount_e8s: u64) {
    PRINCIPAL_TO_ICP_REWARDS.with(|p| {
        let balance = p.borrow().get(&to).unwrap_or(0);
        let new_balance = balance.checked_sub(amount_e8s).unwrap();
        if new_balance == 0 {
            assert!(p.borrow_mut().remove(&to).is_some());
        } else {
            assert!(p.borrow_mut().insert(to, new_balance).is_some());
        }
    });
}

pub fn get_rewards_ready_to_be_distributed(length: usize) -> Vec<(Principal, u64)> {
    PRINCIPAL_TO_ICP_REWARDS.with(|p| {
        const MINIMUM_ICP_AMOUNT_DISTRIBUTION: u64 = 1_000_000;
        let mut result: Vec<(Principal, u64)> = vec![];
        for (p, b) in p.borrow().iter() {
            if b > MINIMUM_ICP_AMOUNT_DISTRIBUTION {
                result.push((p, b));
            }
            if result.len() >= length {
                break;
            }
        }
        result
    })
}

pub fn get_pending_rewards(to: Principal) -> Option<u64> {
    PRINCIPAL_TO_ICP_REWARDS.with(|p| p.borrow().get(&to))
}

pub fn total_pending_rewards() -> u64 {
    PRINCIPAL_TO_ICP_REWARDS.with(|p| p.borrow().values().sum())
}

#[test]
fn should_do_operation_on_rewards() {
    let caller = Principal::anonymous();
    stable_add_rewards(caller, 100_000_000);
    assert_eq!(get_pending_rewards(caller), Some(100_000_000));
    stable_sub_rewards(caller, 50_000_000);
    assert_eq!(get_pending_rewards(caller), Some(50_000_000));
    stable_sub_rewards(caller, 50_000_000);
    assert_eq!(get_pending_rewards(caller), None);
}

#[test]
fn should_return_rewards_ready() {
    let caller = Principal::anonymous();
    stable_add_rewards(caller, 10_000_000_000);
    stable_add_rewards(Principal::management_canister(), 1_000);
    assert_eq!(
        get_rewards_ready_to_be_distributed(10),
        vec![(caller, 10_000_000_000)]
    );
}
