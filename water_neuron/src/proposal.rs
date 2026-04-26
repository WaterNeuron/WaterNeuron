use crate::management::{get_sns_proposal, list_proposals, manage_neuron_sns};
use crate::nns_types::{ProposalId, convert_nns_proposal_to_sns_proposal};
use crate::{
    DEBUG, EventType, INFO, ONE_HOUR_SECONDS, RETRY_DELAY_VOTING, SEC_NANOS, TaskType,
    compute_neuron_staking_subaccount_bytes, is_canister_stopping, mutate_state, process_event,
    read_state, register_vote, schedule_after, self_canister_id, timestamp_nanos,
};
use ic_canister_log::log;
use ic_nns_governance_api::{ListProposalInfoRequest, ProposalInfo};
use ic_sns_governance_api::pb::v1::{
    manage_neuron::Command as CommandSns, manage_neuron_response::Command as CommandSnsResponse,
};
use std::time::Duration;

const BATCH_SIZE_LIMIT: u32 = 100;
const MAX_PAGES: u32 = 5;
const REWARD_STATUS_ACCEPT_VOTES: i32 = 1;
const REWARD_STATUS_READY_TO_SETTLE: i32 = 2;

/// Paginates `list_proposals` until exhausted or MAX_PAGES, whichever comes first.
/// Returns every pending proposal (ACCEPT_VOTES or READY_TO_SETTLE) in one pass
/// so callers can share the result instead of each calling NNS separately.
async fn fetch_pending_proposals() -> Result<Vec<ProposalInfo>, String> {
    let mut all: Vec<ProposalInfo> = Vec::new();
    let mut before_proposal: Option<ic_nns_common::pb::v1::ProposalId> = None;

    for page in 0..MAX_PAGES {
        let args = ListProposalInfoRequest {
            limit: BATCH_SIZE_LIMIT,
            before_proposal,
            exclude_topic: vec![],
            include_reward_status: vec![REWARD_STATUS_ACCEPT_VOTES, REWARD_STATUS_READY_TO_SETTLE],
            omit_large_fields: Some(true),
            ..Default::default()
        };

        let response = list_proposals(args)
            .await
            .map_err(|e| format!("Failed to get pending proposals with error: {e}"))?;

        let batch = response.proposal_info;
        let batch_len = batch.len();
        if batch_len == 0 {
            break;
        }

        let smallest_id = batch
            .iter()
            .filter_map(|p| p.id.as_ref().map(|id| id.id))
            .min();

        all.extend(batch);

        // Partial page means we've reached the end; don't spend another call.
        if batch_len < BATCH_SIZE_LIMIT as usize {
            break;
        }

        match smallest_id {
            Some(id) => before_proposal = Some(ic_nns_common::pb::v1::ProposalId { id }),
            None => break,
        }

        if page + 1 == MAX_PAGES {
            log!(
                INFO,
                "[fetch_pending_proposals] hit MAX_PAGES ({MAX_PAGES}); some proposals may not be seen this tick"
            );
        }
    }

    Ok(all)
}

/// Returns the suggested delay until the next voting cycle. The caller should
/// schedule a follow-up `ProcessVoting` tick after this duration so that we only
/// poll NNS as often as the nearest upcoming vote window requires.
pub async fn process_voting_cycle() -> Result<Duration, String> {
    let pending = fetch_pending_proposals().await?;
    log!(
        DEBUG,
        "[process_voting_cycle] fetched {} pending proposals",
        pending.len()
    );
    mirror_proposals(&pending).await;
    vote_on_nns_proposals(&pending).await;
    Ok(compute_next_tick_delay(&pending))
}

/// Quiet-period ceiling: even if the next vote window is far away, we still
/// refresh the mirror this often so newly-created short-deadline NNS proposals
/// (and any we dropped on a failed mirror) are discovered.
const MAX_IDLE_DELAY: Duration = Duration::from_secs(30 * 60);
/// Lower bound so we never busy-loop if a deadline is imminent.
const MIN_IDLE_DELAY: Duration = Duration::from_secs(60);

