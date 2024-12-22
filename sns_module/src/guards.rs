use crate::state::mutate_state;
use crate::tasks::TaskType;
use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use std::marker::PhantomData;

const MAX_CONCURRENT: usize = 100;

thread_local! {
    static __GUARDS: RefCell<Guards> = RefCell::default();
}

#[derive(CandidType, Serialize, Deserialize, Clone)]
pub struct Guards {
    pub principal_guards: u64,
}

pub fn mutate_guards<F, R>(f: F) -> R
where
    F: FnOnce(&mut State) -> R,
{
    __GUARDS.with(|s| f(s.borrow_mut().as_mut().expect("State not initialized!")))
}

/// Guards a block from executing twice when called by the same user and from being
/// executed [MAX_CONCURRENT] or more times in parallel.
#[must_use]
pub struct GuardPrincipal {
    principal: Principal,
    _marker: PhantomData<GuardPrincipal>,
}

#[derive(Debug, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum GuardError {
    AlreadyProcessing,
    TooManyConcurrentRequests,
}

impl GuardPrincipal {
    /// Attempts to create a new guard for the current block. Fails if there is
    /// already a pending request for the specified [principal] or if there
    /// are at least [MAX_CONCURRENT] pending requests.
    pub fn new(principal: Principal) -> Result<Self, GuardError> {
        mutate_guards(|s| {
            if s.principal_guards.contains(&principal) {
                return Err(GuardError::AlreadyProcessing);
            }
            if s.principal_guards.len() >= MAX_CONCURRENT {
                return Err(GuardError::TooManyConcurrentRequests);
            }
            s.principal_guards.insert(principal);
            Ok(Self {
                principal,
                _marker: PhantomData,
            })
        })
    }
}

impl Drop for GuardPrincipal {
    fn drop(&mut self) {
        mutate_guards(|s| s.principal_guards.remove(&self.principal));
    }
}