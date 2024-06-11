use crate::management::{get_sns_proposal, make_sns_proposal};
use crate::nns_types::convert_nns_proposal_to_sns_proposal;
use crate::{
    compute_neuron_staking_subaccount_bytes, get_pending_proposals, mutate_state, process_event,
    read_state, schedule_after, self_canister_id, timestamp_nanos, EventType, TaskType, INFO,
    ONE_DAY, ONE_HOUR_SECONDS, RETRY_DELAY, SEC_NANOS,
};
use ic_canister_log::log;
use ic_sns_governance::pb::v1::manage_neuron_response::Command as CommandSnsResponse;
use std::time::Duration;

pub async fn mirror_proposals() {
    if read_state(|s| s.sns_governance_id.is_none()) {
        return;
    }

    let subaccount = compute_neuron_staking_subaccount_bytes(self_canister_id(), 0).to_vec();

    match get_pending_proposals().await {
        Ok(mut pending_proposals) => {
            read_state(|s| {
                pending_proposals.retain(|p| !s.proposals.contains_key(&p.id.clone().unwrap()))
            });
            log!(
                INFO,
                "[mirror_proposals] found {} new pending proposals",
                pending_proposals.len()
            );
            for proposal_info in pending_proposals {
                let proposal_id = match proposal_info.id.clone() {
                    Some(proposal_id) => proposal_id,
                    None => {
                        log!(INFO, "[mirror_proposals] bug: found a proposal without id",);
                        continue;
                    }
                };
                if read_state(|s| s.proposals.contains_key(&proposal_id)) {
                    // We already processed this proposal.
                    continue;
                }
                let sns_proposal = convert_nns_proposal_to_sns_proposal(&proposal_info);
                match make_sns_proposal(subaccount.clone(), sns_proposal).await {
                    Ok(manage_neuron_response) => {
                        if let Some(CommandSnsResponse::MakeProposal(make_proposal_response)) =
                            manage_neuron_response.command.clone()
                        {
                            if let Some(sns_proposal_id) = make_proposal_response.proposal_id {
                                mutate_state(|s| {
                                    process_event(
                                        s,
                                        EventType::MirroredProposal {
                                            nns_proposal_id: proposal_id.clone(),
                                            sns_proposal_id: crate::ProposalId {
                                                id: sns_proposal_id.id,
                                            },
                                        },
                                    )
                                });
                            }
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
        Err(error) => {
            log!(
                INFO,
                "[mirror_proposals] Failed to get pending proposals with error: {error}"
            );
            schedule_after(RETRY_DELAY, TaskType::ScheduleVoting);
        }
    }
    schedule_after(ONE_DAY, TaskType::ScheduleVoting);
}

pub async fn schedule_voting_with_sns() {
    let sns_gov_id = read_state(|s| s.sns_governance_id.unwrap());

    match get_pending_proposals().await {
        Ok(pending_proposals) => {
            log!(
                INFO,
                "[schedule_voting] found {} pending proposals",
                pending_proposals.len()
            );
            for proposal in pending_proposals {
                let deadline_timestamp_seconds = proposal.deadline_timestamp_seconds.unwrap_or(0);
                let proposal_id = match proposal.id {
                    Some(proposal_id) => proposal_id,
                    None => continue,
                };
                let diff_secs = deadline_timestamp_seconds
                    .saturating_sub(timestamp_nanos() / SEC_NANOS)
                    .saturating_sub(ONE_HOUR_SECONDS);

                if deadline_timestamp_seconds - ONE_HOUR_SECONDS > timestamp_nanos() / SEC_NANOS {
                    if let Some(sns_proposal_id) =
                        read_state(|s| s.proposals.get(&proposal_id).cloned())
                    {
                        match get_sns_proposal(sns_gov_id, sns_proposal_id.id).await {
                            Ok(proposal_response) => {
                                if let Some(ic_sns_governance::pb::v1::get_proposal_response::Result::Proposal(proposal_data)) =
                                    proposal_response.result
                                {
                                    if let Some(tally) = proposal_data.latest_tally {
                                        let vote_outcome = tally.yes > tally.no;
                                        schedule_after(
                                            Duration::from_secs(diff_secs),
                                            TaskType::VoteOnProposal { id: proposal_id.id, vote: vote_outcome },
                                        );
                                        continue;
                                    }
                                }
                            }
                            Err(e) => log!(
                                INFO,
                                "[schedule_voting] Failed to fetch SNS proposal with error: {e}"
                            ),
                        }
                    }
                    // We didn't manage to fetch the SNS proposal's outcome
                    // we vote not by default.
                    schedule_after(
                        Duration::from_secs(diff_secs),
                        TaskType::VoteOnProposal {
                            id: proposal_id.id,
                            vote: false,
                        },
                    );
                }
            }
        }
        Err(error) => {
            log!(
                INFO,
                "[schedule_voting] Failed to get pending proposals with error: {error}"
            );
            schedule_after(RETRY_DELAY, TaskType::ScheduleVoting);
        }
    }
}

pub async fn schedule_default_voting() {
    match get_pending_proposals().await {
        Ok(pending_proposals) => {
            log!(
                INFO,
                "[ScheduleVoting] found {} pending proposals",
                pending_proposals.len()
            );
            for proposal in pending_proposals {
                let deadline_timestamp_seconds = proposal.deadline_timestamp_seconds.unwrap_or(0);
                let proposal_id = match proposal.id {
                    Some(proposal_id) => proposal_id,
                    None => continue,
                };
                let diff_secs = deadline_timestamp_seconds
                    .saturating_sub(timestamp_nanos() / SEC_NANOS)
                    .saturating_sub(ONE_HOUR_SECONDS);
                log!(
                    INFO,
                    "[ScheduleVoting] Reject proposal with id: {}",
                    proposal_id.id
                );

                schedule_after(
                    Duration::from_secs(diff_secs),
                    TaskType::VoteOnProposal {
                        id: proposal_id.id,
                        vote: false,
                    },
                )
            }
        }
        Err(error) => {
            log!(
                INFO,
                "[ScheduleVoting] Failed to get pending proposals with error: {error}"
            );
            schedule_after(RETRY_DELAY, TaskType::ScheduleVoting);
        }
    }
}
