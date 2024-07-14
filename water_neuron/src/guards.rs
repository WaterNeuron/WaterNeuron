use crate::state::mutate_state;
use crate::tasks::TaskType;
use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use std::marker::PhantomData;

const MAX_CONCURRENT: usize = 100;

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
        mutate_state(|s| {
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
        mutate_state(|s| s.principal_guards.remove(&self.principal));
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TaskGuardError {
    AlreadyProcessing,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TaskGuard {
    task: TaskType,
}

impl TaskGuard {
    pub fn new(task: TaskType) -> Result<Self, TaskGuardError> {
        mutate_state(|s| {
            if !s.active_tasks.insert(task) {
                return Err(TaskGuardError::AlreadyProcessing);
            }
            Ok(Self { task })
        })
    }
}

impl Drop for TaskGuard {
    fn drop(&mut self) {
        mutate_state(|s| {
            s.active_tasks.remove(&self.task);
        });
    }
}

#[test]
fn should_accept_and_reject_duplicate_task() {
    use crate::state::replace_state;
    use crate::state::test::default_state;

    let state = default_state();
    replace_state(state);

    let task_type = TaskType::VoteOnProposal { id: 2, vote: true };
    let _guard = match TaskGuard::new(task_type) {
        Ok(guard) => guard,
        Err(_) => {
            panic!();
        }
    };

    let _guard = match TaskGuard::new(task_type) {
        Ok(_guard) => panic!(),
        Err(_) => {}
    };

    let task_type = TaskType::VoteOnProposal { id: 3, vote: true };
    let _guard = match TaskGuard::new(task_type) {
        Ok(guard) => guard,
        Err(_) => {
            panic!();
        }
    };
}