fn compute_next_tick_delay(pending: &[ProposalInfo]) -> Duration {
    let now = timestamp_nanos() / SEC_NANOS;

    let earliest_vote_start = read_state(|s| {
        pending
            .iter()
            .filter_map(|p| {
                let id = p.id.as_ref()?.id;
                let deadline = p.deadline_timestamp_seconds?;
                if s.voted_proposals.contains(&ProposalId { id }) {
                    return None;
                }
                // `deadline - 1h` is when we must be awake to vote; anything
                // already past that has just been voted on this tick.
                deadline.checked_sub(ONE_HOUR_SECONDS).filter(|t| *t > now)
            })
            .min()
    });

    match earliest_vote_start {
        Some(t) => Duration::from_secs(t - now).clamp(MIN_IDLE_DELAY, MAX_IDLE_DELAY),
        None => MAX_IDLE_DELAY,
    }
}

async fn mirror_proposals(pending: &[ProposalInfo]) {
    let subaccount = compute_neuron_staking_subaccount_bytes(self_canister_id(), 0).to_vec();

    let mut to_mirror: Vec<&ProposalInfo> = read_state(|s| {
        pending
            .iter()
            .filter(|p| {
                p.reward_status == REWARD_STATUS_ACCEPT_VOTES
                    && p.id
                        .is_some_and(|id| !s.proposals.contains_key(&ProposalId { id: id.id }))
            })
            .collect()
    });
    to_mirror.sort_by(|a, b| {
        a.deadline_timestamp_seconds
            .cmp(&b.deadline_timestamp_seconds)
    });

    log!(
        DEBUG,
        "[mirror_proposals] {} new proposals to mirror",
        to_mirror.len()
    );

    for proposal_info in to_mirror {
        if is_canister_stopping() {
            log!(INFO, "[mirror_proposals] Canister is stopping, aborting.");
            return;
        }
        let proposal_id = match proposal_info.id {
            Some(proposal_id) => proposal_id,
            None => continue,
        };

        let sns_proposal = match convert_nns_proposal_to_sns_proposal(proposal_info) {
            Some(sns_proposal) => sns_proposal,
            None => {
                log!(
                    INFO,
                    "[mirror_proposals] failed to convert {proposal_info:?}"
                );
                continue;
            }
        };
        match manage_neuron_sns(subaccount.clone(), CommandSns::MakeProposal(sns_proposal)).await {
            Ok(manage_neuron_response) => {
                if let Some(CommandSnsResponse::MakeProposal(make_proposal_response)) =
                    manage_neuron_response.command.clone()
                    && let Some(sns_proposal_id) = make_proposal_response.proposal_id
                {
                    mutate_state(|s| {
                        process_event(
                            s,
                            EventType::MirroredProposal {
                                nns_proposal_id: ProposalId { id: proposal_id.id },
                                sns_proposal_id: crate::ProposalId {
                                    id: sns_proposal_id.id,
                                },
                            },
                        );
                    });
                    continue;
                }
                log!(
                    INFO,
                    "[mirror_proposals] unexpected response: {:?}",
                    manage_neuron_response
                );
            }
            Err(e) => {
                log!(
                    INFO,
                    "[mirror_proposals] failed to make a proposal with error: {}",
                    e
                );
            }
        }
    }
}

