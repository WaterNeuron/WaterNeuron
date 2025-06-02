use crate::management::{get_sns_proposal, list_proposals, manage_neuron_sns};
use crate::nns_types::{convert_nns_proposal_to_sns_proposal, ProposalId};
use crate::{
    compute_neuron_staking_subaccount_bytes, mutate_state, process_event, read_state,
    register_vote, schedule_after, self_canister_id, timestamp_nanos, EventType, TaskType, INFO,
    ONE_HOUR_SECONDS, RETRY_DELAY_VOTING, SEC_NANOS,
};
use ic_canister_log::log;
use ic_nns_governance_api::ListProposalInfo;
use ic_sns_governance_api::pb::v1::{
    manage_neuron::Command as CommandSns, manage_neuron_response::Command as CommandSnsResponse,
};

const BATCH_SIZE_LIMIT: u32 = 100;
const REWARD_STATUS_ACCEPT_VOTES: i32 = 1;
const REWARD_STATUS_READY_TO_SETTLE: i32 = 2;

pub async fn mirror_proposals() -> Result<(), String> {
    let subaccount = compute_neuron_staking_subaccount_bytes(self_canister_id(), 0).to_vec();

    let list_proposals_args = ListProposalInfo {
        limit: BATCH_SIZE_LIMIT,
        before_proposal: None,
        exclude_topic: vec![],
        include_reward_status: vec![REWARD_STATUS_ACCEPT_VOTES, REWARD_STATUS_READY_TO_SETTLE],
        omit_large_fields: Some(true),
        ..Default::default()
    };

    match list_proposals(list_proposals_args).await {
        Ok(mut pending_proposals) => {
            read_state(|s| {
                pending_proposals.proposal_info.retain(|p| {
                    !s.proposals.contains_key(&ProposalId {
                        id: p.id.unwrap().id,
                    })
                })
            });
            log!(
                INFO,
                "[mirror_proposals] found {} new pending proposals",
                pending_proposals.proposal_info.len()
            );
            pending_proposals.proposal_info.sort_by(|a, b| {
                a.deadline_timestamp_seconds
                    .cmp(&b.deadline_timestamp_seconds)
            });

            for proposal_info in pending_proposals.proposal_info {
                let proposal_id = match proposal_info.id {
                    Some(proposal_id) => proposal_id,
                    None => {
                        log!(INFO, "[mirror_proposals] bug: found a proposal without id",);
                        continue;
                    }
                };

                // Skip proposal ineligible for rewards.
                // https://github.com/dfinity/ic/blob/17df8febdb922c3981475035d830f09d9b990a5a/rs/nns/governance/src/gen/ic_nns_governance.pb.v1.rs#L4127
                if proposal_info.reward_status > 1 {
                    continue;
                }

                let sns_proposal = match convert_nns_proposal_to_sns_proposal(&proposal_info) {
                    Some(sns_proposal) => sns_proposal,
                    None => {
                        log!(
                            INFO,
                            "[mirror_proposals] failed to convert {proposal_info:?}"
                        );
                        continue;
                    }
                };
                match manage_neuron_sns(subaccount.clone(), CommandSns::MakeProposal(sns_proposal))
                    .await
                {
                    Ok(manage_neuron_response) => {
                        if let Some(CommandSnsResponse::MakeProposal(make_proposal_response)) =
                            manage_neuron_response.command.clone()
                        {
                            if let Some(sns_proposal_id) = make_proposal_response.proposal_id {
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
            return Err(format!(
                "Failed to get pending proposals with error: {error}"
            ));
        }
    }
    Ok(())
}

pub async fn vote_on_nns_proposals() {
    let wtn_governance_id = read_state(|s| s.wtn_governance_id);

    let list_proposals_args = ListProposalInfo {
        limit: BATCH_SIZE_LIMIT,
        before_proposal: None,
        exclude_topic: vec![],
        include_reward_status: vec![REWARD_STATUS_ACCEPT_VOTES, REWARD_STATUS_READY_TO_SETTLE],
        omit_large_fields: Some(true),
        ..Default::default()
    };

    match list_proposals(list_proposals_args).await {
        Ok(mut pending_proposals) => {
            log!(
                INFO,
                "[vote_on_nns_proposals] found {} pending proposals",
                pending_proposals.proposal_info.len()
            );
            pending_proposals.proposal_info.sort_by(|a, b| {
                a.deadline_timestamp_seconds
                    .unwrap_or(u64::MAX)
                    .cmp(&b.deadline_timestamp_seconds.unwrap_or(u64::MAX))
            });
            for proposal in pending_proposals.proposal_info {
                let deadline_timestamp_seconds = proposal.deadline_timestamp_seconds.unwrap_or(0);
                let proposal_id = match proposal.id {
                    Some(proposal_id) => proposal_id,
                    None => continue,
                };
                let proposal_id = ProposalId { id: proposal_id.id };
                let diff_secs = deadline_timestamp_seconds
                    .saturating_sub(timestamp_nanos() / SEC_NANOS)
                    .saturating_sub(ONE_HOUR_SECONDS);
                if diff_secs == 0 {
                    if let Some(sns_proposal_id) =
                        read_state(|s| s.proposals.get(&proposal_id).cloned())
                    {
                        match get_sns_proposal(wtn_governance_id, sns_proposal_id.id).await {
                            Ok(proposal_response) => {
                                if let Some(ic_sns_governance_api::pb::v1::get_proposal_response::Result::Proposal(proposal_data)) =
                                    proposal_response.result.clone()
                                {
                                    if let Some(tally) = proposal_data.latest_tally {
                                        let vote_outcome = tally.yes > tally.no;
                                        vote_on_proposal(proposal_id, vote_outcome).await;
                                        continue;
                                    }
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
                    // We didn't manage to fetch the SNS proposal's outcome
                    // we vote no by default.
                    vote_on_proposal(proposal_id, false).await;
                }
            }
        }
        Err(error) => {
            log!(
                INFO,
                "[vote_on_nns_proposals] Failed to get pending proposals with error: {error}"
            );
            schedule_after(RETRY_DELAY_VOTING, TaskType::ProcessVoting);
        }
    }
}

async fn vote_on_proposal(proposal_id: ProposalId, vote: bool) {
    if read_state(|s| s.voted_proposals.contains(&proposal_id)) {
        log!(
            INFO,
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
            .keys()
            .filter(|&k| !s.voted_proposals.contains(k))
            .cloned()
            .collect()
    });

    for proposal_id in not_voted {
        match get_sns_proposal(wtn_governance_id, proposal_id.id).await {
            Ok(proposal_response) => {
                if let Some(
                    ic_sns_governance_api::pb::v1::get_proposal_response::Result::Proposal(
                        proposal_data,
                    ),
                ) = proposal_response.result.clone()
                {
                    if let Some(tally) = proposal_data.latest_tally {
                        if tally.no > tally.total / 2 {
                            vote_on_proposal(proposal_id.clone(), false).await;
                        }
                        if tally.yes > tally.total / 2 {
                            vote_on_proposal(proposal_id.clone(), true).await;
                        }
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