async fn vote_on_nns_proposals(pending: &[ProposalInfo]) {
    let wtn_governance_id = read_state(|s| s.wtn_governance_id);
    let now_secs = timestamp_nanos() / SEC_NANOS;

    let mut near_deadline: Vec<&ProposalInfo> = pending
        .iter()
        .filter(|p| {
            p.deadline_timestamp_seconds
                .unwrap_or(u64::MAX)
                .saturating_sub(now_secs)
                <= ONE_HOUR_SECONDS
        })
        .collect();
    near_deadline.sort_by(|a, b| {
        a.deadline_timestamp_seconds
            .unwrap_or(u64::MAX)
            .cmp(&b.deadline_timestamp_seconds.unwrap_or(u64::MAX))
    });

    for proposal in near_deadline {
        if is_canister_stopping() {
            log!(
                INFO,
                "[vote_on_nns_proposals] Canister is stopping, aborting."
            );
            return;
        }
        let proposal_id = match proposal.id {
            Some(proposal_id) => ProposalId { id: proposal_id.id },
            None => continue,
        };
        if read_state(|s| s.voted_proposals.contains(&proposal_id)) {
            continue;
        }
        if let Some(sns_proposal_id) = read_state(|s| s.proposals.get(&proposal_id).cloned()) {
            match get_sns_proposal(wtn_governance_id, sns_proposal_id.id).await {
                Ok(proposal_response) => {
                    if let Some(
                        ic_sns_governance_api::pb::v1::get_proposal_response::Result::Proposal(
                            proposal_data,
                        ),
                    ) = proposal_response.result.clone()
                        && let Some(tally) = proposal_data.latest_tally
                    {
                        let vote_outcome = tally.yes > tally.no;
                        vote_on_proposal(proposal_id, vote_outcome).await;
                        continue;
                    }
                    log!(
                        INFO,
                        "[vote_on_nns_proposals] Failed to fetch SNS proposal, got: {proposal_response:?}"
                    );
                }
                Err(e) => log!(
                    INFO,
                    "[vote_on_nns_proposals] Failed to fetch SNS proposal with error: {e}"
                ),
            }
        }
        // We didn't manage to fetch the SNS proposal's outcome; vote NO by default.
        vote_on_proposal(proposal_id, false).await;
    }
}

async fn vote_on_proposal(proposal_id: ProposalId, vote: bool) {
    if read_state(|s| s.voted_proposals.contains(&proposal_id)) {
        log!(
            DEBUG,
            "[VoteOnProposal] Already voted {vote} on proposal {}",
            proposal_id.id
        );
        return;
    }

    let neuron_6m = match read_state(|s| s.neuron_id_6m) {
        Some(neuron_6m_id) => neuron_6m_id,
        None => {
            log!(INFO, "[VoteOnProposal] 6 months neuron not set",);
            return;
        }
    };

    match register_vote(neuron_6m, proposal_id.clone(), vote).await {
        Ok(response) => {
            log!(
                INFO,
                "[VoteOnProposal] Successfully voted {vote} on proposal {} with response {response:?}",
                proposal_id.id
            );
            mutate_state(|s| s.voted_proposals.insert(proposal_id));
        }
        Err(error) => {
            log!(
                INFO,
                "[VoteOnProposal] Failed to vote on proposal {} with error: {error}",
                proposal_id.id
            );
            schedule_after(RETRY_DELAY_VOTING, TaskType::ProcessVoting);
        }
    }
}

pub async fn early_voting_on_nns_proposals() {
    let wtn_governance_id = read_state(|s| s.wtn_governance_id);

    let not_voted: Vec<_> = read_state(|s| {
        s.proposals
            .iter()
            .filter(|(k, _)| !s.voted_proposals.contains(k))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    });

    for (proposal_id, sns_proposal_id) in not_voted {
        if is_canister_stopping() {
            log!(
                INFO,
                "[early_voting_on_nns_proposals] Canister is stopping, aborting."
            );
            return;
        }
        match get_sns_proposal(wtn_governance_id, sns_proposal_id.id).await {
            Ok(proposal_response) => {
                if let Some(ic_sns_governance_api::pb::v1::get_proposal_response::Result::Proposal(
                    proposal_data,
                )) = proposal_response.result.clone()
                    && let Some(tally) = proposal_data.latest_tally
                {
                    if tally.no > tally.total / 2 {
                        vote_on_proposal(proposal_id.clone(), false).await;
                    }
                    if tally.yes > tally.total / 2 {
                        vote_on_proposal(proposal_id.clone(), true).await;
                    }
                }
            }
            Err(e) => log!(
                INFO,
                "[early_voting_on_nns_proposals] Failed to fetch SNS proposal with error: {e}"
            ),
        }
    }
}
